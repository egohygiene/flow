#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use flow::{
    AmbientAuthorityPolicy, AuthorityDimension, Authorization, AuthorizedProcess, Configuration,
    Domain, EXTENSION_INVOCATION_V1, EnforcementEvidenceSource, EnforcementGuarantee,
    EnforcementStatus, ExecutionModeKind, ExecutionSubjectLock, ExtensionCatalog,
    ExtensionInvocation, ExtensionLock, ExtensionManifest, ExtensionObservation, GpuAccess,
    GpuCapability, InvocationExtension, InvocationInterface, InvocationPhase,
    MatchedExecutionSubjects, PROCESS_AUTHORITY_PROFILE_V1, PROCESS_ENFORCEMENT_EVIDENCE_V1,
    ProcessAuthority, ProcessAuthorityProfile, ProcessEnforcementEvidence, ProcessIsolation,
    ResolutionRequest, ResolvedExtension, TelemetryPropagation, authorize_process,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

pub const CONFIG_DIGEST: &str = "3333333333333333333333333333333333333333333333333333333333333333";
pub const GRANTS_DIGEST: &str = "4444444444444444444444444444444444444444444444444444444444444444";

const PROCESS_PACKAGE_LOCATOR: &str = "packages/synthetic adapter";
const PROCESS_EXECUTABLE_LOCATOR: &str = "flow synthetic adapter";
const PROCESS_EXECUTABLE_BYTES: &[u8] = b"synthetic executable\n";
const PROCESS_UNICODE_FILE: &str = "説明.txt";
const PROCESS_UNICODE_BYTES: &[u8] = b"portable package evidence\n";

static NEXT_PROCESS_ROOT_ID: AtomicU64 = AtomicU64::new(0);

pub struct ProcessSubjectFixture {
    pub root: ProcessSubjectRoot,
    pub catalog: ExtensionCatalog,
    pub request: ResolutionRequest,
    pub subject_lock: ExecutionSubjectLock,
}

pub struct ProcessSubjectRoot {
    path: PathBuf,
}

impl ProcessSubjectRoot {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn executable_path(&self) -> PathBuf {
        self.path
            .join(PROCESS_PACKAGE_LOCATOR)
            .join(PROCESS_EXECUTABLE_LOCATOR)
    }

    pub fn package_path(&self) -> PathBuf {
        self.path.join(PROCESS_PACKAGE_LOCATOR)
    }
}

impl Drop for ProcessSubjectRoot {
    fn drop(&mut self) {
        let _cleanup_result = fs::remove_dir_all(&self.path);
    }
}

#[derive(Serialize)]
struct CanonicalDirectoryChild<'a> {
    name: &'a str,
    kind: flow::ArtifactKind,
    digest: &'a str,
    size_bytes: u64,
}

pub fn manifest() -> ExtensionManifest {
    serde_json::from_str(include_str!(
        "../../contracts/examples/extension-manifest.v1.example.json"
    ))
    .expect("checked-in manifest example must deserialize")
}

pub fn lock() -> ExtensionLock {
    let mut lock: ExtensionLock = serde_json::from_str(include_str!(
        "../../contracts/examples/extension-lock.v1.example.json"
    ))
    .expect("checked-in lock example must deserialize");
    lock.extensions[0].trust = flow::Trust::Trusted;
    lock
}

pub fn observation(manifest: &ExtensionManifest, available: bool) -> ExtensionObservation {
    ExtensionObservation::new(
        manifest.extension_id.clone(),
        manifest.version.clone(),
        manifest.publisher.id.clone(),
        manifest.integrity.clone(),
        available,
    )
}

pub fn request() -> ResolutionRequest {
    ResolutionRequest::new(
        "test-case",
        "optiflow/inspect-collection",
        Domain::Optiflow,
        "synthetic-library",
        ExecutionModeKind::InProcess,
    )
}

pub fn process_request() -> ResolutionRequest {
    ResolutionRequest::new(
        "test-case-process",
        "optiflow/inspect-collection",
        Domain::Optiflow,
        "synthetic-process",
        ExecutionModeKind::Process,
    )
}

pub fn resolved_fixture() -> (ExtensionCatalog, ResolutionRequest) {
    let manifest = manifest();
    let observation = observation(&manifest, true);
    let catalog = ExtensionCatalog::inspect([manifest], lock(), [observation])
        .expect("fixture catalog must inspect");
    (catalog, request())
}

