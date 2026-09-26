use super::fixture::IDS;
use super::*;
use flow::{
    DurableExecutionError, EventSinkError, ExecutionError, ExtensionEvent, ProcessRunnerError,
    ProcessStream, RecoveryApproval, ResumeEligibility, StateBoundary, StateError,
};
use std::cell::Cell;

fn approval(action: RunRecoveryAction, acknowledge: bool) -> RecoveryApproval {
    RecoveryApproval {
        decision_id: "decision:lifecycle-recovery".to_owned(),
        action,
        acknowledge_uncertain_effects: acknowledge,
    }
}

fn execute_error(fixture: &Fixture, store: &mut RunStore, recipe: &str) -> &'static str {
    let checks = Cell::new(0);
    let control = fixture.nodes[0].kit.lifecycle_control_path();
    let cancel = || match recipe {
        "cancel-before-launch" | "retry-cancelled" | "retry-without-ack" => true,
        "cancel-after-intent" => {
            checks.set(checks.get() + 1);
            checks.get() >= 2
        }
        "cancel-during-provider" => control.is_file(),
        _ => false,
    };
    let error = store
        .execute_in_plan(
            &fixture.plan,
            IDS[0],
            &fixture.contexts(),
            &NoSecrets,
            &cancel,
            &mut Vec::new(),
        )
        .unwrap_err();
    let code = match error {
        DurableExecutionError::CancelledBeforeLaunch => "cancelled-before-launch",
        DurableExecutionError::Process(ProcessRunnerError::Cancelled { .. }) => "cancelled",
        DurableExecutionError::Process(ProcessRunnerError::TimedOut {
            timeout_ms: 5_000, ..
        }) => "timed-out",
        DurableExecutionError::Process(ProcessRunnerError::Validation {
            source: ExecutionError::ProcessExit { code: Some(7) },
        }) => "process-exit-7",
        DurableExecutionError::Process(ProcessRunnerError::Validation {
            source: ExecutionError::ProcessExit { code: None },
        }) => "process-signal",
        DurableExecutionError::Process(ProcessRunnerError::Validation {
            source:
                ExecutionError::ProcessOutputLimit {
                    stream,
                    limit: 65_536,
                    observed: 65_537,
                },
        }) => match stream {
            ProcessStream::Stdout => "stdout-limit",
            ProcessStream::Stderr => "stderr-limit",
        },
        DurableExecutionError::Acceptance(flow::ArtifactAcceptanceError::Mismatch {
            ref message,
        }) if message == "artifact acceptance requires a complete produced or reused result" => {
            "partial-output-rejected"
        }
        other => panic!("unexpected typed lifecycle failure: {other:?}"),
    };
    if matches!(recipe, "cancel-during-provider" | "timeout") {
        assert!(control.is_file());
        crate::assert_recorded_process_reaped(&control);
    }
    code
}

