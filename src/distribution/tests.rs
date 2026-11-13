//! Synthetic file ownership cases, authored and unexecuted in this sprint.
use super::*;
use crate::NeverCancelled;
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let parent = std::env::temp_dir().canonicalize().unwrap();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = parent.join(format!("flow-install-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn request(&self) -> Request {
        let source = self.0.join("synthetic-source");
        fs::write(&source, b"synthetic bytes; never executed").unwrap();
        let executable =
            FilePin::observe(&NativePath::from_path(&source), MAX_FILE, &NeverCancelled).unwrap();
        let target = native::host_target().unwrap().to_owned();
        let cohort = self.0.join("cohort.json");
        let value = serde_json::json!({"schema_version":"flow.release-cohort/v1","id":"synthetic-install","flow_version":env!("CARGO_PKG_VERSION"),"targets":[target],"retained_rows":["flow.synthetic"],"rows":[{"id":"flow.synthetic","tool":"flow","interface":"standalone_cli","target":target,"profile":"synthetic","input":"bytes","output":"bytes","effects":["read_originals"],"implementation":"implemented","artifact":{"version":"0.1.0","source_revision":"a".repeat(40),"name":"synthetic","sha256":executable.sha256,"byte_count":executable.byte_count,"release_url":"https://github.com/egohygiene/flow/releases/download/v0.1.0/synthetic"},"contracts":[],"requirements":[],"owner_issue":"https://github.com/egohygiene/flow/issues/71","next_action":"synthetic ownership case only"}]});
        fs::write(&cohort, serde_json::to_vec(&value).unwrap()).unwrap();
        Request {
            schema_version: REQUEST_SCHEMA.into(),
            cohort: FilePin::observe(&NativePath::from_path(&cohort), 1_048_576, &NeverCancelled)
                .unwrap(),
            target,
            install_directory: NativePath::from_path(&self.0.join("installed")),
            tools: vec![SelectedTool {
                tool: Tool::Flow,
                row_id: "flow.synthetic".into(),
                executable,
                contracts: vec![],
                extras: vec![],
            }],
            timeout_ms: 30_000,
            maximum_total_bytes: 1_048_576,
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[cfg(unix)]
#[test]
fn offline_bundle_rebases_only_owned_members_after_move() {
    let fixture = Fixture::new();
    let request = fixture.request();
    let bundle_root = fixture.0.join("bundle");
    let receipt = bundle(&request, &bundle_root, &NeverCancelled).unwrap();
    let moved = fixture.0.join("moved");
    fs::rename(&bundle_root, &moved).unwrap();
    let restored = open_bundle(
        &moved,
        &receipt.sha256,
        &fixture.0.join("installed"),
        &NeverCancelled,
    )
    .unwrap();
    let installed = install(&restored, true, &NeverCancelled).unwrap();
    assert!(installed.current_copied_bytes);
    assert!(
        !installed.receipt.native_qualified
            && !installed.receipt.provider_invoked
            && !installed.receipt.publisher_authenticated
    );
    assert_eq!(
        fs::read(path(&request.tools[0].executable.path).unwrap()).unwrap(),
        b"synthetic bytes; never executed"
    );
}
#[cfg(unix)]
#[test]
fn unknown_material_refuses_uninstall_before_any_copy_is_removed() {
    let fixture = Fixture::new();
    let request = fixture.request();
    let installed = install(&request, true, &NeverCancelled).unwrap();
    let root = path(&request.install_directory).unwrap();
    let extra = root.join("user-note.txt");
    fs::write(&extra, b"retain me").unwrap();
    let record = fixture.0.join("removal.json");
    assert!(uninstall(&root, &installed.receipt_sha256, &record, &NeverCancelled).is_err());
    assert!(!record.exists());
    assert_eq!(fs::read(extra).unwrap(), b"retain me");
    assert!(root.join("bin/flow").exists());
}
#[cfg(unix)]
#[test]
fn changed_copies_and_stale_selection_cannot_be_accepted() {
    let fixture = Fixture::new();
    let request = fixture.request();
    let installed = install(&request, true, &NeverCancelled).unwrap();
    let root = path(&request.install_directory).unwrap();
    let pointer = fixture.0.join("current.json");
    let first = activate(
        &root,
        &installed.receipt_sha256,
        &pointer,
        None,
        &NeverCancelled,
    )
    .unwrap();
    assert!(
        activate(
            &root,
            &installed.receipt_sha256,
            &pointer,
            Some(&"0".repeat(64)),
            &NeverCancelled
        )
        .is_err()
    );
    assert_eq!(hash(&fs::read(&pointer).unwrap()), first.sha256);
    fs::write(root.join("bin/flow"), b"changed").unwrap();
    assert!(status(&root, &installed.receipt_sha256, &NeverCancelled).is_err());
    assert_eq!(
        fs::read(path(&request.tools[0].executable.path).unwrap()).unwrap(),
        b"synthetic bytes; never executed"
    );
}
