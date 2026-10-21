//! Shared, provider-free receipt regressions. Native JSON below is deliberately
//! minimal: it binds identities and local evidence without qualifying a provider.
//! Pinned adapter release profiles support Unix; symlink confinement is exercised
//! on those platforms rather than panicking on unsupported Windows hosts.

#![cfg(unix)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use flow::optiflow::ReadOnlyReceipt;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const FIXTURE: &str = include_str!("fixtures/optiflow-receipt-parity.json");
static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: String,
    description: String,
    base: Value,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    operations: Vec<Operation>,
    schema_valid: bool,
    runtime_valid: bool,
    needs_recovery: Option<bool>,
    file_effect: Option<FileEffect>,
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "lowercase", deny_unknown_fields)]
enum Operation {
    Replace { path: String, value: Value },
    Remove { path: String },
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum FileEffect {
    TamperPlan,
    ArtifactSymlink,
    MarkerSourceSet,
    MarkerMemberDigest,
    PlanIdentity,
}

struct Fixture {
    root: PathBuf,
    source: PathBuf,
    state: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let suffix = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "flow-receipt-contract-{}-{suffix}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let source = root.join("source 🌿 with spaces");
        let state = root.join("local state");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&state).unwrap();
        Self {
            root: fs::canonicalize(root).unwrap(),
            source: fs::canonicalize(source).unwrap(),
            state: fs::canonicalize(state).unwrap(),
        }
    }

    fn expand(&self, value: &mut Value) {
        match value {
            Value::String(text) => {
                *text = text
                    .replace("@STATE@", self.state.to_str().unwrap())
                    .replace("@SOURCE@", self.source.to_str().unwrap());
            }
            Value::Array(items) => items.iter_mut().for_each(|item| self.expand(item)),
            Value::Object(items) => items.values_mut().for_each(|item| self.expand(item)),
            _ => {}
        }
    }

    fn materialize(&self, receipt: &mut Value) {
        self.expand(receipt);
        let run = json!({
            "schema_version": "optiflow.run.v5",
            "run_id": receipt["source_run_id"],
            "artifact_set_id": receipt["source_set_id"],
            "inputs": [self.source],
        });
        let plan = json!({
            "schema_version": "optiflow.plan.v5",
            "plan_id": receipt["plan_id"],
            "source_run_id": receipt["source_run_id"],
            "source_artifact_set_id": receipt["source_set_id"],
            "safety": {"mutates_files": false, "requires_explicit_apply": true},
        });
        for command in receipt["commands"].as_array_mut().unwrap() {
            for artifact in command["artifacts"].as_array_mut().unwrap() {
                let document = match artifact["kind"].as_str().unwrap() {
                    "effective_policy" => json!({"schema": "optiflow.effective-policy.v1"}),
                    "run" => run.clone(),
                    "report" => json!({"schema_version": "optiflow.report.v6", "run": run}),
                    "plan" => plan.clone(),
                    "artifact_set" => continue,
                    other => panic!("unexpected base artifact {other}"),
                };
                write_artifact(artifact, &document);
            }
        }
        write_markers(receipt);
        receipt["plan_sha256"] = receipt["commands"][2]["artifacts"][0]["sha256"].clone();
    }

    fn effect(&self, receipt: &mut Value, effect: &FileEffect) {
        let plan_path = artifact_path(&receipt["commands"][2]["artifacts"][0]);
        match effect {
            FileEffect::TamperPlan => {
                let mut bytes = fs::read(&plan_path).unwrap();
                bytes.push(b' ');
                fs::write(plan_path, bytes).unwrap();
            }
            FileEffect::ArtifactSymlink => self.symlink_artifact(&plan_path),
            FileEffect::MarkerSourceSet | FileEffect::MarkerMemberDigest => {
                let marker = &mut receipt["commands"][2]["artifacts"][1];
                let mut document: Value =
                    serde_json::from_slice(&fs::read(artifact_path(marker)).unwrap()).unwrap();
                match effect {
                    FileEffect::MarkerSourceSet => {
                        document["source_set_id"] = json!("00000000-0000-4000-8000-000000000099");
                    }
                    FileEffect::MarkerMemberDigest => {
                        document["members"][0]["digest"]["value"] = json!("f".repeat(64));
                    }
                    _ => unreachable!(),
                }
                write_artifact(marker, &document);
                receipt["commands"][2]["marker_sha256"] = marker["sha256"].clone();
            }
            FileEffect::PlanIdentity => {
                let artifact = &mut receipt["commands"][2]["artifacts"][0];
                let mut document: Value =
                    serde_json::from_slice(&fs::read(&plan_path).unwrap()).unwrap();
                document["plan_id"] = json!("00000000-0000-4000-8000-000000000099");
                write_artifact(artifact, &document);
                receipt["plan_sha256"] = artifact["sha256"].clone();
                write_markers(receipt);
            }
        }
    }

    fn symlink_artifact(&self, path: &Path) {
        let outside = self.source.join("identical outside evidence.json");
        fs::copy(path, &outside).unwrap();
        fs::remove_file(path).unwrap();
        std::os::unix::fs::symlink(outside, path).unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn artifact_path(artifact: &Value) -> PathBuf {
    PathBuf::from(artifact["path"]["value"].as_str().unwrap())
}

fn write_artifact(artifact: &mut Value, document: &Value) {
    let path = artifact_path(artifact);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let bytes = serde_json::to_vec(document).unwrap();
    fs::write(path, &bytes).unwrap();
    artifact["size_bytes"] = json!(bytes.len());
    artifact["sha256"] = json!(format!("{:x}", Sha256::digest(&bytes)));
    artifact["blake3_256"] = json!(blake3::hash(&bytes).to_hex().as_str());
}

fn write_markers(receipt: &mut Value) {
    let run_id = receipt["source_run_id"].clone();
    let source_set = receipt["source_set_id"].clone();
    for command in receipt["commands"].as_array_mut().unwrap() {
        let is_plan = command["command"] == "plan";
        let members = command["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|artifact| artifact["kind"] != "artifact_set")
            .map(|artifact| {
                let path = artifact_path(artifact);
                json!({
                    "kind": artifact["kind"], "schema": artifact["schema"],
                    "path": {"encoding": "utf8", "value": path.file_name().unwrap().to_str().unwrap()},
                    "size_bytes": artifact["size_bytes"],
                    "digest": {"algorithm": "blake3-256", "value": artifact["blake3_256"]},
                })
            })
            .collect::<Vec<_>>();
        let marker = json!({
            "schema": "optiflow.artifact-set.v1", "set_id": command["set_id"],
            "set_kind": if is_plan { "plan" } else { "scan" },
            "run_id": run_id, "state": "committed",
            "source_set_id": if is_plan { source_set.clone() } else { Value::Null },
            "members": members,
        });
        let artifact = command["artifacts"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|artifact| artifact["kind"] == "artifact_set")
            .unwrap();
        write_artifact(artifact, &marker);
        command["marker_sha256"] = artifact["sha256"].clone();
    }
}

fn apply_operation(document: &mut Value, operation: &Operation, fixture: &Fixture) {
    let (path, replacement) = match operation {
        Operation::Replace { path, value } => {
            let mut value = value.clone();
            fixture.expand(&mut value);
            (path, Some(value))
        }
        Operation::Remove { path } => (path, None),
    };
    assert!(path.starts_with('/'));
    let (parent_path, key) = path.rsplit_once('/').unwrap();
    let key = key.replace("~1", "/").replace("~0", "~");
    let parent = document.pointer_mut(parent_path).unwrap();
    match (parent, replacement) {
        (Value::Array(items), Some(value)) => items[key.parse::<usize>().unwrap()] = value,
        (Value::Array(items), None) => {
            items.remove(key.parse::<usize>().unwrap());
        }
        (Value::Object(items), Some(value)) => {
            *items.get_mut(&key).unwrap() = value;
        }
        (Value::Object(items), None) => {
            items.remove(&key).unwrap();
        }
        _ => panic!("invalid fixture operation {path}"),
    }
}

#[test]
fn shared_receipt_contract_and_evidence_cases() {
    let manifest: Manifest = serde_json::from_str(FIXTURE).unwrap();
    assert_eq!(
        manifest.schema_version,
        "flow.optiflow-receipt-parity-tests/v1"
    );
    assert!(!manifest.description.is_empty());
    let mut identities = BTreeSet::new();
    let mut failures = Vec::new();
    for case in manifest.cases {
        assert!(identities.insert(case.id.clone()), "duplicate {}", case.id);
        assert!(!case.runtime_valid || case.schema_valid, "{}", case.id);
        let fixture = Fixture::new();
        let mut receipt = manifest.base.clone();
        fixture.materialize(&mut receipt);
        for operation in &case.operations {
            apply_operation(&mut receipt, operation, &fixture);
        }
        if let Some(effect) = &case.file_effect {
            fixture.effect(&mut receipt, effect);
        }
        let path = fixture.root.join("receipt.json");
        fs::write(&path, serde_json::to_vec(&receipt).unwrap()).unwrap();
        let result = ReadOnlyReceipt::load(&path);
        match result {
            Ok(receipt) if case.runtime_valid => {
                if let Some(expected) = case.needs_recovery {
                    assert_eq!(receipt.needs_recovery(), expected, "{}", case.id);
                }
            }
            Err(_) if !case.runtime_valid => {}
            result => failures.push(format!("{}: unexpected {result:?}", case.id)),
        }
    }
    assert_eq!(failures, Vec::<String>::new(), "{}", failures.join("\n"));
}
