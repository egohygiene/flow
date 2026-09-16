mod common;

use flow::{
    Domain, ExtensionCatalog, ExtensionLock, ExtensionManifest, ExtensionObservation,
    FallbackPolicy, ResolutionResult, Trust,
};

use common::{lock, manifest, observation, request, variant};

type CatalogMutation = Box<dyn Fn(&mut ExtensionManifest, &mut ExtensionLock)>;

fn lock_for(
    manifests: &[ExtensionManifest],
    precedences: &[u64],
    fallback: FallbackPolicy,
) -> ExtensionLock {
    assert_eq!(manifests.len(), precedences.len());
    let mut result = lock();
    let template = result.extensions[0].clone();
    result.extensions = manifests
        .iter()
        .zip(precedences)
        .map(|(manifest, precedence)| {
            let mut entry = template.clone();
            entry.extension_id.clone_from(&manifest.extension_id);
            entry.version.clone_from(&manifest.version);
            entry.publisher_id.clone_from(&manifest.publisher.id);
            entry.integrity = manifest.integrity.clone();
            entry.precedence = *precedence;
            entry
        })
        .collect();
    let ids = manifests
        .iter()
        .map(|manifest| manifest.extension_id.clone())
        .collect::<Vec<_>>();
    for policy in &mut result.capability_resolution {
        policy.ordered_extensions.clone_from(&ids);
        if policy.capability_id == "optiflow/inspect-collection" {
            policy.fallback = fallback;
        }
    }
    result
}

#[test]
fn resolution_is_independent_of_manifest_and_observation_order() {
    let base = manifest();
    let high = variant(&base, "org.example.high-adapter", 'a');
    let low = variant(&base, "org.example.low-adapter", 'b');
    let lock = lock_for(
        &[high.clone(), low.clone()],
        &[100, 50],
        FallbackPolicy::Forbidden,
    );
    let high_observation = observation(&high, true);
    let low_observation = observation(&low, true);

    let forward = ExtensionCatalog::inspect(
        [high.clone(), low.clone()],
        lock.clone(),
        [high_observation.clone(), low_observation.clone()],
    )
    .unwrap()
    .resolve(&request());
    let reverse = ExtensionCatalog::inspect([low, high], lock, [low_observation, high_observation])
        .unwrap()
        .resolve(&request());

    assert_eq!(forward.evidence(), reverse.evidence());
    assert_eq!(
        forward.resolved().unwrap().extension_id(),
        "org.example.high-adapter"
    );
    forward.evidence().validate().unwrap();
}

#[test]
fn equal_highest_precedence_is_an_explicit_conflict() {
    let base = manifest();
    let left = variant(&base, "org.example.left-adapter", 'c');
    let right = variant(&base, "org.example.right-adapter", 'd');
    let lock = lock_for(
        &[left.clone(), right.clone()],
        &[50, 50],
        FallbackPolicy::Forbidden,
    );
    let outcome = ExtensionCatalog::inspect(
        [left.clone(), right.clone()],
        lock,
        [observation(&right, true), observation(&left, true)],
    )
    .unwrap()
    .resolve(&request());

    assert_eq!(outcome.evidence().result, ResolutionResult::Conflict);
    assert!(outcome.resolved().is_none());
    outcome.evidence().validate().unwrap();
}

#[test]
fn before_effects_fallback_selects_only_after_observed_unavailability() {
    let base = manifest();
    let preferred = variant(&base, "org.example.preferred-adapter", 'e');
    let fallback = variant(&base, "org.example.local-fallback", 'f');
    let lock = lock_for(
        &[preferred.clone(), fallback.clone()],
        &[100, 50],
        FallbackPolicy::BeforeEffects,
    );
    let outcome = ExtensionCatalog::inspect(
        [fallback.clone(), preferred.clone()],
        lock,
        [observation(&preferred, false), observation(&fallback, true)],
    )
    .unwrap()
    .resolve(&request());

    assert_eq!(outcome.evidence().result, ResolutionResult::Selected);
    assert_eq!(
        outcome.evidence().fallback_order,
        [
            "org.example.preferred-adapter",
            "org.example.local-fallback"
        ]
    );
    assert_eq!(
        outcome.resolved().unwrap().extension_id(),
        "org.example.local-fallback"
    );
    outcome.evidence().validate().unwrap();
}

