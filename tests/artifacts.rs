mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use flow::{
    ARTIFACT_BINDINGS_V1, ArtifactAcceptanceError, ArtifactBindingSet, ArtifactKind,
    ArtifactObservationError, DIRECTORY_MANIFEST_V1, EXTENSION_EVENT_V1, EXTENSION_RESULT_V1,
    EventKind, EventSink, EventState, ExtensionEvent, ExtensionInvocation, ExtensionPort,
    ExtensionResult, Failure, FailureClassification, InputArtifact, InputArtifactBinding,
    Orchestrator, Outcome, OutputArtifactBinding, PortError, PortIdentity, Progress, SHA256,
    ValidatedExecution, accept_artifacts, observe_artifacts,
};
use sha2::{Digest, Sha256};

#[cfg(unix)]
use common::create_unsupported_node;
use common::{invocation, resolved_fixture};

const INPUT_ID: &str = "artifact:source-collection";
const OUTPUT_ID: &str = "artifact:collection-evidence";
const INPUT_TYPE: &str = "application/vnd.flow.collection-reference+json";
const OUTPUT_TYPE: &str = "application/vnd.optiflow.collection-evidence+json";

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TestRoot {
    path: PathBuf,
}

impl TestRoot {
    fn new() -> Self {
        let sequence = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "flow-artifact-test-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("test artifact root must be created");
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

#[derive(Clone, Copy, Debug, Default)]
enum ProviderBehavior {
    #[default]
    Complete,
    DuplicateOutputEvent,
    ExtraOutput,
    OmitInput,
    OmitOutput,
    OmitOutputEvent,
    PartialResult,
}

struct ArtifactPort {
    identity: PortIdentity,
    behavior: ProviderBehavior,
}

impl ExtensionPort for ArtifactPort {
    fn identity(&self) -> &PortIdentity {
        &self.identity
    }

    fn invoke(
        &self,
        invocation: &ExtensionInvocation,
        events: &mut dyn EventSink,
    ) -> Result<ExtensionResult, PortError> {
        emit(
            events,
            &event(
                invocation,
                "started",
                0,
                EventKind::PhaseStarted,
                EventState::Running,
                Vec::new(),
            ),
        )?;
        let mut output_ids = if matches!(self.behavior, ProviderBehavior::OmitOutput) {
            Vec::new()
        } else {
            vec![OUTPUT_ID.to_owned()]
        };
        if matches!(self.behavior, ProviderBehavior::ExtraOutput) {
            output_ids.push("artifact:undeclared".to_owned());
        }

        let mut terminal_sequence = 1;
        if !matches!(self.behavior, ProviderBehavior::OmitOutputEvent) && !output_ids.is_empty() {
            emit(
                events,
                &event(
                    invocation,
                    "artifact",
                    terminal_sequence,
                    EventKind::ArtifactProduced,
                    EventState::Produced,
                    output_ids.clone(),
                ),
            )?;
            terminal_sequence += 1;
        }
        if matches!(self.behavior, ProviderBehavior::DuplicateOutputEvent) {
            emit(
                events,
                &event(
                    invocation,
                    "artifact-duplicate",
                    terminal_sequence,
                    EventKind::ArtifactProduced,
                    EventState::Produced,
                    output_ids.clone(),
                ),
            )?;
            terminal_sequence += 1;
        }
        emit(
            events,
            &event(
                invocation,
                "completed",
                terminal_sequence,
                EventKind::PhaseCompleted,
                EventState::Produced,
                Vec::new(),
            ),
        )?;

        Ok(ExtensionResult {
            schema_version: EXTENSION_RESULT_V1.to_owned(),
            run_id: invocation.run_id.clone(),
            invocation_id: invocation.invocation_id.clone(),
            extension_id: invocation.extension.extension_id.clone(),
            extension_version: invocation.extension.version.clone(),
            extension_integrity: invocation.extension.integrity.clone(),
            capability_id: invocation.capability_id.clone(),
            configuration_digest: invocation.configuration.digest.clone(),
            authorization_id: invocation.authorization.authorization_id.clone(),
            outcome: Outcome::Produced,
            partial_result: matches!(self.behavior, ProviderBehavior::PartialResult),
            consumed_artifacts: if matches!(self.behavior, ProviderBehavior::OmitInput) {
                Vec::new()
            } else {
                vec![INPUT_ID.to_owned()]
            },
            produced_artifacts: output_ids,
            validations: Vec::new(),
            provenance: Vec::new(),
            diagnostics: Vec::new(),
            failure: Failure {
                classification: FailureClassification::None,
                code: String::new(),
                message: String::new(),
                retryable: false,
            },
            checkpoint_refs: Vec::new(),
            explanation: "Synthetic provider artifact evidence for conformance testing.".to_owned(),
        })
    }
}

fn emit(events: &mut dyn EventSink, event: &ExtensionEvent) -> Result<(), PortError> {
    events
        .emit(event)
        .map_err(|error| PortError::new(error.to_string()))
}

fn event(
    invocation: &ExtensionInvocation,
    suffix: &str,
    sequence: u64,
    kind: EventKind,
    state: EventState,
    artifact_refs: Vec<String>,
) -> ExtensionEvent {
    ExtensionEvent {
        schema_version: EXTENSION_EVENT_V1.to_owned(),
        event_id: format!("event:artifact-test-{suffix}"),
        run_id: invocation.run_id.clone(),
        invocation_id: invocation.invocation_id.clone(),
        sequence,
        phase: invocation.phase.lifecycle_point(),
        kind,
        state,
        progress: Progress {
            completed: u64::from(kind != EventKind::PhaseStarted),
            total: 1,
            unit: "artifact-set".to_owned(),
        },
        diagnostics: Vec::new(),
        artifact_refs,
        checkpoint_refs: Vec::new(),
    }
}

fn fixture() -> (TestRoot, ArtifactBindingSet) {
    let root = TestRoot::new();
    fs::create_dir_all(root.path().join("inputs")).expect("input parent directory must be created");
    fs::create_dir_all(root.path().join("outputs/report bundle/nested"))
        .expect("output directory tree must be created");

    let input_bytes = br#"{"collection":"synthetic"}"#;
    fs::write(
        root.path().join("inputs/source collection.json"),
        input_bytes,
    )
    .expect("input fixture must be written");
    fs::write(
        root.path().join("outputs/report bundle/alpha.txt"),
        b"alpha\n",
    )
    .expect("first output fixture must be written");
    fs::write(
        root.path()
            .join("outputs/report bundle/nested/évidence.json"),
        br#"{"valid":true}"#,
    )
    .expect("nested output fixture must be written");

    let bindings = ArtifactBindingSet {
        schema_version: ARTIFACT_BINDINGS_V1.to_owned(),
        binding_set_id: "bindings:artifact-test".to_owned(),
        digest_algorithm: SHA256.to_owned(),
        inputs: vec![InputArtifactBinding {
            artifact_id: INPUT_ID.to_owned(),
            port: "port:collection".to_owned(),
            media_type: INPUT_TYPE.to_owned(),
            kind: ArtifactKind::File,
            locator: "inputs/source collection.json".to_owned(),
            expected_digest: digest(input_bytes),
        }],
        outputs: vec![OutputArtifactBinding {
            artifact_id: OUTPUT_ID.to_owned(),
            port: "port:evidence".to_owned(),
            media_type: OUTPUT_TYPE.to_owned(),
            kind: ArtifactKind::Directory,
            locator: "outputs/report bundle".to_owned(),
        }],
    };
    (root, bindings)
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn configured_invocation(
    resolved: &flow::ResolvedExtension,
    bindings: &ArtifactBindingSet,
) -> ExtensionInvocation {
    let mut invocation = invocation(resolved);
    invocation.input_artifacts = bindings
        .inputs
        .iter()
        .map(|binding| InputArtifact {
            artifact_id: binding.artifact_id.clone(),
            digest: binding.expected_digest.clone(),
        })
        .collect();
    invocation
}

fn execute(
    resolved: &flow::ResolvedExtension,
    invocation: &ExtensionInvocation,
    behavior: ProviderBehavior,
) -> ValidatedExecution {
    let port = ArtifactPort {
        identity: PortIdentity::from_resolved(resolved),
        behavior,
    };
    Orchestrator::execute(resolved, invocation, &port, &mut Vec::new())
        .expect("synthetic artifact provider evidence must validate")
}

#[test]
fn file_and_directory_artifacts_are_observed_deterministically_and_accepted() {
    let (root, bindings) = fixture();
    let first = observe_artifacts(root.path(), &bindings).unwrap();
    let second = observe_artifacts(root.path(), &bindings).unwrap();
    assert_eq!(first, second);
    let debug = format!("{first:?}");
    assert!(!debug.contains(&root.path().display().to_string()));
    assert_eq!(
        first.evidence().directory_manifest_profile,
        DIRECTORY_MANIFEST_V1
    );

    let output = first
        .evidence()
        .artifacts
        .iter()
        .find(|artifact| artifact.artifact_id == OUTPUT_ID)
        .unwrap();
    assert_eq!(output.size_bytes, 20);
    assert_eq!(
        output
            .manifest
            .iter()
            .map(|entry| entry.locator.as_str())
            .collect::<Vec<_>>(),
        ["alpha.txt", "nested", "nested/évidence.json"]
    );

    let (catalog, request) = resolved_fixture();
    let resolution = catalog.resolve(&request);
    let resolved = resolution.resolved().unwrap();
    let invocation = configured_invocation(resolved, &bindings);
    let execution = execute(resolved, &invocation, ProviderBehavior::default());

    let accepted = accept_artifacts(resolved, &invocation, &execution, &bindings, &first).unwrap();
    assert_eq!(accepted.binding_set_id(), bindings.binding_set_id);
    assert_eq!(accepted.inputs().len(), 1);
    assert_eq!(accepted.outputs(), std::slice::from_ref(output));
}

#[test]
fn portable_locator_rules_reject_ambiguous_and_escaping_paths() {
    let (_root, bindings) = fixture();
    for locator in [
        "/absolute/input.json",
        "../outside.json",
        "inputs/../outside.json",
        "inputs\\source.json",
        "inputs//source.json",
        "C:/source.json",
        "inputs/CON.json",
        "inputs/question?.json",
        "inputs/trailing. ",
    ] {
        let mut invalid = bindings.clone();
        locator.clone_into(&mut invalid.inputs[0].locator);
        assert!(
            invalid.validate().is_err(),
            "locator should fail: {locator}"
        );
    }
}

#[test]
fn duplicate_binding_and_observation_keys_are_rejected() {
    let (root, bindings) = fixture();

    let mut duplicate_id = bindings.clone();
    duplicate_id.outputs[0].artifact_id = duplicate_id.inputs[0].artifact_id.clone();
    assert!(duplicate_id.validate().is_err());

    let mut duplicate_port = bindings.clone();
    duplicate_port.outputs[0].port = duplicate_port.inputs[0].port.clone();
    assert!(duplicate_port.validate().is_err());

    let mut duplicate_locator = bindings.clone();
    duplicate_locator.outputs[0].locator = duplicate_locator.inputs[0].locator.clone();
    assert!(duplicate_locator.validate().is_err());

    let observed = observe_artifacts(root.path(), &bindings).unwrap();
    let mut duplicate_observation = observed.evidence().clone();
    let duplicate = duplicate_observation.artifacts[0].clone();
    duplicate_observation.artifacts.push(duplicate);
    assert!(duplicate_observation.validate().is_err());
}

#[cfg(unix)]
#[test]
fn symlinks_are_rejected_without_following_their_targets() {
    use std::os::unix::fs::symlink;

    let (root, mut bindings) = fixture();
    let original_bindings = bindings.clone();
    symlink(
        root.path().join("inputs/source collection.json"),
        root.path().join("inputs/source link.json"),
    )
    .unwrap();
    bindings.inputs[0].locator = "inputs/source link.json".to_owned();

    let error = observe_artifacts(root.path(), &bindings).unwrap_err();
    assert!(matches!(error, ArtifactObservationError::Symlink { .. }));

    let linked_root = root.path().with_extension("link");
    symlink(root.path(), &linked_root).unwrap();
    let error = observe_artifacts(&linked_root, &original_bindings).unwrap_err();
    assert!(matches!(
        error,
        ArtifactObservationError::RootSymlink { .. }
    ));
    fs::remove_file(linked_root).unwrap();
}

#[cfg(unix)]
#[test]
fn non_utf8_names_and_special_nodes_are_rejected() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let (root, bindings) = fixture();
    let invalid_name = OsString::from_vec(vec![b'i', 0xff]);
    fs::write(
        root.path().join("outputs/report bundle").join(invalid_name),
        b"invalid name",
    )
    .unwrap();
    let error = observe_artifacts(root.path(), &bindings).unwrap_err();
    assert!(matches!(
        error,
        ArtifactObservationError::NonUtf8Path { .. }
    ));

    let (root, bindings) = fixture();
    create_unsupported_node(&root.path().join("outputs/report bundle/fifo"));
    let error = observe_artifacts(root.path(), &bindings).unwrap_err();
    assert!(matches!(
        error,
        ArtifactObservationError::UnsupportedNode { .. }
    ));
}

#[test]
fn empty_directories_have_a_stable_identity() {
    let (root, mut bindings) = fixture();
    fs::create_dir(root.path().join("outputs/empty")).unwrap();
    bindings.outputs[0].locator = "outputs/empty".to_owned();

    let observed = observe_artifacts(root.path(), &bindings).unwrap();
    let output = &observed.evidence().artifacts[1];
    assert_eq!(output.digest, digest(b"[]"));
    assert_eq!(output.size_bytes, 0);
    assert!(output.manifest.is_empty());
}

#[test]
fn missing_artifacts_and_kind_mismatches_are_rejected() {
    let (root, bindings) = fixture();
    let mut missing = bindings.clone();
    missing.inputs[0].locator = "inputs/missing.json".to_owned();
    assert!(matches!(
        observe_artifacts(root.path(), &missing),
        Err(ArtifactObservationError::Missing { .. })
    ));

    let mut wrong_kind = bindings;
    wrong_kind.outputs[0].kind = ArtifactKind::File;
    assert!(matches!(
        observe_artifacts(root.path(), &wrong_kind),
        Err(ArtifactObservationError::KindMismatch { .. })
    ));
}

#[test]
fn mismatched_host_provider_and_invocation_evidence_never_becomes_accepted() {
    let (root, bindings) = fixture();
    let observations = observe_artifacts(root.path(), &bindings).unwrap();
    let (catalog, request) = resolved_fixture();
    let resolution = catalog.resolve(&request);
    let resolved = resolution.resolved().unwrap();
    let invocation = configured_invocation(resolved, &bindings);
    let execution = execute(resolved, &invocation, ProviderBehavior::default());

    let mut wrong_bindings = bindings.clone();
    wrong_bindings.outputs[0].port = "port:wrong".to_owned();
    let wrong_observation = observe_artifacts(root.path(), &wrong_bindings).unwrap();
    assert!(matches!(
        accept_artifacts(
            resolved,
            &invocation,
            &execution,
            &bindings,
            &wrong_observation
        ),
        Err(ArtifactAcceptanceError::Mismatch { .. })
    ));

    let mut wrong_identity_bindings = bindings.clone();
    wrong_identity_bindings.outputs[0].artifact_id = "artifact:wrong".to_owned();
    let wrong_identity = observe_artifacts(root.path(), &wrong_identity_bindings).unwrap();
    assert!(matches!(
        accept_artifacts(
            resolved,
            &invocation,
            &execution,
            &bindings,
            &wrong_identity
        ),
        Err(ArtifactAcceptanceError::Mismatch { .. })
    ));

    let mut wrong_type_bindings = bindings.clone();
    wrong_type_bindings.inputs[0].media_type = "text/plain".to_owned();
    let wrong_type = observe_artifacts(root.path(), &wrong_type_bindings).unwrap();
    assert!(matches!(
        accept_artifacts(
            resolved,
            &invocation,
            &execution,
            &wrong_type_bindings,
            &wrong_type
        ),
        Err(ArtifactAcceptanceError::Mismatch { .. })
    ));

    let mut wrong_invocation = invocation.clone();
    wrong_invocation.run_id = "run:different".to_owned();
    let error = accept_artifacts(
        resolved,
        &wrong_invocation,
        &execution,
        &bindings,
        &observations,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ArtifactAcceptanceError::Mismatch { ref message }
            if message == "validated execution context does not match the supplied invocation"
    ));

    let mut wrong_invocation = invocation.clone();
    wrong_invocation.invocation_id = "invocation:different".to_owned();
    let error = accept_artifacts(
        resolved,
        &wrong_invocation,
        &execution,
        &bindings,
        &observations,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ArtifactAcceptanceError::Mismatch { ref message }
            if message == "validated execution context does not match the supplied invocation"
    ));

