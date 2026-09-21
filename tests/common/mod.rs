#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use flow::{
    Authorization, Configuration, Domain, EXTENSION_INVOCATION_V1, ExecutionModeKind,
    ExecutionSubjectLock, ExtensionCatalog, ExtensionInvocation, ExtensionLock, ExtensionManifest,
    ExtensionObservation, InvocationExtension, InvocationInterface, InvocationPhase,
    ResolutionRequest, ResolvedExtension,
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
        PROCESS_EXECUTABLE_BYTES,
    )
    .expect("process executable fixture must be written");
    fs::write(
        package_path.join(PROCESS_UNICODE_FILE),
        PROCESS_UNICODE_BYTES,
    )
    .expect("Unicode package fixture must be written");
    let root = ProcessSubjectRoot { path };

    let executable_digest = format!("{:x}", Sha256::digest(PROCESS_EXECUTABLE_BYTES));
    let unicode_digest = format!("{:x}", Sha256::digest(PROCESS_UNICODE_BYTES));
    let canonical_package = serde_json::to_vec(&[
        CanonicalDirectoryChild {
            name: PROCESS_EXECUTABLE_LOCATOR,
            kind: flow::ArtifactKind::File,
            digest: &executable_digest,
            size_bytes: PROCESS_EXECUTABLE_BYTES.len() as u64,
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
    package_digest.clone_into(&mut manifest.integrity.value);
    let observation = observation(&manifest, true);
    let mut extension_lock = lock();
    package_digest.clone_into(&mut extension_lock.extensions[0].integrity.value);
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
