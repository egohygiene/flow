mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use flow::{
    ARTIFACT_BINDINGS_V1, AcceptedArtifactSet, ArtifactAcceptanceError, ArtifactBindingSet,
    ArtifactKind, ArtifactObservationError, Authorization, AuthorizedProcess, CheckpointMode,
    Configuration, Domain, EXECUTION_SUBJECT_LOCK_V1, EventKind, EventSinkError, EventState,
    ExecutionError, ExecutionModeKind, ExecutionSubjectLock, ExtensionCatalog, ExtensionInvocation,
    ExtensionLock, ExtensionManifest, ExtensionObservation, ExtensionResolution, FallbackPolicy,
    HostArtifactObservation, HostArtifactObservationSet, HostExecutionSubjectObservationSet,
    InputArtifact, InputArtifactBinding, InvocationExtension, InvocationInterface, InvocationPhase,
    LocalProcessRunner, MatchedExecutionSubjects, NoSecrets, OutputArtifactBinding,
    PROCESS_AUTHORITY_PROFILE_V1, ProcessIsolation, ProcessRunnerError, ProcessStream,
    ProvenanceKind, ResolutionOutcome, ResolutionRequest, ResolutionResult, SHA256, Severity,
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
const LIFECYCLE_CONTROL_LOCATOR: &str = "outputs/lifecycle-control.json";
const EXTRA_OUTPUT_ID: &str = "artifact:undeclared-extra-output";
const EXTRA_OUTPUT_LOCATOR: &str = "outputs/undeclared-extra-output.json";
const PROVIDER_IDENTITY_LOCATOR: &str = "PROVIDER-IDENTITY";

static NEXT_ROOT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CapabilitySpec {
    capability_id: &'static str,
    name: &'static str,
    accepted_input_media_types: &'static [&'static str],
    output_media_type: &'static str,
}

