use std::path::Path;

use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{
    AcceptedArtifactSet, ArtifactAcceptanceError, ArtifactBindingSet, ArtifactObservationError,
    AuthorizedProcess, CancellationSignal, EventSink, ExecutionSubjectLock, ExtensionInvocation,
    LocalProcessRunner, Orchestrator, ProcessIsolation, ProcessRunnerError, ResolvedExtension,
    SecretResolver, accept_artifacts, observe_artifacts, observe_execution_subjects,
};

use super::{
    CheckpointContext, PlannedStep, RUN_ARTIFACT_V1, RUN_AUTHORITY_V1, RUN_CHECKPOINT_V1,
    RUN_RECOVERY_V1, RUN_VALIDATION_PROFILE, RUN_VALIDATION_V1, RunArtifactRecord, RunArtifactRole,
    RunAuthorityDecision, RunCheckpoint, RunFailure, RunPlan, RunRecoveryAction,
    RunRecoveryDecision, RunState, RunStepStatus, RunStore, RunValidationEvidence, StateBoundary,
    StateError, digest, require,
};

/// Fresh caller-owned context. Absolute roots and authorization tokens are never serialized.
pub struct ProcessStepContext<'a> {
    pub execution_root: &'a Path,
    pub artifact_root: &'a Path,
    pub resolved: &'a ResolvedExtension,
    pub invocation: &'a ExtensionInvocation,
    pub subjects: &'a ExecutionSubjectLock,
    pub authority: &'a AuthorizedProcess,
    pub bindings: &'a ArtifactBindingSet,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResumeEligibility {
    Ready,
    Reusable,
    ApprovalRequired,
    DependencyBlocked,
    Abandoned,
}

/// An explicit caller decision. It does not itself grant process authority.
pub struct RecoveryApproval {
    pub decision_id: String,
    pub action: RunRecoveryAction,
    pub acknowledge_uncertain_effects: bool,
}