#[test]
fn forbidden_and_resume_only_policies_do_not_fallback() {
    for policy in [FallbackPolicy::Forbidden, FallbackPolicy::ExplicitResume] {
        let base = manifest();
        let preferred = variant(&base, "org.example.preferred-adapter", 'e');
        let fallback = variant(&base, "org.example.local-fallback", 'f');
        let lock = lock_for(&[preferred.clone(), fallback.clone()], &[100, 50], policy);
        let outcome = ExtensionCatalog::inspect(
            [preferred.clone(), fallback.clone()],
            lock,
            [observation(&preferred, false), observation(&fallback, true)],
        )
        .unwrap()
        .resolve(&request());

        assert_eq!(outcome.evidence().result, ResolutionResult::Blocked);
        assert!(outcome.resolved().is_none());
        assert!(outcome.evidence().fallback_order.is_empty());
        assert!(outcome.evidence().candidates.iter().any(|candidate| {
            candidate.extension_id == "org.example.local-fallback"
                && candidate.compatible
                && !candidate.authorized
                && candidate
                    .reasons
                    .iter()
                    .any(|reason| reason.contains("fallback policy forbids"))
        }));
        outcome.evidence().validate().unwrap();
    }
}

#[test]
fn equal_precedence_unavailability_obeys_fallback_policy() {
    for (policy, expected) in [
        (FallbackPolicy::Forbidden, ResolutionResult::Blocked),
        (FallbackPolicy::BeforeEffects, ResolutionResult::Selected),
    ] {
        let base = manifest();
        let unavailable = variant(&base, "org.example.equal-unavailable", 'e');
        let available = variant(&base, "org.example.equal-available", 'f');
        let lock = lock_for(&[unavailable.clone(), available.clone()], &[50, 50], policy);
        let outcome = ExtensionCatalog::inspect(
            [available.clone(), unavailable.clone()],
            lock,
            [
                observation(&unavailable, false),
                observation(&available, true),
            ],
        )
        .unwrap()
        .resolve(&request());

        assert_eq!(outcome.evidence().result, expected);
        outcome.evidence().validate().unwrap();
        if policy == FallbackPolicy::BeforeEffects {
            assert_eq!(
                outcome.resolved().unwrap().extension_id(),
                "org.example.equal-available"
            );
            assert_eq!(
                outcome.evidence().fallback_order,
                [
                    "org.example.equal-unavailable",
                    "org.example.equal-available"
                ]
            );
        } else {
            assert!(outcome.resolved().is_none());
        }
    }
}

#[test]
fn lock_rejects_policy_order_that_increases_precedence() {
    let base = manifest();
    let low = variant(&base, "org.example.policy-first-low", 'f');
    let high = variant(&base, "org.example.policy-second-high", 'e');
    let lock = lock_for(
        &[low.clone(), high.clone()],
        &[50, 100],
        FallbackPolicy::BeforeEffects,
    );
    assert!(
        ExtensionCatalog::inspect(
            [high.clone(), low.clone()],
            lock,
            [observation(&high, false), observation(&low, true)],
        )
        .is_err()
    );
}

#[test]
fn equal_precedence_unavailable_after_selected_does_not_trigger_fallback() {
    let base = manifest();
    let available = variant(&base, "org.example.equal-first-available", 'f');
    let unavailable = variant(&base, "org.example.equal-second-unavailable", 'e');
    let lock = lock_for(
        &[available.clone(), unavailable.clone()],
        &[50, 50],
        FallbackPolicy::Forbidden,
    );
    let outcome = ExtensionCatalog::inspect(
        [unavailable.clone(), available.clone()],
        lock,
        [
            observation(&unavailable, false),
            observation(&available, true),
        ],
    )
    .unwrap()
    .resolve(&request());

    assert_eq!(outcome.evidence().result, ResolutionResult::Selected);
    assert_eq!(
        outcome.resolved().unwrap().extension_id(),
        "org.example.equal-first-available"
    );
    assert!(outcome.evidence().fallback_order.is_empty());
}

