use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{StateError, digest, require, schema, valid_digest, valid_id};
use crate::{
    ArtifactBindingSet, HostArtifactObservation, InvocationExtension, InvocationInterface,
};

pub const RUN_PLAN_V1: &str = "flow.run-plan/v1";
pub const RUN_STATE_V1: &str = "flow.run-state/v1";
pub const RUN_CHECKPOINT_V1: &str = "flow.run-checkpoint/v1";
pub const RUN_ARTIFACT_V1: &str = "flow.run-artifact/v1";
pub const RUN_AUTHORITY_V1: &str = "flow.run-authority/v1";
pub const RUN_VALIDATION_V1: &str = "flow.run-validation/v1";
pub const RUN_RECOVERY_V1: &str = "flow.run-recovery/v1";
pub const RUN_SNAPSHOT_V1: &str = "flow.run-snapshot/v1";
pub const RUN_VALIDATION_PROFILE: &str = "flow.accept-artifacts/v1";
pub const MAX_RUN_STEPS: usize = 256;
pub const MAX_RUN_ARTIFACTS: usize = 4096;
pub const MAX_RUN_RECORD_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_RUN_SNAPSHOTS: u64 = 4096;

/// Immutable prepared intent. Values and secret material are represented by digests.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunPlan {
    pub schema_version: String,
    pub plan_id: String,
    pub run_id: String,
    pub steps: Vec<PlannedStep>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedStep {
    pub step_id: String,
    pub depends_on: Vec<String>,
    pub context: CheckpointContext,
}

/// Complete identity needed to assess a saved step against a current invocation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointContext {
    pub run_id: String,
    pub invocation_id: String,
    pub invocation_digest: String,
    pub provider: InvocationExtension,
    pub interface: InvocationInterface,
    pub capability_id: String,
    pub capability_digest: String,
    pub configuration_schema: String,
    pub configuration_digest: String,
    pub configuration_values_digest: String,
    pub subject_lock_digest: String,
    pub authorization_id: String,
    pub authority_profile_digest: String,
    pub enforcement_evidence_digest: String,
    pub grants_digest: String,
    pub bindings: ArtifactBindingSet,
}

/// A complete state snapshot. Public records are inspectable data, not trust tokens.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunState {
    pub schema_version: String,
    pub sequence: u64,
    pub previous_digest: String,
    pub plan_digest: String,
    pub plan: RunPlan,
    pub steps: Vec<RunStepState>,
    pub authority_decisions: Vec<RunAuthorityDecision>,
    pub recovery_decisions: Vec<RunRecoveryDecision>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunStepState {
    pub step_id: String,
    pub attempt: u64,
    pub status: RunStepStatus,
    pub checkpoint: Option<RunCheckpoint>,
    pub failure: Option<RunFailure>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunStepStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Denied,
    Abandoned,
}

