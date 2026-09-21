#![cfg(unix)]

mod common;

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use flow::{
    AuthorizedProcess, ExecutionError, ExtensionEvent, ExtensionInvocation, ExtensionPort,
    ExtensionResult, HermeticExtension, LocalProcessRunner, NoSecrets, PortIdentity,
    ProcessIsolation, ProcessProtocolError, ProcessRunnerError, ResolvedExtension, SecretValue,
    authorize_process, observe_execution_subjects,
};

use common::{
    invocation, manifest, process_authority_profile, process_enforcement_evidence,
    process_runner_fixture, process_runner_fixture_with_limits,
};

const TRANSCRIPT_ENVIRONMENT: &str = "FLOW_PROVIDER_STDOUT";
const TRANSCRIPT_HANDLE: &str = "secret:env.flow_provider_stdout";
const LAUNCH_MARKER: &str = "runner-launched";
const READY_MARKER: &str = "runner-ready";
const GRACEFUL_MARKER: &str = "runner-graceful";
const PID_RECORD: &str = "runner.pid";
const RUNNER_ARG_ONE: &str = "literal;$(exit 97)";
const RUNNER_ARG_TWO: &str = "value with spaces";

const PROVIDER_SCRIPT: &str = r#"#!/bin/sh
set -eu
: > runner-launched
[ -f "./flow synthetic adapter" ] || exit 81
[ "${HOME+x}" != x ] || exit 82
[ "$#" -eq 2 ] || exit 83
[ "$1" = 'literal;$(exit 97)' ] || exit 84
[ "$2" = 'value with spaces' ] || exit 85
[ "${FLOW_PROVIDER_STDOUT+x}" = x ] || exit 86
IFS= read -r request || exit 87
case "$request" in
  *'"schema_version":"flow.extension-invocation/v1"'*) ;;
  *) exit 88 ;;
esac
if IFS= read -r extra; then
  exit 89
fi
printf '%s' "$FLOW_PROVIDER_STDOUT"
printf '%s' 'bounded operational diagnostic' >&2
"#;

const NONZERO_PROVIDER_SCRIPT: &str = r#"#!/bin/sh
set -eu
: > runner-launched
IFS= read -r request || exit 87
printf '%s' "$FLOW_PROVIDER_STDOUT"
exit 7
"#;

const INTERRUPTIBLE_PROVIDER_SCRIPT: &str = r#"#!/bin/sh
set -eu
trap ': > runner-graceful; exit 0' TERM
printf '%s\n' "$$" > runner.pid
: > runner-ready
while :; do :; done
"#;

const UNCOOPERATIVE_PROVIDER_SCRIPT: &str = r#"#!/bin/sh
set -eu
trap '' TERM
printf '%s\n' "$$" > runner.pid
: > runner-ready
while :; do :; done
"#;

const STDOUT_OVERFLOW_PROVIDER_SCRIPT: &str = r#"#!/bin/sh
set -eu
IFS= read -r request || exit 87
i=0
while [ "$i" -lt 2000 ]; do
  printf '%s' '0123456789'
  i=$((i + 1))
done
"#;

const STDERR_OVERFLOW_PROVIDER_SCRIPT: &str = r#"#!/bin/sh
set -eu
IFS= read -r request || exit 87
i=0
while [ "$i" -lt 2000 ]; do
  printf '%s' '0123456789' >&2
  i=$((i + 1))
done
"#;

fn runner_permissions() -> (flow::Permissions, flow::Permissions) {
    let mut requested = manifest().requested_permissions;
    requested.environment_read = vec![TRANSCRIPT_ENVIRONMENT.to_owned()];
    let granted = requested.clone();
    (requested, granted)
}

fn runner_limits() -> flow::ExecutionLimits {
    manifest()
        .execution_modes
        .into_iter()
        .find(|mode| mode.name == "synthetic-process")
        .expect("process execution mode must exist")
        .limits
}

fn provider_evidence(
    resolved: &ResolvedExtension,
    invocation: &ExtensionInvocation,
) -> (Vec<ExtensionEvent>, ExtensionResult, String) {
    let provider = HermeticExtension::new(PortIdentity::from_resolved(resolved));
    let mut events = Vec::new();
    let result = provider
        .invoke(invocation, &mut events)
        .expect("hermetic provider evidence must be available");
    let mut stdout = Vec::new();
    for event in &events {
        stdout.extend(serde_json::to_vec(event).expect("event fixture must serialize"));
        stdout.push(b'\n');
    }
    stdout.extend(serde_json::to_vec(&result).expect("result fixture must serialize"));
    stdout.push(b'\n');
    let stdout = String::from_utf8(stdout).expect("fixture transcript must be UTF-8");
    (events, result, stdout)
}

