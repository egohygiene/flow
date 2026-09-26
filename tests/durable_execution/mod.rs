use super::{
    CAPABILITIES, INPUT_LOCATOR, KitFixture, NoSecrets, PreparedLifecycleRun, WORKSPACE_LOCATOR,
    provider_binary_snapshot,
};
use flow::{
    DurableExecutionError, EventSinkError, ExtensionEvent, NeverCancelled, PlannedStep,
    ProcessStepContext, RUN_PLAN_V1, RecoveryApproval, ResumeEligibility, RunFailure,
    RunInspectionStatus, RunPlan, RunRecoveryAction, RunState, RunStepStatus, RunStore,
    StateBoundary, StateError,
};
use std::fs;
use std::path::PathBuf;

struct Harness {
    kit: KitFixture,
    prepared: PreparedLifecycleRun,
    artifacts: PathBuf,
}

impl Harness {
    fn new(mode: &str) -> Self {
        let (bytes, name) = provider_binary_snapshot();
        let kit = KitFixture::new(CAPABILITIES[0], &bytes, &name);
        let prepared = kit.prepare_lifecycle(CAPABILITIES[0], mode, mode == "await-interruption");
        let artifacts = kit.root.path().join(WORKSPACE_LOCATOR);
        Self {
            kit,
            prepared,
            artifacts,
        }
    }

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

    fn plan(&self) -> RunPlan {
        RunPlan {
            schema_version: RUN_PLAN_V1.to_owned(),
            plan_id: "plan:durable-kit".to_owned(),
            run_id: self.prepared.invocation.run_id.clone(),
            steps: vec![
                PlannedStep::prepare("step:inspect".to_owned(), Vec::new(), &self.context())
                    .unwrap(),
            ],
        }
    }

    fn workspace(&self) -> PathBuf {
        self.kit.root.path().join("durable state")
    }
}

