use flow::{
    ExecutionTier, NetworkMode, ScenarioEvidenceState, ScenarioManifest, ScenarioTerminalState,
};
use sha2::{Digest, Sha256};

const EXAMPLE: &str = include_str!("../contracts/examples/scenario-manifest.v1.example.json");
const INTERRUPTED: &str =
    include_str!("../contracts/fixtures/scenarios/interrupted.v1.fixture.json");
const MULTI_PROVIDER: &str =
    include_str!("../contracts/fixtures/scenarios/multi-provider.v1.fixture.json");
const OBSERVED_EMPTY: &str =
    include_str!("../contracts/fixtures/scenarios/observed-empty.v1.fixture.json");
const UNAVAILABLE: &str =
    include_str!("../contracts/fixtures/scenarios/unavailable.v1.fixture.json");

fn manifest(source: &str) -> ScenarioManifest {
    serde_json::from_str(source).expect("scenario fixture must deserialize")
}

fn assert_rejected(source: &str) {
    let value: serde_json::Value = serde_json::from_str(source).expect("fixture JSON must parse");
    if let Ok(manifest) = serde_json::from_value::<ScenarioManifest>(value) {
        assert!(manifest.validate().is_err(), "invalid scenario was accepted");
    }
}

#[test]
fn positive_scenarios_roundtrip_and_validate() {
    for source in [
        EXAMPLE,
        INTERRUPTED,
        MULTI_PROVIDER,
        OBSERVED_EMPTY,
        UNAVAILABLE,
    ] {
        let value: serde_json::Value = serde_json::from_str(source).unwrap();
        let manifest: ScenarioManifest = serde_json::from_value(value.clone()).unwrap();
        manifest.validate().unwrap();
        assert_eq!(serde_json::to_value(manifest).unwrap(), value);
    }
}

#[test]
fn invalid_scenarios_are_rejected() {
    for source in [
        include_str!("../contracts/fixtures/scenarios/unknown-version.v1.invalid.json"),
        include_str!("../contracts/fixtures/scenarios/missing-identity.v1.invalid.json"),
        include_str!(
            "../contracts/fixtures/scenarios/contradictory-expectation.v1.invalid.json"
        ),
        include_str!("../contracts/fixtures/scenarios/mutable-reference.v1.invalid.json"),
        include_str!("../contracts/fixtures/scenarios/invalid-budget.v1.invalid.json"),
    ] {
        assert_rejected(source);
    }
}

#[test]
fn checked_in_input_bytes_match_their_immutable_source_identity() {
    for source in [
        EXAMPLE,
        INTERRUPTED,
        MULTI_PROVIDER,
        OBSERVED_EMPTY,
        UNAVAILABLE,
    ] {
        let manifest = manifest(source);
        for input in manifest.inputs {
            let relative = input
                .source
                .locator
                .strip_prefix("contracts/")
                .expect("checked-in scenario inputs must use contract-local sources");
            let bytes = std::fs::read(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("contracts")
                    .join(relative),
            )
            .expect("checked-in source must be readable");
            let actual = format!("{:x}", Sha256::digest(bytes));
            assert_eq!(input.source.digest, actual);
            assert_eq!(input.source.revision, format!("sha256:{actual}"));
        }
    }
}

#[test]
fn explicit_null_provenance_and_out_of_range_versions_are_rejected() {
    let mut value: serde_json::Value = serde_json::from_str(EXAMPLE).unwrap();
    value["inputs"][0]["source"]["generator"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<ScenarioManifest>(value).is_err());

    let mut scenario = manifest(EXAMPLE);
    scenario.fixture_version = "18446744073709551616.0.0".to_owned();
    assert!(scenario.validate().is_err());
}

#[test]
fn canonical_digests_match_the_checked_in_cross_language_catalog() {
    for (source, expected) in [
        (
            EXAMPLE,
            "182708d91965c9e3b2d911eef42e5f200907dac380a0ae4eb0f9a1c15b9ab7cd",
        ),
        (
            INTERRUPTED,
            "8ec6ced28b6faa2f4869d592047f6ef76340b15c9c856ddfdc4a375cd66291c1",
        ),
        (
            MULTI_PROVIDER,
            "cbf8a6e6b436a20438661abb9f0b17206e720b08027f7efc1cd3c19b21493e2c",
        ),
        (
            OBSERVED_EMPTY,
            "e90ff010f4b3ed38c11cbcf59916dd584bbdb6bd6237ac319d3c5529c7c146ff",
        ),
        (
            UNAVAILABLE,
            "476a72b94f7cf0f507d2eca73c1d23d4d08c29a021cc118eae0f14b373300a6d",
        ),
    ] {
        assert_eq!(manifest(source).canonical_sha256().unwrap(), expected);
    }
}