#[test]
fn disabled_high_precedence_candidate_does_not_trigger_fallback_policy() {
    let base = manifest();
    let disabled = variant(&base, "org.example.disabled-high", 'd');
    let enabled = variant(&base, "org.example.enabled-low", 'e');
    let mut lock = lock_for(
        &[disabled.clone(), enabled.clone()],
        &[100, 50],
        FallbackPolicy::Forbidden,
    );
    lock.extensions
        .iter_mut()
        .find(|entry| entry.extension_id == disabled.extension_id)
        .unwrap()
        .enabled = false;
    let outcome = ExtensionCatalog::inspect(
        [disabled.clone(), enabled.clone()],
        lock,
        [observation(&disabled, true), observation(&enabled, true)],
    )
    .unwrap()
    .resolve(&request());

    assert_eq!(outcome.evidence().result, ResolutionResult::Selected);
    assert_eq!(
        outcome.resolved().unwrap().extension_id(),
        "org.example.enabled-low"
    );
    let rejected = outcome
        .evidence()
        .candidates
        .iter()
        .find(|candidate| candidate.extension_id == disabled.extension_id)
        .unwrap();
    assert!(!rejected.authorized);
    assert!(!rejected.available);
    assert!(outcome.evidence().fallback_order.is_empty());
}

#[test]
fn malformed_manifest_makes_catalog_resolution_atomic() {
    let valid = manifest();
    let mut malformed = variant(&valid, "org.example.malformed-local", 'a');
    malformed.capabilities[0].domain = Domain::ThirdParty;
    let outcome = ExtensionCatalog::inspect(
        [malformed.clone(), valid.clone()],
        lock(),
        [observation(&valid, true), observation(&malformed, true)],
    )
    .unwrap()
    .resolve(&request());

    assert_eq!(outcome.evidence().result, ResolutionResult::Malformed);
    assert!(outcome.evidence().candidates.is_empty());
    assert!(outcome.resolved().is_none());
    assert!(outcome.evidence().reasons[0].contains("resolution-v1 cannot represent"));
    outcome.evidence().validate().unwrap();
}

#[test]
fn malformed_manifest_values_do_not_leak_into_portable_reasons() {
    let mut malformed = manifest();
    malformed.inspection.health_mode = flow::HealthMode::Invocation;
    malformed.inspection.health_capability_ids =
        vec!["/private/customer/path?token=sentinel-secret".to_owned()];
    let outcome =
        ExtensionCatalog::inspect([malformed.clone()], lock(), [observation(&malformed, true)])
            .unwrap()
            .resolve(&request());

    let evidence = serde_json::to_string(outcome.evidence()).unwrap();
    assert_eq!(outcome.evidence().result, ResolutionResult::Malformed);
    assert!(!evidence.contains("private/customer"));
    assert!(!evidence.contains("sentinel-secret"));
}

#[test]
fn all_malformed_manifests_fail_as_malformed_resolution() {
    let mut malformed = manifest();
    malformed.capabilities[0].domain = Domain::ThirdParty;
    let outcome =
        ExtensionCatalog::inspect([malformed.clone()], lock(), [observation(&malformed, true)])
            .unwrap()
            .resolve(&request());

    assert_eq!(outcome.evidence().result, ResolutionResult::Malformed);
    assert!(outcome.resolved().is_none());
}

#[test]
fn empty_catalog_reports_no_compatible_provider() {
    let outcome = ExtensionCatalog::inspect([], lock(), [])
        .unwrap()
        .resolve(&request());

    assert_eq!(
        outcome.evidence().result,
        ResolutionResult::NoCompatibleProvider
    );
    assert!(outcome.evidence().candidates.is_empty());
    outcome.evidence().validate().unwrap();
}

#[test]
fn partially_materialized_trusted_policy_fails_closed() {
    for fallback_policy in [FallbackPolicy::Forbidden, FallbackPolicy::BeforeEffects] {
        let base = manifest();
        let preferred = variant(&base, "org.example.configured-preferred", 'a');
        let fallback = variant(&base, "org.example.configured-fallback", 'b');
        let lock = lock_for(&[preferred, fallback.clone()], &[100, 50], fallback_policy);
        let outcome =
            ExtensionCatalog::inspect([fallback.clone()], lock, [observation(&fallback, true)])
                .unwrap()
                .resolve(&request());

        assert_eq!(outcome.evidence().result, ResolutionResult::Malformed);
        assert!(outcome.evidence().candidates.is_empty());
        assert!(outcome.resolved().is_none());
        outcome.evidence().validate().unwrap();
    }
}