pub(super) fn run(fixture: &mut Fixture, recipe: &str) -> Outcome {
    let mut store = RunStore::create(&fixture.workspace(), fixture.plan.clone()).unwrap();
    let initial = fixture.assess(&store, "initial");
    if recipe.starts_with("deny-")
        || recipe.starts_with("effect-")
        || recipe.starts_with("cleanup-")
    {
        return super::safety::run(fixture, store, initial, recipe);
    }
    match recipe {
        "retryable-provider-failure" | "terminal-provider-failure" => {
            classified_failure(fixture, store, initial, recipe)
        }
        "completed-restart"
        | "partial-restart"
        | "host-exit-after-intent"
        | "host-exit-after-provider"
        | "retry-interrupted" => restart(fixture, store, initial, recipe),
        value
            if value.starts_with("changed-")
                || value.starts_with("corrupt-")
                || matches!(
                    value,
                    "incomplete-write" | "future-schema" | "malformed-state"
                ) =>
        {
            drift(fixture, store, initial, recipe)
        }
        "cancel-between-steps" => {
            fixture.execute(&mut store, 0);
            let completed = fixture.assess(&store, "first-complete");
            store.cancel_pending(IDS[1]).unwrap();
            finish(
                fixture,
                store,
                vec![initial, completed],
                "cancelled-between-steps",
            )
        }
        "retry-completed" => {
            fixture.execute(&mut store, 0);
            let before = history(&fixture.workspace());
            assert!(matches!(
                store.decide_recovery(
                    IDS[0],
                    &fixture.nodes[0].context(),
                    approval(RunRecoveryAction::Retry, true)
                ),
                Err(StateError::Transition)
            ));
            assert!(matches!(
                store.execute_in_plan(
                    &fixture.plan,
                    IDS[0],
                    &fixture.contexts(),
                    &NoSecrets,
                    &NeverCancelled,
                    &mut Vec::new()
                ),
                Err(DurableExecutionError::State(StateError::Ineligible {
                    eligibility: ResumeEligibility::Reusable
                }))
            ));
            assert_eq!(history(&fixture.workspace()), before);
            finish(fixture, store, vec![initial], "completed-retry-refused")
        }
        "cancel-before-launch"
        | "cancel-after-intent"
        | "cancel-during-provider"
        | "timeout"
        | "nonzero-exit"
        | "signal-termination"
        | "stdout-limit"
        | "stderr-limit"
        | "partial-output"
        | "retry-cancelled"
        | "retry-without-ack"
        | "abandon-terminal" => {
            let code = execute_error(fixture, &mut store, recipe);
            recover_failure(fixture, store, vec![initial], recipe, code)
        }
        other => panic!("unknown lifecycle recipe: {other}"),
    }
}

pub(super) fn finish(
    fixture: &Fixture,
    store: RunStore,
    mut assessments: Vec<Assessment>,
    code: &str,
) -> Outcome {
    let before = history(&fixture.workspace());
    drop(store);
    let reopened = RunStore::open(&fixture.workspace()).unwrap();
    assessments.push(fixture.assess(&reopened, "final"));
    assert_eq!(
        history(&fixture.workspace()),
        before,
        "assessment must not rewrite history"
    );
    Outcome {
        code: code.to_owned(),
        history: before,
        assessments,
        safety: None,
    }
}

fn recover_failure(
    fixture: &Fixture,
    mut store: RunStore,
    mut assessments: Vec<Assessment>,
    recipe: &str,
    code: &str,
) -> Outcome {
    match recipe {
        "retry-cancelled" => {
            assessments.push(fixture.assess(&store, "recovery-required"));
            store
                .decide_recovery(
                    IDS[0],
                    &fixture.nodes[0].context(),
                    approval(RunRecoveryAction::Retry, true),
                )
                .unwrap();
            assessments.push(fixture.assess(&store, "retry-approved"));
            assert!(!fixture.nodes[0].kit.output_path().exists());
            fixture.execute(&mut store, 0);
            finish(fixture, store, assessments, "explicit-retry-succeeded")
        }
        "abandon-terminal" => {
            assessments.push(fixture.assess(&store, "recovery-required"));
            let candidate = fs::read(fixture.nodes[0].kit.output_path()).unwrap();
            store
                .decide_recovery(
                    IDS[0],
                    &fixture.nodes[0].context(),
                    approval(RunRecoveryAction::Abandon, true),
                )
                .unwrap();
            assert_eq!(
                fs::read(fixture.nodes[0].kit.output_path()).unwrap(),
                candidate
            );
            finish(fixture, store, assessments, "explicit-abandon")
        }
        "retry-without-ack" => {
            let before = history(&fixture.workspace());
            assert!(matches!(
                store.decide_recovery(
                    IDS[0],
                    &fixture.nodes[0].context(),
                    approval(RunRecoveryAction::Retry, false)
                ),
                Err(StateError::Invalid {
                    rule: "explicit recovery acknowledgement"
                })
            ));
            assert_eq!(history(&fixture.workspace()), before);
            finish(fixture, store, assessments, "unacknowledged-retry-refused")
        }
        _ => finish(fixture, store, assessments, code),
    }
}

