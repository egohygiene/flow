//! Exact process authority and host-enforcement evidence.
//!
//! This module keeps provider requests, operator grants, the selected
//! isolation profile, and caller-supplied host-enforcement evidence distinct.
//! A successful [`AuthorizedProcess`] value means those inputs matched for one
//! process preflight. It does not launch a process, authenticate the evidence
//! source, or prove that an operating-system sandbox actually ran.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::ResolvedExtension;
use crate::artifacts::{has_prefixed_id, validate_digest};
use crate::contracts::{
    EXTENSION_INVOCATION_V1, ExecutionModeKind, ExtensionInvocation, InvocationExtension,
    InvocationInterface, InvocationPhase, Permissions, Trust, ValidationError, is_capability_id,
    is_extension_id, is_publisher_id, validate_strict_version,
};
use crate::execution_subjects::{
    CANONICAL_JSON_V1, ExecutionSubjectLock, MatchedExecutionSubjects,
};

pub const PROCESS_AUTHORITY_PROFILE_V1: &str = "flow.process-authority-profile/v1";
pub const PROCESS_ENFORCEMENT_EVIDENCE_V1: &str = "flow.process-enforcement-evidence/v1";

/// Operator-selected authority for one exact process invocation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessAuthorityProfile {
    pub schema_version: String,
    pub canonicalization: String,
    pub authority_profile_id: String,
    pub extension_lock_id: String,
    pub authorization_id: String,
    pub grants_digest: String,
    pub invocation_id: String,
    pub run_id: String,
    pub extension: InvocationExtension,
    pub capability_id: String,
    pub interface: InvocationInterface,
    pub subject_lock_id: String,
    pub subject_lock_digest: String,
    pub operator_trust: Trust,
    pub isolation: ProcessIsolation,
    pub ambient_authority: AmbientAuthorityPolicy,
    pub requested: ProcessAuthority,
    pub granted: ProcessAuthority,
}

/// Whether process execution is explicitly unconfined or requires sandbox
/// evidence before it is eligible for preflight.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProcessIsolation {
    TrustedUnconfined,
    Sandboxed,
}

/// Closed v1 ambient-authority policy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AmbientAuthorityPolicy {
    DenyUnlisted,
}

