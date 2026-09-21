//! Direct, bounded launching for one pre-authorized provider process.
//!
//! The runner owns the exact trusted-unconfined launch path, scrubbed
//! environment, independent bounded stream capture, deadline and caller
//! cancellation control, child termination/reaping, and reuse of Flow's
//! existing transcript validator. It deliberately does not claim sandbox or
//! descendant-process containment.

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use thiserror::Error;

use crate::ResolvedExtension;
use crate::authority::{AuthorizedProcess, ProcessIsolation};
use crate::contracts::ExtensionInvocation;
use crate::execution::{
    EventSink, ExecutionError, Orchestrator, ProcessCompletion, ProcessStream, ProcessTranscript,
    ValidatedExecution,
};
use crate::execution_subjects::{
    ExecutionSubjectError, ExecutionSubjectLock, observe_execution_subjects,
};

/// A secret value resolved only for the child environment.
///
/// Its debug representation is always redacted, and Flow exposes no accessor
/// that converts the value back into portable evidence.
#[derive(Eq, PartialEq)]
pub struct SecretValue(OsString);

impl SecretValue {
    #[must_use]
    pub fn new(value: impl Into<OsString>) -> Self {
        Self(value.into())
    }

    fn as_os_str(&self) -> &OsStr {
        &self.0
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretValue(<redacted>)")
    }
}

/// Caller-owned resolution for one opaque secret handle.
///
/// Returning `None` means the handle is unavailable. The runner resolves only
/// handles named by the already-authorized process profile.
pub trait SecretResolver {
    fn resolve(&self, source_handle: &str) -> Option<SecretValue>;
}

/// Caller-owned cancellation observation for one running child.
///
/// The runner polls this signal after checking whether the child has already
/// completed. Returning `true` is monotonic for the lifetime of one call: once
/// cancellation is observed, Flow begins termination and never falls back to
/// another provider. Implementations must return promptly so they do not stall
/// deadline observation.
pub trait CancellationSignal {
    fn is_cancelled(&self) -> bool;
}

impl<F> CancellationSignal for F
where
    F: Fn() -> bool,
{
    fn is_cancelled(&self) -> bool {
        self()
    }
}

/// Cancellation signal that never requests interruption.
#[derive(Clone, Copy, Debug, Default)]
pub struct NeverCancelled;

impl CancellationSignal for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }
}

impl<F> SecretResolver for F
where
    F: Fn(&str) -> Option<SecretValue>,
{
    fn resolve(&self, source_handle: &str) -> Option<SecretValue> {
        self(source_handle)
    }
}

/// Resolver for invocations that authorize no environment bindings.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoSecrets;

impl SecretResolver for NoSecrets {
    fn resolve(&self, _source_handle: &str) -> Option<SecretValue> {
        None
    }
}

/// Failure while preparing, launching, capturing, or validating one child.
#[derive(Debug, Error)]
pub enum ProcessRunnerError {
    #[error("fresh execution-subject observation failed: {source}")]
    SubjectObservation {
        #[source]
        source: ExecutionSubjectError,
    },
    #[error("process request preflight failed: {source}")]
    RequestPreflight {
        #[source]
        source: ExecutionError,
    },
    #[error("the local runner does not implement the selected isolation profile: {isolation:?}")]
    UnsupportedIsolation { isolation: ProcessIsolation },
    #[error("authorized environment binding {name} could not resolve its opaque handle")]
    MissingSecret { name: String },
    #[error("failed to resolve the selected execution root: {source}")]
    ExecutionRoot {
        #[source]
        source: io::Error,
    },
    #[error("failed to launch the exact locked executable: {source}")]
    Launch {
        #[source]
        source: io::Error,
    },
    #[error("the launched child did not expose its configured {stream:?} pipe")]
    MissingPipe { stream: ProcessPipe },
    #[error("failed to write the invocation request to child stdin: {source}")]
    Stdin {
        #[source]
        source: io::Error,
    },
    #[error("failed to wait for the launched child: {source}")]
    Wait {
        #[source]
        source: io::Error,
    },
    #[error("failed to request graceful child termination: {source}")]
    GracefulTermination {
        #[source]
        source: io::Error,
    },
    #[error("failed to force child termination: {source}")]
    ForceTermination {
        #[source]
        source: io::Error,
    },
    #[error("the child exceeded its {timeout_ms} ms deadline (forced termination: {forced})")]
    TimedOut { timeout_ms: u64, forced: bool },
    #[error("the caller cancelled the child (forced termination: {forced})")]
    Cancelled { forced: bool },
    #[error("failed to capture child {stream:?}: {source}")]
    Capture {
        stream: ProcessStream,
        #[source]
        source: io::Error,
    },
    #[error("the child {task:?} worker terminated unexpectedly")]
    Worker { task: ProcessWorker },
    #[error("captured child evidence failed Flow validation: {source}")]
    Validation {
        #[source]
        source: ExecutionError,
    },
}

