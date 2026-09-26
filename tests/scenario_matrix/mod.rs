//! Test-owned recipes over public Flow boundaries, not a production scheduler.

use super::*;
use flow::{
    Orchestrator, ProcessCompletion, ProcessTranscript, ScenarioEvidenceState, ScenarioManifest,
    ScenarioTerminalState,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::time::Instant;

mod artifacts;
mod contracts;
mod privacy;
mod resolution;

const CATALOG: &str = include_str!("../fixtures/acceptance-scenarios.v1.json");
const CANARIES: [&str; 4] = [
    "FLOW_ARGV_CANARY_7429",
    "FLOW_ENV_CANARY_1938",
    "FLOW_SECRET_CANARY_5813",
    "FLOW_PRIVATE_CANARY_2604",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    schema_version: String,
    fixture_version: String,
    tier: String,
    budget: Budget,
    known_gaps: Vec<String>,
    scenarios: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Budget {
    timeout_ms: u64,
    max_receipt_bytes: usize,
    max_artifact_bytes: u64,
    max_artifacts: usize,
    repetitions: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    scenario_id: String,
    family: String,
    recipe: String,
    expected: Outcome,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Outcome {
    boundary: String,
    code: String,
    terminal_state: ScenarioTerminalState,
    evidence_state: ScenarioEvidenceState,
    accepted: bool,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct Receipt {
    schema_version: &'static str,
    scenario_id: String,
    recipe_digest: String,
    fixture_identity: Value,
    outcome: Outcome,
    recovery: &'static str,
    evidence: Value,
}

fn outcome(boundary: &str, code: &str, state: ScenarioTerminalState) -> Outcome {
    let evidence_state = match state {
        ScenarioTerminalState::Complete => ScenarioEvidenceState::Complete,
        ScenarioTerminalState::Partial | ScenarioTerminalState::Interrupted => {
            ScenarioEvidenceState::Incomplete
        }
        ScenarioTerminalState::Unavailable => ScenarioEvidenceState::Unavailable,
        ScenarioTerminalState::Unsupported => ScenarioEvidenceState::Unsupported,
        ScenarioTerminalState::Invalid => ScenarioEvidenceState::Invalid,
        ScenarioTerminalState::Failed => ScenarioEvidenceState::Failed,
    };
    Outcome {
        boundary: boundary.to_owned(),
        code: code.to_owned(),
        terminal_state: state,
        evidence_state,
        accepted: false,
    }
}

fn invalid(boundary: &str, code: &str) -> Outcome {
    outcome(boundary, code, ScenarioTerminalState::Invalid)
}

fn recovery(boundary: &str) -> &'static str {
    match boundary {
        "resolution" => {
            "Inspect pinned versions, availability, and operator selection policy before invocation."
        }
        "contract" => "Repair the named versioned document before resolving or executing it.",
        "preflight" => {
            "Rebuild matching invocation, subject, and authority evidence before launch."
        }
        "transcript" => {
            "Inspect the provider protocol; retain failure and do not switch providers after invocation."
        }
        "observation" => "Repair root-relative declarations or candidate files and observe again.",
        "acceptance" => {
            "Preserve the rejected candidate; validate complete, current, exactly declared evidence."
        }
        "privacy" => "Keep raw operational evidence local; export only the allowlisted receipt.",
        other => panic!("unhandled recovery boundary: {other}"),
    }
}

#[test]
#[allow(clippy::too_many_lines)] // One visible receipt lifecycle for each catalog row.
fn executable_acceptance_matrix() {
    let catalog: Catalog = serde_json::from_str(CATALOG).unwrap();
    assert_eq!(
        catalog.schema_version,
        "flow.acceptance-scenario-catalog/v1"
    );
    assert_eq!(catalog.fixture_version, "1.0.0");
    assert_eq!(catalog.tier, "pull-request");
    assert!(!catalog.known_gaps.is_empty());
    assert_eq!(catalog.budget.repetitions, 2);
    assert!(catalog.budget.timeout_ms > 0);
    let (provider_bytes, executable_name) = provider_binary_snapshot();
    let mut ids = BTreeSet::new();
    let mut recipes = BTreeSet::new();
    let mut states = BTreeSet::new();
    for case in &catalog.scenarios {
        assert!(ids.insert(&case.scenario_id), "duplicate scenario ID");
        assert!(
            recipes.insert((&case.family, &case.recipe)),
            "duplicate recipe"
        );
        assert!(case.scenario_id.starts_with("scenario:acceptance-"));
        validate_expectation(case);
        let mut previous = None;
        for _ in 0..catalog.budget.repetitions {
            let started = Instant::now();
            let fixture = KitFixture::new(CAPABILITIES[0], &provider_bytes, &executable_name);
            let original_input = fs::read(
                fixture
                    .root
                    .path()
                    .join(WORKSPACE_LOCATOR)
                    .join(INPUT_LOCATOR),
            )
            .unwrap();
            let (actual, evidence) = match case.family.as_str() {
                "resolution" => resolution::run(&fixture, &case.recipe),
                "contract" => contracts::run(&fixture, &case.recipe),
                "artifact" => artifacts::run(&fixture, &case.recipe),
                "provider" => provider_case(&fixture, &case.recipe),
                "privacy" => privacy_case(&fixture, &case.recipe),
                other => panic!("unknown scenario family: {other}"),
            };
            assert_eq!(actual, case.expected, "{}", case.scenario_id);
            // Some artifact recipes intentionally alter the input to prove refusal.
            if !case.recipe.starts_with("changed-input") {
                assert_eq!(
                    fs::read(
                        fixture
                            .root
                            .path()
                            .join(WORKSPACE_LOCATOR)
                            .join(INPUT_LOCATOR)
                    )
                    .unwrap(),
                    original_input
                );
            }
            check_artifact_budget(&fixture, &catalog.budget);
            let receipt = Receipt {
                schema_version: "flow.acceptance-scenario-receipt/v1",
                scenario_id: case.scenario_id.clone(),
                recipe_digest: digest_json(
                    &json!({"version": catalog.fixture_version, "family": case.family, "recipe": case.recipe}),
                ),
                fixture_identity: json!({"package_digest": fixture.package_digest, "executable_digest": fixture.executable_digest, "manifest_digest": digest_json(&fixture.manifest), "input_digest": digest_bytes(INPUT_BYTES)}),
                recovery: recovery(&actual.boundary),
                outcome: actual,
                evidence,
            };
            let encoded = serde_json::to_string(&receipt).unwrap();
            assert!(encoded.len() <= catalog.budget.max_receipt_bytes);
            assert_private_values_absent(&encoded, fixture.root.path());
            assert!(
                started.elapsed().as_millis() <= u128::from(catalog.budget.timeout_ms),
                "scenario budget exceeded: {}",
                case.scenario_id
            );
            if let Some(previous) = &previous {
                assert_eq!(&receipt, previous, "fresh-root drift: {}", case.scenario_id);
            }
            previous = Some(receipt);
        }
        let receipt = previous.unwrap();
        states.insert(serde_json::to_string(&receipt.outcome.evidence_state).unwrap());
        println!(
            "FLOW_ACCEPTANCE_RECEIPT={}",
            serde_json::to_string(&receipt).unwrap()
        );
    }
    for state in [
        "complete",
        "observed-empty",
        "unavailable",
        "incomplete",
        "unsupported",
        "invalid",
        "failed",
    ] {
        assert!(
            states.contains(&format!("\"{state}\"")),
            "missing distinct evidence state {state}"
        );
    }
}

fn validate_expectation(case: &Case) {
    // Reuse the accepted scenario contract for the terminal/evidence vocabulary.
    // This is an intent check, not a claim that its illustrative package ran.
    let mut manifest: ScenarioManifest = serde_json::from_str(include_str!(
        "../../contracts/examples/scenario-manifest.v1.example.json"
    ))
    .unwrap();
    manifest.scenario_id.clone_from(&case.scenario_id);
    manifest.fixture_id = case.scenario_id.replace("scenario:", "fixture:");
    manifest.expectation.terminal_state = case.expected.terminal_state;
    manifest.expectation.evidence_state = case.expected.evidence_state;
    manifest.expectation.expected_artifacts.clear();
    manifest.validate().unwrap();
}

fn check_artifact_budget(fixture: &KitFixture, budget: &Budget) {
    let root = fixture.root.path().join(WORKSPACE_LOCATOR).join("outputs");
    if fs::symlink_metadata(&root)
        .unwrap()
        .file_type()
        .is_symlink()
    {
        // The observer has rejected this recipe; never follow its target here.
        return;
    }
    let entries = fs::read_dir(root)
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    assert!(entries.len() <= budget.max_artifacts);
    let bytes: u64 = entries
        .iter()
        .map(|entry| {
            let metadata = fs::symlink_metadata(entry.path()).unwrap();
            if metadata.is_file() {
                metadata.len()
            } else {
                0
            }
        })
        .sum();
    assert!(bytes <= budget.max_artifact_bytes);
}

fn assert_private_values_absent(value: &str, root: &Path) {
    for canary in CANARIES {
        assert!(!value.contains(canary), "portable evidence leaked a canary");
    }
    assert!(
        !value.contains(root.to_str().unwrap()),
        "portable evidence leaked the host root"
    );
}

fn execute(
    fixture: &KitFixture,
    prepared: &PreparedLifecycleRun,
) -> Result<ValidatedExecution, ProcessRunnerError> {
    LocalProcessRunner::run(
        fixture.root.path(),
        prepared.resolved(),
        &prepared.invocation,
        &prepared.subject_lock,
        &prepared.authority,
        &NoSecrets,
        &mut Vec::new(),
    )
}

fn execution_error(error: &ExecutionError) -> Outcome {
    match error {
        ExecutionError::InvalidInvocation { .. } => invalid("preflight", "invalid-invocation"),
        ExecutionError::Preflight { .. } => invalid("preflight", "context-mismatch"),
        ExecutionError::InvalidEvent { .. } => invalid("transcript", "invalid-event"),
        ExecutionError::InvalidResult { .. } => invalid("transcript", "invalid-result"),
        ExecutionError::ProcessProtocol { .. } => invalid("transcript", "invalid-protocol"),
        ExecutionError::EventSink { .. } => invalid("transcript", "host-rejection"),
        ExecutionError::ProcessExit { .. } => {
            outcome("transcript", "nonzero-exit", ScenarioTerminalState::Failed)
        }
        ExecutionError::ProcessOutputLimit {
            stream,
            limit,
            observed,
        } => {
            assert_eq!(*observed, limit + 1);
            invalid(
                "transcript",
                match stream {
                    ProcessStream::Stdout => "stdout-limit",
                    ProcessStream::Stderr => "stderr-limit",
                },
            )
        }
        other => panic!("unclassified execution failure: {other:?}"),
    }
}

fn acceptance_error(error: &ArtifactAcceptanceError) -> Outcome {
    match error {
        ArtifactAcceptanceError::Mismatch { message } => {
            if message == "artifact acceptance requires a complete produced or reused result" {
                outcome(
                    "acceptance",
                    "incomplete-result",
                    ScenarioTerminalState::Partial,
                )
            } else {
                invalid("acceptance", "evidence-mismatch")
            }
        }
        ArtifactAcceptanceError::ObservationChanged => invalid("acceptance", "observation-changed"),
        ArtifactAcceptanceError::Reobservation { .. } => {
            invalid("acceptance", "reobservation-failed")
        }
        other => panic!("unclassified acceptance failure: {other:?}"),
    }
}

fn observation_error(error: &ArtifactObservationError) -> Outcome {
    let code = match error {
        ArtifactObservationError::InvalidBindings { .. } => "invalid-bindings",
        ArtifactObservationError::Missing { .. } => "missing-artifact",
        ArtifactObservationError::KindMismatch { .. } => "kind-mismatch",
        ArtifactObservationError::Symlink { .. } | ArtifactObservationError::RootSymlink { .. } => {
            "symlink"
        }
        ArtifactObservationError::PathEscape { .. } => "path-escape",
        other => panic!("unclassified observation failure: {other:?}"),
    };
    invalid("observation", code)
}

fn accepted_evidence(accepted: &AcceptedArtifactSet) -> (Outcome, Value) {
    let mut result = outcome("acceptance", "accepted", ScenarioTerminalState::Complete);
    result.accepted = true;
    (
        result,
        json!({"inputs": accepted.inputs(), "outputs": accepted.outputs()}),
    )
}

fn transcript(events: &[flow::ExtensionEvent], result: &flow::ExtensionResult) -> Vec<u8> {
    let mut bytes = Vec::new();
    for event in events {
        serde_json::to_writer(&mut bytes, event).unwrap();
        bytes.push(b'\n');
    }
    serde_json::to_writer(&mut bytes, result).unwrap();
    bytes.push(b'\n');
    bytes
}

fn validate_transcript(
    prepared: &PreparedLifecycleRun,
    bytes: &[u8],
    stderr: &[u8],
) -> Result<ValidatedExecution, ExecutionError> {
    Orchestrator::validate_process_transcript(
        prepared.resolved(),
        &prepared.invocation,
        &prepared.subject_lock,
        &prepared.subjects,
        &prepared.authority,
        ProcessTranscript::new(ProcessCompletion::Exited { code: Some(0) }, bytes, stderr),
        &mut Vec::new(),
    )
}

fn provider_case(fixture: &KitFixture, recipe: &str) -> (Outcome, Value) {
    let prepared = fixture.prepare_lifecycle(CAPABILITIES[0], recipe, false);
    let execution = if recipe == "success-with-host-rejection" {
        LocalProcessRunner::run(
            fixture.root.path(),
            prepared.resolved(),
            &prepared.invocation,
            &prepared.subject_lock,
            &prepared.authority,
            &NoSecrets,
            &mut |_: &flow::ExtensionEvent| Err(EventSinkError::new(CANARIES[3])),
        )
    } else {
        execute(fixture, &prepared)
    };
    fixture.assert_subjects_unchanged(&prepared);
    match execution {
        Err(ProcessRunnerError::Validation { source }) => (
            execution_error(&source),
            json!({"retained_event_count": source.events().len(), "output_exists": fixture.output_path().exists()}),
        ),
        Err(other) => panic!("unexpected runner failure: {other:?}"),
        Ok(execution) => {
            let observed = match observe_artifacts(
                &fixture.root.path().join(WORKSPACE_LOCATOR),
                &fixture.bindings,
            ) {
                Ok(observed) => observed,
                Err(error) => {
                    return (
                        observation_error(&error),
                        json!({"provider_outcome": execution.result().outcome}),
                    );
                }
            };
            match accept_artifacts(
                prepared.resolved(),
                &prepared.invocation,
                &execution,
                &fixture.bindings,
                &observed,
            ) {
                Ok(accepted) => accepted_evidence(&accepted),
                Err(error) => (
                    acceptance_error(&error),
                    json!({"provider_outcome": execution.result().outcome, "partial_result": execution.result().partial_result}),
                ),
            }
        }
    }
}

fn privacy_case(fixture: &KitFixture, recipe: &str) -> (Outcome, Value) {
    if recipe == "argv-environment" {
        return privacy::argv_environment();
    }
    let prepared = fixture.prepare_lifecycle(CAPABILITIES[0], "success", false);
    let execution = execute(fixture, &prepared).unwrap();
    let mut result = execution.result().clone();
    let mut events = execution.events().to_vec();
    match recipe {
        "operational-stream" => {
            let private = CANARIES.join(" ");
            let accepted =
                validate_transcript(&prepared, &transcript(&events, &result), private.as_bytes())
                    .unwrap();
            assert_eq!(accepted, execution);
            assert_private_values_absent(&format!("{accepted:?}"), fixture.root.path());
            assert!(!format!("{:?}", flow::SecretValue::new(CANARIES[2])).contains(CANARIES[2]));
        }
        "event-diagnostic" | "result-diagnostic" => {
            let diagnostic = flow::Diagnostic {
                severity: Severity::Error,
                code: "canary.private".to_owned(),
                message: CANARIES.join(" "),
                redacted: false,
            };
            if recipe == "event-diagnostic" {
                events[0].diagnostics.push(diagnostic);
            } else {
                result.diagnostics.push(diagnostic);
            }
            let error =
                validate_transcript(&prepared, &transcript(&events, &result), &[]).unwrap_err();
            return (
                execution_error(&error),
                json!({"raw_evidence_retained_locally": true}),
            );
        }
        "argv-environment" => {
            let mut profile = process_authority_profile(
                prepared.resolved(),
                &prepared.invocation,
                &prepared.subject_lock,
                &prepared.subjects,
                ProcessIsolation::TrustedUnconfined,
            );
            profile.requested.argv = vec![CANARIES[0].to_owned()];
            profile.granted.argv = profile.requested.argv.clone();
            let enforcement = process_enforcement_evidence(&profile, &prepared.subjects);
            let authority = authorize_process(
                prepared.resolved(),
                &prepared.invocation,
                &prepared.subject_lock,
                &prepared.subjects,
                &profile,
                &enforcement,
            )
            .unwrap();
            // Authority profiles are host-owned inputs and retain literal argv.
            // They must never be copied wholesale into a portable result.
            assert!(authority.profile().requested.argv[0].contains(CANARIES[0]));
            assert_private_values_absent(&format!("{execution:?}"), fixture.root.path());
        }
        other => panic!("unknown privacy recipe {other}"),
    }
    (
        outcome(
            "privacy",
            "canaries-absent",
            ScenarioTerminalState::Complete,
        ),
        json!({"canary_count": CANARIES.len(), "execution_outcome": result.outcome}),
    )
}