pub fn process_subject_fixture() -> ProcessSubjectFixture {
    process_subject_fixture_with_settings(flow::Trust::Trusted, None)
}

pub fn process_subject_fixture_with_trust(trust: flow::Trust) -> ProcessSubjectFixture {
    process_subject_fixture_with_settings(trust, None)
}

pub fn process_subject_fixture_with_permissions(
    trust: flow::Trust,
    requested: flow::Permissions,
    granted: flow::Permissions,
) -> ProcessSubjectFixture {
    process_subject_fixture_with_settings(trust, Some((requested, granted)))
}

pub fn process_runner_fixture(
    executable_bytes: &[u8],
    requested: flow::Permissions,
    granted: flow::Permissions,
) -> ProcessSubjectFixture {
    process_subject_fixture_with_executable(
        flow::Trust::Trusted,
        Some((requested, granted)),
        executable_bytes,
        true,
    )
}

fn process_subject_fixture_with_settings(
    trust: flow::Trust,
    permissions: Option<(flow::Permissions, flow::Permissions)>,
) -> ProcessSubjectFixture {
    process_subject_fixture_with_executable(trust, permissions, PROCESS_EXECUTABLE_BYTES, false)
}

fn process_subject_fixture_with_executable(
    trust: flow::Trust,
    permissions: Option<(flow::Permissions, flow::Permissions)>,
    executable_bytes: &[u8],
    make_executable: bool,
) -> ProcessSubjectFixture {
    #[cfg(not(unix))]
    let _ = make_executable;

    let sequence = NEXT_PROCESS_ROOT_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "flow-process-subject-test-{}-{sequence}",
        std::process::id()
    ));
    let package_path = path.join(PROCESS_PACKAGE_LOCATOR);
    fs::create_dir(&path).expect("process-subject test root must be new");
    fs::create_dir_all(&package_path).expect("process package directory must be created");
    fs::write(
        package_path.join(PROCESS_EXECUTABLE_LOCATOR),
        executable_bytes,
    )
    .expect("process executable fixture must be written");
    #[cfg(unix)]
    if make_executable {
        use std::os::unix::fs::PermissionsExt;

        let executable_path = package_path.join(PROCESS_EXECUTABLE_LOCATOR);
        let mut permissions = fs::metadata(&executable_path)
            .expect("process executable fixture metadata must be available")
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(executable_path, permissions)
            .expect("process executable fixture must be executable");
    }
    fs::write(
        package_path.join(PROCESS_UNICODE_FILE),
        PROCESS_UNICODE_BYTES,
    )
    .expect("Unicode package fixture must be written");
    let root = ProcessSubjectRoot { path };

    let executable_digest = format!("{:x}", Sha256::digest(executable_bytes));
    let unicode_digest = format!("{:x}", Sha256::digest(PROCESS_UNICODE_BYTES));
    let canonical_package = serde_json::to_vec(&[
        CanonicalDirectoryChild {
            name: PROCESS_EXECUTABLE_LOCATOR,
            kind: flow::ArtifactKind::File,
            digest: &executable_digest,
            size_bytes: executable_bytes.len() as u64,
        },
        CanonicalDirectoryChild {
            name: PROCESS_UNICODE_FILE,
            kind: flow::ArtifactKind::File,
            digest: &unicode_digest,
            size_bytes: PROCESS_UNICODE_BYTES.len() as u64,
        },
    ])
    .expect("package manifest fixture must serialize");
    let package_digest = format!("{:x}", Sha256::digest(canonical_package));

    let mut manifest = manifest();
    if let Some((requested, _)) = &permissions {
        manifest.requested_permissions = requested.clone();
    }
    package_digest.clone_into(&mut manifest.integrity.value);
    let observation = observation(&manifest, true);
    let mut extension_lock = lock();
    package_digest.clone_into(&mut extension_lock.extensions[0].integrity.value);
    extension_lock.extensions[0].trust = trust;
    if let Some((_, granted)) = permissions {
        extension_lock.extensions[0].granted_permissions = granted;
    }
    let catalog = ExtensionCatalog::inspect([manifest], extension_lock, [observation])
        .expect("process-subject catalog must inspect");

    let mut subject_lock: ExecutionSubjectLock = serde_json::from_str(include_str!(
        "../../contracts/examples/execution-subject-lock.v1.example.json"
    ))
    .expect("checked-in execution-subject lock must deserialize");
    package_digest.clone_into(&mut subject_lock.extension.integrity);
    package_digest.clone_into(&mut subject_lock.package.digest.value);
    executable_digest.clone_into(&mut subject_lock.executable.digest.value);
    subject_lock
        .validate()
        .expect("process-subject lock fixture must validate");

    ProcessSubjectFixture {
        root,
        catalog,
        request: process_request(),
        subject_lock,
    }
}

