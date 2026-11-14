//! Bounded static/locale publishing preset over the existing native adapter.
use crate::{
    CancellationSignal,
    command::{self, Code, Error},
    distribution,
    native::{self, FilePin},
    optiflow::NativePath,
    pilot, project,
    release_cohort::{Cohort, Tool},
};
use project::{bytes, create, hash, utf8, write};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
const MAX: u64 = 16_777_216;
const DEFAULT_MAXIMUM_STAGED_BYTES: u64 = 268_435_456;
const MAXIMUM_STAGED_BYTES: u64 = 68_719_476_736;
fn default_maximum_staged_bytes() -> u64 {
    DEFAULT_MAXIMUM_STAGED_BYTES
}
fn is_default_maximum_staged_bytes(value: &u64) -> bool {
    *value == DEFAULT_MAXIMUM_STAGED_BYTES
}
fn error(code: Code, s: &str) -> Error {
    Error {
        code,
        message: s.into(),
        recovery_directory: None,
    }
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 96
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}
fn overlap(a: &Path, b: &Path) -> bool {
    a.starts_with(b) || b.starts_with(a)
}
fn read(pin: &FilePin, maximum: u64, token: &dyn CancellationSignal) -> Result<Vec<u8>, Error> {
    pin.current(maximum, token)?;
    let b = command::read_document(&native::path(&pin.path)?, maximum)?;
    if crate::release_cohort::sha256(&b) != pin.sha256 {
        return Err(error(Code::StalePlan, "static project document changed"));
    }
    Ok(b)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageGeometry {
    pub width: f64,
    pub height: f64,
    pub unit: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bleed: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_area: Option<f64>,
}
impl PageGeometry {
    fn validate(&self) -> bool {
        [self.width, self.height]
            .iter()
            .all(|v| v.is_finite() && *v > 0.0)
            && !self.unit.is_empty()
            && self.unit.len() <= 16
            && self.unit.bytes().all(|c| c.is_ascii_alphabetic())
            && [self.margin, self.bleed, self.safe_area]
                .iter()
                .flatten()
                .all(|v| v.is_finite() && *v >= 0.0)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    pub file: FilePin,
    pub format: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry: Option<PageGeometry>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Branch {
    pub id: String,
    pub locale: String,
    pub sources: Vec<Source>,
    pub options: FilePin,
    pub required_artifacts: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub schema_version: String,
    pub metadata: pilot::Metadata,
    pub installation: project::Installation,
    pub workspace: NativePath,
    pub branches: Vec<Branch>,
    pub row_id: Option<String>,
    pub tool_directories: Vec<NativePath>,
    pub limits: native::Limits,
    pub timeout_ms: u64,
    #[serde(
        default = "default_maximum_staged_bytes",
        skip_serializing_if = "is_default_maximum_staged_bytes"
    )]
    pub maximum_staged_bytes: u64,
}
impl Project {
    pub fn from_json_slice(raw: &[u8]) -> Result<Self, Error> {
        if raw.len() > 1_048_576 {
            return Err(error(Code::InvalidInput, "static project exceeds 1 MiB"));
        }
        let value: Self = serde_json::from_slice(raw).map_err(|_| {
            error(
                Code::InvalidInput,
                "closed static publication project required",
            )
        })?;
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<(), Error> {
        self.limits.validate()?;
        native::path(&self.workspace)?;
        if self.maximum_staged_bytes > MAXIMUM_STAGED_BYTES {
            return Err(error(
                Code::InvalidInput,
                "static staging budget exceeds 64 GiB",
            ));
        }
        if self.schema_version != "flow.static-publication-project/v1"
            || self.branches.is_empty()
            || self.branches.len() > 16
            || self.tool_directories.len() > 32
            || self.timeout_ms == 0
            || self.timeout_ms > 86_400_000
            || [
                &self.metadata.work_id,
                &self.metadata.title,
                &self.metadata.language,
            ]
            .iter()
            .any(|s| s.trim().is_empty() || s.len() > 512 || s.chars().any(char::is_control))
        {
            return Err(error(
                Code::InvalidInput,
                "bounded explicit static project required",
            ));
        }
        let mut branches = BTreeSet::new();
        let mut locales = BTreeSet::new();
        let mut total = 0;
        for b in &self.branches {
            total += b.sources.len();
            if !id(&b.id)
                || !branches.insert(&b.id)
                || b.locale.is_empty()
                || b.locale.len() > 35
                || !b
                    .locale
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-')
                || !locales.insert(b.locale.clone())
                || b.sources.is_empty()
                || b.sources.len() > 512
                || b.required_artifacts.is_empty()
                || b.required_artifacts.len() > 128
                || b.required_artifacts
                    .iter()
                    .any(|s| s.is_empty() || s.len() > 256 || s.chars().any(char::is_control))
                || b.required_artifacts.iter().collect::<BTreeSet<_>>().len()
                    != b.required_artifacts.len()
            {
                return Err(error(
                    Code::InvalidInput,
                    "unique locale branches with explicit sources and required target IDs required",
                ));
            }
            let mut ids = BTreeSet::new();
            let mut paths = BTreeSet::new();
            for s in &b.sources {
                if !id(&s.id)
                    || s.id == "flow-collection"
                    || !ids.insert(&s.id)
                    || !paths.insert(native::path(&s.file.path)?)
                    || s.format.is_empty()
                    || s.format.len() > 48
                    || !s
                        .format
                        .bytes()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"_-".contains(&c))
                    || (b.sources.len() > 1 && s.media_type.is_none())
                    || s.media_type.as_ref().is_some_and(|m| {
                        m.is_empty()
                            || m.len() > 128
                            || !m.contains('/')
                            || m.chars().any(char::is_control)
                    })
                    || s.geometry.as_ref().is_some_and(|g| !g.validate())
                {
                    return Err(error(
                        Code::InvalidInput,
                        "unique ordered source IDs/paths and explicit native formats required",
                    ));
                }
            }
        }
        if total > 2048 || !locales.contains(&self.metadata.language) {
            return Err(error(
                Code::InvalidInput,
                "include the declared source/default locale within the total source bound",
            ));
        }
        self.staged_bytes()?;
        Ok(())
    }
    fn staged_bytes(&self) -> Result<u64, Error> {
        // Every collection reference gets its own private copy, including an
        // original reused by another locale. Single-file branches stage none.
        let total = self
            .branches
            .iter()
            .filter(|b| b.sources.len() > 1)
            .flat_map(|b| &b.sources)
            .try_fold(0_u64, |sum, source| {
                sum.checked_add(source.file.byte_count)
                    .ok_or_else(|| error(Code::InvalidInput, "static staged byte total overflow"))
            })?;
        if total > self.maximum_staged_bytes {
            return Err(error(
                Code::InvalidInput,
                "static collection copies exceed maximum_staged_bytes",
            ));
        }
        Ok(total)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BranchPreview {
    pub id: String,
    pub locale: String,
    pub sources: Vec<Source>,
    pub selected_targets: Value,
    pub required_artifacts: Vec<String>,
    pub request: native::Request,
    pub plan_path: NativePath,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preview {
    pub schema_version: String,
    pub project_sha256: String,
    pub metadata: pilot::Metadata,
    pub selected_row: project::SelectedRow,
    pub branches: Vec<BranchPreview>,
    pub source_bytes: u64,
    pub provider_invoked: bool,
    pub native_qualified: bool,
    pub review_pending: Vec<String>,
}
struct Draft {
    preview: Preview,
    inputs: Vec<FilePin>,
    configs: Vec<(PathBuf, Vec<u8>)>,
    copies: Vec<(FilePin, PathBuf)>,
}
fn publication_metadata(
    value: &mut serde_json::Map<String, Value>,
    sources: &[Source],
    locators: &[String],
    options: &Path,
) -> Result<(), Error> {
    if let Some(artwork) = value.get_mut("artwork") {
        let artwork = artwork.as_array_mut().ok_or_else(|| {
            error(
                Code::InvalidInput,
                "native publication artwork array required",
            )
        })?;
        if artwork.len() > 1024 {
            return Err(error(
                Code::InvalidInput,
                "native publication artwork bound",
            ));
        }
        for item in artwork {
            let object = item.as_object_mut().ok_or_else(|| {
                error(
                    Code::InvalidInput,
                    "native publication artwork object required",
                )
            })?;
            let locator = object.get("path").and_then(Value::as_str).ok_or_else(|| {
                error(
                    Code::InvalidInput,
                    "explicit selected publication artwork path required",
                )
            })?;
            let locator = Path::new(locator);
            if locator.as_os_str().is_empty()
                || locator.components().any(|c| {
                    matches!(
                        c,
                        std::path::Component::ParentDir | std::path::Component::CurDir
                    )
                })
            {
                return Err(error(
                    Code::InvalidInput,
                    "publication artwork path must not traverse directories",
                ));
            }
            let selected = if locator.is_absolute() {
                locator.to_path_buf()
            } else {
                options
                    .parent()
                    .ok_or_else(|| error(Code::InvalidInput, "native options parent required"))?
                    .join(locator)
            };
            let index = sources
                .iter()
                .position(|s| native::path(&s.file.path).ok().as_ref() == Some(&selected))
                .ok_or_else(|| {
                    error(
                        Code::InvalidInput,
                        "publication artwork must name an explicitly selected source",
                    )
                })?;
            object.insert("path".into(), json!(locators[index]));
        }
    }
    Ok(())
}
fn draft(p: &Project, token: &dyn CancellationSignal) -> Result<Draft, Error> {
    p.validate()?;
    let root = native::path(&p.workspace)?;
    let parent = root
        .parent()
        .ok_or_else(|| error(Code::InvalidInput, "static workspace parent required"))?;
    if parent.canonicalize().ok().as_deref() != Some(parent)
        || !fs::metadata(parent).is_ok_and(|m| m.is_dir())
    {
        return Err(error(
            Code::InvalidInput,
            "existing canonical static workspace parent required",
        ));
    }
    let installed = distribution::selected_installation(
        &native::path(&p.installation.root)?,
        &p.installation.receipt_sha256,
        token,
    )?;
    let cohort = Cohort::from_json_slice(&read(&installed.cohort, 1_048_576, token)?)
        .map_err(|_| error(Code::InvalidInput, "installed static cohort invalid"))?;
    if Some(installed.target.as_str()) != native::host_target() {
        return Err(error(
            Code::Unavailable,
            "static installation targets another host",
        ));
    }
    let row = project::select_row(
        &installed,
        &cohort,
        Tool::Renderflow,
        "canonical-build-v1",
        p.row_id.as_deref(),
        "static_publication",
    )?;
    let mut protected = vec![native::path(&p.installation.root)?];
    let mut inputs = vec![];
    let mut configs = vec![];
    let mut copies = vec![];
    let mut branches = vec![];
    let mut source_bytes = 0_u64;
    for d in &p.tool_directories {
        let d = native::path(d)?;
        if d.canonicalize().ok().as_deref() != Some(d.as_path())
            || !fs::metadata(&d).is_ok_and(|m| m.is_dir())
        {
            return Err(error(
                Code::InvalidInput,
                "canonical static tool directory required",
            ));
        }
        protected.push(d);
    }
    for (i, b) in p.branches.iter().enumerate() {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Options {
            schema_version: String,
            targets: Value,
            execution: Value,
            #[serde(default)]
            publication: Option<serde_json::Map<String, Value>>,
        }
        let mut options: Options = serde_json::from_slice(&read(&b.options, 1_048_576, token)?)
            .map_err(|_| {
                error(
                    Code::InvalidInput,
                    "closed flow.renderflow-options/v1 required",
                )
            })?;
        if options.schema_version != "flow.renderflow-options/v1"
            || !options.targets.is_object()
            || !options.execution.is_object()
            || options
                .execution
                .get("network")
                .is_some_and(|v| v != "deny")
            || options.execution.get("ai").is_some_and(|v| v != "deny")
            || options.execution.get("requirements").is_some_and(|v| {
                !v.is_object()
                    || v.get("offline").is_some_and(|v| v != true)
                    || v.get("local_only").is_some_and(|v| v != true)
            })
        {
            return Err(error(
                Code::InvalidInput,
                "static preset requires local offline targets without network/AI widening",
            ));
        }
        options.execution["network"] = json!("deny");
        options.execution["ai"] = json!("deny");
        if options.execution.get("requirements").is_none() {
            options.execution["requirements"] = json!({});
        }
        options.execution["requirements"]["offline"] = json!(true);
        options.execution["requirements"]["local_only"] = json!(true);
        let mut sources = vec![];
        let mut native_inputs = vec![];
        let mut locators = vec![];
        for (member, s) in b.sources.iter().enumerate() {
            s.file.current(p.limits.max_input_bytes, token)?;
            let path = native::path(&s.file.path)?;
            protected.push(path.clone());
            inputs.push(s.file.clone());
            source_bytes = source_bytes
                .checked_add(s.file.byte_count)
                .ok_or_else(|| error(Code::InvalidInput, "static source byte overflow"))?;
            let locator = if b.sources.len() > 1 {
                let relative = format!("source-{i:02}/{member:04}.{}", s.format);
                let copied = root.join("configuration").join(&relative);
                copies.push((s.file.clone(), copied.clone()));
                native_inputs.push(NativePath::from_path(&copied));
                relative
            } else {
                native_inputs.push(s.file.path.clone());
                utf8(&path)?.to_owned()
            };
            let mut source = json!({"id":s.id,"path":locator,"format":s.format,"sha256":s.file.sha256,"immutable":true});
            if let Some(media_type) = &s.media_type {
                source["media_type"] = json!(media_type);
            }
            if let Some(geometry) = &s.geometry {
                source["geometry"] = json!(geometry);
            }
            sources.push(source);
            locators.push(locator);
        }
        if b.sources.len() > 1 {
            sources.push(json!({"kind":"collection","id":"flow-collection","members":b.sources.iter().map(|s|&s.id).collect::<Vec<_>>()}));
        }
        let run = root.join("native").join(format!("branch-{i:02}"));
        let configuration = root
            .join("configuration")
            .join(format!("branch-{i:02}.json"));
        let mut config = json!({"schema":"renderflow/v2","sources":sources,"targets":options.targets,"execution":options.execution,"output":{"bundle_root":utf8(&run.join("native/output"))?,"naming_template":"{source.id}/{target.id}.{ext}","collision":"error"}});
        if let Some(mut publication) = options.publication {
            publication_metadata(
                &mut publication,
                &b.sources,
                &locators,
                &native::path(&b.options.path)?,
            )?;
            config["publication"] = Value::Object(publication);
        }
        let data = bytes(&config)?;
        if data.len() > 1_048_576 {
            return Err(error(
                Code::InvalidInput,
                "static configuration exceeds native bound",
            ));
        }
        configs.push((configuration.clone(), data));
        inputs.push(b.options.clone());
        protected.push(native::path(&b.options.path)?);
        branches.push(BranchPreview {
            id: b.id.clone(),
            locale: b.locale.clone(),
            sources: b.sources.clone(),
            selected_targets: config["targets"].clone(),
            required_artifacts: b.required_artifacts.clone(),
            request: native::Request {
                schema_version: native::REQUEST_SCHEMA.into(),
                cohort: installed.cohort.path.clone(),
                cohort_sha256: installed.cohort.sha256.clone(),
                row_id: row.row_id.clone(),
                provider_executable: row.provider.path.clone(),
                release_descriptor: row.descriptor.path.clone(),
                inputs: native_inputs,
                configuration: NativePath::from_path(&configuration),
                run_directory: NativePath::from_path(&run),
                tool_directories: p.tool_directories.clone(),
                limits: p.limits.clone(),
                preparation: None,
            },
            plan_path: NativePath::from_path(
                &root.join("plans").join(format!("branch-{i:02}.json")),
            ),
        });
    }
    if protected.iter().any(|s| overlap(&root, s)) {
        return Err(error(
            Code::InvalidInput,
            "static workspace overlaps source/options, installation or tools",
        ));
    }
    Ok(Draft {
        preview: Preview {
            schema_version: "flow.static-publication-preview/v1".into(),
            project_sha256: hash(p)?,
            metadata: p.metadata.clone(),
            selected_row: row,
            branches,
            source_bytes,
            provider_invoked: false,
            native_qualified: false,
            review_pending: vec![
                "visual and locale/translation content".into(),
                "rights and accessibility".into(),
                "exact final artifact package".into(),
            ],
        },
        inputs,
        configs,
        copies,
    })
}
pub fn preview(p: &Project, token: &dyn CancellationSignal) -> Result<Preview, Error> {
    let start = Instant::now();
    let guard = || token.is_cancelled() || start.elapsed() >= Duration::from_millis(p.timeout_ms);
    Ok(draft(p, &guard)?.preview)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedBranch {
    pub id: String,
    pub locale: String,
    pub required_artifacts: Vec<String>,
    pub plan: native::Plan,
    pub plan_file: FilePin,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema_version: String,
    project: Project,
    preview: Preview,
    inputs: Vec<FilePin>,
    configurations: Vec<FilePin>,
    branches: Vec<PreparedBranch>,
}
#[derive(Debug, Serialize)]
pub struct Prepared {
    pub schema_version: String,
    pub project_sha256: String,
    pub record: FilePin,
    pub branches: Vec<PreparedBranch>,
    pub provider_invoked: bool,
    pub approvals_generated: bool,
    pub native_qualified: bool,
}
pub fn prepare(p: &Project, token: &dyn CancellationSignal) -> Result<Prepared, Error> {
    let start = Instant::now();
    let guard = || token.is_cancelled() || start.elapsed() >= Duration::from_millis(p.timeout_ms);
    let selected = draft(p, &guard)?;
    let root = native::path(&p.workspace)?;
    create(&root)?;
    let action = (|| {
        for name in ["configuration", "plans", "native"] {
            create(&root.join(name))?;
        }
        let mut directories = BTreeSet::new();
        for (source, destination) in &selected.copies {
            let parent = destination
                .parent()
                .ok_or_else(|| error(Code::InvalidInput, "staged source parent required"))?;
            if directories.insert(parent.to_path_buf()) {
                create(parent)?;
            }
            distribution::copy_bounded(
                source,
                destination,
                false,
                p.limits.max_input_bytes,
                &guard,
            )?;
        }
        let mut configurations = vec![];
        for (path, data) in &selected.configs {
            write(path, data)?;
            configurations.push(FilePin::observe(
                &NativePath::from_path(path),
                1_048_576,
                &guard,
            )?);
        }
        let mut branches = vec![];
        for b in &selected.preview.branches {
            let plan = native::prepare(&b.request, &guard)?;
            write(&native::path(&b.plan_path)?, &bytes(&plan)?)?;
            branches.push(PreparedBranch {
                id: b.id.clone(),
                locale: b.locale.clone(),
                required_artifacts: b.required_artifacts.clone(),
                plan,
                plan_file: FilePin::observe(&b.plan_path, MAX, &guard)?,
            });
        }
        for pin in &selected.inputs {
            pin.current(p.limits.max_input_bytes.max(MAX), &guard)?;
        }
        for b in &branches {
            b.plan.require_current(&guard)?;
        }
        let fresh = draft(p, &guard)?;
        if hash(&fresh.preview)? != hash(&selected.preview)? {
            return Err(error(
                Code::StalePlan,
                "static project changed during preparation",
            ));
        }
        let record = Record {
            schema_version: "flow.prepared-static-publication/v1".into(),
            project: p.clone(),
            preview: selected.preview,
            inputs: selected.inputs,
            configurations,
            branches,
        };
        let file = root.join("prepared-static-project.json");
        write(&file, &bytes(&record)?)?;
        fs::File::open(&root)
            .and_then(|f| f.sync_all())
            .map_err(|_| error(Code::Io, "static workspace synchronization"))?;
        Ok(Prepared {
            schema_version: "flow.static-publication-preparation/v1".into(),
            project_sha256: record.preview.project_sha256,
            record: FilePin::observe(&NativePath::from_path(&file), MAX, &guard)?,
            branches: record.branches,
            provider_invoked: false,
            approvals_generated: false,
            native_qualified: false,
        })
    })();
    action.map_err(|mut e: Error| {
        e.recovery_directory = Some(NativePath::from_path(&root));
        e
    })
}
#[derive(Debug, Serialize)]
pub struct BranchStatus {
    pub id: String,
    pub locale: String,
    pub state: String,
    pub artifact_ids: Vec<String>,
    pub missing_required_artifacts: Vec<String>,
}
#[derive(Debug, Serialize)]
pub struct Status {
    pub schema_version: String,
    pub record: FilePin,
    pub branches: Vec<BranchStatus>,
    pub all_required_artifacts_observed: bool,
    pub creative_review_pending: bool,
    pub release_admission_reverified: bool,
    pub upload_authority: bool,
}
pub fn status(
    root: &Path,
    expected_record_sha256: &str,
    token: &dyn CancellationSignal,
) -> Result<Status, Error> {
    let file = FilePin::observe(
        &NativePath::from_path(&root.join("prepared-static-project.json")),
        MAX,
        token,
    )?;
    if file.sha256 != expected_record_sha256 {
        return Err(error(Code::StalePlan, "static record pin differs"));
    }
    let record: Record = serde_json::from_slice(&read(&file, MAX, token)?).map_err(|_| {
        error(
            Code::RecoveryRequired,
            "closed static project record required",
        )
    })?;
    if record.schema_version != "flow.prepared-static-publication/v1"
        || native::path(&record.project.workspace)? != root
    {
        return Err(error(
            Code::RecoveryRequired,
            "static record root/schema differs",
        ));
    }
    let start = Instant::now();
    let guard = || {
        token.is_cancelled() || start.elapsed() >= Duration::from_millis(record.project.timeout_ms)
    };
    let fresh = draft(&record.project, &guard)?;
    if hash(&fresh.preview)? != hash(&record.preview)?
        || hash(&fresh.inputs)? != hash(&record.inputs)?
        || record.branches.len() != fresh.preview.branches.len()
        || record.configurations.len() != fresh.configs.len()
    {
        return Err(error(
            Code::StalePlan,
            "static source/locale/options/provider context changed",
        ));
    }
    for (pin, (path, data)) in record.configurations.iter().zip(&fresh.configs) {
        if native::path(&pin.path)? != *path
            || pin.sha256 != crate::release_cohort::sha256(data)
            || pin.byte_count != data.len() as u64
        {
            return Err(error(
                Code::RecoveryRequired,
                "static configuration mapping differs",
            ));
        }
        pin.current(1_048_576, &guard)?;
    }
    let mut branches = vec![];
    for (b, expected) in record.branches.iter().zip(&fresh.preview.branches) {
        if b.id != expected.id
            || b.locale != expected.locale
            || b.required_artifacts != expected.required_artifacts
            || b.plan.request != expected.request
            || b.plan_file.path != expected.plan_path
        {
            return Err(error(
                Code::RecoveryRequired,
                "static branch/native plan identity differs",
            ));
        }
        b.plan.require_current(&guard)?;
        if native::Plan::from_json_slice(&read(&b.plan_file, MAX, &guard)?)? != b.plan {
            return Err(error(Code::StalePlan, "static native plan changed"));
        }
        let run = native::path(&b.plan.request.run_directory)?;
        let mut artifact_ids = vec![];
        let state = match fs::symlink_metadata(&run) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => "not_started",
            Err(_) => return Err(error(Code::Io, "static native run state unavailable")),
            Ok(m) => {
                if !m.is_dir() || m.file_type().is_symlink() {
                    return Err(error(
                        Code::RecoveryRequired,
                        "static native run is aliased or not a directory",
                    ));
                }
                let observed = native::observe_receipt(&run, &b.plan.plan_sha256, &guard)?;
                if observed.receipt.is_some() {
                    artifact_ids = native::artifact_inventory(&run, &b.plan.plan_sha256, &guard)?
                        .into_iter()
                        .map(|a| a.artifact_id)
                        .collect();
                    "native_accepted"
                } else {
                    "recovery_required"
                }
            }
        };
        let missing = b
            .required_artifacts
            .iter()
            .filter(|id| !artifact_ids.contains(id))
            .cloned()
            .collect();
        branches.push(BranchStatus {
            id: b.id.clone(),
            locale: b.locale.clone(),
            state: state.into(),
            artifact_ids,
            missing_required_artifacts: missing,
        });
    }
    file.current(MAX, &guard)?;
    let complete = branches
        .iter()
        .all(|b| b.state == "native_accepted" && b.missing_required_artifacts.is_empty());
    Ok(Status {
        schema_version: "flow.static-publication-status/v1".into(),
        record: file,
        branches,
        all_required_artifacts_observed: complete,
        creative_review_pending: true,
        release_admission_reverified: false,
        upload_authority: false,
    })
}
#[derive(Debug, Serialize)]
pub struct BranchArtifact {
    pub branch_id: String,
    pub declared_locale: String,
    pub artifact_id: String,
    pub purpose: String,
    pub file: FilePin,
    pub required: bool,
}
#[derive(Debug, Serialize)]
pub struct ArtifactCatalog {
    pub schema_version: String,
    pub metadata: pilot::Metadata,
    pub status: Status,
    pub artifacts: Vec<BranchArtifact>,
    pub locale_content_reviewed: bool,
    pub upload_authority: bool,
}
fn correlate_artifacts(
    branch_id: &str,
    locale: &str,
    required: &[String],
    status: &BranchStatus,
    inventory: Vec<native::ArtifactDescription>,
) -> Result<Vec<BranchArtifact>, Error> {
    let ids = inventory
        .iter()
        .map(|a| a.artifact_id.clone())
        .collect::<Vec<_>>();
    if branch_id != status.id
        || locale != status.locale
        || ids != status.artifact_ids
        || ids.iter().collect::<BTreeSet<_>>().len() != ids.len()
        || status.state != "native_accepted" && !inventory.is_empty()
    {
        return Err(error(
            Code::StalePlan,
            "static artifact inventory differs from observed branch status",
        ));
    }
    Ok(inventory
        .into_iter()
        .map(|a| BranchArtifact {
            branch_id: branch_id.into(),
            declared_locale: locale.into(),
            required: required.contains(&a.artifact_id),
            artifact_id: a.artifact_id,
            purpose: a.purpose,
            file: a.file,
        })
        .collect())
}
/// Exact current output references from existing native owners; declarations
/// and retained acceptance never substitute for locale/content/publication review.
pub fn artifact_catalog(
    root: &Path,
    expected_record_sha256: &str,
    token: &dyn CancellationSignal,
) -> Result<ArtifactCatalog, Error> {
    let started = Instant::now();
    let before = status(root, expected_record_sha256, token)?;
    let record: Record = serde_json::from_slice(&read(&before.record, MAX, token)?)
        .map_err(|_| error(Code::RecoveryRequired, "closed static record required"))?;
    let guard = || {
        token.is_cancelled()
            || started.elapsed() >= Duration::from_millis(record.project.timeout_ms)
    };
    let mut artifacts = vec![];
    if record.branches.len() != before.branches.len() {
        return Err(error(
            Code::StalePlan,
            "static catalog branch membership changed",
        ));
    }
    for (branch, observed) in record.branches.iter().zip(&before.branches) {
        if guard() {
            return Err(error(
                Code::Cancelled,
                "static catalog cancelled or deadline exhausted",
            ));
        }
        let inventory = if observed.state == "native_accepted" {
            native::artifact_inventory(
                &native::path(&branch.plan.request.run_directory)?,
                &branch.plan.plan_sha256,
                &guard,
            )?
        } else {
            vec![]
        };
        artifacts.extend(correlate_artifacts(
            &branch.id,
            &branch.locale,
            &branch.required_artifacts,
            observed,
            inventory,
        )?);
        if artifacts.len() > 4096 {
            return Err(error(
                Code::InvalidInput,
                "static artifact catalog exceeds4096 entries",
            ));
        }
    }
    let after = status(root, expected_record_sha256, &guard)?;
    if hash(&before)? != hash(&after)? {
        return Err(error(
            Code::StalePlan,
            "static branch status changed while collecting output references",
        ));
    }
    if guard() {
        return Err(error(
            Code::Cancelled,
            "static catalog cancelled or deadline exhausted",
        ));
    }
    Ok(ArtifactCatalog {
        schema_version: "flow.static-publication-artifact-catalog/v1".into(),
        metadata: record.project.metadata,
        status: after,
        artifacts,
        locale_content_reviewed: false,
        upload_authority: false,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn staging_project() -> Value {
        let pin = |name: &str| json!({"path":{"encoding":"utf8","value":format!("/synthetic/{name}")},"sha256":"a".repeat(64),"byte_count":219});
        let branch = |locale: &str| {
            json!({"id":locale,"locale":locale,"sources":[
            {"id":"one","file":pin("one.png"),"format":"png","media_type":"image/png"},
            {"id":"two","file":pin("two.png"),"format":"png","media_type":"image/png"}],
            "options":pin("options.json"),"required_artifacts":["synthetic"]})
        };
        json!({"schema_version":"flow.static-publication-project/v1","metadata":{"work_id":"synthetic","title":"Synthetic","language":"en"},
            "installation":{"root":{"encoding":"utf8","value":"/synthetic/install"},"receipt_sha256":"b".repeat(64)},
            "workspace":{"encoding":"utf8","value":"/synthetic/fresh"},"branches":[branch("en"),branch("fr")],
            "tool_directories":[],"limits":native::Limits::default(),"timeout_ms":1000})
    }
    fn parse_project(value: &Value) -> Result<Project, Error> {
        Project::from_json_slice(&serde_json::to_vec(value).unwrap())
    }
    #[test]
    fn staging_budget_counts_repeated_originals_for_each_locale_copy() {
        let mut value = staging_project();
        value["maximum_staged_bytes"] = json!(876);
        let project = parse_project(&value).unwrap();
        assert_eq!(project.staged_bytes().unwrap(), 876);
        value["maximum_staged_bytes"] = json!(875);
        assert_eq!(
            parse_project(&value).unwrap_err().message,
            "static collection copies exceed maximum_staged_bytes"
        );
        value["maximum_staged_bytes"] = json!(MAXIMUM_STAGED_BYTES + 1);
        assert_eq!(
            parse_project(&value).unwrap_err().message,
            "static staging budget exceeds 64 GiB"
        );
    }
    #[test]
    fn zero_staging_budget_allows_only_unstaged_single_file_branches() {
        let mut value = staging_project();
        value["maximum_staged_bytes"] = json!(0);
        assert!(parse_project(&value).is_err());
        for branch in value["branches"].as_array_mut().unwrap() {
            branch["sources"].as_array_mut().unwrap().truncate(1);
        }
        assert_eq!(parse_project(&value).unwrap().staged_bytes().unwrap(), 0);
    }
    #[test]
    fn default_staging_budget_preserves_project_identity_and_explicit_limits_bind() {
        let mut value = staging_project();
        let project = parse_project(&value).unwrap();
        assert!(
            serde_json::to_value(&project)
                .unwrap()
                .get("maximum_staged_bytes")
                .is_none()
        );
        value["maximum_staged_bytes"] = json!(DEFAULT_MAXIMUM_STAGED_BYTES);
        assert_eq!(
            hash(&parse_project(&value).unwrap()).unwrap(),
            hash(&project).unwrap()
        );
        value["maximum_staged_bytes"] = json!(876);
        assert_ne!(
            hash(&parse_project(&value).unwrap()).unwrap(),
            hash(&project).unwrap()
        );
        value["branches"][0]["sources"][0]["file"]["byte_count"] = json!(u64::MAX);
        assert_eq!(
            parse_project(&value).unwrap_err().message,
            "static staged byte total overflow"
        );
    }
    #[test]
    fn selected_artwork_rebinding_preserves_metadata_and_refuses_scope_widening() {
        let source = Source {
            id: "page".into(),
            file: FilePin {
                path: NativePath::from_path(Path::new("/selected/作品.png")),
                sha256: "a".repeat(64),
                byte_count: 10,
            },
            format: "png".into(),
            media_type: Some("image/png".into()),
            geometry: None,
        };
        let mut metadata = serde_json::from_value::<serde_json::Map<String, Value>>(json!({"title":"Explicit title","artwork":[{"role":"page","path":"作品.png","alt_text":"Supplied words"}]})).unwrap();
        publication_metadata(
            &mut metadata,
            &[source.clone()],
            &["source-00/0000.png".into()],
            Path::new("/selected/options.json"),
        )
        .unwrap();
        assert_eq!(metadata["title"], "Explicit title");
        assert_eq!(metadata["artwork"][0]["path"], "source-00/0000.png");
        assert_eq!(metadata["artwork"][0]["alt_text"], "Supplied words");
        for locator in ["/other/unselected.png", "../作品.png"] {
            metadata["artwork"][0]["path"] = json!(locator);
            assert!(
                publication_metadata(
                    &mut metadata,
                    &[source.clone()],
                    &["source-00/0000.png".into()],
                    Path::new("/selected/options.json")
                )
                .is_err()
            );
        }
    }
    #[test]
    fn page_geometry_refuses_nonfinite_and_negative_boundaries() {
        let mut g = PageGeometry {
            width: 90.0,
            height: 90.0,
            unit: "mm".into(),
            margin: None,
            bleed: Some(5.0),
            safe_area: None,
        };
        assert!(g.validate());
        g.width = f64::NAN;
        assert!(!g.validate());
        g.width = 90.0;
        g.bleed = Some(-1.0);
        assert!(!g.validate());
    }
    fn artifact(id: &str) -> native::ArtifactDescription {
        native::ArtifactDescription {
            artifact_id: id.into(),
            purpose: "synthetic_target".into(),
            file: FilePin {
                path: NativePath::from_path(Path::new("/synthetic/output.pdf")),
                sha256: "a".repeat(64),
                byte_count: 10,
            },
        }
    }
    #[test]
    fn catalog_keeps_required_targets_distinct_from_observed_payloads() {
        let status = BranchStatus {
            id: "en".into(),
            locale: "en".into(),
            state: "native_accepted".into(),
            artifact_ids: vec!["pdf".into()],
            missing_required_artifacts: vec!["epub".into()],
        };
        let values = correlate_artifacts(
            "en",
            "en",
            &["pdf".into(), "epub".into()],
            &status,
            vec![artifact("pdf")],
        )
        .unwrap();
        assert_eq!(values.len(), 1);
        assert!(values[0].required);
        assert_eq!(values[0].declared_locale, "en");
        assert_eq!(status.missing_required_artifacts, vec!["epub"]);
    }
    #[test]
    fn stale_or_orphan_artifact_inventories_cannot_enter_a_locale_catalog() {
        let mut status = BranchStatus {
            id: "en".into(),
            locale: "en".into(),
            state: "native_accepted".into(),
            artifact_ids: vec!["pdf".into()],
            missing_required_artifacts: vec![],
        };
        assert!(correlate_artifacts("en", "en", &[], &status, vec![artifact("epub")]).is_err());
        assert!(correlate_artifacts("en", "fr", &[], &status, vec![artifact("pdf")]).is_err());
        status.state = "recovery_required".into();
        assert!(correlate_artifacts("en", "en", &[], &status, vec![artifact("pdf")]).is_err());
    }
    #[test]
    fn locale_labels_require_default_sources_and_unique_artifact_requirements() {
        let pin = json!({"path":{"encoding":"utf8","value":"/synthetic/page.png"},"sha256":"b".repeat(64),"byte_count":10});
        let raw = json!({"schema_version":"flow.static-publication-project/v1","metadata":{"work_id":"synthetic","title":"Synthetic","language":"en"},"installation":{"root":{"encoding":"utf8","value":"/synthetic/install"},"receipt_sha256":"a".repeat(64)},"workspace":{"encoding":"utf8","value":"/synthetic/work"},"branches":[{"id":"en","locale":"en","sources":[{"id":"page-1","file":pin,"format":"png"}],"options":pin,"required_artifacts":["publication-pdf"]}],"tool_directories":[],"limits":native::Limits::default(),"timeout_ms":1000});
        let project = Project::from_json_slice(&serde_json::to_vec(&raw).unwrap()).unwrap();
        let mut wrong = project.clone();
        wrong.branches[0].locale = "fr".into();
        assert!(wrong.validate().is_err());
        let mut duplicate = project;
        duplicate.branches[0]
            .required_artifacts
            .push("publication-pdf".into());
        assert!(duplicate.validate().is_err());
        assert!(!id("../escape"));
    }
}
