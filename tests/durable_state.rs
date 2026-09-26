use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use flow::{RunInspectionStatus, RunPlan, RunState, RunStepStatus, RunStore, StateError};
use serde_json::Value;

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);
// Avoid inheriting unrelated test-owned locks during subprocess fork/exec.
// The explicit contention tests still overlap two opens of the same workspace.
static FILESYSTEM_TEST: Mutex<()> = Mutex::new(());

struct Root {
    path: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

impl Root {
    fn new() -> Self {
        let guard = FILESYSTEM_TEST
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let id = NEXT_ROOT.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "flow durable state 説明 {} {id}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self {
            path,
            _guard: guard,
        }
    }

    fn workspace(&self) -> PathBuf {
        self.path.join("run state")
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _result = fs::remove_dir_all(&self.path);
    }
}

fn plan() -> RunPlan {
    serde_json::from_str(include_str!(
        "../contracts/examples/run-plan.v1.example.json"
    ))
    .unwrap()
}

fn state_files(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files: Vec<_> = fs::read_dir(root)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (
                entry.file_name().into_string().unwrap(),
                fs::read(entry.path()).unwrap(),
            )
        })
        .collect();
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
}

#[test]
fn deterministic_reopen_and_cancellation_preserve_intent_and_old_snapshots() {
    let root = Root::new();
    let initial = RunState::new(plan()).unwrap();
    let mut store = RunStore::create(&root.workspace(), plan()).unwrap();
    assert_eq!(store.state(), &initial);
    let before = state_files(&root.workspace());
    store.cancel_pending("step:inspect").unwrap();
    assert_eq!(store.state().steps[0].attempt, 0);
    assert_eq!(store.state().steps[0].status, RunStepStatus::Cancelled);
    assert_eq!(
        store.state().inspection_status(),
        RunInspectionStatus::Stopped
    );
    assert!(store.state().authority_decisions.is_empty());
    let expected = store.state().clone();
    drop(store);
    let reopened = RunStore::open(&root.workspace()).unwrap();
    assert_eq!(reopened.state(), &expected);
    assert!(
        before
            .iter()
            .all(|item| state_files(&root.workspace()).contains(item))
    );
    drop(reopened);
    let unchanged = state_files(&root.workspace());
    drop(RunStore::open(&root.workspace()).unwrap());
    assert_eq!(
        state_files(&root.workspace()),
        unchanged,
        "inspection must not rewrite history"
    );
}

#[test]
fn same_process_concurrent_open_is_refused_and_drop_releases_lock() {
    let root = Root::new();
    let store = RunStore::create(&root.workspace(), plan()).unwrap();
    assert!(matches!(
        RunStore::open(&root.workspace()),
        Err(StateError::Busy)
    ));
    assert!(matches!(
        RunStore::create(&root.workspace(), plan()),
        Err(StateError::AlreadyExists)
    ));
    drop(store);
    drop(RunStore::open(&root.workspace()).unwrap());
}

#[test]
fn denial_is_durable_and_never_records_granted_authority() {
    let root = Root::new();
    let mut store = RunStore::create(&root.workspace(), plan()).unwrap();
    store.deny_pending("step:inspect").unwrap();
    drop(store);
    let store = RunStore::open(&root.workspace()).unwrap();
    assert_eq!(store.state().steps[0].status, RunStepStatus::Denied);
    assert_eq!(store.state().authority_decisions.len(), 1);
    assert!(!store.state().authority_decisions[0].granted);
    assert_eq!(store.state().steps[0].attempt, 0);
}

#[test]
fn oversized_records_are_refused_before_deserialization() {
    let root = Root::new();
    drop(RunStore::create(&root.workspace(), plan()).unwrap());
    fs::OpenOptions::new()
        .write(true)
        .open(root.workspace().join("00000000000000000000.json"))
        .unwrap()
        .set_len(flow::MAX_RUN_RECORD_BYTES + 1)
        .unwrap();
    assert!(matches!(
        RunStore::open(&root.workspace()),
        Err(StateError::Limit)
    ));
}

#[test]
fn another_process_holds_the_lock_and_crash_releases_it() {
    let root = Root::new();
    drop(RunStore::create(&root.workspace(), plan()).unwrap());
    let mut child = child_command(&root.workspace(), "hold")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    let reader = BufReader::new(child.stdout.take().unwrap());
    let ready = reader
        .lines()
        .map(Result::unwrap)
        .any(|line| line == "FLOW_STATE_CHILD_READY");
    assert!(ready);
    assert!(matches!(
        RunStore::open(&root.workspace()),
        Err(StateError::Busy)
    ));
    child.kill().unwrap();
    let _status = child.wait().unwrap();
    let reopened = RunStore::open(&root.workspace()).unwrap();
    assert_eq!(reopened.state().sequence, 0);
}

#[test]
fn abrupt_process_exit_retains_committed_cancellation_without_running_destructors() {
    let root = Root::new();
    drop(RunStore::create(&root.workspace(), plan()).unwrap());
    let output = child_command(&root.workspace(), "cancel-and-exit")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(73));
    let store = RunStore::open(&root.workspace()).unwrap();
    assert_eq!(store.state().sequence, 1);
    assert_eq!(store.state().steps[0].status, RunStepStatus::Cancelled);
}

fn child_command(workspace: &Path, mode: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", "workspace_child", "--ignored", "--nocapture"])
        .env("FLOW_STATE_TEST_WORKSPACE", workspace)
        .env("FLOW_STATE_TEST_MODE", mode);
    command
}

