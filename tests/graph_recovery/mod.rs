use super::{
    BINDINGS_LOCATOR, CAPABILITIES, INPUT_LOCATOR, KitFixture, NoSecrets, PACKAGE_LOCATOR,
    PreparedLifecycleRun, WORKSPACE_LOCATOR, provider_binary_snapshot,
};
use flow::{
    DurableExecutionError, EventKind, ExtensionEvent, NeverCancelled, PlannedStep,
    ProcessStepContext, RUN_PLAN_V1, RecoveryApproval, ResumeEligibility, RunAssessment, RunPlan,
    RunRecoveryAction, RunState, RunStepContext, RunStepStatus, RunStore, StateBoundary,
    StateError,
};
use std::fs;
use std::path::PathBuf;

const IDS: [&str; 5] = ["step:a", "step:b", "step:c", "step:d", "step:e"];
const DEPENDENCIES: [&[&str]; 5] = [&[], &["step:a"], &["step:b"], &[], &["step:b", "step:d"]];

struct Node {
    kit: KitFixture,
    prepared: PreparedLifecycleRun,
    artifacts: PathBuf,
}

impl Node {
    fn context(&self) -> ProcessStepContext<'_> {
        ProcessStepContext {
            execution_root: self.kit.root.path(),
            artifact_root: &self.artifacts,
            resolved: self.prepared.resolved(),
            invocation: &self.prepared.invocation,
            subjects: &self.prepared.subject_lock,
            authority: &self.prepared.authority,
            bindings: &self.kit.bindings,
        }
    }
}

struct Graph {
    nodes: Vec<Node>,
    plan: RunPlan,
}

impl Graph {
    fn new() -> Self {
        let (bytes, name) = provider_binary_snapshot();
        // Real processes with independent source artifacts and explicit ordering
        // dependencies. Separate roots make local stale evidence distinguishable.
        let nodes: Vec<_> = (0..5)
            .map(|index| {
                let mut kit = KitFixture::new(CAPABILITIES[0], &bytes, &name);
                kit.bindings.outputs[0].artifact_id = format!("artifact:graph-{index}");
                let artifacts = kit.root.path().join(WORKSPACE_LOCATOR);
                fs::write(
                    artifacts.join(BINDINGS_LOCATOR),
                    serde_json::to_vec(&kit.bindings).unwrap(),
                )
                .unwrap();
                let prepared = kit.prepare_lifecycle_named(
                    CAPABILITIES[0],
                    "success",
                    false,
                    &format!("graph-{index}"),
                );
                Node {
                    kit,
                    prepared,
                    artifacts,
                }
            })
            .collect();
        let plan = RunPlan {
            schema_version: RUN_PLAN_V1.to_owned(),
            plan_id: "plan:graph-recovery".to_owned(),
            run_id: nodes[0].prepared.invocation.run_id.clone(),
            steps: nodes
                .iter()
                .enumerate()
                .map(|(index, node)| {
                    PlannedStep::prepare(
                        IDS[index].to_owned(),
                        DEPENDENCIES[index]
                            .iter()
                            .map(|id| (*id).to_owned())
                            .collect(),
                        &node.context(),
                    )
                    .unwrap()
                })
                .collect(),
        };
        plan.validate().unwrap();
        Self { nodes, plan }
    }

    fn contexts(&self) -> Vec<RunStepContext<'_>> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| RunStepContext {
                step_id: IDS[index],
                context: node.context(),
            })
            .collect()
    }

    fn workspace(&self) -> PathBuf {
        self.nodes[0].kit.root.path().join("graph state café")
    }

    fn create(&self) -> RunStore {
        RunStore::create(&self.workspace(), self.plan.clone()).unwrap()
    }

    fn execute(&self, store: &mut RunStore, index: usize, events: &mut Vec<ExtensionEvent>) {
        store
            .execute_in_plan(
                &self.plan,
                IDS[index],
                &self.contexts(),
                &NoSecrets,
                &NeverCancelled,
                events,
            )
            .unwrap();
    }

    fn assess(&self, store: &RunStore) -> RunAssessment {
        let report = store.assess_run(&self.plan, &self.contexts()).unwrap();
        report.validate_against(store.state()).unwrap();
        report
    }

    fn complete(&self, store: &mut RunStore) {
        for index in 0..5 {
            self.execute(store, index, &mut Vec::new());
        }
    }
}