/// A pipe required by the local child lifecycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessPipe {
    Stdin,
    Stdout,
    Stderr,
}

/// A worker used to drain one child stream concurrently.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessWorker {
    Stdin,
    Stdout,
    Stderr,
}

/// Stateless local runner for one exact trusted-unconfined process.
#[derive(Clone, Copy, Debug, Default)]
pub struct LocalProcessRunner;

impl LocalProcessRunner {
    /// Launch one exact locked executable and validate its completed transcript.
    ///
    /// The runner re-observes the package and executable immediately before
    /// request encoding, invokes the absolute locked path directly, supplies
    /// only authorized argv and resolved environment bindings, closes stdin,
    /// captures stdout/stderr independently with bounded retained memory,
    /// enforces the invocation deadline, and reaps the child before calling
    /// Flow's existing transcript validator.
    ///
    /// This convenience entry point never requests caller cancellation. Use
    /// [`Self::run_with_cancellation`] to supply an explicit signal. The runner
    /// accepts only `trusted-unconfined` and does not claim sandboxing,
    /// descendant-process containment, or a race-free binding between the last
    /// observation and the host `exec`.
    ///
    /// # Errors
    ///
    /// Returns a typed error for subject drift, preflight mismatch, unsupported
    /// isolation, unresolved secrets, launch or pipe I/O, wait/reap failure,
    /// capture failure, or rejected transcript evidence.
    pub fn run(
        root: &Path,
        resolved: &ResolvedExtension,
        invocation: &ExtensionInvocation,
        subject_lock: &ExecutionSubjectLock,
        authority: &AuthorizedProcess,
        secrets: &dyn SecretResolver,
        event_sink: &mut dyn EventSink,
    ) -> Result<ValidatedExecution, ProcessRunnerError> {
        Self::run_with_cancellation(
            root,
            resolved,
            invocation,
            subject_lock,
            authority,
            secrets,
            &NeverCancelled,
            event_sink,
        )
    }

