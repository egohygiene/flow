use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{
    MAX_RUN_RECORD_BYTES, MAX_RUN_SNAPSHOTS, RUN_SNAPSHOT_V1, RunPlan, RunRecoveryAction, RunState,
    RunStepStatus, StateError, canonical_bytes, digest, io_error, require, schema,
};

const LOCK_FILE: &str = "workspace.lock";
const PENDING_FILE: &str = "snapshot.pending";
const MAX_HISTORY_BYTES: u64 = 64 * 1024 * 1024;

#[cfg(test)]
#[derive(Clone, Copy, Eq, PartialEq)]
enum CommitFault {
    Created,
    Synced,
    Renamed,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    schema_version: String,
    state_digest: String,
    state: RunState,
}

/// One exclusively held local workspace. Dropping the handle releases its OS lock.
/// The lock file is never deleted; a killed process cannot leave a stale lock claim.
pub struct RunStore {
    root: PathBuf,
    _lock: File,
    pub(super) state: RunState,
    poisoned: bool,
    #[cfg(test)]
    fault: Option<CommitFault>,
}

impl std::fmt::Debug for RunStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunStore")
            .field("run_id", &self.state.plan.run_id)
            .field("sequence", &self.state.sequence)
            .field("poisoned", &self.poisoned)
            .finish_non_exhaustive()
    }
}

impl RunStore {
    /// Create a new private workspace without replacing an existing directory.
    ///
    /// # Errors
    /// Rejects invalid plans, existing paths, failed locking, or failed persistence.
    pub fn create(workspace: &Path, plan: RunPlan) -> Result<Self, StateError> {
        let state = RunState::new(plan)?;
        state.validate()?;
        let mut builder = fs::DirBuilder::new();
        builder.recursive(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(workspace).map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                StateError::AlreadyExists
            } else {
                io_error("create workspace", error)
            }
        })?;
        let root = checked_root(workspace)?;
        if let Some(parent) = root.parent() {
            sync_directory(parent)?;
        }
        let lock = lock_workspace(&root, true)?;
        let store = Self {
            root,
            _lock: lock,
            state,
            poisoned: false,
            #[cfg(test)]
            fault: None,
        };
        store.write_snapshot(&store.state)?;
        Ok(store)
    }

    /// Exclusively reopen and validate every retained snapshot. This never executes work.
    ///
    /// # Errors
    /// Rejects concurrent opens, partial writes, unsupported versions, corrupt history,
    /// unsafe entries, and failed I/O. It never falls back to an older valid snapshot.
    pub fn open(workspace: &Path) -> Result<Self, StateError> {
        let root = checked_root(workspace)?;
        let lock = lock_workspace(&root, false)?;
        let (state, _) = load_history(&root)?;
        Ok(Self {
            root,
            _lock: lock,
            state,
            poisoned: false,
            #[cfg(test)]
            fault: None,
        })
    }

    #[must_use]
    pub const fn state(&self) -> &RunState {
        &self.state
    }

    pub(super) fn ensure_usable(&self) -> Result<(), StateError> {
        if self.poisoned {
            Err(StateError::ReopenRequired)
        } else {
            Ok(())
        }
    }

    pub(super) fn commit(&mut self, mut next: RunState) -> Result<(), StateError> {
        if self.poisoned {
            return Err(StateError::ReopenRequired);
        }
        next.sequence = self
            .state
            .sequence
            .checked_add(1)
            .ok_or(StateError::Limit)?;
        next.previous_digest = digest(&self.state)?;
        next.validate()?;
        validate_transition(&self.state, &next)?;
        let (on_disk, bytes) = match load_history(&self.root) {
            Ok(history) => history,
            Err(error) => {
                self.poisoned = true;
                return Err(error);
            }
        };
        if on_disk != self.state {
            self.poisoned = true;
            return Err(StateError::Corrupt);
        }
        if bytes + canonical_bytes(&next)?.len() as u64 + 256 > MAX_HISTORY_BYTES {
            return Err(StateError::Limit);
        }
        if let Err(error) = self.write_snapshot(&next) {
            self.poisoned = true;
            return Err(error);
        }
        self.state = next;
        Ok(())
    }

    fn write_snapshot(&self, state: &RunState) -> Result<(), StateError> {
        let snapshot = Snapshot {
            schema_version: RUN_SNAPSHOT_V1.to_owned(),
            state_digest: digest(state)?,
            state: state.clone(),
        };
        let bytes = canonical_bytes(&snapshot)?;
        if bytes.len() as u64 > MAX_RUN_RECORD_BYTES {
            return Err(StateError::Limit);
        }
        let pending = self.root.join(PENDING_FILE);
        let mut options = private_options();
        let mut file = options
            .write(true)
            .create_new(true)
            .open(&pending)
            .map_err(|error| io_error("create pending snapshot", error))?;
        #[cfg(test)]
        self.inject_fault(CommitFault::Created)?;
        file.write_all(&bytes)
            .map_err(|error| io_error("write snapshot", error))?;
        file.sync_all()
            .map_err(|error| io_error("sync snapshot", error))?;
        drop(file);
        #[cfg(test)]
        self.inject_fault(CommitFault::Synced)?;
        let destination = self.root.join(snapshot_name(state.sequence));
        if destination
            .try_exists()
            .map_err(|error| io_error("check snapshot destination", error))?
        {
            return Err(StateError::Corrupt);
        }
        fs::rename(pending, destination).map_err(|error| io_error("commit snapshot", error))?;
        #[cfg(test)]
        self.inject_fault(CommitFault::Renamed)?;
        sync_directory(&self.root)
    }

    #[cfg(test)]
    fn inject_fault(&self, phase: CommitFault) -> Result<(), StateError> {
        if self.fault == Some(phase) {
            Err(io_error(
                "injected commit interruption",
                std::io::Error::other("fixture"),
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;

fn checked_root(workspace: &Path) -> Result<PathBuf, StateError> {
    let metadata =
        fs::symlink_metadata(workspace).map_err(|error| io_error("inspect workspace", error))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(StateError::UnsafeEntry);
    }
    workspace
        .canonicalize()
        .map_err(|error| io_error("resolve workspace", error))
}

fn private_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

fn lock_workspace(root: &Path, create: bool) -> Result<File, StateError> {
    let path = root.join(LOCK_FILE);
    if !create {
        check_regular(&path)?;
    }
    let mut options = private_options();
    options.read(true).write(true);
    if create {
        options.create_new(true);
    }
    let file = options
        .open(path)
        .map_err(|error| io_error("open workspace lock", error))?;
    if create {
        file.sync_all()
            .map_err(|error| io_error("sync workspace lock", error))?;
    }
    fs2::FileExt::try_lock_exclusive(&file).map_err(|error| {
        if error.kind() == std::io::ErrorKind::WouldBlock
            || error.raw_os_error() == fs2::lock_contended_error().raw_os_error()
        {
            StateError::Busy
        } else {
            io_error("lock workspace", error)
        }
    })?;
    Ok(file)
}

fn snapshot_name(sequence: u64) -> String {
    format!("{sequence:020}.json")
}

fn check_regular(path: &Path) -> Result<u64, StateError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| io_error("inspect state entry", error))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(StateError::UnsafeEntry);
    }
    Ok(metadata.len())
}