const CAPABILITIES: [CapabilitySpec; 4] = [
    CapabilitySpec {
        capability_id: "flow/inspect-fixture",
        name: "inspection",
        accepted_input_media_types: &["text/plain"],
        output_media_type: "application/vnd.flow.fixture-inspection+json",
    },
    CapabilitySpec {
        capability_id: "flow/transform-fixture",
        name: "transformation",
        accepted_input_media_types: &["application/vnd.flow.fixture-inspection+json", "text/plain"],
        output_media_type: "application/vnd.flow.fixture-transformation+json",
    },
    CapabilitySpec {
        capability_id: "flow/validate-fixture",
        name: "validation",
        accepted_input_media_types: &["text/plain"],
        output_media_type: "application/vnd.flow.fixture-validation+json",
    },
    CapabilitySpec {
        capability_id: "flow/observe-fixture",
        name: "observation",
        accepted_input_media_types: &["text/plain"],
        output_media_type: "application/vnd.flow.fixture-observation+json",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ProviderSpec {
    extension_id: &'static str,
    package_locator: &'static str,
    package_subject_id: &'static str,
    executable_subject_id: &'static str,
    identity_marker: Option<&'static [u8]>,
}

const PRIMARY_PROVIDER: ProviderSpec = ProviderSpec {
    extension_id: "org.egohygiene.synthetic-scenario-provider",
    package_locator: PACKAGE_LOCATOR,
    package_subject_id: "package:hermetic-provider",
    executable_subject_id: "executable:hermetic-provider",
    identity_marker: None,
};

const INSPECTOR_PROVIDER: ProviderSpec = ProviderSpec {
    extension_id: "org.egohygiene.synthetic-inspector",
    package_locator: "packages/hermetic-provider-inspector",
    package_subject_id: "package:hermetic-provider-inspector",
    executable_subject_id: "executable:hermetic-provider-inspector",
    identity_marker: Some(b"org.egohygiene.synthetic-inspector\n"),
};

const RENDERER_PROVIDER: ProviderSpec = ProviderSpec {
    extension_id: "org.egohygiene.synthetic-renderer",
    package_locator: "packages/hermetic-provider-renderer",
    package_subject_id: "package:hermetic-provider-renderer",
    executable_subject_id: "executable:hermetic-provider-renderer",
    identity_marker: Some(b"org.egohygiene.synthetic-renderer\n"),
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CompositionStageSpec {
    stage_id: &'static str,
    provider: ProviderSpec,
    capability: CapabilitySpec,
    phase: InvocationPhase,
    depends_on: &'static [&'static str],
    consumes: &'static [&'static str],
    produces: &'static [&'static str],
    output_port: &'static str,
    output_locator: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CompositionFixtureSpec {
    fixture_id: &'static str,
    stages: [CompositionStageSpec; 2],
}

const SINGLE_PROVIDER_COMPOSITION: CompositionFixtureSpec = CompositionFixtureSpec {
    fixture_id: "single-provider-composition",
    stages: [
        CompositionStageSpec {
            stage_id: "inspect",
            provider: PRIMARY_PROVIDER,
            capability: CAPABILITIES[0],
            phase: InvocationPhase::Inspect,
            depends_on: &[],
            consumes: &[INPUT_ID],
            produces: &["artifact:inspection-report"],
            output_port: "port:inspection-report",
            output_locator: "outputs/inspection-report.json",
        },
        CompositionStageSpec {
            stage_id: "transform",
            provider: PRIMARY_PROVIDER,
            capability: CAPABILITIES[1],
            phase: InvocationPhase::Execute,
            depends_on: &["inspect"],
            consumes: &["artifact:inspection-report"],
            produces: &["artifact:transformation-report"],
            output_port: "port:transformation-report",
            output_locator: "outputs/transformation-report.json",
        },
    ],
};

const MULTI_PROVIDER_COMPOSITION: CompositionFixtureSpec = CompositionFixtureSpec {
    fixture_id: "multi-provider-composition",
    stages: [
        CompositionStageSpec {
            provider: INSPECTOR_PROVIDER,
            ..SINGLE_PROVIDER_COMPOSITION.stages[0]
        },
        CompositionStageSpec {
            provider: RENDERER_PROVIDER,
            ..SINGLE_PROVIDER_COMPOSITION.stages[1]
        },
    ],
};

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

struct CompositionFixture {
    root: TestRoot,
    spec: CompositionFixtureSpec,
    catalog: ExtensionCatalog,
    providers: BTreeMap<String, MaterializedProvider>,
    source_digest: String,
}

struct MaterializedProvider {
    spec: ProviderSpec,
    package_digest: String,
    executable_digest: String,
    executable_locator: String,
    grants_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompositionInput {
    artifact_id: String,
    port: String,
    media_type: String,
    kind: ArtifactKind,
    locator: String,
    digest: String,
}

impl From<&HostArtifactObservation> for CompositionInput {
    fn from(artifact: &HostArtifactObservation) -> Self {
        Self {
            artifact_id: artifact.artifact_id.clone(),
            port: artifact.port.clone(),
            media_type: artifact.media_type.clone(),
            kind: artifact.kind,
            locator: artifact.locator.clone(),
            digest: artifact.digest.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct CatalogProfile {
    available: bool,
    flow_version_requirement: Option<&'static str>,
}

impl Default for CatalogProfile {
    fn default() -> Self {
        Self {
            available: true,
            flow_version_requirement: None,
        }
    }
}

struct PreparedLifecycleRun {
    resolution: ResolutionOutcome,
    invocation: ExtensionInvocation,
    subject_lock: ExecutionSubjectLock,
    subjects: MatchedExecutionSubjects,
    authority: AuthorizedProcess,
}

impl PreparedLifecycleRun {
    fn resolved(&self) -> &flow::ResolvedExtension {
        self.resolution
            .resolved()
            .expect("prepared lifecycle runs must retain their resolution token")
    }
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

#[derive(Debug, Eq, PartialEq)]
struct CompositionRunEvidence {
    fixture_id: String,
    stages: Vec<CompositionStageEvidence>,
}

#[derive(Debug, Eq, PartialEq)]
struct CompositionStageEvidence {
    stage_id: String,
    depends_on: Vec<String>,
    bindings: ArtifactBindingSet,
    resolution: ExtensionResolution,
    invocation: ExtensionInvocation,
    subject_lock: ExecutionSubjectLock,
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
        assert_eq!(
            declared
                .accepts
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            expected.accepted_input_media_types
        );
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
    assert_eq!(
        manifest.requested_permissions.filesystem_read,
        [
            "workspace/artifact-bindings",
            "workspace/inputs",
            "workspace/outputs",
        ]
    );
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

#[test]
fn single_provider_composition_preserves_exact_handoff_and_determinism() {
    assert_composition_fixture(
        SINGLE_PROVIDER_COMPOSITION,
        [PRIMARY_PROVIDER.extension_id, PRIMARY_PROVIDER.extension_id],
        false,
    );
}

#[test]
fn multi_provider_composition_preserves_exact_handoff_and_determinism() {
    assert_composition_fixture(
        MULTI_PROVIDER_COMPOSITION,
        [
            INSPECTOR_PROVIDER.extension_id,
            RENDERER_PROVIDER.extension_id,
        ],
        true,
    );
}

#[test]
fn unavailable_provider_is_blocked_before_invocation_with_retained_evidence() {
    let capability = CAPABILITIES[0];
    let (provider_bytes, executable_name) = provider_binary_snapshot();
    let fixture = KitFixture::new_with_catalog_profile(
        capability,
        &provider_bytes,
        &executable_name,
        CatalogProfile {
            available: false,
            flow_version_requirement: None,
        },
    );
    let tree_before = snapshot_tree(fixture.root.path());

    let outcome = fixture.resolve_case(
        capability,
        "hermetic-inspection-unavailable",
        "hermetic-process",
    );

    outcome.evidence().validate().unwrap();
    assert!(outcome.resolved().is_none());
    assert_eq!(outcome.evidence().result, ResolutionResult::Blocked);
    assert!(outcome.evidence().selected_extension_ids.is_empty());
    assert!(outcome.evidence().fallback_order.is_empty());
    assert_eq!(outcome.evidence().candidates.len(), 1);
    let candidate = &outcome.evidence().candidates[0];
    assert!(candidate.compatible);
    assert!(candidate.authorized);
    assert!(!candidate.available);
    assert!(candidate.reasons.iter().any(|reason| {
        reason == "The caller reports the extension unavailable before invocation."
    }));
    assert_eq!(snapshot_tree(fixture.root.path()), tree_before);
    assert!(!fixture.output_path().exists());
}

#[test]
fn incompatible_provider_is_rejected_before_invocation_with_retained_evidence() {
    let capability = CAPABILITIES[0];
    let (provider_bytes, executable_name) = provider_binary_snapshot();
    let fixture = KitFixture::new_with_catalog_profile(
        capability,
        &provider_bytes,
        &executable_name,
        CatalogProfile {
            available: true,
            flow_version_requirement: Some(">=9.0.0"),
        },
    );
    let tree_before = snapshot_tree(fixture.root.path());

    let outcome = fixture.resolve_case(
        capability,
        "hermetic-inspection-incompatible",
        "hermetic-process",
    );

    outcome.evidence().validate().unwrap();
    assert!(outcome.resolved().is_none());
    assert_eq!(
        outcome.evidence().result,
        ResolutionResult::NoCompatibleProvider
    );
    assert!(outcome.evidence().selected_extension_ids.is_empty());
    assert!(outcome.evidence().fallback_order.is_empty());
    assert_eq!(outcome.evidence().candidates.len(), 1);
    let candidate = &outcome.evidence().candidates[0];
    assert!(!candidate.compatible);
    assert!(candidate.authorized);
    assert!(candidate.available);
    assert!(
        candidate
            .reasons
            .iter()
            .any(|reason| reason == "Flow 0.1.0 does not satisfy >=9.0.0.")
    );
    assert_eq!(snapshot_tree(fixture.root.path()), tree_before);
    assert!(!fixture.output_path().exists());
}

#[test]
fn warning_and_partial_results_remain_semantically_distinct() {
    let capability = CAPABILITIES[0];
    let (provider_bytes, executable_name) = provider_binary_snapshot();

    let warning_fixture = KitFixture::new(capability, &provider_bytes, &executable_name);
    let warning = warning_fixture.prepare_lifecycle(capability, "warning", false);
    let warning_before = warning_fixture.immutable_workspace_bytes();
    let mut warning_events = Vec::new();
    let warning_execution = LocalProcessRunner::run(
        warning_fixture.root.path(),
        warning.resolved(),
        &warning.invocation,
        &warning.subject_lock,
        &warning.authority,
        &NoSecrets,
        &mut warning_events,
    )
    .unwrap();
    assert_eq!(warning_events, warning_execution.events());
    assert_eq!(warning_execution.result().outcome, flow::Outcome::Produced);
    assert!(!warning_execution.result().partial_result);
    assert_eq!(warning_execution.result().diagnostics.len(), 1);
    let diagnostic = &warning_execution.result().diagnostics[0];
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert!(diagnostic.redacted);
    let warning_observed = observe_artifacts(
        &warning_fixture.root.path().join(WORKSPACE_LOCATOR),
        &warning_fixture.bindings,
    )
    .unwrap();
    accept_artifacts(
        warning.resolved(),
        &warning.invocation,
        &warning_execution,
        &warning_fixture.bindings,
        &warning_observed,
    )
    .unwrap();
    warning_fixture.assert_subjects_unchanged(&warning);
    warning_fixture.assert_immutable_workspace_bytes(&warning_before);

    let partial_fixture = KitFixture::new(capability, &provider_bytes, &executable_name);
    let partial = partial_fixture.prepare_lifecycle(capability, "partial-result", false);
    let partial_before = partial_fixture.immutable_workspace_bytes();
    let mut partial_events = Vec::new();
    let partial_execution = LocalProcessRunner::run(
        partial_fixture.root.path(),
        partial.resolved(),
        &partial.invocation,
        &partial.subject_lock,
        &partial.authority,
        &NoSecrets,
        &mut partial_events,
    )
    .unwrap();
    assert_eq!(partial_events, partial_execution.events());
    assert_eq!(partial_execution.result().outcome, flow::Outcome::Produced);
    assert!(partial_execution.result().partial_result);
    assert!(partial_execution.result().diagnostics.is_empty());
    let partial_observed = observe_artifacts(
        &partial_fixture.root.path().join(WORKSPACE_LOCATOR),
        &partial_fixture.bindings,
    )
    .unwrap();
    let error = accept_artifacts(
        partial.resolved(),
        &partial.invocation,
        &partial_execution,
        &partial_fixture.bindings,
        &partial_observed,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ArtifactAcceptanceError::Mismatch { ref message }
            if message == "artifact acceptance requires a complete produced or reused result"
    ));
    partial_fixture.assert_subjects_unchanged(&partial);
    partial_fixture.assert_immutable_workspace_bytes(&partial_before);
}

#[test]
fn missing_bound_output_fails_observation_after_valid_provider_success() {
    let capability = CAPABILITIES[0];
    let (provider_bytes, executable_name) = provider_binary_snapshot();
    let fixture = KitFixture::new(capability, &provider_bytes, &executable_name);
    let prepared = fixture.prepare_lifecycle(capability, "missing-output", false);
    let immutable_before = fixture.immutable_workspace_bytes();
    let mut events = Vec::new();

    let execution = LocalProcessRunner::run(
        fixture.root.path(),
        prepared.resolved(),
        &prepared.invocation,
        &prepared.subject_lock,
        &prepared.authority,
        &NoSecrets,
        &mut events,
    )
    .unwrap();

    assert_eq!(events, execution.events());
    assert_eq!(execution.result().outcome, flow::Outcome::Produced);
    assert!(!execution.result().partial_result);
    assert_eq!(
        execution.result().produced_artifacts,
        [fixture.bindings.outputs[0].artifact_id.as_str()]
    );
    assert!(!fixture.output_path().exists());

    let error = observe_artifacts(
        &fixture.root.path().join(WORKSPACE_LOCATOR),
        &fixture.bindings,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ArtifactObservationError::Missing {
            ref artifact_id,
            ref locator,
        } if artifact_id == &fixture.bindings.outputs[0].artifact_id
            && locator == &fixture.bindings.outputs[0].locator
    ));
    fixture.assert_subjects_unchanged(&prepared);
    fixture.assert_immutable_workspace_bytes(&immutable_before);
}

#[test]
fn extra_provider_output_fails_exact_acceptance_without_auto_discovery() {
    let capability = CAPABILITIES[0];
    let (provider_bytes, executable_name) = provider_binary_snapshot();
    let fixture = KitFixture::new(capability, &provider_bytes, &executable_name);
    let prepared = fixture.prepare_lifecycle(capability, "extra-output", false);
    let immutable_before = fixture.immutable_workspace_bytes();
    let mut events = Vec::new();

    let execution = LocalProcessRunner::run(
        fixture.root.path(),
        prepared.resolved(),
        &prepared.invocation,
        &prepared.subject_lock,
        &prepared.authority,
        &NoSecrets,
        &mut events,
    )
    .unwrap();

    assert_eq!(events, execution.events());
    assert_eq!(
        execution.result().produced_artifacts,
        [
            fixture.bindings.outputs[0].artifact_id.as_str(),
            EXTRA_OUTPUT_ID,
        ]
    );
    assert_eq!(
        execution.events()[1].artifact_refs,
        [
            fixture.bindings.outputs[0].artifact_id.as_str(),
            EXTRA_OUTPUT_ID,
        ]
    );
    assert!(
        fixture
            .root
            .path()
            .join(WORKSPACE_LOCATOR)
            .join(EXTRA_OUTPUT_LOCATOR)
            .is_file()
    );

    let observed = observe_artifacts(
        &fixture.root.path().join(WORKSPACE_LOCATOR),
        &fixture.bindings,
    )
    .unwrap();
    assert_eq!(
        observed
            .evidence()
            .artifacts
            .iter()
            .map(|artifact| artifact.artifact_id.as_str())
            .collect::<Vec<_>>(),
        [
            fixture.bindings.inputs[0].artifact_id.as_str(),
            fixture.bindings.outputs[0].artifact_id.as_str(),
        ]
    );
    let error = accept_artifacts(
        prepared.resolved(),
        &prepared.invocation,
        &execution,
        &fixture.bindings,
        &observed,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ArtifactAcceptanceError::Mismatch { ref message }
            if message == "provider-produced artifacts do not exactly match declared outputs"
    ));
    fixture.assert_subjects_unchanged(&prepared);
    fixture.assert_immutable_workspace_bytes(&immutable_before);
}

#[test]
fn partial_output_is_observable_but_cannot_be_accepted() {
    let capability = CAPABILITIES[0];
    let (provider_bytes, executable_name) = provider_binary_snapshot();
    let fixture = KitFixture::new(capability, &provider_bytes, &executable_name);
    let prepared = fixture.prepare_lifecycle(capability, "partial-output", false);
    let immutable_before = fixture.immutable_workspace_bytes();
    let mut events = Vec::new();

    let execution = LocalProcessRunner::run(
        fixture.root.path(),
        prepared.resolved(),
        &prepared.invocation,
        &prepared.subject_lock,
        &prepared.authority,
        &NoSecrets,
        &mut events,
    )
    .unwrap();

    assert_eq!(events, execution.events());
    assert!(execution.result().partial_result);
    let output_bytes = fs::read(fixture.output_path()).unwrap();
    assert!(!output_bytes.ends_with(b"\n"));
    assert!(serde_json::from_slice::<serde_json::Value>(&output_bytes).is_err());
    let observed = observe_artifacts(
        &fixture.root.path().join(WORKSPACE_LOCATOR),
        &fixture.bindings,
    )
    .unwrap();
    assert_eq!(
        execution.result().provenance[2].value,
        format!("sha256:{}", observed.evidence().artifacts[1].digest)
    );
    let error = accept_artifacts(
        prepared.resolved(),
        &prepared.invocation,
        &execution,
        &fixture.bindings,
        &observed,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ArtifactAcceptanceError::Mismatch { ref message }
            if message == "artifact acceptance requires a complete produced or reused result"
    ));
    fixture.assert_subjects_unchanged(&prepared);
    fixture.assert_immutable_workspace_bytes(&immutable_before);
}

#[test]
fn corrupt_artifact_evidence_never_promotes_execution() {
    let capability = CAPABILITIES[0];
    let (provider_bytes, executable_name) = provider_binary_snapshot();
    let fixture = KitFixture::new(capability, &provider_bytes, &executable_name);
    let prepared = fixture.prepare_lifecycle(capability, "corrupt-artifact-evidence", false);
    let immutable_before = fixture.immutable_workspace_bytes();
    let mut events = Vec::new();

    let error = LocalProcessRunner::run(
        fixture.root.path(),
        prepared.resolved(),
        &prepared.invocation,
        &prepared.subject_lock,
        &prepared.authority,
        &NoSecrets,
        &mut events,
    )
    .unwrap_err();

    match error {
        ProcessRunnerError::Validation {
            source:
                ExecutionError::InvalidResult {
                    message,
                    events: retained_events,
                    result,
                },
        } => {
            assert_eq!(message, "result.provenance[2].value: must not be empty");
            assert_eq!(retained_events.len(), 3);
            assert!(result.provenance[2].value.is_empty());
        }
        other => panic!("unexpected corrupt-artifact-evidence error: {other:?}"),
    }
    assert_eq!(events.len(), 3, "valid events remain authoritative");
    assert!(fixture.output_path().is_file());
    fixture.assert_subjects_unchanged(&prepared);
    fixture.assert_immutable_workspace_bytes(&immutable_before);
}

#[test]
fn contradictory_artifact_evidence_fails_exact_acceptance() {
    let capability = CAPABILITIES[0];
    let (provider_bytes, executable_name) = provider_binary_snapshot();
    let fixture = KitFixture::new(capability, &provider_bytes, &executable_name);
    let prepared = fixture.prepare_lifecycle(capability, "contradictory-artifact-evidence", false);
    let immutable_before = fixture.immutable_workspace_bytes();
    let mut events = Vec::new();

    let execution = LocalProcessRunner::run(
        fixture.root.path(),
        prepared.resolved(),
        &prepared.invocation,
        &prepared.subject_lock,
        &prepared.authority,
        &NoSecrets,
        &mut events,
    )
    .unwrap();

    assert_eq!(events, execution.events());
    assert!(execution.events()[1].artifact_refs.is_empty());
    assert_eq!(
        execution.result().produced_artifacts,
        [fixture.bindings.outputs[0].artifact_id.as_str()]
    );
    let observed = observe_artifacts(
        &fixture.root.path().join(WORKSPACE_LOCATOR),
        &fixture.bindings,
    )
    .unwrap();
    let error = accept_artifacts(
        prepared.resolved(),
        &prepared.invocation,
        &execution,
        &fixture.bindings,
        &observed,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ArtifactAcceptanceError::Mismatch { ref message }
            if message == "artifact-produced events do not exactly match declared outputs"
    ));
    fixture.assert_subjects_unchanged(&prepared);
    fixture.assert_immutable_workspace_bytes(&immutable_before);
}

#[test]
fn changed_output_after_host_observation_fails_freshness_gate() {
    let capability = CAPABILITIES[0];
    let (provider_bytes, executable_name) = provider_binary_snapshot();
    let fixture = KitFixture::new(capability, &provider_bytes, &executable_name);
    let prepared = fixture.prepare_lifecycle(capability, "success", false);
    let immutable_before = fixture.immutable_workspace_bytes();
    let mut events = Vec::new();
    let execution = LocalProcessRunner::run(
        fixture.root.path(),
        prepared.resolved(),
        &prepared.invocation,
        &prepared.subject_lock,
        &prepared.authority,
        &NoSecrets,
        &mut events,
    )
    .unwrap();
    assert_eq!(events, execution.events());
    let observed = observe_artifacts(
        &fixture.root.path().join(WORKSPACE_LOCATOR),
        &fixture.bindings,
    )
    .unwrap();

    fs::write(fixture.output_path(), b"{\"changed\":true}\n").unwrap();

    let error = accept_artifacts(
        prepared.resolved(),
        &prepared.invocation,
        &execution,
        &fixture.bindings,
        &observed,
    )
    .unwrap_err();
    assert!(matches!(error, ArtifactAcceptanceError::ObservationChanged));
    fixture.assert_subjects_unchanged(&prepared);
    fixture.assert_immutable_workspace_bytes(&immutable_before);
}

#[test]
fn stale_invocation_and_binding_contexts_cannot_reuse_valid_evidence() {
    let capability = CAPABILITIES[0];
    let (provider_bytes, executable_name) = provider_binary_snapshot();
    let fixture = KitFixture::new(capability, &provider_bytes, &executable_name);
    let prepared = fixture.prepare_lifecycle(capability, "success", false);
    let immutable_before = fixture.immutable_workspace_bytes();
    let mut events = Vec::new();
    let execution = LocalProcessRunner::run(
        fixture.root.path(),
        prepared.resolved(),
        &prepared.invocation,
        &prepared.subject_lock,
        &prepared.authority,
        &NoSecrets,
        &mut events,
    )
    .unwrap();
    let observed = observe_artifacts(
        &fixture.root.path().join(WORKSPACE_LOCATOR),
        &fixture.bindings,
    )
    .unwrap();

    let mut stale_invocation = prepared.invocation.clone();
    stale_invocation.run_id = "run:stale-hermetic-evidence".to_owned();
    let error = accept_artifacts(
        prepared.resolved(),
        &stale_invocation,
        &execution,
        &fixture.bindings,
        &observed,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ArtifactAcceptanceError::Mismatch { ref message }
            if message == "validated execution context does not match the supplied invocation"
    ));

    let mut stale_bindings = fixture.bindings.clone();
    stale_bindings.binding_set_id = "bindings:stale-hermetic-evidence".to_owned();
    let error = accept_artifacts(
        prepared.resolved(),
        &prepared.invocation,
        &execution,
        &stale_bindings,
        &observed,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ArtifactAcceptanceError::Mismatch { ref message }
            if message == "artifact bindings changed after host observation"
    ));

    fixture.assert_subjects_unchanged(&prepared);
    fixture.assert_immutable_workspace_bytes(&immutable_before);
}