#[derive(Debug, Error)]
pub enum DurableExecutionError {
    #[error("durable coordination failed: {0}")]
    State(#[from] StateError),
    #[error("cancelled before launch")]
    CancelledBeforeLaunch,
    #[error("provider execution failed")]
    Process(#[source] ProcessRunnerError),
    #[error("artifact observation failed")]
    Observation(#[source] ArtifactObservationError),
    #[error("artifact acceptance failed")]
    Acceptance(#[source] ArtifactAcceptanceError),
}

impl PlannedStep {
    /// Prepare immutable intent through the existing subject and authority preflight.
    ///
    /// Input bytes are checked at execution/assessment time, allowing prepared
    /// dependencies whose expected input identities are already known.
    ///
    /// # Errors
    /// Rejects invalid bindings, process identity, subject bytes, or authority.
    pub fn prepare(
        step_id: String,
        depends_on: Vec<String>,
        context: &ProcessStepContext<'_>,
    ) -> Result<Self, StateError> {
        context.preflight()?;
        Ok(Self {
            step_id,
            depends_on,
            context: context.identity()?,
        })
    }
}

impl ProcessStepContext<'_> {
    fn identity(&self) -> Result<CheckpointContext, StateError> {
        let invocation = self.invocation;
        Ok(CheckpointContext {
            run_id: invocation.run_id.clone(),
            invocation_id: invocation.invocation_id.clone(),
            invocation_digest: digest(invocation)?,
            provider: invocation.extension.clone(),
            interface: invocation.interface.clone(),
            capability_id: invocation.capability_id.clone(),
            capability_digest: digest(self.resolved.capability())?,
            configuration_schema: invocation.configuration.schema_id.clone(),
            configuration_digest: invocation.configuration.digest.clone(),
            configuration_values_digest: digest(&invocation.configuration.values)?,
            subject_lock_digest: digest(self.subjects)?,
            authorization_id: invocation.authorization.authorization_id.clone(),
            authority_profile_digest: self.authority.profile_digest().to_owned(),
            enforcement_evidence_digest: self.authority.evidence_digest().to_owned(),
            grants_digest: invocation.authorization.grants_digest.clone(),
            bindings: self.bindings.clone(),
        })
    }

    fn preflight(&self) -> Result<(), StateError> {
        self.bindings.validate().map_err(|_| StateError::Invalid {
            rule: "artifact bindings",
        })?;
        let declared: std::collections::BTreeSet<_> = self
            .invocation
            .input_artifacts
            .iter()
            .map(|a| (&a.artifact_id, &a.digest))
            .collect();
        let bound: std::collections::BTreeSet<_> = self
            .bindings
            .inputs
            .iter()
            .map(|a| (&a.artifact_id, &a.expected_digest))
            .collect();
        require(declared == bound, "invocation input bindings")?;
        let subjects = observe_execution_subjects(
            self.execution_root,
            self.resolved,
            self.invocation,
            self.subjects,
        )
        .map_err(|_| StateError::Stale {
            boundary: StateBoundary::Provider,
        })?;
        Orchestrator::encode_process_request(
            self.resolved,
            self.invocation,
            self.subjects,
            &subjects,
            self.authority,
        )
        .map_err(|_| StateError::Stale {
            boundary: StateBoundary::Authority,
        })?;
        require(
            self.authority.isolation() == ProcessIsolation::TrustedUnconfined,
            "supported durable runner isolation",
        )
    }

    fn check_inputs(&self) -> Result<(), StateError> {
        if self.bindings.inputs.is_empty() {
            return Ok(());
        }
        let bindings = ArtifactBindingSet {
            outputs: Vec::new(),
            ..self.bindings.clone()
        };
        let observed =
            observe_artifacts(self.artifact_root, &bindings).map_err(|_| StateError::Stale {
                boundary: StateBoundary::Inputs,
            })?;
        for (binding, observation) in bindings.inputs.iter().zip(&observed.evidence().artifacts) {
            if binding.expected_digest != observation.digest {
                return Err(StateError::Stale {
                    boundary: StateBoundary::Inputs,
                });
            }
        }
        Ok(())
    }
}

impl RunStore {
    /// Assess current evidence against the caller's expected complete plan.
    ///
    /// `Reusable` is an eligibility decision about recorded acceptance, not a
    /// reconstructed `AcceptedArtifactSet` or permission to launch a provider.
    ///
    /// # Errors
    /// Refuses changed plans, configuration, inputs, subjects, authority, validation
    /// implementations, and output evidence. This method writes no state.
    pub fn assess(
        &self,
        expected_plan: &RunPlan,
        step_id: &str,
        context: &ProcessStepContext<'_>,
    ) -> Result<ResumeEligibility, StateError> {
        self.ensure_usable()?;
        if expected_plan.digest()? != self.state.plan_digest {
            return Err(StateError::Stale {
                boundary: StateBoundary::Plan,
            });
        }
        let index = self.state.step_index(step_id)?;
        self.check_context(index, context)?;
        let planned = &self.state.plan.steps[index];
        if planned.depends_on.iter().any(|id| {
            self.state
                .steps
                .iter()
                .any(|s| &s.step_id == id && s.status != RunStepStatus::Succeeded)
        }) {
            return Ok(ResumeEligibility::DependencyBlocked);
        }
        let step = &self.state.steps[index];
        match step.status {
            RunStepStatus::Pending => Ok(ResumeEligibility::Ready),
            RunStepStatus::Succeeded => {
                let checkpoint = step.checkpoint.as_ref().ok_or(StateError::Corrupt)?;
                let validation = &checkpoint.validation;
                if validation.implementation_version != env!("CARGO_PKG_VERSION")
                    || validation.implementation_digest != validation_implementation_digest()
                    || validation.platform != std::env::consts::OS
                    || validation.profile != RUN_VALIDATION_PROFILE
                {
                    return Err(StateError::Stale {
                        boundary: StateBoundary::Validation,
                    });
                }
                let current =
                    observe_artifacts(context.artifact_root, context.bindings).map_err(|_| {
                        StateError::Stale {
                            boundary: StateBoundary::Artifacts,
                        }
                    })?;
                let saved: Vec<_> = checkpoint
                    .artifacts
                    .iter()
                    .map(|a| a.observation.clone())
                    .collect();
                let mut current = current.into_evidence().artifacts;
                current.sort_by(|a, b| a.artifact_id.cmp(&b.artifact_id));
                if saved != current {
                    return Err(StateError::Stale {
                        boundary: StateBoundary::Artifacts,
                    });
                }
                Ok(ResumeEligibility::Reusable)
            }
            RunStepStatus::Abandoned => Ok(ResumeEligibility::Abandoned),
            RunStepStatus::Running
            | RunStepStatus::Failed
            | RunStepStatus::Cancelled
            | RunStepStatus::Denied => Ok(ResumeEligibility::ApprovalRequired),
        }
    }

    fn check_context(
        &self,
        index: usize,
        context: &ProcessStepContext<'_>,
    ) -> Result<(), StateError> {
        let saved = &self.state.plan.steps[index].context;
        let current = context.identity()?;
        let checks = [
            (
                saved.provider == current.provider
                    && saved.subject_lock_digest == current.subject_lock_digest,
                StateBoundary::Provider,
            ),
            (
                saved.capability_id == current.capability_id
                    && saved.capability_digest == current.capability_digest
                    && saved.interface == current.interface,
                StateBoundary::Capability,
            ),
            (
                saved.configuration_schema == current.configuration_schema
                    && saved.configuration_digest == current.configuration_digest
                    && saved.configuration_values_digest == current.configuration_values_digest,
                StateBoundary::Configuration,
            ),
            (
                saved.authority_profile_digest == current.authority_profile_digest
                    && saved.enforcement_evidence_digest == current.enforcement_evidence_digest
                    && saved.authorization_id == current.authorization_id
                    && saved.grants_digest == current.grants_digest,
                StateBoundary::Authority,
            ),
            (saved.bindings == current.bindings, StateBoundary::Bindings),
            (saved == &current, StateBoundary::Invocation),
        ];
        for (matches, boundary) in checks {
            if !matches {
                return Err(StateError::Stale { boundary });
            }
        }
        context.preflight()?;
        context.check_inputs()
    }

    /// Persist launch intent, execute one prepared step, and persist accepted evidence.
    ///
    /// The store remains locked throughout the attempt. Any failure to commit the
    /// terminal result is an error; the prior running record remains uncertain.
    /// Existing completed steps are never executed again by this method.
    ///
    /// # Errors
    /// Returns typed coordination, cancellation, process, observation, or acceptance
    /// failures. Raw provider errors remain caller-local and are never serialized.
    #[allow(clippy::too_many_lines)]
    pub fn execute(
        &mut self,
        step_id: &str,
        context: &ProcessStepContext<'_>,
        secrets: &dyn SecretResolver,
        cancellation: &dyn CancellationSignal,
        events: &mut dyn EventSink,
    ) -> Result<AcceptedArtifactSet, DurableExecutionError> {
        if self.assess(&self.state.plan, step_id, context)? != ResumeEligibility::Ready {
            return Err(StateError::Transition.into());
        }
        if cancellation.is_cancelled() {
            self.cancel_pending(step_id)?;
            return Err(DurableExecutionError::CancelledBeforeLaunch);
        }
        let index = self.state.step_index(step_id)?;
        let mut next = self.state.clone();
        next.steps[index].attempt += 1;
        next.steps[index].status = RunStepStatus::Running;
        next.authority_decisions
            .push(authority_decision(&next, index, true));
        self.commit(next)?;

        if cancellation.is_cancelled() {
            self.finish_failure(index, RunFailure::Cancelled)?;
            return Err(DurableExecutionError::CancelledBeforeLaunch);
        }

        let execution = match LocalProcessRunner::run_with_cancellation(
            context.execution_root,
            context.resolved,
            context.invocation,
            context.subjects,
            context.authority,
            secrets,
            cancellation,
            events,
        ) {
            Ok(value) => value,
            Err(error) => {
                let failure = match &error {
                    ProcessRunnerError::Cancelled { .. } => RunFailure::Cancelled,
                    ProcessRunnerError::TimedOut { .. } => RunFailure::TimedOut,
                    _ => RunFailure::Process,
                };
                self.finish_failure(index, failure)?;
                return Err(DurableExecutionError::Process(error));
            }
        };
        let observed = match observe_artifacts(context.artifact_root, context.bindings) {
            Ok(value) => value,
            Err(error) => {
                self.finish_failure(index, RunFailure::ArtifactObservation)?;
                return Err(DurableExecutionError::Observation(error));
            }
        };
        let accepted = match accept_artifacts(
            context.resolved,
            context.invocation,
            &execution,
            context.bindings,
            &observed,
        ) {
            Ok(value) => value,
            Err(error) => {
                self.finish_failure(index, RunFailure::ArtifactAcceptance)?;
                return Err(DurableExecutionError::Acceptance(error));
            }
        };
        let mut artifacts: Vec<_> = accepted
            .inputs()
            .iter()
            .map(|a| (RunArtifactRole::Input, a))
            .chain(
                accepted
                    .outputs()
                    .iter()
                    .map(|a| (RunArtifactRole::Output, a)),
            )
            .map(|(role, observation)| RunArtifactRecord {
                schema_version: RUN_ARTIFACT_V1.to_owned(),
                role,
                observation: observation.clone(),
            })
            .collect();
        artifacts.sort_by(|a, b| a.observation.artifact_id.cmp(&b.observation.artifact_id));
        let attempt = self.state.steps[index].attempt;
        let context_digest = digest(&self.state.plan.steps[index].context)?;
        let checkpoint_id = format!(
            "checkpoint:{}",
            digest(&(&self.state.plan_digest, step_id, attempt, &context_digest))?
        );
        let validation = RunValidationEvidence {
            schema_version: RUN_VALIDATION_V1.to_owned(),
            profile: RUN_VALIDATION_PROFILE.to_owned(),
            implementation_version: env!("CARGO_PKG_VERSION").to_owned(),
            implementation_digest: validation_implementation_digest(),
            platform: std::env::consts::OS.to_owned(),
            invocation_digest: self.state.plan.steps[index]
                .context
                .invocation_digest
                .clone(),
            execution_digest: digest(&(execution.events(), execution.result()))?,
            artifacts_digest: digest(&artifacts)?,
        };
        let checkpoint = RunCheckpoint {
            schema_version: RUN_CHECKPOINT_V1.to_owned(),
            checkpoint_id,
            plan_digest: self.state.plan_digest.clone(),
            context_digest,
            attempt,
            artifacts,
            validation,
        };
        let mut next = self.state.clone();
        next.steps[index].status = RunStepStatus::Succeeded;
        next.steps[index].checkpoint = Some(checkpoint);
        self.commit(next)?;
        Ok(accepted)
    }

    fn finish_failure(&mut self, index: usize, failure: RunFailure) -> Result<(), StateError> {
        let mut next = self.state.clone();
        next.steps[index].status = if failure == RunFailure::Cancelled {
            RunStepStatus::Cancelled
        } else {
            RunStepStatus::Failed
        };
        next.steps[index].failure = Some(failure);
        self.commit(next)
    }

    /// Cancel a pending step without entering provider code.
    ///
    /// # Errors
    /// Refuses non-pending steps or failed persistence.
    pub fn cancel_pending(&mut self, step_id: &str) -> Result<(), StateError> {
        let index = self.state.step_index(step_id)?;
        require(
            self.state.steps[index].status == RunStepStatus::Pending,
            "pending cancellation",
        )?;
        let mut next = self.state.clone();
        next.steps[index].status = RunStepStatus::Cancelled;
        next.steps[index].failure = Some(RunFailure::Cancelled);
        self.commit(next)
    }

    /// Record an operator denial without granting authority or entering provider code.
    ///
    /// # Errors
    /// Refuses non-pending steps or failed persistence.
    pub fn deny_pending(&mut self, step_id: &str) -> Result<(), StateError> {
        let index = self.state.step_index(step_id)?;
        require(
            self.state.steps[index].status == RunStepStatus::Pending,
            "pending denial",
        )?;
        let mut next = self.state.clone();
        next.steps[index].status = RunStepStatus::Denied;
        next.authority_decisions
            .push(authority_decision(&next, index, false));
        self.commit(next)
    }

    /// Record an explicit recovery choice using fresh matching authority.
    ///
    /// Retry only returns the step to pending. No process is launched. The caller
    /// acknowledges that an interrupted unconfined provider may still have effects
    /// or surviving descendants and must resolve those before retrying.
    ///
    /// # Errors
    /// Refuses stale evidence, completed work, missing acknowledgement, duplicate
    /// decision identities, or failed persistence.
    pub fn decide_recovery(
        &mut self,
        step_id: &str,
        context: &ProcessStepContext<'_>,
        approval: RecoveryApproval,
    ) -> Result<(), StateError> {
        let index = self.state.step_index(step_id)?;
        self.check_context(index, context)?;
        if !matches!(
            self.state.steps[index].status,
            RunStepStatus::Running
                | RunStepStatus::Failed
                | RunStepStatus::Cancelled
                | RunStepStatus::Denied
        ) {
            return Err(StateError::Transition);
        }
        require(
            approval.acknowledge_uncertain_effects,
            "explicit recovery acknowledgement",
        )?;
        let mut next = self.state.clone();
        let identity = &next.plan.steps[index].context;
        next.recovery_decisions.push(RunRecoveryDecision {
            schema_version: RUN_RECOVERY_V1.to_owned(),
            decision_id: approval.decision_id,
            step_id: step_id.to_owned(),
            attempt: next.steps[index].attempt,
            action: approval.action,
            authorization_id: identity.authorization_id.clone(),
            profile_digest: identity.authority_profile_digest.clone(),
            acknowledged_uncertain_effects: true,
        });
        next.steps[index].status = if approval.action == RunRecoveryAction::Retry {
            RunStepStatus::Pending
        } else {
            RunStepStatus::Abandoned
        };
        next.steps[index].failure = None;
        self.commit(next)
    }
}

fn authority_decision(state: &RunState, index: usize, granted: bool) -> RunAuthorityDecision {
    let identity = &state.plan.steps[index].context;
    RunAuthorityDecision {
        schema_version: RUN_AUTHORITY_V1.to_owned(),
        step_id: state.steps[index].step_id.clone(),
        attempt: state.steps[index].attempt,
        granted,
        authorization_id: identity.authorization_id.clone(),
        profile_digest: identity.authority_profile_digest.clone(),
        enforcement_digest: identity.enforcement_evidence_digest.clone(),
        grants_digest: identity.grants_digest.clone(),
    }
}

/// Identity of the validation sources and locked dependency recipe compiled into Flow.
/// This is reproducibility metadata, not executable or publisher authentication.
#[must_use]
pub fn validation_implementation_digest() -> String {
    let sources: &[&[u8]] = &[
        include_bytes!("../lib.rs"),
        include_bytes!("../contracts.rs"),
        include_bytes!("../catalog.rs"),
        include_bytes!("../execution.rs"),
        include_bytes!("../process.rs"),
        include_bytes!("../runner.rs"),
        include_bytes!("../artifacts.rs"),
        include_bytes!("../authority.rs"),
        include_bytes!("../execution_subjects.rs"),
        include_bytes!("mod.rs"),
        include_bytes!("model.rs"),
        include_bytes!("store.rs"),
        include_bytes!("execution.rs"),
        include_bytes!("../../Cargo.lock"),
    ];
    let mut hasher = Sha256::new();
    hasher.update(b"flow.run-validator-sources/v1\0");
    for source in sources {
        hasher.update((source.len() as u64).to_le_bytes());
        hasher.update(source);
    }
    format!("{:x}", hasher.finalize())
}