/// Exact authority selected for a single direct executable invocation.
///
/// `argv` is an ordered argument tail. Every other collection is a canonical,
/// sorted allowlist. Environment entries carry handles rather than secret
/// values. Empty collections deny that authority dimension.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessAuthority {
    pub argv: Vec<String>,
    pub environment: Vec<EnvironmentBinding>,
    pub filesystem_read: Vec<String>,
    pub filesystem_write: Vec<String>,
    pub network_endpoints: Vec<String>,
    pub subprocesses: Vec<String>,
    pub ai_providers: Vec<String>,
    pub gpus: Vec<GpuAccess>,
    pub source_mutation_targets: Vec<String>,
    pub destructive_operations: Vec<String>,
    pub publication_destinations: Vec<String>,
    pub signing_key_handles: Vec<String>,
    pub telemetry: TelemetryPropagation,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentBinding {
    pub name: String,
    pub source_handle: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GpuAccess {
    pub device_id: String,
    pub capabilities: Vec<GpuCapability>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GpuCapability {
    Compute,
    VideoDecode,
    VideoEncode,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryPropagation {
    pub fields: Vec<TelemetryField>,
    pub targets: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TelemetryField {
    Baggage,
    CorrelationId,
    Traceparent,
    Tracestate,
}

/// Portable caller-supplied statement about host enforcement for one profile.
///
/// Like [`crate::ProcessTranscript`], this document is an observation supplied
/// by a future runner. Flow validates its closed shape and exact correlation;
/// it does not authenticate the caller or operate an OS sandbox here.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessEnforcementEvidence {
    pub schema_version: String,
    pub canonicalization: String,
    pub enforcement_evidence_id: String,
    pub authority_profile_id: String,
    pub authority_profile_digest: String,
    pub invocation_id: String,
    pub run_id: String,
    pub extension: InvocationExtension,
    pub capability_id: String,
    pub interface: InvocationInterface,
    pub subject_lock_id: String,
    pub subject_lock_digest: String,
    pub subject_observation_digest: String,
    pub isolation: ProcessIsolation,
    pub ambient_authority: AmbientAuthorityPolicy,
    pub source: EnforcementEvidenceSource,
    pub backend_id: String,
    pub backend_version: String,
    pub enforced: ProcessAuthority,
    pub guarantees: Vec<EnforcementGuarantee>,
    /// Closed extension point. V1 implements no additional verification claim.
    pub unsupported_claims: Vec<Value>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnforcementEvidenceSource {
    None,
    CallerAttestedHost,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnforcementGuarantee {
    pub dimension: AuthorityDimension,
    pub status: EnforcementStatus,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthorityDimension {
    Argv,
    Environment,
    FilesystemRead,
    FilesystemWrite,
    Network,
    Subprocess,
    AiProvider,
    Gpu,
    SourceMutation,
    Destructive,
    Publication,
    Signing,
    Telemetry,
}

const AUTHORITY_DIMENSIONS: [AuthorityDimension; 13] = [
    AuthorityDimension::Argv,
    AuthorityDimension::Environment,
    AuthorityDimension::FilesystemRead,
    AuthorityDimension::FilesystemWrite,
    AuthorityDimension::Network,
    AuthorityDimension::Subprocess,
    AuthorityDimension::AiProvider,
    AuthorityDimension::Gpu,
    AuthorityDimension::SourceMutation,
    AuthorityDimension::Destructive,
    AuthorityDimension::Publication,
    AuthorityDimension::Signing,
    AuthorityDimension::Telemetry,
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnforcementStatus {
    Enforced,
    NotEnforced,
}

/// Opaque successful process-authority preflight.
///
/// This value authorizes only Flow's request/transcript evidence seams. It is
/// not an OS capability and does not prove that a process was launched or that
/// caller-attested enforcement happened.
///
/// The fields remain private so portable evidence cannot be promoted with a
/// struct literal:
///
/// ```compile_fail
/// use flow::AuthorizedProcess;
///
/// let _unvalidated = AuthorizedProcess {};
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedProcess {
    profile: ProcessAuthorityProfile,
    evidence: ProcessEnforcementEvidence,
    profile_digest: String,
    evidence_digest: String,
    context: AuthorizedProcessContext,
}

impl AuthorizedProcess {
    #[must_use]
    pub const fn profile(&self) -> &ProcessAuthorityProfile {
        &self.profile
    }

    #[must_use]
    pub const fn evidence(&self) -> &ProcessEnforcementEvidence {
        &self.evidence
    }

    #[must_use]
    pub fn profile_digest(&self) -> &str {
        &self.profile_digest
    }

    #[must_use]
    pub fn evidence_digest(&self) -> &str {
        &self.evidence_digest
    }

    #[must_use]
    pub const fn isolation(&self) -> ProcessIsolation {
        self.profile.isolation
    }

    pub(crate) fn matches_context(
        &self,
        resolved: &ResolvedExtension,
        invocation: &ExtensionInvocation,
        subject_lock: &ExecutionSubjectLock,
        subjects: &MatchedExecutionSubjects,
    ) -> Result<(), String> {
        subjects.matches_context(resolved, invocation, subject_lock)?;
        if self.context.extension_lock_id != resolved.lock_id()
            || self.context.authorization_id != invocation.authorization.authorization_id
            || self.context.grants_digest != invocation.authorization.grants_digest
            || self.context.invocation_id != invocation.invocation_id
            || self.context.run_id != invocation.run_id
            || self.context.extension != invocation.extension
            || self.context.capability_id != invocation.capability_id
            || self.context.interface != invocation.interface
            || self.context.subject_lock_id != subject_lock.subject_lock_id
            || self.context.subject_lock_digest != subjects.lock_digest()
            || self.context.subject_observation_digest != subjects.observation_digest()
            || self.context.operator_trust != resolved.trust()
            || self.context.isolation != self.profile.isolation
            || self.context.authority_profile_id != self.profile.authority_profile_id
            || self.context.enforcement_evidence_id != self.evidence.enforcement_evidence_id
            || self.context.authority_profile_digest != self.profile_digest
            || self.context.enforcement_evidence_digest != self.evidence_digest
        {
            return Err(
                "the process authorization token does not belong to this resolution, invocation, and subject observation"
                    .to_owned(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AuthorizedProcessContext {
    authority_profile_id: String,
    authority_profile_digest: String,
    enforcement_evidence_id: String,
    enforcement_evidence_digest: String,
    extension_lock_id: String,
    authorization_id: String,
    grants_digest: String,
    invocation_id: String,
    run_id: String,
    extension: InvocationExtension,
    capability_id: String,
    interface: InvocationInterface,
    subject_lock_id: String,
    subject_lock_digest: String,
    subject_observation_digest: String,
    operator_trust: Trust,
    isolation: ProcessIsolation,
}

impl ProcessAuthorityProfile {
    /// Validate the closed profile and its internal authority invariants.
    ///
    /// # Errors
    ///
    /// Returns the first deterministic contract violation.
    pub fn validate(&self) -> Result<(), ValidationError> {
        expect(
            self.schema_version == PROCESS_AUTHORITY_PROFILE_V1,
            "authority_profile.schema_version",
            format!("must equal {PROCESS_AUTHORITY_PROFILE_V1}"),
        )?;
        expect(
            self.canonicalization == CANONICAL_JSON_V1,
            "authority_profile.canonicalization",
            format!("must equal {CANONICAL_JSON_V1}"),
        )?;
        expect(
            has_prefixed_id(&self.authority_profile_id, "authority-profile:"),
            "authority_profile.authority_profile_id",
            "must be a lowercase authority-profile identifier",
        )?;
        expect(
            has_prefixed_id(&self.extension_lock_id, "lock:"),
            "authority_profile.extension_lock_id",
            "must be a lowercase extension-lock identifier",
        )?;
        expect(
            has_prefixed_id(&self.authorization_id, "authorization:"),
            "authority_profile.authorization_id",
            "must be a lowercase authorization identifier",
        )?;
        validate_digest("authority_profile.grants_digest", &self.grants_digest)?;
        expect(
            has_prefixed_id(&self.invocation_id, "invocation:"),
            "authority_profile.invocation_id",
            "must be a lowercase invocation identifier",
        )?;
        expect(
            has_prefixed_id(&self.run_id, "run:"),
            "authority_profile.run_id",
            "must be a lowercase run identifier",
        )?;
        validate_extension("authority_profile.extension", &self.extension)?;
        expect(
            is_capability_id(&self.capability_id),
            "authority_profile.capability_id",
            "must be a qualified capability identifier",
        )?;
        validate_process_interface("authority_profile.interface", &self.interface)?;
        expect(
            has_prefixed_id(&self.subject_lock_id, "subject-lock:"),
            "authority_profile.subject_lock_id",
            "must be a lowercase subject-lock identifier",
        )?;
        validate_digest(
            "authority_profile.subject_lock_digest",
            &self.subject_lock_digest,
        )?;
        expect(
            self.operator_trust != Trust::Disabled,
            "authority_profile.operator_trust",
            "disabled providers cannot receive process authority",
        )?;
        expect(
            self.operator_trust != Trust::Sandboxed
                || self.isolation == ProcessIsolation::Sandboxed,
            "authority_profile.isolation",
            "sandbox-required operator trust cannot select trusted-unconfined execution",
        )?;
        self.requested.validate("authority_profile.requested")?;
        self.granted.validate("authority_profile.granted")?;
        expect(
            self.granted.allows(&self.requested),
            "authority_profile.granted",
            "must cover every exact requested authority without widening argv",
        )
    }

    /// Return deterministic compact canonical JSON bytes for profile identity.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid profile or serialization failure.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ProcessAuthorityError> {
        self.validate()
            .map_err(|source| ProcessAuthorityError::InvalidProfile { source })?;
        canonical_json_bytes(self)
    }

    /// Return lowercase SHA-256 over [`Self::canonical_bytes`].
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid profile or serialization failure.
    pub fn canonical_digest(&self) -> Result<String, ProcessAuthorityError> {
        Ok(format!("{:x}", Sha256::digest(self.canonical_bytes()?)))
    }
}

impl ProcessEnforcementEvidence {
    /// Validate the closed evidence shape without trusting its host claims.
    ///
    /// # Errors
    ///
    /// Returns the first deterministic contract violation.
    pub fn validate(&self) -> Result<(), ValidationError> {
        expect(
            self.schema_version == PROCESS_ENFORCEMENT_EVIDENCE_V1,
            "enforcement_evidence.schema_version",
            format!("must equal {PROCESS_ENFORCEMENT_EVIDENCE_V1}"),
        )?;
        expect(
            self.canonicalization == CANONICAL_JSON_V1,
            "enforcement_evidence.canonicalization",
            format!("must equal {CANONICAL_JSON_V1}"),
        )?;
        expect(
            has_prefixed_id(&self.enforcement_evidence_id, "enforcement:"),
            "enforcement_evidence.enforcement_evidence_id",
            "must be a lowercase enforcement identifier",
        )?;
        expect(
            has_prefixed_id(&self.authority_profile_id, "authority-profile:"),
            "enforcement_evidence.authority_profile_id",
            "must be a lowercase authority-profile identifier",
        )?;
        validate_digest(
            "enforcement_evidence.authority_profile_digest",
            &self.authority_profile_digest,
        )?;
        expect(
            has_prefixed_id(&self.invocation_id, "invocation:"),
            "enforcement_evidence.invocation_id",
            "must be a lowercase invocation identifier",
        )?;
        expect(
            has_prefixed_id(&self.run_id, "run:"),
            "enforcement_evidence.run_id",
            "must be a lowercase run identifier",
        )?;
        validate_extension("enforcement_evidence.extension", &self.extension)?;
        expect(
            is_capability_id(&self.capability_id),
            "enforcement_evidence.capability_id",
            "must be a qualified capability identifier",
        )?;
        validate_process_interface("enforcement_evidence.interface", &self.interface)?;
        expect(
            has_prefixed_id(&self.subject_lock_id, "subject-lock:"),
            "enforcement_evidence.subject_lock_id",
            "must be a lowercase subject-lock identifier",
        )?;
        validate_digest(
            "enforcement_evidence.subject_lock_digest",
            &self.subject_lock_digest,
        )?;
        validate_digest(
            "enforcement_evidence.subject_observation_digest",
            &self.subject_observation_digest,
        )?;
        validate_authority_value("enforcement_evidence.backend_id", &self.backend_id)?;
        validate_strict_version(
            "enforcement_evidence.backend_version",
            &self.backend_version,
        )?;
        self.enforced.validate("enforcement_evidence.enforced")?;
        expect(
            self.unsupported_claims.is_empty(),
            "enforcement_evidence.unsupported_claims",
            "v1 supports no additional enforcement or verification claims",
        )?;
        validate_guarantees(self)?;
        match self.isolation {
            ProcessIsolation::TrustedUnconfined => {
                expect(
                    self.source == EnforcementEvidenceSource::None,
                    "enforcement_evidence.source",
                    "trusted-unconfined evidence must use source none",
                )?;
                expect(
                    self.backend_id == "none" && self.backend_version == "0.0.0",
                    "enforcement_evidence.backend_id",
                    "trusted-unconfined evidence must name backend none at version 0.0.0",
                )?;
                expect(
                    self.enforced.is_empty(),
                    "enforcement_evidence.enforced",
                    "trusted-unconfined execution cannot claim enforced authority bounds",
                )?;
            }
            ProcessIsolation::Sandboxed => {
                expect(
                    self.source == EnforcementEvidenceSource::CallerAttestedHost,
                    "enforcement_evidence.source",
                    "sandboxed evidence requires caller-attested host enforcement",
                )?;
                expect(
                    self.backend_id != "none",
                    "enforcement_evidence.backend_id",
                    "sandboxed evidence must identify an enforcing backend",
                )?;
            }
        }
        Ok(())
    }

    /// Return deterministic compact canonical JSON bytes for evidence identity.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid evidence or serialization failure.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ProcessAuthorityError> {
        self.validate()
            .map_err(|source| ProcessAuthorityError::InvalidEvidence { source })?;
        canonical_json_bytes(self)
    }

    /// Return lowercase SHA-256 over [`Self::canonical_bytes`].
    ///
    /// # Errors
    ///
    /// Returns an error for invalid evidence or serialization failure.
    pub fn canonical_digest(&self) -> Result<String, ProcessAuthorityError> {
        Ok(format!("{:x}", Sha256::digest(self.canonical_bytes()?)))
    }
}

impl ProcessAuthority {
    fn validate(&self, path: &str) -> Result<(), ValidationError> {
        for (index, argument) in self.argv.iter().enumerate() {
            validate_authority_value(&format!("{path}.argv[{index}]"), argument)?;
        }
        expect_canonical_set(&format!("{path}.environment"), &self.environment)?;
        let mut handles = BTreeSet::new();
        for (index, binding) in self.environment.iter().enumerate() {
            expect(
                is_environment_name(&binding.name),
                format!("{path}.environment[{index}].name"),
                "must be an uppercase environment variable name",
            )?;
            validate_handle(
                &format!("{path}.environment[{index}].source_handle"),
                &binding.source_handle,
            )?;
            expect(
                handles.insert(binding.source_handle.as_str()),
                format!("{path}.environment"),
                "environment source handles must be unique",
            )?;
        }
        validate_canonical_string_set(&format!("{path}.filesystem_read"), &self.filesystem_read)?;
        validate_canonical_string_set(&format!("{path}.filesystem_write"), &self.filesystem_write)?;
        validate_canonical_string_set(
            &format!("{path}.network_endpoints"),
            &self.network_endpoints,
        )?;
        validate_canonical_string_set(&format!("{path}.subprocesses"), &self.subprocesses)?;
        validate_canonical_string_set(&format!("{path}.ai_providers"), &self.ai_providers)?;
        expect_canonical_set(&format!("{path}.gpus"), &self.gpus)?;
        for (index, gpu) in self.gpus.iter().enumerate() {
            validate_authority_value(&format!("{path}.gpus[{index}].device_id"), &gpu.device_id)?;
            expect(
                !gpu.capabilities.is_empty(),
                format!("{path}.gpus[{index}].capabilities"),
                "must contain at least one GPU capability",
            )?;
            expect_canonical_set(
                &format!("{path}.gpus[{index}].capabilities"),
                &gpu.capabilities,
            )?;
        }
        validate_canonical_string_set(
            &format!("{path}.source_mutation_targets"),
            &self.source_mutation_targets,
        )?;
        validate_canonical_string_set(
            &format!("{path}.destructive_operations"),
            &self.destructive_operations,
        )?;
        validate_canonical_string_set(
            &format!("{path}.publication_destinations"),
            &self.publication_destinations,
        )?;
        validate_canonical_string_set(
            &format!("{path}.signing_key_handles"),
            &self.signing_key_handles,
        )?;
        expect_canonical_set(&format!("{path}.telemetry.fields"), &self.telemetry.fields)?;
        validate_canonical_string_set(
            &format!("{path}.telemetry.targets"),
            &self.telemetry.targets,
        )
    }

    fn allows(&self, requested: &Self) -> bool {
        self.argv == requested.argv
            && is_subset(&requested.environment, &self.environment)
            && is_subset(&requested.filesystem_read, &self.filesystem_read)
            && is_subset(&requested.filesystem_write, &self.filesystem_write)
            && is_subset(&requested.network_endpoints, &self.network_endpoints)
            && is_subset(&requested.subprocesses, &self.subprocesses)
            && is_subset(&requested.ai_providers, &self.ai_providers)
            && gpus_allow(&self.gpus, &requested.gpus)
            && is_subset(
                &requested.source_mutation_targets,
                &self.source_mutation_targets,
            )
            && is_subset(
                &requested.destructive_operations,
                &self.destructive_operations,
            )
            && is_subset(
                &requested.publication_destinations,
                &self.publication_destinations,
            )
            && is_subset(&requested.signing_key_handles, &self.signing_key_handles)
            && is_subset(&requested.telemetry.fields, &self.telemetry.fields)
            && is_subset(&requested.telemetry.targets, &self.telemetry.targets)
    }

    fn is_empty(&self) -> bool {
        self.argv.is_empty()
            && self.environment.is_empty()
            && self.filesystem_read.is_empty()
            && self.filesystem_write.is_empty()
            && self.network_endpoints.is_empty()
            && self.subprocesses.is_empty()
            && self.ai_providers.is_empty()
            && self.gpus.is_empty()
            && self.source_mutation_targets.is_empty()
            && self.destructive_operations.is_empty()
            && self.publication_destinations.is_empty()
            && self.signing_key_handles.is_empty()
            && self.telemetry.fields.is_empty()
            && self.telemetry.targets.is_empty()
    }
}

/// Correlate one process profile, matched subject token, and enforcement
/// observation, returning an opaque preflight token only on exact agreement.
///
/// # Errors
///
/// Returns a typed error for invalid contracts, context mismatch, authority
/// escalation, incomplete sandbox evidence, or identity failure.
pub fn authorize_process(
    resolved: &ResolvedExtension,
    invocation: &ExtensionInvocation,
    subject_lock: &ExecutionSubjectLock,
    subjects: &MatchedExecutionSubjects,
    profile: &ProcessAuthorityProfile,
    evidence: &ProcessEnforcementEvidence,
) -> Result<AuthorizedProcess, ProcessAuthorityError> {
    profile
        .validate()
        .map_err(|source| ProcessAuthorityError::InvalidProfile { source })?;
    evidence
        .validate()
        .map_err(|source| ProcessAuthorityError::InvalidEvidence { source })?;
    invocation
        .validate()
        .map_err(|source| ProcessAuthorityError::InvalidInvocation { source })?;
    subjects
        .matches_context(resolved, invocation, subject_lock)
        .map_err(|message| ProcessAuthorityError::ContextMismatch { message })?;
    correlate_profile(resolved, invocation, subject_lock, subjects, profile)?;
    let profile_digest = profile.canonical_digest()?;
    correlate_evidence(profile, &profile_digest, subjects, evidence)?;
    let evidence_digest = evidence.canonical_digest()?;
    let context = AuthorizedProcessContext {
        authority_profile_id: profile.authority_profile_id.clone(),
        authority_profile_digest: profile_digest.clone(),
        enforcement_evidence_id: evidence.enforcement_evidence_id.clone(),
        enforcement_evidence_digest: evidence_digest.clone(),
        extension_lock_id: profile.extension_lock_id.clone(),
        authorization_id: profile.authorization_id.clone(),
        grants_digest: profile.grants_digest.clone(),
        invocation_id: profile.invocation_id.clone(),
        run_id: profile.run_id.clone(),
        extension: profile.extension.clone(),
        capability_id: profile.capability_id.clone(),
        interface: profile.interface.clone(),
        subject_lock_id: profile.subject_lock_id.clone(),
        subject_lock_digest: profile.subject_lock_digest.clone(),
        subject_observation_digest: subjects.observation_digest().to_owned(),
        operator_trust: profile.operator_trust,
        isolation: profile.isolation,
    };
    Ok(AuthorizedProcess {
        profile: profile.clone(),
        evidence: evidence.clone(),
        profile_digest,
        evidence_digest,
        context,
    })
}

#[derive(Debug, Error)]
pub enum ProcessAuthorityError {
    #[error("invalid process-authority profile: {source}")]
    InvalidProfile {
        #[source]
        source: ValidationError,
    },
    #[error("invalid process-enforcement evidence: {source}")]
    InvalidEvidence {
        #[source]
        source: ValidationError,
    },
    #[error("invalid process invocation: {source}")]
    InvalidInvocation {
        #[source]
        source: ValidationError,
    },
    #[error("process-authority context mismatch: {message}")]
    ContextMismatch { message: String },
    #[error("process-authority request exceeds operator grants: {message}")]
    AuthorityExceeded { message: String },
    #[error("process sandbox evidence is incomplete or contradictory: {message}")]
    EnforcementMismatch { message: String },
    #[error("failed to encode deterministic process-authority identity: {message}")]
    Canonicalization { message: String },
}

fn correlate_profile(
    resolved: &ResolvedExtension,
    invocation: &ExtensionInvocation,
    subject_lock: &ExecutionSubjectLock,
    subjects: &MatchedExecutionSubjects,
    profile: &ProcessAuthorityProfile,
) -> Result<(), ProcessAuthorityError> {
    if resolved.execution_mode().kind != ExecutionModeKind::Process
        || invocation.phase != InvocationPhase::Execute
    {
        return context_mismatch("process authority requires a process-mode execute invocation");
    }
    if profile.extension_lock_id != resolved.lock_id()
        || profile.extension_lock_id != invocation.authorization.lock_id
        || profile.authorization_id != invocation.authorization.authorization_id
        || profile.grants_digest != invocation.authorization.grants_digest
    {
        return context_mismatch("authority profile references different operator authorization");
    }
    if profile.invocation_id != invocation.invocation_id || profile.run_id != invocation.run_id {
        return context_mismatch("authority profile references a different run or invocation");
    }
    if profile.extension != invocation.extension
        || profile.extension.extension_id != resolved.extension_id()
        || profile.extension.version != resolved.version()
        || profile.extension.publisher_id != resolved.publisher_id()
        || profile.extension.integrity != resolved.integrity().value
    {
        return context_mismatch("authority profile provider identity does not match resolution");
    }
    if profile.capability_id != invocation.capability_id
        || profile.capability_id != resolved.capability().capability_id
    {
        return context_mismatch("authority profile capability does not match resolution");
    }
    if profile.interface != invocation.interface
        || profile.interface.name != resolved.execution_mode().name
        || profile.interface.kind != resolved.execution_mode().kind
        || profile.interface.protocol != resolved.execution_mode().protocol
    {
        return context_mismatch("authority profile process interface does not match resolution");
    }
    if profile.subject_lock_id != subject_lock.subject_lock_id
        || profile.subject_lock_digest != subjects.lock_digest()
    {
        return context_mismatch("authority profile references different execution subjects");
    }
    if profile.operator_trust != resolved.trust() {
        return context_mismatch("authority profile operator trust does not match resolution");
    }
    if resolved.trust() == Trust::Sandboxed && profile.isolation != ProcessIsolation::Sandboxed {
        return Err(ProcessAuthorityError::EnforcementMismatch {
            message: "sandbox-required operator trust cannot run unconfined".to_owned(),
        });
    }
    if !requested_matches_permissions(&profile.requested, resolved.requested_permissions()) {
        return context_mismatch(
            "authority profile requests do not exactly represent the provider declaration",
        );
    }
    if !granted_within_permissions(&profile.granted, resolved.granted_permissions()) {
        return Err(ProcessAuthorityError::AuthorityExceeded {
            message: "authority profile grants exceed the resolved operator lock".to_owned(),
        });
    }
    if !profile.granted.allows(&profile.requested) {
        return Err(ProcessAuthorityError::AuthorityExceeded {
            message: "one or more requested authority dimensions are not explicitly granted"
                .to_owned(),
        });
    }
    let mut invocation_handles = invocation.secret_handles.clone();
    invocation_handles.sort();
    let requested_handles = profile
        .requested
        .environment
        .iter()
        .map(|binding| binding.source_handle.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if invocation_handles != requested_handles {
        return context_mismatch(
            "invocation secret handles must exactly match requested environment handles",
        );
    }
    Ok(())
}

fn correlate_evidence(
    profile: &ProcessAuthorityProfile,
    profile_digest: &str,
    subjects: &MatchedExecutionSubjects,
    evidence: &ProcessEnforcementEvidence,
) -> Result<(), ProcessAuthorityError> {
    if evidence.authority_profile_id != profile.authority_profile_id
        || evidence.authority_profile_digest != profile_digest
        || evidence.invocation_id != profile.invocation_id
        || evidence.run_id != profile.run_id
        || evidence.extension != profile.extension
        || evidence.capability_id != profile.capability_id
        || evidence.interface != profile.interface
        || evidence.subject_lock_id != profile.subject_lock_id
        || evidence.subject_lock_digest != profile.subject_lock_digest
        || evidence.subject_observation_digest != subjects.observation_digest()
        || evidence.isolation != profile.isolation
        || evidence.ambient_authority != profile.ambient_authority
    {
        return Err(ProcessAuthorityError::EnforcementMismatch {
            message: "host enforcement evidence does not identify the exact authority profile and execution subjects"
                .to_owned(),
        });
    }
    match profile.isolation {
        ProcessIsolation::TrustedUnconfined => {
            if !evidence.enforced.is_empty() {
                return Err(ProcessAuthorityError::EnforcementMismatch {
                    message: "trusted-unconfined evidence cannot claim enforced authority bounds"
                        .to_owned(),
                });
            }
        }
        ProcessIsolation::Sandboxed => {
            if evidence.enforced != profile.requested {
                return Err(ProcessAuthorityError::EnforcementMismatch {
                    message: "sandbox evidence must enforce the exact requested authority without omissions or additions"
                        .to_owned(),
                });
            }
        }
    }
    Ok(())
}

fn requested_matches_permissions(authority: &ProcessAuthority, permissions: &Permissions) -> bool {
    same_members(&authority.filesystem_read, &permissions.filesystem_read)
        && same_members(&authority.filesystem_write, &permissions.filesystem_write)
        && same_members(
            &authority
                .environment
                .iter()
                .map(|binding| binding.name.clone())
                .collect::<Vec<_>>(),
            &permissions.environment_read,
        )
        && same_members(&authority.subprocesses, &permissions.subprocesses)
        && same_members(&authority.network_endpoints, &permissions.network_hosts)
        && same_members(&authority.ai_providers, &permissions.ai_providers)
        && permissions.gpu != authority.gpus.is_empty()
        && permissions.source_mutation != authority.source_mutation_targets.is_empty()
        && permissions.destructive != authority.destructive_operations.is_empty()
        && permissions.sign != authority.signing_key_handles.is_empty()
        && permissions.publish != authority.publication_destinations.is_empty()
}

fn granted_within_permissions(authority: &ProcessAuthority, permissions: &Permissions) -> bool {
    is_subset(&authority.filesystem_read, &permissions.filesystem_read)
        && is_subset(&authority.filesystem_write, &permissions.filesystem_write)
        && authority
            .environment
            .iter()
            .all(|binding| permissions.environment_read.contains(&binding.name))
        && is_subset(&authority.subprocesses, &permissions.subprocesses)
        && is_subset(&authority.network_endpoints, &permissions.network_hosts)
        && is_subset(&authority.ai_providers, &permissions.ai_providers)
        && (authority.gpus.is_empty() || permissions.gpu)
        && (authority.source_mutation_targets.is_empty() || permissions.source_mutation)
        && (authority.destructive_operations.is_empty() || permissions.destructive)
        && (authority.signing_key_handles.is_empty() || permissions.sign)
        && (authority.publication_destinations.is_empty() || permissions.publish)
}

fn validate_guarantees(evidence: &ProcessEnforcementEvidence) -> Result<(), ValidationError> {
    expect(
        evidence.guarantees.len() == AUTHORITY_DIMENSIONS.len(),
        "enforcement_evidence.guarantees",
        "must contain exactly one entry for every authority dimension",
    )?;
    let expected_status = match evidence.isolation {
        ProcessIsolation::TrustedUnconfined => EnforcementStatus::NotEnforced,
        ProcessIsolation::Sandboxed => EnforcementStatus::Enforced,
    };
    for (index, dimension) in AUTHORITY_DIMENSIONS.iter().enumerate() {
        let guarantee = &evidence.guarantees[index];
        expect(
            guarantee.dimension == *dimension,
            format!("enforcement_evidence.guarantees[{index}].dimension"),
            "guarantees must use the complete canonical dimension order",
        )?;
        expect(
            guarantee.status == expected_status,
            format!("enforcement_evidence.guarantees[{index}].status"),
            match evidence.isolation {
                ProcessIsolation::TrustedUnconfined => {
                    "trusted-unconfined profiles must report every dimension not-enforced"
                }
                ProcessIsolation::Sandboxed => {
                    "sandboxed profiles must report every dimension enforced"
                }
            },
        )?;
    }
    Ok(())
}

fn validate_extension(path: &str, extension: &InvocationExtension) -> Result<(), ValidationError> {
    expect(
        is_extension_id(&extension.extension_id),
        format!("{path}.extension_id"),
        "must be a lowercase qualified extension identifier",
    )?;
    validate_strict_version(&format!("{path}.version"), &extension.version)?;
    expect(
        is_publisher_id(&extension.publisher_id),
        format!("{path}.publisher_id"),
        "must be a lowercase publisher identifier",
    )?;
    validate_digest(&format!("{path}.integrity"), &extension.integrity)
}

fn validate_process_interface(
    path: &str,
    interface: &InvocationInterface,
) -> Result<(), ValidationError> {
    expect(
        interface.kind == ExecutionModeKind::Process,
        format!("{path}.kind"),
        "must equal process",
    )?;
    validate_authority_value(&format!("{path}.name"), &interface.name)?;
    expect(
        interface.protocol == EXTENSION_INVOCATION_V1,
        format!("{path}.protocol"),
        format!("must equal {EXTENSION_INVOCATION_V1}"),
    )
}

fn validate_canonical_string_set(path: &str, values: &[String]) -> Result<(), ValidationError> {
    for (index, value) in values.iter().enumerate() {
        validate_authority_value(&format!("{path}[{index}]"), value)?;
    }
    expect_canonical_set(path, values)
}

fn validate_authority_value(path: &str, value: &str) -> Result<(), ValidationError> {
    expect(
        !value.is_empty() && value != "*" && !value.chars().any(char::is_control),
        path,
        "must be explicit, nonempty, contain no wildcard-only or control value",
    )
}

fn validate_handle(path: &str, value: &str) -> Result<(), ValidationError> {
    expect(
        value.starts_with("secret:")
            && value.len() > "secret:".len()
            && value.chars().all(|character| {
                character.is_ascii_lowercase()
                    || character.is_ascii_digit()
                    || matches!(character, ':' | '.' | '_' | '-')
            }),
        path,
        "must be a lowercase opaque secret handle",
    )
}

fn expect_canonical_set<T>(path: &str, values: &[T]) -> Result<(), ValidationError>
where
    T: Ord,
{
    expect(
        values.windows(2).all(|pair| pair[0] < pair[1]),
        path,
        "must be strictly sorted and contain no duplicates",
    )
}

fn is_subset<T>(requested: &[T], granted: &[T]) -> bool
where
    T: Eq,
{
    requested.iter().all(|item| granted.contains(item))
}

fn same_members<T>(left: &[T], right: &[T]) -> bool
where
    T: Ord + Clone,
{
    let mut left = left.to_vec();
    let mut right = right.to_vec();
    left.sort();
    right.sort();
    left == right
}

fn gpus_allow(granted: &[GpuAccess], requested: &[GpuAccess]) -> bool {
    requested.iter().all(|request| {
        granted.iter().any(|grant| {
            grant.device_id == request.device_id
                && is_subset(&request.capabilities, &grant.capabilities)
        })
    })
}

fn is_environment_name(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some('A'..='Z' | '_'))
        && characters.all(|character| matches!(character, 'A'..='Z' | '0'..='9' | '_'))
}

fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, ProcessAuthorityError> {
    let value =
        serde_json::to_value(value).map_err(|error| ProcessAuthorityError::Canonicalization {
            message: error.to_string(),
        })?;
    serde_json::to_vec(&sort_json(value)).map_err(|error| ProcessAuthorityError::Canonicalization {
        message: error.to_string(),
    })
}

fn sort_json(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(sort_json).collect()),
        Value::Object(values) => {
            let mut entries = values.into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            let mut sorted = Map::new();
            for (key, value) in entries {
                sorted.insert(key, sort_json(value));
            }
            Value::Object(sorted)
        }
        scalar => scalar,
    }
}

fn context_mismatch<T>(message: impl Into<String>) -> Result<T, ProcessAuthorityError> {
    Err(ProcessAuthorityError::ContextMismatch {
        message: message.into(),
    })
}

fn expect(
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