fn eligibility(report: &RunAssessment) -> Vec<ResumeEligibility> {
    report.steps.iter().map(|step| step.eligibility).collect()
}

fn history(graph: &Graph) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files: Vec<_> = fs::read_dir(graph.workspace())
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_owned();
            // Never read a byte-range locked file on Windows.
            let bytes = if name == "workspace.lock" {
                Vec::new()
            } else {
                fs::read(path).unwrap()
            };
            (PathBuf::from(name), bytes)
        })
        .collect();
    files.sort();
    files
}

#[test]
fn graph_reopen_preserves_success_and_never_repeats_a_completed_launch() {
    use ResumeEligibility::{DependencyBlocked, Ready, Reusable};
    let mut deterministic_reports = Vec::new();
    for _ in 0..2 {
        let graph = Graph::new();
        let mut store = graph.create();
        assert_eq!(
            eligibility(&graph.assess(&store)),
            [
                Ready,
                DependencyBlocked,
                DependencyBlocked,
                Ready,
                DependencyBlocked
            ]
        );
        let mut events = Vec::new();
        graph.execute(&mut store, 0, &mut events);
        graph.execute(&mut store, 3, &mut events);
        drop(store);
        let mut store = RunStore::open(&graph.workspace()).unwrap();
        let partial = graph.assess(&store);
        assert_eq!(
            eligibility(&partial),
            [
                Reusable,
                Ready,
                DependencyBlocked,
                Reusable,
                DependencyBlocked
            ]
        );
        deterministic_reports.push(serde_json::to_vec(&partial).unwrap());
        for index in [1, 2, 4] {
            graph.execute(&mut store, index, &mut events);
        }
        assert_eq!(
            events
                .iter()
                .filter(|event| event.kind == EventKind::PhaseStarted)
                .count(),
            5
        );
        drop(store);
        let mut store = RunStore::open(&graph.workspace()).unwrap();
        assert_eq!(eligibility(&graph.assess(&store)), [Reusable; 5]);
        let before = history(&graph);
        let mut repeated = Vec::new();
        for id in IDS {
            assert!(matches!(
                store.execute_in_plan(
                    &graph.plan,
                    id,
                    &graph.contexts(),
                    &NoSecrets,
                    &NeverCancelled,
                    &mut repeated
                ),
                Err(DurableExecutionError::State(StateError::Ineligible {
                    eligibility: Reusable
                }))
            ));
        }
        assert!(repeated.is_empty());
        assert_eq!(history(&graph), before);
        assert!(store.state().steps.iter().all(|step| step.attempt == 1));
    }
    assert_eq!(deterministic_reports[0], deterministic_reports[1]);
}

#[test]
fn graph_stale_evidence_invalidates_only_affected_descendants_and_preserves_history() {
    use ResumeEligibility::{Invalidated, Reusable};
    for boundary in [
        StateBoundary::Inputs,
        StateBoundary::Artifacts,
        StateBoundary::Provider,
        StateBoundary::Configuration,
        StateBoundary::Authority,
        StateBoundary::Bindings,
        StateBoundary::Invocation,
        StateBoundary::Capability,
    ] {
        let mut graph = Graph::new();
        let mut store = graph.create();
        graph.complete(&mut store);
        drop(store);
        let store = RunStore::open(&graph.workspace()).unwrap();
        let before = history(&graph);
        let node = &mut graph.nodes[0];
        match boundary {
            StateBoundary::Inputs => fs::write(
                node.artifacts.join(INPUT_LOCATOR),
                b"changed private source",
            )
            .unwrap(),
            StateBoundary::Artifacts => {
                fs::write(node.kit.output_path(), b"damaged output").unwrap();
            }
            StateBoundary::Provider => fs::write(
                node.kit.root.path().join(PACKAGE_LOCATOR).join("LICENSE"),
                b"changed package",
            )
            .unwrap(),
            StateBoundary::Configuration => {
                node.prepared
                    .invocation
                    .configuration
                    .values
                    .insert("private".to_owned(), "configuration-canary".into());
            }
            StateBoundary::Authority => {
                node.prepared.invocation.authorization.grants_digest = "9".repeat(64);
            }
            StateBoundary::Bindings => {
                node.kit.bindings.outputs[0].locator = "outputs/changed.json".to_owned();
            }
            StateBoundary::Invocation => {
                node.prepared.invocation.cancellation_id = "cancel:changed".to_owned();
            }
            StateBoundary::Capability => {
                node.prepared.invocation.capability_id = "flow/changed".to_owned();
            }
            _ => unreachable!(),
        }
        let report = graph.assess(&store);
        assert_eq!(
            eligibility(&report),
            [Invalidated, Invalidated, Invalidated, Reusable, Invalidated],
            "{boundary:?}"
        );
        assert_eq!(report.steps[0].stale_boundary, Some(boundary));
        assert_eq!(report.steps[1].blocked_by, ["step:a"]);
        assert_eq!(report.steps[2].blocked_by, ["step:b"]);
        assert_eq!(report.steps[4].blocked_by, ["step:b"]);
        assert!(
            report
                .steps
                .iter()
                .all(|step| step.recorded_status == RunStepStatus::Succeeded)
        );
        assert_eq!(history(&graph), before);
        let serialized = serde_json::to_string(&report).unwrap();
        for private in [
            "configuration-canary",
            "changed private source",
            graph.workspace().to_str().unwrap(),
        ] {
            assert!(!serialized.contains(private));
        }
    }
}

