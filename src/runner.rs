//! Direct, bounded launching for one pre-authorized provider process.
//!
//! This module is intentionally smaller than the final FLO-3.2e lifecycle.
//! It proves the exact trusted-unconfined launch path, scrubbed environment,
//! independent stream capture, normal wait/reap behavior, and reuse of Flow's
//! existing transcript validator. Timeout and cancellation enforcement remain
//! explicit follow-up work before the runner is complete.

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};
use std::thread;

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
    /// captures stdout/stderr independently with bounded retained memory, and
    /// waits for the child before calling Flow's existing transcript validator.
    ///
    /// This checkpoint accepts only `trusted-unconfined`. It does not yet
    /// enforce timeout or cancellation, and it does not claim sandboxing or a
    /// race-free binding between the last observation and the host `exec`.
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
        let stdout_worker = thread::spawn(move || read_bounded(stdout, stdout_limit));
        let stderr_worker = thread::spawn(move || read_bounded(stderr, stderr_limit));

        if let Err(source) = stdin.write_all(&request) {
            terminate_and_reap(&mut child);
            let _ = stdout_worker.join();
            let _ = stderr_worker.join();
            return Err(ProcessRunnerError::Stdin { source });
        }
        drop(stdin);

        let status = match child.wait() {
            Ok(status) => status,
            Err(source) => {
                terminate_and_reap(&mut child);
                let _ = stdout_worker.join();
                let _ = stderr_worker.join();
                return Err(ProcessRunnerError::Wait { source });
            }
        };
        let stdout = join_capture(stdout_worker, ProcessStream::Stdout, ProcessWorker::Stdout);
        let stderr = join_capture(stderr_worker, ProcessStream::Stderr, ProcessWorker::Stderr);
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
    let _ = child.kill();
    let _ = child.wait();
}