#[test]
fn nonzero_after_success_never_promotes_provider_evidence() {
    let capability = CAPABILITIES[0];
    let (provider_bytes, executable_name) = provider_binary_snapshot();
    let fixture = KitFixture::new(capability, &provider_bytes, &executable_name);
    let prepared = fixture.prepare_lifecycle(capability, "nonzero-after-success", false);
    let immutable_before = fixture.immutable_workspace_bytes();
    let mut events = Vec::new();

    let error = LocalProcessRunner::run(
        fixture.root.path(),
        prepared.resolved(),
        &prepared.invocation,
        &prepared.subject_lock,
        &prepared.authority,
        &NoSecrets,
        &mut events,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ProcessRunnerError::Validation {
            source: ExecutionError::ProcessExit { code: Some(7) }
        }
    ));
    assert!(
        events.is_empty(),
        "exit failure must precede transcript parsing"
    );
    assert!(fixture.output_path().is_file());
    assert_eq!(
        prepared.resolved().fallback_policy(),
        FallbackPolicy::Forbidden
    );
    fixture.assert_subjects_unchanged(&prepared);
    fixture.assert_immutable_workspace_bytes(&immutable_before);
}

#[test]
fn invalid_event_and_result_return_typed_semantic_errors() {
    let capability = CAPABILITIES[0];
    let (provider_bytes, executable_name) = provider_binary_snapshot();

    let event_fixture = KitFixture::new(capability, &provider_bytes, &executable_name);
    let invalid_event = event_fixture.prepare_lifecycle(capability, "invalid-event", false);
    let event_before = event_fixture.immutable_workspace_bytes();
    let mut forwarded_events = Vec::new();
    let event_error = LocalProcessRunner::run(
        event_fixture.root.path(),
        invalid_event.resolved(),
        &invalid_event.invocation,
        &invalid_event.subject_lock,
        &invalid_event.authority,
        &NoSecrets,
        &mut forwarded_events,
    )
    .unwrap_err();
    match event_error {
        ProcessRunnerError::Validation {
            source:
                ExecutionError::InvalidEvent {
                    message,
                    events,
                    result: Some(result),
                },
        } => {
            assert!(message.contains("strictly increasing"));
            assert_eq!(events.len(), 3, "raw provider events must be retained");
            assert_eq!(result.outcome, flow::Outcome::Produced);
        }
        other => panic!("unexpected invalid-event error: {other:?}"),
    }
    assert_eq!(
        forwarded_events.len(),
        1,
        "only the valid prefix may reach the authoritative sink"
    );
    event_fixture.assert_subjects_unchanged(&invalid_event);
    event_fixture.assert_immutable_workspace_bytes(&event_before);

    let result_fixture = KitFixture::new(capability, &provider_bytes, &executable_name);
    let invalid_result = result_fixture.prepare_lifecycle(capability, "invalid-result", false);
    let result_before = result_fixture.immutable_workspace_bytes();
    let mut result_events = Vec::new();
    let result_error = LocalProcessRunner::run(
        result_fixture.root.path(),
        invalid_result.resolved(),
        &invalid_result.invocation,
        &invalid_result.subject_lock,
        &invalid_result.authority,
        &NoSecrets,
        &mut result_events,
    )
    .unwrap_err();
    match result_error {
        ProcessRunnerError::Validation {
            source:
                ExecutionError::InvalidResult {
                    message,
                    events,
                    result,
                },
        } => {
            assert!(message.contains("authorization_id"));
            assert_eq!(events.len(), 3);
            assert_eq!(result.outcome, flow::Outcome::Produced);
        }
        other => panic!("unexpected invalid-result error: {other:?}"),
    }
    assert_eq!(result_events.len(), 3, "valid events remain authoritative");
    result_fixture.assert_subjects_unchanged(&invalid_result);
    result_fixture.assert_immutable_workspace_bytes(&result_before);
}

