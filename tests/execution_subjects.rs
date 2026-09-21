mod common;

use std::fs;

use flow::{
    ArtifactObservationError, ContentDigestStatus, CryptographicVerificationStatus, ExecutionError,
    ExecutionModeKind, ExecutionSubjectError, ExecutionSubjectRole, LockEqualityStatus,
    OperatorTrustStatus, Orchestrator, ProcessCompletion, ProcessTranscript,
    PublisherIdentityStatus, TransparencyLogStatus, observe_execution_subjects,
};

#[cfg(unix)]
use common::create_unsupported_node;
use common::{invocation, process_subject_fixture};

#[test]
fn exact_package_and_executable_bytes_produce_stable_separated_evidence() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = common::invocation(resolved);

    let first = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let second = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();

    assert_eq!(first, second);
    assert_eq!(
        first.lock_digest(),
        fixture.subject_lock.canonical_digest().unwrap()
    );
    assert_eq!(
        first.observation_digest(),
        first.evidence().canonical_digest().unwrap()
    );
    assert_eq!(first.evidence().subjects.len(), 2);
    assert_eq!(
        first.evidence().subjects[0].role,
        ExecutionSubjectRole::Package
    );
    assert_eq!(
        first.evidence().subjects[1].role,
        ExecutionSubjectRole::Executable
    );
    assert!(
        first.evidence().subjects[0]
            .manifest
            .iter()
            .any(|entry| entry.locator == "説明.txt")
    );

    let claims = &first.evidence().claims;
    assert_eq!(claims.content_digest.status, ContentDigestStatus::Observed);
    assert_eq!(claims.lock_equality.status, LockEqualityStatus::Matched);
    assert_eq!(
        claims.cryptographic_verification.status,
        CryptographicVerificationStatus::NotPerformed
    );
    assert!(claims.cryptographic_verification.evidence.is_empty());
    assert_eq!(
        claims.publisher_identity.status,
        PublisherIdentityStatus::DeclaredAndCorrelated
    );
    assert_eq!(
        claims.operator_trust.status,
        OperatorTrustStatus::Configured
    );
    assert_eq!(claims.operator_trust.trust, resolved.trust());
    assert_eq!(
        claims.transparency_log.status,
        TransparencyLogStatus::NotChecked
    );
    assert!(claims.transparency_log.evidence.is_empty());
}

#[test]
fn altered_package_or_executable_bytes_never_match_the_lock() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = common::invocation(resolved);

    fs::write(
        fixture
            .root
            .path()
            .join("packages/synthetic adapter/説明.txt"),
        b"altered package bytes\n",
    )
    .unwrap();
    let package_error = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap_err();
    assert!(matches!(
        package_error,
        ExecutionSubjectError::ContentMismatch { ref subject_id }
            if subject_id == &fixture.subject_lock.package.subject_id
    ));

    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = common::invocation(resolved);
    fs::write(
        fixture.root.executable_path(),
        b"altered executable bytes\n",
    )
    .unwrap();
    let executable_error = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap_err();
    assert!(matches!(
        executable_error,
        ExecutionSubjectError::ContentMismatch { ref subject_id }
            if subject_id == &fixture.subject_lock.executable.subject_id
    ));
}

#[test]
fn provider_version_publisher_capability_interface_and_entrypoint_must_match() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);

    let mut variants = Vec::new();
    let mut wrong_provider = fixture.subject_lock.clone();
    wrong_provider.extension.extension_id = "org.example.other-adapter".to_owned();
    variants.push(wrong_provider);
    let mut wrong_version = fixture.subject_lock.clone();
    wrong_version.extension.version = "0.2.0".to_owned();
    variants.push(wrong_version);
    let mut wrong_publisher = fixture.subject_lock.clone();
    wrong_publisher.extension.publisher_id = "org.example".to_owned();
    variants.push(wrong_publisher);
    let mut wrong_capability = fixture.subject_lock.clone();
    wrong_capability.capability_id = "renderflow/inspect-collection".to_owned();
    variants.push(wrong_capability);
    let mut wrong_interface = fixture.subject_lock.clone();
    wrong_interface.interface.name = "other-process".to_owned();
    variants.push(wrong_interface);
    let mut wrong_entrypoint = fixture.subject_lock.clone();
    wrong_entrypoint.declared_entrypoint = "other-adapter".to_owned();
    variants.push(wrong_entrypoint);

    for variant in variants {
        let error =
            observe_execution_subjects(fixture.root.path(), resolved, &invocation, &variant)
                .unwrap_err();
        assert!(matches!(
            error,
            ExecutionSubjectError::ContextMismatch { .. }
        ));
    }

    let mut wrong_mode = fixture.subject_lock.clone();
    wrong_mode.interface.kind = ExecutionModeKind::InProcess;
    assert!(matches!(
        observe_execution_subjects(fixture.root.path(), resolved, &invocation, &wrong_mode,),
        Err(ExecutionSubjectError::InvalidLock { .. })
    ));
}

