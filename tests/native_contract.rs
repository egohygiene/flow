//! Authored cases only. No native/tool execution required by these model cases.
use flow::native::{Limits, Request};
use serde_json::json;
fn request() -> serde_json::Value {
    let path = |p: &str| flow::optiflow::NativePath::from_path(std::path::Path::new(p));
    json!({"schema_version":"flow.native-request/v1", "cohort":path("/private/cohort.json"), "cohort_sha256":"a".repeat(64), "row_id":"aniflow.source",
        "provider_executable":path("/private/aniflow"), "release_descriptor":path("/private/contract.json"),
        "inputs":[path("/private/source 曲.mkv")], "configuration":path("/private/recipe.json"),
        "run_directory":path("/private/run"), "tool_directories":[], "limits":Limits::default()})
}
#[test]
fn request_cannot_widen_or_duplicate_source_selection() {
    let mut value = request();
    assert!(Request::from_json_slice(&serde_json::to_vec(&value).unwrap()).is_ok());
    let duplicate = value["inputs"][0].clone();
    value["inputs"].as_array_mut().unwrap().push(duplicate);
    assert!(Request::from_json_slice(&serde_json::to_vec(&value).unwrap()).is_err());
}
#[test]
fn unknown_authority_and_unbounded_deadline_are_refused() {
    let mut value = request();
    assert!(Request::from_json_slice(&serde_json::to_vec(&value).unwrap()).is_ok());
    value["network_granted"] = true.into();
    assert!(Request::from_json_slice(&serde_json::to_vec(&value).unwrap()).is_err());
    value.as_object_mut().unwrap().remove("network_granted");
    value["limits"]["timeout_ms"] = 86_400_001_u64.into();
    assert!(Request::from_json_slice(&serde_json::to_vec(&value).unwrap()).is_err());
}