#[test]
fn substituted_preferred_pin_cannot_bypass_fallback_policy() {
    let substitutions: [fn(&mut ExtensionManifest); 3] = [
        |manifest| manifest.version = "0.1.1".to_owned(),
        |manifest| manifest.publisher.id = "org.example.substitute".to_owned(),
        |manifest| manifest.integrity.value = "c".repeat(64),
    ];

    for fallback_policy in [FallbackPolicy::Forbidden, FallbackPolicy::BeforeEffects] {
        for substitute in substitutions {
            let base = manifest();
            let preferred = variant(&base, "org.example.pinned-preferred", 'a');
            let fallback = variant(&base, "org.example.pinned-fallback", 'b');
            let lock = lock_for(
                &[preferred.clone(), fallback.clone()],
                &[100, 50],
                fallback_policy,
            );
            let mut substituted = preferred;
            substitute(&mut substituted);
            let outcome = ExtensionCatalog::inspect(
                [substituted.clone(), fallback.clone()],
                lock,
                [
                    observation(&substituted, false),
                    observation(&fallback, true),
                ],
            )
            .unwrap()
            .resolve(&request());

            assert_eq!(outcome.evidence().result, ResolutionResult::Malformed);
            assert!(outcome.evidence().candidates.is_empty());
            assert!(outcome.resolved().is_none());
            outcome.evidence().validate().unwrap();
        }
    }
}

#[test]
fn identity_integrity_contract_and_flow_version_fail_closed() {
    let cases: Vec<CatalogMutation> = vec![
        Box::new(|_, lock| lock.extensions[0].version = "9.9.9".to_owned()),
        Box::new(|_, lock| lock.extensions[0].publisher_id = "org.someone-else".to_owned()),
        Box::new(|_, lock| lock.extensions[0].integrity.value = "a".repeat(64)),
        Box::new(|manifest, _| {
            manifest
                .compatibility
                .contract_families
                .push("flow.extension-result/v2".to_owned());
        }),
        Box::new(|manifest, _| {
            manifest.compatibility.flow_version_requirement = ">=9.0.0".to_owned();
        }),
    ];

    for mutate in cases {
        let mut manifest = manifest();
        let mut lock = lock();
        mutate(&mut manifest, &mut lock);
        let outcome =
            ExtensionCatalog::inspect([manifest.clone()], lock, [observation(&manifest, true)])
                .unwrap()
                .resolve(&request());
        assert_eq!(
            outcome.evidence().result,
            ResolutionResult::NoCompatibleProvider
        );
        assert!(outcome.resolved().is_none());
        outcome.evidence().validate().unwrap();
    }
}

#[test]
fn all_three_runtime_contract_families_are_required() {
    let mut manifest = manifest();
    manifest.compatibility.contract_families = vec!["flow.artifact/v1".to_owned()];
    let outcome =
        ExtensionCatalog::inspect([manifest.clone()], lock(), [observation(&manifest, true)])
            .unwrap()
            .resolve(&request());

    assert_eq!(
        outcome.evidence().result,
        ResolutionResult::NoCompatibleProvider
    );
    assert!(
        outcome.evidence().candidates[0]
            .reasons
            .iter()
            .any(|reason| reason.contains("flow.extension-invocation/v1"))
    );
}

#[test]
fn permission_denial_is_blocked_with_complete_candidate_evidence() {
    let manifest = manifest();
    let mut lock = lock();
    lock.extensions[0]
        .granted_permissions
        .filesystem_write
        .clear();
    let outcome =
        ExtensionCatalog::inspect([manifest.clone()], lock, [observation(&manifest, true)])
            .unwrap()
            .resolve(&request());

    assert_eq!(outcome.evidence().result, ResolutionResult::Blocked);
    let candidate = &outcome.evidence().candidates[0];
    assert!(candidate.compatible);
    assert!(!candidate.authorized);
    assert!(candidate.available);
    assert!(
        candidate
            .reasons
            .iter()
            .any(|reason| reason.contains("permissions exceed"))
    );
}

#[test]
fn missing_lock_entry_is_compatible_but_blocked_by_missing_authority() {
    let unlocked = variant(&manifest(), "org.example.unlocked-adapter", 'a');
    let outcome =
        ExtensionCatalog::inspect([unlocked.clone()], lock(), [observation(&unlocked, true)])
            .unwrap()
            .resolve(&request());

    assert_eq!(outcome.evidence().result, ResolutionResult::Blocked);
    let candidate = &outcome.evidence().candidates[0];
    assert!(candidate.compatible);
    assert!(!candidate.authorized);
    assert!(!candidate.available);
    assert!(
        candidate
            .reasons
            .iter()
            .any(|reason| reason.contains("No operator lock entry authorizes"))
    );
}