#[test]
fn missing_escaped_and_kind_mismatched_subjects_fail_closed() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);

    let mut escaped = fixture.subject_lock.clone();
    escaped.executable.locator = "../outside".to_owned();
    assert!(matches!(
        observe_execution_subjects(fixture.root.path(), resolved, &invocation, &escaped,),
        Err(ExecutionSubjectError::InvalidLock { .. })
    ));

    fs::remove_file(fixture.root.executable_path()).unwrap();
    let missing = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap_err();
    assert!(matches!(
        missing,
        ExecutionSubjectError::Observation {
            source: ArtifactObservationError::Missing { .. }
        }
    ));

    fs::create_dir(fixture.root.executable_path()).unwrap();
    let wrong_kind = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap_err();
    assert!(matches!(
        wrong_kind,
        ExecutionSubjectError::Observation {
            source: ArtifactObservationError::KindMismatch { .. }
        }
    ));
}

#[test]
fn duplicate_contradictory_and_unsupported_portable_evidence_is_rejected() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let matched = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();

    let mut duplicate = matched.evidence().clone();
    duplicate.subjects.push(duplicate.subjects[0].clone());
    assert!(duplicate.validate().is_err());

    let mut reversed = matched.evidence().clone();
    reversed.subjects.swap(0, 1);
    assert!(reversed.validate().is_err());

    let mut contradictory = matched.evidence().clone();
    let executable_digest = contradictory.subjects[1].digest.value.clone();
    let package = &mut contradictory.subjects[0];
    let entry = package
        .manifest
        .iter_mut()
        .find(|entry| entry.locator == fixture.subject_lock.executable.locator)
        .unwrap();
    entry.digest = if executable_digest.starts_with('a') {
        "b".repeat(64)
    } else {
        "a".repeat(64)
    };
    assert!(contradictory.validate().is_err());

    let mut contradictory = matched.evidence().clone();
    contradictory.extension.integrity = "f".repeat(64);
    assert!(contradictory.validate().is_err());

    let mut unsupported = matched.evidence().clone();
    unsupported
        .claims
        .cryptographic_verification
        .evidence
        .push(serde_json::json!({"kind": "fabricated-signature"}));
    assert!(unsupported.validate().is_err());

    let mut unsupported_json = serde_json::to_value(matched.evidence()).unwrap();
    unsupported_json["claims"]["transparency_log"]["status"] = serde_json::json!("verified");
    assert!(
        serde_json::from_value::<flow::HostExecutionSubjectObservationSet>(unsupported_json)
            .is_err()
    );
}

#[test]
fn matched_subject_tokens_are_bound_to_the_exact_lock_and_invocation() {
    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    let matched = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();

    let mut different_invocation = invocation.clone();
    different_invocation.invocation_id = "invocation:different".to_owned();
    let error = Orchestrator::encode_process_request(
        resolved,
        &different_invocation,
        &fixture.subject_lock,
        &matched,
    )
    .unwrap_err();
    assert!(matches!(error, ExecutionError::Preflight { .. }));

    let mut different_run = invocation.clone();
    different_run.run_id = "run:different".to_owned();
    let error = Orchestrator::validate_process_transcript(
        resolved,
        &different_run,
        &fixture.subject_lock,
        &matched,
        ProcessTranscript::new(ProcessCompletion::Exited { code: Some(0) }, &[], &[]),
        &mut Vec::new(),
    )
    .unwrap_err();
    assert!(matches!(error, ExecutionError::Preflight { .. }));

    let mut different_lock = fixture.subject_lock.clone();
    different_lock.subject_lock_id = "subject-lock:different".to_owned();
    let error =
        Orchestrator::encode_process_request(resolved, &invocation, &different_lock, &matched)
            .unwrap_err();
    assert!(matches!(error, ExecutionError::Preflight { .. }));
}

#[cfg(unix)]
#[test]
fn symlinks_and_special_nodes_in_packages_are_rejected() {
    use std::os::unix::fs::symlink;

    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = invocation(resolved);
    fs::remove_file(fixture.root.executable_path()).unwrap();
    symlink("説明.txt", fixture.root.executable_path()).unwrap();
    let error = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ExecutionSubjectError::Observation {
            source: ArtifactObservationError::Symlink { .. }
        }
    ));

    let fixture = process_subject_fixture();
    let outcome = fixture.catalog.resolve(&fixture.request);
    let resolved = outcome.resolved().unwrap();
    let invocation = common::invocation(resolved);
    create_unsupported_node(
        &fixture
            .root
            .path()
            .join("packages/synthetic adapter/provider.fifo"),
    );
    let error = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ExecutionSubjectError::Observation {
            source: ArtifactObservationError::UnsupportedNode { .. }
        }
    ));
}