    let mut incomplete_bindings = bindings.clone();
    incomplete_bindings.outputs.clear();
    let missing_observation = observe_artifacts(root.path(), &incomplete_bindings).unwrap();
    assert!(matches!(
        accept_artifacts(
            resolved,
            &invocation,
            &execution,
            &bindings,
            &missing_observation
        ),
        Err(ArtifactAcceptanceError::Mismatch { .. })
    ));
}

#[test]
fn stale_binding_context_cannot_reuse_an_observation_token() {
    let (root, bindings) = fixture();
    let observations = observe_artifacts(root.path(), &bindings).unwrap();
    let (catalog, request) = resolved_fixture();
    let resolution = catalog.resolve(&request);
    let resolved = resolution.resolved().unwrap();
    let invocation = configured_invocation(resolved, &bindings);
    let execution = execute(resolved, &invocation, ProviderBehavior::default());
    let mut stale_bindings = bindings.clone();
    stale_bindings.binding_set_id = "bindings:stale-artifact-test".to_owned();

    let error = accept_artifacts(
        resolved,
        &invocation,
        &execution,
        &stale_bindings,
        &observations,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ArtifactAcceptanceError::Mismatch { ref message }
            if message == "artifact bindings changed after host observation"
    ));
}

#[test]
fn duplicate_artifact_events_and_partial_results_are_not_completion() {
    let (root, bindings) = fixture();
    let observations = observe_artifacts(root.path(), &bindings).unwrap();
    let (catalog, request) = resolved_fixture();
    let resolution = catalog.resolve(&request);
    let resolved = resolution.resolved().unwrap();
    let invocation = configured_invocation(resolved, &bindings);

    let duplicate = execute(
        resolved,
        &invocation,
        ProviderBehavior::DuplicateOutputEvent,
    );
    assert!(matches!(
        accept_artifacts(resolved, &invocation, &duplicate, &bindings, &observations),
        Err(ArtifactAcceptanceError::Mismatch { .. })
    ));

    let partial = execute(resolved, &invocation, ProviderBehavior::PartialResult);
    assert!(matches!(
        accept_artifacts(resolved, &invocation, &partial, &bindings, &observations),
        Err(ArtifactAcceptanceError::Mismatch { .. })
    ));
}