#[test]
fn authoritative_host_rejection_blocks_valid_provider_success() {
    let capability = CAPABILITIES[0];
    let (provider_bytes, executable_name) = provider_binary_snapshot();
    let fixture = KitFixture::new(capability, &provider_bytes, &executable_name);
    let prepared = fixture.prepare_lifecycle(capability, "success-with-host-rejection", false);
    let immutable_before = fixture.immutable_workspace_bytes();
    let mut forwarded_events = Vec::new();
    let mut rejecting_sink = |event: &flow::ExtensionEvent| {
        forwarded_events.push(event.clone());
        Err(EventSinkError::new(
            "checkpoint-2 synthetic host observer rejection",
        ))
    };

    let error = LocalProcessRunner::run(
        fixture.root.path(),
        prepared.resolved(),
        &prepared.invocation,
        &prepared.subject_lock,
        &prepared.authority,
        &NoSecrets,
        &mut rejecting_sink,
    )
    .unwrap_err();

    match error {
        ProcessRunnerError::Validation {
            source:
                ExecutionError::EventSink {
                    source,
                    events,
                    result: Some(result),
                },
        } => {
            assert_eq!(
                source.message,
                "checkpoint-2 synthetic host observer rejection"
            );
            assert_eq!(events.len(), 3);
            assert_eq!(result.outcome, flow::Outcome::Produced);
        }
        other => panic!("unexpected host-rejection error: {other:?}"),
    }
    assert_eq!(forwarded_events.len(), 1);
    assert!(fixture.output_path().is_file());
    assert_eq!(
        prepared.resolved().fallback_policy(),
        FallbackPolicy::Forbidden
    );
    fixture.assert_subjects_unchanged(&prepared);
    fixture.assert_immutable_workspace_bytes(&immutable_before);
}

