//! Optiflow v0.1.1 read-only CLI adapter. The native command-result protocol is
//! separate from Flow's extension JSONL protocol; this adapter validates it
//! before recording a Flow-owned receipt. A receipt is evidence, not authority.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const RECEIPT_SCHEMA: &str = "flow.optiflow-read-only-receipt/v1";
pub const PROVIDER_VERSION: &str = "0.1.1";
pub const PROVIDER_SOURCE_REVISION: &str = "b82599a2231e997d42fd9f26f4b59587f4ae14cf";
pub const COMMAND_RESULT_SCHEMA: &str = "optiflow.command-result.v1";
pub const RUN_SCHEMA: &str = "optiflow.run.v5";
pub const REPORT_SCHEMA: &str = "optiflow.report.v6";
pub const PLAN_SCHEMA: &str = "optiflow.plan.v5";
pub const ARTIFACT_SET_SCHEMA: &str = "optiflow.artifact-set.v1";
const POLICY_SCHEMA: &str = "optiflow.effective-policy.v1";
const MAX_STDOUT: usize = 16 * 1024 * 1024;
const MAX_STDERR: usize = 64 * 1024;
const MAX_ARTIFACT: u64 = 16 * 1024 * 1024;
const SCAN_DEADLINE: Duration = Duration::from_secs(120);
const SHORT_DEADLINE: Duration = Duration::from_secs(30);

/// The pinned release archives and their executable digests. The Linux
/// executable was exercised locally; the two macOS archives were inspected and
/// hashed, but still require native execution validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReleaseProfile {
    pub target: &'static str,
    pub archive_sha256: &'static str,
    pub executable_sha256: &'static str,
}