fn restart(fixture: &Fixture, mut store: RunStore, initial: Assessment, recipe: &str) -> Outcome {
    let mut assessments = vec![initial];
    if matches!(recipe, "completed-restart" | "partial-restart") {
        fixture.execute(&mut store, 0);
        assessments.push(fixture.assess(&store, "before-restart"));
    }
    drop(store);
    let mode = match recipe {
        "completed-restart" => "inspect",
        "partial-restart" => "resume",
        "host-exit-after-intent" => "exit-after-intent",
        "host-exit-after-provider" | "retry-interrupted" => "exit-after-provider",
        _ => unreachable!(),
    };
    let before = history(&fixture.workspace());
    fixture.child(mode);
    let mut store = RunStore::open(&fixture.workspace()).unwrap();
    assessments.push(fixture.assess(&store, "reopened"));
    if recipe == "completed-restart" {
        assert_eq!(history(&fixture.workspace()), before);
    }
    if recipe == "host-exit-after-intent" {
        assert!(!fixture.nodes[0].kit.output_path().exists());
    }
    if matches!(recipe, "host-exit-after-provider" | "retry-interrupted") {
        assert!(fixture.nodes[0].kit.output_path().is_file());
        assert!(store.state().steps[0].checkpoint.is_none());
    }
    if recipe == "retry-interrupted" {
        let output = fixture.nodes[0].kit.output_path();
        let uncertain = fs::read(&output).unwrap();
        let retained = fixture.nodes[0]
            .kit
            .root
            .path()
            .join("retained-interrupted-output.json");
        fs::rename(output, &retained).unwrap();
        store
            .decide_recovery(
                IDS[0],
                &fixture.nodes[0].context(),
                approval(RunRecoveryAction::Retry, true),
            )
            .unwrap();
        assessments.push(fixture.assess(&store, "retry-approved"));
        fixture.execute(&mut store, 0);
        assert_eq!(fs::read(retained).unwrap(), uncertain);
    }
    finish(
        fixture,
        store,
        assessments,
        match recipe {
            "completed-restart" => "completed-reused",
            "partial-restart" => "partial-run-resumed",
            "retry-interrupted" => "interrupted-retry-succeeded",
            _ => "uncertain-intent-retained",
        },
    )
}

fn replace_snapshot(fixture: &Fixture, recipe: &str) {
    let path = snapshot_paths(&fixture.workspace()).pop().unwrap();
    let original = fs::read(&path).unwrap();
    fs::write(
        fixture.nodes[0]
            .kit
            .root
            .path()
            .join("preserved-snapshot.json"),
        &original,
    )
    .unwrap();
    if recipe == "malformed-state" {
        fs::write(path, b"{broken").unwrap();
        return;
    }
    let mut value: Value = serde_json::from_slice(&original).unwrap();
    let checkpoint = &mut value["state"]["steps"][0]["checkpoint"];
    match recipe {
        "changed-validator" => {
            checkpoint["validation"]["implementation_digest"] = "0".repeat(64).into();
        }
        "changed-validation-profile" => {
            checkpoint["validation"]["profile"] = "flow.other-profile/v1".into();
        }
        "corrupt-checkpoint" => checkpoint["context_digest"] = "0".repeat(64).into(),
        "corrupt-checksum" => value["state_digest"] = "0".repeat(64).into(),
        "future-schema" => value["schema_version"] = "flow.run-snapshot/v999".into(),
        _ => unreachable!(),
    }
    if recipe != "corrupt-checksum" {
        value["state_digest"] = digest_json(&value["state"]).into();
    }
    fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
}

