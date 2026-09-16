//! A deterministic, no-effects, no-artifacts reference extension port.

use crate::contracts::{
    Diagnostic, EXTENSION_EVENT_V1, EXTENSION_RESULT_V1, EventKind, EventState, ExtensionEvent,
    ExtensionInvocation, ExtensionResult, Failure, FailureClassification, Outcome, Progress,
    Severity,
};
use crate::execution::{EventSink, ExtensionPort, PortError, PortIdentity};

/// Deterministic modes used to prove both accepted and adversarial evidence.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum HermeticBehavior {
    #[default]
    Success,
    DuplicateSequence,
    MismatchedEventInvocation,
    MismatchedResultCapability,
    InconsistentOutcomeFailure,
    UnredactedEventDiagnostic,
    UnredactedResultDiagnostic,
    PortFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HermeticExtension {
    identity: PortIdentity,
    behavior: HermeticBehavior,
}

impl HermeticExtension {
    #[must_use]
    pub fn new(identity: PortIdentity) -> Self {
        Self {
            identity,
            behavior: HermeticBehavior::Success,
        }
    }

    #[must_use]
    pub const fn with_behavior(mut self, behavior: HermeticBehavior) -> Self {
        self.behavior = behavior;
        self
    }
}

impl ExtensionPort for HermeticExtension {
    fn identity(&self) -> &PortIdentity {
        &self.identity
    }

    fn invoke(
        &self,
        invocation: &ExtensionInvocation,
        events: &mut dyn EventSink,
    ) -> Result<ExtensionResult, PortError> {
        let started = event(
            invocation,
            "started",
            0,
            EventKind::PhaseStarted,
            EventState::Running,
            0,
        );
        events
            .emit(&started)
            .map_err(|error| PortError::new(error.to_string()))?;

        if self.behavior == HermeticBehavior::PortFailure {
            return Err(PortError::new("deterministic hermetic provider failure"));
        }

        let mut completed = event(
            invocation,
            "completed",
            if self.behavior == HermeticBehavior::DuplicateSequence {
                0
            } else {
                2
            },
            EventKind::PhaseCompleted,
            EventState::Produced,
            1,
        );
        if self.behavior == HermeticBehavior::MismatchedEventInvocation {
            "invocation:adversarial-mismatch".clone_into(&mut completed.invocation_id);
        }
        if self.behavior == HermeticBehavior::UnredactedEventDiagnostic {
            completed.diagnostics.push(unredacted_diagnostic());
        }
        events
            .emit(&completed)
            .map_err(|error| PortError::new(error.to_string()))?;

        let mut result = ExtensionResult {
            schema_version: EXTENSION_RESULT_V1.to_owned(),
            run_id: invocation.run_id.clone(),
            invocation_id: invocation.invocation_id.clone(),
            extension_id: invocation.extension.extension_id.clone(),
            extension_version: invocation.extension.version.clone(),
            extension_integrity: invocation.extension.integrity.clone(),
            capability_id: invocation.capability_id.clone(),
            configuration_digest: invocation.configuration.digest.clone(),
            authorization_id: invocation.authorization.authorization_id.clone(),
            outcome: Outcome::Produced,
            partial_result: false,
            consumed_artifacts: Vec::new(),
            produced_artifacts: Vec::new(),
            validations: Vec::new(),
            provenance: Vec::new(),
            diagnostics: Vec::new(),
            failure: Failure {
                classification: FailureClassification::None,
                code: String::new(),
                message: String::new(),
                retryable: false,
            },
            checkpoint_refs: Vec::new(),
            explanation: "The hermetic extension completed without effects or artifacts."
                .to_owned(),
        };
        match self.behavior {
            HermeticBehavior::MismatchedResultCapability => {
                "flow/adversarial-mismatch".clone_into(&mut result.capability_id);
            }
            HermeticBehavior::InconsistentOutcomeFailure => {
                result.failure.classification = FailureClassification::Provider;
                "adversarial".clone_into(&mut result.failure.code);
                "provider-reported success with failure evidence"
                    .clone_into(&mut result.failure.message);
            }
            HermeticBehavior::UnredactedResultDiagnostic => {
                result.diagnostics.push(unredacted_diagnostic());
            }
            HermeticBehavior::Success
            | HermeticBehavior::DuplicateSequence
            | HermeticBehavior::MismatchedEventInvocation
            | HermeticBehavior::UnredactedEventDiagnostic
            | HermeticBehavior::PortFailure => {}
        }
        Ok(result)
    }
}

fn unredacted_diagnostic() -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        code: "hermetic.unredacted".to_owned(),
        message: "synthetic private diagnostic".to_owned(),
        redacted: false,
    }
}

fn event(
    invocation: &ExtensionInvocation,
    suffix: &str,
    sequence: u64,
    kind: EventKind,
    state: EventState,
    completed: u64,
) -> ExtensionEvent {
    ExtensionEvent {
        schema_version: EXTENSION_EVENT_V1.to_owned(),
        event_id: format!("event:{}-{suffix}", invocation.invocation_id),
        run_id: invocation.run_id.clone(),
        invocation_id: invocation.invocation_id.clone(),
        sequence,
        phase: invocation.phase.lifecycle_point(),
        kind,
        state,
        progress: Progress {
            completed,
            total: 1,
            unit: "invocation".to_owned(),
        },
        diagnostics: Vec::new(),
        artifact_refs: Vec::new(),
        checkpoint_refs: Vec::new(),
    }
}