#[test]
fn provider_artifact_omissions_and_extras_are_rejected_after_execution_validation() {
    let (root, bindings) = fixture();
    let observations = observe_artifacts(root.path(), &bindings).unwrap();
    let (catalog, request) = resolved_fixture();
    let resolution = catalog.resolve(&request);
    let resolved = resolution.resolved().unwrap();
    let invocation = configured_invocation(resolved, &bindings);

    let cases = [
        ("missing consumed input", ProviderBehavior::OmitInput),
        ("missing produced output", ProviderBehavior::OmitOutput),
        ("missing artifact event", ProviderBehavior::OmitOutputEvent),
        ("undeclared output", ProviderBehavior::ExtraOutput),
    ];

    for (case, behavior) in cases {
        let execution = execute(resolved, &invocation, behavior);
        assert!(
            matches!(
                accept_artifacts(resolved, &invocation, &execution, &bindings, &observations),
                Err(ArtifactAcceptanceError::Mismatch { .. })
            ),
            "provider case should fail artifact acceptance: {case}"
        );
    }
}

#[test]
fn changed_input_bytes_fail_the_immutable_digest_gate() {
    let (root, bindings) = fixture();
    fs::write(
        root.path().join("inputs/source collection.json"),
        b"changed after binding",
    )
    .unwrap();
    let observations = observe_artifacts(root.path(), &bindings).unwrap();
    let (catalog, request) = resolved_fixture();
    let resolution = catalog.resolve(&request);
    let resolved = resolution.resolved().unwrap();
    let invocation = configured_invocation(resolved, &bindings);
    let execution = execute(resolved, &invocation, ProviderBehavior::default());

    let error =
        accept_artifacts(resolved, &invocation, &execution, &bindings, &observations).unwrap_err();
    assert!(matches!(
        error,
        ArtifactAcceptanceError::Mismatch { ref message }
            if message == "host-observed input digest conflicts with the immutable binding"
    ));
}