fn snapshots(path: &std::path::Path) -> Vec<Vec<u8>> {
    let mut paths: Vec<_> = fs::read_dir(path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    paths.sort();
    paths.into_iter().map(|p| fs::read(p).unwrap()).collect()
}

#[test]
fn durable_success_records_intent_before_events_and_reopens_without_reexecution() {
    let fixture = Harness::new("success");
    let plan = fixture.plan();
    let mut store = RunStore::create(&fixture.workspace(), plan.clone()).unwrap();
    let source = fs::read(fixture.artifacts.join(INPUT_LOCATOR)).unwrap();
    let mut event_count = 0;
    let mut sink = |_: &ExtensionEvent| {
        event_count += 1;
        let snapshot: serde_json::Value = serde_json::from_slice(
            &fs::read(fixture.workspace().join("00000000000000000001.json")).unwrap(),
        )
        .unwrap();
        let running: RunState = serde_json::from_value(snapshot["state"].clone()).unwrap();
        assert_eq!(running.steps[0].status, RunStepStatus::Running);
        assert!(running.steps[0].checkpoint.is_none());
        assert!(running.authority_decisions[0].granted);
        Ok::<(), EventSinkError>(())
    };
    let accepted = store
        .execute(
            "step:inspect",
            &fixture.context(),
            &NoSecrets,
            &NeverCancelled,
            &mut sink,
        )
        .unwrap();
    assert!(event_count > 0);
    assert_eq!(
        store.state().inspection_status(),
        RunInspectionStatus::Succeeded
    );
    assert_eq!(store.state().sequence, 2);
    assert_eq!(
        store.state().steps[0]
            .checkpoint
            .as_ref()
            .unwrap()
            .artifacts
            .len(),
        accepted.inputs().len() + accepted.outputs().len()
    );
    let recorded = store.state().clone();
    drop(store);
    let mut reopened = RunStore::open(&fixture.workspace()).unwrap();
    assert_eq!(reopened.state(), &recorded);
    assert_eq!(
        reopened
            .assess(&plan, "step:inspect", &fixture.context())
            .unwrap(),
        ResumeEligibility::Reusable
    );
    let before = snapshots(&fixture.workspace());
    let mut events = Vec::new();
    assert!(matches!(
        reopened.execute(
            "step:inspect",
            &fixture.context(),
            &NoSecrets,
            &NeverCancelled,
            &mut events
        ),
        Err(DurableExecutionError::State(StateError::Transition))
    ));
    assert!(events.is_empty());
    assert_eq!(snapshots(&fixture.workspace()), before);
    assert_eq!(
        fs::read(fixture.artifacts.join(INPUT_LOCATOR)).unwrap(),
        source
    );
}

#[test]
fn durable_resume_refuses_changed_plan_configuration_provider_inputs_and_artifacts() {
    let fixture = Harness::new("success");
    let plan = fixture.plan();
    let mut store = RunStore::create(&fixture.workspace(), plan.clone()).unwrap();
    store
        .execute(
            "step:inspect",
            &fixture.context(),
            &NoSecrets,
            &NeverCancelled,
            &mut Vec::new(),
        )
        .unwrap();
    drop(store);
    let store = RunStore::open(&fixture.workspace()).unwrap();
    let mut changed_plan = plan.clone();
    changed_plan.plan_id = "plan:changed".to_owned();
    assert!(matches!(
        store.assess(&changed_plan, "step:inspect", &fixture.context()),
        Err(StateError::Stale {
            boundary: StateBoundary::Plan
        })
    ));
    for (field, boundary) in [
        ("configuration", StateBoundary::Configuration),
        ("provider", StateBoundary::Provider),
        ("capability", StateBoundary::Capability),
        ("authority", StateBoundary::Authority),
        ("invocation", StateBoundary::Invocation),
    ] {
        let mut invocation = fixture.prepared.invocation.clone();
        match field {
            "configuration" => {
                invocation
                    .configuration
                    .values
                    .insert("unhashed-value".to_owned(), serde_json::json!("changed"));
            }
            "provider" => "0.2.0".clone_into(&mut invocation.extension.version),
            "capability" => "flow/other-fixture".clone_into(&mut invocation.capability_id),
            "authority" => invocation.authorization.grants_digest = "a".repeat(64),
            "invocation" => "invocation:changed".clone_into(&mut invocation.invocation_id),
            _ => unreachable!(),
        }
        let context = ProcessStepContext {
            invocation: &invocation,
            ..fixture.context()
        };
        assert!(
            matches!(store.assess(&plan, "step:inspect", &context), Err(StateError::Stale { boundary: observed }) if observed == boundary),
            "{field}"
        );
    }
    let input = fixture.artifacts.join(INPUT_LOCATOR);
    let original = fs::read(&input).unwrap();
    fs::write(&input, b"changed input").unwrap();
    assert!(matches!(
        store.assess(&plan, "step:inspect", &fixture.context()),
        Err(StateError::Stale {
            boundary: StateBoundary::Inputs
        })
    ));
    fs::write(&input, original).unwrap();
    fs::write(fixture.kit.output_path(), b"changed output").unwrap();
    assert!(matches!(
        store.assess(&plan, "step:inspect", &fixture.context()),
        Err(StateError::Stale {
            boundary: StateBoundary::Artifacts
        })
    ));
    fs::remove_file(fixture.kit.output_path()).unwrap();
    assert!(matches!(
        store.assess(&plan, "step:inspect", &fixture.context()),
        Err(StateError::Stale {
            boundary: StateBoundary::Artifacts
        })
    ));
}

#[test]
fn durable_input_drift_and_prestart_cancellation_prevent_launch() {
    let fixture = Harness::new("success");
    let mut store = RunStore::create(&fixture.workspace(), fixture.plan()).unwrap();
    fs::write(
        fixture.artifacts.join(INPUT_LOCATOR),
        b"changed before launch",
    )
    .unwrap();
    assert!(matches!(
        store.execute(
            "step:inspect",
            &fixture.context(),
            &NoSecrets,
            &NeverCancelled,
            &mut Vec::new()
        ),
        Err(DurableExecutionError::State(StateError::Stale {
            boundary: StateBoundary::Inputs
        }))
    ));
    assert_eq!(store.state().sequence, 0);
    assert!(!fixture.kit.output_path().exists());

    let cancelled = Harness::new("success");
    let mut store = RunStore::create(&cancelled.workspace(), cancelled.plan()).unwrap();
    assert!(matches!(
        store.execute(
            "step:inspect",
            &cancelled.context(),
            &NoSecrets,
            &|| true,
            &mut Vec::new()
        ),
        Err(DurableExecutionError::CancelledBeforeLaunch)
    ));
    assert!(!cancelled.kit.output_path().exists());
    assert_eq!(store.state().steps[0].attempt, 0);
    assert!(store.state().authority_decisions.is_empty());
}

#[test]
fn durable_cancellation_observed_after_intent_commit_still_prevents_launch() {
    use std::cell::Cell;
    let fixture = Harness::new("success");
    let mut store = RunStore::create(&fixture.workspace(), fixture.plan()).unwrap();
    let checks = Cell::new(0);
    let cancellation = || {
        checks.set(checks.get() + 1);
        checks.get() >= 2
    };
    assert!(matches!(
        store.execute(
            "step:inspect",
            &fixture.context(),
            &NoSecrets,
            &cancellation,
            &mut Vec::new()
        ),
        Err(DurableExecutionError::CancelledBeforeLaunch)
    ));
    assert!(!fixture.kit.output_path().exists());
    drop(store);
    let store = RunStore::open(&fixture.workspace()).unwrap();
    assert_eq!(store.state().steps[0].attempt, 1);
    assert_eq!(store.state().steps[0].status, RunStepStatus::Cancelled);
}

#[test]
fn durable_changed_validator_identity_is_inspectable_but_not_reusable() {
    let fixture = Harness::new("success");
    let plan = fixture.plan();
    let mut store = RunStore::create(&fixture.workspace(), plan.clone()).unwrap();
    store
        .execute(
            "step:inspect",
            &fixture.context(),
            &NoSecrets,
            &NeverCancelled,
            &mut Vec::new(),
        )
        .unwrap();
    drop(store);
    let snapshot = fixture.workspace().join("00000000000000000002.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&snapshot).unwrap()).unwrap();
    value["state"]["steps"][0]["checkpoint"]["validation"]["implementation_digest"] =
        "0".repeat(64).into();
    value["state_digest"] = super::digest_json(&value["state"]).into();
    fs::write(snapshot, serde_json::to_vec(&value).unwrap()).unwrap();
    let store = RunStore::open(&fixture.workspace()).unwrap();
    assert_eq!(store.state().steps[0].status, RunStepStatus::Succeeded);
    assert!(matches!(
        store.assess(&plan, "step:inspect", &fixture.context()),
        Err(StateError::Stale {
            boundary: StateBoundary::Validation
        })
    ));
}

#[test]
fn durable_dependency_status_blocks_launch_and_abandonment_survives_reopen() {
    let fixture = Harness::new("success");
    let mut plan = fixture.plan();
    let mut predecessor = plan.steps[0].clone();
    predecessor.step_id = "step:predecessor".to_owned();
    predecessor.context.invocation_id = "invocation:predecessor".to_owned();
    predecessor.context.invocation_digest = "9".repeat(64);
    predecessor.context.bindings.outputs[0].artifact_id = "artifact:predecessor".to_owned();
    plan.steps[0].depends_on = vec![predecessor.step_id.clone()];
    plan.steps.insert(0, predecessor);
    let mut store = RunStore::create(&fixture.workspace(), plan.clone()).unwrap();
    assert!(matches!(
        store.assess(&plan, "step:inspect", &fixture.context()),
        Err(StateError::DependencyEvidenceRequired)
    ));
    assert!(
        store
            .execute(
                "step:inspect",
                &fixture.context(),
                &NoSecrets,
                &NeverCancelled,
                &mut Vec::new()
            )
            .is_err()
    );
    assert!(!fixture.kit.output_path().exists());
    store.cancel_pending("step:inspect").unwrap();
    store
        .decide_recovery(
            "step:inspect",
            &fixture.context(),
            RecoveryApproval {
                decision_id: "decision:abandon".to_owned(),
                action: RunRecoveryAction::Abandon,
                acknowledge_uncertain_effects: true,
            },
        )
        .unwrap();
    drop(store);
    let store = RunStore::open(&fixture.workspace()).unwrap();
    assert_eq!(store.state().steps[1].status, RunStepStatus::Abandoned);
    assert_eq!(
        store.state().recovery_decisions[0].action,
        RunRecoveryAction::Abandon
    );
}

#[test]
fn durable_provider_and_artifact_failures_survive_restart_without_checkpoints() {
    for (mode, expected) in [
        ("nonzero-after-success", RunFailure::Process),
        ("missing-output", RunFailure::ArtifactObservation),
        (
            "contradictory-artifact-evidence",
            RunFailure::ArtifactAcceptance,
        ),
    ] {
        let fixture = Harness::new(mode);
        let mut store = RunStore::create(&fixture.workspace(), fixture.plan()).unwrap();
        assert!(
            store
                .execute(
                    "step:inspect",
                    &fixture.context(),
                    &NoSecrets,
                    &NeverCancelled,
                    &mut Vec::new()
                )
                .is_err()
        );
        drop(store);
        let store = RunStore::open(&fixture.workspace()).unwrap();
        assert_eq!(
            store.state().steps[0].status,
            RunStepStatus::Failed,
            "{mode}"
        );
        assert_eq!(store.state().steps[0].failure, Some(expected), "{mode}");
        assert!(store.state().steps[0].checkpoint.is_none());
        assert_eq!(
            store
                .assess(&fixture.plan(), "step:inspect", &fixture.context())
                .unwrap(),
            ResumeEligibility::ApprovalRequired
        );
    }
}

#[test]
fn durable_interruption_leaves_uncertain_intent_and_requires_explicit_retry() {
    let fixture = Harness::new("success");
    let plan = fixture.plan();
    let mut store = RunStore::create(&fixture.workspace(), plan.clone()).unwrap();
    let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        store.execute(
            "step:inspect",
            &fixture.context(),
            &NoSecrets,
            &NeverCancelled,
            &mut |_: &ExtensionEvent| -> Result<(), EventSinkError> {
                panic!("simulate host interruption after provider completion");
            },
        )
    }));
    assert!(interrupted.is_err());
    drop(store);
    let mut store = RunStore::open(&fixture.workspace()).unwrap();
    assert!(fixture.kit.output_path().exists());
    assert_eq!(
        store.state().inspection_status(),
        RunInspectionStatus::RecoveryRequired
    );
    assert_eq!(
        store
            .assess(&plan, "step:inspect", &fixture.context())
            .unwrap(),
        ResumeEligibility::ApprovalRequired
    );
    assert!(
        store
            .execute(
                "step:inspect",
                &fixture.context(),
                &NoSecrets,
                &NeverCancelled,
                &mut Vec::new()
            )
            .is_err()
    );
    let before = snapshots(&fixture.workspace());
    assert!(
        store
            .decide_recovery(
                "step:inspect",
                &fixture.context(),
                RecoveryApproval {
                    decision_id: "decision:missing-ack".to_owned(),
                    action: RunRecoveryAction::Retry,
                    acknowledge_uncertain_effects: false,
                }
            )
            .is_err()
    );
    assert_eq!(snapshots(&fixture.workspace()), before);
    store
        .decide_recovery(
            "step:inspect",
            &fixture.context(),
            RecoveryApproval {
                decision_id: "decision:explicit-retry".to_owned(),
                action: RunRecoveryAction::Retry,
                acknowledge_uncertain_effects: true,
            },
        )
        .unwrap();
    assert_eq!(
        store
            .assess(&plan, "step:inspect", &fixture.context())
            .unwrap(),
        ResumeEligibility::Ready
    );
    assert_eq!(store.state().steps[0].attempt, 1);
    // Explicit operator cleanup preserves the uncertain output before retry;
    // the store itself never overwrites or deletes provider artifacts.
    fs::rename(
        fixture.kit.output_path(),
        fixture.kit.root.path().join("quarantined-output.json"),
    )
    .unwrap();
    drop(store);
    let mut store = RunStore::open(&fixture.workspace()).unwrap();
    store
        .execute(
            "step:inspect",
            &fixture.context(),
            &NoSecrets,
            &NeverCancelled,
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(store.state().steps[0].attempt, 2);
    assert_eq!(store.state().recovery_decisions.len(), 1);
    assert_eq!(store.state().authority_decisions.len(), 2);
}

