//! Pinned native CLI translation. Domain algorithms stay in the owning provider.
//! Source-candidate execution requires explicit trusted-unconfined approval and
//! does not establish publication, qualification, creative review or provenance.
mod aniflow;
mod audio_policy;
mod audio_processing;
mod renderflow;
mod preparation;
mod released;
pub use released::{ReleasedApproval, ReleasedReceipt, ReleasedStatus, approve_released, run_released, status_released, resume_released};
use std::{collections::BTreeSet, ffi::OsString, fs::{self, File}, io::Write, path::{Path, PathBuf}, time::Instant};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::{CancellationSignal, optiflow::NativePath, release_cohort::{self, Cohort, Interface, Row, Tool}};
use crate::command::{self, Code, Error};

pub const REQUEST_SCHEMA: &str = "flow.native-request/v1";
pub const PLAN_SCHEMA: &str = "flow.native-plan/v1";
pub const RECEIPT_SCHEMA: &str = "flow.native-receipt/v1";
const MAX_DOCUMENT: u64 = 16_777_216;
fn error(code: Code, message: &str) -> Error { Error { code, message: message.into(), recovery_directory: None } }
fn cancelled(token: &dyn CancellationSignal) -> Result<(), Error> { if token.is_cancelled() { Err(error(Code::Cancelled, "native operation cancelled; preserve retained state")) } else { Ok(()) } }
fn cohort_error(e: release_cohort::Error) -> Error { match e { release_cohort::Error::Cancelled => error(Code::Cancelled, "observation cancelled"), _ => error(Code::StalePlan, "pinned file or release cohort could not be reobserved") } }
fn canonical<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> { serde_json::to_vec(&serde_json::to_value(value).map_err(|_| error(Code::InvalidInput, "native document encoding"))?).map_err(|_| error(Code::InvalidInput, "native document encoding")) }
fn digest<T: Serialize>(value: &T) -> Result<String, Error> { Ok(release_cohort::sha256(&canonical(value)?)) }
pub(crate) fn path(value: &NativePath) -> Result<PathBuf, Error> {
    let path = value.path().map_err(|_| error(Code::InvalidInput, "native path encoding"))?;
    if !path.is_absolute() || path.components().any(|c| matches!(c, std::path::Component::CurDir | std::path::Component::ParentDir)) { return Err(error(Code::InvalidInput, "normalized absolute native path required")); }
    Ok(path)
}
fn directory(value: &Path) -> Result<(), Error> { if value.canonicalize().ok().as_deref() == Some(value) && fs::metadata(value).is_ok_and(|m| m.is_dir()) { Ok(()) } else { Err(error(Code::InvalidInput, "canonical directory required")) } }
#[derive(Debug, Clone, Copy, Serialize)]
pub struct SourceRoute { pub tool: Tool, pub profile: &'static str, pub contract: &'static str, pub maximum_inputs: usize, pub state: &'static str }
pub const SOURCE_ROUTES: &[SourceRoute] = &[
    SourceRoute {tool:Tool::Aniflow,profile:"fixed-audio-gain-v1",contract:"aniflow.release-contract/v1",maximum_inputs:1,state:"implemented_selected_gain_requires_exact_pins_and_explicit_approval"},
    SourceRoute { tool: Tool::Aniflow, profile: "music-video-source-v1", contract: "aniflow.release-contract/v1", maximum_inputs: 1, state: "implemented_source_candidate_requires_exact_pins_and_explicit_approval" },
    SourceRoute { tool: Tool::Renderflow, profile: "canonical-build-v1", contract: "renderflow.release-contract/v2", maximum_inputs: 8192, state: "implemented_source_candidate_requires_exact_pins_and_explicit_approval" },
    SourceRoute {tool:Tool::Aniflow,profile:"audio-policy-check-v1",contract:"aniflow.release-contract/v1",maximum_inputs:1,state:"implemented_read_only_owner_check_requires_exact_pins_and_explicit_approval"},
];
fn profile_schema(row: &Row, inputs: usize) -> Result<&'static str, Error> {
    SOURCE_ROUTES.iter().find(|route| route.tool == row.tool && route.profile == row.profile && inputs > 0 && inputs <= route.maximum_inputs)
        .map(|route| route.contract).ok_or_else(|| error(Code::Unavailable, "native profile or exact input cardinality is unavailable"))
}
fn configuration_limit(tool: Tool) -> u64 { if tool == Tool::Renderflow { 1_048_576 } else { 131_072 } }
fn validate_effects(row: &Row) -> bool {
    if row.tool==Tool::Aniflow&&row.profile==audio_policy::PROFILE {
        return row.effects.iter().any(|e|e.as_str()=="read_originals")&&row.effects.iter().all(|e|["read_originals","write_new_outputs","direct_provider","trusted_unconfined"].contains(&e.as_str()));
    }
    ["read_originals", "write_new_outputs"].iter().all(|required| row.effects.iter().any(|effect| effect.as_str() == *required))
        && row.effects.iter().all(|effect| ["read_originals", "write_new_outputs", "direct_provider", "trusted_unconfined"].contains(&effect.as_str()))
}
fn validate_output_scope(request: &Request) -> Result<(), Error> {
    let root = path(&request.run_directory)?;
    for selected in [&request.cohort, &request.provider_executable, &request.release_descriptor, &request.configuration].into_iter().chain(request.inputs.iter()).chain(request.tool_directories.iter()) {
        let selected = path(selected)?;
        if root.starts_with(&selected) || selected.starts_with(&root) { return Err(error(Code::InvalidInput, "native output must be disjoint from selected source, executable, contracts and tools")); }
    }
    Ok(())
}
fn create_directory(value: &Path) -> Result<(), Error> {
    directory(value.parent().ok_or_else(|| error(Code::InvalidInput, "existing parent required"))?)?;
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)] { use std::os::unix::fs::DirBuilderExt; builder.mode(0o700); }
    builder.create(value).map_err(|_| error(Code::RecoveryRequired, "fresh native output directory required; preserve existing state"))?;
    directory(value)
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub timeout_ms: u64,
    pub max_input_bytes: u64,
    pub max_output_bytes: u64,
    pub max_total_output_bytes: u64,
    pub max_artifacts: usize,
}
impl Default for Limits { fn default() -> Self { Self { timeout_ms: 3_600_000, max_input_bytes: 8_589_934_592, max_output_bytes: 8_589_934_592, max_total_output_bytes: 17_179_869_184, max_artifacts: 256 } } }
impl Limits {
    pub(crate) fn validate(&self) -> Result<(), Error> { if self.timeout_ms == 0 || self.timeout_ms > 86_400_000 || self.max_input_bytes == 0 || self.max_input_bytes > 68_719_476_736 || self.max_output_bytes == 0 || self.max_output_bytes > 68_719_476_736 || self.max_total_output_bytes < self.max_output_bytes || self.max_total_output_bytes > 1_099_511_627_776 || self.max_artifacts == 0 || self.max_artifacts > 8192 { Err(error(Code::InvalidInput, "bounded native limits required")) } else { Ok(()) } }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: String,
    pub cohort: NativePath,
    pub cohort_sha256: String,
    pub row_id: String,
    pub provider_executable: NativePath,
    pub release_descriptor: NativePath,
    /// Selection order is retained. No widening a file to its containing tree.
    pub inputs: Vec<NativePath>,
    pub configuration: NativePath,
    pub run_directory: NativePath,
    /// Explicit tool search directories only; no inherited PATH, HOME or secrets.
    pub tool_directories: Vec<NativePath>,
    pub limits: Limits,
    /// Optional exact SDK setup marker; omission preserves existing plan encoding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preparation: Option<FilePin>,
}
impl Request {
    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > 262_144 { return Err(error(Code::InvalidInput, "native request bound")); }
        let value: Self = serde_json::from_slice(bytes).map_err(|_| error(Code::InvalidInput, "closed native request required"))?;
        value.validate()?; Ok(value)
    }
    fn validate(&self) -> Result<(), Error> {
        self.limits.validate()?;
        if self.schema_version != REQUEST_SCHEMA || !release_cohort::is_digest(&self.cohort_sha256) || self.row_id.is_empty() || self.row_id.len() > 4096 || self.inputs.is_empty() || self.inputs.len() > 8192 || self.tool_directories.len() > 32 { return Err(error(Code::InvalidInput, "native request identity or bounds")); }
        for value in [&self.cohort, &self.provider_executable, &self.release_descriptor, &self.configuration, &self.run_directory].into_iter().chain(self.inputs.iter()).chain(self.tool_directories.iter()) { path(value)?; }
        if let Some(pin) = &self.preparation { path(&pin.path)?; if !release_cohort::is_digest(&pin.sha256) || pin.byte_count==0 || pin.byte_count>16_384 { return Err(error(Code::InvalidInput, "preparation marker pin bounds")); } }
        let unique: BTreeSet<_> = self.inputs.iter().map(path).collect::<Result<_, _>>()?;
        if unique.len() != self.inputs.len() { return Err(error(Code::InvalidInput, "duplicate selected source")); }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilePin { pub path: NativePath, pub sha256: String, pub byte_count: u64 }
impl FilePin {
    pub(crate) fn observe(value: &NativePath, maximum: u64, token: &dyn CancellationSignal) -> Result<Self, Error> {
        let (byte_count, sha256) = release_cohort::observe_file(&path(value)?, maximum, token).map_err(cohort_error)?;
        if byte_count == 0 { return Err(error(Code::InvalidInput, "empty native input or artifact")); }
        Ok(Self { path: value.clone(), sha256, byte_count })
    }
    pub(crate) fn current(&self, maximum: u64, token: &dyn CancellationSignal) -> Result<(), Error> {
        if *self == Self::observe(&self.path, maximum, token)? { Ok(()) } else { Err(error(Code::StalePlan, "native pinned bytes changed; create a new reviewed plan")) }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub schema_version: String,
    pub plan_sha256: String,
    pub request: Request,
    pub row: Row,
    pub row_subject_sha256: String,
    pub provider: FilePin,
    pub descriptor: FilePin,
    pub configuration: FilePin,
    pub sources: Vec<FilePin>,
    pub release_evidence: String,
}
impl Plan {
    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() as u64 > MAX_DOCUMENT { return Err(error(Code::InvalidInput, "native plan bound")); }
        let value: Self = serde_json::from_slice(bytes).map_err(|_| error(Code::InvalidInput, "closed native plan required"))?;
        value.validate()?; Ok(value)
    }
    pub fn validate(&self) -> Result<(), Error> {
        self.request.validate()?;
        validate_output_scope(&self.request)?;
        let schema = profile_schema(&self.row, self.sources.len())?;
        let mut unsigned = self.clone(); unsigned.plan_sha256.clear();
        if self.schema_version != PLAN_SCHEMA || digest(&unsigned)? != self.plan_sha256 || release_cohort::subject(&self.row).map_err(cohort_error)? != self.row_subject_sha256
            || self.request.row_id != self.row.id || self.row.interface != Interface::FlowRoute || self.sources.len() != self.request.inputs.len()
            || self.provider.path != self.request.provider_executable || self.descriptor.path != self.request.release_descriptor || self.configuration.path != self.request.configuration
            || self.sources.iter().zip(&self.request.inputs).any(|(pin, input)| pin.path != *input)
            || !self.row.contracts.iter().any(|c| c.identifier == schema && c.sha256 == self.descriptor.sha256)
            || self.row.implementation != release_cohort::Implementation::Implemented || Some(self.row.target.as_str()) != host_target()
            || !validate_effects(&self.row)
            || self.release_evidence != "operator_pinned_source_candidate; publication_and_qualification_not_established" { return Err(error(Code::InvalidInput, "native plan identity or effects changed")); }
        for pin in [&self.provider, &self.descriptor, &self.configuration].into_iter().chain(self.sources.iter()) { if !release_cohort::is_digest(&pin.sha256) || pin.byte_count == 0 { return Err(error(Code::InvalidInput, "native file pin")); } }
        let Some(artifact) = &self.row.artifact else { return Err(error(Code::Unavailable, "cohort has no immutable provider artifact pin")); };
        if artifact.sha256 != self.provider.sha256 || artifact.byte_count != self.provider.byte_count { return Err(error(Code::InvalidInput, "cohort artifact and executable differ")); }
        Ok(())
    }
    pub(crate) fn require_current(&self, token: &dyn CancellationSignal) -> Result<(), Error> {
        self.validate()?; cancelled(token)?;
        let cohort_bytes = command::read_document(&path(&self.request.cohort)?, 1_048_576)?;
        if release_cohort::sha256(&cohort_bytes) != self.request.cohort_sha256 { return Err(error(Code::StalePlan, "selected cohort bytes changed")); }
        let cohort = Cohort::from_json_slice(&cohort_bytes).map_err(cohort_error)?;
        if cohort.flow_version != env!("CARGO_PKG_VERSION") || !cohort.rows.iter().any(|row| row == &self.row) { return Err(error(Code::StalePlan, "selected cohort row or Flow version changed")); }
        if let Some(pin) = &self.request.preparation { preparation::current(pin, &path(&self.request.run_directory)?, token)?; }
        self.provider.current(536_870_912, token)?; self.descriptor.current(MAX_DOCUMENT, token)?; self.configuration.current(configuration_limit(self.row.tool), token)?;
        for source in &self.sources { source.current(self.request.limits.max_input_bytes, token)?; }
        for value in &self.request.tool_directories { directory(&path(value)?)?; }
        Ok(())
    }
}
pub(crate) fn host_target() -> Option<&'static str> {
    if cfg!(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu")) { Some("x86_64-unknown-linux-gnu") }
    else if cfg!(all(target_os = "macos", target_arch = "aarch64")) { Some("aarch64-apple-darwin") }
    else if cfg!(all(target_os = "macos", target_arch = "x86_64")) { Some("x86_64-apple-darwin") }
    else { None }
}
/// Read-only preparation. It does not probe or launch the selected provider.
pub fn prepare(request: &Request, token: &dyn CancellationSignal) -> Result<Plan, Error> {
    request.validate()?; cancelled(token)?;
    let bytes = command::read_document(&path(&request.cohort)?, 1_048_576)?;
    if release_cohort::sha256(&bytes) != request.cohort_sha256 { return Err(error(Code::StalePlan, "exact selected cohort digest required")); }
    let cohort = Cohort::from_json_slice(&bytes).map_err(cohort_error)?;
    if cohort.flow_version != env!("CARGO_PKG_VERSION") { return Err(error(Code::Unavailable, "cohort Flow version differs")); }
    let row = cohort.rows.iter().find(|r| r.id == request.row_id).ok_or_else(|| error(Code::Unavailable, "selected release row is absent"))?.clone();
    if row.interface != Interface::FlowRoute || row.implementation != release_cohort::Implementation::Implemented || Some(row.target.as_str()) != host_target() { return Err(error(Code::Unavailable, "selected native target/interface is unsupported on this host")); }
    if !validate_effects(&row) { return Err(error(Code::InvalidInput, "native row must declare source reads and new output writes without additional effects")); }
    let descriptor = FilePin::observe(&request.release_descriptor, MAX_DOCUMENT, token)?;
    let description: Value = serde_json::from_slice(&command::read_document(&path(&request.release_descriptor)?, MAX_DOCUMENT)?).map_err(|_| error(Code::InvalidInput, "native descriptor JSON required"))?;
    let schema = profile_schema(&row, request.inputs.len())?;
    let tool = match row.tool { Tool::Aniflow => "aniflow", Tool::Renderflow => "renderflow", _ => return Err(error(Code::Unavailable, "native tool unavailable")) };
    if description["schema_version"] != schema || description["tool"] != tool
        || !description["profiles"].as_array().is_some_and(|profiles| profiles.iter().any(|p| p["id"] == row.profile && p["source_mutation"] == false))
        || !row.contracts.iter().any(|c| c.identifier == schema && c.sha256 == descriptor.sha256)
        || row.artifact.as_ref().is_none_or(|a| description["version"] != a.version) { return Err(error(Code::Unavailable, "pinned native profile/descriptor/release contract differs")); }
    match row.tool { Tool::Aniflow if row.profile==audio_policy::PROFILE => audio_policy::validate_configuration(request)?, Tool::Aniflow if audio_processing::supports(&row.profile) => audio_processing::validate_configuration(request)?, Tool::Aniflow => aniflow::validate_configuration(request)?, Tool::Renderflow => renderflow::validate_configuration(request)?, _ => return Err(error(Code::Unavailable, "native configuration route unavailable")) }
    let root = path(&request.run_directory)?;
    directory(root.parent().ok_or_else(|| error(Code::InvalidInput, "existing native output parent required"))?)?;
    if fs::symlink_metadata(&root).is_ok() { return Err(error(Code::RecoveryRequired, "fresh run directory required; use status/resume for retained attempts")); }
    validate_output_scope(request)?;
    let configuration_maximum = configuration_limit(row.tool);
    let mut plan = Plan { schema_version: PLAN_SCHEMA.into(), plan_sha256: String::new(), row_subject_sha256: release_cohort::subject(&row).map_err(cohort_error)?, row,
        provider: FilePin::observe(&request.provider_executable, 536_870_912, token)?, descriptor,
        configuration: FilePin::observe(&request.configuration, configuration_maximum, token)?, sources: request.inputs.iter().map(|v| FilePin::observe(v, request.limits.max_input_bytes, token)).collect::<Result<_, _>>()?,
        request: request.clone(), release_evidence: "operator_pinned_source_candidate; publication_and_qualification_not_established".into() };
    plan.plan_sha256 = digest(&plan)?; plan.require_current(token)?; Ok(plan)
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Approval { pub schema_version: String, pub plan_sha256: String, pub approved_by: String, pub trusted_unconfined: bool, pub unqualified_candidate: bool }
pub fn approve(plan: &Plan, approved_by: String, trusted_unconfined: bool, unqualified_candidate: bool) -> Result<Approval, Error> {
    plan.validate()?;
    let value = Approval { schema_version: "flow.native-approval/v1".into(), plan_sha256: plan.plan_sha256.clone(), approved_by, trusted_unconfined, unqualified_candidate };
    value.validate(plan)?; Ok(value)
}
/// Create a reviewable plan/approval document outside every selected input and
/// reserved run tree. This does not launch the provider or create run state.
pub fn save_document<T: Serialize>(plan: &Plan, output: &Path, value: &T) -> Result<(), Error> {
    plan.validate()?;
    let output = command::absolute(output)?;
    for selected in [&plan.request.cohort, &plan.request.provider_executable, &plan.request.release_descriptor, &plan.request.configuration, &plan.request.run_directory].into_iter().chain(plan.request.inputs.iter()).chain(plan.request.tool_directories.iter()) {
        let selected = path(selected)?;
        if output.starts_with(&selected) || selected.starts_with(&output) { return Err(error(Code::InvalidInput, "native plan/approval document must be separate from all selected inputs and reserved run output")); }
    }
    if let Some(pin) = &plan.request.preparation { let root=path(&pin.path)?.parent().ok_or_else(||error(Code::InvalidInput,"preparation parent required"))?.to_path_buf(); if output.starts_with(root) { return Err(error(Code::InvalidInput,"saved document overlaps preparation bundle")); } }
    write(&output, value)
}
impl Approval {
    fn validate(&self, plan: &Plan) -> Result<(), Error> { if self.schema_version != "flow.native-approval/v1" || self.plan_sha256 != plan.plan_sha256 || self.approved_by.is_empty() || self.approved_by.len() > 256 || self.approved_by.chars().any(char::is_control) || !self.trusted_unconfined || !self.unqualified_candidate { Err(error(Code::ApprovalRequired, "exact plan approval must acknowledge trusted-unconfined, unqualified native candidate execution")) } else { Ok(()) } }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State { Complete, RecoveryRequired }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub schema_version: String, pub plan_sha256: String, pub row_subject_sha256: String,
    pub native_run: NativePath, pub native_intent_sha256: String,
    pub acceptance_file: NativePath, pub acceptance_sha256: String,
    pub artifacts: Vec<FilePin>, pub state: State,
    pub release_qualified: bool, pub creative_approval_inferred: bool, pub publication_authority: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct Status { pub schema_version: String, pub plan_sha256: String, pub state: State, pub receipt: Option<Receipt>, pub domain_evidence: String, pub next_action: String }
struct RunLock(File);
impl Drop for RunLock { fn drop(&mut self) { let _ = FileExt::unlock(&self.0); } }
fn lock(root: &Path, first: bool) -> Result<RunLock, Error> {
    directory(root)?;
    let mut options = fs::OpenOptions::new(); options.read(true).write(true).create_new(first);
    #[cfg(unix)] { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600).custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_CLOEXEC); }
    let file = options.open(root.join("operation.lock")).map_err(|_| error(Code::RecoveryRequired, "retained native operation lock unavailable"))?;
    file.try_lock_exclusive().map_err(|_| error(Code::RecoveryRequired, "native operation is active; preserve current state"))?;
    Ok(RunLock(file))
}
fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    if bytes.len() as u64 > MAX_DOCUMENT { return Err(error(Code::InvalidInput, "native evidence document bound")); }
    directory(path.parent().ok_or_else(|| error(Code::InvalidInput, "native evidence parent"))?)?;
    let mut options = fs::OpenOptions::new(); options.write(true).create_new(true);
    #[cfg(unix)] { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600).custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_CLOEXEC); }
    let mut file = options.open(path).map_err(|_| error(Code::RecoveryRequired, "native evidence already exists or cannot be reserved"))?;
    file.write_all(bytes).and_then(|_| file.sync_all()).map_err(|_| error(Code::Io, "native evidence write incomplete; preserve attempt"))?;
    File::open(path.parent().ok_or_else(|| error(Code::Io, "evidence parent"))?).and_then(|f| f.sync_all()).map_err(|_| error(Code::Io, "native evidence synchronization"))
}
fn write<T: Serialize>(path: &Path, value: &T) -> Result<(), Error> { let mut bytes = serde_json::to_vec_pretty(value).map_err(|_| error(Code::InvalidInput, "native evidence encoding"))?; bytes.push(b'\n'); write_bytes(path, &bytes) }
fn read<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, Error> { serde_json::from_slice(&command::read_document(path, MAX_DOCUMENT)?).map_err(|_| error(Code::RecoveryRequired, "retained native evidence malformed")) }
struct Context<'a> { plan: &'a Plan, root: PathBuf, evidence: PathBuf, started: Instant, token: &'a dyn CancellationSignal, release_evidence: Option<&'a crate::release_evidence::VerifiedEvidence> }
impl Context<'_> {
    fn require_current(&self) -> Result<(),Error> {
        self.plan.require_current(self.token)?;
        if let Some(evidence)=self.release_evidence { evidence.admit(self.plan,self.token)?; }
        Ok(())
    }
    fn invoke(&self, label: &str, args: &[OsString]) -> Result<crate::runner::NativeCapture, Error> {
        self.require_current()?;
        let elapsed: u64 = self.started.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
        let remaining = self.plan.request.limits.timeout_ms.checked_sub(elapsed).filter(|v| *v > 0).ok_or_else(|| error(Code::RecoveryRequired, "native operation deadline exceeded; preserve attempt"))?;
        let mut environment = vec![];
        if !self.plan.request.tool_directories.is_empty() {
            let paths = self.plan.request.tool_directories.iter().map(path).collect::<Result<Vec<_>, _>>()?;
            environment.push((OsString::from("PATH"), std::env::join_paths(paths).map_err(|_| error(Code::InvalidInput, "tool directory PATH encoding"))?));
        }
        let captured = crate::runner::capture_native(&path(&self.plan.provider.path)?, args, &environment, remaining, MAX_DOCUMENT, 65_536, self.token)
            .map_err(|e| match e { crate::runner::ProcessRunnerError::Cancelled { .. } => error(Code::Cancelled, "native direct child cancelled; inspect retained attempt before explicit resume"), _ => error(Code::RecoveryRequired, "native process failed or exceeded bounds; inspect retained attempt") })?;
        write_bytes(&self.evidence.join(format!("{label}.stdout.json")), &captured.stdout)?;
        write_bytes(&self.evidence.join(format!("{label}.stderr")), &captured.stderr)?;
        write(&self.evidence.join(format!("{label}.exit.json")), &serde_json::json!({"code":captured.code}))?;
        self.require_current()?;
        Ok(captured)
    }
    fn native_root(&self) -> PathBuf { self.root.join("native") }
    fn require_native_run(&self, run: &Path) -> Result<(), Error> { directory(run)?; if !run.starts_with(self.native_root()) || run == self.native_root() { return Err(error(Code::ProviderRefused, "native run escaped reserved output parent")); } Ok(()) }
    fn observe_artifacts(&self, entries: Vec<(PathBuf, String, u64)>) -> Result<Vec<FilePin>, Error> {
        if entries.is_empty() || entries.len() > self.plan.request.limits.max_artifacts { return Err(error(Code::ProviderRefused, "native accepted artifact count")); }
        let mut seen = BTreeSet::new(); let mut total = 0_u64; let mut outputs = vec![];
        for (file, sha256, byte_count) in entries {
            if self.started.elapsed().as_millis() >= u128::from(self.plan.request.limits.timeout_ms) { return Err(error(Code::RecoveryRequired, "native artifact observation deadline exceeded")); }
            if !file.starts_with(self.native_root()) || !seen.insert(file.clone()) || !release_cohort::is_digest(&sha256) || byte_count == 0 { return Err(error(Code::ProviderRefused, "native artifact identity or confinement")); }
            total = total.checked_add(byte_count).ok_or_else(|| error(Code::ProviderRefused, "native output size overflow"))?;
            if total > self.plan.request.limits.max_total_output_bytes { return Err(error(Code::ProviderRefused, "native total output bytes")); }
            let observed = FilePin::observe(&NativePath::from_path(&file), self.plan.request.limits.max_output_bytes, self.token)?;
            if observed.sha256 != sha256 || observed.byte_count != byte_count { return Err(error(Code::ProviderRefused, "native accepted output bytes changed")); }
            outputs.push(observed);
        }
        Ok(outputs)
    }
    fn finish(&self, native_run: &Path, native_intent: &str, evidence: &Value, artifacts: Vec<FilePin>) -> Result<Receipt, Error> {
        self.require_current()?; self.require_native_run(native_run)?;
        if artifacts.iter().any(|artifact| !path(&artifact.path).is_ok_and(|p| p.starts_with(native_run))) { return Err(error(Code::ProviderRefused, "accepted artifact belongs to another native run")); }
        if self.started.elapsed().as_millis() >= u128::from(self.plan.request.limits.timeout_ms) { return Err(error(Code::RecoveryRequired, "native acceptance deadline exceeded")); }
        if !release_cohort::is_digest(native_intent) { return Err(error(Code::ProviderRefused, "native intent digest")); }
        let acceptance = self.evidence.join("domain-acceptance.json"); write(&acceptance, evidence)?;
        let bytes = command::read_document(&acceptance, MAX_DOCUMENT)?;
        let value = Receipt { schema_version: RECEIPT_SCHEMA.into(), plan_sha256: self.plan.plan_sha256.clone(), row_subject_sha256: self.plan.row_subject_sha256.clone(), native_run: NativePath::from_path(native_run), native_intent_sha256: native_intent.into(), acceptance_file: NativePath::from_path(&acceptance), acceptance_sha256: release_cohort::sha256(&bytes), artifacts, state: State::Complete, release_qualified: false, creative_approval_inferred: false, publication_authority: false };
        write(&self.root.join("accepted.json"), &value)?; Ok(value)
    }
}
fn context_error(mut value: Error, root: &Path) -> Error { value.recovery_directory = Some(NativePath::from_path(root)); value }
/// Only this explicit operation can launch a new native run. Approval is a local
/// operator declaration, not a signature or host isolation guarantee.
pub fn run(plan: &Plan, approval: &Approval, token: &dyn CancellationSignal) -> Result<Receipt, Error> {
    plan.require_current(token)?; approval.validate(plan)?;
    run_authorized(plan,approval,None,token)
}
fn run_authorized<T: Serialize>(plan:&Plan,approval:&T,release_evidence:Option<&crate::release_evidence::VerifiedEvidence>,token:&dyn CancellationSignal)->Result<Receipt,Error>{
    plan.require_current(token)?;if let Some(e)=release_evidence{e.admit(plan,token)?;}
    let root = path(&plan.request.run_directory)?; create_directory(&root)?; let _lock = lock(&root, true)?;
    let action = (|| {
        write(&root.join("operation-plan.json"), plan)?; write(&root.join("operation-approval.json"), approval)?;
        for (name, selected) in [("configuration-snapshot.json", &plan.request.configuration), ("descriptor-snapshot.json", &plan.request.release_descriptor), ("cohort-snapshot.json", &plan.request.cohort)] {
            write_bytes(&root.join(name), &command::read_document(&path(selected)?, MAX_DOCUMENT)?)?;
        }
        create_directory(&root.join("native"))?; create_directory(&root.join("attempt-0000"))?;
        let context = Context { plan, root: root.clone(), evidence: root.join("attempt-0000"), started: Instant::now(), token, release_evidence };
        match plan.row.tool { Tool::Aniflow if plan.row.profile==audio_policy::PROFILE => audio_policy::execute(&context,None), Tool::Aniflow if audio_processing::supports(&plan.row.profile) => audio_processing::execute(&context,None), Tool::Aniflow => aniflow::execute(&context, None), Tool::Renderflow => renderflow::execute(&context, None), _ => Err(error(Code::Unavailable, "native route unavailable")) }
    })(); action.map_err(|e| context_error(e, &root))
}
fn basis(root: &Path) -> Result<(Plan, Approval), Error> {
    let plan = Plan::from_json_slice(&command::read_document(&root.join("operation-plan.json"), MAX_DOCUMENT)?)?;
    let approval: Approval = read(&root.join("operation-approval.json"))?; approval.validate(&plan)?;
    if path(&plan.request.run_directory)? != root { return Err(error(Code::RecoveryRequired, "native operation moved from approved destination")); }
    Ok((plan, approval))
}
fn current_status(root: &Path, plan: &Plan, token: &dyn CancellationSignal) -> Result<Status, Error> {
    plan.require_current(token)?;
    let accepted = root.join("accepted.json");
    if !accepted.try_exists().map_err(|_| error(Code::Io, "native receipt state unavailable"))? { return Ok(Status { schema_version: "flow.native-status/v1".into(), plan_sha256: plan.plan_sha256.clone(), state: State::RecoveryRequired, receipt: None, domain_evidence: "no_committed_domain_acceptance".into(), next_action: "inspect retained attempt; explicitly identify an existing native run for resume; never guess or replay".into() }); }
    let receipt: Receipt = read(&accepted)?;
    let acceptance = path(&receipt.acceptance_file)?;
    let native_run = path(&receipt.native_run)?;
    directory(&native_run)?;
    if receipt.schema_version != RECEIPT_SCHEMA || receipt.plan_sha256 != plan.plan_sha256 || receipt.row_subject_sha256 != plan.row_subject_sha256 || receipt.state != State::Complete || receipt.release_qualified || receipt.creative_approval_inferred || receipt.publication_authority
        || !release_cohort::is_digest(&receipt.native_intent_sha256) || !native_run.starts_with(root.join("native")) || native_run == root.join("native") || !acceptance.starts_with(root) || receipt.artifacts.is_empty() || receipt.artifacts.len() > plan.request.limits.max_artifacts
        || release_cohort::sha256(&command::read_document(&acceptance, MAX_DOCUMENT)?) != receipt.acceptance_sha256 { return Err(error(Code::RecoveryRequired, "retained native acceptance identity changed")); }
    let domain: Value = read(&acceptance)?;
    match plan.row.tool { Tool::Aniflow if plan.row.profile==audio_policy::PROFILE => audio_policy::validate_retained(plan,&receipt,&domain)?, Tool::Aniflow if audio_processing::supports(&plan.row.profile) => audio_processing::validate_retained(plan,&receipt,&domain)?, Tool::Aniflow => aniflow::validate_retained(plan, &receipt, &domain)?, Tool::Renderflow => renderflow::validate_retained(plan, &receipt, &domain)?, _ => return Err(error(Code::Unavailable, "native retained route unsupported")) }
    let mut seen = BTreeSet::new(); let mut total = 0_u64;
    for artifact in &receipt.artifacts {
        let file = path(&artifact.path)?;
        if !file.starts_with(&native_run) || !seen.insert(file) { return Err(error(Code::RecoveryRequired, "retained native output scope changed")); }
        total = total.checked_add(artifact.byte_count).ok_or_else(|| error(Code::RecoveryRequired, "native output total overflow"))?;
        if total > plan.request.limits.max_total_output_bytes { return Err(error(Code::RecoveryRequired, "native output total bound")); }
        artifact.current(plan.request.limits.max_output_bytes, token)?;
    }
    Ok(Status { schema_version: "flow.native-status/v1".into(), plan_sha256: plan.plan_sha256.clone(), state: State::Complete, receipt: Some(receipt), domain_evidence: "retained_native_acceptance_and_current_payload_hashes; no new native domain probe".into(), next_action: "retain evidence; source-candidate completion does not qualify or publish the release".into() })
}
/// Read-only local observation; never invokes a provider or repairs state.
pub fn status(root: &Path, token: &dyn CancellationSignal) -> Result<Status, Error> { let _lock = lock(root, false)?; let (plan, _) = basis(root)?; current_status(root, &plan, token).map_err(|e| context_error(e, root)) }
/// Read-only domain/payload evidence for an exact retained plan, independently
/// of the serialized authority mode. This cannot authorize run or resume and
/// does not reverify publisher/native admission. Consumers must establish those
/// facts through their separate opaque trust boundary when claiming them.
pub fn observe_receipt(root:&Path,expected_plan_sha256:&str,token:&dyn CancellationSignal)->Result<Status,Error>{
    let _lock=lock(root,false)?;
    let plan=Plan::from_json_slice(&command::read_document(&root.join("operation-plan.json"),MAX_DOCUMENT)?)?;
    if plan.plan_sha256!=expected_plan_sha256||path(&plan.request.run_directory)?!=root{return Err(error(Code::StalePlan,"exact retained native plan identity required"));}
    let mut value=current_status(root,&plan,token)?;
    value.domain_evidence="retained native domain acceptance and current payload bytes only; serialized approval and release admission were not reverified".into();
    value.next_action="use separate opaque release admission for qualification or released execution; this readback grants no authority".into();
    if plan.row.profile==audio_policy::PROFILE {value.domain_evidence="retained audio-policy snapshot and current selected media bytes; current QC owner was not rechecked".into();value.next_action="start a fresh explicitly approved owner check for current audio delivery readiness".into();}
    Ok(value)
}
#[derive(Debug,Clone,PartialEq,Eq,Serialize)]
pub struct ArtifactDescription{pub artifact_id:String,pub purpose:String,pub file:FilePin}

