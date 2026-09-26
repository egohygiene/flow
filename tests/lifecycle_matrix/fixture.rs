use super::*;
use crate::{
    BINDINGS_LOCATOR, CAPABILITIES, INPUT_BYTES, INPUT_LOCATOR, KitFixture, PreparedLifecycleRun,
    TestRoot, WORKSPACE_LOCATOR,
};
use flow::{
    ArtifactBindingSet, ExtensionCatalog, ExtensionLock, ExtensionManifest, ExtensionObservation,
    HostArtifactObservationSet, PlannedStep, ProcessStepContext, RUN_PLAN_V1, RunPlan,
    RunStepContext,
};
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) const IDS: [&str; 3] = ["step:a", "step:b", "step:c"];

pub(super) struct Node {
    pub kit: KitFixture,
    pub prepared: PreparedLifecycleRun,
    artifacts: PathBuf,
}

impl Node {
    fn prepare(kit: KitFixture, mode: &str, index: usize) -> Self {
        let prepared = kit.prepare_lifecycle_named(
            CAPABILITIES[0],
            mode,
            mode == "await-interruption",
            &format!("lifecycle-{index}"),
        );
        let artifacts = kit.root.path().join(WORKSPACE_LOCATOR);
        Self {
            kit,
            prepared,
            artifacts,
        }
    }

    pub fn context(&self) -> ProcessStepContext<'_> {
        ProcessStepContext {
            execution_root: self.kit.root.path(),
            artifact_root: &self.artifacts,
            resolved: self.prepared.resolved(),
            invocation: &self.prepared.invocation,
            subjects: &self.prepared.subject_lock,
            authority: &self.prepared.authority,
            bindings: &self.kit.bindings,
        }
    }

    pub fn preserve_and_replace(&self, relative: &str, bytes: &[u8]) {
        let path = self.kit.root.path().join(relative);
        let backup = self.kit.root.path().join("preserved-original");
        assert!(!backup.exists());
        fs::copy(&path, &backup).unwrap();
        fs::write(path, bytes).unwrap();
    }
}

pub(super) struct Fixture {
    pub nodes: Vec<Node>,
    pub plan: RunPlan,
    mode: String,
}

// Host-local handoff only: paths and fixture configuration never enter receipts.
// Reopening freshly resolves, observes, and authorizes; no opaque tokens are saved.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor {
    mode: String,
    nodes: Vec<NodeDescriptor>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NodeDescriptor {
    root: PathBuf,
    manifest: ExtensionManifest,
    lock: ExtensionLock,
    bindings: ArtifactBindingSet,
    package_digest: String,
    executable_digest: String,
    executable_locator: String,
    grants_digest: String,
    package_observations: HostArtifactObservationSet,
}

impl Fixture {
    pub fn new(mode: &str, graph: bool, bytes: &[u8], name: &str) -> Self {
        let nodes = (0..if graph { 3 } else { 1 })
            .map(|index| {
                let mut kit = KitFixture::new(CAPABILITIES[0], bytes, name);
                kit.bindings.outputs[0].artifact_id = format!("artifact:lifecycle-{index}");
                fs::write(
                    kit.root
                        .path()
                        .join(WORKSPACE_LOCATOR)
                        .join(BINDINGS_LOCATOR),
                    serde_json::to_vec(&kit.bindings).unwrap(),
                )
                .unwrap();
                Node::prepare(kit, mode, index)
            })
            .collect();
        Self::from_nodes(nodes, mode.to_owned())
    }

    fn from_nodes(nodes: Vec<Node>, mode: String) -> Self {
        let plan = RunPlan {
            schema_version: RUN_PLAN_V1.to_owned(),
            plan_id: "plan:lifecycle-matrix".to_owned(),
            run_id: nodes[0].prepared.invocation.run_id.clone(),
            steps: nodes
                .iter()
                .enumerate()
                .map(|(index, node)| {
                    let dependencies = if index == 1 {
                        vec![IDS[0].to_owned()]
                    } else {
                        Vec::new()
                    };
                    PlannedStep::prepare(IDS[index].to_owned(), dependencies, &node.context())
                        .unwrap()
                })
                .collect(),
        };
        plan.validate().unwrap();
        Self { nodes, plan, mode }
    }

    pub fn workspace(&self) -> PathBuf {
        self.nodes[0].kit.root.path().join("lifecycle state café")
    }