#[cfg(unix)]
pub fn create_unsupported_node(path: &Path) {
    let status = Command::new("mkfifo")
        .arg(path)
        .status()
        .expect("mkfifo must be available on supported Unix test hosts");
    assert!(
        status.success(),
        "mkfifo must create the special-node fixture"
    );
}

pub fn invocation(resolved: &ResolvedExtension) -> ExtensionInvocation {
    ExtensionInvocation {
        schema_version: EXTENSION_INVOCATION_V1.to_owned(),
        invocation_id: "invocation:test".to_owned(),
        run_id: "run:test".to_owned(),
        phase: InvocationPhase::Execute,
        extension: InvocationExtension {
            extension_id: resolved.extension_id().to_owned(),
            version: resolved.version().to_owned(),
            publisher_id: resolved.publisher_id().to_owned(),
            integrity: resolved.integrity().value.clone(),
        },
        capability_id: resolved.capability().capability_id.clone(),
        interface: InvocationInterface {
            kind: resolved.execution_mode().kind,
            name: resolved.execution_mode().name.clone(),
            protocol: EXTENSION_INVOCATION_V1.to_owned(),
        },
        input_artifacts: Vec::new(),
        expected_output_types: resolved.capability().produces.clone(),
        configuration: Configuration {
            schema_id: resolved.capability().configuration_schema.clone(),
            digest: CONFIG_DIGEST.to_owned(),
            values: BTreeMap::new(),
        },
        authorization: Authorization {
            authorization_id: "authorization:test".to_owned(),
            lock_id: resolved.lock_id().to_owned(),
            grants_digest: GRANTS_DIGEST.to_owned(),
        },
        limits: resolved.execution_mode().limits.clone(),
        checkpoint_refs: Vec::new(),
        secret_handles: Vec::new(),
        cancellation_id: "cancel:test".to_owned(),
    }
}

pub fn process_authority_profile(
    resolved: &ResolvedExtension,
    invocation: &ExtensionInvocation,
    subject_lock: &ExecutionSubjectLock,
    subjects: &MatchedExecutionSubjects,
    isolation: ProcessIsolation,
) -> ProcessAuthorityProfile {
    ProcessAuthorityProfile {
        schema_version: PROCESS_AUTHORITY_PROFILE_V1.to_owned(),
        canonicalization: flow::CANONICAL_JSON_V1.to_owned(),
        authority_profile_id: "authority-profile:test".to_owned(),
        extension_lock_id: resolved.lock_id().to_owned(),
        authorization_id: invocation.authorization.authorization_id.clone(),
        grants_digest: invocation.authorization.grants_digest.clone(),
        invocation_id: invocation.invocation_id.clone(),
        run_id: invocation.run_id.clone(),
        extension: invocation.extension.clone(),
        capability_id: invocation.capability_id.clone(),
        interface: invocation.interface.clone(),
        subject_lock_id: subject_lock.subject_lock_id.clone(),
        subject_lock_digest: subjects.lock_digest().to_owned(),
        operator_trust: resolved.trust(),
        isolation,
        ambient_authority: AmbientAuthorityPolicy::DenyUnlisted,
        requested: authority_from_permissions(resolved.requested_permissions()),
        granted: authority_from_permissions(resolved.granted_permissions()),
    }
}