fn inject_drift(fixture: &mut Fixture, recipe: &str) {
    match recipe {
        "changed-input" => fixture.nodes[0]
            .preserve_and_replace("workspace/inputs/source-text.txt", b"changed input"),
        "changed-artifact" => fixture.nodes[0].preserve_and_replace(
            "workspace/outputs/inspection-report.json",
            b"changed output",
        ),
        "changed-implementation" => {
            let path = format!(
                "{}/{}",
                crate::PACKAGE_LOCATOR,
                fixture.nodes[0].kit.executable_locator
            );
            fixture.nodes[0].preserve_and_replace(&path, b"changed executable");
        }
        "changed-configuration" => {
            fixture.nodes[0]
                .prepared
                .invocation
                .configuration
                .values
                .insert("seed".to_owned(), json!("FLOW_LIFECYCLE_PRIVATE_CANARY"));
        }
        "changed-provider" => {
            "0.2.0".clone_into(&mut fixture.nodes[0].prepared.invocation.extension.version);
        }
        "changed-plan" => "plan:changed".clone_into(&mut fixture.plan.plan_id),
        "incomplete-write" => fs::write(
            fixture.workspace().join("snapshot.pending"),
            b"partial commit",
        )
        .unwrap(),
        _ => replace_snapshot(fixture, recipe),
    }
}

fn drift(fixture: &mut Fixture, mut store: RunStore, initial: Assessment, recipe: &str) -> Outcome {
    // Complete the independent branch first so a validator mutation changes only
    // the final Running -> Succeeded transition, not an older accepted checkpoint.
    if fixture.nodes.len() == 3 {
        fixture.execute(&mut store, 2);
    }
    fixture.execute(&mut store, 0);
    let before = history(&fixture.workspace());
    let assessments = vec![initial, fixture.assess(&store, "before-change")];
    drop(store);
    inject_drift(fixture, recipe);
    let refused = match recipe {
        "changed-validation-profile" => Some("invalid-validation-profile"),
        "corrupt-checkpoint" => Some("invalid-checkpoint-context"),
        "corrupt-checksum" => Some("corrupt-snapshot"),
        "future-schema" => Some("unsupported-schema"),
        "malformed-state" => Some("malformed-state"),
        "incomplete-write" => Some("incomplete-write"),
        _ => None,
    };
    if let Some(code) = refused {
        let error = RunStore::open(&fixture.workspace()).unwrap_err();
        assert!(matches!(
            (recipe, error),
            (
                "changed-validation-profile",
                StateError::Invalid {
                    rule: "validation profile"
                }
            ) | (
                "corrupt-checkpoint",
                StateError::Invalid {
                    rule: "checkpoint context"
                }
            ) | ("corrupt-checksum", StateError::Corrupt)
                | ("future-schema", StateError::UnsupportedSchema)
                | ("malformed-state", StateError::Malformed)
                | ("incomplete-write", StateError::IncompleteWrite)
        ));
        // A corrupt replacement cannot produce a legitimate later state frame.
        return Outcome {
            code: code.to_owned(),
            history: before,
            assessments,
            safety: None,
        };
    }
    let mut store = RunStore::open(&fixture.workspace()).unwrap();
    if recipe == "changed-plan" {
        assert!(matches!(
            store.assess_run(&fixture.plan, &fixture.contexts()),
            Err(StateError::Stale {
                boundary: StateBoundary::Plan
            })
        ));
        assert_eq!(history(&fixture.workspace()), before);
        return Outcome {
            code: "changed-plan-refused".to_owned(),
            history: before,
            assessments,
            safety: None,
        };
    }
    let after = history(&fixture.workspace());
    assert!(matches!(
        store.execute_in_plan(
            &fixture.plan,
            IDS[1],
            &fixture.contexts(),
            &NoSecrets,
            &NeverCancelled,
            &mut Vec::new()
        ),
        Err(DurableExecutionError::State(StateError::Ineligible {
            eligibility: ResumeEligibility::Invalidated
        }))
    ));
    assert!(!fixture.nodes[1].kit.output_path().exists());
    assert_eq!(history(&fixture.workspace()), after);
    finish(fixture, store, assessments, "stale-branch-invalidated")
}

