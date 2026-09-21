//! Locked package and executable subjects for external-process execution.
//!
//! This module observes provider package and executable bytes without running
//! them. A successful [`MatchedExecutionSubjects`] value proves only that a
//! fresh Flow-owned observation matched an operator-controlled lock for one
//! invocation context. It does not prove a publisher signature, provenance,
//! transparency-log inclusion, operator trustworthiness, sandbox enforcement,
//! or that a later launcher executed the same open file object.

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::ResolvedExtension;
use crate::artifacts::{
    ARTIFACT_BINDINGS_V1, ARTIFACT_OBSERVATIONS_V1, ArtifactBindingSet, ArtifactKind,
    ArtifactObservationError, DIRECTORY_MANIFEST_V1, DirectoryManifestEntry,
    HostArtifactObservation, HostArtifactObservationSet, InputArtifactBinding, SHA256,
    has_prefixed_id, observe_artifacts, validate_digest, validate_locator,
};
use crate::contracts::{
    EXTENSION_INVOCATION_V1, ExecutionModeKind, ExtensionInvocation, Integrity, IntegrityAlgorithm,
    InvocationExtension, InvocationInterface, InvocationPhase, Trust, ValidationError,
    is_capability_id, is_extension_id, is_publisher_id, validate_strict_version,
};

pub const EXECUTION_SUBJECT_LOCK_V1: &str = "flow.execution-subject-lock/v1";
pub const EXECUTION_SUBJECT_OBSERVATIONS_V1: &str = "flow.execution-subject-observations/v1";
pub const CANONICAL_JSON_V1: &str = "flow.canonical-json/v1";

const PACKAGE_ARTIFACT_MEDIA_TYPE: &str = "application/vnd.flow.execution-package-directory";
const EXECUTABLE_ARTIFACT_MEDIA_TYPE: &str = "application/vnd.flow.execution-executable";
const PACKAGE_PORT: &str = "port:execution-package";
const EXECUTABLE_PORT: &str = "port:execution-executable";

