mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use flow::{
    ARTIFACT_BINDINGS_V1, AcceptedArtifactSet, ArtifactBindingSet, ArtifactKind, Authorization,
    CheckpointMode, Configuration, Domain, EXECUTION_SUBJECT_LOCK_V1, EventKind, EventState,
    ExecutionModeKind, ExecutionSubjectLock, ExtensionCatalog, ExtensionInvocation, ExtensionLock,
    ExtensionManifest, ExtensionObservation, ExtensionResolution, HostArtifactObservationSet,
    HostExecutionSubjectObservationSet, InputArtifact, InputArtifactBinding, InvocationExtension,
    InvocationInterface, InvocationPhase, LocalProcessRunner, NoSecrets, OutputArtifactBinding,
    PROCESS_AUTHORITY_PROFILE_V1, ProcessIsolation, ProvenanceKind, ResolutionRequest, SHA256,
    Trust, ValidatedExecution, ValidationStatus, accept_artifacts, authorize_process,
    observe_artifacts, observe_execution_subjects,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use common::{process_authority_profile, process_enforcement_evidence};

const PROVIDER_BINARY: &str = env!("CARGO_BIN_EXE_flow-hermetic-provider");
const MANIFEST_TEMPLATE: &str =
    include_str!("fixtures/hermetic-provider/extension-manifest.v1.json");
const LOCK_TEMPLATE: &str = include_str!("fixtures/hermetic-provider/extension-lock.v1.json");
const LICENSE_BYTES: &[u8] = include_bytes!("../LICENSE");
const INPUT_BYTES: &[u8] =
    include_bytes!("../contracts/fixtures/scenarios/sources/source-text.txt");
const PACKAGE_LOCATOR: &str = "packages/hermetic-provider";
const WORKSPACE_LOCATOR: &str = "workspace";
const BINDINGS_LOCATOR: &str = "artifact-bindings.v1.json";
const INPUT_LOCATOR: &str = "inputs/source-text.txt";
const INPUT_ID: &str = "artifact:source-text";
const CONFIGURATION_SCHEMA: &str = "flow.hermetic-provider-configuration/v1";

static NEXT_ROOT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy)]
struct CapabilitySpec {
    capability_id: &'static str,
    name: &'static str,
    output_media_type: &'static str,
}

const CAPABILITIES: [CapabilitySpec; 4] = [
    CapabilitySpec {
        capability_id: "flow/inspect-fixture",
        name: "inspection",
        output_media_type: "application/vnd.flow.fixture-inspection+json",
    },
    CapabilitySpec {
        capability_id: "flow/transform-fixture",
        name: "transformation",
        output_media_type: "application/vnd.flow.fixture-transformation+json",
    },
    CapabilitySpec {
        capability_id: "flow/validate-fixture",
        name: "validation",
        output_media_type: "application/vnd.flow.fixture-validation+json",
    },
    CapabilitySpec {
        capability_id: "flow/observe-fixture",
        name: "observation",
        output_media_type: "application/vnd.flow.fixture-observation+json",
    },
];

struct TestRoot {
    path: PathBuf,
}

impl TestRoot {
    fn new() -> Self {
        let sequence = NEXT_ROOT_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "flow-hermetic-provider-test-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("hermetic provider test root must be new");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _cleanup_result = fs::remove_dir_all(&self.path);
    }
}

struct KitFixture {
    root: TestRoot,
    catalog: ExtensionCatalog,
    bindings: ArtifactBindingSet,
    package_digest: String,
    executable_digest: String,
    executable_locator: String,
    grants_digest: String,
    package_observations: HostArtifactObservationSet,
}

