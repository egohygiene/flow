//! Versioned, test-owned projections of public durable state, never a runtime model.
use crate::{NoSecrets, digest_bytes, digest_json, provider_binary_snapshot};
use flow::{
    NeverCancelled, RunFailure, RunRecoveryAction, RunState, RunStepAssessment, RunStepStatus,
    RunStore,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::time::Instant;

mod fixture;
mod recipes;
mod safety;
use fixture::Fixture;

const CATALOG: &str = include_str!("../fixtures/lifecycle-scenarios.v1.json");

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
    test_threads: usize,
    repetitions: usize,
    timeout_ms: u64,
    max_receipt_bytes: usize,
    max_snapshots: usize,
    max_history_bytes: u64,
    max_artifacts: usize,
    max_artifact_bytes: u64,
    max_fixture_bytes: u64,
    max_process_address_space_bytes: u64,
    max_process_file_bytes: u64,
    max_driver_output_bytes: u64,
    driver_timeout_seconds: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    scenario_id: String,
    recipe: String,
    mode: String,
    graph: bool,
    recovery: String,
    expected: Outcome,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Outcome {
    code: String,
    history: Vec<Frame>,
    assessments: Vec<Assessment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    safety: Option<safety::SafetyEvidence>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Frame {
    sequence: u64,
    steps: Vec<Step>,
    recovery: Vec<RunRecoveryAction>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    status: RunStepStatus,
    attempt: u64,
    failure: Option<RunFailure>,
    checkpoint: bool,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Assessment {
    at: String,
    sequence: u64,
    steps: Vec<RunStepAssessment>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct Receipt {
    schema_version: &'static str,
    scenario_id: String,
    recipe_digest: String,
    fixture_identity: Vec<Value>,
    outcome: Outcome,
    recovery: String,
    plan_digest: String,
    history_digest: String,
    artifact_digests: Vec<Option<String>>,
}

fn history(path: &Path) -> Vec<Frame> {
    snapshot_paths(path)
        .iter()
        .map(|path| {
            let value: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            let state: RunState = serde_json::from_value(value["state"].clone()).unwrap();
            Frame {
                sequence: state.sequence,
                steps: state
                    .steps
                    .iter()
                    .map(|step| Step {
                        status: step.status,
                        attempt: step.attempt,
                        failure: step.failure,
                        checkpoint: step.checkpoint.is_some(),
                    })
                    .collect(),
                recovery: state
                    .recovery_decisions
                    .iter()
                    .map(|decision| decision.action)
                    .collect(),
            }
        })
        .collect()
}

fn snapshot_paths(path: &Path) -> Vec<std::path::PathBuf> {
    let mut paths: Vec<_> = fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    paths.sort();
    paths
}

fn footprint(path: &Path) -> (usize, u64) {
    let mut count = 0;
    let mut bytes = 0;
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let metadata = fs::symlink_metadata(entry.path()).unwrap();
        assert!(!metadata.is_symlink());
        if metadata.is_dir() {
            let (nested_count, nested_bytes) = footprint(&entry.path());
            count += nested_count;
            bytes += nested_bytes;
        } else {
            count += 1;
            bytes += metadata.len();
        }
    }
    (count, bytes)
}

fn check_budget(fixture: &Fixture, budget: &Budget) {
    let snapshots = snapshot_paths(&fixture.workspace());
    assert!(snapshots.len() <= budget.max_snapshots);
    assert!(footprint(&fixture.workspace()).1 <= budget.max_history_bytes);
    let mut fixture_bytes = 0;
    for node in &fixture.nodes {
        fixture_bytes += footprint(node.kit.root.path()).1;
        let (artifacts, bytes) = footprint(&node.kit.root.path().join(crate::WORKSPACE_LOCATOR));
        assert!(artifacts <= budget.max_artifacts && bytes <= budget.max_artifact_bytes);
    }
    assert!(fixture_bytes <= budget.max_fixture_bytes);
}

#[test]
fn executable_lifecycle_matrix() {
    let catalog: Catalog = serde_json::from_str(CATALOG).unwrap();
    assert_eq!(catalog.schema_version, "flow.lifecycle-scenario-catalog/v1");
    assert_eq!(catalog.fixture_version, "1.1.0");
    assert_eq!(catalog.tier, "pull-request-linux");
    assert!(!catalog.known_gaps.is_empty());
    assert_eq!(catalog.budget.repetitions, 2);
    assert_eq!(catalog.budget.test_threads, 1);
    // These are enforced by the Python runner; direct cargo is a diagnostic run.
    assert_eq!(
        catalog.budget.max_process_address_space_bytes,
        4_294_967_296
    );
    assert_eq!(catalog.budget.max_process_file_bytes, 16_777_216);
    assert_eq!(catalog.budget.max_driver_output_bytes, 4_194_304);
    assert_eq!(catalog.budget.driver_timeout_seconds, 600);
    let (bytes, name) = provider_binary_snapshot();
    let mut ids = BTreeSet::new();
    let mut recipes = BTreeSet::new();
    for case in &catalog.scenarios {
        assert!(ids.insert(&case.scenario_id) && recipes.insert(&case.recipe));
        assert!(case.scenario_id.starts_with("scenario:lifecycle-"));
        let mut previous = None;
        for _ in 0..catalog.budget.repetitions {
            let started = Instant::now();
            let mut fixture = Fixture::new(&case.mode, case.graph, &bytes, &name);
            let fixture_identity = fixture.identities();
            let plan_digest = digest_json(&fixture.plan);
            let outcome = recipes::run(&mut fixture, &case.recipe);
            assert_eq!(outcome, case.expected, "{}", case.scenario_id);
            fixture.verify_preservation(&case.recipe);
            check_budget(&fixture, &catalog.budget);
            let receipt = Receipt {
                schema_version: "flow.lifecycle-scenario-receipt/v1",
                scenario_id: case.scenario_id.clone(),
                recipe_digest: digest_json(
                    &json!({"fixture_version": catalog.fixture_version, "case": case}),
                ),
                fixture_identity,
                outcome,
                recovery: case.recovery.clone(),
                plan_digest,
                history_digest: digest_json(
                    &snapshot_paths(&fixture.workspace())
                        .iter()
                        .map(|path| digest_bytes(&fs::read(path).unwrap()))
                        .collect::<Vec<_>>(),
                ),
                artifact_digests: fixture
                    .nodes
                    .iter()
                    .map(|node| {
                        node.kit
                            .output_path()
                            .is_file()
                            .then(|| digest_bytes(&fs::read(node.kit.output_path()).unwrap()))
                    })
                    .collect(),
            };
            let encoded = serde_json::to_string(&receipt).unwrap();
            for canary in [
                "FLOW_LIFECYCLE_FAILURE_PRIVATE_CANARY",
                "FLOW_EFFECT_PRIVATE_CANARY",
                "FLOW_CLEANUP_PRIVATE_CANARY",
            ] {
                assert!(!encoded.contains(canary));
            }
            assert!(encoded.len() <= catalog.budget.max_receipt_bytes);
            for node in &fixture.nodes {
                for private in [
                    node.kit.root.path().to_str().unwrap(),
                    node.prepared.invocation.configuration.values["seed"]
                        .as_str()
                        .unwrap(),
                    std::str::from_utf8(crate::INPUT_BYTES).unwrap(),
                ] {
                    assert!(
                        !encoded.contains(private),
                        "portable receipt leaks raw fixture values"
                    );
                }
            }
            assert!(started.elapsed().as_millis() <= u128::from(catalog.budget.timeout_ms));
            if let Some(prior) = &previous {
                assert_eq!(&receipt, prior, "{}", case.scenario_id);
            }
            previous = Some(receipt);
        }
        println!(
            "FLOW_LIFECYCLE_RECEIPT={}",
            serde_json::to_string(&previous.unwrap()).unwrap()
        );
    }
}

#[test]
#[ignore = "subprocess entry point; invoked by the executable lifecycle matrix"]
fn lifecycle_child() {
    recipes::child();
}