    pub fn contexts(&self) -> Vec<RunStepContext<'_>> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| RunStepContext {
                step_id: IDS[index],
                context: node.context(),
            })
            .collect()
    }

    pub fn execute(&self, store: &mut RunStore, index: usize) {
        store
            .execute_in_plan(
                &self.plan,
                IDS[index],
                &self.contexts(),
                &NoSecrets,
                &NeverCancelled,
                &mut Vec::new(),
            )
            .unwrap();
    }

    pub fn assess(&self, store: &RunStore, at: &str) -> Assessment {
        let report = store.assess_run(&self.plan, &self.contexts()).unwrap();
        report.validate_against(store.state()).unwrap();
        Assessment {
            at: at.to_owned(),
            sequence: report.sequence,
            steps: report.steps,
        }
    }

    pub fn child(&self, mode: &str) {
        let descriptor = Descriptor {
            mode: self.mode.clone(),
            nodes: self
                .nodes
                .iter()
                .map(|node| NodeDescriptor {
                    root: node.kit.root.path().to_owned(),
                    manifest: node.kit.manifest.clone(),
                    lock: node.kit.lock.clone(),
                    bindings: node.kit.bindings.clone(),
                    package_digest: node.kit.package_digest.clone(),
                    executable_digest: node.kit.executable_digest.clone(),
                    executable_locator: node.kit.executable_locator.clone(),
                    grants_digest: node.kit.grants_digest.clone(),
                    package_observations: node.kit.package_observations.clone(),
                })
                .collect(),
        };
        let path = self.nodes[0].kit.root.path().join("host-handoff.json");
        fs::write(&path, serde_json::to_vec(&descriptor).unwrap()).unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "lifecycle_matrix::lifecycle_child",
                "--ignored",
                "--nocapture",
            ])
            .env("FLOW_LIFECYCLE_HANDOFF", path)
            .env("FLOW_LIFECYCLE_CHILD_MODE", mode)
            .output()
            .unwrap();
        assert!(output.stdout.len() + output.stderr.len() < 16_384);
        assert_eq!(
            output.status.code(),
            Some(if mode.starts_with("exit-") { 73 } else { 0 }),
            "child {mode}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    pub fn from_handoff(path: &Path) -> Self {
        let descriptor: Descriptor = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        let nodes = descriptor
            .nodes
            .into_iter()
            .enumerate()
            .map(|(index, saved)| {
                let observation = ExtensionObservation::new(
                    saved.manifest.extension_id.clone(),
                    saved.manifest.version.clone(),
                    saved.manifest.publisher.id.clone(),
                    saved.manifest.integrity.clone(),
                    true,
                );
                let catalog = ExtensionCatalog::inspect(
                    [saved.manifest.clone()],
                    saved.lock.clone(),
                    [observation],
                )
                .unwrap();
                let kit = KitFixture {
                    root: TestRoot {
                        path: saved.root,
                        owned: false,
                    },
                    catalog,
                    manifest: saved.manifest,
                    lock: saved.lock,
                    bindings: saved.bindings,
                    package_digest: saved.package_digest,
                    executable_digest: saved.executable_digest,
                    executable_locator: saved.executable_locator,
                    grants_digest: saved.grants_digest,
                    package_observations: saved.package_observations,
                };
                Node::prepare(kit, &descriptor.mode, index)
            })
            .collect();
        Self::from_nodes(nodes, descriptor.mode)
    }

    pub fn identities(&self) -> Vec<Value> {
        self.nodes
            .iter()
            .map(|node| {
                json!({
                    "package_digest": node.kit.package_digest,
                    "executable_digest": node.kit.executable_digest,
                    "manifest_digest": digest_json(&node.kit.manifest),
                    "input_digest": digest_bytes(INPUT_BYTES),
                    "bindings_digest": digest_json(&node.kit.bindings),
                })
            })
            .collect()
    }

    pub fn verify_preservation(&self, recipe: &str) {
        for (index, node) in self.nodes.iter().enumerate() {
            let root = node.kit.root.path();
            let source = if recipe == "changed-input" && index == 0 {
                root.join("preserved-original")
            } else {
                root.join(WORKSPACE_LOCATOR).join(INPUT_LOCATOR)
            };
            assert_eq!(fs::read(source).unwrap(), INPUT_BYTES);
            if recipe != "changed-implementation" || index != 0 {
                assert_eq!(
                    digest_bytes(
                        &fs::read(
                            root.join(crate::PACKAGE_LOCATOR)
                                .join(&node.kit.executable_locator)
                        )
                        .unwrap()
                    ),
                    node.kit.executable_digest
                );
                assert_eq!(
                    fs::read(root.join(crate::PACKAGE_LOCATOR).join("LICENSE")).unwrap(),
                    crate::LICENSE_BYTES
                );
            } else {
                assert_eq!(
                    digest_bytes(&fs::read(root.join("preserved-original")).unwrap()),
                    node.kit.executable_digest
                );
            }
        }
    }
}
