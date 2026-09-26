use super::fixture::IDS;
use super::*;
use flow::{
    DurableExecutionError, ExecutionError, ProcessAuthorityError, ProcessRunnerError,
    RecoveryApproval, ResolutionResult, ResumeEligibility, StateBoundary, StateError,
};
use std::cell::Cell;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SafetyEvidence {
    observations: Vec<EffectObservation>,
    authority: Vec<AuthorityObservation>,
    refusals: Vec<String>,
    residual: ResidualDisposition,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EffectObservation {
    at: String,
    counters: Vec<Counters>,
}

#[derive(Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Counters {
    launches: u64,
    effects: u64,
    cleanup_started: u64,
    cleanup_items_removed: u64,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorityObservation {
    step_id: String,
    attempt: u64,
    granted: bool,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ResidualDisposition {
    None,
    RetainedBeforeRetry,
    RetainedAfterAbandon,
    CleanupCancelled,
    CleanupFailed,
}

fn observe(fixture: &Fixture, at: &str) -> EffectObservation {
    let counters = fixture
        .nodes
        .iter()
        .map(|node| {
            let path = node
                .kit
                .root
                .path()
                .join("workspace/outputs/effect-journal.txt");
            let mut counters = Counters::default();
            if path.exists() {
                let bytes = fs::read_to_string(path).unwrap();
                assert!(bytes.len() <= 512);
                for line in bytes.lines() {
                    match line {
                        "launch" => counters.launches += 1,
                        "effect" => counters.effects += 1,
                        "cleanup-started" => counters.cleanup_started += 1,
                        "cleanup-item-removed" => counters.cleanup_items_removed += 1,
                        _ => panic!("unknown effect journal record"),
                    }
                }
            }
            counters
        })
        .collect();
    EffectObservation {
        at: at.to_owned(),
        counters,
    }
}

fn approval(action: RunRecoveryAction, acknowledge: bool, id: &str) -> RecoveryApproval {
    RecoveryApproval {
        decision_id: format!("decision:{id}"),
        action,
        acknowledge_uncertain_effects: acknowledge,
    }
}

fn execute(
    fixture: &Fixture,
    store: &mut RunStore,
    index: usize,
    cleanup_cancel: bool,
) -> Result<flow::AcceptedArtifactSet, DurableExecutionError> {
    let calls = Cell::new(0);
    let before = observe(fixture, "before-intent");
    let control = fixture.nodes[index].kit.lifecycle_control_path();
    let cancellation = || {
        calls.set(calls.get() + 1);
        if calls.get() == 2 {
            // The coordinator checks cancellation after persisting intent and
            // before invoking the runner. Inspect actual on-disk bytes here.
            let last = snapshot_paths(&fixture.workspace()).pop().unwrap();
            let snapshot: Value = serde_json::from_slice(&fs::read(last).unwrap()).unwrap();
            let state: RunState = serde_json::from_value(snapshot["state"].clone()).unwrap();
            state.validate().unwrap();
            assert_eq!(state.steps[index].status, RunStepStatus::Running);
            let decision = state.authority_decisions.last().unwrap();
            assert!(decision.granted);
            assert_eq!(decision.step_id, IDS[index]);
            assert_eq!(decision.attempt, state.steps[index].attempt);
            assert_eq!(
                observe(fixture, "before-intent"),
                before,
                "no launch/effect may precede persisted authority and intent"
            );
        }
        cleanup_cancel && control.is_file()
    };
    store.execute_in_plan(
        &fixture.plan,
        IDS[index],
        &fixture.contexts(),
        &NoSecrets,
        &cancellation,
        &mut Vec::new(),
    )
}

fn refuse_execution(
    fixture: &Fixture,
    store: &mut RunStore,
    index: usize,
    expected: ResumeEligibility,
) {
    let before = history(&fixture.workspace());
    let effects = observe(fixture, "refusal");
    let mut events = Vec::new();
    assert!(
        matches!(store.execute_in_plan(&fixture.plan, IDS[index], &fixture.contexts(),
        &NoSecrets, &NeverCancelled, &mut events),
        Err(DurableExecutionError::State(StateError::Ineligible { eligibility })) if eligibility == expected)
    );
    assert!(events.is_empty());
    assert_eq!(history(&fixture.workspace()), before);
    assert_eq!(observe(fixture, "refusal"), effects);
}

fn finish(
    fixture: &Fixture,
    store: RunStore,
    assessments: Vec<Assessment>,
    code: &str,
    observations: Vec<EffectObservation>,
    refusals: &[&str],
    residual: ResidualDisposition,
) -> Outcome {
    let authority = store
        .state()
        .authority_decisions
        .iter()
        .map(|decision| {
            let index = IDS.iter().position(|id| *id == decision.step_id).unwrap();
            let planned = &fixture.plan.steps[index].context;
            assert_eq!(decision.authorization_id, planned.authorization_id);
            assert_eq!(decision.profile_digest, planned.authority_profile_digest);
            assert_eq!(
                decision.enforcement_digest,
                planned.enforcement_evidence_digest
            );
            assert_eq!(decision.grants_digest, planned.grants_digest);
            AuthorityObservation {
                step_id: decision.step_id.clone(),
                attempt: decision.attempt,
                granted: decision.granted,
            }
        })
        .collect();
    for decision in &store.state().recovery_decisions {
        let index = IDS.iter().position(|id| *id == decision.step_id).unwrap();
        assert_eq!(
            decision.authorization_id,
            fixture.plan.steps[index].context.authorization_id
        );
        assert_eq!(
            decision.profile_digest,
            fixture.plan.steps[index].context.authority_profile_digest
        );
        assert!(decision.acknowledged_uncertain_effects);
    }
    for path in snapshot_paths(&fixture.workspace()) {
        let bytes = fs::read_to_string(path).unwrap();
        for node in &fixture.nodes {
            for private in [
                node.kit.root.path().to_str().unwrap(),
                node.prepared.invocation.configuration.values["seed"]
                    .as_str()
                    .unwrap(),
                "FLOW_EFFECT_PRIVATE_CANARY",
                "FLOW_CLEANUP_PRIVATE_CANARY",
            ] {
                assert!(
                    !bytes.contains(private),
                    "private values must remain outside state"
                );
            }
        }
    }
    let mut outcome = super::recipes::finish(fixture, store, assessments, code);
    outcome.safety = Some(SafetyEvidence {
        observations,
        authority,
        refusals: refusals.iter().map(|value| (*value).to_owned()).collect(),
        residual,
    });
    outcome
}

pub(super) fn run(
    fixture: &mut Fixture,
    store: RunStore,
    initial: Assessment,
    recipe: &str,
) -> Outcome {
    if recipe.starts_with("deny-") {
        return deny(fixture, store, initial, recipe);
    }
    match recipe {
        "effect-completed-reuse" => reuse(fixture, store, initial),
        "effect-stale-authority" => stale_authority(fixture, store, initial),
        "effect-retry-failed"
        | "effect-retry-interrupted"
        | "effect-abandon-interrupted"
        | "effect-recovery-decision-replay" => recover(fixture, store, initial, recipe),
        "cleanup-cancelled" | "cleanup-failed" => cleanup(fixture, store, initial, recipe),
        _ => panic!("unknown safety recipe"),
    }
}

fn request_effect(
    manifest: &mut flow::ExtensionManifest,
    profile: &mut flow::ProcessAuthorityProfile,
    recipe: &str,
) {
    match recipe {
        "deny-source-mutation" => {
            manifest.requested_permissions.source_mutation = true;
            profile
                .granted
                .source_mutation_targets
                .push("workspace/inputs".to_owned());
        }
        "deny-upload" | "deny-network" => {
            let endpoint = if recipe == "deny-upload" {
                "upload.example.test:443"
            } else {
                "api.example.test:443"
            };
            manifest
                .requested_permissions
                .network_hosts
                .push(endpoint.to_owned());
            profile.granted.network_endpoints.push(endpoint.to_owned());
        }
        "deny-publication" => {
            manifest.requested_permissions.publish = true;
            profile
                .granted
                .publication_destinations
                .push("destination:synthetic".to_owned());
        }
        "deny-signing" => {
            manifest.requested_permissions.sign = true;
            profile
                .granted
                .signing_key_handles
                .push("key:synthetic".to_owned());
        }
        "deny-paid-service" => {
            manifest
                .requested_permissions
                .ai_providers
                .push("provider:paid-synthetic".to_owned());
            profile
                .granted
                .ai_providers
                .push("provider:paid-synthetic".to_owned());
        }
        "deny-destructive" => {
            manifest.requested_permissions.destructive = true;
            profile
                .granted
                .destructive_operations
                .push("operation:synthetic-delete".to_owned());
        }
        _ => unreachable!(),
    }
}

fn deny(fixture: &Fixture, mut store: RunStore, initial: Assessment, recipe: &str) -> Outcome {
    let mut observations = vec![observe(fixture, "initial")];
    let node = &fixture.nodes[0];
    let mut manifest = node.kit.manifest.clone();
    let mut profile = node.prepared.authority.profile().clone();
    request_effect(&mut manifest, &mut profile, recipe);
    // Declarations cannot widen the operator lock. No usable resolution token
    // exists for this effect request, even with a real runnable provider present.
    let catalog = flow::ExtensionCatalog::inspect(
        [manifest.clone()],
        node.kit.lock.clone(),
        [crate::common::observation(&manifest, true)],
    )
    .unwrap();
    let resolution = catalog.resolve(&flow::ResolutionRequest::new(
        "effect-denial",
        crate::CAPABILITIES[0].capability_id,
        flow::Domain::Flow,
        "hermetic-process",
        flow::ExecutionModeKind::Process,
    ));
    assert_eq!(resolution.evidence().result, ResolutionResult::Blocked);
    assert!(resolution.resolved().is_none());
    assert!(!resolution.evidence().candidates[0].authorized);
    // A caller cannot widen the exact profile after resolution either.
    let enforcement = crate::process_enforcement_evidence(&profile, &node.prepared.subjects);
    assert!(matches!(
        flow::authorize_process(
            node.prepared.resolved(),
            &node.prepared.invocation,
            &node.prepared.subject_lock,
            &node.prepared.subjects,
            &profile,
            &enforcement
        ),
        Err(ProcessAuthorityError::AuthorityExceeded { .. })
    ));
    store.deny_pending(IDS[0]).unwrap();
    let denied = fixture.assess(&store, "denied");
    refuse_execution(fixture, &mut store, 0, ResumeEligibility::ApprovalRequired);
    refuse_execution(fixture, &mut store, 1, ResumeEligibility::DependencyBlocked);
    observations.push(observe(fixture, "denied"));
    // Positive control: the identical harmless counter mode really executes on
    // independently authorized C. Denying A must not disable unrelated work.
    execute(fixture, &mut store, 2, false).unwrap();
    observations.push(observe(fixture, "independent-complete"));
    assert!(!node.kit.output_path().exists());
    assert!(!fixture.nodes[1].kit.output_path().exists());
    finish(
        fixture,
        store,
        vec![initial, denied],
        "authority-denied-before-effects",
        observations,
        &[
            "catalog-blocked",
            "profile-exceeds-grants",
            "denied-step",
            "dependent-blocked",
        ],
        ResidualDisposition::None,
    )
}

fn reuse(fixture: &Fixture, mut store: RunStore, initial: Assessment) -> Outcome {
    let mut observations = vec![observe(fixture, "initial")];
    execute(fixture, &mut store, 0, false).unwrap();
    execute(fixture, &mut store, 2, false).unwrap();
    let accepted = fs::read(fixture.nodes[0].kit.output_path()).unwrap();
    observations.push(observe(fixture, "accepted"));
    let before = fixture.assess(&store, "before-restart");
    drop(store);
    fixture.child("resume");
    let mut store = RunStore::open(&fixture.workspace()).unwrap();
    let resumed = fixture.assess(&store, "reopened");
    for (index, id) in IDS.iter().enumerate() {
        refuse_execution(fixture, &mut store, index, ResumeEligibility::Reusable);
        assert!(matches!(
            store.decide_recovery(
                id,
                &fixture.nodes[index].context(),
                approval(RunRecoveryAction::Retry, true, "completed")
            ),
            Err(StateError::Transition)
        ));
    }
    assert_eq!(
        fs::read(fixture.nodes[0].kit.output_path()).unwrap(),
        accepted
    );
    observations.push(observe(fixture, "reused-without-relaunch"));
    finish(
        fixture,
        store,
        vec![initial, before, resumed],
        "accepted-effects-not-repeated",
        observations,
        &["completed-execution", "completed-retry"],
        ResidualDisposition::None,
    )
}

fn stale_authority(fixture: &mut Fixture, mut store: RunStore, initial: Assessment) -> Outcome {
    let mut observations = vec![observe(fixture, "initial")];
    execute(fixture, &mut store, 0, false).unwrap();
    execute(fixture, &mut store, 2, false).unwrap();
    let before = fixture.assess(&store, "before-change");
    "authorization:changed".clone_into(
        &mut fixture.nodes[0]
            .prepared
            .invocation
            .authorization
            .authorization_id,
    );
    refuse_execution(fixture, &mut store, 0, ResumeEligibility::Invalidated);
    refuse_execution(fixture, &mut store, 1, ResumeEligibility::Invalidated);
    observations.push(observe(fixture, "authority-invalidated"));
    finish(
        fixture,
        store,
        vec![initial, before],
        "stale-authority-blocks-descendants",
        observations,
        &["stale-authority", "dependent-invalidated"],
        ResidualDisposition::None,
    )
}

fn recovery_refusals(fixture: &mut Fixture, store: &mut RunStore) {
    let before = history(&fixture.workspace());
    let effects = observe(fixture, "recovery-refused");
    refuse_execution(fixture, store, 0, ResumeEligibility::ApprovalRequired);
    assert!(matches!(
        store.decide_recovery(
            IDS[0],
            &fixture.nodes[0].context(),
            approval(RunRecoveryAction::Retry, false, "unacknowledged")
        ),
        Err(StateError::Invalid {
            rule: "explicit recovery acknowledgement"
        })
    ));
    // Both changed authorization identity and changed exact grants are stale.
    let original = fixture.nodes[0].prepared.invocation.authorization.clone();
    for change_grants in [false, true] {
        let auth = &mut fixture.nodes[0].prepared.invocation.authorization;
        *auth = original.clone();
        if change_grants {
            auth.grants_digest = "a".repeat(64);
        } else {
            "authorization:wrong-recovery".clone_into(&mut auth.authorization_id);
        }
        assert!(matches!(
            store.decide_recovery(
                IDS[0],
                &fixture.nodes[0].context(),
                approval(RunRecoveryAction::Retry, true, "wrong-authority")
            ),
            Err(StateError::Stale {
                boundary: StateBoundary::Authority
            })
        ));
    }
    fixture.nodes[0].prepared.invocation.authorization = original;
    assert_eq!(history(&fixture.workspace()), before);
    assert_eq!(observe(fixture, "recovery-refused"), effects);
}

fn process_failure(error: &DurableExecutionError, code: i32) {
    assert!(
        matches!(error, DurableExecutionError::Process(ProcessRunnerError::Validation {
        source: ExecutionError::ProcessExit { code: Some(actual) }
    }) if *actual == code)
    );
}

fn preserve_candidate(fixture: &Fixture, name: &str) -> Vec<u8> {
    let path = fixture.nodes[0].kit.output_path();
    let bytes = fs::read(&path).unwrap();
    let retained = fixture.nodes[0].kit.root.path().join(name);
    assert!(!retained.exists());
    fs::rename(path, &retained).unwrap();
    assert_eq!(fs::read(retained).unwrap(), bytes);
    bytes
}

#[allow(clippy::too_many_lines)]
fn recover(
    fixture: &mut Fixture,
    mut store: RunStore,
    initial: Assessment,
    recipe: &str,
) -> Outcome {
    let mut observations = vec![observe(fixture, "initial")];
    if recipe.contains("interrupted") {
        drop(store);
        fixture.child("exit-after-provider");
        store = RunStore::open(&fixture.workspace()).unwrap();
        assert_eq!(store.state().steps[0].status, RunStepStatus::Running);
    } else {
        process_failure(&execute(fixture, &mut store, 0, false).unwrap_err(), 7);
    }
    assert!(store.state().steps[0].checkpoint.is_none());
    let unresolved = fixture.assess(&store, "recovery-required");
    recovery_refusals(fixture, &mut store);
    observations.push(observe(fixture, "refused-without-authority"));
    let mut assessments = vec![initial, unresolved];
    if recipe == "effect-abandon-interrupted" {
        let bytes = fs::read(fixture.nodes[0].kit.output_path()).unwrap();
        store
            .decide_recovery(
                IDS[0],
                &fixture.nodes[0].context(),
                approval(RunRecoveryAction::Abandon, true, "abandon"),
            )
            .unwrap();
        refuse_execution(fixture, &mut store, 0, ResumeEligibility::Abandoned);
        assert_eq!(fs::read(fixture.nodes[0].kit.output_path()).unwrap(), bytes);
        observations.push(observe(fixture, "abandoned-with-evidence"));
        return finish(
            fixture,
            store,
            assessments,
            "uncertain-effects-abandoned",
            observations,
            &[
                "unapproved-execution",
                "missing-acknowledgement",
                "wrong-authorization",
                "wrong-grants",
                "abandoned-execution",
            ],
            ResidualDisposition::RetainedAfterAbandon,
        );
    }
    // This is an explicit test-operator retention action, never automatic Flow cleanup.
    let original = preserve_candidate(fixture, "retained-first-candidate.json");
    store
        .decide_recovery(
            IDS[0],
            &fixture.nodes[0].context(),
            approval(RunRecoveryAction::Retry, true, "retry"),
        )
        .unwrap();
    assert_eq!(store.state().recovery_decisions.last().unwrap().attempt, 1);
    assessments.push(fixture.assess(&store, "retry-approved"));
    observations.push(observe(fixture, "approval-does-not-launch"));
    let mut refusals = vec![
        "unapproved-execution",
        "missing-acknowledgement",
        "wrong-authorization",
        "wrong-grants",
    ];
    let residual = if recipe == "effect-recovery-decision-replay" {
        process_failure(&execute(fixture, &mut store, 0, false).unwrap_err(), 7);
        let before = history(&fixture.workspace());
        assert!(matches!(
            store.decide_recovery(
                IDS[0],
                &fixture.nodes[0].context(),
                approval(RunRecoveryAction::Retry, true, "retry")
            ),
            Err(StateError::Invalid {
                rule: "recovery identity"
            })
        ));
        assert_eq!(history(&fixture.workspace()), before);
        refuse_execution(fixture, &mut store, 0, ResumeEligibility::ApprovalRequired);
        refusals.push("replayed-decision");
        store
            .decide_recovery(
                IDS[0],
                &fixture.nodes[0].context(),
                approval(RunRecoveryAction::Abandon, true, "abandon-second"),
            )
            .unwrap();
        assert_eq!(store.state().recovery_decisions.last().unwrap().attempt, 2);
        ResidualDisposition::RetainedAfterAbandon
    } else {
        execute(fixture, &mut store, 0, false).unwrap();
        refuse_execution(fixture, &mut store, 0, ResumeEligibility::Reusable);
        ResidualDisposition::RetainedBeforeRetry
    };
    assert_eq!(
        fs::read(
            fixture.nodes[0]
                .kit
                .root
                .path()
                .join("retained-first-candidate.json")
        )
        .unwrap(),
        original
    );
    assert_eq!(
        fs::read(fixture.nodes[0].kit.output_path()).unwrap(),
        original
    );
    observations.push(observe(fixture, "after-explicit-retry"));
    finish(
        fixture,
        store,
        assessments,
        if recipe == "effect-recovery-decision-replay" {
            "replayed-recovery-refused"
        } else {
            "acknowledged-effect-retry"
        },
        observations,
        &refusals,
        residual,
    )
}

fn cleanup(fixture: &Fixture, mut store: RunStore, initial: Assessment, recipe: &str) -> Outcome {
    let mut observations = vec![observe(fixture, "initial")];
    let cancelled = recipe == "cleanup-cancelled";
    let error = execute(fixture, &mut store, 0, cancelled).unwrap_err();
    if cancelled {
        assert!(matches!(
            error,
            DurableExecutionError::Process(ProcessRunnerError::Cancelled { .. })
        ));
        crate::assert_recorded_process_reaped(&fixture.nodes[0].kit.lifecycle_control_path());
    } else {
        process_failure(&error, 2);
    }
    assert!(store.state().steps[0].checkpoint.is_none());
    let artifact_root = fixture.nodes[0].kit.root.path().join("workspace");
    let pending = artifact_root.join("outputs/cleanup-pending/evidence.txt");
    let residual = fs::read(&pending).unwrap();
    assert_eq!(
        residual,
        b"FLOW_CLEANUP_PRIVATE_CANARY: unfinished cleanup\n"
    );
    assert!(
        !artifact_root
            .join("outputs/cleanup-disposable.txt")
            .exists()
    );
    let candidate = fs::read(fixture.nodes[0].kit.output_path()).unwrap();
    // The direct child is done, but cleanup is incomplete. Reopening and refusal
    // must preserve both unaccepted candidate bytes and unfinished work.
    drop(store);
    let mut store = RunStore::open(&fixture.workspace()).unwrap();
    let reopened = fixture.assess(&store, "cleanup-incomplete");
    refuse_execution(fixture, &mut store, 0, ResumeEligibility::ApprovalRequired);
    refuse_execution(fixture, &mut store, 1, ResumeEligibility::DependencyBlocked);
    assert_eq!(fs::read(pending).unwrap(), residual);
    assert_eq!(
        fs::read(fixture.nodes[0].kit.output_path()).unwrap(),
        candidate
    );
    observations.push(observe(fixture, "residuals-preserved"));
    finish(
        fixture,
        store,
        vec![initial, reopened],
        "cleanup-incomplete-evidence-retained",
        observations,
        &["unapproved-execution", "dependent-blocked"],
        if cancelled {
            ResidualDisposition::CleanupCancelled
        } else {
            ResidualDisposition::CleanupFailed
        },
    )
}