pub fn process_enforcement_evidence(
    profile: &ProcessAuthorityProfile,
    subjects: &MatchedExecutionSubjects,
) -> ProcessEnforcementEvidence {
    let (source, backend_id, backend_version, enforced, status) = match profile.isolation {
        ProcessIsolation::TrustedUnconfined => (
            EnforcementEvidenceSource::None,
            "none".to_owned(),
            "0.0.0".to_owned(),
            ProcessAuthority::default(),
            EnforcementStatus::NotEnforced,
        ),
        ProcessIsolation::Sandboxed => (
            EnforcementEvidenceSource::CallerAttestedHost,
            "backend:synthetic-conformance".to_owned(),
            "1.0.0".to_owned(),
            profile.requested.clone(),
            EnforcementStatus::Enforced,
        ),
    };
    ProcessEnforcementEvidence {
        schema_version: PROCESS_ENFORCEMENT_EVIDENCE_V1.to_owned(),
        canonicalization: flow::CANONICAL_JSON_V1.to_owned(),
        enforcement_evidence_id: "enforcement:test".to_owned(),
        authority_profile_id: profile.authority_profile_id.clone(),
        authority_profile_digest: profile.canonical_digest().unwrap(),
        invocation_id: profile.invocation_id.clone(),
        run_id: profile.run_id.clone(),
        extension: profile.extension.clone(),
        capability_id: profile.capability_id.clone(),
        interface: profile.interface.clone(),
        subject_lock_id: profile.subject_lock_id.clone(),
        subject_lock_digest: profile.subject_lock_digest.clone(),
        subject_observation_digest: subjects.observation_digest().to_owned(),
        isolation: profile.isolation,
        ambient_authority: profile.ambient_authority,
        source,
        backend_id,
        backend_version,
        enforced,
        guarantees: authority_dimensions()
            .into_iter()
            .map(|dimension| EnforcementGuarantee { dimension, status })
            .collect(),
        unsupported_claims: Vec::new(),
    }
}

pub fn authorized_process(
    resolved: &ResolvedExtension,
    invocation: &ExtensionInvocation,
    subject_lock: &ExecutionSubjectLock,
    subjects: &MatchedExecutionSubjects,
) -> AuthorizedProcess {
    let profile = process_authority_profile(
        resolved,
        invocation,
        subject_lock,
        subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    let evidence = process_enforcement_evidence(&profile, subjects);
    authorize_process(
        resolved,
        invocation,
        subject_lock,
        subjects,
        &profile,
        &evidence,
    )
    .expect("fixture process authority must match")
}

fn authority_from_permissions(permissions: &flow::Permissions) -> ProcessAuthority {
    let mut environment = permissions
        .environment_read
        .iter()
        .map(|name| flow::EnvironmentBinding {
            name: name.clone(),
            source_handle: format!("secret:env.{}", name.to_ascii_lowercase()),
        })
        .collect::<Vec<_>>();
    environment.sort();
    let mut filesystem_read = permissions.filesystem_read.clone();
    filesystem_read.sort();
    let mut filesystem_write = permissions.filesystem_write.clone();
    filesystem_write.sort();
    let mut network_endpoints = permissions.network_hosts.clone();
    network_endpoints.sort();
    let mut subprocesses = permissions.subprocesses.clone();
    subprocesses.sort();
    let mut ai_providers = permissions.ai_providers.clone();
    ai_providers.sort();
    ProcessAuthority {
        argv: vec!["--contract".to_owned(), EXTENSION_INVOCATION_V1.to_owned()],
        environment,
        filesystem_read,
        filesystem_write,
        network_endpoints,
        subprocesses,
        ai_providers,
        gpus: if permissions.gpu {
            vec![GpuAccess {
                device_id: "gpu:synthetic-0".to_owned(),
                capabilities: vec![GpuCapability::Compute],
            }]
        } else {
            Vec::new()
        },
        source_mutation_targets: permissions
            .source_mutation
            .then(|| "artifact:authorized-source".to_owned())
            .into_iter()
            .collect(),
        destructive_operations: permissions
            .destructive
            .then(|| "operation:authorized-delete".to_owned())
            .into_iter()
            .collect(),
        publication_destinations: permissions
            .publish
            .then(|| "destination:authorized-publication".to_owned())
            .into_iter()
            .collect(),
        signing_key_handles: permissions
            .sign
            .then(|| "handle:signing.test".to_owned())
            .into_iter()
            .collect(),
        telemetry: TelemetryPropagation::default(),
    }
}

fn authority_dimensions() -> [AuthorityDimension; 13] {
    [
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
    ]
}

pub fn variant(
    base: &ExtensionManifest,
    extension_id: &str,
    digest_character: char,
) -> ExtensionManifest {
    let mut variant = base.clone();
    extension_id.clone_into(&mut variant.extension_id);
    variant.integrity.value = digest_character.to_string().repeat(64);
    variant
}