fn load_history(root: &Path) -> Result<(RunState, u64), StateError> {
    let mut paths = Vec::new();
    let mut bytes = 0_u64;
    for entry in fs::read_dir(root).map_err(|error| io_error("enumerate state", error))? {
        let entry = entry.map_err(|error| io_error("read state entry", error))?;
        let name = entry.file_name();
        if name == PENDING_FILE {
            return Err(StateError::IncompleteWrite);
        }
        let length = check_regular(&entry.path())?;
        if name == LOCK_FILE {
            require(length == 0, "empty lock file")?;
            continue;
        }
        if length > MAX_RUN_RECORD_BYTES {
            return Err(StateError::Limit);
        }
        bytes = bytes.checked_add(length).ok_or(StateError::Limit)?;
        if bytes > MAX_HISTORY_BYTES || paths.len() as u64 >= MAX_RUN_SNAPSHOTS {
            return Err(StateError::Limit);
        }
        paths.push(entry.path());
    }
    paths.sort();
    let mut previous: Option<RunState> = None;
    for (index, path) in paths.iter().enumerate() {
        if path.file_name().and_then(|name| name.to_str()) != Some(&snapshot_name(index as u64)) {
            return Err(StateError::Corrupt);
        }
        let mut encoded = Vec::new();
        File::open(path)
            .map_err(|error| io_error("open snapshot", error))?
            .take(MAX_RUN_RECORD_BYTES + 1)
            .read_to_end(&mut encoded)
            .map_err(|error| io_error("read snapshot", error))?;
        if encoded.len() as u64 > MAX_RUN_RECORD_BYTES {
            return Err(StateError::Limit);
        }
        let header: serde_json::Value =
            serde_json::from_slice(&encoded).map_err(|_| StateError::Malformed)?;
        schema(
            header
                .get("schema_version")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(""),
            RUN_SNAPSHOT_V1,
        )?;
        let snapshot: Snapshot =
            serde_json::from_slice(&encoded).map_err(|_| StateError::Malformed)?;
        snapshot.state.validate()?;
        if snapshot.state.sequence != index as u64
            || snapshot.state_digest != digest(&snapshot.state)?
        {
            return Err(StateError::Corrupt);
        }
        if let Some(old) = &previous {
            if snapshot.state.previous_digest != digest(old)? {
                return Err(StateError::Corrupt);
            }
            validate_transition(old, &snapshot.state)?;
        } else if snapshot.state != RunState::new(snapshot.state.plan.clone())? {
            return Err(StateError::Corrupt);
        }
        previous = Some(snapshot.state);
    }
    previous
        .map(|state| (state, bytes))
        .ok_or(StateError::IncompleteWrite)
}