pub const RELEASE_PROFILES: [ReleaseProfile; 3] = [
    ReleaseProfile {
        target: "x86_64-unknown-linux-gnu",
        archive_sha256: "5e8a0eea84f5ab55fe75fc8a8abc2e9d36e3512537b754262b1f34a9c40aae31",
        executable_sha256: "46af9399f3785607f835d9a805a8daa42c93597772c6ee812911615ea4b76280",
    },
    ReleaseProfile {
        target: "aarch64-apple-darwin",
        archive_sha256: "f83fa18a98e99198e535e37dd93e1584024985df4960f2defd8124d52e656ab1",
        executable_sha256: "2a2411dbbafd5c22a2b398f13eaf4f0be84e5dedb8f4c9d80f7df418522c6ca3",
    },
    ReleaseProfile {
        target: "x86_64-apple-darwin",
        archive_sha256: "71f1cc9be7ec7d3373807e012114bb64aba62f550fd9da6b4ddccda62397c694",
        executable_sha256: "a3097b5060ae05a361c97dd004857e7152a019f3f11dc0459b993183a6cf2135",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    Scan,
    Report,
    PlanExactDuplicates,
    DryRun,
    Quarantine,
    Restore,
    Finalize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityState {
    AvailableReadOnly,
    UnsupportedByPinnedRelease,
}

impl Capability {
    #[must_use]
    pub const fn state(self) -> CapabilityState {
        match self {
            Self::Scan | Self::Report | Self::PlanExactDuplicates => {
                CapabilityState::AvailableReadOnly
            }
            Self::DryRun | Self::Quarantine | Self::Restore | Self::Finalize => {
                CapabilityState::UnsupportedByPinnedRelease
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("pinned Optiflow executable is unavailable")]
    Unavailable,
    #[error("Optiflow executable, version, or host target does not match v0.1.1")]
    Incompatible,
    #[error("{0:?} is unsupported by pinned Optiflow v0.1.1")]
    UnsupportedCapability(Capability),
    #[error("source root changed identity or filesystem")]
    SourceChanged,
    #[error("source root and local evidence directory overlap, or an artifact escapes it")]
    PathEscape,
    #[error("Optiflow returned stale state")]
    StalePlan {
        diagnostics: Vec<Value>,
        stdout_sha256: String,
    },
    #[error("Optiflow reported {class} for {command}")]
    ProviderFailure {
        command: String,
        class: String,
        diagnostics: Vec<Value>,
        stdout_sha256: String,
    },
    #[error("provider output or committed artifact is incompatible: {0}")]
    InvalidEvidence(&'static str),
    #[error("provider exceeded its {0} bound")]
    Limit(&'static str),
    #[error("provider exceeded its deadline")]
    Timeout,
    #[error("local evidence I/O failed: {0}")]
    Io(#[from] io::Error),
    #[error("local evidence serialization failed: {0}")]
    Json(#[from] serde_json::Error),
}

impl AdapterError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Unavailable => "provider_unavailable",
            Self::Incompatible => "provider_incompatible",
            Self::UnsupportedCapability(_) => "capability_unsupported",
            Self::SourceChanged => "source_identity_changed",
            Self::PathEscape => "path_escape",
            Self::StalePlan { .. } => "stale_plan",
            Self::ProviderFailure { .. } => "provider_failure",
            Self::InvalidEvidence(_) => "invalid_provider_evidence",
            Self::Limit(_) => "limit_exceeded",
            Self::Timeout => "provider_timeout",
            Self::Io(_) | Self::Json(_) => "local_evidence_failure",
        }
    }
}

/// Lossless provider path. Non-UTF-8 paths retain their original Unix bytes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "encoding", rename_all = "snake_case")]
pub enum NativePath {
    Utf8 { value: String },
    UnixBytes { base64: String },
}

impl NativePath {
    fn path(&self) -> Result<PathBuf, AdapterError> {
        match self {
            Self::Utf8 { value } => Ok(PathBuf::from(value)),
            Self::UnixBytes { base64 } => {
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(base64)
                    .map_err(|_| AdapterError::InvalidEvidence("invalid native path bytes"))?;
                #[cfg(unix)]
                {
                    use std::os::unix::ffi::OsStringExt;
                    Ok(PathBuf::from(OsString::from_vec(bytes)))
                }
                #[cfg(not(unix))]
                {
                    let _ = bytes;
                    Err(AdapterError::Incompatible)
                }
            }
        }
    }

    fn from_path(path: &Path) -> Self {
        if let Some(value) = path.to_str() {
            return Self::Utf8 {
                value: value.to_owned(),
            };
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            Self::UnixBytes {
                base64: base64::engine::general_purpose::STANDARD
                    .encode(path.as_os_str().as_bytes()),
            }
        }
        #[cfg(not(unix))]
        {
            unreachable!("supported release profiles are Unix-only")
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceIdentity {
    pub root: NativePath,
    pub filesystem_id: u64,
    pub file_id: u64,
}

impl SourceIdentity {
    fn observe(root: &Path) -> Result<Self, AdapterError> {
        if fs::symlink_metadata(root)?.file_type().is_symlink() {
            return Err(AdapterError::PathEscape);
        }
        let canonical = fs::canonicalize(root)?;
        let meta = fs::metadata(&canonical)?;
        if !meta.is_dir() {
            return Err(AdapterError::InvalidEvidence(
                "source root must be a directory",
            ));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            Ok(Self {
                root: NativePath::from_path(&canonical),
                filesystem_id: meta.dev(),
                file_id: meta.ino(),
            })
        }
        #[cfg(not(unix))]
        {
            let _ = meta;
            Err(AdapterError::Incompatible)
        }
    }

    fn require_current(&self) -> Result<(), AdapterError> {
        let current = Self::observe(&self.root.path()?).map_err(|_| AdapterError::SourceChanged)?;
        if current == *self {
            Ok(())
        } else {
            Err(AdapterError::SourceChanged)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderIdentity {
    pub version: String,
    pub source_revision: String,
    pub target: String,
    pub archive_sha256: String,
    pub executable_sha256: String,
    pub interface: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReceiptStatus {
    Scanning,
    Reporting,
    Planning,
    Complete,
    Partial,
    Refused,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactEvidence {
    pub kind: String,
    pub schema: String,
    pub path: NativePath,
    pub size_bytes: u64,
    pub sha256: String,
    pub blake3_256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandEvidence {
    pub command: String,
    pub arguments: Vec<NativePath>,
    pub outcome: String,
    pub coverage: String,
    pub stdout_sha256: String,
    pub diagnostics: Vec<Value>,
    pub set_id: String,
    pub marker_sha256: String,
    pub artifacts: Vec<ArtifactEvidence>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureEvidence {
    pub code: String,
    pub detail: String,
    pub provider_diagnostics: Vec<Value>,
    pub provider_stdout_sha256: Option<String>,
}

/// A local, durable attempt record. An intermediate status after reopen is
/// recovery-required. No record conveys approval or source mutation authority.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadOnlyReceipt {
    pub schema_version: String,
    pub attempt_id: String,
    pub status: ReceiptStatus,
    pub provider: ProviderIdentity,
    pub source: SourceIdentity,
    pub state_directory: NativePath,
    pub environment: String,
    pub capabilities: BTreeMap<String, CapabilityState>,
    pub commands: Vec<CommandEvidence>,
    pub source_run_id: Option<String>,
    pub source_set_id: Option<String>,
    pub plan_id: Option<String>,
    pub plan_sha256: Option<String>,
    pub approval: Option<String>,
    pub recovery: String,
    pub duplicate_logical_bytes: Option<u64>,
    pub physical_savings_bytes: Option<u64>,
    pub failure: Option<FailureEvidence>,
}

impl ReadOnlyReceipt {
    /// Reopen evidence without promoting an unfinished attempt to completion.
    /// # Errors
    /// Rejects invalid JSON, a different receipt schema, or changed evidence
    /// files. A receipt and its hashes are local audit evidence, not a signature.
    pub fn load(path: &Path) -> Result<Self, AdapterError> {
        let receipt: Self = serde_json::from_slice(&read_regular(path, MAX_ARTIFACT)?)?;
        if receipt.schema_version != RECEIPT_SCHEMA {
            return Err(AdapterError::InvalidEvidence("receipt schema"));
        }
        let profile = RELEASE_PROFILES
            .iter()
            .find(|profile| profile.target == receipt.provider.target)
            .ok_or(AdapterError::InvalidEvidence("receipt target"))?;
        if receipt.provider.version != PROVIDER_VERSION
            || receipt.provider.source_revision != PROVIDER_SOURCE_REVISION
            || receipt.provider.interface != COMMAND_RESULT_SCHEMA
            || receipt.provider.archive_sha256 != profile.archive_sha256
            || receipt.provider.executable_sha256 != profile.executable_sha256
        {
            return Err(AdapterError::InvalidEvidence("receipt provider identity"));
        }
        let names = receipt
            .commands
            .iter()
            .map(|command| command.command.as_str())
            .collect::<Vec<_>>();
        let partial = receipt
            .commands
            .iter()
            .any(|command| command.coverage == "partial");
        let consistent = match receipt.status {
            ReceiptStatus::Scanning => names.is_empty(),
            ReceiptStatus::Reporting => names == ["scan"],
            ReceiptStatus::Planning => names == ["scan", "report"],
            ReceiptStatus::Complete | ReceiptStatus::Partial => {
                names == ["scan", "report", "plan"]
                    && receipt.source_run_id.is_some()
                    && receipt.source_set_id.is_some()
                    && receipt.plan_id.is_some()
                    && receipt.plan_sha256.as_deref()
                        == receipt.commands[2]
                            .artifacts
                            .iter()
                            .find(|artifact| artifact.kind == "plan")
                            .map(|artifact| artifact.sha256.as_str())
                    && receipt.failure.is_none()
                    && receipt.recovery == "none"
                    && partial == matches!(receipt.status, ReceiptStatus::Partial)
            }
            ReceiptStatus::Refused => receipt.failure.is_some(),
        };
        if !consistent || receipt.approval.is_some() || receipt.physical_savings_bytes.is_some() {
            return Err(AdapterError::InvalidEvidence("receipt state or authority"));
        }
        for (key, capability) in [
            ("scan", Capability::Scan),
            ("report", Capability::Report),
            ("plan-exact-duplicates", Capability::PlanExactDuplicates),
            ("dry-run", Capability::DryRun),
            ("quarantine", Capability::Quarantine),
            ("restore", Capability::Restore),
            ("finalize", Capability::Finalize),
        ] {
            if receipt.capabilities.get(key) != Some(&capability.state()) {
                return Err(AdapterError::InvalidEvidence("receipt capability state"));
            }
        }
        let state = receipt.state_directory.path()?;
        for command in &receipt.commands {
            for artifact in &command.artifacts {
                let path = artifact.path.path()?;
                if !path.starts_with(&state)
                    || path
                        .components()
                        .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
                    || fs::canonicalize(&path)? != path
                {
                    return Err(AdapterError::PathEscape);
                }
                let bytes = read_regular(&path, MAX_ARTIFACT)?;
                if bytes.len() as u64 != artifact.size_bytes
                    || sha256(&bytes) != artifact.sha256
                    || blake3::hash(&bytes).to_hex().as_str() != artifact.blake3_256
                {
                    return Err(AdapterError::InvalidEvidence("receipt artifact changed"));
                }
            }
        }
        Ok(receipt)
    }

    #[must_use]
    pub fn needs_recovery(&self) -> bool {
        matches!(
            self.status,
            ReceiptStatus::Scanning | ReceiptStatus::Reporting | ReceiptStatus::Planning
        ) || self.recovery == "inspect-provider-state-before-retry"
    }
}

/// A pinned native CLI, with exactly three read-only capabilities.
pub struct OptiflowReadOnlyAdapter {
    executable: PathBuf,
    identity: ProviderIdentity,
}

impl OptiflowReadOnlyAdapter {
    /// Check the host target, executable bytes, and bounded `--version` output.
    /// # Errors
    /// Refuses missing, different, or unsupported executables.
    pub fn probe(executable: &Path) -> Result<Self, AdapterError> {
        let target = match (std::env::consts::ARCH, std::env::consts::OS) {
            ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
            ("x86_64", "macos") => "x86_64-apple-darwin",
            ("aarch64", "macos") => "aarch64-apple-darwin",
            _ => return Err(AdapterError::Incompatible),
        };
        let profile = RELEASE_PROFILES
            .iter()
            .find(|profile| profile.target == target)
            .ok_or(AdapterError::Incompatible)?;
        if !executable.is_absolute() {
            return Err(AdapterError::Incompatible);
        }
        let meta = fs::symlink_metadata(executable).map_err(|_| AdapterError::Unavailable)?;
        if !meta.is_file() || meta.file_type().is_symlink() {
            return Err(AdapterError::Incompatible);
        }
        if sha256_file(executable, u64::MAX)? != profile.executable_sha256 {
            return Err(AdapterError::Incompatible);
        }
        let version = capture(executable, &[OsString::from("--version")], SHORT_DEADLINE)?;
        if version.code != Some(0)
            || version.stdout != b"optiflow 0.1.1\n"
            || !version.stderr.is_empty()
        {
            return Err(AdapterError::Incompatible);
        }
        Ok(Self {
            executable: executable.to_owned(),
            identity: ProviderIdentity {
                version: PROVIDER_VERSION.to_owned(),
                source_revision: PROVIDER_SOURCE_REVISION.to_owned(),
                target: target.to_owned(),
                archive_sha256: profile.archive_sha256.to_owned(),
                executable_sha256: profile.executable_sha256.to_owned(),
                interface: COMMAND_RESULT_SCHEMA.to_owned(),
            },
        })
    }

    #[must_use]
    pub const fn identity(&self) -> &ProviderIdentity {
        &self.identity
    }

    /// Refuse a mutation request as a distinct unavailable capability.
    /// # Errors
    /// Unsupported requests are always refused; read-only requests use
    /// `inspect` with a selected root and local state directory.
    pub fn require_capability(&self, capability: Capability) -> Result<(), AdapterError> {
        match capability.state() {
            CapabilityState::AvailableReadOnly => Ok(()),
            CapabilityState::UnsupportedByPinnedRelease => {
                Err(AdapterError::UnsupportedCapability(capability))
            }
        }
    }

    /// Scan a selected directory and retain the report and review-only plan.
    ///
    /// The state directory must be disjoint from the source. Only synthetic
    /// fixtures should be used in tests. A receipt is synced before the first
    /// child and after every verified step. Provider and Flow evidence live in
    /// the state directory, while the source remains read-only.
    /// # Errors
    /// Returns typed refusals; `receipt_path` can be inspected after failure.
    pub fn inspect(
        &self,
        source_root: &Path,
        state_directory: &Path,
    ) -> Result<(ReadOnlyReceipt, PathBuf), AdapterError> {
        let source = SourceIdentity::observe(source_root)?;
        let root = source.root.path()?;
        let prospective_state = prospective_path(state_directory)?;
        if root.starts_with(&prospective_state) || prospective_state.starts_with(&root) {
            return Err(AdapterError::PathEscape);
        }
        fs::create_dir_all(&prospective_state)?;
        if fs::symlink_metadata(&prospective_state)?
            .file_type()
            .is_symlink()
        {
            return Err(AdapterError::PathEscape);
        }
        let state = fs::canonicalize(prospective_state)?;
        if root.starts_with(&state) || state.starts_with(&root) {
            return Err(AdapterError::PathEscape);
        }
        let receipt_dir = state.join("flow-optiflow-v0.1.1");
        fs::create_dir_all(&receipt_dir)?;
        let attempt_id = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| AdapterError::InvalidEvidence("host clock"))?
                .as_nanos()
        );
        let path = receipt_dir.join(format!("{attempt_id}.json"));
        let capabilities = [
            ("scan", Capability::Scan),
            ("report", Capability::Report),
            ("plan-exact-duplicates", Capability::PlanExactDuplicates),
            ("dry-run", Capability::DryRun),
            ("quarantine", Capability::Quarantine),
            ("restore", Capability::Restore),
            ("finalize", Capability::Finalize),
        ]
        .into_iter()
        .map(|(key, capability)| (key.to_owned(), capability.state()))
        .collect();
        let mut receipt = ReadOnlyReceipt {
            schema_version: RECEIPT_SCHEMA.to_owned(),
            attempt_id: attempt_id.clone(),
            status: ReceiptStatus::Scanning,
            provider: self.identity.clone(),
            source: source.clone(),
            state_directory: NativePath::from_path(&state),
            environment: "cleared; no config; no media probe; no network grant".to_owned(),
            capabilities,
            commands: Vec::new(),
            source_run_id: None,
            source_set_id: None,
            plan_id: None,
            plan_sha256: None,
            approval: None,
            recovery: "required-if-interrupted".to_owned(),
            duplicate_logical_bytes: None,
            physical_savings_bytes: None,
            failure: None,
        };
        persist(&path, &receipt, true)?;
        let result = self.inspect_steps(&state, &source, &attempt_id, &path, &mut receipt);
        if let Err(error) = result {
            receipt.status = ReceiptStatus::Refused;
            receipt.recovery = if matches!(error, AdapterError::Timeout | AdapterError::Io(_)) {
                "inspect-provider-state-before-retry".to_owned()
            } else {
                "no-source-mutation".to_owned()
            };
            receipt.failure = Some(FailureEvidence {
                code: error.code().to_owned(),
                detail: error.to_string(),
                provider_diagnostics: match &error {
                    AdapterError::ProviderFailure { diagnostics, .. }
                    | AdapterError::StalePlan { diagnostics, .. } => diagnostics.clone(),
                    _ => Vec::new(),
                },
                provider_stdout_sha256: match &error {
                    AdapterError::ProviderFailure { stdout_sha256, .. }
                    | AdapterError::StalePlan { stdout_sha256, .. } => Some(stdout_sha256.clone()),
                    _ => None,
                },
            });
            persist(&path, &receipt, false)?;
            return Err(error);
        }
        Ok((receipt, path))
    }

    #[allow(clippy::too_many_lines)]
    fn inspect_steps(
        &self,
        state: &Path,
        source: &SourceIdentity,
        attempt_id: &str,
        receipt_path: &Path,
        receipt: &mut ReadOnlyReceipt,
    ) -> Result<(), AdapterError> {
        source.require_current()?;
        let root = source.root.path()?;
        let scan_args = vec![
            OsString::from("scan"),
            OsString::from("--no-follow-symlinks"),
            OsString::from("--stay-on-filesystem"),
            OsString::from("--no-probe"),
            root.as_os_str().to_owned(),
        ];
        let scan = self.invoke(state, "scan", &scan_args, SCAN_DEADLINE)?;
        let run = field(&scan.result, "/run/run_id")?
            .as_str()
            .ok_or(AdapterError::InvalidEvidence("run id"))?;
        validate_id(run)?;
        let source_set = field(&scan.result, "/run/artifact_set_id")?
            .as_str()
            .ok_or(AdapterError::InvalidEvidence("source set id"))?;
        validate_id(source_set)?;
        if field(&scan.result, "/run/schema_version")?.as_str() != Some(RUN_SCHEMA)
            || field(&scan.result, "/schema_version")?.as_str() != Some(REPORT_SCHEMA)
            || field(&scan.result, "/run/options/follow_symlinks")?.as_bool() != Some(false)
            || field(&scan.result, "/run/options/cross_filesystems")?.as_bool() != Some(false)
            || field(&scan.result, "/run/options/probe_media")?.as_bool() != Some(false)
        {
            return Err(AdapterError::InvalidEvidence("scan schema or policy"));
        }
        let expected_root = source.root.path()?;
        let inputs = field(&scan.result, "/run/inputs")?
            .as_array()
            .ok_or(AdapterError::InvalidEvidence("source roots"))?;
        if inputs.len() != 1 || inputs[0].as_str().map(Path::new) != Some(expected_root.as_path()) {
            return Err(AdapterError::InvalidEvidence("source root binding"));
        }
        validate_report_paths(&scan.result, source)?;
        let scan_dir = state.join("runs").join(run);
        let scan_evidence = validate_set(
            &scan,
            &scan_dir.join("artifact-set.json"),
            &scan_dir,
            "scan",
            run,
            None,
            &[
                ("effective_policy", POLICY_SCHEMA),
                ("report", REPORT_SCHEMA),
                ("run", RUN_SCHEMA),
            ],
            "report",
        )?;
        receipt.source_run_id = Some(run.to_owned());
        receipt.source_set_id = Some(source_set.to_owned());
        if scan_evidence.set_id != source_set {
            return Err(AdapterError::InvalidEvidence("scan set identity"));
        }
        receipt.duplicate_logical_bytes = Some(
            field(&scan.result, "/storage/duplicate_logical_bytes")?
                .as_u64()
                .ok_or(AdapterError::InvalidEvidence("logical byte accounting"))?,
        );
        receipt.commands.push(scan_evidence);
        receipt.status = ReceiptStatus::Reporting;
        persist(receipt_path, receipt, false)?;
        source.require_current()?;

        let report = self.invoke(
            state,
            "report",
            &[OsString::from("report"), OsString::from(run)],
            SHORT_DEADLINE,
        )?;
        if field(&report.result, "/run/run_id")?.as_str() != Some(run) {
            return Err(AdapterError::InvalidEvidence("report run identity"));
        }
        validate_report_paths(&report.result, source)?;
        let report_evidence = validate_set(
            &report,
            &scan_dir.join("artifact-set.json"),
            &scan_dir,
            "scan",
            run,
            None,
            &[
                ("effective_policy", POLICY_SCHEMA),
                ("report", REPORT_SCHEMA),
                ("run", RUN_SCHEMA),
            ],
            "report",
        )?;
        if report_evidence.set_id != source_set {
            return Err(AdapterError::InvalidEvidence("report set identity"));
        }
        receipt.commands.push(report_evidence);
        receipt.status = ReceiptStatus::Planning;
        persist(receipt_path, receipt, false)?;
        source.require_current()?;

        let plan_dir = state.join("plans");
        fs::create_dir_all(&plan_dir)?;
        let plan_path = plan_dir.join(format!("{attempt_id}.json"));
        let plan = self.invoke(
            state,
            "plan",
            &[
                OsString::from("plan"),
                OsString::from("exact-duplicates"),
                OsString::from("--run"),
                OsString::from(run),
                OsString::from("--output"),
                plan_path.as_os_str().to_owned(),
            ],
            SHORT_DEADLINE,
        )?;
        if field(&plan.result, "/schema_version")?.as_str() != Some(PLAN_SCHEMA)
            || field(&plan.result, "/source_run_id")?.as_str() != Some(run)
            || field(&plan.result, "/source_artifact_set_id")?.as_str() != Some(source_set)
            || field(&plan.result, "/safety/mutates_files")?.as_bool() != Some(false)
            || field(&plan.result, "/safety/requires_explicit_apply")?.as_bool() != Some(true)
        {
            return Err(AdapterError::InvalidEvidence(
                "plan safety or source binding",
            ));
        }
        let plan_id = field(&plan.result, "/plan_id")?
            .as_str()
            .ok_or(AdapterError::InvalidEvidence("plan id"))?;
        validate_id(plan_id)?;
        validate_plan_paths(&plan.result, source)?;
        let plan_marker = plan_dir.join(format!("{attempt_id}.json.artifact-set.json"));
        let plan_evidence = validate_set(
            &plan,
            &plan_marker,
            &plan_dir,
            "plan",
            run,
            Some(source_set),
            &[("plan", PLAN_SCHEMA)],
            "plan",
        )?;
        receipt.plan_sha256 = Some(
            plan_evidence
                .artifacts
                .iter()
                .find(|artifact| artifact.kind == "plan")
                .ok_or(AdapterError::InvalidEvidence("plan artifact"))?
                .sha256
                .clone(),
        );
        receipt.plan_id = Some(plan_id.to_owned());
        receipt.commands.push(plan_evidence);
        source.require_current()?;
        let final_source = validate_set(
            &scan,
            &scan_dir.join("artifact-set.json"),
            &scan_dir,
            "scan",
            run,
            None,
            &[
                ("effective_policy", POLICY_SCHEMA),
                ("report", REPORT_SCHEMA),
                ("run", RUN_SCHEMA),
            ],
            "report",
        )?;
        if final_source.marker_sha256 != receipt.commands[0].marker_sha256
            || final_source
                .artifacts
                .iter()
                .map(|artifact| &artifact.sha256)
                .collect::<Vec<_>>()
                != receipt.commands[0]
                    .artifacts
                    .iter()
                    .map(|artifact| &artifact.sha256)
                    .collect::<Vec<_>>()
        {
            return Err(AdapterError::InvalidEvidence(
                "source artifacts changed after scan",
            ));
        }
        receipt.status = if receipt
            .commands
            .iter()
            .any(|command| command.coverage == "partial")
        {
            ReceiptStatus::Partial
        } else {
            ReceiptStatus::Complete
        };
        "none".clone_into(&mut receipt.recovery);
        persist(receipt_path, receipt, false)
    }

    fn invoke(
        &self,
        state: &Path,
        name: &str,
        specific: &[OsString],
        deadline: Duration,
    ) -> Result<NativeResult, AdapterError> {
        if sha256_file(&self.executable, u64::MAX)? != self.identity.executable_sha256 {
            return Err(AdapterError::Incompatible);
        }
        let mut args = vec![
            OsString::from("--output-format"),
            OsString::from("json"),
            OsString::from("--state-directory"),
            state.as_os_str().to_owned(),
            OsString::from("--no-config"),
        ];
        args.extend_from_slice(specific);
        let output = capture(&self.executable, &args, deadline)?;
        let digest = sha256(&output.stdout);
        let response: NativeEnvelope = serde_json::from_slice(&output.stdout)
            .map_err(|_| AdapterError::InvalidEvidence("command result JSON"))?;
        if response.schema != COMMAND_RESULT_SCHEMA || response.command != name {
            return Err(AdapterError::InvalidEvidence(
                "command result schema or command",
            ));
        }
        let (class, coverage) = match (
            response.outcome.class.as_str(),
            response.outcome.exit_code,
            output.code,
            response.coverage.as_ref().map(|c| c.status.as_str()),
        ) {
            ("success", 0, Some(0), Some("complete")) => ("success", "complete"),
            ("partial_success", 3, Some(3), Some("partial")) if !response.artifacts.is_empty() => {
                ("partial_success", "partial")
            }
            ("stale_state", 5, Some(5), _) => {
                return Err(AdapterError::StalePlan {
                    diagnostics: response.diagnostics,
                    stdout_sha256: digest,
                });
            }
            _ if output.code == Some(response.outcome.exit_code) => {
                return Err(AdapterError::ProviderFailure {
                    command: name.to_owned(),
                    class: response.outcome.class,
                    diagnostics: response.diagnostics,
                    stdout_sha256: digest,
                });
            }
            _ => {
                return Err(AdapterError::InvalidEvidence(
                    "process exit and declared outcome differ",
                ));
            }
        };
        if !output.stderr.is_empty() {
            return Err(AdapterError::InvalidEvidence("unexpected provider stderr"));
        }
        let result = response
            .result
            .ok_or(AdapterError::InvalidEvidence("missing domain result"))?;
        Ok(NativeResult {
            command: name.to_owned(),
            arguments: args
                .iter()
                .map(|arg| NativePath::from_path(Path::new(arg)))
                .collect(),
            class: class.to_owned(),
            coverage: coverage.to_owned(),
            stdout_sha256: digest,
            diagnostics: response.diagnostics,
            artifacts: response.artifacts,
            result,
        })
    }
}

fn field<'a>(value: &'a Value, pointer: &str) -> Result<&'a Value, AdapterError> {
    value
        .pointer(pointer)
        .ok_or(AdapterError::InvalidEvidence("required domain field"))
}

fn prospective_path(path: &Path) -> Result<PathBuf, AdapterError> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    if absolute
        .components()
        .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
    {
        return Err(AdapterError::PathEscape);
    }
    let mut ancestor = absolute.as_path();
    let mut missing = Vec::new();
    while !ancestor.exists() {
        missing.push(
            ancestor
                .file_name()
                .ok_or(AdapterError::PathEscape)?
                .to_owned(),
        );
        ancestor = ancestor.parent().ok_or(AdapterError::PathEscape)?;
    }
    let mut resolved = fs::canonicalize(ancestor)?;
    for segment in missing.into_iter().rev() {
        resolved.push(segment);
    }
    Ok(resolved)
}

fn validate_id(value: &str) -> Result<(), AdapterError> {
    if value.len() == 36
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
    {
        Ok(())
    } else {
        Err(AdapterError::InvalidEvidence("identifier format"))
    }
}

fn ensure_source_path(value: &Value, source: &SourceIdentity) -> Result<(), AdapterError> {
    let native: NativePath = serde_json::from_value(value.clone())?;
    let path = native.path()?;
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
        || !path.starts_with(source.root.path()?)
    {
        return Err(AdapterError::PathEscape);
    }
    Ok(())
}

fn validate_report_paths(report: &Value, source: &SourceIdentity) -> Result<(), AdapterError> {
    let observations = field(report, "/observations")?
        .as_array()
        .ok_or(AdapterError::InvalidEvidence("report observations"))?;
    for observation in observations {
        ensure_source_path(field(observation, "/path")?, source)?;
        if field(observation, "/device_id")?.as_u64() != Some(source.filesystem_id) {
            return Err(AdapterError::SourceChanged);
        }
        let identity = field(observation, "/filesystem_identity/filesystem_id")?.as_str();
        if identity != Some(source.filesystem_id.to_string().as_str()) {
            return Err(AdapterError::SourceChanged);
        }
    }
    Ok(())
}

fn validate_plan_paths(plan: &Value, source: &SourceIdentity) -> Result<(), AdapterError> {
    let actions = field(plan, "/actions")?
        .as_array()
        .ok_or(AdapterError::InvalidEvidence("plan actions"))?;
    for action in actions {
        if field(action, "/classification")?.as_str() != Some("exact")
            || field(action, "/proposed_operation")?.as_str() != Some("review_and_select")
        {
            return Err(AdapterError::InvalidEvidence("unexpected plan action"));
        }
        ensure_source_path(field(action, "/keep_path")?, source)?;
        for key in ["/candidate_paths", "/keep_alias_paths"] {
            let paths = field(action, key)?
                .as_array()
                .ok_or(AdapterError::InvalidEvidence("plan paths"))?;
            for path in paths {
                ensure_source_path(path, source)?;
            }
        }
        let preconditions = field(action, "/preconditions")?
            .as_array()
            .ok_or(AdapterError::InvalidEvidence("plan preconditions"))?;
        for precondition in preconditions {
            ensure_source_path(field(precondition, "/path")?, source)?;
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct NativeEnvelope {
    schema: String,
    command: String,
    outcome: NativeOutcome,
    coverage: Option<NativeCoverage>,
    artifacts: Vec<NativeArtifact>,
    diagnostics: Vec<Value>,
    result: Option<Value>,
}
#[derive(Deserialize)]
struct NativeOutcome {
    class: String,
    exit_code: i32,
}
#[derive(Deserialize)]
struct NativeCoverage {
    status: String,
}
#[derive(Deserialize)]
struct NativeArtifact {
    kind: String,
    schema: String,
    run_id: Option<String>,
    path: NativePath,
}
struct NativeResult {
    command: String,
    arguments: Vec<NativePath>,
    class: String,
    coverage: String,
    stdout_sha256: String,
    diagnostics: Vec<Value>,
    artifacts: Vec<NativeArtifact>,
    result: Value,
}

#[derive(Deserialize)]
struct NativeMarker {
    schema: String,
    set_id: String,
    set_kind: String,
    run_id: String,
    state: String,
    source_set_id: Option<String>,
    members: Vec<NativeMember>,
}
#[derive(Deserialize)]
struct NativeMember {
    kind: String,
    schema: String,
    path: NativePath,
    size_bytes: u64,
    digest: NativeDigest,
}
#[derive(Deserialize)]
struct NativeDigest {
    algorithm: String,
    value: String,
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn validate_set(
    response: &NativeResult,
    marker_path: &Path,
    directory: &Path,
    kind: &str,
    run_id: &str,
    source_set_id: Option<&str>,
    expected: &[(&str, &str)],
    result_kind: &str,
) -> Result<CommandEvidence, AdapterError> {
    if fs::canonicalize(directory)? != directory {
        return Err(AdapterError::PathEscape);
    }
    let marker_bytes = read_regular(marker_path, MAX_ARTIFACT)?;
    let marker: NativeMarker = serde_json::from_slice(&marker_bytes)?;
    if marker.schema != ARTIFACT_SET_SCHEMA
        || marker.set_kind != kind
        || marker.run_id != run_id
        || marker.state != "committed"
        || marker.source_set_id.as_deref() != source_set_id
    {
        return Err(AdapterError::InvalidEvidence("artifact set binding"));
    }
    validate_id(&marker.set_id)?;
    if marker.members.len() != expected.len() {
        return Err(AdapterError::InvalidEvidence("artifact member count"));
    }
    let mut artifacts = Vec::with_capacity(expected.len() + 1);
    for (expected_kind, expected_schema) in expected {
        let mut members = marker
            .members
            .iter()
            .filter(|member| member.kind == *expected_kind);
        let member = members
            .next()
            .ok_or(AdapterError::InvalidEvidence("missing artifact member"))?;
        if members.next().is_some()
            || member.schema != *expected_schema
            || member.digest.algorithm != "blake3-256"
        {
            return Err(AdapterError::InvalidEvidence(
                "artifact member kind, schema, or digest",
            ));
        }
        let relative = member.path.path()?;
        if relative.components().count() != 1
            || !matches!(relative.components().next(), Some(Component::Normal(_)))
        {
            return Err(AdapterError::PathEscape);
        }
        let path = directory.join(relative);
        let bytes = read_regular(&path, MAX_ARTIFACT)?;
        if bytes.len() as u64 != member.size_bytes
            || blake3::hash(&bytes).to_hex().as_str() != member.digest.value
        {
            return Err(AdapterError::InvalidEvidence("artifact member digest"));
        }
        let document: Value = serde_json::from_slice(&bytes)?;
        let schema_field = if *expected_kind == "effective_policy" {
            "/schema"
        } else {
            "/schema_version"
        };
        if field(&document, schema_field)?.as_str() != Some(*expected_schema) {
            return Err(AdapterError::InvalidEvidence("artifact member schema"));
        }
        if *expected_kind == result_kind && document != response.result {
            return Err(AdapterError::InvalidEvidence(
                "stdout and committed member differ",
            ));
        }
        if (*expected_kind == "run"
            && field(&document, "/artifact_set_id")?.as_str() != Some(&marker.set_id))
            || (*expected_kind == "report"
                && field(&document, "/run/artifact_set_id")?.as_str() != Some(&marker.set_id))
            || (*expected_kind == "plan"
                && field(&document, "/source_artifact_set_id")?.as_str() != source_set_id)
        {
            return Err(AdapterError::InvalidEvidence("member set identity"));
        }
        artifacts.push(ArtifactEvidence {
            kind: member.kind.clone(),
            schema: member.schema.clone(),
            path: NativePath::from_path(&path),
            size_bytes: member.size_bytes,
            sha256: sha256(&bytes),
            blake3_256: member.digest.value.clone(),
        });
    }
    // Every reference must point to a verified member, policy, or marker.
    for reference in &response.artifacts {
        if reference.run_id.as_deref() != Some(run_id) {
            return Err(AdapterError::InvalidEvidence("artifact run identity"));
        }
        let claimed = reference.path.path()?;
        if reference.kind == "artifact_set" {
            if claimed != marker_path || reference.schema != ARTIFACT_SET_SCHEMA {
                return Err(AdapterError::PathEscape);
            }
        } else if !artifacts.iter().any(|artifact| {
            artifact.kind == reference.kind
                && artifact.schema == reference.schema
                && artifact.path.path().ok().as_deref() == Some(claimed.as_path())
        }) {
            // A plan command also references the source effective policy.
            if !(kind == "plan"
                && reference.kind == "effective_policy"
                && reference.schema == POLICY_SCHEMA
                && claimed
                    == directory
                        .parent()
                        .unwrap_or(directory)
                        .join("runs")
                        .join(run_id)
                        .join("effective-policy.json"))
            {
                return Err(AdapterError::PathEscape);
            }
        }
    }
    let claimed_result = response
        .artifacts
        .iter()
        .any(|reference| reference.kind == result_kind);
    let claimed_marker = response
        .artifacts
        .iter()
        .any(|reference| reference.kind == "artifact_set");
    if !claimed_result || !claimed_marker {
        return Err(AdapterError::InvalidEvidence(
            "missing committed artifact reference",
        ));
    }
    artifacts.push(ArtifactEvidence {
        kind: "artifact_set".to_owned(),
        schema: ARTIFACT_SET_SCHEMA.to_owned(),
        path: NativePath::from_path(marker_path),
        size_bytes: marker_bytes.len() as u64,
        sha256: sha256(&marker_bytes),
        blake3_256: blake3::hash(&marker_bytes).to_hex().to_string(),
    });
    Ok(CommandEvidence {
        command: response.command.clone(),
        arguments: response.arguments.clone(),
        outcome: response.class.clone(),
        coverage: response.coverage.clone(),
        stdout_sha256: response.stdout_sha256.clone(),
        diagnostics: response.diagnostics.clone(),
        set_id: marker.set_id,
        marker_sha256: sha256(&marker_bytes),
        artifacts,
    })
}

fn read_regular(path: &Path, limit: u64) -> Result<Vec<u8>, AdapterError> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > limit {
        return Err(AdapterError::InvalidEvidence(
            "artifact is not a bounded regular file",
        ));
    }
    let mut bytes = Vec::new();
    File::open(path)?.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 != meta.len() || bytes.len() as u64 > limit {
        return Err(AdapterError::InvalidEvidence(
            "artifact changed during observation",
        ));
    }
    Ok(bytes)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn sha256_file(path: &Path, limit: u64) -> Result<String, AdapterError> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut total = 0_u64;
    let mut chunk = [0_u8; 8192];
    loop {
        let count = file.read(&mut chunk)?;
        if count == 0 {
            break;
        }
        total = total.saturating_add(count as u64);
        if total > limit {
            return Err(AdapterError::Limit("file"));
        }
        hash.update(&chunk[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

struct Captured {
    code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn capture(
    executable: &Path,
    args: &[OsString],
    deadline: Duration,
) -> Result<Captured, AdapterError> {
    let mut child = Command::new(executable)
        .args(args)
        .current_dir(executable.parent().ok_or(AdapterError::Incompatible)?)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| AdapterError::Unavailable)?;
    let out = child
        .stdout
        .take()
        .ok_or(AdapterError::InvalidEvidence("stdout pipe"))?;
    let err = child
        .stderr
        .take()
        .ok_or(AdapterError::InvalidEvidence("stderr pipe"))?;
    let stdout = thread::spawn(move || {
        out.take((MAX_STDOUT + 1) as u64)
            .bytes()
            .collect::<io::Result<Vec<_>>>()
    });
    let stderr = thread::spawn(move || {
        err.take((MAX_STDERR + 1) as u64)
            .bytes()
            .collect::<io::Result<Vec<_>>>()
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if start.elapsed() < deadline => thread::sleep(Duration::from_millis(5)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(AdapterError::Timeout);
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(AdapterError::Io(error));
            }
        }
    };
    let stdout = stdout
        .join()
        .map_err(|_| AdapterError::InvalidEvidence("stdout worker"))??;
    let stderr = stderr
        .join()
        .map_err(|_| AdapterError::InvalidEvidence("stderr worker"))??;
    if stdout.len() > MAX_STDOUT || stderr.len() > MAX_STDERR {
        return Err(AdapterError::Limit("output"));
    }
    Ok(Captured {
        code: status.code(),
        stdout,
        stderr,
    })
}

fn persist(path: &Path, receipt: &ReadOnlyReceipt, first: bool) -> Result<(), AdapterError> {
    let mut bytes = serde_json::to_vec_pretty(receipt)?;
    bytes.push(b'\n');
    let target = if first {
        path.to_owned()
    } else {
        path.with_extension("tmp")
    };
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    if !first {
        fs::rename(&target, path)?;
    }
    File::open(
        path.parent()
            .ok_or(AdapterError::InvalidEvidence("receipt parent"))?,
    )?
    .sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{AdapterError, NativePath, SourceIdentity, validate_plan_paths};
    use serde_json::json;
    use std::fs;

    #[test]
    fn plan_candidate_escape_is_refused_even_with_a_safe_keep_path() {
        let root = std::env::temp_dir().join(format!("flow-optiflow-root-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let source = SourceIdentity::observe(&root).unwrap();
        let keep = NativePath::from_path(&root.join("keep"));
        let escape = NativePath::from_path(&root.join("..").join("outside"));
        let plan = json!({
            "actions": [{
                "classification": "exact", "proposed_operation": "review_and_select",
                "keep_path": keep, "candidate_paths": [escape], "keep_alias_paths": [],
                "preconditions": []
            }]
        });
        assert!(matches!(
            validate_plan_paths(&plan, &source),
            Err(AdapterError::PathEscape)
        ));
        fs::remove_dir_all(root).unwrap();
    }
}