#[test]
#[ignore = "subprocess entrypoint invoked by parent tests"]
fn workspace_child() {
    let path = PathBuf::from(std::env::var_os("FLOW_STATE_TEST_WORKSPACE").unwrap());
    let mut store = RunStore::open(&path).unwrap();
    match std::env::var("FLOW_STATE_TEST_MODE").unwrap().as_str() {
        "hold" => {
            println!("FLOW_STATE_CHILD_READY");
            std::io::stdout().flush().unwrap();
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).unwrap();
        }
        "cancel-and-exit" => {
            store.cancel_pending("step:inspect").unwrap();
            std::process::exit(73);
        }
        mode => panic!("unknown test mode {mode}"),
    }
}

#[test]
fn partial_write_is_reported_without_removing_or_ignoring_evidence() {
    let root = Root::new();
    drop(RunStore::create(&root.workspace(), plan()).unwrap());
    fs::write(root.workspace().join("snapshot.pending"), b"{partial").unwrap();
    let before = state_files(&root.workspace());
    let opened = RunStore::open(&root.workspace());
    assert!(
        matches!(opened, Err(StateError::IncompleteWrite)),
        "{opened:?}"
    );
    assert_eq!(state_files(&root.workspace()), before);
}

#[test]
fn malformed_or_corrupt_latest_snapshot_never_falls_back_to_previous_success() {
    for mutation in [
        "truncated",
        "checksum",
        "identity",
        "predecessor",
        "unknown-field",
        "future-schema",
    ] {
        let root = Root::new();
        let mut store = RunStore::create(&root.workspace(), plan()).unwrap();
        store.cancel_pending("step:inspect").unwrap();
        drop(store);
        let path = root.workspace().join("00000000000000000001.json");
        let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        match mutation {
            "truncated" => {
                fs::write(&path, b"{").unwrap();
            }
            "checksum" => {
                value["state_digest"] = "0".repeat(64).into();
            }
            "identity" => {
                value["state"]["plan"]["run_id"] = "run:other".into();
            }
            "predecessor" => {
                value["state"]["previous_digest"] = "0".repeat(64).into();
            }
            "unknown-field" => {
                value["unexpected"] = true.into();
            }
            "future-schema" => {
                value["schema_version"] = "flow.run-snapshot/v2".into();
            }
            _ => unreachable!(),
        }
        if mutation != "truncated" {
            fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        }
        let before = state_files(&root.workspace());
        let error = RunStore::open(&root.workspace()).unwrap_err();
        if mutation == "future-schema" {
            assert!(matches!(error, StateError::UnsupportedSchema));
        }
        assert_eq!(
            state_files(&root.workspace()),
            before,
            "{mutation} evidence must remain intact"
        );
    }
}

#[test]
fn sequence_gaps_and_unrecognized_entries_fail_closed() {
    for name in ["00000000000000000002.json", "foreign.txt"] {
        let root = Root::new();
        drop(RunStore::create(&root.workspace(), plan()).unwrap());
        fs::write(root.workspace().join(name), b"{}").unwrap();
        assert!(matches!(
            RunStore::open(&root.workspace()),
            Err(StateError::Corrupt)
        ));
    }
}

#[test]
fn plan_validation_rejects_cycles_duplicates_and_identity_drift() {
    let valid = plan();
    let mut mutated = valid.clone();
    let own_id = mutated.steps[0].step_id.clone();
    mutated.steps[0].depends_on.push(own_id);
    assert!(mutated.validate().is_err());
    mutated = valid.clone();
    mutated.steps.push(mutated.steps[0].clone());
    assert!(mutated.validate().is_err());
    mutated = valid.clone();
    mutated.run_id = "run:other".to_owned();
    assert!(mutated.validate().is_err());
    mutated = valid;
    mutated.schema_version = "flow.run-plan/v2".to_owned();
    assert!(matches!(
        mutated.validate(),
        Err(StateError::UnsupportedSchema)
    ));
}

#[test]
fn records_reject_unknown_fields_and_false_completion() {
    let mut value = serde_json::to_value(RunState::new(plan()).unwrap()).unwrap();
    value["steps"][0]["status"] = "succeeded".into();
    let state: RunState = serde_json::from_value(value.clone()).unwrap();
    assert!(state.validate().is_err());
    value["steps"][0]["secret"] = "PRIVATE_CANARY".into();
    assert!(serde_json::from_value::<RunState>(value).is_err());
    let mut state: RunState = serde_json::from_str(include_str!(
        "../contracts/fixtures/state/completed.v1.fixture.json"
    ))
    .unwrap();
    state.validate().unwrap();
    state.steps[0].checkpoint.as_mut().unwrap().context_digest = "0".repeat(64);
    assert!(state.validate().is_err());
}

#[cfg(unix)]
#[test]
fn symlink_state_and_workspace_are_rejected_and_permissions_are_private() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let root = Root::new();
    drop(RunStore::create(&root.workspace(), plan()).unwrap());
    assert_eq!(
        fs::metadata(root.workspace()).unwrap().permissions().mode() & 0o777,
        0o700
    );
    for (name, _) in state_files(&root.workspace()) {
        assert_eq!(
            fs::metadata(root.workspace().join(name))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    let alias = root.path.join("alias");
    symlink(root.workspace(), &alias).unwrap();
    assert!(matches!(
        RunStore::open(&alias),
        Err(StateError::UnsafeEntry)
    ));
    let snapshot = root.workspace().join("00000000000000000000.json");
    let outside = root.path.join("outside.json");
    fs::rename(&snapshot, &outside).unwrap();
    symlink(&outside, &snapshot).unwrap();
    assert!(matches!(
        RunStore::open(&root.workspace()),
        Err(StateError::UnsafeEntry)
    ));
}