/// Operator-controlled lock for the exact package and executable eligible for
/// one process-mode provider context.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSubjectLock {
    pub schema_version: String,
    pub canonicalization: String,
    pub subject_lock_id: String,
    pub extension_lock_id: String,
    pub extension: InvocationExtension,
    pub capability_id: String,
    pub interface: InvocationInterface,
    pub declared_entrypoint: String,
    pub package: LockedPackageSubject,
    pub executable: LockedExecutableSubject,
    pub evidence_policy: ExecutionSubjectEvidencePolicy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedPackageSubject {
    pub subject_id: String,
    pub kind: ArtifactKind,
    pub locator: String,
    pub digest: Integrity,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockedExecutableSubject {
    pub subject_id: String,
    pub kind: ArtifactKind,
    /// Portable locator relative to [`ExecutionSubjectLock::package`].
    pub locator: String,
    pub digest: Integrity,
}

/// Closed v1 policy vocabulary. Future signature, attestation, or
/// transparency-log requirements require a new version rather than an unknown
/// claim silently becoming authoritative.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSubjectEvidencePolicy {
    pub content_digest: ContentDigestRequirement,
    pub lock_equality: LockEqualityRequirement,
    pub cryptographic_verification: CryptographicVerificationRequirement,
    pub publisher_identity: PublisherIdentityRequirement,
    pub operator_trust: OperatorTrustSource,
    pub transparency_log: TransparencyLogRequirement,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentDigestRequirement {
    Required,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LockEqualityRequirement {
    Required,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CryptographicVerificationRequirement {
    NotRequired,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PublisherIdentityRequirement {
    DeclarationCorrelation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperatorTrustSource {
    ExtensionLock,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransparencyLogRequirement {
    NotRequired,
}

/// Portable evidence produced from one execution-subject lock and invocation.
///
/// Deserialization validates a document's shape only. It cannot create
/// [`MatchedExecutionSubjects`], which requires a fresh filesystem observation
/// and exact lock comparison.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostExecutionSubjectObservationSet {
    pub schema_version: String,
    pub canonicalization: String,
    pub subject_lock_id: String,
    pub extension_lock_id: String,
    pub invocation_id: String,
    pub run_id: String,
    pub extension: InvocationExtension,
    pub capability_id: String,
    pub interface: InvocationInterface,
    pub declared_entrypoint: String,
    pub directory_manifest_profile: String,
    pub subjects: Vec<HostExecutionSubjectObservation>,
    pub claims: ExecutionSubjectClaims,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostExecutionSubjectObservation {
    pub subject_id: String,
    pub role: ExecutionSubjectRole,
    pub kind: ArtifactKind,
    /// Portable root-relative locator. Executable observations contain the
    /// package locator joined to the executable's package-relative locator.
    pub locator: String,
    pub digest: Integrity,
    pub size_bytes: u64,
    pub manifest: Vec<DirectoryManifestEntry>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionSubjectRole {
    Package,
    Executable,
}

/// Evidence claims remain deliberately separate. Only content observation and
/// equality to the supplied lock are established by this v1 implementation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSubjectClaims {
    pub content_digest: ContentDigestClaim,
    pub lock_equality: LockEqualityClaim,
    pub cryptographic_verification: CryptographicVerificationClaim,
    pub publisher_identity: PublisherIdentityClaim,
    pub operator_trust: OperatorTrustClaim,
    pub transparency_log: TransparencyLogClaim,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContentDigestClaim {
    pub status: ContentDigestStatus,
    pub algorithm: IntegrityAlgorithm,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentDigestStatus {
    Observed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LockEqualityClaim {
    pub status: LockEqualityStatus,
    pub subject_lock_digest: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LockEqualityStatus {
    Matched,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CryptographicVerificationClaim {
    pub status: CryptographicVerificationStatus,
    /// Closed v1 evidence collection. It must remain empty because this
    /// checkpoint implements no signature or attestation verifier.
    pub evidence: Vec<Value>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CryptographicVerificationStatus {
    NotPerformed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PublisherIdentityClaim {
    pub status: PublisherIdentityStatus,
    pub publisher_id: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PublisherIdentityStatus {
    DeclaredAndCorrelated,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperatorTrustClaim {
    pub status: OperatorTrustStatus,
    pub trust: Trust,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperatorTrustStatus {
    Configured,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TransparencyLogClaim {
    pub status: TransparencyLogStatus,
    /// Closed v1 evidence collection. It must remain empty because this
    /// checkpoint implements no log inclusion or checkpoint verifier.
    pub evidence: Vec<Value>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransparencyLogStatus {
    NotChecked,
}

/// Fresh Flow-owned subject observation that exactly matched its lock.
///
/// The fields are private so portable JSON, provider output, or a struct
/// literal cannot impersonate the result of [`observe_execution_subjects`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchedExecutionSubjects {
    evidence: HostExecutionSubjectObservationSet,
    lock_digest: String,
    observation_digest: String,
    context: ExecutionSubjectContext,
}

impl MatchedExecutionSubjects {
    #[must_use]
    pub const fn evidence(&self) -> &HostExecutionSubjectObservationSet {
        &self.evidence
    }

    #[must_use]
    pub fn lock_digest(&self) -> &str {
        &self.lock_digest
    }

    #[must_use]
    pub fn observation_digest(&self) -> &str {
        &self.observation_digest
    }

    #[must_use]
    pub fn into_evidence(self) -> HostExecutionSubjectObservationSet {
        self.evidence
    }

    pub(crate) fn matches_context(
        &self,
        resolved: &ResolvedExtension,
        invocation: &ExtensionInvocation,
        lock: &ExecutionSubjectLock,
    ) -> Result<(), String> {
        correlate_context(resolved, invocation, lock)?;
        let lock_digest = lock
            .canonical_digest()
            .map_err(|error| format!("execution-subject lock identity failed: {error}"))?;
        let expected =
            ExecutionSubjectContext::new(resolved, invocation, lock, lock_digest.clone());
        if self.context != expected
            || self.lock_digest != lock_digest
            || self.evidence.claims.lock_equality.subject_lock_digest != lock_digest
        {
            return Err(
                "the matched package/executable token does not belong to this lock and invocation"
                    .to_owned(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExecutionSubjectContext {
    subject_lock_digest: String,
    subject_lock_id: String,
    extension_lock_id: String,
    invocation_id: String,
    run_id: String,
    extension: InvocationExtension,
    capability_id: String,
    interface: InvocationInterface,
    declared_entrypoint: String,
    operator_trust: Trust,
}

impl ExecutionSubjectContext {
    fn new(
        resolved: &ResolvedExtension,
        invocation: &ExtensionInvocation,
        lock: &ExecutionSubjectLock,
        subject_lock_digest: String,
    ) -> Self {
        Self {
            subject_lock_digest,
            subject_lock_id: lock.subject_lock_id.clone(),
            extension_lock_id: lock.extension_lock_id.clone(),
            invocation_id: invocation.invocation_id.clone(),
            run_id: invocation.run_id.clone(),
            extension: lock.extension.clone(),
            capability_id: lock.capability_id.clone(),
            interface: lock.interface.clone(),
            declared_entrypoint: lock.declared_entrypoint.clone(),
            operator_trust: resolved.trust(),
        }
    }
}

impl ExecutionSubjectLock {
    /// Validate the closed lock and cross-field subject invariants.
    ///
    /// # Errors
    ///
    /// Returns the first deterministic contract violation.
    pub fn validate(&self) -> Result<(), ValidationError> {
        expect(
            self.schema_version == EXECUTION_SUBJECT_LOCK_V1,
            "subject_lock.schema_version",
            format!("must equal {EXECUTION_SUBJECT_LOCK_V1}"),
        )?;
        expect(
            self.canonicalization == CANONICAL_JSON_V1,
            "subject_lock.canonicalization",
            format!("must equal {CANONICAL_JSON_V1}"),
        )?;
        expect(
            has_prefixed_id(&self.subject_lock_id, "subject-lock:"),
            "subject_lock.subject_lock_id",
            "must be a lowercase subject-lock identifier",
        )?;
        expect(
            has_prefixed_id(&self.extension_lock_id, "lock:"),
            "subject_lock.extension_lock_id",
            "must be a lowercase extension-lock identifier",
        )?;
        validate_extension_identity("subject_lock.extension", &self.extension)?;
        expect(
            is_capability_id(&self.capability_id),
            "subject_lock.capability_id",
            "must be a qualified capability identifier",
        )?;
        validate_process_interface("subject_lock.interface", &self.interface)?;
        validate_entrypoint(
            "subject_lock.declared_entrypoint",
            &self.declared_entrypoint,
        )?;
        validate_locked_subject(
            "subject_lock.package",
            &self.package.subject_id,
            "package:",
            self.package.kind,
            ArtifactKind::Directory,
            &self.package.locator,
            &self.package.digest,
        )?;
        validate_locked_subject(
            "subject_lock.executable",
            &self.executable.subject_id,
            "executable:",
            self.executable.kind,
            ArtifactKind::File,
            &self.executable.locator,
            &self.executable.digest,
        )?;
        expect(
            self.package.subject_id != self.executable.subject_id,
            "subject_lock",
            "package and executable subject identifiers must be distinct",
        )?;
        expect(
            self.package.digest.value == self.extension.integrity,
            "subject_lock.package.digest",
            "package digest must equal the locked extension integrity",
        )?;
        let _full_executable_locator = full_executable_locator(self)?;
        Ok(())
    }

    /// Return deterministic compact canonical JSON bytes for lock identity.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid lock or serialization failure.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ExecutionSubjectError> {
        self.validate()
            .map_err(|source| ExecutionSubjectError::InvalidLock { source })?;
        canonical_json_bytes(self)
    }

    /// Return lowercase SHA-256 over [`Self::canonical_bytes`].
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid lock or serialization failure.
    pub fn canonical_digest(&self) -> Result<String, ExecutionSubjectError> {
        Ok(format!("{:x}", Sha256::digest(self.canonical_bytes()?)))
    }
}

impl HostExecutionSubjectObservationSet {
    /// Validate portable subject evidence and its internal claim separation.
    ///
    /// # Errors
    ///
    /// Returns the first deterministic contract or evidence contradiction.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_observation_metadata(self)?;
        validate_observation_subjects(self)?;
        validate_observation_claims(self)
    }

    /// Return deterministic compact canonical JSON bytes for evidence identity.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid evidence or serialization failure.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ExecutionSubjectError> {
        self.validate()
            .map_err(|source| ExecutionSubjectError::InvalidObservations { source })?;
        canonical_json_bytes(self)
    }

    /// Return lowercase SHA-256 over [`Self::canonical_bytes`].
    ///
    /// # Errors
    ///
    /// Returns an error for invalid evidence or serialization failure.
    pub fn canonical_digest(&self) -> Result<String, ExecutionSubjectError> {
        Ok(format!("{:x}", Sha256::digest(self.canonical_bytes()?)))
    }
}

fn validate_observation_metadata(
    observations: &HostExecutionSubjectObservationSet,
) -> Result<(), ValidationError> {
    expect(
        observations.schema_version == EXECUTION_SUBJECT_OBSERVATIONS_V1,
        "subject_observations.schema_version",
        format!("must equal {EXECUTION_SUBJECT_OBSERVATIONS_V1}"),
    )?;
    expect(
        observations.canonicalization == CANONICAL_JSON_V1,
        "subject_observations.canonicalization",
        format!("must equal {CANONICAL_JSON_V1}"),
    )?;
    expect(
        has_prefixed_id(&observations.subject_lock_id, "subject-lock:"),
        "subject_observations.subject_lock_id",
        "must be a lowercase subject-lock identifier",
    )?;
    expect(
        has_prefixed_id(&observations.extension_lock_id, "lock:"),
        "subject_observations.extension_lock_id",
        "must be a lowercase extension-lock identifier",
    )?;
    expect(
        has_prefixed_id(&observations.invocation_id, "invocation:"),
        "subject_observations.invocation_id",
        "must be a lowercase invocation identifier",
    )?;
    expect(
        has_prefixed_id(&observations.run_id, "run:"),
        "subject_observations.run_id",
        "must be a lowercase run identifier",
    )?;
    validate_extension_identity("subject_observations.extension", &observations.extension)?;
    expect(
        is_capability_id(&observations.capability_id),
        "subject_observations.capability_id",
        "must be a qualified capability identifier",
    )?;
    validate_process_interface("subject_observations.interface", &observations.interface)?;
    validate_entrypoint(
        "subject_observations.declared_entrypoint",
        &observations.declared_entrypoint,
    )?;
    expect(
        observations.directory_manifest_profile == DIRECTORY_MANIFEST_V1,
        "subject_observations.directory_manifest_profile",
        format!("must equal {DIRECTORY_MANIFEST_V1}"),
    )
}

fn validate_observation_subjects(
    observations: &HostExecutionSubjectObservationSet,
) -> Result<(), ValidationError> {
    expect(
        observations.subjects.len() == 2,
        "subject_observations.subjects",
        "must contain exactly one package followed by one executable",
    )?;
    let package = &observations.subjects[0];
    let executable = &observations.subjects[1];
    expect(
        package.role == ExecutionSubjectRole::Package
            && executable.role == ExecutionSubjectRole::Executable,
        "subject_observations.subjects",
        "subjects must use canonical package-then-executable order",
    )?;
    expect(
        package.subject_id != executable.subject_id,
        "subject_observations.subjects",
        "subject identifiers must be unique",
    )?;
    validate_observed_subject(package, 0)?;
    validate_observed_subject(executable, 1)?;
    expect(
        package.kind == ArtifactKind::Directory && executable.kind == ArtifactKind::File,
        "subject_observations.subjects",
        "the package must be a directory and the executable must be a file",
    )?;
    expect(
        package.digest.value == observations.extension.integrity,
        "subject_observations.subjects[0].digest",
        "package digest must equal the observed extension integrity",
    )?;
    let package_prefix = format!("{}/", package.locator);
    let relative_executable = executable
        .locator
        .strip_prefix(&package_prefix)
        .ok_or_else(|| {
            ValidationError::new(
                "subject_observations.subjects[1].locator",
                "executable must be a descendant of the package locator",
            )
        })?;
    validate_as_artifact_observations(observations)?;

    let matching_manifest_entries = package
        .manifest
        .iter()
        .filter(|entry| entry.locator == relative_executable)
        .collect::<Vec<_>>();
    expect(
        matching_manifest_entries.len() == 1,
        "subject_observations.subjects[0].manifest",
        "package manifest must contain the executable exactly once",
    )?;
    let manifest_executable = matching_manifest_entries[0];
    expect(
        manifest_executable.kind == ArtifactKind::File
            && manifest_executable.digest == executable.digest.value
            && manifest_executable.size_bytes == executable.size_bytes,
        "subject_observations.subjects",
        "package-manifest executable evidence contradicts the executable observation",
    )
}

fn validate_observation_claims(
    observations: &HostExecutionSubjectObservationSet,
) -> Result<(), ValidationError> {
    let claims = &observations.claims;
    validate_digest(
        "subject_observations.claims.lock_equality.subject_lock_digest",
        &claims.lock_equality.subject_lock_digest,
    )?;
    expect(
        claims.cryptographic_verification.evidence.is_empty(),
        "subject_observations.claims.cryptographic_verification.evidence",
        "v1 does not support cryptographic verification evidence",
    )?;
    expect(
        is_publisher_id(&claims.publisher_identity.publisher_id),
        "subject_observations.claims.publisher_identity.publisher_id",
        "must be a lowercase publisher identifier",
    )?;
    expect(
        claims.publisher_identity.publisher_id == observations.extension.publisher_id,
        "subject_observations.claims.publisher_identity",
        "publisher claim must correlate the declared extension publisher",
    )?;
    expect(
        claims.transparency_log.evidence.is_empty(),
        "subject_observations.claims.transparency_log.evidence",
        "v1 does not support transparency-log evidence",
    )
}

/// Observe and exactly match the locked package and executable subjects.
///
/// The function reads package and executable bytes but never executes them. It
/// binds the returned opaque token to the exact resolved extension, subject
/// lock, run, invocation, capability, process interface, declared entrypoint,
/// and operator trust mode.
///
/// # Errors
///
/// Returns a typed error for invalid contracts, context mismatch, unsafe
/// filesystem content, altered bytes, contradictory evidence, or canonical
/// identity failure.
pub fn observe_execution_subjects(
    root: &Path,
    resolved: &ResolvedExtension,
    invocation: &ExtensionInvocation,
    lock: &ExecutionSubjectLock,
) -> Result<MatchedExecutionSubjects, ExecutionSubjectError> {
    lock.validate()
        .map_err(|source| ExecutionSubjectError::InvalidLock { source })?;
    invocation
        .validate()
        .map_err(|source| ExecutionSubjectError::InvalidInvocation { source })?;
    correlate_context(resolved, invocation, lock)
        .map_err(|message| ExecutionSubjectError::ContextMismatch { message })?;

    let bindings = artifact_bindings(lock)?;
    let observed = observe_artifacts(root, &bindings)
        .map_err(|source| ExecutionSubjectError::Observation { source })?;
    let artifacts = &observed.evidence().artifacts;
    let package_artifact =
        artifacts
            .first()
            .ok_or_else(|| ExecutionSubjectError::ContradictoryEvidence {
                message: "Flow observation omitted the locked package".to_owned(),
            })?;
    let executable_artifact =
        artifacts
            .get(1)
            .ok_or_else(|| ExecutionSubjectError::ContradictoryEvidence {
                message: "Flow observation omitted the locked executable".to_owned(),
            })?;

    match_digest(
        &lock.executable.subject_id,
        &lock.executable.digest.value,
        &executable_artifact.digest,
    )?;
    match_digest(
        &lock.package.subject_id,
        &lock.package.digest.value,
        &package_artifact.digest,
    )?;

    let lock_digest = lock.canonical_digest()?;
    let evidence = HostExecutionSubjectObservationSet {
        schema_version: EXECUTION_SUBJECT_OBSERVATIONS_V1.to_owned(),
        canonicalization: CANONICAL_JSON_V1.to_owned(),
        subject_lock_id: lock.subject_lock_id.clone(),
        extension_lock_id: lock.extension_lock_id.clone(),
        invocation_id: invocation.invocation_id.clone(),
        run_id: invocation.run_id.clone(),
        extension: lock.extension.clone(),
        capability_id: lock.capability_id.clone(),
        interface: lock.interface.clone(),
        declared_entrypoint: lock.declared_entrypoint.clone(),
        directory_manifest_profile: DIRECTORY_MANIFEST_V1.to_owned(),
        subjects: vec![
            subject_observation(
                &lock.package.subject_id,
                ExecutionSubjectRole::Package,
                package_artifact,
            ),
            subject_observation(
                &lock.executable.subject_id,
                ExecutionSubjectRole::Executable,
                executable_artifact,
            ),
        ],
        claims: ExecutionSubjectClaims {
            content_digest: ContentDigestClaim {
                status: ContentDigestStatus::Observed,
                algorithm: IntegrityAlgorithm::Sha256,
            },
            lock_equality: LockEqualityClaim {
                status: LockEqualityStatus::Matched,
                subject_lock_digest: lock_digest.clone(),
            },
            cryptographic_verification: CryptographicVerificationClaim {
                status: CryptographicVerificationStatus::NotPerformed,
                evidence: Vec::new(),
            },
            publisher_identity: PublisherIdentityClaim {
                status: PublisherIdentityStatus::DeclaredAndCorrelated,
                publisher_id: lock.extension.publisher_id.clone(),
            },
            operator_trust: OperatorTrustClaim {
                status: OperatorTrustStatus::Configured,
                trust: resolved.trust(),
            },
            transparency_log: TransparencyLogClaim {
                status: TransparencyLogStatus::NotChecked,
                evidence: Vec::new(),
            },
        },
    };
    evidence
        .validate()
        .map_err(|source| ExecutionSubjectError::InvalidObservations { source })?;
    correlate_observations(lock, &evidence)?;
    let observation_digest = evidence.canonical_digest()?;
    let context = ExecutionSubjectContext::new(resolved, invocation, lock, lock_digest.clone());

    Ok(MatchedExecutionSubjects {
        evidence,
        lock_digest,
        observation_digest,
        context,
    })
}

#[derive(Debug, Error)]
pub enum ExecutionSubjectError {
    #[error("invalid execution-subject lock: {source}")]
    InvalidLock {
        #[source]
        source: ValidationError,
    },
    #[error("invalid process invocation: {source}")]
    InvalidInvocation {
        #[source]
        source: ValidationError,
    },
    #[error("execution-subject context mismatch: {message}")]
    ContextMismatch { message: String },
    #[error("execution-subject observation failed: {source}")]
    Observation {
        #[source]
        source: ArtifactObservationError,
    },
    #[error("invalid generated execution-subject observations: {source}")]
    InvalidObservations {
        #[source]
        source: ValidationError,
    },
    #[error("execution subject {subject_id} bytes do not match the lock")]
    ContentMismatch { subject_id: String },
    #[error("execution-subject evidence is contradictory: {message}")]
    ContradictoryEvidence { message: String },
    #[error("failed to encode deterministic execution-subject identity: {message}")]
    Canonicalization { message: String },
}

fn validate_extension_identity(
    path: &str,
    extension: &InvocationExtension,
) -> Result<(), ValidationError> {
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
    expect(
        !interface.name.is_empty(),
        format!("{path}.name"),
        "must not be empty",
    )?;
    expect(
        interface.protocol == EXTENSION_INVOCATION_V1,
        format!("{path}.protocol"),
        format!("must equal {EXTENSION_INVOCATION_V1}"),
    )
}

fn validate_entrypoint(path: &str, entrypoint: &str) -> Result<(), ValidationError> {
    expect(
        !entrypoint.is_empty() && !entrypoint.chars().any(char::is_control),
        path,
        "must be nonempty and contain no control characters",
    )
}

#[allow(clippy::too_many_arguments)]
fn validate_locked_subject(
    path: &str,
    subject_id: &str,
    id_prefix: &str,
    actual_kind: ArtifactKind,
    expected_kind: ArtifactKind,
    locator: &str,
    digest: &Integrity,
) -> Result<(), ValidationError> {
    expect(
        has_prefixed_id(subject_id, id_prefix),
        format!("{path}.subject_id"),
        format!("must be a lowercase {id_prefix} identifier"),
    )?;
    expect(
        actual_kind == expected_kind,
        format!("{path}.kind"),
        format!("must equal {expected_kind:?}"),
    )?;
    validate_locator(&format!("{path}.locator"), locator)?;
    validate_digest(&format!("{path}.digest.value"), &digest.value)
}

fn validate_observed_subject(
    subject: &HostExecutionSubjectObservation,
    index: usize,
) -> Result<(), ValidationError> {
    let path = format!("subject_observations.subjects[{index}]");
    let prefix = match subject.role {
        ExecutionSubjectRole::Package => "package:",
        ExecutionSubjectRole::Executable => "executable:",
    };
    expect(
        has_prefixed_id(&subject.subject_id, prefix),
        format!("{path}.subject_id"),
        format!("must be a lowercase {prefix} identifier"),
    )?;
    validate_locator(&format!("{path}.locator"), &subject.locator)?;
    validate_digest(&format!("{path}.digest.value"), &subject.digest.value)
}

fn full_executable_locator(lock: &ExecutionSubjectLock) -> Result<String, ValidationError> {
    let locator = format!("{}/{}", lock.package.locator, lock.executable.locator);
    validate_locator("subject_lock.executable.full_locator", &locator)?;
    Ok(locator)
}

fn artifact_bindings(
    lock: &ExecutionSubjectLock,
) -> Result<ArtifactBindingSet, ExecutionSubjectError> {
    let full_executable_locator = full_executable_locator(lock)
        .map_err(|source| ExecutionSubjectError::InvalidLock { source })?;
    Ok(ArtifactBindingSet {
        schema_version: ARTIFACT_BINDINGS_V1.to_owned(),
        binding_set_id: format!(
            "bindings:{}",
            lock.subject_lock_id
                .strip_prefix("subject-lock:")
                .unwrap_or("invalid")
        ),
        digest_algorithm: SHA256.to_owned(),
        inputs: vec![
            InputArtifactBinding {
                artifact_id: subject_artifact_id(
                    ExecutionSubjectRole::Package,
                    &lock.package.subject_id,
                ),
                port: PACKAGE_PORT.to_owned(),
                media_type: PACKAGE_ARTIFACT_MEDIA_TYPE.to_owned(),
                kind: ArtifactKind::Directory,
                locator: lock.package.locator.clone(),
                expected_digest: lock.package.digest.value.clone(),
            },
            InputArtifactBinding {
                artifact_id: subject_artifact_id(
                    ExecutionSubjectRole::Executable,
                    &lock.executable.subject_id,
                ),
                port: EXECUTABLE_PORT.to_owned(),
                media_type: EXECUTABLE_ARTIFACT_MEDIA_TYPE.to_owned(),
                kind: ArtifactKind::File,
                locator: full_executable_locator,
                expected_digest: lock.executable.digest.value.clone(),
            },
        ],
        outputs: Vec::new(),
    })
}

fn subject_artifact_id(role: ExecutionSubjectRole, subject_id: &str) -> String {
    let suffix = subject_id
        .split_once(':')
        .map_or(subject_id, |(_, value)| value);
    match role {
        ExecutionSubjectRole::Package => format!("artifact:execution-package-{suffix}"),
        ExecutionSubjectRole::Executable => format!("artifact:execution-executable-{suffix}"),
    }
}

fn subject_observation(
    subject_id: &str,
    role: ExecutionSubjectRole,
    artifact: &HostArtifactObservation,
) -> HostExecutionSubjectObservation {
    HostExecutionSubjectObservation {
        subject_id: subject_id.to_owned(),
        role,
        kind: artifact.kind,
        locator: artifact.locator.clone(),
        digest: Integrity {
            algorithm: IntegrityAlgorithm::Sha256,
            value: artifact.digest.clone(),
        },
        size_bytes: artifact.size_bytes,
        manifest: artifact.manifest.clone(),
    }
}

fn validate_as_artifact_observations(
    observations: &HostExecutionSubjectObservationSet,
) -> Result<(), ValidationError> {
    let artifact_observations = observations
        .subjects
        .iter()
        .map(|subject| HostArtifactObservation {
            artifact_id: subject_artifact_id(subject.role, &subject.subject_id),
            port: match subject.role {
                ExecutionSubjectRole::Package => PACKAGE_PORT,
                ExecutionSubjectRole::Executable => EXECUTABLE_PORT,
            }
            .to_owned(),
            media_type: match subject.role {
                ExecutionSubjectRole::Package => PACKAGE_ARTIFACT_MEDIA_TYPE,
                ExecutionSubjectRole::Executable => EXECUTABLE_ARTIFACT_MEDIA_TYPE,
            }
            .to_owned(),
            kind: subject.kind,
            locator: subject.locator.clone(),
            digest: subject.digest.value.clone(),
            size_bytes: subject.size_bytes,
            manifest: subject.manifest.clone(),
        })
        .collect();
    HostArtifactObservationSet {
        schema_version: ARTIFACT_OBSERVATIONS_V1.to_owned(),
        binding_set_id: format!(
            "bindings:{}",
            observations
                .subject_lock_id
                .strip_prefix("subject-lock:")
                .unwrap_or("invalid")
        ),
        digest_algorithm: SHA256.to_owned(),
        directory_manifest_profile: observations.directory_manifest_profile.clone(),
        artifacts: artifact_observations,
    }
    .validate()
}

fn correlate_context(
    resolved: &ResolvedExtension,
    invocation: &ExtensionInvocation,
    lock: &ExecutionSubjectLock,
) -> Result<(), String> {
    if invocation.phase != InvocationPhase::Execute {
        return Err("execution subjects require invocation phase execute".to_owned());
    }
    if lock.extension_lock_id != resolved.lock_id()
        || lock.extension_lock_id != invocation.authorization.lock_id
    {
        return Err("execution-subject lock references a different extension lock".to_owned());
    }
    if lock.extension.extension_id != resolved.extension_id()
        || lock.extension.version != resolved.version()
        || lock.extension.publisher_id != resolved.publisher_id()
        || lock.extension.integrity != resolved.integrity().value
        || lock.extension != invocation.extension
    {
        return Err("execution-subject provider identity does not match resolution".to_owned());
    }
    if lock.capability_id != resolved.capability().capability_id
        || lock.capability_id != invocation.capability_id
    {
        return Err("execution-subject capability does not match resolution".to_owned());
    }
    if lock.interface != invocation.interface
        || lock.interface.name != resolved.execution_mode().name
        || lock.interface.kind != resolved.execution_mode().kind
        || lock.interface.protocol != resolved.execution_mode().protocol
    {
        return Err("execution-subject process interface does not match resolution".to_owned());
    }
    if lock.declared_entrypoint != resolved.execution_mode().entrypoint {
        return Err(
            "execution-subject entrypoint does not match the provider declaration".to_owned(),
        );
    }
    Ok(())
}

fn correlate_observations(
    lock: &ExecutionSubjectLock,
    observations: &HostExecutionSubjectObservationSet,
) -> Result<(), ExecutionSubjectError> {
    let package = observations.subjects.first().ok_or_else(|| {
        ExecutionSubjectError::ContradictoryEvidence {
            message: "package observation is missing".to_owned(),
        }
    })?;
    let executable = observations.subjects.get(1).ok_or_else(|| {
        ExecutionSubjectError::ContradictoryEvidence {
            message: "executable observation is missing".to_owned(),
        }
    })?;
    let full_executable_locator = full_executable_locator(lock)
        .map_err(|source| ExecutionSubjectError::InvalidLock { source })?;
    if package.subject_id != lock.package.subject_id
        || package.role != ExecutionSubjectRole::Package
        || package.kind != lock.package.kind
        || package.locator != lock.package.locator
        || package.digest != lock.package.digest
        || executable.subject_id != lock.executable.subject_id
        || executable.role != ExecutionSubjectRole::Executable
        || executable.kind != lock.executable.kind
        || executable.locator != full_executable_locator
        || executable.digest != lock.executable.digest
    {
        return Err(ExecutionSubjectError::ContradictoryEvidence {
            message: "observed subject identity, locator, kind, or digest conflicts with the lock"
                .to_owned(),
        });
    }
    Ok(())
}

fn match_digest(
    subject_id: &str,
    expected: &str,
    observed: &str,
) -> Result<(), ExecutionSubjectError> {
    if expected == observed {
        Ok(())
    } else {
        Err(ExecutionSubjectError::ContentMismatch {
            subject_id: subject_id.to_owned(),
        })
    }
}

fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, ExecutionSubjectError> {
    let value =
        serde_json::to_value(value).map_err(|error| ExecutionSubjectError::Canonicalization {
            message: error.to_string(),
        })?;
    serde_json::to_vec(&sort_json(value)).map_err(|error| ExecutionSubjectError::Canonicalization {
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
