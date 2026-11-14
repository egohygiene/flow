//! Authored source-contract cases, not signature/native qualification evidence.
use super::publisher::*;
use super::*;
#[test]
fn publisher_policy_needs_no_existing_host_or_runtime_witness() {
    let policy:PublisherPolicy=serde_json::from_value(serde_json::json!({"schema_version":"flow.publisher-policy/v1","owners":[{"tool":"aniflow","repository":"egohygiene/aniflow","signer_workflow":"egohygiene/aniflow/.github/workflows/release-candidate.yml","signer_revision":"a".repeat(40),"source_revision":"b".repeat(40),"source_ref":"refs/heads/main"}],"maximum_age_seconds":900})).unwrap();
    // Deserialization accepts the separate publisher contract without creating
    // authority or inventing a local host/native qualification fact.
    assert_eq!(policy.owners.len(), 1);
    assert!(serde_json::from_value::<Policy>(serde_json::to_value(policy).unwrap()).is_err());
}
#[test]
fn pipeline_cannot_supply_publisher_trust_or_verified_authority() {
    let selected = serde_json::json!({"path":{"encoding":"utf8","value":"/synthetic/never-invoked"},"sha256":"a".repeat(64),"byte_count":1});
    let mut request = serde_json::json!({"schema_version":"flow.publisher-verification-request/v1","cohort":selected,"selections":[{"row_id":"synthetic","report":selected,"sigstore_bundle":selected,"archive":selected}],"timeout_ms":1000,"verified":true});
    assert!(publisher::Request::from_json_slice(&serde_json::to_vec(&request).unwrap()).is_err());
    request.as_object_mut().unwrap().remove("verified");
    request["policy"] = selected;
    assert!(publisher::Request::from_json_slice(&serde_json::to_vec(&request).unwrap()).is_err());
}