#[test]
fn stdout_and_stderr_overflow_retain_only_the_limit_sentinel() {
    let capability = CAPABILITIES[0];
    let (provider_bytes, executable_name) = provider_binary_snapshot();

    for (mode, expected_stream) in [
        ("stdout-overflow", ProcessStream::Stdout),
        ("stderr-overflow", ProcessStream::Stderr),
    ] {
        let fixture = KitFixture::new(capability, &provider_bytes, &executable_name);
        let prepared = fixture.prepare_lifecycle(capability, mode, false);
        let immutable_before = fixture.immutable_workspace_bytes();
        let limit = match expected_stream {
            ProcessStream::Stdout => prepared.invocation.limits.max_stdout_bytes,
            ProcessStream::Stderr => prepared.invocation.limits.max_stderr_bytes,
        };
        let mut events = Vec::new();

        let error = LocalProcessRunner::run(
            fixture.root.path(),
            prepared.resolved(),
            &prepared.invocation,
            &prepared.subject_lock,
            &prepared.authority,
            &NoSecrets,
            &mut events,
        )
        .unwrap_err();

        assert!(matches!(
            error,
            ProcessRunnerError::Validation {
                source: ExecutionError::ProcessOutputLimit {
                    stream,
                    limit: observed_limit,
                    observed,
                }
            } if stream == expected_stream
                && observed_limit == limit
                && observed == limit + 1
        ));
        assert!(events.is_empty());
        assert!(!fixture.output_path().exists());
        fixture.assert_subjects_unchanged(&prepared);
        fixture.assert_immutable_workspace_bytes(&immutable_before);
    }
}

#[cfg(unix)]
#[test]
fn timeout_and_readiness_gated_cancellation_reap_the_direct_child() {
    let capability = CAPABILITIES[0];
    let (provider_bytes, executable_name) = provider_binary_snapshot();

    let cancellation_fixture = KitFixture::new(capability, &provider_bytes, &executable_name);
    let cancellation =
        cancellation_fixture.prepare_lifecycle(capability, "await-interruption", true);
    let cancellation_before = cancellation_fixture.immutable_workspace_bytes();
    let cancellation_control = cancellation_fixture.lifecycle_control_path();
    let cancel_when_ready = || cancellation_control.is_file();
    let mut cancellation_events = Vec::new();
    let cancellation_error = LocalProcessRunner::run_with_cancellation(
        cancellation_fixture.root.path(),
        cancellation.resolved(),
        &cancellation.invocation,
        &cancellation.subject_lock,
        &cancellation.authority,
        &NoSecrets,
        &cancel_when_ready,
        &mut cancellation_events,
    )
    .unwrap_err();
    assert!(matches!(
        cancellation_error,
        ProcessRunnerError::Cancelled { forced: false }
    ));
    assert!(cancellation_events.is_empty());
    assert!(cancellation_control.is_file());
    assert_recorded_process_reaped(&cancellation_control);
    assert!(!cancellation_fixture.output_path().exists());
    cancellation_fixture.assert_subjects_unchanged(&cancellation);
    cancellation_fixture.assert_immutable_workspace_bytes(&cancellation_before);

    let timeout_fixture = KitFixture::new(capability, &provider_bytes, &executable_name);
    let timeout = timeout_fixture.prepare_lifecycle(capability, "await-interruption", true);
    let timeout_before = timeout_fixture.immutable_workspace_bytes();
    let timeout_control = timeout_fixture.lifecycle_control_path();
    let mut timeout_events = Vec::new();
    let timeout_error = LocalProcessRunner::run(
        timeout_fixture.root.path(),
        timeout.resolved(),
        &timeout.invocation,
        &timeout.subject_lock,
        &timeout.authority,
        &NoSecrets,
        &mut timeout_events,
    )
    .unwrap_err();
    assert!(matches!(
        timeout_error,
        ProcessRunnerError::TimedOut {
            timeout_ms: 5_000,
            forced: false,
        }
    ));
    assert!(timeout_events.is_empty());
    assert!(timeout_control.is_file());
    assert_recorded_process_reaped(&timeout_control);
    assert!(!timeout_fixture.output_path().exists());
    timeout_fixture.assert_subjects_unchanged(&timeout);
    timeout_fixture.assert_immutable_workspace_bytes(&timeout_before);
}

