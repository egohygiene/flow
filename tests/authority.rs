mod common;

use flow::{
    AmbientAuthorityPolicy, EnforcementEvidenceSource, EnforcementStatus, ExecutionError,
    Orchestrator, ProcessAuthorityError, ProcessIsolation, TelemetryField, Trust,
    authorize_process, observe_execution_subjects,
};

use common::{
    invocation, process_authority_profile, process_enforcement_evidence, process_subject_fixture,
    process_subject_fixture_with_permissions, process_subject_fixture_with_trust,
};

#[test]
fn trusted_unconfined_preflight_has_stable_identity_and_explicit_non_enforcement() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let profile = process_authority_profile(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    let evidence = process_enforcement_evidence(&profile, &subjects);

    let first = authorize_process(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &profile,
        &evidence,
    )
    .unwrap();
    let second = authorize_process(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &profile,
        &evidence,
    )
    .unwrap();

    assert_eq!(first, second);
    assert_eq!(first.profile_digest(), profile.canonical_digest().unwrap());
    assert_eq!(
        first.evidence_digest(),
        evidence.canonical_digest().unwrap()
    );
    assert_eq!(first.isolation(), ProcessIsolation::TrustedUnconfined);
    assert_eq!(
        profile.ambient_authority,
        AmbientAuthorityPolicy::DenyUnlisted
    );
    assert_eq!(evidence.source, EnforcementEvidenceSource::None);
    assert!(
        evidence
            .guarantees
            .iter()
            .all(|claim| claim.status == EnforcementStatus::NotEnforced)
    );
    assert!(evidence.enforced.argv.is_empty());
}

#[test]
fn sandbox_required_processes_resolve_only_with_exact_host_enforcement_evidence() {
    let fixture = process_subject_fixture_with_trust(Trust::Sandboxed);
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().expect("sandboxed process must resolve");
    assert!(
        outcome.evidence().candidates[0]
            .reasons
            .iter()
            .any(|reason| { reason.contains("downstream authority and enforcement preflight") })
    );
    let invocation = invocation(resolved);
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let profile = process_authority_profile(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::Sandboxed,
    );
    let evidence = process_enforcement_evidence(&profile, &subjects);

    let authorized = authorize_process(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &profile,
        &evidence,
    )
    .unwrap();
    assert_eq!(authorized.isolation(), ProcessIsolation::Sandboxed);
    assert_eq!(evidence.enforced, profile.requested);
    assert!(
        evidence
            .guarantees
            .iter()
            .all(|claim| claim.status == EnforcementStatus::Enforced)
    );

    let mut incomplete = evidence.clone();
    incomplete.enforced.filesystem_read.clear();
    assert!(matches!(
        authorize_process(
            resolved,
            &invocation,
            &fixture.subject_lock,
            &subjects,
            &profile,
            &incomplete,
        ),
        Err(ProcessAuthorityError::EnforcementMismatch { .. })
    ));

    let mut unconfined = profile.clone();
    unconfined.isolation = ProcessIsolation::TrustedUnconfined;
    assert!(matches!(
        authorize_process(
            resolved,
            &invocation,
            &fixture.subject_lock,
            &subjects,
            &unconfined,
            &evidence,
        ),
        Err(ProcessAuthorityError::InvalidProfile { .. })
    ));
}

