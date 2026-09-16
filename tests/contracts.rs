use flow::{
    ExtensionEvent, ExtensionInvocation, ExtensionLock, ExtensionManifest, ExtensionResolution,
    ExtensionResult, FailureClassification, Outcome, ValidationEvidence, ValidationStatus,
    parse_flow_version_requirement,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

fn roundtrip_and_validate<T>(source: &str, validate: impl FnOnce(&T))
where
    T: DeserializeOwned + Serialize,
{
    let value: serde_json::Value = serde_json::from_str(source).expect("fixture JSON must parse");
    let typed: T = serde_json::from_value(value.clone()).expect("fixture must deserialize");
    validate(&typed);
    assert_eq!(serde_json::to_value(typed).expect("must serialize"), value);
}

#[test]
fn six_checked_in_examples_roundtrip_and_validate() {
    roundtrip_and_validate::<ExtensionManifest>(
        include_str!("../contracts/examples/extension-manifest.v1.example.json"),
        |value| value.validate().unwrap(),
    );
    roundtrip_and_validate::<ExtensionLock>(
        include_str!("../contracts/examples/extension-lock.v1.example.json"),
        |value| value.validate().unwrap(),
    );
    roundtrip_and_validate::<ExtensionInvocation>(
        include_str!("../contracts/examples/extension-invocation.v1.example.json"),
        |value| value.validate().unwrap(),
    );
    roundtrip_and_validate::<ExtensionEvent>(
        include_str!("../contracts/examples/extension-event.v1.example.json"),
        |value| value.validate().unwrap(),
    );
    roundtrip_and_validate::<ExtensionResult>(
        include_str!("../contracts/examples/extension-result.v1.example.json"),
        |value| value.validate().unwrap(),
    );
    roundtrip_and_validate::<ExtensionResolution>(
        include_str!("../contracts/examples/extension-resolution.v1.example.json"),
        |value| value.validate().unwrap(),
    );
}

#[test]
fn resolution_fixtures_remain_semantically_valid() {
    for source in [
        include_str!("../contracts/fixtures/extensions/incompatible-version.v1.fixture.json"),
        include_str!("../contracts/fixtures/extensions/over-permissioned.v1.fixture.json"),
        include_str!("../contracts/fixtures/extensions/duplicate-conflict.v1.fixture.json"),
        include_str!("../contracts/fixtures/extensions/malformed.v1.fixture.json"),
        include_str!("../contracts/fixtures/extensions/fallback.v1.fixture.json"),
    ] {
        let resolution: ExtensionResolution = serde_json::from_str(source).unwrap();
        resolution.validate().unwrap();
    }
}

#[test]
fn closed_models_reject_unknown_root_and_nested_fields() {
    let source = include_str!("../contracts/examples/extension-manifest.v1.example.json");
    let mut root: serde_json::Value = serde_json::from_str(source).unwrap();
    root.as_object_mut()
        .unwrap()
        .insert("surprise".to_owned(), serde_json::json!(true));
    assert!(serde_json::from_value::<ExtensionManifest>(root).is_err());

    let mut nested: serde_json::Value = serde_json::from_str(source).unwrap();
    nested["publisher"]["surprise"] = serde_json::json!(true);
    assert!(serde_json::from_value::<ExtensionManifest>(nested).is_err());
}

#[test]
fn semver_requirements_require_comma_separated_comparators() {
    assert!(parse_flow_version_requirement(">=0.1.0, <0.2.0").is_ok());
    assert!(parse_flow_version_requirement(">=0.1.0 <0.2.0").is_err());
    assert!(parse_flow_version_requirement("not-a-range").is_err());
}

#[test]
fn manifest_semantics_reject_undeclared_domains_duplicate_ids_and_bad_hooks() {
    let source = include_str!("../contracts/examples/extension-manifest.v1.example.json");
    let mut manifest: ExtensionManifest = serde_json::from_str(source).unwrap();
    manifest.capabilities[0].domain = flow::Domain::ThirdParty;
    assert!(manifest.validate().is_err());

    let mut manifest: ExtensionManifest = serde_json::from_str(source).unwrap();
    manifest.capabilities[1].capability_id = manifest.capabilities[0].capability_id.clone();
    assert!(manifest.validate().is_err());

    let mut manifest: ExtensionManifest = serde_json::from_str(source).unwrap();
    manifest.transform_hooks[0].accepts = vec!["application/mismatch".to_owned()];
    assert!(manifest.validate().is_err());
}

#[test]
fn result_outcome_and_failure_must_be_consistent() {
    let source = include_str!("../contracts/examples/extension-result.v1.example.json");
    let mut result: ExtensionResult = serde_json::from_str(source).unwrap();
    result.failure.classification = FailureClassification::Provider;
    assert!(result.validate().is_err());

    result.outcome = Outcome::Failed;
    result.failure.code = "provider.failed".to_owned();
    result.failure.message = "The provider failed.".to_owned();
    assert!(result.validate().is_ok());
    result.failure.classification = FailureClassification::None;
    assert!(result.validate().is_err());

    for classification in [
        FailureClassification::Unavailable,
        FailureClassification::Cancelled,
        FailureClassification::Unauthorized,
        FailureClassification::Incompatible,
    ] {
        result.failure.classification = classification;
        assert!(result.validate().is_err());
    }
}

#[test]
fn result_failure_payload_matches_its_classification() {
    let source = include_str!("../contracts/examples/extension-result.v1.example.json");
    let mut result: ExtensionResult = serde_json::from_str(source).unwrap();
    result.failure.code = "unexpected".to_owned();
    assert!(result.validate().is_err());

    let mut result: ExtensionResult = serde_json::from_str(source).unwrap();
    result.failure.message = "unexpected".to_owned();
    assert!(result.validate().is_err());

    let mut result: ExtensionResult = serde_json::from_str(source).unwrap();
    result.failure.retryable = true;
    assert!(result.validate().is_err());

    let mut result: ExtensionResult = serde_json::from_str(source).unwrap();
    result.outcome = Outcome::Failed;
    result.failure.classification = FailureClassification::Provider;
    assert!(result.validate().is_err());

    result.failure.code = "provider.failed".to_owned();
    assert!(result.validate().is_err());
    result.failure.code.clear();
    result.failure.message = "The provider failed.".to_owned();
    assert!(result.validate().is_err());
    result.failure.code = "provider.failed".to_owned();
    assert!(result.validate().is_ok());
}

#[test]
fn success_like_result_cannot_contain_failed_validation_evidence() {
    let source = include_str!("../contracts/examples/extension-result.v1.example.json");
    for outcome in [Outcome::Produced, Outcome::Reused, Outcome::Skipped] {
        let mut result: ExtensionResult = serde_json::from_str(source).unwrap();
        result.outcome = outcome;
        result.validations.push(ValidationEvidence {
            validator: "synthetic.adversarial/v1".to_owned(),
            status: ValidationStatus::Failed,
            evidence: "fixture:validation-failed".to_owned(),
        });
        assert!(result.validate().is_err());
    }

    for outcome in [Outcome::Produced, Outcome::Reused] {
        let mut result: ExtensionResult = serde_json::from_str(source).unwrap();
        result.outcome = outcome;
        result.validations[0].status = ValidationStatus::NotRun;
        assert!(result.validate().is_err());
    }

    let mut skipped: ExtensionResult = serde_json::from_str(source).unwrap();
    skipped.outcome = Outcome::Skipped;
    skipped.validations[0].status = ValidationStatus::NotRun;
    assert!(skipped.validate().is_ok());
}

#[test]
fn resolution_evidence_rejects_ambiguous_or_dangling_ids() {
    let source = include_str!("../contracts/examples/extension-resolution.v1.example.json");

    let mut duplicate: ExtensionResolution = serde_json::from_str(source).unwrap();
    duplicate.candidates.push(duplicate.candidates[0].clone());
    assert!(duplicate.validate().is_err());

    let mut dangling: ExtensionResolution = serde_json::from_str(source).unwrap();
    dangling.fallback_order = vec![
        "org.example.unknown".to_owned(),
        dangling.selected_extension_ids[0].clone(),
    ];
    assert!(dangling.validate().is_err());

    let mut non_selected: ExtensionResolution = serde_json::from_str(include_str!(
        "../contracts/fixtures/extensions/over-permissioned.v1.fixture.json"
    ))
    .unwrap();
    non_selected.fallback_order = vec![non_selected.candidates[0].extension_id.clone()];
    assert!(non_selected.validate().is_err());

    let mut wrong_terminal: ExtensionResolution = serde_json::from_str(source).unwrap();
    let mut other = wrong_terminal.candidates[0].clone();
    "org.example.other-adapter".clone_into(&mut other.extension_id);
    other.available = false;
    wrong_terminal.candidates.push(other);
    wrong_terminal.fallback_order = vec![
        wrong_terminal.selected_extension_ids[0].clone(),
        "org.example.other-adapter".to_owned(),
    ];
    assert!(wrong_terminal.validate().is_err());

    let mut singleton: ExtensionResolution = serde_json::from_str(source).unwrap();
    singleton.fallback_order = singleton.selected_extension_ids.clone();
    assert!(singleton.validate().is_err());

    let mut available_predecessor: ExtensionResolution = serde_json::from_str(source).unwrap();
    let mut predecessor = available_predecessor.candidates[0].clone();
    "org.example.available-predecessor".clone_into(&mut predecessor.extension_id);
    predecessor.precedence = 50;
    available_predecessor.candidates.push(predecessor);
    available_predecessor.fallback_order = vec![
        "org.example.available-predecessor".to_owned(),
        available_predecessor.selected_extension_ids[0].clone(),
    ];
    assert!(available_predecessor.validate().is_err());

    let mut increasing: ExtensionResolution = serde_json::from_str(source).unwrap();
    let mut lower_unavailable = increasing.candidates[0].clone();
    "org.example.lower-unavailable".clone_into(&mut lower_unavailable.extension_id);
    lower_unavailable.precedence = 50;
    lower_unavailable.available = false;
    increasing.candidates.push(lower_unavailable);
    increasing.fallback_order = vec![
        "org.example.lower-unavailable".to_owned(),
        increasing.selected_extension_ids[0].clone(),
    ];
    assert!(increasing.validate().is_err());
}
