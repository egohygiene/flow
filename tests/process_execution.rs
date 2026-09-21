mod common;

use flow::{
    EventKind, EventSink, EventSinkError, EventState, ExecutionError, ExtensionEvent,
    ExtensionInvocation, ExtensionPort, ExtensionResult, FailureClassification, HermeticExtension,
    Orchestrator, Outcome, PortIdentity, ProcessCompletion, ProcessStream, ProcessTranscript,
    observe_execution_subjects,
};

use common::{authorized_process, invocation, process_subject_fixture, resolved_fixture};

fn provider_evidence(
    resolved: &flow::ResolvedExtension,
    invocation: &ExtensionInvocation,
) -> (Vec<ExtensionEvent>, ExtensionResult) {
    let provider = HermeticExtension::new(PortIdentity::from_resolved(resolved));
    let mut events = Vec::new();
    let result = provider
        .invoke(invocation, &mut events)
        .expect("hermetic provider evidence must be available");
    (events, result)
}

fn stdout(events: &[ExtensionEvent], result: &ExtensionResult) -> Vec<u8> {
    let mut stdout = Vec::new();
    for event in events {
        stdout.extend(serde_json::to_vec(event).expect("event fixture must serialize"));
        stdout.push(b'\n');
    }
    stdout.extend(serde_json::to_vec(result).expect("result fixture must serialize"));
    stdout.push(b'\n');
    stdout
}

#[test]
fn process_request_and_transcript_reuse_flow_owned_validation() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = authorized_process(resolved, &invocation, &fixture.subject_lock, &subjects);
    let (events, result) = provider_evidence(resolved, &invocation);
    let provider_stdout = stdout(&events, &result);
    let request_bytes = Orchestrator::encode_process_request(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &authority,
    )
    .unwrap();
    let mut observed = Vec::new();

    let validated = Orchestrator::validate_process_transcript(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &authority,
        ProcessTranscript::new(
            ProcessCompletion::Exited { code: Some(0) },
            &provider_stdout,
            b"bounded operational diagnostic",
        ),
        &mut observed,
    )
    .unwrap();

    assert_eq!(request_bytes.last(), Some(&b'\n'));
    assert_eq!(validated.events(), events);
    assert_eq!(validated.result(), &result);
    assert_eq!(observed, events);
}

#[test]
fn execution_modes_fail_closed_at_the_wrong_seam() {
    let fixture = process_subject_fixture();
    let process_outcome = fixture.catalog.resolve(&fixture.request);
    let process_resolved = process_outcome.resolved().unwrap();
    let process_invocation = invocation(process_resolved);
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        process_resolved,
        &process_invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = authorized_process(
        process_resolved,
        &process_invocation,
        &fixture.subject_lock,
        &subjects,
    );
    let process_port = HermeticExtension::new(PortIdentity::from_resolved(process_resolved));
    let mut observed = Vec::new();

    let error = Orchestrator::execute(
        process_resolved,
        &process_invocation,
        &process_port,
        &mut observed,
    )
    .unwrap_err();
    assert!(matches!(error, ExecutionError::Preflight { .. }));
    assert!(observed.is_empty());

    let (in_process_catalog, in_process_request) = resolved_fixture();
    let in_process_outcome = in_process_catalog.resolve(&in_process_request);
    let in_process_resolved = in_process_outcome.resolved().unwrap();
    let in_process_invocation = invocation(in_process_resolved);
    let error = Orchestrator::encode_process_request(
        in_process_resolved,
        &in_process_invocation,
        &fixture.subject_lock,
        &subjects,
        &authority,
    )
    .unwrap_err();
    assert!(matches!(error, ExecutionError::Preflight { .. }));
}