#[derive(Debug, Eq, PartialEq)]
struct RunEvidence {
    resolution: ExtensionResolution,
    subjects: HostExecutionSubjectObservationSet,
    authority_profile_digest: String,
    enforcement_evidence_digest: String,
    execution: ValidatedExecution,
    observations: HostArtifactObservationSet,
    accepted: AcceptedArtifactSet,
    output_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TreeEntry {
    path: PathBuf,
    kind: TreeEntryKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TreeEntryKind {
    Directory,
    File,
}

#[test]
fn templates_freeze_the_provider_identity_and_four_capabilities() {
    let manifest: ExtensionManifest = serde_json::from_str(MANIFEST_TEMPLATE).unwrap();
    manifest.validate().unwrap();
    let lock: ExtensionLock = serde_json::from_str(LOCK_TEMPLATE).unwrap();
    lock.validate().unwrap();

    assert_eq!(
        manifest.extension_id,
        "org.egohygiene.synthetic-scenario-provider"
    );
    assert_eq!(manifest.version, "0.1.0");
    assert_eq!(manifest.publisher.id, "org.egohygiene");
    assert_eq!(manifest.checkpoint.mode, CheckpointMode::None);
    assert!(manifest.checkpoint.compatibility_keys.is_empty());
    assert_eq!(
        manifest
            .capabilities
            .iter()
            .map(|capability| capability.capability_id.as_str())
            .collect::<Vec<_>>(),
        CAPABILITIES
            .iter()
            .map(|capability| capability.capability_id)
            .collect::<Vec<_>>()
    );
    assert!(
        manifest
            .capabilities
            .iter()
            .all(|capability| capability.domain == Domain::Flow && capability.deterministic)
    );
    for (declared, expected) in manifest.capabilities.iter().zip(CAPABILITIES) {
        assert_eq!(declared.configuration_schema, CONFIGURATION_SCHEMA);
        assert_eq!(declared.accepts, ["text/plain"]);
        assert_eq!(declared.produces, [expected.output_media_type]);
        assert_eq!(declared.content_changes, expected.name == "transformation");
        assert!(declared.cacheable);
    }
    assert_eq!(manifest.execution_modes.len(), 1);
    let process = &manifest.execution_modes[0];
    assert_eq!(process.name, "hermetic-process");
    assert_eq!(process.kind, ExecutionModeKind::Process);
    assert_eq!(process.protocol, flow::EXTENSION_INVOCATION_V1);
    assert_eq!(process.entrypoint, "flow-hermetic-provider");
    assert_eq!(process.limits.timeout_ms, 5_000);
    assert_eq!(process.limits.max_stdout_bytes, 65_536);
    assert_eq!(process.limits.max_stderr_bytes, 65_536);
    assert_eq!(process.limits.cancellation_grace_ms, 250);
    assert!(manifest.requested_permissions.network_hosts.is_empty());
    assert!(manifest.requested_permissions.subprocesses.is_empty());
    assert!(manifest.requested_permissions.ai_providers.is_empty());
    assert!(!manifest.requested_permissions.source_mutation);
    assert!(!manifest.requested_permissions.destructive);
    assert_eq!(lock.extensions.len(), 1);
    assert_eq!(lock.extensions[0].trust, Trust::Trusted);
    assert_eq!(
        lock.extensions[0].granted_permissions,
        manifest.requested_permissions
    );
    assert_eq!(
        lock.capability_resolution
            .iter()
            .map(|resolution| resolution.capability_id.as_str())
            .collect::<Vec<_>>(),
        CAPABILITIES
            .iter()
            .map(|capability| capability.capability_id)
            .collect::<Vec<_>>()
    );
}

#[test]
fn inspection_is_deterministic_across_fresh_process_and_artifact_boundaries() {
    let capability = CAPABILITIES[0];
    let provider_path = Path::new(PROVIDER_BINARY);
    let provider_bytes = fs::read(provider_path).unwrap();
    let executable_name = provider_path
        .file_name()
        .and_then(|name| name.to_str())
        .expect("Cargo provider binary path must have a UTF-8 file name");
    let first_fixture = KitFixture::new(capability, &provider_bytes, executable_name);
    let first = first_fixture.run(capability);
    let second_fixture = KitFixture::new(capability, &provider_bytes, executable_name);
    let second = second_fixture.run(capability);

    assert_eq!(
        first_fixture.package_observations,
        second_fixture.package_observations
    );
    assert_eq!(first_fixture.package_digest, second_fixture.package_digest);
    assert_eq!(
        first_fixture.executable_digest,
        second_fixture.executable_digest
    );
    assert_eq!(first, second, "inspection evidence drifted");
}

impl KitFixture {
    fn new(capability: CapabilitySpec, provider_bytes: &[u8], executable_name: &str) -> Self {
        let root = TestRoot::new();
        let package_path = root.path().join(PACKAGE_LOCATOR);
        let workspace_path = root.path().join(WORKSPACE_LOCATOR);
        fs::create_dir_all(&package_path).unwrap();
        fs::create_dir_all(workspace_path.join("inputs")).unwrap();
        fs::create_dir_all(workspace_path.join("outputs")).unwrap();

        let executable_path = package_path.join(executable_name);
        fs::write(&executable_path, provider_bytes).unwrap();
        make_executable(&executable_path);
        fs::write(package_path.join("LICENSE"), LICENSE_BYTES).unwrap();
        fs::write(workspace_path.join(INPUT_LOCATOR), INPUT_BYTES).unwrap();

        let input_digest = digest_bytes(INPUT_BYTES);
        let output_id = format!("artifact:{}-report", capability.name);
        let output_locator = format!("outputs/{}-report.json", capability.name);
        let bindings = ArtifactBindingSet {
            schema_version: ARTIFACT_BINDINGS_V1.to_owned(),
            binding_set_id: format!("bindings:hermetic-{}", capability.name),
            digest_algorithm: SHA256.to_owned(),
            inputs: vec![InputArtifactBinding {
                artifact_id: INPUT_ID.to_owned(),
                port: "port:source-text".to_owned(),
                media_type: "text/plain".to_owned(),
                kind: ArtifactKind::File,
                locator: INPUT_LOCATOR.to_owned(),
                expected_digest: input_digest,
            }],
            outputs: vec![OutputArtifactBinding {
                artifact_id: output_id,
                port: format!("port:{}-report", capability.name),
                media_type: capability.output_media_type.to_owned(),
                kind: ArtifactKind::File,
                locator: output_locator,
            }],
        };
        bindings.validate().unwrap();
        fs::write(
            workspace_path.join(BINDINGS_LOCATOR),
            serde_json::to_vec_pretty(&bindings).unwrap(),
        )
        .unwrap();

        let package_bindings = ArtifactBindingSet {
            schema_version: ARTIFACT_BINDINGS_V1.to_owned(),
            binding_set_id: "bindings:hermetic-provider-package".to_owned(),
            digest_algorithm: SHA256.to_owned(),
            inputs: Vec::new(),
            outputs: vec![OutputArtifactBinding {
                artifact_id: "artifact:hermetic-provider-package".to_owned(),
                port: "port:hermetic-provider-package".to_owned(),
                media_type: "application/vnd.flow.execution-package-directory".to_owned(),
                kind: ArtifactKind::Directory,
                locator: PACKAGE_LOCATOR.to_owned(),
            }],
        };
        let package_observations = observe_artifacts(root.path(), &package_bindings)
            .unwrap()
            .into_evidence();
        let package_digest = package_observations.artifacts[0].digest.clone();
        let executable_digest = digest_bytes(&fs::read(&executable_path).unwrap());

        let mut manifest: ExtensionManifest = serde_json::from_str(MANIFEST_TEMPLATE).unwrap();
        package_digest.clone_into(&mut manifest.integrity.value);
        let observation = ExtensionObservation::new(
            manifest.extension_id.clone(),
            manifest.version.clone(),
            manifest.publisher.id.clone(),
            manifest.integrity.clone(),
            true,
        );
        let mut lock: ExtensionLock = serde_json::from_str(LOCK_TEMPLATE).unwrap();
        package_digest.clone_into(&mut lock.extensions[0].integrity.value);
        let grants_digest = digest_json(&lock.extensions[0].granted_permissions);
        let catalog = ExtensionCatalog::inspect([manifest], lock, [observation]).unwrap();

        Self {
            root,
            catalog,
            bindings,
            package_digest,
            executable_digest,
            executable_locator: executable_name.to_owned(),
            grants_digest,
            package_observations,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn run(&self, capability: CapabilitySpec) -> RunEvidence {
        let request = ResolutionRequest::new(
            format!("hermetic-{}-success", capability.name),
            capability.capability_id,
            Domain::Flow,
            "hermetic-process",
            ExecutionModeKind::Process,
        );
        let resolution = self.catalog.resolve(&request);
        let resolved = resolution.resolved().unwrap_or_else(|| {
            panic!(
                "hermetic provider must resolve: {:#?}",
                resolution.evidence()
            )
        });
        let configuration_values = BTreeMap::from([
            ("mode".to_owned(), serde_json::json!("success")),
            ("seed".to_owned(), serde_json::json!("hermetic-success-v1")),
        ]);
        let invocation = ExtensionInvocation {
            schema_version: flow::EXTENSION_INVOCATION_V1.to_owned(),
            invocation_id: format!("invocation:hermetic-{}", capability.name),
            run_id: "run:hermetic-success".to_owned(),
            phase: InvocationPhase::Execute,
            extension: InvocationExtension {
                extension_id: resolved.extension_id().to_owned(),
                version: resolved.version().to_owned(),
                publisher_id: resolved.publisher_id().to_owned(),
                integrity: resolved.integrity().value.clone(),
            },
            capability_id: capability.capability_id.to_owned(),
            interface: InvocationInterface {
                kind: resolved.execution_mode().kind,
                name: resolved.execution_mode().name.clone(),
                protocol: resolved.execution_mode().protocol.clone(),
            },
            input_artifacts: vec![InputArtifact {
                artifact_id: INPUT_ID.to_owned(),
                digest: self.bindings.inputs[0].expected_digest.clone(),
            }],
            expected_output_types: vec![capability.output_media_type.to_owned()],
            configuration: Configuration {
                schema_id: CONFIGURATION_SCHEMA.to_owned(),
                digest: digest_json(&configuration_values),
                values: configuration_values,
            },
            authorization: Authorization {
                authorization_id: "authorization:hermetic-success".to_owned(),
                lock_id: resolved.lock_id().to_owned(),
                grants_digest: self.grants_digest.clone(),
            },
            limits: resolved.execution_mode().limits.clone(),
            checkpoint_refs: Vec::new(),
            secret_handles: Vec::new(),
            cancellation_id: "cancel:hermetic-success".to_owned(),
        };
        invocation.validate().unwrap();

        let subject_lock = self.subject_lock(resolved, &invocation, capability);
        let subjects_before =
            observe_execution_subjects(self.root.path(), resolved, &invocation, &subject_lock)
                .unwrap();
        let mut authority_profile = process_authority_profile(
            resolved,
            &invocation,
            &subject_lock,
            &subjects_before,
            ProcessIsolation::TrustedUnconfined,
        );
        assert_eq!(
            authority_profile.schema_version,
            PROCESS_AUTHORITY_PROFILE_V1
        );
        authority_profile.authority_profile_id =
            format!("authority-profile:hermetic-{}", capability.name);
        authority_profile.requested.argv = provider_argv();
        authority_profile
            .granted
            .argv
            .clone_from(&authority_profile.requested.argv);
        let mut enforcement = process_enforcement_evidence(&authority_profile, &subjects_before);
        enforcement.enforcement_evidence_id = format!("enforcement:hermetic-{}", capability.name);
        let authority = authorize_process(
            resolved,
            &invocation,
            &subject_lock,
            &subjects_before,
            &authority_profile,
            &enforcement,
        )
        .unwrap();

        let tree_before = snapshot_tree(self.root.path());
        let input_before =
            fs::read(self.root.path().join(WORKSPACE_LOCATOR).join(INPUT_LOCATOR)).unwrap();
        let bindings_before = fs::read(
            self.root
                .path()
                .join(WORKSPACE_LOCATOR)
                .join(BINDINGS_LOCATOR),
        )
        .unwrap();
        let mut events = Vec::new();
        let execution = LocalProcessRunner::run(
            self.root.path(),
            resolved,
            &invocation,
            &subject_lock,
            &authority,
            &NoSecrets,
            &mut events,
        )
        .unwrap();
        assert_eq!(events, execution.events());
        assert_eq!(execution.result().outcome, flow::Outcome::Produced);
        assert_eq!(
            execution
                .events()
                .iter()
                .map(|event| (event.sequence, event.kind, event.state))
                .collect::<Vec<_>>(),
            [
                (0, EventKind::PhaseStarted, EventState::Running),
                (1, EventKind::ArtifactProduced, EventState::Produced),
                (2, EventKind::PhaseCompleted, EventState::Produced),
            ]
        );
        assert!(execution.events()[0].artifact_refs.is_empty());
        assert_eq!(
            execution.events()[1].artifact_refs,
            [self.bindings.outputs[0].artifact_id.as_str()]
        );
        assert!(execution.events()[2].artifact_refs.is_empty());
        assert_eq!(execution.result().consumed_artifacts, [INPUT_ID]);
        assert_eq!(
            execution.result().produced_artifacts,
            [self.bindings.outputs[0].artifact_id.as_str()]
        );
        assert!(!execution.result().partial_result);
        assert_eq!(execution.result().validations.len(), 1);
        assert_eq!(
            execution.result().validations[0].validator,
            "flow/hermetic-provider-success"
        );
        assert_eq!(
            execution.result().validations[0].status,
            ValidationStatus::Passed
        );
        assert_eq!(
            execution.result().validations[0].evidence,
            "binding-and-input-digest-match"
        );
        assert_eq!(
            execution
                .result()
                .provenance
                .iter()
                .map(|evidence| evidence.kind)
                .collect::<Vec<_>>(),
            [
                ProvenanceKind::Extension,
                ProvenanceKind::Configuration,
                ProvenanceKind::Artifact,
            ]
        );

        let observed =
            observe_artifacts(&self.root.path().join(WORKSPACE_LOCATOR), &self.bindings).unwrap();
        let accepted =
            accept_artifacts(resolved, &invocation, &execution, &self.bindings, &observed).unwrap();
        let output_path = self
            .root
            .path()
            .join(WORKSPACE_LOCATOR)
            .join(&self.bindings.outputs[0].locator);
        let output_bytes = fs::read(&output_path).unwrap();
        let output_digest = digest_bytes(&output_bytes);
        assert_eq!(
            accepted.outputs()[0].digest,
            output_digest,
            "host identity must name the provider's exact output bytes"
        );
        assert_eq!(
            execution.result().provenance[2].value,
            format!("sha256:{output_digest}")
        );
        let output: serde_json::Value = serde_json::from_slice(&output_bytes).unwrap();
        assert_eq!(output["schema_version"], "flow.hermetic-artifact/v1");
        assert_eq!(output["capability_id"], capability.capability_id);
        assert_eq!(output["operation"], capability.name);
        assert_eq!(
            output["configuration_digest"],
            invocation.configuration.digest
        );
        assert_eq!(output["seed"], "hermetic-success-v1");
        assert_eq!(output["inputs"].as_array().unwrap().len(), 1);
        assert_eq!(output["inputs"][0]["artifact_id"], INPUT_ID);
        assert_eq!(
            output["inputs"][0]["digest"],
            self.bindings.inputs[0].expected_digest
        );
        assert_eq!(output.as_object().unwrap().len(), 6);

        let subjects_after =
            observe_execution_subjects(self.root.path(), resolved, &invocation, &subject_lock)
                .unwrap();
        assert_eq!(subjects_before.evidence(), subjects_after.evidence());
        assert_eq!(
            input_before,
            fs::read(self.root.path().join(WORKSPACE_LOCATOR).join(INPUT_LOCATOR)).unwrap()
        );
        assert_eq!(
            bindings_before,
            fs::read(
                self.root
                    .path()
                    .join(WORKSPACE_LOCATOR)
                    .join(BINDINGS_LOCATOR)
            )
            .unwrap()
        );
        assert_eq!(
            digest_bytes(&input_before),
            self.bindings.inputs[0].expected_digest
        );

        let mut expected_tree = tree_before;
        expected_tree.push(TreeEntry {
            path: output_path
                .strip_prefix(self.root.path())
                .unwrap()
                .to_path_buf(),
            kind: TreeEntryKind::File,
        });
        expected_tree.sort_by(|left, right| left.path.cmp(&right.path));
        assert_eq!(snapshot_tree(self.root.path()), expected_tree);

        RunEvidence {
            resolution: resolution.evidence().clone(),
            subjects: subjects_after.evidence().clone(),
            authority_profile_digest: authority.profile_digest().to_owned(),
            enforcement_evidence_digest: authority.evidence_digest().to_owned(),
            execution,
            observations: observed.into_evidence(),
            accepted,
            output_bytes,
        }
    }

    fn subject_lock(
        &self,
        resolved: &flow::ResolvedExtension,
        invocation: &ExtensionInvocation,
        capability: CapabilitySpec,
    ) -> ExecutionSubjectLock {
        let mut lock: ExecutionSubjectLock = serde_json::from_str(include_str!(
            "../contracts/examples/execution-subject-lock.v1.example.json"
        ))
        .unwrap();
        EXECUTION_SUBJECT_LOCK_V1.clone_into(&mut lock.schema_version);
        lock.subject_lock_id = format!("subject-lock:hermetic-{}", capability.name);
        resolved.lock_id().clone_into(&mut lock.extension_lock_id);
        lock.extension.clone_from(&invocation.extension);
        capability.capability_id.clone_into(&mut lock.capability_id);
        lock.interface.clone_from(&invocation.interface);
        lock.declared_entrypoint
            .clone_from(&resolved.execution_mode().entrypoint);
        "package:hermetic-provider".clone_into(&mut lock.package.subject_id);
        PACKAGE_LOCATOR.clone_into(&mut lock.package.locator);
        self.package_digest
            .clone_into(&mut lock.package.digest.value);
        "executable:hermetic-provider".clone_into(&mut lock.executable.subject_id);
        lock.executable.locator.clone_from(&self.executable_locator);
        self.executable_digest
            .clone_into(&mut lock.executable.digest.value);
        lock.validate().unwrap();
        lock
    }
}

fn provider_argv() -> Vec<String> {
    vec![
        "--artifact-root".to_owned(),
        "../../workspace".to_owned(),
        "--artifact-bindings".to_owned(),
        BINDINGS_LOCATOR.to_owned(),
    ]
}

fn snapshot_tree(root: &Path) -> Vec<TreeEntry> {
    fn visit(root: &Path, path: &Path, entries: &mut Vec<TreeEntry>) {
        let mut children = fs::read_dir(path)
            .unwrap()
            .map(|entry| entry.unwrap())
            .collect::<Vec<_>>();
        children.sort_by_key(fs::DirEntry::file_name);
        for child in children {
            let path = child.path();
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            let metadata = fs::symlink_metadata(&path).unwrap();
            if metadata.is_dir() {
                entries.push(TreeEntry {
                    path: relative,
                    kind: TreeEntryKind::Directory,
                });
                visit(root, &path, entries);
            } else {
                assert!(metadata.is_file(), "fixture tree contains a special node");
                entries.push(TreeEntry {
                    path: relative,
                    kind: TreeEntryKind::File,
                });
            }
        }
    }

    let mut entries = Vec::new();
    visit(root, root, &mut entries);
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    entries
}

fn digest_json(value: &impl Serialize) -> String {
    digest_bytes(&serde_json::to_vec(value).unwrap())
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}