fn runner_authority(
    resolved: &ResolvedExtension,
    invocation: &ExtensionInvocation,
    lock: &flow::ExecutionSubjectLock,
    subjects: &flow::MatchedExecutionSubjects,
    isolation: ProcessIsolation,
) -> AuthorizedProcess {
    let mut profile = process_authority_profile(resolved, invocation, lock, subjects, isolation);
    profile.requested.argv = vec![RUNNER_ARG_ONE.to_owned(), RUNNER_ARG_TWO.to_owned()];
    profile.granted.argv.clone_from(&profile.requested.argv);
    let evidence = process_enforcement_evidence(&profile, subjects);
    authorize_process(resolved, invocation, lock, subjects, &profile, &evidence)
        .expect("runner authority fixture must validate")
}

fn transcript_secret(transcript: String) -> impl Fn(&str) -> Option<SecretValue> {
    move |handle: &str| (handle == TRANSCRIPT_HANDLE).then(|| SecretValue::new(transcript.clone()))
}

fn assert_recorded_process_reaped(package: &Path) {
    use nix::errno::Errno;
    use nix::sys::wait::{WaitPidFlag, waitpid};
    use nix::unistd::Pid;

    let pid = std::fs::read_to_string(package.join(PID_RECORD))
        .unwrap()
        .trim()
        .parse::<i32>()
        .unwrap();
    assert_eq!(
        waitpid(Pid::from_raw(pid), Some(WaitPidFlag::WNOHANG)),
        Err(Errno::ECHILD)
    );
}

#[test]
fn runner_launches_exact_executable_with_scrubbed_process_state() {
    let (requested, granted) = runner_permissions();
    let fixture = process_runner_fixture(PROVIDER_SCRIPT.as_bytes(), requested, granted);
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let mut invocation = invocation(resolved);
    invocation.secret_handles = vec![TRANSCRIPT_HANDLE.to_owned()];
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = runner_authority(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    let (events, result, transcript) = provider_evidence(resolved, &invocation);
    let secrets =
        |handle: &str| (handle == TRANSCRIPT_HANDLE).then(|| SecretValue::new(transcript.clone()));
    let mut observed = Vec::new();

    let validated = LocalProcessRunner::run(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
        &authority,
        &secrets,
        &mut observed,
    )
    .unwrap();

    assert_eq!(validated.events(), events);
    assert_eq!(validated.result(), &result);
    assert_eq!(observed, events);
    assert!(fixture.root.package_path().join(LAUNCH_MARKER).is_file());
}

#[test]
fn runner_resolves_every_authorized_secret_before_launch() {
    let (requested, granted) = runner_permissions();
    let fixture = process_runner_fixture(PROVIDER_SCRIPT.as_bytes(), requested, granted);
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let mut invocation = invocation(resolved);
    invocation.secret_handles = vec![TRANSCRIPT_HANDLE.to_owned()];
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = runner_authority(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );

    let error = LocalProcessRunner::run(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
        &authority,
        &NoSecrets,
        &mut Vec::new(),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ProcessRunnerError::MissingSecret { ref name } if name == TRANSCRIPT_ENVIRONMENT
    ));
    assert!(!fixture.root.package_path().join(LAUNCH_MARKER).exists());
}

#[test]
fn runner_reobserves_subjects_and_rejects_altered_bytes_before_launch() {
    let (requested, granted) = runner_permissions();
    let fixture = process_runner_fixture(PROVIDER_SCRIPT.as_bytes(), requested, granted);
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let mut invocation = invocation(resolved);
    invocation.secret_handles = vec![TRANSCRIPT_HANDLE.to_owned()];
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = runner_authority(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    OpenOptions::new()
        .append(true)
        .open(fixture.root.executable_path())
        .unwrap()
        .write_all(b"\n# altered after authorization\n")
        .unwrap();

    let error = LocalProcessRunner::run(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
        &authority,
        &NoSecrets,
        &mut Vec::new(),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ProcessRunnerError::SubjectObservation { .. }
    ));
    assert!(!fixture.root.package_path().join(LAUNCH_MARKER).exists());
}