#[test]
fn captured_stream_limits_are_inclusive_and_checked_before_protocol_acceptance() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = authorized_process(resolved, &invocation, &fixture.subject_lock, &subjects);
    let (events, mut result) = provider_evidence(resolved, &invocation);

    let mut provider_stdout = stdout(&events, &result);
    let stdout_limit = usize::try_from(invocation.limits.max_stdout_bytes).unwrap();
    let padding = stdout_limit - provider_stdout.len();
    result.explanation.push_str(&"x".repeat(padding));
    provider_stdout = stdout(&events, &result);
    assert_eq!(provider_stdout.len(), stdout_limit);

    let stderr_limit = usize::try_from(invocation.limits.max_stderr_bytes).unwrap();
    let provider_stderr = vec![b'x'; stderr_limit];
    let mut observed = Vec::new();
    Orchestrator::validate_process_transcript(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &authority,
        ProcessTranscript::new(
            ProcessCompletion::Exited { code: Some(0) },
            &provider_stdout,
            &provider_stderr,
        ),
        &mut observed,
    )
    .unwrap();

    provider_stdout.push(b' ');
    let error = Orchestrator::validate_process_transcript(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &authority,
        ProcessTranscript::new(
            ProcessCompletion::Exited { code: Some(1) },
            &provider_stdout,
            &[],
        ),
        &mut Vec::new(),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ExecutionError::ProcessOutputLimit {
            stream: ProcessStream::Stdout,
            ..
        }
    ));

    let oversized_stderr = vec![b'x'; stderr_limit + 1];
    let error = Orchestrator::validate_process_transcript(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &authority,
        ProcessTranscript::new(
            ProcessCompletion::Exited { code: Some(0) },
            &stdout(&events, &result),
            &oversized_stderr,
        ),
        &mut Vec::new(),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ExecutionError::ProcessOutputLimit {
            stream: ProcessStream::Stderr,
            ..
        }
    ));
}

#[test]
fn abnormal_completion_cannot_promote_provider_success() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = authorized_process(resolved, &invocation, &fixture.subject_lock, &subjects);
    let (events, result) = provider_evidence(resolved, &invocation);
    let provider_stdout = stdout(&events, &result);

    let cases = [
        (ProcessCompletion::Exited { code: Some(7) }, "nonzero-exit"),
        (ProcessCompletion::Exited { code: None }, "signal-exit"),
        (ProcessCompletion::TimedOut, "timeout"),
        (ProcessCompletion::Cancelled, "forced-cancellation"),
    ];

    for (completion, case) in cases {
        let mut observed = Vec::new();
        let error = Orchestrator::validate_process_transcript(
            resolved,
            &invocation,
            &fixture.subject_lock,
            &subjects,
            &authority,
            ProcessTranscript::new(completion, &provider_stdout, &[]),
            &mut observed,
        )
        .unwrap_err();
        assert!(
            matches!(
                error,
                ExecutionError::ProcessExit { .. }
                    | ExecutionError::ProcessTimeout
                    | ExecutionError::ProcessCancelled
            ),
            "unexpected failure for {case}: {error}"
        );
        assert!(error.provider_result().is_none());
        assert!(observed.is_empty());
    }
}

#[test]
fn result_without_a_terminal_event_remains_unaccepted() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = authorized_process(resolved, &invocation, &fixture.subject_lock, &subjects);
    let (_, result) = provider_evidence(resolved, &invocation);
    let provider_stdout = stdout(&[], &result);

    let error = Orchestrator::validate_process_transcript(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &authority,
        ProcessTranscript::new(
            ProcessCompletion::Exited { code: Some(0) },
            &provider_stdout,
            &[],
        ),
        &mut Vec::new(),
    )
    .unwrap_err();

    assert!(matches!(error, ExecutionError::InvalidResult { .. }));
    assert!(error.events().is_empty());
    assert_eq!(error.provider_result(), Some(&result));
}

