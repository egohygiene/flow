use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use flow::optiflow::{
    AdapterError, Capability, CapabilityState, OptiflowReadOnlyAdapter, ReadOnlyReceipt,
    ReceiptStatus,
};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("flow-optiflow-{}-{suffix}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn source(&self) -> PathBuf {
        self.0.join("source 🌿 with spaces")
    }

    fn state(&self) -> PathBuf {
        self.0.join("local state")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn release_executable() -> Option<PathBuf> {
    // The test runs against the independently pinned release when installed.
    // Unit-level refusal checks remain available without a downloaded binary.
    std::env::var_os("FLOW_OPTIFLOW_V011_EXECUTABLE").map(PathBuf::from)
}

#[test]
fn missing_or_incompatible_provider_is_refused() {
    let fixture = Fixture::new();
    assert!(matches!(
        OptiflowReadOnlyAdapter::probe(&fixture.0.join("missing")),
        Err(AdapterError::Unavailable)
    ));
    let impostor = fixture.0.join("optiflow");
    fs::write(&impostor, b"optiflow 0.1.1\n").unwrap();
    assert!(matches!(
        OptiflowReadOnlyAdapter::probe(&impostor),
        Err(AdapterError::Incompatible)
    ));
}

#[test]
fn native_release_inspects_only_synthetic_sources_and_verifies_durable_evidence() {
    let Some(executable) = release_executable() else {
        return;
    };
    let fixture = Fixture::new();
    let root = fixture.source();
    fs::create_dir_all(&root).unwrap();
    let a = root.join("a 🍃.txt");
    let b = root.join("b 🍃.txt");
    let other = root.join("different.txt");
    let bytes = b"same bytes, distinct inodes\n";
    fs::write(&a, bytes).unwrap();
    fs::write(&b, bytes).unwrap();
    fs::write(&other, b"other\n").unwrap();
    let before = [
        fs::read(&a).unwrap(),
        fs::read(&b).unwrap(),
        fs::read(&other).unwrap(),
    ];

    let adapter = OptiflowReadOnlyAdapter::probe(&executable).unwrap();
    for unsupported in [
        Capability::DryRun,
        Capability::Quarantine,
        Capability::Restore,
        Capability::Finalize,
    ] {
        assert_eq!(
            unsupported.state(),
            CapabilityState::UnsupportedByPinnedRelease
        );
        assert!(matches!(
            adapter.require_capability(unsupported),
            Err(AdapterError::UnsupportedCapability(_))
        ));
    }
    let (receipt, path) = adapter.inspect(&root, &fixture.state()).unwrap();
    assert!(matches!(receipt.status, ReceiptStatus::Complete));
    assert_eq!(
        receipt
            .commands
            .iter()
            .map(|command| command.command.as_str())
            .collect::<Vec<_>>(),
        ["scan", "report", "plan"]
    );
    assert_eq!(receipt.duplicate_logical_bytes, Some(bytes.len() as u64));
    assert_eq!(receipt.physical_savings_bytes, None);
    assert_eq!(receipt.approval, None);
    assert_eq!(receipt.recovery, "none");
    assert_eq!(receipt.provider.version, "0.1.1");
    assert_eq!(receipt.provider.interface, "optiflow.command-result.v1");
    assert!(
        receipt
            .plan_sha256
            .as_ref()
            .is_some_and(|digest| digest.len() == 64)
    );
    assert!(
        receipt
            .commands
            .iter()
            .all(|command| command.artifacts.iter().any(|a| a.kind == "artifact_set"))
    );
    assert!(!receipt.needs_recovery());
    assert_eq!(fs::read(&a).unwrap(), before[0]);
    assert_eq!(fs::read(&b).unwrap(), before[1]);
    assert_eq!(fs::read(&other).unwrap(), before[2]);

    let reopened = ReadOnlyReceipt::load(&path).unwrap();
    assert_eq!(reopened.plan_sha256, receipt.plan_sha256);
    // Simulate a crash after the report was committed but before the plan
    // transition. Reopening cannot infer completion from provider files.
    let original_receipt = fs::read(&path).unwrap();
    let mut interrupted = receipt;
    interrupted.status = ReceiptStatus::Planning;
    interrupted.commands.pop();
    interrupted.plan_id = None;
    interrupted.plan_sha256 = None;
    interrupted.recovery = "required-if-interrupted".to_owned();
    fs::write(&path, serde_json::to_vec(&interrupted).unwrap()).unwrap();
    assert!(ReadOnlyReceipt::load(&path).unwrap().needs_recovery());
    fs::write(&path, original_receipt).unwrap();

    let plan_path = reopened.commands[2]
        .artifacts
        .iter()
        .find(|a| a.kind == "plan")
        .unwrap()
        .path
        .clone();
    let plan_path = native_path(&plan_path);
    let mut altered = fs::read(&plan_path).unwrap();
    altered.push(b' ');
    fs::write(&plan_path, altered).unwrap();
    assert!(matches!(
        ReadOnlyReceipt::load(&path),
        Err(AdapterError::InvalidEvidence("receipt artifact changed"))
    ));
}

#[test]
fn source_and_state_overlap_is_refused_before_any_provider_run() {
    let Some(executable) = release_executable() else {
        return;
    };
    let fixture = Fixture::new();
    let root = fixture.source();
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("original"), b"preserve me").unwrap();
    let adapter = OptiflowReadOnlyAdapter::probe(&executable).unwrap();
    let nested = root.join("evidence");
    assert!(matches!(
        adapter.inspect(&root, &nested),
        Err(AdapterError::PathEscape)
    ));
    assert_eq!(fs::read(root.join("original")).unwrap(), b"preserve me");
    assert!(!nested.exists());
}

#[cfg(unix)]
fn native_path(value: &flow::optiflow::NativePath) -> PathBuf {
    match value {
        flow::optiflow::NativePath::Utf8 { value } => PathBuf::from(value),
        flow::optiflow::NativePath::UnixBytes { base64 } => {
            use base64::Engine;
            use std::os::unix::ffi::OsStringExt;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(base64)
                .unwrap();
            Path::new(&std::ffi::OsString::from_vec(bytes)).to_owned()
        }
    }
}