#[test]
fn runner_refuses_sandbox_claims_without_an_enforcing_backend() {
    let (requested, granted) = runner_permissions();
    let fixture = process_runner_fixture(PROVIDER_SCRIPT.as_bytes(), requested, granted);
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let mut invocation = invocation(resolved);
    invocation.secret_handles = vec![TRANSCRIPT_HANDLE.to_owned()];
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = runner_authority(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::Sandboxed,
    );

    let error = LocalProcessRunner::run(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
        &authority,
        &NoSecrets,
        &mut Vec::new(),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ProcessRunnerError::UnsupportedIsolation {
            isolation: ProcessIsolation::Sandboxed
        }
    ));
    assert!(!fixture.root.package_path().join(LAUNCH_MARKER).exists());
}

#[test]
fn runner_keeps_process_exit_separate_from_provider_success() {
    let (requested, granted) = runner_permissions();
    let fixture = process_runner_fixture(NONZERO_PROVIDER_SCRIPT.as_bytes(), requested, granted);
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let mut invocation = invocation(resolved);
    invocation.secret_handles = vec![TRANSCRIPT_HANDLE.to_owned()];
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = runner_authority(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    let (_, _, transcript) = provider_evidence(resolved, &invocation);
    let secrets =
        |handle: &str| (handle == TRANSCRIPT_HANDLE).then(|| SecretValue::new(transcript.clone()));

    let error = LocalProcessRunner::run(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
        &authority,
        &secrets,
        &mut Vec::new(),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ProcessRunnerError::Validation {
            source: ExecutionError::ProcessExit { code: Some(7) }
        }
    ));
    assert!(fixture.root.package_path().join(LAUNCH_MARKER).is_file());
}

#[test]
fn malformed_provider_bytes_remain_untrusted_after_a_real_launch() {
    let (requested, granted) = runner_permissions();
    let fixture = process_runner_fixture(PROVIDER_SCRIPT.as_bytes(), requested, granted);
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let mut invocation = invocation(resolved);
    invocation.secret_handles = vec![TRANSCRIPT_HANDLE.to_owned()];
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = runner_authority(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    let secrets =
        |handle: &str| (handle == TRANSCRIPT_HANDLE).then(|| SecretValue::new("not-json\n"));

    let error = LocalProcessRunner::run(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
        &authority,
        &secrets,
        &mut Vec::new(),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ProcessRunnerError::Validation {
            source: ExecutionError::ProcessProtocol {
                source: ProcessProtocolError::MalformedJson { .. }
            }
        }
    ));
}

#[test]
fn secret_debug_output_is_always_redacted() {
    let secret = SecretValue::new("do-not-print-me");
    assert_eq!(format!("{secret:?}"), "SecretValue(<redacted>)");
}

#[test]
fn runner_cancels_after_a_deterministic_ready_signal_and_reaps() {
    let (requested, granted) = runner_permissions();
    let mut limits = runner_limits();
    limits.timeout_ms = 5_000;
    limits.cancellation_grace_ms = 500;
    let fixture = process_runner_fixture_with_limits(
        INTERRUPTIBLE_PROVIDER_SCRIPT.as_bytes(),
        requested,
        granted,
        limits,
    );
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let mut invocation = invocation(resolved);
    invocation.secret_handles = vec![TRANSCRIPT_HANDLE.to_owned()];
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = runner_authority(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    let (_, _, transcript) = provider_evidence(resolved, &invocation);
    let secrets = transcript_secret(transcript);
    let package = fixture.root.package_path();
    let cancellation_marker = package.join(READY_MARKER);
    let cancellation = || cancellation_marker.is_file();

    let error = LocalProcessRunner::run_with_cancellation(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
        &authority,
        &secrets,
        &cancellation,
        &mut Vec::new(),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ProcessRunnerError::Cancelled { forced: false }
    ));
    assert!(package.join(GRACEFUL_MARKER).is_file());
    assert_recorded_process_reaped(&package);
}

#[test]
fn runner_enforces_the_invocation_deadline_and_reaps() {
    let (requested, granted) = runner_permissions();
    let mut limits = runner_limits();
    limits.timeout_ms = 250;
    limits.cancellation_grace_ms = 500;
    let fixture = process_runner_fixture_with_limits(
        INTERRUPTIBLE_PROVIDER_SCRIPT.as_bytes(),
        requested,
        granted,
        limits,
    );
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let mut invocation = invocation(resolved);
    invocation.secret_handles = vec![TRANSCRIPT_HANDLE.to_owned()];
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = runner_authority(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    let (_, _, transcript) = provider_evidence(resolved, &invocation);
    let secrets = transcript_secret(transcript);

    let error = LocalProcessRunner::run(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
        &authority,
        &secrets,
        &mut Vec::new(),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ProcessRunnerError::TimedOut {
            timeout_ms: 250,
            forced: false
        }
    ));
    let package = fixture.root.package_path();
    assert!(package.join(READY_MARKER).is_file());
    assert!(package.join(GRACEFUL_MARKER).is_file());
    assert_recorded_process_reaped(&package);
}