#[test]
fn graceful_provider_cancellation_with_exit_zero_is_valid_evidence() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = authorized_process(resolved, &invocation, &fixture.subject_lock, &subjects);
    let (mut events, mut result) = provider_evidence(resolved, &invocation);
    let terminal = events.last_mut().unwrap();
    terminal.kind = EventKind::Cancelled;
    terminal.state = EventState::Cancelled;
    result.outcome = Outcome::Cancelled;
    result.failure.classification = FailureClassification::Cancelled;
    result.failure.code = "provider.cancelled".to_owned();
    result.failure.message = "provider observed the cancellation request".to_owned();
    result.failure.retryable = true;
    let provider_stdout = stdout(&events, &result);

    let validated = Orchestrator::validate_process_transcript(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &authority,
        ProcessTranscript::new(
            ProcessCompletion::Exited { code: Some(0) },
            &provider_stdout,
            &[],
        ),
        &mut Vec::new(),
    )
    .unwrap();

    assert_eq!(validated.result().outcome, Outcome::Cancelled);
}

#[test]
fn framed_events_still_use_existing_order_and_identity_validation() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = authorized_process(resolved, &invocation, &fixture.subject_lock, &subjects);
    let (mut events, result) = provider_evidence(resolved, &invocation);
    events[1].sequence = events[0].sequence;
    let provider_stdout = stdout(&events, &result);

    let error = Orchestrator::validate_process_transcript(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &authority,
        ProcessTranscript::new(
            ProcessCompletion::Exited { code: Some(0) },
            &provider_stdout,
            &[],
        ),
        &mut Vec::new(),
    )
    .unwrap_err();
    assert!(matches!(error, ExecutionError::InvalidEvent { .. }));
    assert_eq!(error.events(), events);
    assert!(error.provider_result().is_some());

    let (events, mut result) = provider_evidence(resolved, &invocation);
    result.capability_id = "flow/mismatch".to_owned();
    let provider_stdout = stdout(&events, &result);
    let error = Orchestrator::validate_process_transcript(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &authority,
        ProcessTranscript::new(
            ProcessCompletion::Exited { code: Some(0) },
            &provider_stdout,
            &[],
        ),
        &mut Vec::new(),
    )
    .unwrap_err();
    assert!(matches!(error, ExecutionError::InvalidResult { .. }));
    assert_eq!(error.provider_result(), Some(&result));
}

#[test]
fn successful_observers_do_not_change_authoritative_result_identity() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = authorized_process(resolved, &invocation, &fixture.subject_lock, &subjects);
    let (events, result) = provider_evidence(resolved, &invocation);
    let provider_stdout = stdout(&events, &result);
    let transcript = ProcessTranscript::new(
        ProcessCompletion::Exited { code: Some(0) },
        &provider_stdout,
        &[],
    );
    let mut no_op = |_event: &ExtensionEvent| Ok::<(), EventSinkError>(());
    let without_collection = Orchestrator::validate_process_transcript(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &authority,
        transcript,
        &mut no_op,
    )
    .unwrap();
    let mut collected = Vec::new();
    let with_collection = Orchestrator::validate_process_transcript(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &authority,
        transcript,
        &mut collected,
    )
    .unwrap();

    assert_eq!(without_collection, with_collection);
    assert_eq!(collected, events);
}

#[test]
fn rejecting_event_sink_rejects_process_evidence_without_fallback() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let authority = authorized_process(resolved, &invocation, &fixture.subject_lock, &subjects);
    let (events, result) = provider_evidence(resolved, &invocation);
    let provider_stdout = stdout(&events, &result);
    let mut sink = RejectingSink;

    let error = Orchestrator::validate_process_transcript(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &authority,
        ProcessTranscript::new(
            ProcessCompletion::Exited { code: Some(0) },
            &provider_stdout,
            &[],
        ),
        &mut sink,
    )
    .unwrap_err();

    assert!(matches!(error, ExecutionError::EventSink { .. }));
    assert_eq!(error.events(), events);
    assert_eq!(error.provider_result(), Some(&result));
}

struct RejectingSink;

impl EventSink for RejectingSink {
    fn emit(&mut self, _event: &ExtensionEvent) -> Result<(), EventSinkError> {
        Err(EventSinkError::new("authoritative observer rejected event"))
    }
}
