mod common;

use std::cell::Cell;

use flow::{
    EventKind, EventSink, EventSinkError, EventState, ExecutionError, ExtensionEvent,
    ExtensionInvocation, ExtensionPort, ExtensionResult, HermeticBehavior, HermeticExtension,
    InputArtifact, Orchestrator, Outcome, PortError, PortIdentity, ValidationEvidence,
    ValidationStatus,
};

use common::{invocation, resolved_fixture};

#[test]
fn hermetic_success_accepts_strictly_increasing_sequences_with_gaps() {
    let (catalog, request) = resolved_fixture();
    let outcome = catalog.resolve(&request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let port = HermeticExtension::new(PortIdentity::from_resolved(resolved));
    let mut observed = Vec::new();

    let validated = Orchestrator::execute(resolved, &invocation, &port, &mut observed).unwrap();

    assert_eq!(
        validated
            .events()
            .iter()
            .map(|event| event.sequence)
            .collect::<Vec<_>>(),
        [0, 2]
    );
    assert_eq!(validated.result().outcome, Outcome::Produced);
    assert_eq!(observed, validated.events());
}

#[test]
fn duplicate_or_decreasing_event_sequence_is_rejected_and_retained() {
    let (catalog, request) = resolved_fixture();
    let outcome = catalog.resolve(&request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let port = HermeticExtension::new(PortIdentity::from_resolved(resolved))
        .with_behavior(HermeticBehavior::DuplicateSequence);
    let mut observed = Vec::new();

    let error = Orchestrator::execute(resolved, &invocation, &port, &mut observed).unwrap_err();

    assert!(matches!(error, ExecutionError::InvalidEvent { .. }));
    assert_eq!(error.events().len(), 2);
    assert_eq!(observed.len(), 1, "invalid event is not forwarded");
}

#[test]
fn cross_invocation_event_is_rejected_and_retained() {
    let (catalog, request) = resolved_fixture();
    let outcome = catalog.resolve(&request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let port = HermeticExtension::new(PortIdentity::from_resolved(resolved))
        .with_behavior(HermeticBehavior::MismatchedEventInvocation);
    let mut observed = Vec::new();

    let error = Orchestrator::execute(resolved, &invocation, &port, &mut observed).unwrap_err();

    assert!(matches!(error, ExecutionError::InvalidEvent { .. }));
    assert_eq!(
        error.events()[1].invocation_id,
        "invocation:adversarial-mismatch"
    );
    assert_eq!(observed.len(), 1);
}

#[test]
fn provider_reported_success_remains_unaccepted_evidence_on_identity_mismatch() {
    let (catalog, request) = resolved_fixture();
    let outcome = catalog.resolve(&request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let port = HermeticExtension::new(PortIdentity::from_resolved(resolved))
        .with_behavior(HermeticBehavior::MismatchedResultCapability);
    let mut observed = Vec::new();

    let error = Orchestrator::execute(resolved, &invocation, &port, &mut observed).unwrap_err();

    assert!(matches!(error, ExecutionError::InvalidResult { .. }));
    let raw_result = error.provider_result().expect("raw result is retained");
    assert_eq!(raw_result.outcome, Outcome::Produced);
    assert_eq!(raw_result.capability_id, "flow/adversarial-mismatch");
    assert_eq!(error.events().len(), 2);
}

#[test]
fn outcome_failure_inconsistency_is_rejected_after_invoke() {
    let (catalog, request) = resolved_fixture();
    let outcome = catalog.resolve(&request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let port = HermeticExtension::new(PortIdentity::from_resolved(resolved))
        .with_behavior(HermeticBehavior::InconsistentOutcomeFailure);
    let mut observed = Vec::new();

    let error = Orchestrator::execute(resolved, &invocation, &port, &mut observed).unwrap_err();

    assert!(matches!(error, ExecutionError::InvalidResult { .. }));
    assert_eq!(error.provider_result().unwrap().outcome, Outcome::Produced);
}

#[test]
fn provider_failure_after_invoke_never_attempts_fallback() {
    let (catalog, request) = resolved_fixture();
    let outcome = catalog.resolve(&request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let port = HermeticExtension::new(PortIdentity::from_resolved(resolved))
        .with_behavior(HermeticBehavior::PortFailure);
    let mut observed = Vec::new();

    let error = Orchestrator::execute(resolved, &invocation, &port, &mut observed).unwrap_err();

    assert!(matches!(error, ExecutionError::Provider { .. }));
    assert_eq!(error.events().len(), 1);
}

#[test]
fn mismatched_port_is_rejected_before_invocation() {
    let (catalog, request) = resolved_fixture();
    let outcome = catalog.resolve(&request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let mut identity = PortIdentity::from_resolved(resolved);
    identity.integrity = "a".repeat(64);
    let port = CountingPort {
        identity,
        calls: Cell::new(0),
    };
    let mut observed = Vec::new();

    let error = Orchestrator::execute(resolved, &invocation, &port, &mut observed).unwrap_err();

    assert!(matches!(error, ExecutionError::Preflight { .. }));
    assert_eq!(port.calls.get(), 0);
    assert!(observed.is_empty());
}

#[test]
fn invocation_identity_mismatch_is_rejected_before_invocation() {
    let (catalog, request) = resolved_fixture();
    let outcome = catalog.resolve(&request);
    let resolved = outcome.resolved().unwrap();
    let mut invocation = invocation(resolved);
    invocation.extension.version = "9.9.9".to_owned();
    let port = CountingPort {
        identity: PortIdentity::from_resolved(resolved),
        calls: Cell::new(0),
    };
    let mut observed = Vec::new();

    let error = Orchestrator::execute(resolved, &invocation, &port, &mut observed).unwrap_err();

    assert!(matches!(error, ExecutionError::Preflight { .. }));
    assert_eq!(port.calls.get(), 0);
}

#[test]
fn every_terminal_result_identity_is_correlated() {
    let mutations: [fn(&mut ExtensionResult); 8] = [
        |result| "run:mismatch".clone_into(&mut result.run_id),
        |result| "invocation:mismatch".clone_into(&mut result.invocation_id),
        |result| "org.example.mismatch".clone_into(&mut result.extension_id),
        |result| "9.9.9".clone_into(&mut result.extension_version),
        |result| result.extension_integrity = "a".repeat(64),
        |result| "flow/mismatch".clone_into(&mut result.capability_id),
        |result| result.configuration_digest = "a".repeat(64),
        |result| "authorization:mismatch".clone_into(&mut result.authorization_id),
    ];

    for mutate in mutations {
        let (catalog, request) = resolved_fixture();
        let outcome = catalog.resolve(&request);
        let resolved = outcome.resolved().unwrap();
        let invocation = invocation(resolved);
        let port = MutatingResultPort {
            inner: HermeticExtension::new(PortIdentity::from_resolved(resolved)),
            mutate,
        };
        let mut observed = Vec::new();

        let error = Orchestrator::execute(resolved, &invocation, &port, &mut observed).unwrap_err();
        assert!(matches!(error, ExecutionError::InvalidResult { .. }));
        assert_eq!(error.provider_result().unwrap().outcome, Outcome::Produced);
    }
}

#[test]
fn downstream_event_sink_rejection_fails_execution_without_losing_raw_event() {
    let (catalog, request) = resolved_fixture();
    let outcome = catalog.resolve(&request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let port = HermeticExtension::new(PortIdentity::from_resolved(resolved));
    let mut sink = RejectingSink;

    let error = Orchestrator::execute(resolved, &invocation, &port, &mut sink).unwrap_err();

    assert!(matches!(error, ExecutionError::EventSink { .. }));
    assert_eq!(error.events().len(), 1);
    assert!(error.provider_result().is_none());
}

#[test]
fn unredacted_event_diagnostic_is_rejected_before_observation() {
    let (catalog, request) = resolved_fixture();
    let outcome = catalog.resolve(&request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let port = HermeticExtension::new(PortIdentity::from_resolved(resolved))
        .with_behavior(HermeticBehavior::UnredactedEventDiagnostic);
    let mut observed = Vec::new();

    let error = Orchestrator::execute(resolved, &invocation, &port, &mut observed).unwrap_err();

    assert!(matches!(error, ExecutionError::InvalidEvent { .. }));
    assert_eq!(error.events().len(), 2);
    assert!(!error.events()[1].diagnostics[0].redacted);
    assert_eq!(observed.len(), 1, "unredacted event is not forwarded");
}

#[test]
fn unredacted_result_diagnostic_is_retained_but_never_validated() {
    let (catalog, request) = resolved_fixture();
    let outcome = catalog.resolve(&request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let port = HermeticExtension::new(PortIdentity::from_resolved(resolved))
        .with_behavior(HermeticBehavior::UnredactedResultDiagnostic);
    let mut observed = Vec::new();

    let error = Orchestrator::execute(resolved, &invocation, &port, &mut observed).unwrap_err();

    assert!(matches!(error, ExecutionError::InvalidResult { .. }));
    assert!(!error.provider_result().unwrap().diagnostics[0].redacted);
}

#[test]
fn provider_success_with_failed_validation_is_retained_but_not_accepted() {
    let (catalog, request) = resolved_fixture();
    let outcome = catalog.resolve(&request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let port = MutatingResultPort {
        inner: HermeticExtension::new(PortIdentity::from_resolved(resolved)),
        mutate: |result| {
            result.validations.push(ValidationEvidence {
                validator: "synthetic.adversarial/v1".to_owned(),
                status: ValidationStatus::Failed,
                evidence: "fixture:validation-failed".to_owned(),
            });
        },
    };
    let mut observed = Vec::new();

    let error = Orchestrator::execute(resolved, &invocation, &port, &mut observed).unwrap_err();

    assert!(matches!(error, ExecutionError::InvalidResult { .. }));
    assert_eq!(
        error.provider_result().unwrap().validations[0].status,
        ValidationStatus::Failed
    );
}

#[test]
fn consumed_artifact_identifiers_must_name_invocation_inputs() {
    let (catalog, request) = resolved_fixture();
    let outcome = catalog.resolve(&request);
    let resolved = outcome.resolved().unwrap();
    let mut invocation = invocation(resolved);
    invocation.input_artifacts.push(InputArtifact {
        artifact_id: "artifact:declared-input".to_owned(),
        digest: "5".repeat(64),
    });

    let matching_port = MutatingResultPort {
        inner: HermeticExtension::new(PortIdentity::from_resolved(resolved)),
        mutate: |result| {
            result
                .consumed_artifacts
                .push("artifact:declared-input".to_owned());
        },
    };
    let mut observed = Vec::new();
    Orchestrator::execute(resolved, &invocation, &matching_port, &mut observed).unwrap();

    let unknown_port = MutatingResultPort {
        inner: HermeticExtension::new(PortIdentity::from_resolved(resolved)),
        mutate: |result| {
            result
                .consumed_artifacts
                .push("artifact:undeclared-input".to_owned());
        },
    };
    let mut observed = Vec::new();
    let error =
        Orchestrator::execute(resolved, &invocation, &unknown_port, &mut observed).unwrap_err();
    assert!(matches!(error, ExecutionError::InvalidResult { .. }));
}

#[test]
fn duplicate_invocation_artifact_identifiers_are_rejected_before_invocation() {
    let (catalog, request) = resolved_fixture();
    let outcome = catalog.resolve(&request);
    let resolved = outcome.resolved().unwrap();
    let mut invocation = invocation(resolved);
    invocation.input_artifacts = vec![
        InputArtifact {
            artifact_id: "artifact:ambiguous".to_owned(),
            digest: "5".repeat(64),
        },
        InputArtifact {
            artifact_id: "artifact:ambiguous".to_owned(),
            digest: "6".repeat(64),
        },
    ];
    let port = CountingPort {
        identity: PortIdentity::from_resolved(resolved),
        calls: Cell::new(0),
    };
    let mut observed = Vec::new();

    let error = Orchestrator::execute(resolved, &invocation, &port, &mut observed).unwrap_err();

    assert!(matches!(error, ExecutionError::InvalidInvocation { .. }));
    assert_eq!(port.calls.get(), 0);
}

#[test]
fn artifact_produced_events_correlate_with_terminal_result_identifiers() {
    let (catalog, request) = resolved_fixture();
    let outcome = catalog.resolve(&request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);

    let matching_port = ArtifactEventPort {
        inner: HermeticExtension::new(PortIdentity::from_resolved(resolved)),
        declare_in_result: true,
    };
    let mut observed = Vec::new();
    let validated =
        Orchestrator::execute(resolved, &invocation, &matching_port, &mut observed).unwrap();
    assert_eq!(validated.events().len(), 3);

    let missing_port = ArtifactEventPort {
        inner: HermeticExtension::new(PortIdentity::from_resolved(resolved)),
        declare_in_result: false,
    };
    let mut observed = Vec::new();
    let error =
        Orchestrator::execute(resolved, &invocation, &missing_port, &mut observed).unwrap_err();
    assert!(matches!(error, ExecutionError::InvalidResult { .. }));
    assert_eq!(error.events().len(), 3);
}

struct CountingPort {
    identity: PortIdentity,
    calls: Cell<u64>,
}

impl ExtensionPort for CountingPort {
    fn identity(&self) -> &PortIdentity {
        &self.identity
    }

    fn invoke(
        &self,
        _invocation: &ExtensionInvocation,
        _events: &mut dyn EventSink,
    ) -> Result<ExtensionResult, PortError> {
        self.calls.set(self.calls.get() + 1);
        Err(PortError::new("counting port should not have been invoked"))
    }
}

struct MutatingResultPort {
    inner: HermeticExtension,
    mutate: fn(&mut ExtensionResult),
}

impl ExtensionPort for MutatingResultPort {
    fn identity(&self) -> &PortIdentity {
        self.inner.identity()
    }

    fn invoke(
        &self,
        invocation: &ExtensionInvocation,
        events: &mut dyn EventSink,
    ) -> Result<ExtensionResult, PortError> {
        let mut result = self.inner.invoke(invocation, events)?;
        (self.mutate)(&mut result);
        Ok(result)
    }
}

struct RejectingSink;

impl EventSink for RejectingSink {
    fn emit(&mut self, _event: &ExtensionEvent) -> Result<(), EventSinkError> {
        Err(EventSinkError::new("observer rejected event"))
    }
}

struct ArtifactEventPort {
    inner: HermeticExtension,
    declare_in_result: bool,
}

impl ExtensionPort for ArtifactEventPort {
    fn identity(&self) -> &PortIdentity {
        self.inner.identity()
    }

    fn invoke(
        &self,
        invocation: &ExtensionInvocation,
        events: &mut dyn EventSink,
    ) -> Result<ExtensionResult, PortError> {
        let mut injecting_sink = ArtifactInjectingSink { downstream: events };
        let mut result = self.inner.invoke(invocation, &mut injecting_sink)?;
        if self.declare_in_result {
            result
                .produced_artifacts
                .push("artifact:synthetic-output".to_owned());
        }
        Ok(result)
    }
}

struct ArtifactInjectingSink<'a> {
    downstream: &'a mut dyn EventSink,
}

impl EventSink for ArtifactInjectingSink<'_> {
    fn emit(&mut self, event: &ExtensionEvent) -> Result<(), EventSinkError> {
        if event.kind == EventKind::PhaseCompleted {
            let mut artifact_event = event.clone();
            artifact_event.event_id.push_str(":artifact");
            artifact_event.sequence = event.sequence.saturating_sub(1);
            artifact_event.kind = EventKind::ArtifactProduced;
            artifact_event.state = EventState::Running;
            artifact_event.progress.completed = 0;
            artifact_event.artifact_refs = vec!["artifact:synthetic-output".to_owned()];
            self.downstream.emit(&artifact_event)?;
        }
        self.downstream.emit(event)
    }
}
