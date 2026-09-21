use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use flow::{
    ArtifactKind, Authorization, AuthorizedProcess, Configuration, Domain, EXTENSION_INVOCATION_V1,
    ExecutionModeKind, ExecutionSubjectLock, ExtensionCatalog, ExtensionInvocation, ExtensionLock,
    ExtensionManifest, ExtensionObservation, ExtensionPort, HermeticExtension, InvocationExtension,
    InvocationInterface, InvocationPhase, MatchedExecutionSubjects, Orchestrator, PortIdentity,
    ProcessAuthorityProfile, ProcessCompletion, ProcessEnforcementEvidence, ProcessTranscript,
    ResolutionRequest, ResolvedExtension, Trust, authorize_process, observe_execution_subjects,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const CONFIG_DIGEST: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const GRANTS_DIGEST: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const PACKAGE_LOCATOR: &str = "packages/synthetic adapter";
const EXECUTABLE_LOCATOR: &str = "flow synthetic adapter";
const EXECUTABLE_BYTES: &[u8] = b"synthetic executable\n";

struct ExampleRoot {
    path: PathBuf,
}

impl ExampleRoot {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!(
            "flow-hermetic-process-example-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(path.join(PACKAGE_LOCATOR))?;
        fs::write(
            path.join(PACKAGE_LOCATOR).join(EXECUTABLE_LOCATOR),
            EXECUTABLE_BYTES,
        )?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ExampleRoot {
    fn drop(&mut self) {
        let _cleanup_result = fs::remove_dir_all(&self.path);
    }
}

#[derive(Serialize)]
struct CanonicalDirectoryChild<'a> {
    name: &'a str,
    kind: ArtifactKind,
    digest: &'a str,
    size_bytes: u64,
}

fn package_identity() -> Result<(String, String), Box<dyn std::error::Error>> {
    let executable_digest = format!("{:x}", Sha256::digest(EXECUTABLE_BYTES));
    let package_manifest = serde_json::to_vec(&[CanonicalDirectoryChild {
        name: EXECUTABLE_LOCATOR,
        kind: ArtifactKind::File,
        digest: &executable_digest,
        size_bytes: EXECUTABLE_BYTES.len() as u64,
    }])?;
    let package_digest = format!("{:x}", Sha256::digest(package_manifest));
    Ok((package_digest, executable_digest))
}

fn catalog(package_digest: &str) -> Result<ExtensionCatalog, Box<dyn std::error::Error>> {
    let mut manifest: ExtensionManifest = serde_json::from_str(include_str!(
        "../contracts/examples/extension-manifest.v1.example.json"
    ))?;
    package_digest.clone_into(&mut manifest.integrity.value);
    let mut lock: ExtensionLock = serde_json::from_str(include_str!(
        "../contracts/examples/extension-lock.v1.example.json"
    ))?;
    lock.extensions[0].trust = Trust::Trusted;
    package_digest.clone_into(&mut lock.extensions[0].integrity.value);
    let observation = ExtensionObservation::new(
        manifest.extension_id.clone(),
        manifest.version.clone(),
        manifest.publisher.id.clone(),
        manifest.integrity.clone(),
        true,
    );
    Ok(ExtensionCatalog::inspect([manifest], lock, [observation])?)
}

fn invocation(resolved: &ResolvedExtension) -> ExtensionInvocation {
    ExtensionInvocation {
        schema_version: EXTENSION_INVOCATION_V1.to_owned(),
        invocation_id: "invocation:hermetic-process-example".to_owned(),
        run_id: "run:hermetic-process-example".to_owned(),
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
            protocol: resolved.execution_mode().protocol.clone(),
        },
        input_artifacts: Vec::new(),
        expected_output_types: resolved.capability().produces.clone(),
        configuration: Configuration {
            schema_id: resolved.capability().configuration_schema.clone(),
            digest: CONFIG_DIGEST.to_owned(),
            values: BTreeMap::new(),
        },
        authorization: Authorization {
            authorization_id: "authorization:hermetic-process-example".to_owned(),
            lock_id: resolved.lock_id().to_owned(),
            grants_digest: GRANTS_DIGEST.to_owned(),
        },
        limits: resolved.execution_mode().limits.clone(),
        checkpoint_refs: Vec::new(),
        secret_handles: Vec::new(),
        cancellation_id: "cancel:hermetic-process-example".to_owned(),
    }
}

fn subject_lock(
    package_digest: &str,
    executable_digest: &str,
) -> Result<ExecutionSubjectLock, Box<dyn std::error::Error>> {
    let mut lock: ExecutionSubjectLock = serde_json::from_str(include_str!(
        "../contracts/examples/execution-subject-lock.v1.example.json"
    ))?;
    package_digest.clone_into(&mut lock.extension.integrity);
    package_digest.clone_into(&mut lock.package.digest.value);
    executable_digest.clone_into(&mut lock.executable.digest.value);
    Ok(lock)
}

fn fixture_stdout(
    resolved: &ResolvedExtension,
    invocation: &ExtensionInvocation,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let fixture = HermeticExtension::new(PortIdentity::from_resolved(resolved));
    let mut provider_events = Vec::new();
    let provider_result = fixture.invoke(invocation, &mut provider_events)?;
    let mut stdout = Vec::new();
    for event in provider_events {
        serde_json::to_writer(&mut stdout, &event)?;
        stdout.push(b'\n');
    }
    serde_json::to_writer(&mut stdout, &provider_result)?;
    stdout.push(b'\n');
    Ok(stdout)
}

fn process_authorization(
    resolved: &ResolvedExtension,
    invocation: &ExtensionInvocation,
    subject_lock: &ExecutionSubjectLock,
    subjects: &MatchedExecutionSubjects,
) -> Result<AuthorizedProcess, Box<dyn std::error::Error>> {
    let mut profile: ProcessAuthorityProfile = serde_json::from_str(include_str!(
        "../contracts/examples/process-authority-profile.v1.example.json"
    ))?;
    resolved
        .lock_id()
        .clone_into(&mut profile.extension_lock_id);
    invocation
        .authorization
        .authorization_id
        .clone_into(&mut profile.authorization_id);
    invocation
        .authorization
        .grants_digest
        .clone_into(&mut profile.grants_digest);
    invocation
        .invocation_id
        .clone_into(&mut profile.invocation_id);
    invocation.run_id.clone_into(&mut profile.run_id);
    profile.extension = invocation.extension.clone();
    invocation
        .capability_id
        .clone_into(&mut profile.capability_id);
    profile.interface = invocation.interface.clone();
    subject_lock
        .subject_lock_id
        .clone_into(&mut profile.subject_lock_id);
    subjects
        .lock_digest()
        .clone_into(&mut profile.subject_lock_digest);
    profile.operator_trust = resolved.trust();

    let mut evidence: ProcessEnforcementEvidence = serde_json::from_str(include_str!(
        "../contracts/examples/process-enforcement-evidence.v1.example.json"
    ))?;
    profile
        .authority_profile_id
        .clone_into(&mut evidence.authority_profile_id);
    profile
        .canonical_digest()?
        .clone_into(&mut evidence.authority_profile_digest);
    profile
        .invocation_id
        .clone_into(&mut evidence.invocation_id);
    profile.run_id.clone_into(&mut evidence.run_id);
    evidence.extension = profile.extension.clone();
    profile
        .capability_id
        .clone_into(&mut evidence.capability_id);
    evidence.interface = profile.interface.clone();
    profile
        .subject_lock_id
        .clone_into(&mut evidence.subject_lock_id);
    profile
        .subject_lock_digest
        .clone_into(&mut evidence.subject_lock_digest);
    subjects
        .observation_digest()
        .clone_into(&mut evidence.subject_observation_digest);

    Ok(authorize_process(
        resolved,
        invocation,
        subject_lock,
        subjects,
        &profile,
        &evidence,
    )?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = ExampleRoot::new()?;
    let (package_digest, executable_digest) = package_identity()?;
    let catalog = catalog(&package_digest)?;
    let resolution = catalog.resolve(&ResolutionRequest::new(
        "hermetic-process-example",
        "optiflow/inspect-collection",
        Domain::Optiflow,
        "synthetic-process",
        ExecutionModeKind::Process,
    ));
    let resolved = resolution
        .resolved()
        .ok_or("the hermetic process interface did not resolve")?;
    let invocation = invocation(resolved);
    let subject_lock = subject_lock(&package_digest, &executable_digest)?;
    let subjects = observe_execution_subjects(root.path(), resolved, &invocation, &subject_lock)?;
    let authority = process_authorization(resolved, &invocation, &subject_lock, &subjects)?;

    let request = Orchestrator::encode_process_request(
        resolved,
        &invocation,
        &subject_lock,
        &subjects,
        &authority,
    )?;

    // The hermetic port only manufactures deterministic fixture evidence. No
    // operating-system child is launched by this example or the Flow library.
    let provider_stdout = fixture_stdout(resolved, &invocation)?;

    let mut observed = Vec::new();
    let execution = Orchestrator::validate_process_transcript(
        resolved,
        &invocation,
        &subject_lock,
        &subjects,
        &authority,
        ProcessTranscript::new(
            ProcessCompletion::Exited { code: Some(0) },
            &provider_stdout,
            &[],
        ),
        &mut observed,
    )?;

    println!(
        "{} matched {} package/executable observations, encoded {} request bytes, and validated {} process events with outcome {:?}",
        resolution.evidence().case_id,
        subjects.evidence().subjects.len(),
        request.len(),
        execution.events().len(),
        execution.result().outcome
    );
    Ok(())
}