fn validate_transition(old: &RunState, next: &RunState) -> Result<(), StateError> {
    require(
        next.sequence == old.sequence + 1 && next.plan == old.plan,
        "immutable plan and ordered history",
    )?;
    require(
        next.authority_decisions
            .starts_with(&old.authority_decisions)
            && next.recovery_decisions.starts_with(&old.recovery_decisions),
        "append-only decisions",
    )?;
    let changed: Vec<_> = old
        .steps
        .iter()
        .zip(&next.steps)
        .filter(|(a, b)| a != b)
        .collect();
    require(changed.len() == 1, "one state transition per snapshot")?;
    let (before, after) = changed[0];
    let authority = &next.authority_decisions[old.authority_decisions.len()..];
    let recovery = &next.recovery_decisions[old.recovery_decisions.len()..];
    match (before.status, after.status) {
        (RunStepStatus::Pending, RunStepStatus::Running) => {
            require(
                after.attempt == before.attempt + 1 && authority.len() == 1 && recovery.is_empty(),
                "launch transition",
            )?;
            require(
                authority[0].granted
                    && authority[0].step_id == after.step_id
                    && authority[0].attempt == after.attempt,
                "launch decision",
            )?;
            let index = next.step_index(&after.step_id)?;
            require(
                next.plan.steps[index].depends_on.iter().all(|id| {
                    old.steps
                        .iter()
                        .any(|s| &s.step_id == id && s.status == RunStepStatus::Succeeded)
                }),
                "completed dependencies",
            )?;
        }
        (RunStepStatus::Pending, RunStepStatus::Denied) => {
            require(
                after.attempt == before.attempt && authority.len() == 1 && recovery.is_empty(),
                "denial transition",
            )?;
            require(
                !authority[0].granted
                    && authority[0].step_id == after.step_id
                    && authority[0].attempt == after.attempt,
                "denial decision",
            )?;
        }
        (RunStepStatus::Pending, RunStepStatus::Cancelled)
        | (
            RunStepStatus::Running,
            RunStepStatus::Succeeded | RunStepStatus::Failed | RunStepStatus::Cancelled,
        ) => {
            require(
                after.attempt == before.attempt && authority.is_empty() && recovery.is_empty(),
                "terminal transition",
            )?;
        }
        (
            RunStepStatus::Running
            | RunStepStatus::Failed
            | RunStepStatus::Cancelled
            | RunStepStatus::Denied,
            RunStepStatus::Pending | RunStepStatus::Abandoned,
        ) => {
            require(
                after.attempt == before.attempt && authority.is_empty() && recovery.len() == 1,
                "explicit recovery transition",
            )?;
            let decision = &recovery[0];
            require(
                decision.step_id == after.step_id && decision.attempt == after.attempt,
                "recovery attempt identity",
            )?;
            require(
                (decision.action == RunRecoveryAction::Retry)
                    == (after.status == RunStepStatus::Pending),
                "recovery action",
            )?;
        }
        _ => return Err(StateError::Transition),
    }
    Ok(())
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), StateError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| io_error("sync state directory", error))
}

// Windows lacks a portable directory fsync in std. Files are flushed before an
// atomic rename; the v1 Windows guarantee covers process crashes, not power loss.
#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), StateError> {
    Ok(())
}
