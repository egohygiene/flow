//! Compiled release interfaces; no provider/filesystem/environment observation.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
pub const RELEASE_CONTRACT_V1: &str = "flow.release-contract/v1";
pub fn describe() -> Value {
    let pin = |name: &str, bytes: &[u8]| json!({"identifier":name,"sha256":format!("{:x}",Sha256::digest(bytes))});
    json!({"schema_version":RELEASE_CONTRACT_V1,"tool":"flow","version":env!("CARGO_PKG_VERSION"),
        "library_crate":"flow","library_version":env!("CARGO_PKG_VERSION"),"registry_channel":"unavailable",
        "primary_binary":"flow","commands":crate::command::delivery::COMMANDS,"excluded_release_binaries":["flow-hermetic-provider"],
        "source_candidate_routes":crate::native::SOURCE_ROUTES,
        "released_optiflow_version":crate::optiflow::PROVIDER_VERSION,
        "contracts":[
            pin("flow.release-coverage-request/v1",include_bytes!("../schemas/release-coverage-request-v1.schema.json")),
            pin("flow.release-coverage/v1",include_bytes!("../schemas/release-coverage-v1.schema.json")),
            pin("flow.command-result/v1",include_bytes!("../schemas/command-result-v1.schema.json")),
            pin("flow.release-cohort/v1",include_bytes!("../schemas/flow-release-cohort-v1.schema.json")),
            pin("flow.release-matrix/v1",include_bytes!("../schemas/flow-release-matrix-v1.schema.json")),
            pin("flow.native-request/v1",include_bytes!("../schemas/flow-native-request-v1.schema.json")),
            pin("flow.publisher-verification-request/v1",include_bytes!("../schemas/publisher-verification-request-v1.schema.json")),
            pin("flow.delivery-request/v1",include_bytes!("../schemas/delivery-request-v1.schema.json")),
            pin("flow.adjusted-delivery-lineage/v1",include_bytes!("../schemas/adjusted-delivery-lineage-v1.schema.json")),
            pin("flow.delivery-review/v1",include_bytes!("../schemas/delivery-review-v1.schema.json")),
            pin("flow.audio-readiness-request/v1",include_bytes!("../schemas/audio-readiness-request-v1.schema.json")),
            pin("flow.audio-readiness-summary/v1",include_bytes!("../schemas/audio-readiness-summary-v1.schema.json")),
            pin("flow.audio-check-setup/v1",include_bytes!("../schemas/audio-check-setup-v1.schema.json")),
            pin("flow.audio-check-approvals/v1",include_bytes!("../schemas/audio-check-approvals-v1.schema.json")),
            pin("flow.publishing-readiness-request/v1",include_bytes!("../schemas/publishing-readiness-request-v1.schema.json")),
            pin("flow.publishing-readiness/v1",include_bytes!("../schemas/publishing-readiness-v1.schema.json")),
            pin("flow.released-pilot-approval/v1",include_bytes!("../schemas/released-pilot-approval-v1.schema.json")),
            pin("flow.publishing-project/v1",include_bytes!("../schemas/publishing-project-v1.schema.json")),
            pin("flow.static-publication-project/v1",include_bytes!("../schemas/static-publication-project-v1.schema.json")),
            pin("flow.ai-preparation-profile/v1",include_bytes!("../schemas/ai-preparation-profile-v1.schema.json")),
            pin("flow.ai-local-setup/v1",include_bytes!("../schemas/ai-local-setup-v1.schema.json")),
            pin("flow.ai-local-preparation/v1",include_bytes!("../schemas/ai-local-preparation-v1.schema.json")),
            pin("flow.ai-local-doctor/v1",include_bytes!("../schemas/ai-local-doctor-v1.schema.json")),
            pin("flow.ai-setup-command/v1",include_bytes!("../schemas/ai-setup-command-v1.schema.json")),
            pin("flow.publisher-artifact-report/v1",include_bytes!("../schemas/publisher-artifact-report-v1.schema.json")),
            pin("flow.release-evidence-request/v1",include_bytes!("../schemas/release-evidence-request-v1.schema.json")),
            pin("flow.released-native-approval/v1",include_bytes!("../schemas/released-native-approval-v1.schema.json"))],
        "target_objectives":["x86_64-unknown-linux-gnu","aarch64-apple-darwin","x86_64-apple-darwin","x86_64-pc-windows-msvc"],
        "qualification":"not_established_by_contract_description"})
}