#[test]
fn canonicalization_normalizes_only_set_like_collections() {
    let original = manifest(MULTI_PROVIDER);
    let expected = original.canonical_sha256().unwrap();
    let mut reordered = original.clone();
    reordered.tags.reverse();
    reordered.inputs.reverse();
    reordered.providers.reverse();
    for provider in &mut reordered.providers {
        provider.required_capabilities.reverse();
    }
    for stage in &mut reordered.stages {
        stage.depends_on.reverse();
        stage.consumes.reverse();
        stage.produces.reverse();
    }
    reordered.expectation.expected_artifacts.reverse();
    reordered.expectation.expected_diagnostics.reverse();
    reordered.expectation.evidence_refs.reverse();
    reordered.expectation.state_trace_refs.reverse();
    reordered.execution.external_services.reverse();
    reordered.coverage.covered_behaviors.reverse();
    reordered.coverage.known_gaps.reverse();
    assert_eq!(reordered.canonical_sha256().unwrap(), expected);

    let mut reordered_stages = original;
    reordered_stages.stages.reverse();
    assert!(reordered_stages.canonical_sha256().is_err());

    let mut independent = manifest(EXAMPLE);
    let mut second = independent.stages[0].clone();
    second.stage_id = "inspect-secondary".to_owned();
    second.produces = vec!["secondary-report".to_owned()];
    independent.stages.push(second);
    let first_order = independent.canonical_sha256().unwrap();
    independent.stages.reverse();
    assert!(independent.validate().is_ok());
    assert_ne!(independent.canonical_sha256().unwrap(), first_order);
}

#[test]
fn topology_requires_declared_providers_and_explicit_artifact_dependencies() {
    let mut unknown_provider = manifest(EXAMPLE);
    unknown_provider.stages[0].provider_id = "org.egohygiene.unknown-provider".to_owned();
    assert!(unknown_provider.validate().is_err());

    let mut missing_dependency = manifest(MULTI_PROVIDER);
    let consuming_stage = missing_dependency
        .stages
        .iter_mut()
        .find(|stage| !stage.depends_on.is_empty())
        .expect("multi-provider fixture must contain a dependent stage");
    consuming_stage.depends_on.clear();
    assert!(missing_dependency.validate().is_err());
}

#[test]
fn identifiers_match_the_closed_schema_domain() {
    let mut scenario = manifest(EXAMPLE);
    scenario.providers[0].provider_id = "1.invalid".to_owned();
    scenario.stages[0].provider_id = "1.invalid".to_owned();
    assert!(scenario.validate().is_err());

    let mut scenario = manifest(EXAMPLE);
    scenario.providers[0].required_capabilities = vec!["a/b".to_owned()];
    scenario.stages[0].capability_id = "a/b".to_owned();
    assert!(scenario.validate().is_err());

    let mut scenario = manifest(EXAMPLE);
    scenario.stages[0].produces = vec!["!!!".to_owned()];
    assert!(scenario.validate().is_err());
}

#[test]
fn evidence_states_remain_distinct_and_coherent() {
    let mut scenario = manifest(OBSERVED_EMPTY);
    assert_eq!(
        scenario.expectation.evidence_state,
        ScenarioEvidenceState::ObservedEmpty
    );
    scenario.expectation.evidence_state = ScenarioEvidenceState::Unavailable;
    assert!(scenario.validate().is_err());

    scenario.expectation.terminal_state = ScenarioTerminalState::Unavailable;
    assert!(scenario.validate().is_ok());

    scenario.expectation.evidence_state = ScenarioEvidenceState::Incomplete;
    assert!(scenario.validate().is_err());
    scenario.expectation.terminal_state = ScenarioTerminalState::Interrupted;
    assert!(scenario.validate().is_ok());

    scenario.expectation.evidence_state = ScenarioEvidenceState::Unsupported;
    assert!(scenario.validate().is_err());
    scenario.expectation.terminal_state = ScenarioTerminalState::Unsupported;
    assert!(scenario.validate().is_ok());
}

#[test]
fn pull_request_scenarios_are_hermetic_and_budgets_are_nonzero() {
    let mut scenario = manifest(EXAMPLE);
    scenario.execution.clean_room = false;
    assert!(scenario.validate().is_err());

    let mut scenario = manifest(EXAMPLE);
    scenario.execution.network_mode = NetworkMode::Allowlisted;
    scenario.execution.external_services = vec!["example.invalid".to_owned()];
    assert!(scenario.validate().is_err());

    let mut scenario = manifest(EXAMPLE);
    scenario.execution.budget.max_artifacts = 0;
    assert!(scenario.validate().is_err());

    let mut scheduled = manifest(EXAMPLE);
    scheduled.execution.tier = ExecutionTier::Scheduled;
    scheduled.execution.clean_room = false;
    scheduled.execution.network_mode = NetworkMode::Allowlisted;
    scheduled.execution.external_services = vec!["fixtures.example.invalid".to_owned()];
    assert!(scheduled.validate().is_ok());

    scheduled.execution.tier = ExecutionTier::Release;
    assert!(scheduled.validate().is_ok());
}
