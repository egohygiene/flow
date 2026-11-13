//! Authored security-contract cases; not executed in the source-only sprint.
use super::*;
fn pin() -> serde_json::Value {
    serde_json::json!({"path":{"encoding":"utf8","value":"/synthetic/not-executed"},"sha256":"a".repeat(64),"byte_count":1})
}
#[test]
fn provider_request_cannot_select_host_trust() {
    let mut value = serde_json::json!({"schema_version":"flow.release-evidence-request/v1","cohort":pin(),"selections":[{"row_id":"synthetic","report":pin(),"sigstore_bundle":pin(),"executable":pin(),"contracts":[]}],"timeout_ms":1000});
    // This never contacts files or invokes a verifier, even when all pins parse.
    value["verifier"] = pin();
    assert!(Request::from_json_slice(&serde_json::to_vec(&value).unwrap()).is_err());
    value.as_object_mut().unwrap().remove("verifier");
    value["policy"] = pin();
    assert!(Request::from_json_slice(&serde_json::to_vec(&value).unwrap()).is_err());
}
#[test]
fn copy_receipt_and_passed_flag_are_not_clean_install_acceptance() {
    let value = serde_json::json!({"schema_version":"flow.suite-installation/v1","root":{"encoding":"utf8","value":"/synthetic"},"candidate_acknowledged":true,"provider_invoked":false,"native_qualified":false,"passed":true});
    assert!(serde_json::from_value::<CleanInstall>(value).is_err());
}
#[test]
fn host_policy_cannot_redirect_a_tool_to_another_owner() {
    let p:Policy=serde_json::from_value(serde_json::json!({"schema_version":"flow.release-evidence-policy/v1","owners":[{"tool":"aniflow","repository":"egohygiene/renderflow","signer_workflow":"egohygiene/renderflow/.github/workflows/release.yml","signer_revision":"a".repeat(40),"source_revision":"a".repeat(40),"source_ref":"refs/heads/main"}],"host_identity":pin(),"maximum_age_seconds":3600})).unwrap();
    assert!(p.validate().is_err());
}