#[test]
fn graph_execution_rechecks_ancestors_and_refuses_single_step_bypass() {
    use ResumeEligibility::{Invalidated, Ready};
    let graph = Graph::new();
    let mut store = graph.create();
    graph.execute(&mut store, 0, &mut Vec::new());
    let mut stale_report = graph.assess(&store);
    assert_eq!(stale_report.steps[1].eligibility, Ready);
    fs::write(
        graph.nodes[0].artifacts.join(INPUT_LOCATOR),
        b"changed after assessment",
    )
    .unwrap();
    // A structurally valid forged claim remains mere data; execution has no report argument.
    stale_report.steps[0].eligibility = ResumeEligibility::Reusable;
    stale_report.validate_against(store.state()).unwrap();
    let before = history(&graph);
    let mut events = Vec::new();
    assert!(matches!(
        store.execute_in_plan(
            &graph.plan,
            IDS[1],
            &graph.contexts(),
            &NoSecrets,
            &NeverCancelled,
            &mut events
        ),
        Err(DurableExecutionError::State(StateError::Ineligible {
            eligibility: Invalidated
        }))
    ));
    assert!(matches!(
        store.assess(&graph.plan, IDS[1], &graph.nodes[1].context()),
        Err(StateError::DependencyEvidenceRequired)
    ));
    assert!(matches!(
        store.execute(
            IDS[1],
            &graph.nodes[1].context(),
            &NoSecrets,
            &NeverCancelled,
            &mut events
        ),
        Err(DurableExecutionError::State(
            StateError::DependencyEvidenceRequired
        ))
    ));
    assert!(events.is_empty());
    assert!(!graph.nodes[1].kit.output_path().exists());
    assert_eq!(history(&graph), before);
    // Unrelated ready work still runs; the stale branch does not poison the whole plan.
    graph.execute(&mut store, 3, &mut events);
}

#[test]
fn graph_inventory_and_plan_identity_are_required_before_any_launch() {
    let graph = Graph::new();
    let mut store = graph.create();
    let before = history(&graph);
    let mut events = Vec::new();
    for mutation in 0..4 {
        let mut contexts = graph.contexts();
        match mutation {
            0 => {
                contexts.pop();
            }
            1 => contexts.swap(0, 1),
            2 => contexts[1].step_id = IDS[0],
            3 => contexts.push(RunStepContext {
                step_id: "step:unknown",
                context: graph.nodes[0].context(),
            }),
            _ => unreachable!(),
        }
        assert!(matches!(
            store.execute_in_plan(
                &graph.plan,
                IDS[0],
                &contexts,
                &NoSecrets,
                &NeverCancelled,
                &mut events
            ),
            Err(DurableExecutionError::State(StateError::ContextInventory))
        ));
    }
    let mut changed = graph.plan.clone();
    changed.plan_id = "plan:changed".to_owned();
    assert!(matches!(
        store.execute_in_plan(
            &changed,
            IDS[0],
            &graph.contexts(),
            &NoSecrets,
            &NeverCancelled,
            &mut events
        ),
        Err(DurableExecutionError::State(StateError::Stale {
            boundary: StateBoundary::Plan
        }))
    ));
    assert!(events.is_empty());
    assert_eq!(history(&graph), before);
}