#[test]
fn durable_terminal_commit_failure_never_returns_accepted_artifacts() {
    let fixture = Harness::new("success");
    let mut store = RunStore::create(&fixture.workspace(), fixture.plan()).unwrap();
    let mut sink = |_: &ExtensionEvent| {
        fs::write(
            fixture.workspace().join("snapshot.pending"),
            b"simulated incomplete write",
        )
        .unwrap();
        Ok::<(), EventSinkError>(())
    };
    assert!(matches!(
        store.execute(
            "step:inspect",
            &fixture.context(),
            &NoSecrets,
            &NeverCancelled,
            &mut sink
        ),
        Err(DurableExecutionError::State(StateError::IncompleteWrite))
    ));
    assert_eq!(store.state().steps[0].status, RunStepStatus::Running);
    assert!(store.state().steps[0].checkpoint.is_none());
}

#[cfg(unix)]
#[test]
fn durable_cancellation_and_timeout_are_distinct_persisted_failures() {
    for cancel in [true, false] {
        let fixture = Harness::new("await-interruption");
        let mut store = RunStore::create(&fixture.workspace(), fixture.plan()).unwrap();
        let signal = || cancel && fixture.kit.lifecycle_control_path().is_file();
        assert!(
            store
                .execute(
                    "step:inspect",
                    &fixture.context(),
                    &NoSecrets,
                    &signal,
                    &mut Vec::new()
                )
                .is_err()
        );
        drop(store);
        let store = RunStore::open(&fixture.workspace()).unwrap();
        assert_eq!(
            store.state().steps[0].failure,
            Some(if cancel {
                RunFailure::Cancelled
            } else {
                RunFailure::TimedOut
            })
        );
        assert!(store.state().steps[0].checkpoint.is_none());
        super::assert_recorded_process_reaped(&fixture.kit.lifecycle_control_path());
    }
}

#[test]
fn durable_store_omits_configuration_values_and_provider_diagnostics() {
    let mut fixture = Harness::new("success");
    fixture.prepared.invocation.configuration.values.insert(
        "seed".to_owned(),
        serde_json::json!("FLOW_STATE_PRIVATE_CANARY_4951"),
    );
    fixture.prepared.invocation.configuration.digest =
        super::digest_json(&fixture.prepared.invocation.configuration.values);
    let plan = fixture.plan();
    let mut store = RunStore::create(&fixture.workspace(), plan).unwrap();
    store
        .execute(
            "step:inspect",
            &fixture.context(),
            &NoSecrets,
            &NeverCancelled,
            &mut Vec::new(),
        )
        .unwrap();
    let encoded = snapshots(&fixture.workspace()).concat();
    let text = String::from_utf8(encoded).unwrap();
    assert!(!text.contains("FLOW_STATE_PRIVATE_CANARY_4951"));
    assert!(!text.contains(&fixture.kit.root.path().to_string_lossy().to_string()));
}
