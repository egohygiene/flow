//! Initial product-command SDK over the released native read-only adapter.
//! No implicit provider launch, installation or generic graph/receipt conversion.
pub mod delivery;
mod documents;
use crate::{
    CancellationSignal,
    optiflow::{
        self, NativePath, OptiflowReadOnlyAdapter, ProviderIdentity, ReadOnlyReceipt,
        ReceiptStatus, SourceIdentity,
    },
};
pub use documents::{read_document, write_document};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Code {
    Usage,
    InvalidInput,
    Unavailable,
    StalePlan,
    ApprovalRequired,
    RecoveryRequired,
    Cancelled,
    Io,
    ProviderRefused,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Error {
    pub code: Code,
    pub message: String,
    pub recovery_directory: Option<NativePath>,
}
impl Error {
    pub fn exit_code(&self) -> u8 {
        match self.code {
            Code::Usage => 2,
            Code::InvalidInput | Code::StalePlan => 3,
            Code::Unavailable => 4,
            Code::ApprovalRequired => 5,
            Code::ProviderRefused => 6,
            Code::RecoveryRequired => 7,
            Code::Io => 8,
            Code::Cancelled => 130,
        }
    }
}
fn error(code: Code, message: &str) -> Error {
    Error {
        code,
        message: message.into(),
        recovery_directory: None,
    }
}
fn native(e: optiflow::AdapterError) -> Error {
    let code = match e {
        optiflow::AdapterError::Cancelled => Code::Cancelled,
        optiflow::AdapterError::Unavailable
        | optiflow::AdapterError::Incompatible
        | optiflow::AdapterError::UnsupportedCapability(_) => Code::Unavailable,
        optiflow::AdapterError::SourceChanged => Code::StalePlan,
        _ => Code::ProviderRefused,
    };
    error(
        code,
        "selected native adapter refused; inspect private retained evidence",
    )
}
fn cancelled(token: &dyn CancellationSignal) -> Result<(), Error> {
    if token.is_cancelled() {
        Err(error(
            Code::Cancelled,
            "operation cancelled; preserve any existing state",
        ))
    } else {
        Ok(())
    }
}
fn canonical<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    let json = serde_json::to_value(value)
        .map_err(|_| error(Code::InvalidInput, "document cannot be represented"))?;
    serde_json::to_vec(&json)
        .map_err(|_| error(Code::InvalidInput, "canonical document unavailable"))
}
pub(crate) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn digest<T: Serialize>(value: &T) -> Result<String, Error> {
    Ok(hash(&canonical(value)?))
}
fn pin(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub(crate) fn absolute(path: &Path) -> Result<PathBuf, Error> {
    let p = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|_| error(Code::Io, "working directory unavailable"))?
            .join(path)
    };
    if p.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(error(
            Code::InvalidInput,
            "parent traversal in selected path refused",
        ));
    }
    Ok(p.components()
        .filter(|c| !matches!(c, Component::CurDir))
        .collect())
}
fn selected_path(path: &NativePath) -> Result<PathBuf, Error> {
    let p = path.path().map_err(native)?;
    if !p.is_absolute()
        || p.components()
            .any(|c| matches!(c, Component::CurDir | Component::ParentDir))
    {
        return Err(error(
            Code::InvalidInput,
            "normalized absolute native path required",
        ));
    }
    Ok(p)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Route {
    OptiflowReadOnly,
    RenderflowPublication,
    AniflowTemporal,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: String,
    pub route: Route,
    pub provider_executable: NativePath,
    pub source_root: NativePath,
    pub run_directory: NativePath,
}
impl Request {
    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > 16_384 {
            return Err(error(Code::InvalidInput, "command request exceeds16KiB"));
        }
        let value: Self = serde_json::from_slice(bytes)
            .map_err(|_| error(Code::InvalidInput, "closed command request required"))?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<(), Error> {
        if self.schema_version != "flow.command-request/v1" {
            return Err(error(Code::InvalidInput, "unknown command request"));
        }
        for p in [
            &self.provider_executable,
            &self.source_root,
            &self.run_directory,
        ] {
            selected_path(p)?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Effects {
    pub source_read_only: bool,
    pub local_evidence_write: bool,
    pub direct_provider_process: bool,
    pub trusted_unconfined: bool,
    pub network_granted: bool,
    pub source_mutation_granted: bool,
}
fn effects() -> Effects {
    Effects {
        source_read_only: true,
        local_evidence_write: true,
        direct_provider_process: true,
        trusted_unconfined: true,
        network_granted: false,
        source_mutation_granted: false,
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub schema_version: String,
    pub plan_sha256: String,
    pub request: Request,
    pub source_scope: SourceIdentity,
    pub provider: ProviderIdentity,
    pub effects: Effects,
    pub content_binding: String,
}
impl Plan {
    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > 32_768 {
            return Err(error(Code::InvalidInput, "command plan exceeds32KiB"));
        }
        let value: Self = serde_json::from_slice(bytes)
            .map_err(|_| error(Code::InvalidInput, "closed command plan required"))?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<(), Error> {
        self.request.validate()?;
        let mut copy = self.clone();
        copy.plan_sha256.clear();
        let profile = optiflow::RELEASE_PROFILES
            .iter()
            .find(|p| p.target == self.provider.target);
        if self.schema_version != "flow.command-plan/v1"
            || self.request.route != Route::OptiflowReadOnly
            || !pin(&self.plan_sha256)
            || digest(&copy)? != self.plan_sha256
            || self.effects != effects()
            || self.content_binding
                != "source_scope_only; native scan binds actual members at execution"
            || self.source_scope.root != self.request.source_root
            || self.provider.version != optiflow::PROVIDER_VERSION
            || self.provider.source_revision != optiflow::PROVIDER_SOURCE_REVISION
            || self.provider.interface != optiflow::COMMAND_RESULT_SCHEMA
            || profile.is_none_or(|p| {
                p.archive_sha256 != self.provider.archive_sha256
                    || p.executable_sha256 != self.provider.executable_sha256
            })
        {
            return Err(error(
                Code::InvalidInput,
                "command plan identity/effects/pinned release differ",
            ));
        }
        Ok(())
    }
}
fn run_destination(request: &Request) -> Result<PathBuf, Error> {
    let source = selected_path(&request.source_root)?;
    let provider = selected_path(&request.provider_executable)?;
    let root = selected_path(&request.run_directory)?;
    let parent = root.parent().ok_or_else(|| {
        error(
            Code::InvalidInput,
            "run directory requires an existing parent",
        )
    })?;
    if parent.canonicalize().ok().as_deref() != Some(parent)
        || !fs::metadata(parent).is_ok_and(|m| m.is_dir())
        || root.starts_with(&source)
        || source.starts_with(&root)
        || provider.starts_with(&root)
        || root.starts_with(&provider)
    {
        return Err(error(
            Code::InvalidInput,
            "run output must be canonical and disjoint from source/provider",
        ));
    }
    Ok(root)
}
/// Read-only scope and exact release-byte observation, never a native version probe.
pub fn plan(request: &Request, token: &dyn CancellationSignal) -> Result<Plan, Error> {
    request.validate()?;
    cancelled(token)?;
    if request.route != Route::OptiflowReadOnly {
        return Err(error(
            Code::Unavailable,
            "requested immutable released-provider route is unavailable; no fallback",
        ));
    }
    let root = run_destination(request)?;
    if fs::symlink_metadata(&root).is_ok() {
        return Err(error(
            Code::RecoveryRequired,
            "run output already exists; inspect its status or select a new directory",
        ));
    }
    let source = selected_path(&request.source_root)?;
    if source.canonicalize().ok().as_deref() != Some(source.as_path()) {
        return Err(error(Code::InvalidInput, "source root must be canonical"));
    }
    let scope = SourceIdentity::observe(&source).map_err(native)?;
    let provider = OptiflowReadOnlyAdapter::observe_release(
        &selected_path(&request.provider_executable)?,
        token,
    )
    .map_err(native)?;
    scope.require_current().map_err(native)?;
    cancelled(token)?;
    let mut value = Plan {
        schema_version: "flow.command-plan/v1".into(),
        plan_sha256: String::new(),
        request: request.clone(),
        source_scope: scope,
        provider,
        effects: effects(),
        content_binding: "source_scope_only; native scan binds actual members at execution".into(),
    };
    value.plan_sha256 = digest(&value)?;
    value.validate()?;
    Ok(value)
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Approval {
    pub schema_version: String,
    pub plan_sha256: String,
    pub approved_by: String,
    pub acknowledge_trusted_unconfined: bool,
    pub scope: String,
}
impl Approval {
    pub fn validate(&self, plan: &Plan) -> Result<(), Error> {
        plan.validate()?;
        if self.schema_version != "flow.command-approval/v1"
            || self.plan_sha256 != plan.plan_sha256
            || self.approved_by.trim().is_empty()
            || self.approved_by.len() > 256
            || self.approved_by.chars().any(char::is_control)
            || !self.acknowledge_trusted_unconfined
            || self.scope != "selected_read_only_scan_and_local_evidence_only"
        {
            return Err(error(
                Code::ApprovalRequired,
                "exact supplied operation approval and unconfined-process acknowledgement required",
            ));
        }
        Ok(())
    }
}
pub fn approve(
    plan: &Plan,
    approved_by: String,
    acknowledge_trusted_unconfined: bool,
) -> Result<Approval, Error> {
    let value = Approval {
        schema_version: "flow.command-approval/v1".into(),
        plan_sha256: plan.plan_sha256.clone(),
        approved_by,
        acknowledge_trusted_unconfined,
        scope: "selected_read_only_scan_and_local_evidence_only".into(),
    };
    value.validate(plan)?;
    Ok(value)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Complete,
    Partial,
    RecoveryRequired,
    Refused,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Marker {
    schema_version: String,
    plan_sha256: String,
    approval_sha256: String,
    receipt_sha256: String,
    receipt: NativePath,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub schema_version: String,
    pub plan_sha256: String,
    pub state: State,
    pub receipt: Option<ReadOnlyReceipt>,
    pub receipt_path: Option<NativePath>,
    pub operation_record_complete: bool,
    pub source_content_reobserved: bool,
    pub next_action: String,
    pub source_mutation_authority: bool,
}
fn result(
    plan: &Plan,
    state: State,
    receipt: Option<ReadOnlyReceipt>,
    path: Option<NativePath>,
    complete: bool,
) -> Status {
    Status {
        schema_version: "flow.command-status/v1".into(),
        plan_sha256: plan.plan_sha256.clone(),
        state,
        receipt,
        receipt_path: path,
        operation_record_complete: complete,
        source_content_reobserved: false,
        next_action: if complete {
            "stored native attempt; use a new plan for a fresh scan"
        } else {
            "inspect retained native state; no automatic replay"
        }
        .into(),
        source_mutation_authority: false,
    }
}
fn basis(root: &Path) -> Result<(Plan, Approval), Error> {
    let plan = Plan::from_json_slice(&read_document(&root.join("operation-plan.json"), 32_768)?)?;
    let approval: Approval = serde_json::from_slice(&read_document(
        &root.join("operation-approval.json"),
        16_384,
    )?)
    .map_err(|_| error(Code::InvalidInput, "closed saved approval required"))?;
    approval.validate(&plan)?;
    if selected_path(&plan.request.run_directory)? != root {
        return Err(error(
            Code::InvalidInput,
            "saved operation points to a different run root",
        ));
    }
    Ok((plan, approval))
}
fn receipt(root: &Path, plan: &Plan) -> Result<Option<(ReadOnlyReceipt, PathBuf, String)>, Error> {
    let directory = root.join("flow-optiflow-v0.1.1");
    if !directory
        .try_exists()
        .map_err(|_| error(Code::Io, "cannot inspect native receipt namespace"))?
    {
        return Ok(None);
    }
    if directory.canonicalize().ok().as_deref() != Some(directory.as_path()) {
        return Err(error(
            Code::InvalidInput,
            "native receipt namespace is redirected",
        ));
    }
    let mut entries = fs::read_dir(&directory)
        .map_err(|_| error(Code::Io, "native receipt namespace unavailable"))?;
    let Some(entry) = entries.next() else {
        return Ok(None);
    };
    let path = entry
        .map_err(|_| error(Code::Io, "native receipt member unavailable"))?
        .path();
    if entries.next().is_some() || path.extension().is_none_or(|e| e != "json") {
        return Err(error(
            Code::RecoveryRequired,
            "ambiguous native attempts; no latest-attempt adoption",
        ));
    }
    let native_receipt = ReadOnlyReceipt::load(&path).map_err(native)?;
    if native_receipt.provider != plan.provider
        || native_receipt.source != plan.source_scope
        || native_receipt.state_directory.path().map_err(native)? != root
    {
        return Err(error(
            Code::StalePlan,
            "native receipt does not match this exact selected operation",
        ));
    }
    let bytes = read_document(&path, 16_777_216)?;
    let rebound: ReadOnlyReceipt = serde_json::from_slice(&bytes)
        .map_err(|_| error(Code::InvalidInput, "closed native receipt required"))?;
    if canonical(&rebound)? != canonical(&native_receipt)? {
        return Err(error(
            Code::StalePlan,
            "native receipt changed between acceptance and completion-byte observation",
        ));
    }
    Ok(Some((native_receipt, path, hash(&bytes))))
}
fn marker(plan: &Plan, approval: &Approval, path: &Path, sha: String) -> Result<Marker, Error> {
    Ok(Marker {
        schema_version: "flow.command-completion/v1".into(),
        plan_sha256: plan.plan_sha256.clone(),
        approval_sha256: digest(approval)?,
        receipt_sha256: sha,
        receipt: NativePath::from_path(path),
    })
}
/// Reopen exact native attempt evidence; no provider launch or automatic replay.
pub fn status(root: &Path, token: &dyn CancellationSignal) -> Result<Status, Error> {
    cancelled(token)?;
    let root = absolute(root)?;
    documents::directory(&root)?;
    let (plan, approval) = basis(&root)?;
    let observed = receipt(&root, &plan)?;
    let marker_path = root.join("operation-result.json");
    let Some((native_receipt, path, sha)) = observed else {
        if marker_path
            .try_exists()
            .map_err(|_| error(Code::Io, "cannot inspect completion marker"))?
        {
            return Err(error(
                Code::RecoveryRequired,
                "completion marker lacks its native receipt",
            ));
        }
        return Ok(result(&plan, State::RecoveryRequired, None, None, false));
    };
    let complete = marker_path
        .try_exists()
        .map_err(|_| error(Code::Io, "cannot inspect completion marker"))?;
    if complete {
        let saved: Marker = serde_json::from_slice(&read_document(&marker_path, 16_384)?)
            .map_err(|_| error(Code::InvalidInput, "closed operation completion required"))?;
        if saved != marker(&plan, &approval, &path, sha)?
            || native_receipt.needs_recovery()
            || !matches!(
                native_receipt.status,
                ReceiptStatus::Complete | ReceiptStatus::Partial
            )
        {
            return Err(error(
                Code::RecoveryRequired,
                "operation completion differs from retained native acceptance",
            ));
        }
    }
    let state = if complete {
        if native_receipt.status == ReceiptStatus::Complete {
            State::Complete
        } else {
            State::Partial
        }
    } else if native_receipt.status == ReceiptStatus::Refused && !native_receipt.needs_recovery() {
        State::Refused
    } else {
        State::RecoveryRequired
    };
    cancelled(token)?;
    Ok(result(
        &plan,
        state,
        Some(native_receipt),
        Some(NativePath::from_path(&path)),
        complete,
    ))
}
/// Execute only this approved released route; all provider work stays in the
/// existing native adapter. Persist explicit scope before its first probe.
pub fn run(
    plan: &Plan,
    approval: &Approval,
    token: &dyn CancellationSignal,
) -> Result<Status, Error> {
    approval.validate(plan)?;
    cancelled(token)?;
    let fresh = self::plan(&plan.request, token)?;
    if fresh != *plan {
        return Err(error(
            Code::StalePlan,
            "source scope or selected provider changed since planning",
        ));
    }
    let root = run_destination(&plan.request)?;
    documents::create_directory(&root)?;
    let execute = (|| -> Result<Status, Error> {
        write_document(&root.join("operation-plan.json"), plan)?;
        write_document(&root.join("operation-approval.json"), approval)?;
        cancelled(token)?;
        let adapter = OptiflowReadOnlyAdapter::probe_with_cancellation(
            &selected_path(&plan.request.provider_executable)?,
            token,
        )
        .map_err(native)?;
        if adapter.identity() != &plan.provider {
            return Err(error(Code::StalePlan, "native provider identity changed"));
        }
        adapter
            .inspect_with_cancellation(&selected_path(&plan.request.source_root)?, &root, token)
            .map_err(native)?;
        resume(&root, token)
    })();
    execute.map_err(|mut e| {
        e.recovery_directory = Some(NativePath::from_path(&root));
        e
    })
}
/// Explicitly finish an interrupted outer marker only when the existing native
/// receipt already completed. Never rerun scan/report/plan or repair partial work.
pub fn resume(root: &Path, token: &dyn CancellationSignal) -> Result<Status, Error> {
    let current = status(root, token)?;
    if current.operation_record_complete {
        return Ok(current);
    }
    let root = absolute(root)?;
    let (plan, approval) = basis(&root)?;
    cancelled(token)?;
    plan.source_scope.require_current().map_err(native)?;
    let provider = OptiflowReadOnlyAdapter::observe_release(
        &selected_path(&plan.request.provider_executable)?,
        token,
    )
    .map_err(native)?;
    if provider != plan.provider {
        return Err(error(
            Code::StalePlan,
            "selected provider changed before metadata reconciliation",
        ));
    }
    let Some((native_receipt, path, sha)) = receipt(&root, &plan)? else {
        return Err(error(
            Code::RecoveryRequired,
            "native receipt is absent; preserve the attempt, no replay",
        ));
    };
    if native_receipt.needs_recovery()
        || !matches!(
            native_receipt.status,
            ReceiptStatus::Complete | ReceiptStatus::Partial
        )
    {
        return Err(error(
            Code::RecoveryRequired,
            "native attempt is unfinished/refused; inspect it before a separately approved fresh run",
        ));
    }
    if basis(&root)? != (plan.clone(), approval.clone()) {
        return Err(error(
            Code::StalePlan,
            "operation declaration changed during reconciliation",
        ));
    }
    cancelled(token)?;
    write_document(
        &root.join("operation-result.json"),
        &marker(&plan, &approval, &path, sha)?,
    )?;
    status(&root, token)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Doctor {
    pub schema_version: String,
    pub selected_provider: Option<ProviderIdentity>,
    pub selected_state: String,
    pub unavailable_routes: Vec<Route>,
    pub provider_invoked: bool,
    pub installation_performed: bool,
    pub release_qualified: bool,
}
pub fn doctor(executable: Option<&Path>, token: &dyn CancellationSignal) -> Result<Doctor, Error> {
    cancelled(token)?;
    let (selected_provider, selected_state) = match executable {
        None => (None, "not_configured"),
        Some(p) => match OptiflowReadOnlyAdapter::observe_release(&absolute(p)?, token) {
            Ok(identity) => (
                Some(identity),
                "matched_release_bytes; native_version_unobserved",
            ),
            Err(optiflow::AdapterError::Cancelled) => {
                return Err(error(Code::Cancelled, "doctor cancelled"));
            }
            Err(_) => (None, "missing_or_incompatible_selected_provider"),
        },
    };
    Ok(Doctor {
        schema_version: "flow.command-doctor/v1".into(),
        selected_provider,
        selected_state: selected_state.into(),
        unavailable_routes: vec![Route::RenderflowPublication, Route::AniflowTemporal],
        provider_invoked: false,
        installation_performed: false,
        release_qualified: false,
    })
}

#[cfg(test)]
mod tests;