#[test]
fn runner_deadline_covers_a_provider_that_never_reads_stdin() {
    let (requested, granted) = runner_permissions();
    let mut limits = runner_limits();
    limits.timeout_ms = 250;
    limits.cancellation_grace_ms = 500;
    let fixture = process_runner_fixture_with_limits(
        INTERRUPTIBLE_PROVIDER_SCRIPT.as_bytes(),
        requested,
        granted,
        limits,
    );
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let mut invocation = invocation(resolved);
    invocation.secret_handles = vec![TRANSCRIPT_HANDLE.to_owned()];
    invocation.configuration.values.insert(
        "large-blocked-request".to_owned(),
        serde_json::Value::String("x".repeat(1_000_000)),
    );
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = runner_authority(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    let secrets = transcript_secret(String::new());

    let error = LocalProcessRunner::run(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
        &authority,
        &secrets,
        &mut Vec::new(),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ProcessRunnerError::TimedOut {
            timeout_ms: 250,
            forced: false
        }
    ));
    let package = fixture.root.package_path();
    assert!(package.join(GRACEFUL_MARKER).is_file());
    assert_recorded_process_reaped(&package);
}

#[test]
fn runner_force_kills_and_reaps_after_the_declared_grace() {
    let (requested, granted) = runner_permissions();
    let mut limits = runner_limits();
    limits.timeout_ms = 5_000;
    limits.cancellation_grace_ms = 25;
    let fixture = process_runner_fixture_with_limits(
        UNCOOPERATIVE_PROVIDER_SCRIPT.as_bytes(),
        requested,
        granted,
        limits,
    );
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let mut invocation = invocation(resolved);
    invocation.secret_handles = vec![TRANSCRIPT_HANDLE.to_owned()];
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = runner_authority(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    let (_, _, transcript) = provider_evidence(resolved, &invocation);
    let secrets = transcript_secret(transcript);
    let package = fixture.root.package_path();
    let cancellation_marker = package.join(READY_MARKER);
    let cancellation = || cancellation_marker.is_file();

    let error = LocalProcessRunner::run_with_cancellation(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
        &authority,
        &secrets,
        &cancellation,
        &mut Vec::new(),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ProcessRunnerError::Cancelled { forced: true }
    ));
    assert!(!package.join(GRACEFUL_MARKER).exists());
    assert_recorded_process_reaped(&package);
}

#[test]
fn runner_drains_stdout_but_retains_only_the_limit_plus_one_byte() {
    let (requested, granted) = runner_permissions();
    let mut limits = runner_limits();
    limits.max_stdout_bytes = 8;
    let fixture = process_runner_fixture_with_limits(
        STDOUT_OVERFLOW_PROVIDER_SCRIPT.as_bytes(),
        requested,
        granted,
        limits,
    );
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let mut invocation = invocation(resolved);
    invocation.secret_handles = vec![TRANSCRIPT_HANDLE.to_owned()];
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = runner_authority(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    let secrets = transcript_secret(String::new());

    let error = LocalProcessRunner::run(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
        &authority,
        &secrets,
        &mut Vec::new(),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ProcessRunnerError::Validation {
            source: ExecutionError::ProcessOutputLimit {
                stream: flow::ProcessStream::Stdout,
                limit: 8,
                observed: 9
            }
        }
    ));
}

#[test]
fn runner_keeps_stderr_separate_and_bounded() {
    let (requested, granted) = runner_permissions();
    let mut limits = runner_limits();
    limits.max_stderr_bytes = 8;
    let fixture = process_runner_fixture_with_limits(
        STDERR_OVERFLOW_PROVIDER_SCRIPT.as_bytes(),
        requested,
        granted,
        limits,
    );
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let mut invocation = invocation(resolved);
    invocation.secret_handles = vec![TRANSCRIPT_HANDLE.to_owned()];
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = runner_authority(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    let secrets = transcript_secret(String::new());

    let error = LocalProcessRunner::run(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
        &authority,
        &secrets,
        &mut Vec::new(),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ProcessRunnerError::Validation {
            source: ExecutionError::ProcessOutputLimit {
                stream: flow::ProcessStream::Stderr,
                limit: 8,
                observed: 9
            }
        }
    ));
}
