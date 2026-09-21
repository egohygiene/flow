//! Closed Rust models for the six `flow.extension-*/v1` contracts.

use std::collections::{BTreeMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub const EXTENSION_MANIFEST_V1: &str = "flow.extension-manifest/v1";
pub const EXTENSION_LOCK_V1: &str = "flow.extension-lock/v1";
pub const EXTENSION_INVOCATION_V1: &str = "flow.extension-invocation/v1";
pub const EXTENSION_EVENT_V1: &str = "flow.extension-event/v1";
pub const EXTENSION_RESULT_V1: &str = "flow.extension-result/v1";
pub const EXTENSION_RESOLUTION_V1: &str = "flow.extension-resolution/v1";

/// A schema or cross-field invariant violation.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{path}: {message}")]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl ValidationError {
    pub(crate) fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

/// Provider-owned extension declaration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionManifest {
    pub schema_version: String,
    pub extension_id: String,
    pub version: String,
    pub publisher: Publisher,
    pub integrity: Integrity,
    pub compatibility: Compatibility,
    pub inspection: Inspection,
    pub domain_ownership: Vec<Domain>,
    pub capabilities: Vec<Capability>,
    pub execution_modes: Vec<ExecutionMode>,
    pub requested_permissions: Permissions,
    pub checkpoint: Checkpoint,
    pub replacement: Replacement,
    pub observer_hooks: Vec<ObserverHook>,
    pub transform_hooks: Vec<TransformHook>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Publisher {
    pub id: String,
    pub display_name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Integrity {
    pub algorithm: IntegrityAlgorithm,
    pub value: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IntegrityAlgorithm {
    Sha256,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Compatibility {
    pub flow_version_requirement: String,
    pub contract_families: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Inspection {
    pub version_source: VersionSource,
    pub capability_source: CapabilitySource,
    pub health_mode: HealthMode,
    pub health_capability_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VersionSource {
    Manifest,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilitySource {
    Manifest,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HealthMode {
    ManifestOnly,
    Invocation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Domain {
    Flow,
    Aniflow,
    Optiflow,
    Renderflow,
    ThirdParty,
}

impl Domain {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Flow => "flow",
            Self::Aniflow => "aniflow",
            Self::Optiflow => "optiflow",
            Self::Renderflow => "renderflow",
            Self::ThirdParty => "third-party",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Capability {
    pub capability_id: String,
    pub domain: Domain,
    pub configuration_schema: String,
    pub accepts: Vec<String>,
    pub produces: Vec<String>,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
    pub deterministic: bool,
    pub cacheable: bool,
    pub loss: Loss,
    pub content_changes: bool,
    pub effects: Vec<Effect>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Loss {
    None,
    Declared,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Effect {
    FilesystemRead,
    FilesystemWrite,
    EnvironmentRead,
    Subprocess,
    Network,
    Ai,
    Gpu,
    SourceMutation,
    Destructive,
    Sign,
    Publish,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionMode {
    pub name: String,
    pub kind: ExecutionModeKind,
    pub protocol: String,
    pub entrypoint: String,
    pub limits: ExecutionLimits,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionModeKind {
    InProcess,
    Process,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
/// Declared execution policy metadata.
///
/// The current injected in-process seam correlates these values with the
/// resolved declaration but does not enforce time, cancellation, or output
/// bounds. The host-neutral process seam checks captured stdout/stderr lengths;
/// it does not enforce limits against a running child.
pub struct ExecutionLimits {
    pub timeout_ms: u64,
    pub max_stdout_bytes: u64,
    pub max_stderr_bytes: u64,
    pub cancellation_grace_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct Permissions {
    pub filesystem_read: Vec<String>,
    pub filesystem_write: Vec<String>,
    pub environment_read: Vec<String>,
    pub subprocesses: Vec<String>,
    pub network_hosts: Vec<String>,
    pub ai_providers: Vec<String>,
    pub gpu: bool,
    pub source_mutation: bool,
    pub destructive: bool,
    pub sign: bool,
    pub publish: bool,
}

impl Permissions {
    /// A permission value must be explicitly included in the operator grant.
    #[must_use]
    pub fn allows(&self, requested: &Self) -> bool {
        is_subset(&requested.filesystem_read, &self.filesystem_read)
            && is_subset(&requested.filesystem_write, &self.filesystem_write)
            && is_subset(&requested.environment_read, &self.environment_read)
            && is_subset(&requested.subprocesses, &self.subprocesses)
            && is_subset(&requested.network_hosts, &self.network_hosts)
            && is_subset(&requested.ai_providers, &self.ai_providers)
            && (!requested.gpu || self.gpu)
            && (!requested.source_mutation || self.source_mutation)
            && (!requested.destructive || self.destructive)
            && (!requested.sign || self.sign)
            && (!requested.publish || self.publish)
    }

    #[must_use]
    pub fn allows_effect(&self, effect: Effect) -> bool {
        match effect {
            Effect::FilesystemRead => !self.filesystem_read.is_empty(),
            Effect::FilesystemWrite => !self.filesystem_write.is_empty(),
            Effect::EnvironmentRead => !self.environment_read.is_empty(),
            Effect::Subprocess => !self.subprocesses.is_empty(),
            Effect::Network => !self.network_hosts.is_empty(),
            Effect::Ai => !self.ai_providers.is_empty(),
            Effect::Gpu => self.gpu,
            Effect::SourceMutation => self.source_mutation,
            Effect::Destructive => self.destructive,
            Effect::Sign => self.sign,
            Effect::Publish => self.publish,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub mode: CheckpointMode,
    pub contract_family: String,
    pub compatibility_keys: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckpointMode {
    None,
    Provider,
    Flow,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Replacement {
    pub replaces: Vec<String>,
    pub fallbacks_for: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObserverHook {
    pub hook_id: String,
    pub lifecycle_points: Vec<LifecyclePoint>,
    pub read_only: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LifecyclePoint {
    Discover,
    Inspect,
    Plan,
    Authorize,
    Execute,
    Validate,
    CommitEvidence,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TransformHook {
    pub hook_id: String,
    pub lifecycle_points: Vec<TransformLifecyclePoint>,
    pub capability_id: String,
    pub accepts: Vec<String>,
    pub produces: Vec<String>,
    pub content_changes: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransformLifecyclePoint {
    Inspect,
    Execute,
    Validate,
}

/// Operator-owned trust, grant, and precedence policy.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionLock {
    pub schema_version: String,
    pub flow_version: String,
    pub lock_id: String,
    pub extensions: Vec<LockedExtension>,
    pub capability_resolution: Vec<CapabilityResolution>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedExtension {
    pub extension_id: String,
    pub version: String,
    pub publisher_id: String,
    pub integrity: Integrity,
    pub discovery: Discovery,
    pub enabled: bool,
    pub trust: Trust,
    pub granted_permissions: Permissions,
    pub precedence: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Discovery {
    pub kind: DiscoveryKind,
    pub location: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiscoveryKind {
    ConfiguredPath,
    Package,
    Embedded,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Trust {
    Trusted,
    Sandboxed,
    Disabled,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityResolution {
    pub capability_id: String,
    pub ordered_extensions: Vec<String>,
    pub fallback: FallbackPolicy,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FallbackPolicy {
    Forbidden,
    BeforeEffects,
    ExplicitResume,
}

/// Flow-owned request envelope for one extension call.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionInvocation {
    pub schema_version: String,
    pub invocation_id: String,
    pub run_id: String,
    pub phase: InvocationPhase,
    pub extension: InvocationExtension,
    pub capability_id: String,
    pub interface: InvocationInterface,
    pub input_artifacts: Vec<InputArtifact>,
    pub expected_output_types: Vec<String>,
    pub configuration: Configuration,
    pub authorization: Authorization,
    pub limits: ExecutionLimits,
    pub checkpoint_refs: Vec<String>,
    pub secret_handles: Vec<String>,
    pub cancellation_id: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InvocationPhase {
    Inspect,
    Plan,
    Execute,
    Validate,
}

impl InvocationPhase {
    #[must_use]
    pub const fn lifecycle_point(self) -> LifecyclePoint {
        match self {
            Self::Inspect => LifecyclePoint::Inspect,
            Self::Plan => LifecyclePoint::Plan,
            Self::Execute => LifecyclePoint::Execute,
            Self::Validate => LifecyclePoint::Validate,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationExtension {
    pub extension_id: String,
    pub version: String,
    pub publisher_id: String,
    pub integrity: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationInterface {
    pub kind: ExecutionModeKind,
    pub name: String,
    pub protocol: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InputArtifact {
    pub artifact_id: String,
    pub digest: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    pub schema_id: String,
    pub digest: String,
    pub values: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Authorization {
    pub authorization_id: String,
    pub lock_id: String,
    pub grants_digest: String,
}

/// Provider-contributed lifecycle observation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionEvent {
    pub schema_version: String,
    pub event_id: String,
    pub run_id: String,
    pub invocation_id: String,
    pub sequence: u64,
    pub phase: LifecyclePoint,
    pub kind: EventKind,
    pub state: EventState,
    pub progress: Progress,
    pub diagnostics: Vec<Diagnostic>,
    pub artifact_refs: Vec<String>,
    pub checkpoint_refs: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventKind {
    PhaseStarted,
    Progress,
    Diagnostic,
    ArtifactProduced,
    Checkpoint,
    PhaseCompleted,
    PhaseFailed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventState {
    Planned,
    Running,
    Validating,
    Produced,
    Reused,
    Skipped,
    Blocked,
    Unavailable,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    pub completed: u64,
    pub total: u64,
    pub unit: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub redacted: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// Provider-contributed terminal evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionResult {
    pub schema_version: String,
    pub run_id: String,
    pub invocation_id: String,
    pub extension_id: String,
    pub extension_version: String,
    pub extension_integrity: String,
    pub capability_id: String,
    pub configuration_digest: String,
    pub authorization_id: String,
    pub outcome: Outcome,
    pub partial_result: bool,
    pub consumed_artifacts: Vec<String>,
    pub produced_artifacts: Vec<String>,
    pub validations: Vec<ValidationEvidence>,
    pub provenance: Vec<ProvenanceEvidence>,
    pub diagnostics: Vec<Diagnostic>,
    pub failure: Failure,
    pub checkpoint_refs: Vec<String>,
    pub explanation: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    Produced,
    Reused,
    Skipped,
    Blocked,
    Unavailable,
    Failed,
    Cancelled,
}

impl Outcome {
    #[must_use]
    pub const fn event_state(self) -> EventState {
        match self {
            Self::Produced => EventState::Produced,
            Self::Reused => EventState::Reused,
            Self::Skipped => EventState::Skipped,
            Self::Blocked => EventState::Blocked,
            Self::Unavailable => EventState::Unavailable,
            Self::Failed => EventState::Failed,
            Self::Cancelled => EventState::Cancelled,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationEvidence {
    pub validator: String,
    pub status: ValidationStatus,
    pub evidence: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationStatus {
    Passed,
    Failed,
    NotRun,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceEvidence {
    pub kind: ProvenanceKind,
    pub value: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProvenanceKind {
    Extension,
    Configuration,
    Authorization,
    Process,
    Artifact,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Failure {
    pub classification: FailureClassification,
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FailureClassification {
    None,
    Malformed,
    Incompatible,
    Unauthorized,
    Unavailable,
    Timeout,
    Cancelled,
    Provider,
    Validation,
    Contract,
    Internal,
}

/// Versioned Flow-owned explanation of a resolution decision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionResolution {
    pub schema_version: String,
    pub case_id: String,
    pub requested_capability: String,
    pub candidates: Vec<ResolutionCandidate>,
    pub result: ResolutionResult,
    pub selected_extension_ids: Vec<String>,
    pub fallback_order: Vec<String>,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResolutionCandidate {
    pub extension_id: String,
    pub version: String,
    pub publisher_id: String,
    pub integrity: String,
    pub precedence: u64,
    pub compatible: bool,
    pub authorized: bool,
    pub available: bool,
    pub reasons: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResolutionResult {
    Selected,
    NoCompatibleProvider,
    Blocked,
    Conflict,
    Malformed,
}

impl ExtensionManifest {
    /// Validate schema-level shapes and manifest cross-field invariants.
    ///
    /// # Errors
    ///
    /// Returns the first deterministic schema or cross-field violation.
    #[allow(clippy::too_many_lines)]
    pub fn validate(&self) -> Result<(), ValidationError> {
        expect_schema(
            "manifest.schema_version",
            &self.schema_version,
            EXTENSION_MANIFEST_V1,
        )?;
        ensure(
            is_extension_id(&self.extension_id),
            "manifest.extension_id",
            "must be a lowercase qualified extension identifier",
        )?;
        validate_strict_version("manifest.version", &self.version)?;
        ensure(
            is_publisher_id(&self.publisher.id),
            "manifest.publisher.id",
            "must be a lowercase publisher identifier",
        )?;
        nonempty(
            "manifest.publisher.display_name",
            &self.publisher.display_name,
        )?;
        validate_integrity("manifest.integrity", &self.integrity)?;
        parse_flow_version_requirement(&self.compatibility.flow_version_requirement)?;
        ensure(
            !self.compatibility.contract_families.is_empty(),
            "manifest.compatibility.contract_families",
            "must contain at least one contract family",
        )?;
        unique(
            "manifest.compatibility.contract_families",
            &self.compatibility.contract_families,
        )?;
        for (index, family) in self.compatibility.contract_families.iter().enumerate() {
            ensure(
                contract_family_major(family).is_some(),
                format!("manifest.compatibility.contract_families[{index}]"),
                "must use flow.<family>/v<positive-major> syntax",
            )?;
        }

        unique(
            "manifest.inspection.health_capability_ids",
            &self.inspection.health_capability_ids,
        )?;
        ensure(
            !self.domain_ownership.is_empty(),
            "manifest.domain_ownership",
            "must contain at least one domain",
        )?;
        unique("manifest.domain_ownership", &self.domain_ownership)?;
        ensure(
            !self.capabilities.is_empty(),
            "manifest.capabilities",
            "must contain at least one capability",
        )?;
        unique_by(
            "manifest.capabilities",
            &self.capabilities,
            |capability| &capability.capability_id,
            "capability identifiers must be unique",
        )?;
        for (index, capability) in self.capabilities.iter().enumerate() {
            validate_capability(capability, index)?;
            ensure(
                self.domain_ownership.contains(&capability.domain),
                format!("manifest.capabilities[{index}].domain"),
                "capability uses an undeclared domain",
            )?;
            for effect in &capability.effects {
                ensure(
                    self.requested_permissions.allows_effect(*effect),
                    format!("manifest.capabilities[{index}].effects"),
                    format!("effect {effect:?} has no matching permission request"),
                )?;
            }
        }

        match self.inspection.health_mode {
            HealthMode::ManifestOnly => ensure(
                self.inspection.health_capability_ids.is_empty(),
                "manifest.inspection.health_capability_ids",
                "manifest-only health cannot invoke capabilities",
            )?,
            HealthMode::Invocation => {
                ensure(
                    !self.inspection.health_capability_ids.is_empty(),
                    "manifest.inspection.health_capability_ids",
                    "invocation health must reference declared capabilities",
                )?;
                for capability_id in &self.inspection.health_capability_ids {
                    ensure(
                        self.capability(capability_id).is_some(),
                        "manifest.inspection.health_capability_ids",
                        format!("unknown health capability {capability_id}"),
                    )?;
                }
            }
        }

        ensure(
            !self.execution_modes.is_empty(),
            "manifest.execution_modes",
            "must contain at least one execution mode",
        )?;
        unique_by(
            "manifest.execution_modes",
            &self.execution_modes,
            |mode| &mode.name,
            "execution mode names must be unique",
        )?;
        for (index, mode) in self.execution_modes.iter().enumerate() {
            validate_execution_mode(mode, index)?;
        }
        validate_permissions(
            "manifest.requested_permissions",
            &self.requested_permissions,
        )?;
        nonempty(
            "manifest.checkpoint.contract_family",
            &self.checkpoint.contract_family,
        )?;
        unique(
            "manifest.checkpoint.compatibility_keys",
            &self.checkpoint.compatibility_keys,
        )?;
        for (index, key) in self.checkpoint.compatibility_keys.iter().enumerate() {
            nonempty(
                format!("manifest.checkpoint.compatibility_keys[{index}]"),
                key,
            )?;
        }
        validate_string_list("manifest.replacement.replaces", &self.replacement.replaces)?;
        validate_string_list(
            "manifest.replacement.fallbacks_for",
            &self.replacement.fallbacks_for,
        )?;

        let mut hook_ids = HashSet::new();
        for (index, hook) in self.observer_hooks.iter().enumerate() {
            nonempty(
                format!("manifest.observer_hooks[{index}].hook_id"),
                &hook.hook_id,
            )?;
            ensure(
                hook_ids.insert(hook.hook_id.as_str()),
                "manifest.observer_hooks",
                "hook identifiers must be unique",
            )?;
            ensure(
                !hook.lifecycle_points.is_empty(),
                format!("manifest.observer_hooks[{index}].lifecycle_points"),
                "must contain at least one lifecycle point",
            )?;
            unique(
                format!("manifest.observer_hooks[{index}].lifecycle_points"),
                &hook.lifecycle_points,
            )?;
            ensure(
                hook.read_only,
                format!("manifest.observer_hooks[{index}].read_only"),
                "observer hooks must be read-only",
            )?;
        }
        for (index, hook) in self.transform_hooks.iter().enumerate() {
            nonempty(
                format!("manifest.transform_hooks[{index}].hook_id"),
                &hook.hook_id,
            )?;
            ensure(
                hook_ids.insert(hook.hook_id.as_str()),
                "manifest.transform_hooks",
                "hook identifiers must be unique",
            )?;
            ensure(
                !hook.lifecycle_points.is_empty(),
                format!("manifest.transform_hooks[{index}].lifecycle_points"),
                "must contain at least one lifecycle point",
            )?;
            unique(
                format!("manifest.transform_hooks[{index}].lifecycle_points"),
                &hook.lifecycle_points,
            )?;
            ensure(
                hook.content_changes,
                format!("manifest.transform_hooks[{index}].content_changes"),
                "transform hooks must change content",
            )?;
            ensure(
                !hook.accepts.is_empty() && !hook.produces.is_empty(),
                format!("manifest.transform_hooks[{index}]"),
                "transform hooks must declare accepted and produced types",
            )?;
            validate_string_list(
                &format!("manifest.transform_hooks[{index}].accepts"),
                &hook.accepts,
            )?;
            validate_string_list(
                &format!("manifest.transform_hooks[{index}].produces"),
                &hook.produces,
            )?;
            let capability = self.capability(&hook.capability_id).ok_or_else(|| {
                ValidationError::new(
                    format!("manifest.transform_hooks[{index}].capability_id"),
                    "transform hook references an unknown capability",
                )
            })?;
            ensure(
                capability.content_changes,
                format!("manifest.transform_hooks[{index}].capability_id"),
                "transform hook capability must declare content changes",
            )?;
            ensure(
                hook.accepts == capability.accepts && hook.produces == capability.produces,
                format!("manifest.transform_hooks[{index}]"),
                "transform hook artifact types must match its capability",
            )?;
        }
        Ok(())
    }

    #[must_use]
    pub fn capability(&self, capability_id: &str) -> Option<&Capability> {
        self.capabilities
            .iter()
            .find(|capability| capability.capability_id == capability_id)
    }

    #[must_use]
    pub fn execution_mode(&self, name: &str, kind: ExecutionModeKind) -> Option<&ExecutionMode> {
        self.execution_modes
            .iter()
            .find(|mode| mode.name == name && mode.kind == kind)
    }
}

impl ExtensionLock {
    /// Validate lock shapes, identities, and policy references.
    ///
    /// # Errors
    ///
    /// Returns the first deterministic schema or cross-field violation.
    pub fn validate(&self) -> Result<(), ValidationError> {
        expect_schema(
            "lock.schema_version",
            &self.schema_version,
            EXTENSION_LOCK_V1,
        )?;
        validate_strict_version("lock.flow_version", &self.flow_version)?;
        ensure(
            has_prefixed_id(&self.lock_id, "lock:"),
            "lock.lock_id",
            "must be a lowercase lock identifier",
        )?;
        ensure(
            !self.extensions.is_empty(),
            "lock.extensions",
            "must contain at least one locked extension",
        )?;
        unique_by(
            "lock.extensions",
            &self.extensions,
            |extension| &extension.extension_id,
            "locked extension identifiers must be unique",
        )?;
        for (index, extension) in self.extensions.iter().enumerate() {
            nonempty(
                format!("lock.extensions[{index}].extension_id"),
                &extension.extension_id,
            )?;
            nonempty(
                format!("lock.extensions[{index}].version"),
                &extension.version,
            )?;
            nonempty(
                format!("lock.extensions[{index}].publisher_id"),
                &extension.publisher_id,
            )?;
            validate_integrity(
                &format!("lock.extensions[{index}].integrity"),
                &extension.integrity,
            )?;
            nonempty(
                format!("lock.extensions[{index}].discovery.location"),
                &extension.discovery.location,
            )?;
            validate_permissions(
                &format!("lock.extensions[{index}].granted_permissions"),
                &extension.granted_permissions,
            )?;
        }
        unique_by(
            "lock.capability_resolution",
            &self.capability_resolution,
            |resolution| &resolution.capability_id,
            "capability resolution identifiers must be unique",
        )?;
        let known: HashSet<&str> = self
            .extensions
            .iter()
            .map(|extension| extension.extension_id.as_str())
            .collect();
        let precedence_by_id: BTreeMap<&str, u64> = self
            .extensions
            .iter()
            .map(|extension| (extension.extension_id.as_str(), extension.precedence))
            .collect();
        for (index, resolution) in self.capability_resolution.iter().enumerate() {
            nonempty(
                format!("lock.capability_resolution[{index}].capability_id"),
                &resolution.capability_id,
            )?;
            ensure(
                !resolution.ordered_extensions.is_empty(),
                format!("lock.capability_resolution[{index}].ordered_extensions"),
                "must contain at least one extension",
            )?;
            unique(
                format!("lock.capability_resolution[{index}].ordered_extensions"),
                &resolution.ordered_extensions,
            )?;
            for extension_id in &resolution.ordered_extensions {
                ensure(
                    known.contains(extension_id.as_str()),
                    format!("lock.capability_resolution[{index}].ordered_extensions"),
                    format!("references unlocked extension {extension_id}"),
                )?;
            }
            let mut previous_precedence = None;
            for extension_id in &resolution.ordered_extensions {
                let Some(precedence) = precedence_by_id.get(extension_id.as_str()).copied() else {
                    continue;
                };
                ensure(
                    previous_precedence.is_none_or(|previous| previous >= precedence),
                    format!("lock.capability_resolution[{index}].ordered_extensions"),
                    "must be ordered by non-increasing locked precedence",
                )?;
                previous_precedence = Some(precedence);
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn extension(&self, extension_id: &str) -> Option<&LockedExtension> {
        self.extensions
            .iter()
            .find(|extension| extension.extension_id == extension_id)
    }

    #[must_use]
    pub fn resolution_for(&self, capability_id: &str) -> Option<&CapabilityResolution> {
        self.capability_resolution
            .iter()
            .find(|resolution| resolution.capability_id == capability_id)
    }
}

impl ExtensionInvocation {
    /// Validate the invocation envelope before any extension code runs.
    ///
    /// # Errors
    ///
    /// Returns the first deterministic schema or field violation.
    pub fn validate(&self) -> Result<(), ValidationError> {
        expect_schema(
            "invocation.schema_version",
            &self.schema_version,
            EXTENSION_INVOCATION_V1,
        )?;
        ensure(
            has_prefixed_id(&self.invocation_id, "invocation:"),
            "invocation.invocation_id",
            "must be a lowercase invocation identifier",
        )?;
        ensure(
            has_prefixed_id(&self.run_id, "run:"),
            "invocation.run_id",
            "must be a lowercase run identifier",
        )?;
        nonempty(
            "invocation.extension.extension_id",
            &self.extension.extension_id,
        )?;
        nonempty("invocation.extension.version", &self.extension.version)?;
        nonempty(
            "invocation.extension.publisher_id",
            &self.extension.publisher_id,
        )?;
        validate_digest("invocation.extension.integrity", &self.extension.integrity)?;
        nonempty("invocation.capability_id", &self.capability_id)?;
        nonempty("invocation.interface.name", &self.interface.name)?;
        expect_schema(
            "invocation.interface.protocol",
            &self.interface.protocol,
            EXTENSION_INVOCATION_V1,
        )?;
        for (index, artifact) in self.input_artifacts.iter().enumerate() {
            nonempty(
                format!("invocation.input_artifacts[{index}].artifact_id"),
                &artifact.artifact_id,
            )?;
            validate_digest(
                &format!("invocation.input_artifacts[{index}].digest"),
                &artifact.digest,
            )?;
        }
        unique_by(
            "invocation.input_artifacts",
            &self.input_artifacts,
            |artifact| artifact.artifact_id.as_str(),
            "input artifact identifiers must be unique",
        )?;
        validate_string_list(
            "invocation.expected_output_types",
            &self.expected_output_types,
        )?;
        nonempty(
            "invocation.configuration.schema_id",
            &self.configuration.schema_id,
        )?;
        validate_digest(
            "invocation.configuration.digest",
            &self.configuration.digest,
        )?;
        nonempty(
            "invocation.authorization.authorization_id",
            &self.authorization.authorization_id,
        )?;
        nonempty(
            "invocation.authorization.lock_id",
            &self.authorization.lock_id,
        )?;
        validate_digest(
            "invocation.authorization.grants_digest",
            &self.authorization.grants_digest,
        )?;
        validate_limits("invocation.limits", &self.limits)?;
        validate_string_list("invocation.checkpoint_refs", &self.checkpoint_refs)?;
        unique("invocation.secret_handles", &self.secret_handles)?;
        for (index, handle) in self.secret_handles.iter().enumerate() {
            ensure(
                has_prefixed_id(handle, "secret:"),
                format!("invocation.secret_handles[{index}]"),
                "must be a lowercase secret handle",
            )?;
        }
        nonempty("invocation.cancellation_id", &self.cancellation_id)?;
        Ok(())
    }
}

impl ExtensionEvent {
    /// Validate one event independently of stream correlation.
    ///
    /// # Errors
    ///
    /// Returns the first deterministic schema or field violation.
    pub fn validate(&self) -> Result<(), ValidationError> {
        expect_schema(
            "event.schema_version",
            &self.schema_version,
            EXTENSION_EVENT_V1,
        )?;
        nonempty("event.event_id", &self.event_id)?;
        nonempty("event.run_id", &self.run_id)?;
        nonempty("event.invocation_id", &self.invocation_id)?;
        ensure(
            self.progress.completed <= self.progress.total,
            "event.progress",
            "completed work cannot exceed total work",
        )?;
        nonempty("event.progress.unit", &self.progress.unit)?;
        for (index, diagnostic) in self.diagnostics.iter().enumerate() {
            validate_diagnostic(diagnostic, &format!("event.diagnostics[{index}]"))?;
        }
        validate_string_list("event.artifact_refs", &self.artifact_refs)?;
        validate_string_list("event.checkpoint_refs", &self.checkpoint_refs)?;
        Ok(())
    }
}

impl ExtensionResult {
    /// Validate terminal evidence, including outcome/failure consistency.
    ///
    /// # Errors
    ///
    /// Returns the first deterministic schema or cross-field violation.
    pub fn validate(&self) -> Result<(), ValidationError> {
        expect_schema(
            "result.schema_version",
            &self.schema_version,
            EXTENSION_RESULT_V1,
        )?;
        nonempty("result.run_id", &self.run_id)?;
        nonempty("result.invocation_id", &self.invocation_id)?;
        nonempty("result.extension_id", &self.extension_id)?;
        nonempty("result.extension_version", &self.extension_version)?;
        validate_digest("result.extension_integrity", &self.extension_integrity)?;
        nonempty("result.capability_id", &self.capability_id)?;
        validate_digest("result.configuration_digest", &self.configuration_digest)?;
        nonempty("result.authorization_id", &self.authorization_id)?;
        validate_string_list("result.consumed_artifacts", &self.consumed_artifacts)?;
        validate_string_list("result.produced_artifacts", &self.produced_artifacts)?;
        for (index, validation) in self.validations.iter().enumerate() {
            nonempty(
                format!("result.validations[{index}].validator"),
                &validation.validator,
            )?;
            nonempty(
                format!("result.validations[{index}].evidence"),
                &validation.evidence,
            )?;
        }
        if matches!(self.outcome, Outcome::Produced | Outcome::Reused) {
            ensure(
                self.validations
                    .iter()
                    .all(|validation| validation.status == ValidationStatus::Passed),
                "result.validations",
                "produced or reused outcomes require every reported validation to pass",
            )?;
        } else if self.outcome == Outcome::Skipped {
            ensure(
                self.validations
                    .iter()
                    .all(|validation| validation.status != ValidationStatus::Failed),
                "result.validations",
                "a skipped outcome cannot contain failed validation evidence",
            )?;
        }
        for (index, evidence) in self.provenance.iter().enumerate() {
            nonempty(format!("result.provenance[{index}].value"), &evidence.value)?;
        }
        for (index, diagnostic) in self.diagnostics.iter().enumerate() {
            validate_diagnostic(diagnostic, &format!("result.diagnostics[{index}]"))?;
        }
        validate_string_list("result.checkpoint_refs", &self.checkpoint_refs)?;
        nonempty("result.explanation", &self.explanation)?;
        validate_outcome_failure(self.outcome, &self.failure)?;
        Ok(())
    }
}

impl ExtensionResolution {
    /// Validate versioned selection or rejection evidence.
    ///
    /// # Errors
    ///
    /// Returns the first deterministic schema or cross-field violation.
    #[allow(clippy::too_many_lines)]
    pub fn validate(&self) -> Result<(), ValidationError> {
        expect_schema(
            "resolution.schema_version",
            &self.schema_version,
            EXTENSION_RESOLUTION_V1,
        )?;
        nonempty("resolution.case_id", &self.case_id)?;
        nonempty(
            "resolution.requested_capability",
            &self.requested_capability,
        )?;
        for (index, candidate) in self.candidates.iter().enumerate() {
            nonempty(
                format!("resolution.candidates[{index}].extension_id"),
                &candidate.extension_id,
            )?;
            nonempty(
                format!("resolution.candidates[{index}].version"),
                &candidate.version,
            )?;
            nonempty(
                format!("resolution.candidates[{index}].publisher_id"),
                &candidate.publisher_id,
            )?;
            validate_digest(
                &format!("resolution.candidates[{index}].integrity"),
                &candidate.integrity,
            )?;
            for (reason_index, reason) in candidate.reasons.iter().enumerate() {
                nonempty(
                    format!("resolution.candidates[{index}].reasons[{reason_index}]"),
                    reason,
                )?;
            }
        }
        unique_by(
            "resolution.candidates",
            &self.candidates,
            |candidate| &candidate.extension_id,
            "candidate extension identifiers must be unique",
        )?;
        unique(
            "resolution.selected_extension_ids",
            &self.selected_extension_ids,
        )?;
        unique("resolution.fallback_order", &self.fallback_order)?;
        let candidate_ids: HashSet<&str> = self
            .candidates
            .iter()
            .map(|candidate| candidate.extension_id.as_str())
            .collect();
        for extension_id in &self.fallback_order {
            ensure(
                candidate_ids.contains(extension_id.as_str()),
                "resolution.fallback_order",
                format!("references unknown candidate {extension_id}"),
            )?;
        }
        ensure(
            !self.reasons.is_empty(),
            "resolution.reasons",
            "must contain at least one reason",
        )?;
        for (index, reason) in self.reasons.iter().enumerate() {
            nonempty(format!("resolution.reasons[{index}]"), reason)?;
        }

        let eligible: Vec<&ResolutionCandidate> = self
            .candidates
            .iter()
            .filter(|candidate| candidate.compatible && candidate.authorized && candidate.available)
            .collect();
        if self.result != ResolutionResult::Selected {
            ensure(
                self.fallback_order.is_empty(),
                "resolution.fallback_order",
                "non-selected resolution cannot record fallback order",
            )?;
        }
        match self.result {
            ResolutionResult::Selected => {
                ensure(
                    self.selected_extension_ids.len() == 1,
                    "resolution.selected_extension_ids",
                    "selected resolution must name exactly one extension",
                )?;
                let selected_id = &self.selected_extension_ids[0];
                let selected = eligible
                    .iter()
                    .find(|candidate| candidate.extension_id == *selected_id)
                    .ok_or_else(|| {
                        ValidationError::new(
                            "resolution.selected_extension_ids",
                            "selected extension must be compatible, authorized, and available",
                        )
                    })?;
                let max_precedence = eligible
                    .iter()
                    .map(|candidate| candidate.precedence)
                    .max()
                    .unwrap_or_default();
                ensure(
                    selected.precedence == max_precedence,
                    "resolution.selected_extension_ids",
                    "selected extension must have the highest eligible precedence",
                )?;
                ensure(
                    eligible
                        .iter()
                        .filter(|candidate| candidate.precedence == max_precedence)
                        .count()
                        == 1,
                    "resolution.selected_extension_ids",
                    "selected extension precedence must be unique",
                )?;
                ensure(
                    self.fallback_order
                        .last()
                        .is_none_or(|extension_id| extension_id == selected_id),
                    "resolution.fallback_order",
                    "selected fallback evidence must end with the selected extension",
                )?;
                if !self.fallback_order.is_empty() {
                    ensure(
                        self.fallback_order.len() >= 2,
                        "resolution.fallback_order",
                        "fallback evidence must include an unavailable predecessor and the selected extension",
                    )?;
                    let by_id: BTreeMap<&str, &ResolutionCandidate> = self
                        .candidates
                        .iter()
                        .map(|candidate| (candidate.extension_id.as_str(), candidate))
                        .collect();
                    for extension_id in
                        &self.fallback_order[..self.fallback_order.len().saturating_sub(1)]
                    {
                        let candidate = by_id.get(extension_id.as_str()).ok_or_else(|| {
                            ValidationError::new(
                                "resolution.fallback_order",
                                "fallback order references an unknown candidate",
                            )
                        })?;
                        ensure(
                            candidate.compatible && candidate.authorized && !candidate.available,
                            "resolution.fallback_order",
                            "every fallback predecessor must be compatible, authorized, and unavailable",
                        )?;
                    }
                    for pair in self.fallback_order.windows(2) {
                        let left = by_id.get(pair[0].as_str()).ok_or_else(|| {
                            ValidationError::new(
                                "resolution.fallback_order",
                                "fallback order references an unknown candidate",
                            )
                        })?;
                        let right = by_id.get(pair[1].as_str()).ok_or_else(|| {
                            ValidationError::new(
                                "resolution.fallback_order",
                                "fallback order references an unknown candidate",
                            )
                        })?;
                        ensure(
                            left.precedence >= right.precedence,
                            "resolution.fallback_order",
                            "fallback precedence must be non-increasing",
                        )?;
                    }
                }
            }
            ResolutionResult::NoCompatibleProvider => {
                ensure(
                    self.selected_extension_ids.is_empty(),
                    "resolution.selected_extension_ids",
                    "non-selected resolution cannot name an extension",
                )?;
                ensure(
                    !self.candidates.iter().any(|candidate| candidate.compatible),
                    "resolution.result",
                    "no-compatible-provider requires every candidate to be incompatible",
                )?;
            }
            ResolutionResult::Blocked => {
                ensure(
                    self.selected_extension_ids.is_empty(),
                    "resolution.selected_extension_ids",
                    "non-selected resolution cannot name an extension",
                )?;
                ensure(
                    self.candidates.iter().any(|candidate| candidate.compatible)
                        && eligible.is_empty(),
                    "resolution.result",
                    "blocked requires compatible but ineligible candidates",
                )?;
            }
            ResolutionResult::Conflict => {
                ensure(
                    self.selected_extension_ids.is_empty(),
                    "resolution.selected_extension_ids",
                    "non-selected resolution cannot name an extension",
                )?;
                let max_precedence = eligible.iter().map(|candidate| candidate.precedence).max();
                ensure(
                    max_precedence.is_some_and(|precedence| {
                        eligible
                            .iter()
                            .filter(|candidate| candidate.precedence == precedence)
                            .count()
                            > 1
                    }),
                    "resolution.result",
                    "conflict requires equal-precedence eligible candidates",
                )?;
            }
            ResolutionResult::Malformed => {
                ensure(
                    self.selected_extension_ids.is_empty() && self.candidates.is_empty(),
                    "resolution.result",
                    "malformed resolution cannot expose executable candidates",
                )?;
            }
        }
        Ok(())
    }
}

/// Parse the documented Rust-semver comparator grammar.
///
/// Multiple comparators must be comma-separated, for example
/// `>=0.1.0, <0.2.0`.
///
/// # Errors
///
/// Returns an error when the requirement is empty or is not accepted by the
/// Rust `semver` comparator grammar.
pub fn parse_flow_version_requirement(requirement: &str) -> Result<VersionReq, ValidationError> {
    nonempty(
        "manifest.compatibility.flow_version_requirement",
        requirement,
    )?;
    VersionReq::parse(requirement).map_err(|error| {
        ValidationError::new(
            "manifest.compatibility.flow_version_requirement",
            format!("invalid comma-separated semver requirement: {error}"),
        )
    })
}

/// Return the positive contract major for a syntactically valid Flow family.
#[must_use]
pub fn contract_family_major(family: &str) -> Option<u64> {
    let rest = family.strip_prefix("flow.")?;
    let (name, major) = rest.rsplit_once("/v")?;
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
        || major.starts_with('0')
        || major.is_empty()
        || !major.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    major.parse().ok()
}

fn validate_capability(capability: &Capability, index: usize) -> Result<(), ValidationError> {
    ensure(
        is_capability_id(&capability.capability_id),
        format!("manifest.capabilities[{index}].capability_id"),
        "must be a qualified capability identifier",
    )?;
    nonempty(
        format!("manifest.capabilities[{index}].configuration_schema"),
        &capability.configuration_schema,
    )?;
    validate_string_list(
        &format!("manifest.capabilities[{index}].accepts"),
        &capability.accepts,
    )?;
    validate_string_list(
        &format!("manifest.capabilities[{index}].produces"),
        &capability.produces,
    )?;
    validate_string_list(
        &format!("manifest.capabilities[{index}].preconditions"),
        &capability.preconditions,
    )?;
    validate_string_list(
        &format!("manifest.capabilities[{index}].postconditions"),
        &capability.postconditions,
    )?;
    unique(
        format!("manifest.capabilities[{index}].effects"),
        &capability.effects,
    )
}

fn validate_execution_mode(mode: &ExecutionMode, index: usize) -> Result<(), ValidationError> {
    nonempty(
        format!("manifest.execution_modes[{index}].name"),
        &mode.name,
    )?;
    expect_schema(
        &format!("manifest.execution_modes[{index}].protocol"),
        &mode.protocol,
        EXTENSION_INVOCATION_V1,
    )?;
    nonempty(
        format!("manifest.execution_modes[{index}].entrypoint"),
        &mode.entrypoint,
    )?;
    validate_limits(
        &format!("manifest.execution_modes[{index}].limits"),
        &mode.limits,
    )
}

fn validate_limits(path: &str, limits: &ExecutionLimits) -> Result<(), ValidationError> {
    ensure(
        limits.timeout_ms > 0,
        format!("{path}.timeout_ms"),
        "must be greater than zero",
    )
}

fn validate_permissions(path: &str, permissions: &Permissions) -> Result<(), ValidationError> {
    validate_string_list(
        &format!("{path}.filesystem_read"),
        &permissions.filesystem_read,
    )?;
    validate_string_list(
        &format!("{path}.filesystem_write"),
        &permissions.filesystem_write,
    )?;
    unique(
        format!("{path}.environment_read"),
        &permissions.environment_read,
    )?;
    for (index, variable) in permissions.environment_read.iter().enumerate() {
        ensure(
            is_environment_name(variable),
            format!("{path}.environment_read[{index}]"),
            "must be an uppercase environment variable name",
        )?;
    }
    validate_string_list(&format!("{path}.subprocesses"), &permissions.subprocesses)?;
    validate_string_list(&format!("{path}.network_hosts"), &permissions.network_hosts)?;
    validate_string_list(&format!("{path}.ai_providers"), &permissions.ai_providers)
}

fn validate_diagnostic(diagnostic: &Diagnostic, path: &str) -> Result<(), ValidationError> {
    nonempty(format!("{path}.code"), &diagnostic.code)?;
    nonempty(format!("{path}.message"), &diagnostic.message)
}

fn validate_outcome_failure(outcome: Outcome, failure: &Failure) -> Result<(), ValidationError> {
    let classification = failure.classification;
    let consistent = match outcome {
        Outcome::Produced | Outcome::Reused | Outcome::Skipped => {
            classification == FailureClassification::None
        }
        Outcome::Unavailable => classification == FailureClassification::Unavailable,
        Outcome::Cancelled => classification == FailureClassification::Cancelled,
        Outcome::Blocked => matches!(
            classification,
            FailureClassification::Unauthorized
                | FailureClassification::Incompatible
                | FailureClassification::Validation
                | FailureClassification::Contract
        ),
        Outcome::Failed => matches!(
            classification,
            FailureClassification::Malformed
                | FailureClassification::Timeout
                | FailureClassification::Provider
                | FailureClassification::Validation
                | FailureClassification::Contract
                | FailureClassification::Internal
        ),
    };
    ensure(
        consistent,
        "result.failure.classification",
        "failure classification is inconsistent with the terminal outcome",
    )?;
    if classification == FailureClassification::None {
        ensure(
            failure.code.is_empty() && failure.message.is_empty() && !failure.retryable,
            "result.failure",
            "a none classification requires empty code/message and retryable false",
        )
    } else {
        nonempty("result.failure.code", &failure.code)?;
        nonempty("result.failure.message", &failure.message)
    }
}

fn validate_integrity(path: &str, integrity: &Integrity) -> Result<(), ValidationError> {
    validate_digest(&format!("{path}.value"), &integrity.value)
}

fn validate_digest(path: &str, digest: &str) -> Result<(), ValidationError> {
    ensure(
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        path,
        "must be 64 lowercase hexadecimal characters",
    )
}

pub(crate) fn validate_strict_version(
    path: &str,
    version: &str,
) -> Result<Version, ValidationError> {
    let parsed = Version::parse(version).map_err(|error| {
        ValidationError::new(path, format!("invalid semantic version: {error}"))
    })?;
    ensure(
        parsed.pre.is_empty() && parsed.build.is_empty() && parsed.to_string() == version,
        path,
        "must use strict major.minor.patch syntax",
    )?;
    Ok(parsed)
}

fn validate_string_list(path: &str, values: &[String]) -> Result<(), ValidationError> {
    unique(path, values)?;
    for (index, value) in values.iter().enumerate() {
        nonempty(format!("{path}[{index}]"), value)?;
    }
    Ok(())
}

fn expect_schema(path: &str, actual: &str, expected: &str) -> Result<(), ValidationError> {
    ensure(actual == expected, path, format!("must equal {expected}"))
}

fn ensure(
    condition: bool,
    path: impl Into<String>,
    message: impl Into<String>,
) -> Result<(), ValidationError> {
    if condition {
        Ok(())
    } else {
        Err(ValidationError::new(path, message))
    }
}

fn nonempty(path: impl Into<String>, value: &str) -> Result<(), ValidationError> {
    ensure(!value.is_empty(), path, "must not be empty")
}

fn unique<T>(path: impl Into<String>, values: &[T]) -> Result<(), ValidationError>
where
    T: Eq + Hash,
{
    let unique: HashSet<&T> = values.iter().collect();
    ensure(unique.len() == values.len(), path, "items must be unique")
}

fn unique_by<'a, T, K, F>(
    path: impl Into<String>,
    values: &'a [T],
    key: F,
    message: impl Into<String>,
) -> Result<(), ValidationError>
where
    K: Eq + Hash + ?Sized + 'a,
    F: Fn(&'a T) -> &'a K,
{
    let unique: HashSet<&K> = values.iter().map(key).collect();
    ensure(unique.len() == values.len(), path, message)
}

fn is_subset<T>(requested: &[T], granted: &[T]) -> bool
where
    T: Eq,
{
    requested.iter().all(|item| granted.contains(item))
}

pub(crate) fn is_extension_id(value: &str) -> bool {
    let mut segments = value.split('.');
    let Some(first) = segments.next() else {
        return false;
    };
    let remaining: Vec<&str> = segments.collect();
    is_identifier_segment(first, false)
        && !remaining.is_empty()
        && remaining
            .iter()
            .all(|segment| is_identifier_segment(segment, true))
}

pub(crate) fn is_publisher_id(value: &str) -> bool {
    let mut bytes = value.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_lowercase())
        && bytes.clone().next().is_some()
        && bytes
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b".-".contains(&byte))
}

fn is_identifier_segment(value: &str, allow_hyphen: bool) -> bool {
    let mut bytes = value.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_lowercase())
        && bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || (allow_hyphen && byte == b'-')
        })
}

pub(crate) fn is_capability_id(value: &str) -> bool {
    let Some((owner, name)) = value.split_once('/') else {
        return false;
    };
    !name.contains('/') && is_capability_segment(owner) && is_capability_segment(name)
}

fn is_capability_segment(value: &str) -> bool {
    let mut bytes = value.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_lowercase())
        && bytes.clone().next().is_some()
        && bytes
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b".-".contains(&byte))
}

fn is_environment_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_uppercase())
        && bytes.all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn has_prefixed_id(value: &str, prefix: &str) -> bool {
    let Some(identifier) = value.strip_prefix(prefix) else {
        return false;
    };
    let mut bytes = identifier.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes.all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || byte == b'.'
                || byte == b'_'
                || byte == b'-'
        })
}