impl CompositionFixture {
    #[allow(clippy::too_many_lines)]
    fn new(spec: CompositionFixtureSpec, provider_bytes: &[u8], executable_name: &str) -> Self {
        let root = TestRoot::new();
        let workspace_path = root.path().join(WORKSPACE_LOCATOR);
        fs::create_dir_all(workspace_path.join("inputs")).unwrap();
        fs::create_dir_all(workspace_path.join("outputs")).unwrap();
        fs::create_dir_all(workspace_path.join("artifact-bindings")).unwrap();
        fs::write(workspace_path.join(INPUT_LOCATOR), INPUT_BYTES).unwrap();
        let source_digest = digest_bytes(INPUT_BYTES);

        let manifest_template: ExtensionManifest = serde_json::from_str(MANIFEST_TEMPLATE).unwrap();
        let mut lock: ExtensionLock = serde_json::from_str(LOCK_TEMPLATE).unwrap();
        let locked_extension_template = lock.extensions[0].clone();
        lock.lock_id = format!("lock:{}", spec.fixture_id);
        lock.extensions.clear();

        let mut provider_specs = BTreeMap::new();
        for stage in spec.stages {
            provider_specs.insert(stage.provider.extension_id, stage.provider);
        }

        let mut manifests = Vec::new();
        let mut observations = Vec::new();
        let mut providers = BTreeMap::new();
        for provider_spec in provider_specs.values().copied() {
            let package_path = root.path().join(provider_spec.package_locator);
            fs::create_dir_all(&package_path).unwrap();
            let executable_path = package_path.join(executable_name);
            fs::write(&executable_path, provider_bytes).unwrap();
            make_executable(&executable_path);
            fs::write(package_path.join("LICENSE"), LICENSE_BYTES).unwrap();
            if let Some(identity_marker) = provider_spec.identity_marker {
                fs::write(
                    package_path.join(PROVIDER_IDENTITY_LOCATOR),
                    identity_marker,
                )
                .unwrap();
            }

            let package_bindings = ArtifactBindingSet {
                schema_version: ARTIFACT_BINDINGS_V1.to_owned(),
                binding_set_id: format!(
                    "bindings:composition-package-{}",
                    provider_spec.extension_id
                ),
                digest_algorithm: SHA256.to_owned(),
                inputs: Vec::new(),
                outputs: vec![OutputArtifactBinding {
                    artifact_id: format!(
                        "artifact:composition-package-{}",
                        provider_spec.extension_id
                    ),
                    port: format!("port:composition-package-{}", provider_spec.extension_id),
                    media_type: "application/vnd.flow.execution-package-directory".to_owned(),
                    kind: ArtifactKind::Directory,
                    locator: provider_spec.package_locator.to_owned(),
                }],
            };
            package_bindings.validate().unwrap();
            let package_observations = observe_artifacts(root.path(), &package_bindings)
                .unwrap()
                .into_evidence();
            let package_digest = package_observations.artifacts[0].digest.clone();
            let executable_digest = digest_bytes(&fs::read(&executable_path).unwrap());

            let mut manifest = manifest_template.clone();
            provider_spec
                .extension_id
                .clone_into(&mut manifest.extension_id);
            package_digest.clone_into(&mut manifest.integrity.value);
            manifest.validate().unwrap();
            observations.push(ExtensionObservation::new(
                manifest.extension_id.clone(),
                manifest.version.clone(),
                manifest.publisher.id.clone(),
                manifest.integrity.clone(),
                true,
            ));

            let mut locked_extension = locked_extension_template.clone();
            provider_spec
                .extension_id
                .clone_into(&mut locked_extension.extension_id);
            provider_spec
                .package_locator
                .clone_into(&mut locked_extension.discovery.location);
            package_digest.clone_into(&mut locked_extension.integrity.value);
            let grants_digest = digest_json(&locked_extension.granted_permissions);
            lock.extensions.push(locked_extension);
            manifests.push(manifest);
            providers.insert(
                provider_spec.extension_id.to_owned(),
                MaterializedProvider {
                    spec: provider_spec,
                    package_digest,
                    executable_digest,
                    executable_locator: executable_name.to_owned(),
                    grants_digest,
                },
            );
        }

        let default_provider = spec.stages[0].provider.extension_id;
        for resolution in &mut lock.capability_resolution {
            let selected_provider = spec
                .stages
                .iter()
                .find(|stage| stage.capability.capability_id == resolution.capability_id)
                .map_or(default_provider, |stage| stage.provider.extension_id);
            resolution.ordered_extensions = vec![selected_provider.to_owned()];
        }
        lock.validate().unwrap();
        let catalog = ExtensionCatalog::inspect(manifests, lock, observations).unwrap();

        Self {
            root,
            spec,
            catalog,
            providers,
            source_digest,
        }
    }

    fn run(&self) -> CompositionRunEvidence {
        let inspect_spec = self.spec.stages[0];
        assert_eq!(inspect_spec.stage_id, "inspect");
        assert!(inspect_spec.depends_on.is_empty());
        assert_eq!(inspect_spec.consumes, [INPUT_ID]);
        let inspect_input = CompositionInput {
            artifact_id: INPUT_ID.to_owned(),
            port: "port:source-text".to_owned(),
            media_type: "text/plain".to_owned(),
            kind: ArtifactKind::File,
            locator: INPUT_LOCATOR.to_owned(),
            digest: self.source_digest.clone(),
        };
        let inspect = self.run_stage(inspect_spec, &inspect_input);

        let transform_spec = self.spec.stages[1];
        assert_eq!(transform_spec.stage_id, "transform");
        assert_eq!(transform_spec.depends_on, [inspect_spec.stage_id]);
        let accepted_handoff = inspect.accepted.outputs()[0].clone();
        assert_eq!(
            transform_spec.consumes,
            [accepted_handoff.artifact_id.as_str()]
        );
        let transform_input = CompositionInput::from(&accepted_handoff);
        let transform = self.run_stage(transform_spec, &transform_input);
        assert_eq!(
            accepted_handoff,
            transform.accepted.inputs()[0],
            "the downstream input must be the exact accepted upstream artifact"
        );
        assert_eq!(
            fs::read(self.root.path().join(WORKSPACE_LOCATOR).join(INPUT_LOCATOR)).unwrap(),
            INPUT_BYTES
        );

        CompositionRunEvidence {
            fixture_id: self.spec.fixture_id.to_owned(),
            stages: vec![inspect, transform],
        }
    }