/// Retained owner snapshot with current selected file pins. This does not rerun
/// the QC owner; use a fresh approved native check for current delivery readiness.
pub fn audio_policy_snapshot(root:&Path,expected_plan_sha256:&str,token:&dyn CancellationSignal)->Result<Value,Error>{
    let status=observe_receipt(root,expected_plan_sha256,token)?;
    let receipt=status.receipt.ok_or_else(||error(Code::RecoveryRequired,"complete audio owner check required"))?;
    let value:Value=read(&path(&receipt.acceptance_file)?)?;
    if value["schema_version"]!="flow.aniflow-audio-policy-acceptance/v1"{return Err(error(Code::InvalidInput,"selected receipt is not an audio policy owner check"));}
    Ok(value["assessment"].clone())
}
pub(crate) fn audio_policy_protected_paths(request:&Request)->Result<Vec<PathBuf>,Error>{audio_policy::protected_paths(request)}
pub(crate) fn retained_audio_policy_record(value:&Value,sha256:&str,byte_count:u64)->Result<Value,Error>{audio_policy::retained_record(value,sha256,byte_count)}
pub(crate) fn retained_audio_policy_paths(assessment:&Value)->Result<Vec<PathBuf>,Error>{audio_policy::retained_paths(assessment)}
/// Translate accepted owner artifact identities and roles without inferring a
/// master from a filename or invoking a provider.
pub fn artifact_inventory(root:&Path,expected_plan_sha256:&str,token:&dyn CancellationSignal)->Result<Vec<ArtifactDescription>,Error>{
    let observed=observe_receipt(root,expected_plan_sha256,token)?;
    let receipt=observed.receipt.ok_or_else(||error(Code::RecoveryRequired,"complete native acceptance required for artifact inventory"))?;
    let bytes=command::read_document(&path(&receipt.acceptance_file)?,MAX_DOCUMENT)?;
    if release_cohort::sha256(&bytes)!=receipt.acceptance_sha256{return Err(error(Code::StalePlan,"domain acceptance changed during artifact inventory"));}
    let domain:Value=serde_json::from_slice(&bytes).map_err(|_|error(Code::RecoveryRequired,"domain acceptance malformed"))?;
    let mut result=vec![];let mut ids=BTreeSet::new();
    if domain["schema_version"]=="flow.aniflow-audio-processing-acceptance/v1"{
        result=audio_processing::inventory(&receipt,&domain)?;
    }else if domain["schema_version"]=="flow.aniflow-audio-policy-acceptance/v1"{
        if receipt.artifacts.len()!=1{return Err(error(Code::RecoveryRequired,"one retained policy assessment required"));}
        result.push(ArtifactDescription {artifact_id:"audio_policy_assessment".into(),purpose:"supporting_document".into(),file:receipt.artifacts[0].clone()});
    }else if domain["schema_version"]=="flow.aniflow-domain-acceptance/v1"{
        for a in domain["native_completion"]["artifacts"].as_array().ok_or_else(||error(Code::RecoveryRequired,"native video artifact inventory missing"))?{
            let id=a["artifact_id"].as_str().ok_or_else(||error(Code::RecoveryRequired,"native video artifact identity missing"))?;
            let purpose=a["variant"].as_str().ok_or_else(||error(Code::RecoveryRequired,"native video artifact role missing"))?;
            if !["clean_archival_master","destination_copy"].contains(&purpose)||!ids.insert(id.to_owned()){return Err(error(Code::RecoveryRequired,"native video artifact identity/role differs"));}
            let locator=PathBuf::from(a["locator"].as_str().ok_or_else(||error(Code::RecoveryRequired,"native video locator missing"))?);
            let file=receipt.artifacts.iter().find(|p|path(&p.path).ok().as_deref()==Some(locator.as_path())&&a["sha256"]==p.sha256&&a["byte_count"].as_u64()==Some(p.byte_count)).ok_or_else(||error(Code::RecoveryRequired,"native video artifact differs from committed receipt"))?;
            result.push(ArtifactDescription{artifact_id:id.into(),purpose:purpose.into(),file:file.clone()});
        }
    }else if domain["schema_version"]=="flow.renderflow-domain-acceptance/v1"{
        for a in domain["native_publications"]["verified_publications"].as_array().ok_or_else(||error(Code::RecoveryRequired,"native publication inventory missing"))?{
            let id=a["target_id"].as_str().ok_or_else(||error(Code::RecoveryRequired,"native target identity missing"))?;
            let relative=a["locator"].as_str().and_then(|p|p.strip_prefix("bundle:")).ok_or_else(||error(Code::RecoveryRequired,"native publication locator missing"))?;
            if !crate::publisher_report::safe_member(relative)||!ids.insert(id.to_owned()){return Err(error(Code::RecoveryRequired,"native publication identity/path differs"));}
            let file=path(&receipt.native_run)?.join(relative);
            let pin=receipt.artifacts.iter().find(|p|path(&p.path).ok().as_deref()==Some(file.as_path())&&a["sha256"]==p.sha256&&a["byte_count"].as_u64()==Some(p.byte_count)).ok_or_else(||error(Code::RecoveryRequired,"native publication differs from committed receipt"))?;
            result.push(ArtifactDescription{artifact_id:id.into(),purpose:"domain_publication".into(),file:pin.clone()});
        }
    }else{return Err(error(Code::Unavailable,"native artifact inventory schema unsupported"));}
    if result.len()!=receipt.artifacts.len(){return Err(error(Code::RecoveryRequired,"native accepted artifact inventory differs"));}
    FilePin{path:receipt.acceptance_file,sha256:receipt.acceptance_sha256,byte_count:bytes.len() as u64}.current(MAX_DOCUMENT,token)?;
    Ok(result)
}
/// Resume is explicit and names an existing native run. The provider rechecks its
/// own intent and accepted prefix. No directory scan, substitute run or replay.
pub fn resume(root: &Path, native_run: &Path, token: &dyn CancellationSignal) -> Result<Receipt, Error> {
    let _lock = lock(root, false)?; let (plan, approval) = basis(root)?; plan.require_current(token)?; approval.validate(&plan)?;
    resume_authorized(root,native_run,&plan,None,token)
}
fn resume_authorized(root:&Path,native_run:&Path,plan:&Plan,release_evidence:Option<&crate::release_evidence::VerifiedEvidence>,token:&dyn CancellationSignal)->Result<Receipt,Error>{
    plan.require_current(token)?;if let Some(e)=release_evidence{e.admit(plan,token)?;}
    if let Some(receipt) = current_status(root, plan, token)?.receipt {
        if path(&receipt.native_run)? != native_run { return Err(error(Code::RecoveryRequired, "resume native run differs from accepted operation")); }
        return Ok(receipt);
    }
    let mut attempt = None;
    for index in 1..=128 { let candidate = root.join(format!("attempt-{index:04}")); if !candidate.try_exists().map_err(|_| error(Code::Io, "resume attempt inventory"))? { create_directory(&candidate)?; attempt = Some(candidate); break; } }
    let evidence = attempt.ok_or_else(|| error(Code::RecoveryRequired, "retained resume attempt bound reached"))?;
    let context = Context { plan, root: root.to_owned(), evidence, started: Instant::now(), token, release_evidence };
    context.require_native_run(native_run)?;
    match plan.row.tool { Tool::Aniflow if plan.row.profile==audio_policy::PROFILE => audio_policy::execute(&context,Some(native_run)), Tool::Aniflow if audio_processing::supports(&plan.row.profile) => audio_processing::execute(&context,Some(native_run)), Tool::Aniflow => aniflow::execute(&context, Some(native_run)), Tool::Renderflow => renderflow::execute(&context, Some(native_run)), _ => Err(error(Code::Unavailable, "native route unavailable")) }.map_err(|e| context_error(e, root))
}