#[test]
fn changed_input_after_observation_cannot_be_accepted() {
    let (root, bindings) = fixture();
    let observations = observe_artifacts(root.path(), &bindings).unwrap();
    let (catalog, request) = resolved_fixture();
    let resolution = catalog.resolve(&request);
    let resolved = resolution.resolved().unwrap();
    let invocation = configured_invocation(resolved, &bindings);
    let execution = execute(resolved, &invocation, ProviderBehavior::default());

    fs::write(
        root.path().join("inputs/source collection.json"),
        b"changed after observation",
    )
    .unwrap();

    let error =
        accept_artifacts(resolved, &invocation, &execution, &bindings, &observations).unwrap_err();
    assert!(matches!(error, ArtifactAcceptanceError::ObservationChanged));
}

#[test]
fn changed_output_after_observation_cannot_be_accepted() {
    let (root, bindings) = fixture();
    let observations = observe_artifacts(root.path(), &bindings).unwrap();
    let (catalog, request) = resolved_fixture();
    let resolution = catalog.resolve(&request);
    let resolved = resolution.resolved().unwrap();
    let invocation = configured_invocation(resolved, &bindings);
    let execution = execute(resolved, &invocation, ProviderBehavior::default());

    fs::write(
        root.path().join("outputs/report bundle/alpha.txt"),
        b"changed after observation\n",
    )
    .unwrap();

    let error =
        accept_artifacts(resolved, &invocation, &execution, &bindings, &observations).unwrap_err();
    assert!(matches!(error, ArtifactAcceptanceError::ObservationChanged));
}