#[test]
fn disabled_missing_and_integrity_mismatched_observations_are_unavailable() {
    let base_manifest = manifest();

    let mut disabled_lock = lock();
    disabled_lock.extensions[0].enabled = false;
    let disabled = ExtensionCatalog::inspect(
        [base_manifest.clone()],
        disabled_lock,
        [observation(&base_manifest, true)],
    )
    .unwrap()
    .resolve(&request());
    assert!(!disabled.evidence().candidates[0].available);

    let mut trust_lock = lock();
    trust_lock.extensions[0].trust = Trust::Disabled;
    let trust_disabled = ExtensionCatalog::inspect(
        [base_manifest.clone()],
        trust_lock,
        [observation(&base_manifest, true)],
    )
    .unwrap()
    .resolve(&request());
    assert!(!trust_disabled.evidence().candidates[0].available);

    let mut sandboxed_lock = lock();
    sandboxed_lock.extensions[0].trust = Trust::Sandboxed;
    let sandboxed = ExtensionCatalog::inspect(
        [base_manifest.clone()],
        sandboxed_lock,
        [observation(&base_manifest, true)],
    )
    .unwrap()
    .resolve(&request());
    assert_eq!(sandboxed.evidence().result, ResolutionResult::Blocked);
    assert!(!sandboxed.evidence().candidates[0].authorized);
    assert!(!sandboxed.evidence().candidates[0].available);
    assert!(
        sandboxed.evidence().candidates[0]
            .reasons
            .iter()
            .any(|reason| reason.contains("Sandboxed trust"))
    );

    let missing = ExtensionCatalog::inspect([base_manifest.clone()], lock(), []).unwrap();
    assert!(!missing.resolve(&request()).evidence().candidates[0].available);

    let mut mismatched = observation(&base_manifest, true);
    mismatched.integrity.value = "a".repeat(64);
    let mismatch = ExtensionCatalog::inspect([base_manifest], lock(), [mismatched])
        .unwrap()
        .resolve(&request());
    assert!(!mismatch.evidence().candidates[0].available);
}

#[test]
fn duplicate_observations_are_rejected_not_last_wins() {
    let manifest = manifest();
    let first = observation(&manifest, true);
    let second = ExtensionObservation {
        available: false,
        ..first.clone()
    };
    assert!(ExtensionCatalog::inspect([manifest], lock(), [first, second]).is_err());
}

#[test]
fn duplicate_observation_error_is_independent_of_input_order() {
    let base = manifest();
    let alpha = variant(&base, "org.example.alpha", 'a');
    let beta = variant(&base, "org.example.beta", 'b');
    let alpha_available = observation(&alpha, true);
    let alpha_unavailable = observation(&alpha, false);
    let beta_available = observation(&beta, true);
    let beta_unavailable = observation(&beta, false);
    let lock = lock_for(
        &[alpha.clone(), beta.clone()],
        &[100, 50],
        FallbackPolicy::Forbidden,
    );

    let forward = ExtensionCatalog::inspect(
        [alpha.clone(), beta.clone()],
        lock.clone(),
        [
            beta_available.clone(),
            beta_unavailable.clone(),
            alpha_available.clone(),
            alpha_unavailable.clone(),
        ],
    )
    .unwrap_err();
    let reverse = ExtensionCatalog::inspect(
        [beta, alpha],
        lock,
        [
            alpha_unavailable,
            alpha_available,
            beta_unavailable,
            beta_available,
        ],
    )
    .unwrap_err();

    assert_eq!(forward, reverse);
}

#[test]
fn caller_free_form_unavailability_text_cannot_enter_resolution_evidence() {
    let manifest = manifest();
    let mut encoded = serde_json::to_value(observation(&manifest, false)).unwrap();
    encoded["unavailable_reason"] = serde_json::json!("/private/path?token=secret");
    assert!(serde_json::from_value::<ExtensionObservation>(encoded).is_err());

    let outcome =
        ExtensionCatalog::inspect([manifest.clone()], lock(), [observation(&manifest, false)])
            .unwrap()
            .resolve(&request());
    let evidence = serde_json::to_string(outcome.evidence()).unwrap();
    assert!(!evidence.contains("private"));
    assert!(!evidence.contains("secret"));
}

#[test]
fn duplicate_extension_ids_are_rejected_even_when_versions_differ() {
    let first = manifest();
    let mut second = first.clone();
    second.version = "0.1.1".to_owned();
    let observations = [observation(&first, true), observation(&second, true)];

    assert!(ExtensionCatalog::inspect([first, second], lock(), observations).is_err());
}
