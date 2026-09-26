use super::*;

pub(super) fn run(fixture: &KitFixture, recipe: &str) -> (Outcome, Value) {
    if recipe.starts_with("fallback-") || recipe == "conflict" {
        return policy_case(fixture, recipe);
    }
    let mut manifest = fixture.manifest.clone();
    let mut lock = fixture.lock.clone();
    let mut observed = common::observation(&manifest, true);
    let mut manifests = Vec::new();
    let mut observations = Vec::new();
    let mut capability = CAPABILITIES[0].capability_id;
    match recipe {
        "compatible" | "missing-provider" | "missing-observation" => {}
        "unavailable" => observed.available = false,
        "stale-version" => "0.0.9".clone_into(&mut lock.extensions[0].version),
        "future-version" => "9.0.0".clone_into(&mut lock.extensions[0].version),
        "publisher-mismatch" => {
            "org.example.other".clone_into(&mut lock.extensions[0].publisher_id);
        }
        "integrity-mismatch" => lock.extensions[0].integrity.value = "a".repeat(64),
        "observation-mismatch" => observed.integrity.value = "b".repeat(64),
        "future-flow" => ">=9.0.0".clone_into(&mut manifest.compatibility.flow_version_requirement),
        "future-contract" => manifest
            .compatibility
            .contract_families
            .push("flow.extension-result/v2".to_owned()),
        "missing-capability" => capability = "flow/missing-fixture",
        "permission-denied" => lock.extensions[0]
            .granted_permissions
            .filesystem_write
            .clear(),
        "malformed-manifest" => {
            "flow.extension-manifest/v99".clone_into(&mut manifest.schema_version);
        }
        other => panic!("unknown resolution recipe: {other}"),
    }
    if recipe != "missing-provider" {
        manifests.push(manifest);
    }
    if recipe != "missing-observation" && recipe != "missing-provider" {
        observations.push(observed);
    }
    let catalog = ExtensionCatalog::inspect(manifests, lock, observations).unwrap();
    let result = catalog.resolve(&ResolutionRequest::new(
        "acceptance-resolution",
        capability,
        Domain::Flow,
        "hermetic-process",
        ExecutionModeKind::Process,
    ));
    resolution_evidence(&result)
}

fn resolution_evidence(result: &ResolutionOutcome) -> (Outcome, Value) {
    result.evidence().validate().unwrap();
    let (code, state) = match result.evidence().result {
        ResolutionResult::Selected => {
            assert!(result.resolved().is_some());
            ("selected", ScenarioTerminalState::Complete)
        }
        ResolutionResult::Blocked => ("blocked", ScenarioTerminalState::Unavailable),
        ResolutionResult::NoCompatibleProvider => {
            ("no-compatible-provider", ScenarioTerminalState::Unsupported)
        }
        ResolutionResult::Malformed => ("malformed", ScenarioTerminalState::Invalid),
        ResolutionResult::Conflict => ("conflict", ScenarioTerminalState::Invalid),
    };
    if code != "selected" {
        assert!(result.resolved().is_none());
    }
    (
        outcome("resolution", code, state),
        serde_json::to_value(result.evidence()).unwrap(),
    )
}

fn policy_case(fixture: &KitFixture, recipe: &str) -> (Outcome, Value) {
    let primary = fixture.manifest.clone();
    let mut secondary = primary.clone();
    "org.egohygiene.synthetic-fallback".clone_into(&mut secondary.extension_id);
    let mut lock = fixture.lock.clone();
    let mut second_lock = lock.extensions[0].clone();
    second_lock.extension_id.clone_from(&secondary.extension_id);
    second_lock.precedence = if recipe == "conflict" { 100 } else { 50 };
    lock.extensions.push(second_lock);
    let policy = match recipe {
        "fallback-forbidden" | "conflict" => FallbackPolicy::Forbidden,
        "fallback-explicit-resume" => FallbackPolicy::ExplicitResume,
        "fallback-before-effects" | "fallback-after-invocation" => FallbackPolicy::BeforeEffects,
        _ => unreachable!(),
    };
    for item in &mut lock.capability_resolution {
        item.ordered_extensions.push(secondary.extension_id.clone());
        item.fallback = policy;
    }
    let available = matches!(recipe, "conflict" | "fallback-after-invocation");
    let observations = vec![
        common::observation(&primary, available),
        common::observation(&secondary, true),
    ];
    let request = ResolutionRequest::new(
        "acceptance-policy",
        CAPABILITIES[0].capability_id,
        Domain::Flow,
        "hermetic-process",
        ExecutionModeKind::Process,
    );
    let forward = ExtensionCatalog::inspect(
        [primary.clone(), secondary.clone()],
        lock.clone(),
        observations.clone(),
    )
    .unwrap()
    .resolve(&request);
    let reverse =
        ExtensionCatalog::inspect([secondary, primary], lock, observations.into_iter().rev())
            .unwrap()
            .resolve(&request);
    assert_eq!(forward.evidence(), reverse.evidence());
    if recipe == "fallback-before-effects" {
        assert_eq!(
            forward.resolved().unwrap().extension_id(),
            "org.egohygiene.synthetic-fallback"
        );
        assert_eq!(forward.evidence().fallback_order.len(), 2);
    }
    if recipe == "fallback-after-invocation" {
        // Exercise a selected before-effects policy against a real failing process.
        let mut prepared =
            fixture.prepare_lifecycle(CAPABILITIES[0], "nonzero-after-success", false);
        prepared.resolution = forward;
        assert_eq!(
            prepared.resolved().fallback_policy(),
            FallbackPolicy::BeforeEffects
        );
        let error = execute(fixture, &prepared).unwrap_err();
        assert!(
            fixture.output_path().is_file(),
            "the selected provider actually ran"
        );
        if let ProcessRunnerError::Validation { source } = error {
            // The executor receives one resolved token, never a catalog to reselect.
            return (
                execution_error(&source),
                json!({"selected_provider": prepared.resolved().extension_id(), "fallback_policy": "before-effects", "selected_process_created_output": true}),
            );
        }
        panic!("unexpected runner failure: {error:?}");
    }
    resolution_evidence(&forward)
}