#[test]
fn removed_output_after_observation_returns_typed_reobservation_error() {
    let (root, bindings) = fixture();
    let observations = observe_artifacts(root.path(), &bindings).unwrap();
    let (catalog, request) = resolved_fixture();
    let resolution = catalog.resolve(&request);
    let resolved = resolution.resolved().unwrap();
    let invocation = configured_invocation(resolved, &bindings);
    let execution = execute(resolved, &invocation, ProviderBehavior::default());

    fs::remove_dir_all(root.path().join("outputs/report bundle")).unwrap();

    let error =
        accept_artifacts(resolved, &invocation, &execution, &bindings, &observations).unwrap_err();
    assert!(matches!(
        error,
        ArtifactAcceptanceError::Reobservation {
            source: ArtifactObservationError::Missing {
                ref artifact_id,
                ref locator,
            }
        } if artifact_id == OUTPUT_ID && locator == "outputs/report bundle"
    ));
}

#[test]
fn corrupt_or_contradictory_portable_observations_are_invalid_before_correlation() {
    let (root, bindings) = fixture();
    let observed = observe_artifacts(root.path(), &bindings).unwrap();

    let mut corrupt_digest = observed.evidence().clone();
    corrupt_digest.artifacts[1].digest = "corrupt".to_owned();
    let error = corrupt_digest.validate().unwrap_err();
    assert_eq!(error.path, "observations.artifacts[1].digest");
    assert_eq!(error.message, "must be 64 lowercase hexadecimal characters");

    let mut observations = observed.evidence().clone();
    let output = observations
        .artifacts
        .iter_mut()
        .find(|artifact| artifact.artifact_id == OUTPUT_ID)
        .unwrap();
    output.manifest.swap(0, 1);

    let error = observations.validate().unwrap_err();
    assert_eq!(error.path, "observations.artifacts[1].manifest");
    assert_eq!(
        error.message,
        "entries must be unique and sorted by locator"
    );
}
