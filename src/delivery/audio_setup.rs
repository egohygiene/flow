//! Installed, read-only preparation of exact delivery audio policy checks.
//! Aniflow retains current QC and audio math; approvals are supplied separately.
use super::*;
use crate::release_cohort::{Cohort, Tool};
use std::path::PathBuf;

pub const REQUEST_SCHEMA: &str = "flow.audio-check-setup/v1";
pub const PROFILE: &str = "audio-policy-check-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Choice {
    /// Existing Aniflow current-final-QC request, including the selected policy.
    Check {
        owner_request: FilePin,
        #[serde(default)]
        row_id: Option<String>,
    },
    KeepArchiveMaster {
        reason: String,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub payload_id: String,
    pub destination: Destination,
    pub choice: Choice,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: String,
    pub package_manifest: FilePin,
    pub installation: Option<project::Installation>,
    pub workspace: NativePath,
    pub selections: Vec<Selection>,
    pub tool_directories: Vec<NativePath>,
    pub limits: native::Limits,
    pub timeout_ms: u64,
}
impl Request {
    pub fn from_json_slice(raw: &[u8]) -> Result<Self, Error> {
        if raw.len() > 1_048_576 {
            return Err(error(
                Code::InvalidInput,
                "audio setup request exceeds 1 MiB",
            ));
        }
        let value: Self = serde_json::from_slice(raw)
            .map_err(|_| error(Code::InvalidInput, "closed audio setup request required"))?;
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<(), Error> {
        self.limits.validate()?;
        native::path(&self.workspace)?;
        if self.schema_version != REQUEST_SCHEMA
            || self.selections.len() > 128
            || self.tool_directories.len() > 32
            || self.timeout_ms == 0
            || self.timeout_ms > 86_400_000
        {
            return Err(error(
                Code::InvalidInput,
                "bounded audio setup selection required",
            ));
        }
        if self
            .selections
            .iter()
            .any(|s| matches!(&s.choice, Choice::Check { .. }))
            && self.installation.is_none()
        {
            return Err(error(
                Code::Unavailable,
                "an explicit installed Aniflow selection is required for checks",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResolvedChoice {
    Check {
        source: FilePin,
        owner_request: FilePin,
        row: project::SelectedRow,
        policy: Value,
        request: native::Request,
        plan_file: NativePath,
        approval_file: NativePath,
    },
    KeepArchiveMaster {
        reason: String,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedSelection {
    pub payload_id: String,
    pub destination: Destination,
    pub choice: ResolvedChoice,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preview {
    pub schema_version: String,
    pub setup_sha256: String,
    pub package_subject_sha256: String,
    pub package_manifest: FilePin,
    pub workspace: NativePath,
    pub selections: Vec<ResolvedSelection>,
    pub publisher_authenticated_at_install: bool,
    pub provider_invoked: bool,
    pub approvals_generated: bool,
    pub native_qualified: bool,
    pub next_step: String,
}
struct Draft {
    preview: Preview,
    package: Status,
    inputs: Vec<FilePin>,
    configurations: Vec<(PathBuf, Vec<u8>)>,
    protected: Vec<PathBuf>,
}
fn overlaps(a: &Path, b: &Path) -> bool {
    a.starts_with(b) || b.starts_with(a)
}
fn ensure_separate(workspace: &Path, selected: &Path) -> Result<(), Error> {
    if overlaps(workspace, selected) {
        Err(error(
            Code::InvalidInput,
            "audio setup workspace overlaps package, source, installation, tools or QC evidence",
        ))
    } else {
        Ok(())
    }
}
fn validate_coverage(payloads: &[Payload], selections: &[Selection]) -> Result<(), Error> {
    let expected = payloads
        .iter()
        .filter(|p| audio_readiness::audio_role(p.role))
        .flat_map(|p| {
            p.destinations
                .iter()
                .map(move |d| audio_readiness::key(&p.id, *d))
        })
        .collect::<BTreeSet<_>>();
    let mut selected = BTreeSet::new();
    for selection in selections {
        let id = audio_readiness::key(&selection.payload_id, selection.destination);
        if !expected.contains(&id) || !selected.insert(id) {
            return Err(error(
                Code::InvalidInput,
                "unique exact assigned audio payload/destination required",
            ));
        }
        if let Choice::KeepArchiveMaster { reason } = &selection.choice {
            let payload = payloads
                .iter()
                .find(|p| p.id == selection.payload_id)
                .ok_or_else(|| error(Code::InvalidInput, "selected payload missing"))?;
            if !audio_readiness::archive_skip(payload.role, selection.destination, reason) {
                return Err(error(
                    Code::InvalidInput,
                    "only an explicitly unchanged archival master/original can skip a policy check",
                ));
            }
        }
    }
    if selected != expected {
        return Err(error(
            Code::InvalidInput,
            "audio setup must cover every assigned media payload and destination",
        ));
    }
    Ok(())
}
fn draft(request: &Request, token: &dyn CancellationSignal) -> Result<Draft, Error> {
    request.validate()?;
    let manifest = native::path(&request.package_manifest.path)?;
    if manifest.file_name().and_then(|n| n.to_str()) != Some("manifest.json") {
        return Err(error(
            Code::InvalidInput,
            "exact delivery manifest.json required",
        ));
    }
    request.package_manifest.current(MAX_DOC, token)?;
    let package = super::status(
        manifest
            .parent()
            .ok_or_else(|| error(Code::InvalidInput, "package parent required"))?,
        &request.package_manifest.sha256,
        token,
    )?;
    validate_coverage(&package.preview.payloads, &request.selections)?;
    let workspace = native::path(&request.workspace)?;
    let parent = workspace
        .parent()
        .ok_or_else(|| error(Code::InvalidInput, "audio setup parent required"))?;
    if parent.canonicalize().ok().as_deref() != Some(parent)
        || !fs::metadata(parent).is_ok_and(|m| m.is_dir())
    {
        return Err(error(
            Code::InvalidInput,
            "existing canonical audio setup parent required",
        ));
    }
    let mut protected = vec![native::path(&package.current_root)?];
    let installed = request
        .installation
        .as_ref()
        .map(|s| {
            distribution::selected_installation(&native::path(&s.root)?, &s.receipt_sha256, token)
        })
        .transpose()?;
    let cohort = installed
        .as_ref()
        .map(|i| {
            Cohort::from_json_slice(&super::read(&i.cohort, 1_048_576, token)?)
                .map_err(|_| error(Code::InvalidInput, "installed audio cohort invalid"))
        })
        .transpose()?;
    if installed
        .as_ref()
        .is_some_and(|i| Some(i.target.as_str()) != native::host_target())
        || cohort
            .as_ref()
            .is_some_and(|c| c.flow_version != env!("CARGO_PKG_VERSION"))
    {
        return Err(error(
            Code::Unavailable,
            "installed audio cohort targets another host or Flow version",
        ));
    }
    if let Some(i) = &installed {
        protected.push(native::path(&i.root)?);
    }
    for p in &request.tool_directories {
        let p = native::path(p)?;
        if p.canonicalize().ok().as_deref() != Some(p.as_path())
            || !fs::metadata(&p).is_ok_and(|m| m.is_dir())
        {
            return Err(error(
                Code::InvalidInput,
                "canonical selected tool directory required",
            ));
        }
        protected.push(p);
    }
    let mut inputs = vec![request.package_manifest.clone()];
    let mut configurations = vec![];
    let mut selections = vec![];
    for (index, s) in request.selections.iter().enumerate() {
        if token.is_cancelled() {
            return Err(error(Code::Cancelled, "audio setup observation cancelled"));
        }
        let choice = match &s.choice {
            Choice::KeepArchiveMaster { reason } => ResolvedChoice::KeepArchiveMaster {
                reason: reason.clone(),
            },
            Choice::Check {
                owner_request,
                row_id,
            } => {
                let payload = package
                    .preview
                    .payloads
                    .iter()
                    .find(|p| p.id == s.payload_id)
                    .ok_or_else(|| error(Code::InvalidInput, "audio payload missing"))?;
                let source = package
                    .preview
                    .selected_sources
                    .iter()
                    .find(|p| p.payload_id == s.payload_id)
                    .ok_or_else(|| {
                        error(
                            Code::InvalidInput,
                            "original final-file locator absent from delivery package",
                        )
                    })?
                    .file
                    .clone();
                if source.sha256 != payload.sha256 || source.byte_count != payload.byte_count {
                    return Err(error(
                        Code::StalePlan,
                        "original final bytes differ from copied delivery bytes",
                    ));
                }
                source.current(request.limits.max_input_bytes, token)?;
                let config = super::read(owner_request, 131_072, token)?;
                let value: Value = serde_json::from_slice(&config).map_err(|_| {
                    error(
                        Code::InvalidInput,
                        "explicit Aniflow policy-check JSON required",
                    )
                })?;
                let i = installed
                    .as_ref()
                    .ok_or_else(|| error(Code::Unavailable, "installed audio provider required"))?;
                let row = project::select_row(
                    i,
                    cohort
                        .as_ref()
                        .ok_or_else(|| error(Code::Unavailable, "audio cohort required"))?,
                    Tool::Aniflow,
                    PROFILE,
                    row_id.as_deref(),
                    "audio_policy_check",
                )?;
                let descriptor: Value =
                    serde_json::from_slice(&super::read(&row.descriptor, MAX_DOC, token)?)
                        .map_err(|_| {
                            error(
                                Code::InvalidInput,
                                "installed Aniflow descriptor JSON required",
                            )
                        })?;
                if descriptor["schema_version"] != "aniflow.release-contract/v1"
                    || descriptor["tool"] != "aniflow"
                    || !descriptor["profiles"].as_array().is_some_and(|profiles| {
                        profiles
                            .iter()
                            .any(|p| p["id"] == PROFILE && p["source_mutation"] == false)
                    })
                {
                    return Err(error(
                        Code::Unavailable,
                        "installed Aniflow descriptor does not advertise the selected read-only policy profile",
                    ));
                }
                let name = format!("check-{index:03}");
                let configuration = workspace.join("configuration").join(format!("{name}.json"));
                let mut planned = native::Request {
                    schema_version: native::REQUEST_SCHEMA.into(),
                    cohort: i.cohort.path.clone(),
                    cohort_sha256: i.cohort.sha256.clone(),
                    row_id: row.row_id.clone(),
                    provider_executable: row.provider.path.clone(),
                    release_descriptor: row.descriptor.path.clone(),
                    inputs: vec![source.path.clone()],
                    configuration: owner_request.path.clone(),
                    run_directory: NativePath::from_path(&workspace.join("native").join(&name)),
                    tool_directories: request.tool_directories.clone(),
                    limits: request.limits.clone(),
                    preparation: None,
                };
                protected.extend(native::audio_policy_protected_paths(&planned)?);
                protected.extend([
                    native::path(&source.path)?,
                    native::path(&owner_request.path)?,
                ]);
                planned.configuration = NativePath::from_path(&configuration);
                configurations.push((configuration, config));
                inputs.extend([source.clone(), owner_request.clone()]);
                ResolvedChoice::Check {
                    source,
                    owner_request: owner_request.clone(),
                    row,
                    policy: value["policy"].clone(),
                    request: planned,
                    plan_file: NativePath::from_path(
                        &workspace.join("plans").join(format!("{name}.json")),
                    ),
                    approval_file: NativePath::from_path(
                        &workspace.join("approvals").join(format!("{name}.json")),
                    ),
                }
            }
        };
        selections.push(ResolvedSelection {
            payload_id: s.payload_id.clone(),
            destination: s.destination,
            choice,
        });
    }
    for p in &protected {
        ensure_separate(&workspace, p)?;
    }
    let preview=Preview {schema_version:"flow.audio-check-setup-preview/v1".into(),setup_sha256:hash(request)?,package_subject_sha256:package.preview.package_subject_sha256.clone(),package_manifest:request.package_manifest.clone(),workspace:request.workspace.clone(),selections,publisher_authenticated_at_install:installed.as_ref().is_some_and(|i|i.publisher_authenticated_at_install),provider_invoked:false,approvals_generated:false,native_qualified:false,next_step:"Explicit preparation creates native plans only; supply exact approvals separately before a fresh owner check. Preview does not recheck Aniflow QC.".into()};
    Ok(Draft {
        preview,
        package,
        inputs,
        configurations,
        protected,
    })
}
/// Reads exact local selections only. No provider process, directory or approval.
pub fn preview(request: &Request, token: &dyn CancellationSignal) -> Result<Preview, Error> {
    let start = Instant::now();
    let guard =
        || token.is_cancelled() || start.elapsed() >= Duration::from_millis(request.timeout_ms);
    Ok(draft(request, &guard)?.preview)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedPlan {
    pub payload_id: String,
    pub destination: Destination,
    pub plan_file: FilePin,
    pub plan: native::Plan,
    pub approval_file: NativePath,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema_version: String,
    request: Request,
    preview: Preview,
    inputs: Vec<FilePin>,
    intent_file: FilePin,
    configuration_files: Vec<FilePin>,
    plans: Vec<PreparedPlan>,
}
#[derive(Debug, Serialize)]
pub struct Prepared {
    pub schema_version: String,
    pub root: NativePath,
    pub setup_sha256: String,
    pub package_subject_sha256: String,
    pub record: FilePin,
    pub plans: Vec<PreparedPlan>,
    pub provider_invoked: bool,
    pub approvals_generated: bool,
    pub native_qualified: bool,
    pub evidence_scope: String,
}
fn prepared(record: &Record, file: FilePin) -> Prepared {
    Prepared {schema_version:"flow.audio-check-preparation/v1".into(),root:record.request.workspace.clone(),setup_sha256:record.preview.setup_sha256.clone(),package_subject_sha256:record.preview.package_subject_sha256.clone(),record:file,plans:record.plans.clone(),provider_invoked:false,approvals_generated:false,native_qualified:false,evidence_scope:"local configuration/plan identity only; no provider process, current owner QC, release qualification or approval".into()}
}
/// Explicit local configuration writing; does not execute Aniflow or approve it.
/// The last record is a completion marker. Errors retain the partial root.
pub fn prepare(request: &Request, token: &dyn CancellationSignal) -> Result<Prepared, Error> {
    let started = Instant::now();
    let guard =
        || token.is_cancelled() || started.elapsed() >= Duration::from_millis(request.timeout_ms);
    let selected = draft(request, &guard)?;
    let root = native::path(&request.workspace)?;
    create(&root).map_err(|mut e: Error| {
        e.recovery_directory = Some(NativePath::from_path(&root));
        e
    })?;
    let action = (|| {
        for name in ["configuration", "plans", "approvals", "native"] {
            create(&root.join(name))?;
        }
        let intent = root.join("intent.json");
        write(&intent, &bytes(request)?)?;
        let intent_file = FilePin::observe(&NativePath::from_path(&intent), MAX_DOC, &guard)?;
        let mut configuration_files = vec![];
        for (path, data) in &selected.configurations {
            if guard() {
                return Err(error(
                    Code::Cancelled,
                    "audio setup preparation cancelled; preserve partial root",
                ));
            }
            write(path, data)?;
            configuration_files.push(FilePin::observe(
                &NativePath::from_path(path),
                131_072,
                &guard,
            )?);
        }
        let mut plans = vec![];
        for s in &selected.preview.selections {
            if let ResolvedChoice::Check {
                request,
                plan_file,
                approval_file,
                ..
            } = &s.choice
            {
                let plan = native::prepare(request, &guard)?;
                write(&native::path(plan_file)?, &bytes(&plan)?)?;
                plans.push(PreparedPlan {
                    payload_id: s.payload_id.clone(),
                    destination: s.destination,
                    plan_file: FilePin::observe(plan_file, MAX_DOC, &guard)?,
                    plan,
                    approval_file: approval_file.clone(),
                });
            }
        }
        for pin in selected
            .inputs
            .iter()
            .chain(configuration_files.iter())
            .chain(std::iter::once(&intent_file))
        {
            pin.current(request.limits.max_input_bytes.max(MAX_DOC), &guard)?;
        }
        for p in &plans {
            p.plan.require_current(&guard)?;
            p.plan_file.current(MAX_DOC, &guard)?;
        }
        if let Some(i) = &request.installation {
            distribution::selected_installation(
                &native::path(&i.root)?,
                &i.receipt_sha256,
                &guard,
            )?;
        }
        let current = super::status(
            &native::path(&selected.package.current_root)?,
            &request.package_manifest.sha256,
            &guard,
        )?;
        if current.preview != selected.package.preview {
            return Err(error(
                Code::StalePlan,
                "delivery package changed during check preparation",
            ));
        }
        let record = Record {
            schema_version: "flow.prepared-audio-check-setup/v1".into(),
            request: request.clone(),
            preview: selected.preview,
            inputs: selected.inputs,
            intent_file,
            configuration_files,
            plans,
        };
        let path = root.join("prepared-audio-checks.json");
        write(&path, &bytes(&record)?)?;
        fs::File::open(&root)
            .and_then(|f| f.sync_all())
            .map_err(|_| error(Code::Io, "audio preparation directory synchronization"))?;
        let file = FilePin::observe(&NativePath::from_path(&path), MAX_DOC, &guard)?;
        Ok(prepared(&record, file))
    })();
    action.map_err(|mut e: Error| {
        e.recovery_directory = Some(NativePath::from_path(&root));
        e
    })
}

fn exact_members(
    root: &Path,
    expected: &BTreeSet<PathBuf>,
    directories: &BTreeSet<PathBuf>,
) -> Result<(), Error> {
    if root.canonicalize().ok().as_deref() != Some(root) {
        return Err(error(
            Code::RecoveryRequired,
            "canonical audio setup directory required",
        ));
    }
    let mut found = BTreeSet::new();
    for e in fs::read_dir(root).map_err(|_| error(Code::Io, "audio setup directory unavailable"))? {
        let path = e
            .map_err(|_| error(Code::Io, "audio setup member unavailable"))?
            .path();
        let m = fs::symlink_metadata(&path)
            .map_err(|_| error(Code::Io, "audio setup metadata unavailable"))?;
        let wrong_kind = if directories.contains(&path) {
            !m.is_dir() || path.canonicalize().ok().as_deref() != Some(path.as_path())
        } else {
            !m.is_file()
        };
        if found.len() >= 512
            || !expected.contains(&path)
            || m.file_type().is_symlink()
            || wrong_kind
        {
            return Err(error(
                Code::RecoveryRequired,
                "unknown, aliased or changed audio setup core member; preserve it",
            ));
        }
        found.insert(path);
    }
    if &found != expected {
        return Err(error(
            Code::RecoveryRequired,
            "audio setup core member missing",
        ));
    }
    Ok(())
}
fn load_current(
    root: &Path,
    expected_record_sha256: &str,
    token: &dyn CancellationSignal,
) -> Result<(Record, FilePin, Draft), Error> {
    if !release_cohort::is_digest(expected_record_sha256) {
        return Err(error(
            Code::InvalidInput,
            "exact audio preparation record digest required",
        ));
    }
    let file = FilePin::observe(
        &NativePath::from_path(&root.join("prepared-audio-checks.json")),
        MAX_DOC,
        token,
    )?;
    if file.sha256 != expected_record_sha256 {
        return Err(error(
            Code::StalePlan,
            "audio preparation record pin changed",
        ));
    }
    let record: Record = object(&file, token)?;
    if record.schema_version != "flow.prepared-audio-check-setup/v1"
        || native::path(&record.request.workspace)? != root
        || native::path(&record.intent_file.path)? != root.join("intent.json")
    {
        return Err(error(
            Code::RecoveryRequired,
            "audio preparation root/record identity changed",
        ));
    }
    record.request.validate()?;
    let started = Instant::now();
    let guard = || {
        token.is_cancelled()
            || started.elapsed() >= Duration::from_millis(record.request.timeout_ms)
    };
    let token: &dyn CancellationSignal = &guard;
    let request: Request = object(&record.intent_file, token)?;
    if hash(&request)? != hash(&record.request)? {
        return Err(error(
            Code::RecoveryRequired,
            "audio setup intent differs from final record",
        ));
    }
    let fresh = draft(&record.request, token)?;
    if hash(&fresh.preview)? != hash(&record.preview)?
        || hash(&fresh.inputs)? != hash(&record.inputs)?
    {
        return Err(error(
            Code::StalePlan,
            "audio setup package, source, configuration or installed row changed",
        ));
    }
    let configurations = fresh
        .configurations
        .iter()
        .map(|(p, b)| FilePin {
            path: NativePath::from_path(p),
            sha256: release_cohort::sha256(b),
            byte_count: b.len() as u64,
        })
        .collect::<Vec<_>>();
    if configurations != record.configuration_files {
        return Err(error(
            Code::RecoveryRequired,
            "prepared configuration membership or source bytes differ",
        ));
    }
    let checks = fresh
        .preview
        .selections
        .iter()
        .filter_map(|s| match &s.choice {
            ResolvedChoice::Check {
                request,
                source,
                plan_file,
                approval_file,
                ..
            } => Some((s, request, source, plan_file, approval_file)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if checks.len() != record.plans.len() {
        return Err(error(
            Code::RecoveryRequired,
            "prepared native plan membership differs",
        ));
    }
    for (p, (s, request, source, plan_file, approval_file)) in record.plans.iter().zip(checks) {
        if p.payload_id != s.payload_id
            || p.destination != s.destination
            || p.plan.request != *request
            || p.plan.sources != vec![source.clone()]
            || p.plan.row.profile != PROFILE
            || p.plan.row.tool != Tool::Aniflow
            || p.plan_file.path != *plan_file
            || p.approval_file != *approval_file
        {
            return Err(error(
                Code::RecoveryRequired,
                "native plan/source/reserved-path binding differs",
            ));
        }
        p.plan.require_current(token)?;
        let current = native::Plan::from_json_slice(&super::read(&p.plan_file, MAX_DOC, token)?)?;
        if current != p.plan {
            return Err(error(
                Code::StalePlan,
                "native plan file differs from prepared record",
            ));
        }
    }
    let dirs = ["configuration", "plans", "approvals", "native"]
        .into_iter()
        .map(|n| root.join(n))
        .collect::<BTreeSet<_>>();
    let mut core = dirs.clone();
    core.extend([
        root.join("intent.json"),
        root.join("prepared-audio-checks.json"),
    ]);
    exact_members(root, &core, &dirs)?;
    let configs = record
        .configuration_files
        .iter()
        .map(|f| native::path(&f.path))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let plans = record
        .plans
        .iter()
        .map(|p| native::path(&p.plan_file.path))
        .collect::<Result<BTreeSet<_>, _>>()?;
    exact_members(&root.join("configuration"), &configs, &BTreeSet::new())?;
    exact_members(&root.join("plans"), &plans, &BTreeSet::new())?;
    for p in record
        .inputs
        .iter()
        .chain(record.configuration_files.iter())
    {
        p.current(record.request.limits.max_input_bytes.max(MAX_DOC), token)?;
    }
    file.current(MAX_DOC, token)?;
    Ok((record, file, fresh))
}
/// Read-only local scope/integrity observation, without rechecking domain QC.
pub fn status(
    root: &Path,
    expected_record_sha256: &str,
    token: &dyn CancellationSignal,
) -> Result<Prepared, Error> {
    let (record, file, _) = load_current(root, expected_record_sha256, token)?;
    Ok(prepared(&record, file))
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuppliedApproval {
    pub payload_id: String,
    pub destination: Destination,
    pub authority: audio_readiness::Authority,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalSet {
    pub schema_version: String,
    pub approvals: Vec<SuppliedApproval>,
}
impl ApprovalSet {
    pub fn from_json_slice(raw: &[u8]) -> Result<Self, Error> {
        if raw.len() > 1_048_576 {
            return Err(error(
                Code::InvalidInput,
                "audio approval bindings exceed 1 MiB",
            ));
        }
        let value: Self = serde_json::from_slice(raw).map_err(|_| {
            error(
                Code::InvalidInput,
                "closed supplied audio approval set required",
            )
        })?;
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<(), Error> {
        if self.schema_version != "flow.audio-check-approvals/v1" || self.approvals.len() > 128 {
            return Err(error(
                Code::InvalidInput,
                "bounded explicit audio approval bindings required",
            ));
        }
        Ok(())
    }
}
fn bind_record(
    record: &Record,
    approvals: &ApprovalSet,
    token: &dyn CancellationSignal,
) -> Result<audio_readiness::Request, Error> {
    approvals.validate()?;
    let mut supplied = BTreeMap::new();
    for a in &approvals.approvals {
        if supplied
            .insert(
                audio_readiness::key(&a.payload_id, a.destination),
                &a.authority,
            )
            .is_some()
        {
            return Err(error(
                Code::InvalidInput,
                "duplicate supplied audio approval selection",
            ));
        }
    }
    let mut selections = vec![];
    for s in &record.preview.selections {
        let choice = match &s.choice {
            ResolvedChoice::KeepArchiveMaster { reason } => {
                audio_readiness::Choice::KeepArchiveMaster {
                    reason: reason.clone(),
                }
            }
            ResolvedChoice::Check { .. } => {
                let p = record
                    .plans
                    .iter()
                    .find(|p| p.payload_id == s.payload_id && p.destination == s.destination)
                    .ok_or_else(|| error(Code::RecoveryRequired, "prepared check plan missing"))?;
                let authority = supplied
                    .remove(&audio_readiness::key(&s.payload_id, s.destination))
                    .ok_or_else(|| {
                        error(
                            Code::ApprovalRequired,
                            "every prepared check requires a separately supplied exact approval",
                        )
                    })?;
                let pin = match authority {
                    audio_readiness::Authority::SourceCandidate { approval }
                    | audio_readiness::Authority::Released { approval } => approval,
                };
                if pin.path != p.approval_file {
                    return Err(error(
                        Code::ApprovalRequired,
                        "supplied approval must use the exact reserved path for this prepared check",
                    ));
                }
                let raw = super::read(pin, 1_048_576, token)?;
                match authority {
                    audio_readiness::Authority::SourceCandidate { .. } => {
                        let a: native::Approval = serde_json::from_slice(&raw).map_err(|_| {
                            error(Code::InvalidInput, "closed candidate approval required")
                        })?;
                        if native::approve(
                            &p.plan,
                            a.approved_by.clone(),
                            a.trusted_unconfined,
                            a.unqualified_candidate,
                        )? != a
                        {
                            return Err(error(
                                Code::ApprovalRequired,
                                "supplied candidate approval belongs to another exact plan",
                            ));
                        }
                    }
                    audio_readiness::Authority::Released { .. } => {
                        let a: native::ReleasedApproval =
                            serde_json::from_slice(&raw).map_err(|_| {
                                error(Code::InvalidInput, "closed released approval required")
                            })?;
                        if a.schema_version != "flow.released-native-approval/v1"
                            || a.plan_sha256 != p.plan.plan_sha256
                            || !a.trusted_unconfined
                            || a.approved_by.trim().is_empty()
                            || a.approved_by.len() > 256
                            || a.approved_by.chars().any(char::is_control)
                        {
                            return Err(error(
                                Code::ApprovalRequired,
                                "supplied released approval belongs to another exact plan",
                            ));
                        }
                        // Binding is not release admission. The existing run path must
                        // authenticate current opaque evidence before any invocation.
                    }
                }
                audio_readiness::Choice::Check {
                    plan: p.plan_file.clone(),
                    authority: authority.clone(),
                }
            }
        };
        selections.push(audio_readiness::Selection {
            payload_id: s.payload_id.clone(),
            destination: s.destination,
            choice,
        });
    }
    if !supplied.is_empty() {
        return Err(error(
            Code::InvalidInput,
            "supplied approval is not assigned to a prepared check",
        ));
    }
    Ok(audio_readiness::Request {
        schema_version: "flow.audio-readiness-request/v1".into(),
        package_manifest: record.request.package_manifest.clone(),
        selections,
        timeout_ms: record.request.timeout_ms,
    })
}
/// Produces an existing readiness request using supplied approvals only.
pub fn bind(
    root: &Path,
    expected_record_sha256: &str,
    approvals: &ApprovalSet,
    token: &dyn CancellationSignal,
) -> Result<audio_readiness::Request, Error> {
    let started = Instant::now();
    let (record, _, _) = load_current(root, expected_record_sha256, token)?;
    let guard = || {
        token.is_cancelled()
            || started.elapsed() >= Duration::from_millis(record.request.timeout_ms)
    };
    bind_record(&record, approvals, &guard)
}
/// Saves a binding outside the immutable bundle and all selected evidence.
pub fn save_binding(
    root: &Path,
    expected_record_sha256: &str,
    approvals: &ApprovalSet,
    output: &Path,
    token: &dyn CancellationSignal,
) -> Result<audio_readiness::Request, Error> {
    let started = Instant::now();
    let (record, file, selected) = load_current(root, expected_record_sha256, token)?;
    let guard = || {
        token.is_cancelled()
            || started.elapsed() >= Duration::from_millis(record.request.timeout_ms)
    };
    let output = command::absolute(output)?;
    ensure_separate(&output, root)?;
    for p in &selected.protected {
        ensure_separate(&output, p)?;
    }
    let value = bind_record(&record, approvals, &guard)?;
    file.current(MAX_DOC, &guard)?;
    command::write_document(&output, &value)?;
    Ok(value)
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    fn payload() -> Payload {
        Payload {
            id: "video".into(),
            role: Role::VideoDelivery,
            relative: "payload/video.mp4".into(),
            sha256: "a".repeat(64),
            byte_count: 100,
            destinations: vec![Destination::Tiktok],
            source_state: "accepted_native_artifact".into(),
        }
    }
    #[test]
    fn coverage_cannot_omit_duplicate_or_exempt_a_platform_copy() {
        let p = payload();
        assert!(validate_coverage(&[p.clone()], &[]).is_err());
        let s = Selection {
            payload_id: p.id.clone(),
            destination: Destination::Tiktok,
            choice: Choice::KeepArchiveMaster {
                reason: "do not process".into(),
            },
        };
        assert!(validate_coverage(&[p.clone()], &[s]).is_err());
        let mut master = p;
        master.role = Role::CleanMaster;
        master.destinations = vec![Destination::Archive];
        let s = Selection {
            payload_id: master.id.clone(),
            destination: Destination::Archive,
            choice: Choice::KeepArchiveMaster {
                reason: "Preserve selected approved master".into(),
            },
        };
        validate_coverage(&[master.clone()], &[s.clone()]).unwrap();
        assert!(validate_coverage(&[master], &[s.clone(), s]).is_err());
    }
    #[test]
    fn scope_refuses_both_ancestor_directions_without_creating_paths() {
        let root = Path::new("/synthetic/QC 音");
        assert!(ensure_separate(&root.join("setup"), root).is_err());
        assert!(ensure_separate(root, &root.join("checks")).is_err());
        assert!(ensure_separate(Path::new("/synthetic/setup"), root).is_ok());
    }

    pub(in crate::delivery) fn archive_fixture() -> (tempfile::TempDir, Request) {
        use crate::delivery as d;
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let original = root.join("synthetic 音.wav");
        fs::write(&original, b"synthetic bytes, not real audio").unwrap();
        let original =
            FilePin::observe(&NativePath::from_path(&original), MAX_DOC, &|| false).unwrap();
        let package = root.join("package");
        fs::create_dir(&package).unwrap();
        fs::create_dir(package.join("payload")).unwrap();
        let fake = FilePin {
            path: NativePath::from_path(&root.join("unused-fixture-reference")),
            sha256: "b".repeat(64),
            byte_count: 1,
        };
        let intent = d::Request {
            schema_version: "flow.delivery-request/v1".into(),
            project_record: fake.clone(),
            handoff: fake,
            payloads: vec![d::Selection {
                id: "original".into(),
                role: Role::OriginalAudio,
                source: d::Source::OriginalAudio,
                file_name: "original.wav".into(),
                destinations: vec![Destination::Archive],
            }],
            metadata: d::OperatorMetadata::default(),
            reviews: vec![],
            evidence_attachments: vec![],
            output_directory: NativePath::from_path(&package),
            limits: d::Limits {
                timeout_ms: 10_000,
                maximum_payload_bytes: 1_048_576,
                maximum_total_bytes: 8_388_608,
            },
        };
        let metadata =
            bytes(&json!({"fixture":"synthetic; no real approval or domain acceptance"})).unwrap();
        let preview = d::Preview {
            schema_version: "flow.delivery-preview/v1".into(),
            package_subject_sha256: "a".repeat(64),
            plan_sha256: "c".repeat(64),
            metadata_sha256: release_cohort::sha256(&metadata),
            requested_destinations: vec![Destination::Archive],
            payloads: vec![Payload {
                id: "original".into(),
                role: Role::OriginalAudio,
                relative: "payload/original.wav".into(),
                sha256: original.sha256.clone(),
                byte_count: original.byte_count,
                destinations: vec![Destination::Archive],
                source_state: "unchanged_selected_original_audio".into(),
            }],
            selected_sources: vec![d::SelectedSource {
                payload_id: "original".into(),
                file: original.clone(),
                source_state: "unchanged_selected_original_audio".into(),
            }],
            reviews: d::reviews(&[], &"a".repeat(64)).unwrap(),
            missing_destination_payloads: vec![],
            missing_metadata: vec!["creators".into()],
            provider_history: "synthetic local copied-byte fixture; no provider invoked".into(),
            publishing_authority: false,
            current_platform_compliance_verified: false,
        };
        let intent_bytes = bytes(&intent).unwrap();
        let request_sha256 = release_cohort::sha256(&intent_bytes);
        let members = vec![
            ("intent.json", intent_bytes),
            ("metadata.json", metadata),
            (
                "delivery-index.json",
                bytes(&d::public_index(&preview)).unwrap(),
            ),
            (
                "payload/original.wav",
                fs::read(native::path(&original.path).unwrap()).unwrap(),
            ),
        ];
        let mut files = vec![];
        for (relative, data) in members {
            fs::write(package.join(relative), &data).unwrap();
            files.push(d::OwnedFile {
                relative: relative.into(),
                sha256: release_cohort::sha256(&data),
                byte_count: data.len() as u64,
            });
        }
        let manifest = d::Manifest {
            schema_version: "flow.delivery-package/v1".into(),
            original_root: NativePath::from_path(&package),
            visibility: "private_review_package".into(),
            request_sha256,
            preview,
            files,
            review_files: vec![],
            proof_state: "not_asserted".into(),
            evidence_scope: "synthetic copied-byte fixture".into(),
        };
        fs::write(package.join("manifest.json"), bytes(&manifest).unwrap()).unwrap();
        let request = Request {
            schema_version: REQUEST_SCHEMA.into(),
            package_manifest: FilePin::observe(
                &NativePath::from_path(&package.join("manifest.json")),
                MAX_DOC,
                &|| false,
            )
            .unwrap(),
            installation: None,
            workspace: NativePath::from_path(&root.join("setup")),
            selections: vec![Selection {
                payload_id: "original".into(),
                destination: Destination::Archive,
                choice: Choice::KeepArchiveMaster {
                    reason: "Keep synthetic original bytes unchanged".into(),
                },
            }],
            tool_directories: vec![],
            limits: native::Limits::default(),
            timeout_ms: 10_000,
        };
        (temp, request)
    }
    #[test]
    fn archive_only_preparation_readback_and_binding_create_no_provider_or_approval() {
        let (_temp, request) = archive_fixture();
        let value = prepare(&request, &|| false).unwrap();
        assert!(!value.provider_invoked && !value.approvals_generated && !value.native_qualified);
        assert!(value.plans.is_empty());
        let root = native::path(&value.root).unwrap();
        status(&root, &value.record.sha256, &|| false).unwrap();
        let bound = bind(
            &root,
            &value.record.sha256,
            &ApprovalSet {
                schema_version: "flow.audio-check-approvals/v1".into(),
                approvals: vec![],
            },
            &|| false,
        )
        .unwrap();
        assert_eq!(bound.selections.len(), 1);
        assert!(matches!(
            &bound.selections[0].choice,
            audio_readiness::Choice::KeepArchiveMaster { .. }
        ));
        assert_eq!(fs::read_dir(root.join("approvals")).unwrap().count(), 0);
        assert_eq!(fs::read_dir(root.join("native")).unwrap().count(), 0);
        let unknown = root.join("configuration/unknown.json");
        fs::write(&unknown, b"retain me").unwrap();
        assert!(status(&root, &value.record.sha256, &|| false).is_err());
        assert!(unknown.exists());
    }
    #[test]
    fn stale_package_pin_is_refused_before_preparation_writes() {
        let (_temp, mut request) = archive_fixture();
        request.package_manifest.sha256 = "0".repeat(64);
        assert!(prepare(&request, &|| false).is_err());
        assert!(!native::path(&request.workspace).unwrap().exists());
    }
}