#[test]
fn every_authority_dimension_can_be_explicitly_requested_granted_and_bound() {
    let mut permissions = common::manifest().requested_permissions;
    permissions.environment_read = vec!["FLOW_TOKEN".to_owned()];
    permissions.network_hosts = vec!["api.example.test:443".to_owned()];
    permissions.ai_providers = vec!["provider:synthetic".to_owned()];
    permissions.gpu = true;
    permissions.source_mutation = true;
    permissions.destructive = true;
    permissions.sign = true;
    permissions.publish = true;
    let fixture =
        process_subject_fixture_with_permissions(Trust::Trusted, permissions.clone(), permissions);
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let mut invocation = invocation(resolved);
    invocation.secret_handles = vec!["secret:env.flow_token".to_owned()];
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let mut profile = process_authority_profile(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    profile.requested.telemetry.fields = vec![TelemetryField::Traceparent];
    profile.requested.telemetry.targets = vec!["observer:synthetic".to_owned()];
    profile.granted.telemetry = profile.requested.telemetry.clone();
    let evidence = process_enforcement_evidence(&profile, &subjects);

    authorize_process(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &profile,
        &evidence,
    )
    .unwrap();
    assert!(!profile.requested.argv.is_empty());
    assert!(!profile.requested.environment.is_empty());
    assert!(!profile.requested.filesystem_read.is_empty());
    assert!(!profile.requested.filesystem_write.is_empty());
    assert!(!profile.requested.network_endpoints.is_empty());
    assert!(!profile.requested.subprocesses.is_empty());
    assert!(!profile.requested.ai_providers.is_empty());
    assert!(!profile.requested.gpus.is_empty());
    assert!(!profile.requested.source_mutation_targets.is_empty());
    assert!(!profile.requested.destructive_operations.is_empty());
    assert!(!profile.requested.publication_destinations.is_empty());
    assert!(!profile.requested.signing_key_handles.is_empty());
    assert!(!profile.requested.telemetry.fields.is_empty());
    assert!(!profile.requested.telemetry.targets.is_empty());
}

#[test]
fn profile_context_rejects_wrong_provider_version_capability_interface_and_subject() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let profile = process_authority_profile(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    let evidence = process_enforcement_evidence(&profile, &subjects);

    let mut variants = Vec::new();
    let mut wrong_provider = profile.clone();
    wrong_provider.extension.extension_id = "org.example.other-adapter".to_owned();
    variants.push(wrong_provider);
    let mut wrong_version = profile.clone();
    wrong_version.extension.version = "0.2.0".to_owned();
    variants.push(wrong_version);
    let mut wrong_publisher = profile.clone();
    wrong_publisher.extension.publisher_id = "org.example".to_owned();
    variants.push(wrong_publisher);
    let mut wrong_capability = profile.clone();
    wrong_capability.capability_id = "renderflow/inspect-collection".to_owned();
    variants.push(wrong_capability);
    let mut wrong_interface = profile.clone();
    wrong_interface.interface.name = "other-process".to_owned();
    variants.push(wrong_interface);
    let mut wrong_subject = profile.clone();
    wrong_subject.subject_lock_digest = "f".repeat(64);
    variants.push(wrong_subject);

    for variant in variants {
        assert!(matches!(
            authorize_process(
                resolved,
                &invocation,
                &fixture.subject_lock,
                &subjects,
                &variant,
                &evidence,
            ),
            Err(ProcessAuthorityError::ContextMismatch { .. })
        ));
    }
}

#[test]
fn duplicate_wildcard_ungranted_and_overbroad_authority_fail_closed() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let profile = process_authority_profile(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    let evidence = process_enforcement_evidence(&profile, &subjects);

    let mut duplicate = profile.clone();
    duplicate.requested.filesystem_read.push("input".to_owned());
    assert!(duplicate.validate().is_err());

    let mut wildcard = profile.clone();
    wildcard.requested.network_endpoints.push("*".to_owned());
    wildcard.granted.network_endpoints.push("*".to_owned());
    assert!(wildcard.validate().is_err());

    let mut ungranted = profile.clone();
    ungranted.granted.filesystem_read.clear();
    assert!(ungranted.validate().is_err());

    let mut overbroad = profile.clone();
    overbroad
        .granted
        .network_endpoints
        .push("api.example.test:443".to_owned());
    let overbroad_evidence = process_enforcement_evidence(&overbroad, &subjects);
    assert!(matches!(
        authorize_process(
            resolved,
            &invocation,
            &fixture.subject_lock,
            &subjects,
            &overbroad,
            &overbroad_evidence,
        ),
        Err(ProcessAuthorityError::AuthorityExceeded { .. })
    ));

    let mut different_argv = profile.clone();
    different_argv.granted.argv.push("--extra".to_owned());
    assert!(different_argv.validate().is_err());

    assert!(
        authorize_process(
            resolved,
            &invocation,
            &fixture.subject_lock,
            &subjects,
            &profile,
            &evidence,
        )
        .is_ok()
    );
}

#[test]
fn duplicate_contradictory_unsupported_and_mismatched_enforcement_evidence_is_rejected() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let profile = process_authority_profile(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    let evidence = process_enforcement_evidence(&profile, &subjects);

    let mut duplicate = evidence.clone();
    duplicate.guarantees.push(duplicate.guarantees[0].clone());
    assert!(duplicate.validate().is_err());

    let mut contradictory = evidence.clone();
    contradictory.guarantees[0].status = EnforcementStatus::Enforced;
    assert!(contradictory.validate().is_err());

    let mut reordered = evidence.clone();
    reordered.guarantees.swap(0, 1);
    assert!(reordered.validate().is_err());

    let mut unsupported = evidence.clone();
    unsupported
        .unsupported_claims
        .push(serde_json::json!({"kind": "fabricated-sandbox-proof"}));
    assert!(unsupported.validate().is_err());

    let mut wrong_profile = evidence.clone();
    wrong_profile.authority_profile_digest = "f".repeat(64);
    assert!(matches!(
        authorize_process(
            resolved,
            &invocation,
            &fixture.subject_lock,
            &subjects,
            &profile,
            &wrong_profile,
        ),
        Err(ProcessAuthorityError::EnforcementMismatch { .. })
    ));

    let mut wrong_subject = evidence;
    wrong_subject.subject_observation_digest = "e".repeat(64);
    assert!(matches!(
        authorize_process(
            resolved,
            &invocation,
            &fixture.subject_lock,
            &subjects,
            &profile,
            &wrong_subject,
        ),
        Err(ProcessAuthorityError::EnforcementMismatch { .. })
    ));
}

#[test]
fn opaque_authorization_tokens_cannot_be_reused_for_another_invocation() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let profile = process_authority_profile(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    let evidence = process_enforcement_evidence(&profile, &subjects);
    let authorized = authorize_process(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &profile,
        &evidence,
    )
    .unwrap();

    let mut different = invocation.clone();
    different.invocation_id = "invocation:different".to_owned();
    assert!(matches!(
        Orchestrator::encode_process_request(
            resolved,
            &different,
            &fixture.subject_lock,
            &subjects,
            &authorized,
        ),
        Err(ExecutionError::Preflight { .. })
    ));
}