pub(super) fn child() {
    let path = std::env::var_os("FLOW_LIFECYCLE_HANDOFF").expect("matrix handoff");
    let fixture = Fixture::from_handoff(Path::new(&path));
    let mode = std::env::var("FLOW_LIFECYCLE_CHILD_MODE").unwrap();
    let mut store = RunStore::open(&fixture.workspace()).unwrap();
    match mode.as_str() {
        "inspect" => assert_eq!(
            fixture.assess(&store, "child").steps[0].eligibility,
            ResumeEligibility::Reusable
        ),
        "resume" => {
            assert_eq!(
                fixture.assess(&store, "child").steps[0].eligibility,
                ResumeEligibility::Reusable
            );
            fixture.execute(&mut store, 1);
        }
        "exit-after-intent" => {
            let checks = Cell::new(0);
            let cancel = || {
                checks.set(checks.get() + 1);
                if checks.get() == 2 {
                    std::process::exit(73);
                }
                false
            };
            store
                .execute_in_plan(
                    &fixture.plan,
                    IDS[0],
                    &fixture.contexts(),
                    &NoSecrets,
                    &cancel,
                    &mut Vec::new(),
                )
                .unwrap();
            panic!("host must exit after intent");
        }
        "exit-after-provider" => {
            // LocalProcessRunner forwards validated events after reaping its child.
            let mut sink =
                |_: &ExtensionEvent| -> Result<(), EventSinkError> { std::process::exit(73) };
            store
                .execute_in_plan(
                    &fixture.plan,
                    IDS[0],
                    &fixture.contexts(),
                    &NoSecrets,
                    &NeverCancelled,
                    &mut sink,
                )
                .unwrap();
            panic!("host must exit before acceptance");
        }
        _ => panic!("unknown child mode"),
    }
}

fn classified_failure(
    fixture: &Fixture,
    mut store: RunStore,
    initial: Assessment,
    recipe: &str,
) -> Outcome {
    let mut events = Vec::new();
    let error = store
        .execute_in_plan(
            &fixture.plan,
            IDS[0],
            &fixture.contexts(),
            &NoSecrets,
            &NeverCancelled,
            &mut events,
        )
        .unwrap_err();
    assert!(matches!(error, DurableExecutionError::Acceptance(
        flow::ArtifactAcceptanceError::Mismatch { ref message }
    ) if message == "artifact acceptance requires a complete produced or reused result"));
    let node = &fixture.nodes[0];
    let transcript_path = node
        .kit
        .root
        .path()
        .join("workspace/outputs/failure-transcript.jsonl");
    let transcript = fs::read(&transcript_path).unwrap();
    let execution = flow::Orchestrator::validate_process_transcript(
        node.prepared.resolved(),
        &node.prepared.invocation,
        &node.prepared.subject_lock,
        &node.prepared.subjects,
        &node.prepared.authority,
        flow::ProcessTranscript::new(
            flow::ProcessCompletion::Exited { code: Some(0) },
            &transcript,
            &[],
        ),
        &mut Vec::new(),
    )
    .unwrap();
    assert_eq!(execution.events(), events);
    assert_eq!(execution.result().outcome, flow::Outcome::Failed);
    let (action, code) = match (
        &execution.result().failure.classification,
        execution.result().failure.retryable,
    ) {
        (flow::FailureClassification::Provider, true) => {
            assert_eq!(recipe, "retryable-provider-failure");
            (
                RunRecoveryAction::Retry,
                "retryable-provider-failure-approved",
            )
        }
        (flow::FailureClassification::Validation, false) => {
            assert_eq!(recipe, "terminal-provider-failure");
            (
                RunRecoveryAction::Abandon,
                "terminal-validation-failure-abandoned",
            )
        }
        other => panic!("unexpected provider failure classification: {other:?}"),
    };
    let assessments = vec![initial, fixture.assess(&store, "recovery-required")];
    let candidate = fs::read(node.kit.output_path()).unwrap();
    // The hint alone never retries. The test operator makes an explicit decision;
    // Retry leaves work pending and retains failed artifacts for operator handling.
    store
        .decide_recovery(IDS[0], &node.context(), approval(action, true))
        .unwrap();
    assert_eq!(fs::read(node.kit.output_path()).unwrap(), candidate);
    assert_eq!(fs::read(transcript_path).unwrap(), transcript);
    finish(fixture, store, assessments, code)
}