    /// Launch, supervise, and validate one exact locked executable.
    ///
    /// Normal completion observed by the supervisor wins over a concurrent
    /// cancellation request. Otherwise caller cancellation wins over the
    /// deadline when both become observable in the same polling cycle. On
    /// Unix, interruption requests `SIGTERM`, waits the declared cancellation
    /// grace, then uses forced termination if the child remains alive. Other
    /// hosts use immediate forced termination because Rust's standard process
    /// API exposes no portable graceful signal.
    ///
    /// Every normally completed, cancelled, or timed-out direct child is waited
    /// and reaped before this method returns. Cleanup failures are reported as
    /// typed errors. The control applies only to that direct child; an
    /// unconfined provider may create descendants outside this lifecycle.
    ///
    /// # Errors
    ///
    /// Returns [`ProcessRunnerError::Cancelled`] or
    /// [`ProcessRunnerError::TimedOut`] only after the direct child is reaped.
    /// Other typed errors cover preflight, launch, capture, cleanup, or
    /// transcript-validation failure.
    #[allow(clippy::too_many_arguments)]
    pub fn run_with_cancellation(
        root: &Path,
        resolved: &ResolvedExtension,
        invocation: &ExtensionInvocation,
        subject_lock: &ExecutionSubjectLock,
        authority: &AuthorizedProcess,
        secrets: &dyn SecretResolver,
        cancellation: &dyn CancellationSignal,
        event_sink: &mut dyn EventSink,
    ) -> Result<ValidatedExecution, ProcessRunnerError> {
        let launch_subjects = observe_execution_subjects(root, resolved, invocation, subject_lock)
            .map_err(|source| ProcessRunnerError::SubjectObservation { source })?;
        let request = Orchestrator::encode_process_request(
            resolved,
            invocation,
            subject_lock,
            &launch_subjects,
            authority,
        )
        .map_err(|source| ProcessRunnerError::RequestPreflight { source })?;

        if authority.isolation() != ProcessIsolation::TrustedUnconfined {
            return Err(ProcessRunnerError::UnsupportedIsolation {
                isolation: authority.isolation(),
            });
        }

        let environment = authority
            .profile()
            .requested
            .environment
            .iter()
            .map(|binding| {
                secrets
                    .resolve(&binding.source_handle)
                    .map(|value| (binding.name.clone(), value))
                    .ok_or_else(|| ProcessRunnerError::MissingSecret {
                        name: binding.name.clone(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let canonical_root = fs::canonicalize(root)
            .map_err(|source| ProcessRunnerError::ExecutionRoot { source })?;
        let package = canonical_root.join(&subject_lock.package.locator);
        let executable = package.join(&subject_lock.executable.locator);

        let mut command = Command::new(executable);
        command
            .args(&authority.profile().requested.argv)
            .current_dir(package)
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (name, value) in &environment {
            command.env(name, value.as_os_str());
        }

        let mut child = command
            .spawn()
            .map_err(|source| ProcessRunnerError::Launch { source })?;
        let (mut stdin, stdout, stderr) = match take_pipes(&mut child) {
            Ok(pipes) => pipes,
            Err(error) => {
                terminate_and_reap(&mut child);
                return Err(error);
            }
        };

        let stdout_limit = invocation.limits.max_stdout_bytes;
        let stderr_limit = invocation.limits.max_stderr_bytes;
        let mut stdin_worker = Some(thread::spawn(move || stdin.write_all(&request)));
        let stdout_worker = thread::spawn(move || read_bounded(stdout, stdout_limit));
        let stderr_worker = thread::spawn(move || read_bounded(stderr, stderr_limit));

        let completion = supervise_child(
            &mut child,
            &mut stdin_worker,
            cancellation,
            invocation.limits.timeout_ms,
            invocation.limits.cancellation_grace_ms,
        );
        let stdin = join_stdin_worker(&mut stdin_worker);
        let stdout = join_capture(stdout_worker, ProcessStream::Stdout, ProcessWorker::Stdout);
        let stderr = join_capture(stderr_worker, ProcessStream::Stderr, ProcessWorker::Stderr);
        let completion = completion?;

        let status = completed_status(completion, invocation.limits.timeout_ms)?;
        stdin?;
        let stdout = stdout?;
        let stderr = stderr?;

        Orchestrator::validate_process_transcript(
            resolved,
            invocation,
            subject_lock,
            &launch_subjects,
            authority,
            ProcessTranscript::new(
                ProcessCompletion::Exited {
                    code: status.code(),
                },
                &stdout,
                &stderr,
            ),
            event_sink,
        )
        .map_err(|source| ProcessRunnerError::Validation { source })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Supervision {
    Exited(ExitStatus),
    TimedOut { forced: bool },
    Cancelled { forced: bool },
}

fn completed_status(
    completion: Supervision,
    timeout_ms: u64,
) -> Result<ExitStatus, ProcessRunnerError> {
    match completion {
        Supervision::Exited(status) => Ok(status),
        Supervision::TimedOut { forced } => {
            Err(ProcessRunnerError::TimedOut { timeout_ms, forced })
        }
        Supervision::Cancelled { forced } => Err(ProcessRunnerError::Cancelled { forced }),
    }
}

fn supervise_child(
    child: &mut Child,
    stdin_worker: &mut Option<thread::JoinHandle<io::Result<()>>>,
    cancellation: &dyn CancellationSignal,
    timeout_ms: u64,
    cancellation_grace_ms: u64,
) -> Result<Supervision, ProcessRunnerError> {
    let started = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);

    loop {
        if stdin_worker
            .as_ref()
            .is_some_and(thread::JoinHandle::is_finished)
        {
            if let Err(error) = join_stdin_worker(stdin_worker) {
                terminate_and_reap(child);
                return Err(error);
            }
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                child
                    .wait()
                    .map_err(|source| ProcessRunnerError::Wait { source })?;
                return Ok(Supervision::Exited(status));
            }
            Ok(None) => {}
            Err(source) => {
                terminate_and_reap(child);
                return Err(ProcessRunnerError::Wait { source });
            }
        }
        if cancellation.is_cancelled() {
            let forced = interrupt_and_reap(child, cancellation_grace_ms)?;
            return Ok(Supervision::Cancelled { forced });
        }
        if started.elapsed() >= timeout {
            let forced = interrupt_and_reap(child, cancellation_grace_ms)?;
            return Ok(Supervision::TimedOut { forced });
        }
        thread::sleep(Duration::from_millis(1));
    }
}

fn join_stdin_worker(
    worker: &mut Option<thread::JoinHandle<io::Result<()>>>,
) -> Result<(), ProcessRunnerError> {
    let Some(worker) = worker.take() else {
        return Ok(());
    };
    worker
        .join()
        .map_err(|_| ProcessRunnerError::Worker {
            task: ProcessWorker::Stdin,
        })?
        .map_err(|source| ProcessRunnerError::Stdin { source })
}

fn interrupt_and_reap(
    child: &mut Child,
    cancellation_grace_ms: u64,
) -> Result<bool, ProcessRunnerError> {
    #[cfg(unix)]
    {
        if let Err(error) = request_graceful_termination(child) {
            terminate_and_reap(child);
            return Err(error);
        }
        let grace_started = Instant::now();
        let grace = Duration::from_millis(cancellation_grace_ms);
        loop {
            match child.try_wait() {
                Ok(Some(_)) => {
                    child
                        .wait()
                        .map_err(|source| ProcessRunnerError::Wait { source })?;
                    return Ok(false);
                }
                Ok(None) => {}
                Err(source) => {
                    terminate_and_reap(child);
                    return Err(ProcessRunnerError::Wait { source });
                }
            }
            if grace_started.elapsed() >= grace {
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
    }

    force_terminate_and_reap(child)?;
    Ok(true)
}

#[cfg(unix)]
fn request_graceful_termination(child: &Child) -> Result<(), ProcessRunnerError> {
    use nix::sys::signal::{Signal, kill};
    use nix::unistd::Pid;

    let pid = i32::try_from(child.id()).map_err(|_| ProcessRunnerError::GracefulTermination {
        source: io::Error::new(io::ErrorKind::InvalidInput, "child PID exceeds i32"),
    })?;
    kill(Pid::from_raw(pid), Signal::SIGTERM).map_err(|error| {
        ProcessRunnerError::GracefulTermination {
            source: io::Error::from_raw_os_error(error as i32),
        }
    })
}

fn force_terminate_and_reap(child: &mut Child) -> Result<(), ProcessRunnerError> {
    match child.kill() {
        Ok(()) => {}
        Err(source) if source.kind() == io::ErrorKind::InvalidInput => {}
        Err(source) => return Err(ProcessRunnerError::ForceTermination { source }),
    }
    child
        .wait()
        .map_err(|source| ProcessRunnerError::Wait { source })?;
    Ok(())
}

fn take_pipes(
    child: &mut Child,
) -> Result<(ChildStdin, ChildStdout, ChildStderr), ProcessRunnerError> {
    let stdin = child.stdin.take().ok_or(ProcessRunnerError::MissingPipe {
        stream: ProcessPipe::Stdin,
    })?;
    let stdout = child.stdout.take().ok_or(ProcessRunnerError::MissingPipe {
        stream: ProcessPipe::Stdout,
    })?;
    let stderr = child.stderr.take().ok_or(ProcessRunnerError::MissingPipe {
        stream: ProcessPipe::Stderr,
    })?;
    Ok((stdin, stdout, stderr))
}

fn read_bounded(mut stream: impl Read, limit: u64) -> io::Result<Vec<u8>> {
    let retained_limit = limit.saturating_add(1);
    let mut captured = Vec::new();
    let mut chunk = [0_u8; 8192];
    loop {
        let count = stream.read(&mut chunk)?;
        if count == 0 {
            break;
        }
        let captured_len = u64::try_from(captured.len()).unwrap_or(u64::MAX);
        let remaining = retained_limit.saturating_sub(captured_len);
        let count_u64 = u64::try_from(count).unwrap_or(u64::MAX);
        let keep = usize::try_from(remaining.min(count_u64)).unwrap_or(count);
        captured.extend_from_slice(&chunk[..keep]);
    }
    Ok(captured)
}

fn join_capture(
    worker: thread::JoinHandle<io::Result<Vec<u8>>>,
    stream: ProcessStream,
    task: ProcessWorker,
) -> Result<Vec<u8>, ProcessRunnerError> {
    worker
        .join()
        .map_err(|_| ProcessRunnerError::Worker { task })?
        .map_err(|source| ProcessRunnerError::Capture { stream, source })
}

fn terminate_and_reap(child: &mut Child) {
    let _ = force_terminate_and_reap(child);
}