/// A running record observed after reopen is uncertain, never automatically retried.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunInspectionStatus {
    Prepared,
    Succeeded,
    RecoveryRequired,
    Stopped,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunFailure {
    Cancelled,
    TimedOut,
    Process,
    ArtifactObservation,
    ArtifactAcceptance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunCheckpoint {
    pub schema_version: String,
    pub checkpoint_id: String,
    pub plan_digest: String,
    pub context_digest: String,
    pub attempt: u64,
    pub artifacts: Vec<RunArtifactRecord>,
    pub validation: RunValidationEvidence,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunArtifactRecord {
    pub schema_version: String,
    pub role: RunArtifactRole,
    pub observation: HostArtifactObservation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunArtifactRole {
    Input,
    Output,
}

/// Digests of accepted evidence only; provider free text is deliberately absent.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunValidationEvidence {
    pub schema_version: String,
    pub profile: String,
    pub implementation_version: String,
    pub implementation_digest: String,
    pub platform: String,
    pub invocation_digest: String,
    pub execution_digest: String,
    pub artifacts_digest: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunAuthorityDecision {
    pub schema_version: String,
    pub step_id: String,
    pub attempt: u64,
    pub granted: bool,
    pub authorization_id: String,
    pub profile_digest: String,
    pub enforcement_digest: String,
    pub grants_digest: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecoveryDecision {
    pub schema_version: String,
    pub decision_id: String,
    pub step_id: String,
    pub attempt: u64,
    pub action: RunRecoveryAction,
    pub authorization_id: String,
    pub profile_digest: String,
    pub acknowledged_uncertain_effects: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunRecoveryAction {
    Retry,
    Abandon,
}

impl RunPlan {
    /// Validate a bounded, ordered graph of fully pinned process steps.
    ///
    /// # Errors
    /// Rejects unknown versions, malformed identities, or invalid dependencies.
    pub fn validate(&self) -> Result<(), StateError> {
        schema(&self.schema_version, RUN_PLAN_V1)?;
        require(valid_id(&self.plan_id, "plan:"), "plan identity")?;
        require(valid_id(&self.run_id, "run:"), "run identity")?;
        require(
            !self.steps.is_empty() && self.steps.len() <= MAX_RUN_STEPS,
            "step budget",
        )?;
        let mut seen = BTreeSet::new();
        let mut invocations = BTreeSet::new();
        let mut outputs = BTreeSet::new();
        for step in &self.steps {
            require(valid_id(&step.step_id, "step:"), "step identity")?;
            require(
                step.depends_on.windows(2).all(|p| p[0] < p[1]),
                "sorted unique dependencies",
            )?;
            require(
                step.depends_on.iter().all(|id| seen.contains(id)),
                "dependencies precede step",
            )?;
            require(seen.insert(step.step_id.clone()), "unique steps")?;
            require(
                invocations.insert(&step.context.invocation_id),
                "unique invocations",
            )?;
            step.context.validate()?;
            require(step.context.run_id == self.run_id, "step run identity")?;
            for output in &step.context.bindings.outputs {
                require(
                    outputs.insert(&output.artifact_id),
                    "unique output artifact identities",
                )?;
            }
        }
        Ok(())
    }

    /// Deterministic digest of the complete immutable plan.
    ///
    /// # Errors
    /// Rejects an invalid plan or an unencodable record.
    pub fn digest(&self) -> Result<String, StateError> {
        self.validate()?;
        digest(self)
    }
}

impl CheckpointContext {
    fn validate(&self) -> Result<(), StateError> {
        require(valid_id(&self.run_id, "run:"), "context run identity")?;
        require(
            valid_id(&self.invocation_id, "invocation:"),
            "invocation identity",
        )?;
        require(
            valid_id(&self.authorization_id, "authorization:"),
            "authorization identity",
        )?;
        require(
            crate::contracts::is_extension_id(&self.provider.extension_id),
            "provider identity",
        )?;
        require(
            crate::contracts::is_publisher_id(&self.provider.publisher_id),
            "publisher identity",
        )?;
        crate::contracts::validate_strict_version("provider version", &self.provider.version)
            .map_err(|_| StateError::Invalid {
                rule: "provider version",
            })?;
        require(
            crate::contracts::is_capability_id(&self.capability_id),
            "capability identity",
        )?;
        require(
            self.interface.kind == crate::ExecutionModeKind::Process,
            "process interface",
        )?;
        require(
            !self.interface.name.is_empty() && !self.interface.protocol.is_empty(),
            "interface identity",
        )?;
        require(
            !self.configuration_schema.is_empty() && self.configuration_schema.len() <= 256,
            "configuration schema",
        )?;
        for value in [
            &self.invocation_digest,
            &self.provider.integrity,
            &self.capability_digest,
            &self.configuration_digest,
            &self.configuration_values_digest,
            &self.subject_lock_digest,
            &self.authority_profile_digest,
            &self.enforcement_evidence_digest,
            &self.grants_digest,
        ] {
            require(valid_digest(value), "sha256 identity")?;
        }
        require(
            self.bindings.inputs.len() + self.bindings.outputs.len() <= MAX_RUN_ARTIFACTS,
            "artifact budget",
        )?;
        self.bindings.validate().map_err(|_| StateError::Invalid {
            rule: "artifact bindings",
        })
    }
}

impl RunState {
    /// Construct the initial in-memory record without touching the filesystem.
    ///
    /// # Errors
    /// Rejects invalid prepared intent.
    pub fn new(plan: RunPlan) -> Result<Self, StateError> {
        let plan_digest = plan.digest()?;
        let steps = plan
            .steps
            .iter()
            .map(|step| RunStepState {
                step_id: step.step_id.clone(),
                attempt: 0,
                status: RunStepStatus::Pending,
                checkpoint: None,
                failure: None,
            })
            .collect();
        Ok(Self {
            schema_version: RUN_STATE_V1.to_owned(),
            sequence: 0,
            previous_digest: String::new(),
            plan_digest,
            plan,
            steps,
            authority_decisions: Vec::new(),
            recovery_decisions: Vec::new(),
        })
    }

    #[must_use]
    pub fn inspection_status(&self) -> RunInspectionStatus {
        if self
            .steps
            .iter()
            .all(|step| step.status == RunStepStatus::Succeeded)
        {
            RunInspectionStatus::Succeeded
        } else if self
            .steps
            .iter()
            .any(|step| step.status == RunStepStatus::Running)
        {
            RunInspectionStatus::RecoveryRequired
        } else if self.steps.iter().any(|step| {
            matches!(
                step.status,
                RunStepStatus::Failed
                    | RunStepStatus::Cancelled
                    | RunStepStatus::Denied
                    | RunStepStatus::Abandoned
            )
        }) {
            RunInspectionStatus::Stopped
        } else {
            RunInspectionStatus::Prepared
        }
    }

    /// Validate structure and correlation without recreating trusted runtime evidence.
    ///
    /// # Errors
    /// Rejects unsupported schemas or inconsistent persisted identities and states.
    pub fn validate(&self) -> Result<(), StateError> {
        schema(&self.schema_version, RUN_STATE_V1)?;
        require(self.sequence < MAX_RUN_SNAPSHOTS, "snapshot budget")?;
        require(
            if self.sequence == 0 {
                self.previous_digest.is_empty()
            } else {
                valid_digest(&self.previous_digest)
            },
            "snapshot predecessor",
        )?;
        require(self.plan.digest()? == self.plan_digest, "plan digest")?;
        require(self.steps.len() == self.plan.steps.len(), "step inventory")?;
        require(
            self.authority_decisions.len() as u64 <= MAX_RUN_SNAPSHOTS
                && self.recovery_decisions.len() as u64 <= MAX_RUN_SNAPSHOTS,
            "decision budget",
        )?;
        require(
            self.steps
                .iter()
                .filter(|s| s.status == RunStepStatus::Running)
                .count()
                <= 1,
            "single active step",
        )?;
        for (step, planned) in self.steps.iter().zip(&self.plan.steps) {
            require(step.step_id == planned.step_id, "step order")?;
            require(step.attempt <= self.sequence, "attempt sequence")?;
            require(
                (step.status == RunStepStatus::Succeeded) == step.checkpoint.is_some(),
                "completion requires checkpoint",
            )?;
            require(
                matches!(
                    step.status,
                    RunStepStatus::Failed | RunStepStatus::Cancelled
                ) == step.failure.is_some(),
                "failure classification",
            )?;
            require(
                (step.status == RunStepStatus::Cancelled)
                    == (step.failure == Some(RunFailure::Cancelled)),
                "cancellation classification",
            )?;
            if matches!(
                step.status,
                RunStepStatus::Running | RunStepStatus::Succeeded | RunStepStatus::Failed
            ) {
                require(step.attempt > 0, "started attempt")?;
            }
            if let Some(checkpoint) = &step.checkpoint {
                checkpoint.validate(&self.plan_digest, planned, step.attempt)?;
            }
            if step.attempt > 0 {
                require(
                    self.authority_decisions.iter().any(|decision| {
                        decision.step_id == step.step_id
                            && decision.attempt == step.attempt
                            && decision.granted
                    }),
                    "recorded launch authority",
                )?;
            }
        }
        self.validate_decisions()
    }

    fn validate_decisions(&self) -> Result<(), StateError> {
        let mut authority_keys = BTreeSet::new();
        for decision in &self.authority_decisions {
            schema(&decision.schema_version, RUN_AUTHORITY_V1)?;
            let index = self.step_index(&decision.step_id)?;
            let context = &self.plan.steps[index].context;
            require(
                !decision.granted || authority_keys.insert((&decision.step_id, decision.attempt)),
                "unique launch authority",
            )?;
            require(
                decision.attempt <= self.steps[index].attempt,
                "authority attempt",
            )?;
            require(
                decision.authorization_id == context.authorization_id
                    && decision.profile_digest == context.authority_profile_digest
                    && decision.enforcement_digest == context.enforcement_evidence_digest
                    && decision.grants_digest == context.grants_digest,
                "authority identity",
            )?;
        }
        let mut recovery_ids = BTreeSet::new();
        for decision in &self.recovery_decisions {
            schema(&decision.schema_version, RUN_RECOVERY_V1)?;
            let index = self.step_index(&decision.step_id)?;
            let context = &self.plan.steps[index].context;
            require(
                valid_id(&decision.decision_id, "decision:")
                    && recovery_ids.insert(&decision.decision_id),
                "recovery identity",
            )?;
            require(
                decision.attempt <= self.steps[index].attempt,
                "recovery attempt",
            )?;
            require(
                decision.authorization_id == context.authorization_id
                    && decision.profile_digest == context.authority_profile_digest,
                "recovery authority",
            )?;
            require(
                decision.acknowledged_uncertain_effects,
                "explicit recovery acknowledgement",
            )?;
        }
        Ok(())
    }

    pub(super) fn step_index(&self, id: &str) -> Result<usize, StateError> {
        self.steps
            .iter()
            .position(|step| step.step_id == id)
            .ok_or(StateError::UnknownStep)
    }
}

impl RunCheckpoint {
    fn validate(
        &self,
        plan_digest: &str,
        planned: &PlannedStep,
        attempt: u64,
    ) -> Result<(), StateError> {
        schema(&self.schema_version, RUN_CHECKPOINT_V1)?;
        require(
            valid_id(&self.checkpoint_id, "checkpoint:"),
            "checkpoint identity",
        )?;
        require(
            self.plan_digest == plan_digest
                && self.context_digest == digest(&planned.context)?
                && self.attempt == attempt,
            "checkpoint context",
        )?;
        schema(&self.validation.schema_version, RUN_VALIDATION_V1)?;
        require(
            self.validation.profile == RUN_VALIDATION_PROFILE,
            "validation profile",
        )?;
        require(
            !self.validation.implementation_version.is_empty()
                && self.validation.implementation_version.len() <= 256,
            "validation implementation",
        )?;
        require(
            valid_digest(&self.validation.implementation_digest)
                && !self.validation.platform.is_empty()
                && self.validation.platform.len() <= 256,
            "validation implementation identity",
        )?;
        require(
            self.validation.invocation_digest == planned.context.invocation_digest,
            "validation invocation",
        )?;
        require(
            valid_digest(&self.validation.execution_digest)
                && self.validation.artifacts_digest == digest(&self.artifacts)?,
            "validation evidence identity",
        )?;
        let bindings = &planned.context.bindings;
        require(
            self.artifacts.len() == bindings.inputs.len() + bindings.outputs.len(),
            "checkpoint artifacts",
        )?;
        let observations = crate::HostArtifactObservationSet {
            schema_version: crate::ARTIFACT_OBSERVATIONS_V1.to_owned(),
            binding_set_id: bindings.binding_set_id.clone(),
            digest_algorithm: crate::SHA256.to_owned(),
            directory_manifest_profile: crate::DIRECTORY_MANIFEST_V1.to_owned(),
            artifacts: self
                .artifacts
                .iter()
                .map(|a| a.observation.clone())
                .collect(),
        };
        observations.validate().map_err(|_| StateError::Invalid {
            rule: "checkpoint observations",
        })?;
        for artifact in &self.artifacts {
            schema(&artifact.schema_version, RUN_ARTIFACT_V1)?;
            let observation = &artifact.observation;
            let matches = match artifact.role {
                RunArtifactRole::Input => bindings.inputs.iter().any(|b| {
                    b.artifact_id == observation.artifact_id
                        && b.port == observation.port
                        && b.locator == observation.locator
                        && b.kind == observation.kind
                        && b.media_type == observation.media_type
                        && b.expected_digest == observation.digest
                }),
                RunArtifactRole::Output => bindings.outputs.iter().any(|b| {
                    b.artifact_id == observation.artifact_id
                        && b.port == observation.port
                        && b.locator == observation.locator
                        && b.kind == observation.kind
                        && b.media_type == observation.media_type
                }),
            };
            require(matches, "checkpoint artifact binding")?;
        }
        Ok(())
    }
}