#[test]
fn graph_unresolved_dependencies_block_without_observing_future_inputs() {
    use ResumeEligibility::{Abandoned, ApprovalRequired, DependencyBlocked, Ready};
    let graph = Graph::new();
    let mut store = graph.create();
    fs::remove_file(graph.nodes[1].artifacts.join(INPUT_LOCATOR)).unwrap();
    store.deny_pending(IDS[0]).unwrap();
    assert_eq!(
        eligibility(&graph.assess(&store)),
        [
            ApprovalRequired,
            DependencyBlocked,
            DependencyBlocked,
            Ready,
            DependencyBlocked
        ]
    );
    let before = history(&graph);
    let mut events = Vec::new();
    assert!(matches!(
        store.execute_in_plan(
            &graph.plan,
            IDS[1],
            &graph.contexts(),
            &NoSecrets,
            &NeverCancelled,
            &mut events
        ),
        Err(DurableExecutionError::State(StateError::Ineligible {
            eligibility: DependencyBlocked
        }))
    ));
    assert!(events.is_empty());
    assert_eq!(history(&graph), before);
    store
        .decide_recovery(
            IDS[0],
            &graph.nodes[0].context(),
            RecoveryApproval {
                decision_id: "decision:abandon-root".to_owned(),
                action: RunRecoveryAction::Abandon,
                acknowledge_uncertain_effects: true,
            },
        )
        .unwrap();
    drop(store);
    let store = RunStore::open(&graph.workspace()).unwrap();
    assert_eq!(
        eligibility(&graph.assess(&store)),
        [
            Abandoned,
            DependencyBlocked,
            DependencyBlocked,
            Ready,
            DependencyBlocked
        ]
    );
    assert_eq!(
        graph.assess(&store).steps[4].blocked_by,
        ["step:b", "step:d"]
    );
}

#[test]
fn graph_assessment_rejects_history_corruption_even_on_an_open_handle() {
    let graph = Graph::new();
    let store = graph.create();
    fs::write(
        graph.workspace().join("00000000000000000000.json"),
        b"corrupt",
    )
    .unwrap();
    assert!(matches!(
        store.assess_run(&graph.plan, &graph.contexts()),
        Err(StateError::Malformed)
    ));
    drop(store);
    assert!(RunStore::open(&graph.workspace()).is_err());
}

#[test]
fn graph_changed_validator_blocks_descendants_despite_recorded_success() {
    let graph = Graph::new();
    let mut store = graph.create();
    graph.execute(&mut store, 0, &mut Vec::new());
    drop(store);
    let path = graph.workspace().join("00000000000000000002.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["state"]["steps"][0]["checkpoint"]["validation"]["implementation_digest"] =
        "0".repeat(64).into();
    value["state_digest"] = super::digest_json(&value["state"]).into();
    fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    let store = RunStore::open(&graph.workspace()).unwrap();
    let report = graph.assess(&store);
    assert_eq!(
        report.steps[0].stale_boundary,
        Some(StateBoundary::Validation)
    );
    assert_eq!(report.steps[0].recorded_status, RunStepStatus::Succeeded);
    assert_eq!(report.steps[2].eligibility, ResumeEligibility::Invalidated);
    assert_eq!(report.steps[3].eligibility, ResumeEligibility::Ready);
}

#[test]
fn graph_report_contract_refuses_unknown_or_contradictory_evidence() {
    let state: RunState = serde_json::from_str(include_str!(
        "../../contracts/examples/run-state.v1.example.json"
    ))
    .unwrap();
    let report: RunAssessment = serde_json::from_str(include_str!(
        "../../contracts/examples/run-assessment.v1.example.json"
    ))
    .unwrap();
    report.validate_against(&state).unwrap();
    for mutation in 0..7 {
        let mut value = report.clone();
        match mutation {
            0 => value.schema_version = "flow.run-assessment/v2".to_owned(),
            1 => value.state_digest = "0".repeat(64),
            2 => value.steps.clear(),
            3 => value.steps[0].eligibility = ResumeEligibility::Reusable,
            4 => value.steps[0].blocked_by.push("step:unknown".to_owned()),
            5 => value.steps[0].stale_boundary = Some(StateBoundary::Plan),
            6 => value.sequence += 1,
            _ => unreachable!(),
        }
        assert!(
            value.validate_against(&state).is_err(),
            "mutation {mutation}"
        );
    }
    let mut value = serde_json::to_value(report).unwrap();
    value["steps"][0]["authorization_token"] = "private".into();
    assert!(serde_json::from_value::<RunAssessment>(value).is_err());
}