    #[allow(clippy::too_many_lines)]
    fn run_stage(
        &self,
        stage: CompositionStageSpec,
        input: &CompositionInput,
    ) -> CompositionStageEvidence {
        assert_eq!(stage.consumes, [input.artifact_id.as_str()]);
        assert_eq!(stage.produces.len(), 1);
        assert!(
            stage
                .capability
                .accepted_input_media_types
                .contains(&input.media_type.as_str())
        );
        let bindings = ArtifactBindingSet {
            schema_version: ARTIFACT_BINDINGS_V1.to_owned(),
            binding_set_id: format!("bindings:{}-{}", self.spec.fixture_id, stage.stage_id),
            digest_algorithm: SHA256.to_owned(),
            inputs: vec![InputArtifactBinding {
                artifact_id: input.artifact_id.clone(),
                port: input.port.clone(),
                media_type: input.media_type.clone(),
                kind: input.kind,
                locator: input.locator.clone(),
                expected_digest: input.digest.clone(),
            }],
            outputs: vec![OutputArtifactBinding {
                artifact_id: stage.produces[0].to_owned(),
                port: stage.output_port.to_owned(),
                media_type: stage.capability.output_media_type.to_owned(),
                kind: ArtifactKind::File,
                locator: stage.output_locator.to_owned(),
            }],
        };
        bindings.validate().unwrap();
        let bindings_locator = format!(
            "artifact-bindings/{}-{}.v1.json",
            self.spec.fixture_id, stage.stage_id
        );
        let bindings_path = self
            .root
            .path()
            .join(WORKSPACE_LOCATOR)
            .join(&bindings_locator);
        assert!(!bindings_path.exists());
        fs::write(
            &bindings_path,
            serde_json::to_vec_pretty(&bindings).unwrap(),
        )
        .unwrap();

        let request = ResolutionRequest::new(
            format!("{}-{}", self.spec.fixture_id, stage.stage_id),
            stage.capability.capability_id,
            Domain::Flow,
            "hermetic-process",
            ExecutionModeKind::Process,
        );
        let resolution = self.catalog.resolve(&request);
        let resolved = resolution.resolved().unwrap_or_else(|| {
            panic!(
                "composition stage must resolve: {:#?}",
                resolution.evidence()
            )
        });
        assert_eq!(resolved.extension_id(), stage.provider.extension_id);
        assert_eq!(resolved.fallback_policy(), FallbackPolicy::Forbidden);
        let provider = self.providers.get(stage.provider.extension_id).unwrap();
        let seed = format!("{}-{}-v1", self.spec.fixture_id, stage.stage_id);
        let configuration_values = BTreeMap::from([
            ("mode".to_owned(), serde_json::json!("success")),
            ("seed".to_owned(), serde_json::json!(seed)),
        ]);
        let invocation = ExtensionInvocation {
            schema_version: flow::EXTENSION_INVOCATION_V1.to_owned(),
            invocation_id: format!("invocation:{}-{}", self.spec.fixture_id, stage.stage_id),
            run_id: format!("run:{}", self.spec.fixture_id),
            phase: stage.phase,
            extension: InvocationExtension {
                extension_id: resolved.extension_id().to_owned(),
                version: resolved.version().to_owned(),
                publisher_id: resolved.publisher_id().to_owned(),
                integrity: resolved.integrity().value.clone(),
            },
            capability_id: stage.capability.capability_id.to_owned(),
            interface: InvocationInterface {
                kind: resolved.execution_mode().kind,
                name: resolved.execution_mode().name.clone(),
                protocol: resolved.execution_mode().protocol.clone(),
            },
            input_artifacts: vec![InputArtifact {
                artifact_id: input.artifact_id.clone(),
                digest: input.digest.clone(),
            }],
            expected_output_types: vec![stage.capability.output_media_type.to_owned()],
            configuration: Configuration {
                schema_id: CONFIGURATION_SCHEMA.to_owned(),
                digest: digest_json(&configuration_values),
                values: configuration_values,
            },
            authorization: Authorization {
                authorization_id: format!(
                    "authorization:{}-{}",
                    self.spec.fixture_id, stage.stage_id
                ),
                lock_id: resolved.lock_id().to_owned(),
                grants_digest: provider.grants_digest.clone(),
            },
            limits: resolved.execution_mode().limits.clone(),
            checkpoint_refs: Vec::new(),
            secret_handles: Vec::new(),
            cancellation_id: format!("cancel:{}-{}", self.spec.fixture_id, stage.stage_id),
        };
        invocation.validate().unwrap();

        let subject_lock = self.subject_lock(resolved, &invocation, stage, provider);
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
        authority_profile.authority_profile_id = format!(
            "authority-profile:{}-{}",
            self.spec.fixture_id, stage.stage_id
        );
        authority_profile.requested.argv = composition_provider_argv(&bindings_locator);
        authority_profile
            .granted
            .argv
            .clone_from(&authority_profile.requested.argv);
        let mut enforcement = process_enforcement_evidence(&authority_profile, &subjects_before);
        enforcement.enforcement_evidence_id =
            format!("enforcement:{}-{}", self.spec.fixture_id, stage.stage_id);
        let authority = authorize_process(
            resolved,
            &invocation,
            &subject_lock,
            &subjects_before,
            &authority_profile,
            &enforcement,
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
        assert_eq!(
            execution.result().consumed_artifacts,
            [input.artifact_id.as_str()]
        );
        assert_eq!(execution.result().produced_artifacts, [stage.produces[0]]);
        assert_eq!(
            execution.result().configuration_digest,
            invocation.configuration.digest
        );
        assert_eq!(execution.result().extension_id, stage.provider.extension_id);

        let observed =
            observe_artifacts(&self.root.path().join(WORKSPACE_LOCATOR), &bindings).unwrap();
        let accepted =
            accept_artifacts(resolved, &invocation, &execution, &bindings, &observed).unwrap();
        let output_bytes = fs::read(
            self.root
                .path()
                .join(WORKSPACE_LOCATOR)
                .join(stage.output_locator),
        )
        .unwrap();
        assert_eq!(accepted.outputs()[0].digest, digest_bytes(&output_bytes));
        let subjects_after =
            observe_execution_subjects(self.root.path(), resolved, &invocation, &subject_lock)
                .unwrap();
        assert_eq!(subjects_before.evidence(), subjects_after.evidence());

        CompositionStageEvidence {
            stage_id: stage.stage_id.to_owned(),
            depends_on: stage
                .depends_on
                .iter()
                .map(|dependency| (*dependency).to_owned())
                .collect(),
            bindings,
            resolution: resolution.evidence().clone(),
            invocation,
            subject_lock,
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
        stage: CompositionStageSpec,
        provider: &MaterializedProvider,
    ) -> ExecutionSubjectLock {
        let mut lock: ExecutionSubjectLock = serde_json::from_str(include_str!(
            "../contracts/examples/execution-subject-lock.v1.example.json"
        ))
        .unwrap();
        EXECUTION_SUBJECT_LOCK_V1.clone_into(&mut lock.schema_version);
        lock.subject_lock_id = format!("subject-lock:{}-{}", self.spec.fixture_id, stage.stage_id);
        resolved.lock_id().clone_into(&mut lock.extension_lock_id);
        lock.extension.clone_from(&invocation.extension);
        stage
            .capability
            .capability_id
            .clone_into(&mut lock.capability_id);
        lock.interface.clone_from(&invocation.interface);
        lock.declared_entrypoint
            .clone_from(&resolved.execution_mode().entrypoint);
        provider
            .spec
            .package_subject_id
            .clone_into(&mut lock.package.subject_id);
        provider
            .spec
            .package_locator
            .clone_into(&mut lock.package.locator);
        provider
            .package_digest
            .clone_into(&mut lock.package.digest.value);
        provider
            .spec
            .executable_subject_id
            .clone_into(&mut lock.executable.subject_id);
        lock.executable
            .locator
            .clone_from(&provider.executable_locator);
        provider
            .executable_digest
            .clone_into(&mut lock.executable.digest.value);
        lock.validate().unwrap();
        lock
    }
}

fn assert_composition_fixture(
    spec: CompositionFixtureSpec,
    expected_providers: [&str; 2],
    distinct_packages: bool,
) {
    let (provider_bytes, executable_name) = provider_binary_snapshot();
    let first_fixture = CompositionFixture::new(spec, &provider_bytes, &executable_name);
    let first = first_fixture.run();
    let second_fixture = CompositionFixture::new(spec, &provider_bytes, &executable_name);
    let second = second_fixture.run();

    assert_eq!(first, second, "normalized composition evidence drifted");
    assert_eq!(first.fixture_id, spec.fixture_id);
    assert_eq!(
        first
            .stages
            .iter()
            .map(|stage| stage.stage_id.as_str())
            .collect::<Vec<_>>(),
        ["inspect", "transform"]
    );
    assert!(first.stages[0].depends_on.is_empty());
    assert_eq!(first.stages[1].depends_on, ["inspect"]);
    assert_eq!(
        first.stages[0].accepted.outputs()[0],
        first.stages[1].accepted.inputs()[0]
    );

    for (stage, expected_provider) in first.stages.iter().zip(expected_providers) {
        assert_eq!(stage.invocation.extension.extension_id, expected_provider);
        assert_eq!(stage.subject_lock.extension, stage.invocation.extension);
        assert_eq!(stage.subjects.extension, stage.invocation.extension);
        assert_eq!(stage.execution.result().extension_id, expected_provider);
        assert_eq!(
            stage.execution.result().configuration_digest,
            stage.invocation.configuration.digest
        );
        assert_eq!(stage.accepted.outputs().len(), 1);
        assert_eq!(
            stage.accepted.outputs()[0].digest,
            digest_bytes(&stage.output_bytes)
        );
    }

    if distinct_packages {
        assert_ne!(
            first.stages[0].subject_lock.package.digest.value,
            first.stages[1].subject_lock.package.digest.value
        );
    } else {
        assert_eq!(
            first.stages[0].subject_lock.package.digest.value,
            first.stages[1].subject_lock.package.digest.value
        );
    }
    assert_eq!(
        first.stages[0].subject_lock.executable.digest.value,
        first.stages[1].subject_lock.executable.digest.value,
        "both synthetic providers intentionally reuse the immutable generic executable"
    );
}

fn composition_provider_argv(bindings_locator: &str) -> Vec<String> {
    vec![
        "--artifact-root".to_owned(),
        "../../workspace".to_owned(),
        "--artifact-bindings".to_owned(),
        bindings_locator.to_owned(),
    ]
}

impl KitFixture {
    fn new(capability: CapabilitySpec, provider_bytes: &[u8], executable_name: &str) -> Self {
        Self::new_with_catalog_profile(
            capability,
            provider_bytes,
            executable_name,
            CatalogProfile::default(),
        )
    }

    fn new_with_catalog_profile(
        capability: CapabilitySpec,
        provider_bytes: &[u8],
        executable_name: &str,
        profile: CatalogProfile,
    ) -> Self {
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
        if let Some(requirement) = profile.flow_version_requirement {
            requirement.clone_into(&mut manifest.compatibility.flow_version_requirement);
        }
        package_digest.clone_into(&mut manifest.integrity.value);
        let observation = ExtensionObservation::new(
            manifest.extension_id.clone(),
            manifest.version.clone(),
            manifest.publisher.id.clone(),
            manifest.integrity.clone(),
            profile.available,
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

    fn resolve_case(
        &self,
        capability: CapabilitySpec,
        case_id: &str,
        execution_mode_name: &str,
    ) -> ResolutionOutcome {
        self.catalog.resolve(&ResolutionRequest::new(
            case_id,
            capability.capability_id,
            Domain::Flow,
            execution_mode_name,
            ExecutionModeKind::Process,
        ))
    }

    fn prepare_lifecycle(
        &self,
        capability: CapabilitySpec,
        mode: &str,
        lifecycle_control: bool,
    ) -> PreparedLifecycleRun {
        let resolution = self.resolve_case(
            capability,
            &format!("hermetic-{}-{mode}", capability.name),
            "hermetic-process",
        );
        let resolved = resolution.resolved().unwrap_or_else(|| {
            panic!(
                "hermetic lifecycle provider must resolve: {:#?}",
                resolution.evidence()
            )
        });
        assert_eq!(resolved.fallback_policy(), FallbackPolicy::Forbidden);
        let seed = format!("hermetic-{mode}-v1");
        let configuration_values = BTreeMap::from([
            ("mode".to_owned(), serde_json::json!(mode)),
            ("seed".to_owned(), serde_json::json!(seed)),
        ]);
        let invocation = ExtensionInvocation {
            schema_version: flow::EXTENSION_INVOCATION_V1.to_owned(),
            invocation_id: format!("invocation:hermetic-{mode}"),
            run_id: format!("run:hermetic-{mode}"),
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
                authorization_id: format!("authorization:hermetic-{mode}"),
                lock_id: resolved.lock_id().to_owned(),
                grants_digest: self.grants_digest.clone(),
            },
            limits: resolved.execution_mode().limits.clone(),
            checkpoint_refs: Vec::new(),
            secret_handles: Vec::new(),
            cancellation_id: format!("cancel:hermetic-{mode}"),
        };
        invocation.validate().unwrap();

        let subject_lock = self.subject_lock(resolved, &invocation, capability);
        let subjects =
            observe_execution_subjects(self.root.path(), resolved, &invocation, &subject_lock)
                .unwrap();
        let mut authority_profile = process_authority_profile(
            resolved,
            &invocation,
            &subject_lock,
            &subjects,
            ProcessIsolation::TrustedUnconfined,
        );
        authority_profile.authority_profile_id = format!("authority-profile:hermetic-{mode}");
        authority_profile.requested.argv =
            lifecycle_provider_argv(lifecycle_control.then_some(LIFECYCLE_CONTROL_LOCATOR));
        authority_profile
            .granted
            .argv
            .clone_from(&authority_profile.requested.argv);
        let mut enforcement = process_enforcement_evidence(&authority_profile, &subjects);
        enforcement.enforcement_evidence_id = format!("enforcement:hermetic-{mode}");
        let authority = authorize_process(
            resolved,
            &invocation,
            &subject_lock,
            &subjects,
            &authority_profile,
            &enforcement,
        )
        .unwrap();

        PreparedLifecycleRun {
            resolution,
            invocation,
            subject_lock,
            subjects,
            authority,
        }
    }

    fn output_path(&self) -> PathBuf {
        self.root
            .path()
            .join(WORKSPACE_LOCATOR)
            .join(&self.bindings.outputs[0].locator)
    }

    fn lifecycle_control_path(&self) -> PathBuf {
        self.root
            .path()
            .join(WORKSPACE_LOCATOR)
            .join(LIFECYCLE_CONTROL_LOCATOR)
    }

    fn assert_subjects_unchanged(&self, prepared: &PreparedLifecycleRun) {
        let after = observe_execution_subjects(
            self.root.path(),
            prepared.resolved(),
            &prepared.invocation,
            &prepared.subject_lock,
        )
        .unwrap();
        assert_eq!(prepared.subjects.evidence(), after.evidence());
    }

    fn immutable_workspace_bytes(&self) -> (Vec<u8>, Vec<u8>) {
        let workspace = self.root.path().join(WORKSPACE_LOCATOR);
        (
            fs::read(workspace.join(INPUT_LOCATOR)).unwrap(),
            fs::read(workspace.join(BINDINGS_LOCATOR)).unwrap(),
        )
    }

    fn assert_immutable_workspace_bytes(&self, before: &(Vec<u8>, Vec<u8>)) {
        assert_eq!(&self.immutable_workspace_bytes(), before);
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
        assert!(execution.result().diagnostics.is_empty());
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

fn lifecycle_provider_argv(control_locator: Option<&str>) -> Vec<String> {
    let mut argv = provider_argv();
    if let Some(locator) = control_locator {
        argv.push("--lifecycle-control".to_owned());
        argv.push(locator.to_owned());
    }
    argv
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

fn provider_binary_snapshot() -> (Vec<u8>, String) {
    let provider_path = Path::new(PROVIDER_BINARY);
    let bytes = fs::read(provider_path).unwrap();
    let executable_name = provider_path
        .file_name()
        .and_then(|name| name.to_str())
        .expect("Cargo provider binary path must have a UTF-8 file name")
        .to_owned();
    (bytes, executable_name)
}

#[cfg(unix)]
fn assert_recorded_process_reaped(control_path: &Path) {
    use nix::errno::Errno;
    use nix::sys::wait::{WaitPidFlag, waitpid};
    use nix::unistd::Pid;

    let control: serde_json::Value =
        serde_json::from_slice(&fs::read(control_path).unwrap()).unwrap();
    assert_eq!(control["state"], "ready");
    let pid = i32::try_from(control["pid"].as_u64().unwrap()).unwrap();
    assert_eq!(
        waitpid(Pid::from_raw(pid), Some(WaitPidFlag::WNOHANG)),
        Err(Errno::ECHILD)
    );
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
