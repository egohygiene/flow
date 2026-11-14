//! One publishing project over selected installed interfaces and existing native
//! configuration owners. Preview is read-only; prepare writes a fresh bundle.
pub mod preset;
use crate::{
    CancellationSignal,
    command::{self, Code, Error},
    distribution,
    native::{self, FilePin},
    optiflow::NativePath,
    pilot,
    release_cohort::{self, Cohort, Interface, Row, Tool},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

fn error(code: Code, text: &str) -> Error {
    Error {
        code,
        message: text.into(),
        recovery_directory: None,
    }
}
pub(crate) fn hash<T: Serialize>(value: &T) -> Result<String, Error> {
    let value =
        serde_json::to_value(value).map_err(|_| error(Code::InvalidInput, "project encoding"))?;
    Ok(release_cohort::sha256(
        &serde_json::to_vec(&value).map_err(|_| error(Code::InvalidInput, "project encoding"))?,
    ))
}
fn cancelled(token: &dyn CancellationSignal) -> Result<(), Error> {
    if token.is_cancelled() {
        Err(error(
            Code::Cancelled,
            "project preparation cancelled; preserve partial workspace",
        ))
    } else {
        Ok(())
    }
}
pub(crate) fn utf8(path: &Path) -> Result<&str, Error> {
    path.to_str().ok_or_else(|| {
        error(
            Code::Unavailable,
            "selected native JSON interface requires a UTF-8 path; no lossy conversion",
        )
    })
}
fn overlap(a: &Path, b: &Path) -> bool {
    a.starts_with(b) || b.starts_with(a)
}
pub(crate) fn create(path: &Path) -> Result<(), Error> {
    let parent = path
        .parent()
        .ok_or_else(|| error(Code::InvalidInput, "project parent required"))?;
    if parent.canonicalize().ok().as_deref() != Some(parent)
        || !fs::metadata(parent).is_ok_and(|m| m.is_dir())
    {
        return Err(error(
            Code::InvalidInput,
            "existing canonical project parent required",
        ));
    }
    let mut b = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        b.mode(0o700);
    }
    b.create(path)
        .map_err(|_| error(Code::RecoveryRequired, "fresh project workspace required"))
}
pub(crate) fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    let mut b = serde_json::to_vec_pretty(value)
        .map_err(|_| error(Code::InvalidInput, "project configuration encoding"))?;
    b.push(b'\n');
    Ok(b)
}
pub(crate) fn write(path: &Path, data: &[u8]) -> Result<(), Error> {
    use std::io::Write;
    if data.is_empty() || data.len() > 16_777_216 {
        return Err(error(Code::InvalidInput, "project member size bound"));
    }
    let mut o = fs::OpenOptions::new();
    o.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600)
            .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_CLOEXEC);
    }
    let mut f = o
        .open(path)
        .map_err(|_| error(Code::RecoveryRequired, "new project member required"))?;
    f.write_all(data)
        .and_then(|_| f.sync_all())
        .map_err(|_| error(Code::Io, "project member incomplete; preserve it"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Destination {
    Archive,
    Soundcloud,
    Distrokid,
    Tiktok,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Installation {
    pub root: NativePath,
    pub receipt_sha256: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RowSelection {
    pub video: Option<String>,
    pub timed_text: Option<String>,
    pub audio: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Audio {
    pub path: NativePath,
    pub format: String,
    pub options: NativePath,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub schema_version: String,
    pub metadata: pilot::Metadata,
    pub installation: Installation,
    pub video: NativePath,
    pub manual_srt: NativePath,
    pub audio: Option<Audio>,
    pub video_configuration: NativePath,
    pub workspace: NativePath,
    pub destinations: Vec<Destination>,
    pub tool_directories: Vec<NativePath>,
    pub limits: native::Limits,
    #[serde(default)]
    pub rows: RowSelection,
    #[serde(default)]
    pub preparation: Option<FilePin>,
}
impl Project {
    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > 131_072 {
            return Err(error(Code::InvalidInput, "project exceeds 128 KiB"));
        }
        let p: Self = serde_json::from_slice(bytes)
            .map_err(|_| error(Code::InvalidInput, "closed publishing project required"))?;
        p.validate()?;
        Ok(p)
    }
    fn validate(&self) -> Result<(), Error> {
        self.limits.validate()?;
        let destinations: BTreeSet<_> =
            self.destinations.iter().map(|d| format!("{d:?}")).collect();
        if self.schema_version != "flow.publishing-project/v1"
            || self.destinations.is_empty()
            || self.destinations.len() > 4
            || destinations.len() != self.destinations.len()
            || !release_cohort::is_digest(&self.installation.receipt_sha256)
            || self.tool_directories.len() > 32
            || [
                &self.metadata.work_id,
                &self.metadata.title,
                &self.metadata.language,
            ]
            .into_iter()
            .any(|s| s.is_empty() || s.len() > 512 || s.chars().any(char::is_control))
        {
            return Err(error(
                Code::InvalidInput,
                "project identity, destination or installation selection differs",
            ));
        }
        for path in [
            &self.installation.root,
            &self.video,
            &self.manual_srt,
            &self.video_configuration,
            &self.workspace,
        ]
        .into_iter()
        .chain(self.tool_directories.iter())
        {
            native::path(path)?;
        }
        if let Some(a) = &self.audio {
            native::path(&a.path)?;
            native::path(&a.options)?;
            if a.format.is_empty()
                || a.format.len() > 32
                || !a
                    .format
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
            {
                return Err(error(
                    Code::InvalidInput,
                    "explicit native audio source format required",
                ));
            }
        }
        if self.rows.audio.is_some() && self.audio.is_none() {
            return Err(error(
                Code::InvalidInput,
                "audio row selected without an audio source",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedRow {
    pub role: String,
    pub row_id: String,
    pub tool: Tool,
    pub provider: FilePin,
    pub descriptor: FilePin,
}
#[derive(Debug, Clone, Serialize)]
pub struct Preview {
    pub schema_version: String,
    pub project_sha256: String,
    pub metadata: pilot::Metadata,
    pub sources: Vec<FilePin>,
    pub cohort_sha256: String,
    pub selected_rows: Vec<SelectedRow>,
    pub destinations: Vec<Destination>,
    pub publisher_authenticated_at_install: bool,
    pub provider_invoked: bool,
    pub native_qualified: bool,
    pub review_pending: Vec<String>,
}
struct Draft {
    preview: Preview,
    cohort: FilePin,
    configurations: Vec<(String, Vec<u8>)>,
    sources: Vec<FilePin>,
    requests: Vec<native::Request>,
}

pub(crate) fn select_row(
    installation: &distribution::InstalledSelection,
    cohort: &Cohort,
    tool: Tool,
    profile: &str,
    id: Option<&str>,
    role: &str,
) -> Result<SelectedRow, Error> {
    let installed = installation
        .tools
        .iter()
        .find(|t| t.tool == tool)
        .ok_or_else(|| {
            error(
                Code::Unavailable,
                "project provider is absent from selected installation",
            )
        })?;
    let contract = match tool {
        Tool::Aniflow => "aniflow.release-contract/v1",
        Tool::Renderflow => "renderflow.release-contract/v2",
        _ => {
            return Err(error(
                Code::Unavailable,
                "project provider profile unavailable",
            ));
        }
    };
    let descriptor = installed
        .contracts
        .iter()
        .find(|c| c.identifier == contract)
        .ok_or_else(|| {
            error(
                Code::Unavailable,
                "selected installation lacks compiled native release descriptor",
            )
        })?;
    let rows: Vec<&Row> = cohort
        .rows
        .iter()
        .filter(|row| {
            row.tool == tool
                && row.profile == profile
                && row.interface == Interface::FlowRoute
                && row.implementation == release_cohort::Implementation::Implemented
                && row.target == installation.target
                && id.is_none_or(|id| row.id == id)
                && row.artifact.as_ref().is_some_and(|a| {
                    a.sha256 == installed.executable.sha256
                        && a.byte_count == installed.executable.byte_count
                })
                && row
                    .contracts
                    .iter()
                    .any(|c| c.identifier == contract && c.sha256 == descriptor.file.sha256)
        })
        .collect();
    if rows.len() != 1 {
        return Err(error(
            Code::Unavailable,
            "select exactly one compatible installed Flow row in project.rows; missing or ambiguous profiles are not guessed",
        ));
    }
    Ok(SelectedRow {
        role: role.into(),
        row_id: rows[0].id.clone(),
        tool,
        provider: installed.executable.clone(),
        descriptor: descriptor.file.clone(),
    })
}
fn draft(project: &Project, token: &dyn CancellationSignal) -> Result<Draft, Error> {
    project.validate()?;
    cancelled(token)?;
    let installation = distribution::selected_installation(
        &native::path(&project.installation.root)?,
        &project.installation.receipt_sha256,
        token,
    )?;
    let cohort = Cohort::from_json_slice(&command::read_document(
        &native::path(&installation.cohort.path)?,
        1_048_576,
    )?)
    .map_err(|_| error(Code::InvalidInput, "installed project cohort invalid"))?;
    if Some(installation.target.as_str()) != native::host_target() {
        return Err(error(
            Code::Unavailable,
            "project installation targets another host",
        ));
    }
    let workspace = native::path(&project.workspace)?;
    let parent = workspace
        .parent()
        .ok_or_else(|| error(Code::InvalidInput, "project workspace parent required"))?;
    if parent.canonicalize().ok().as_deref() != Some(parent)
        || !fs::metadata(parent).is_ok_and(|m| m.is_dir())
    {
        return Err(error(
            Code::InvalidInput,
            "existing canonical project parent required",
        ));
    }
    let mut sources = vec![
        FilePin::observe(&project.video, project.limits.max_input_bytes, token)?,
        FilePin::observe(&project.manual_srt, 16_777_216, token)?,
        FilePin::observe(&project.video_configuration, 131_072, token)?,
    ];
    if native::path(&project.manual_srt)?
        .extension()
        .and_then(|s| s.to_str())
        .is_none_or(|s| !s.eq_ignore_ascii_case("srt"))
    {
        return Err(error(
            Code::InvalidInput,
            "project requires manually supplied SRT",
        ));
    }
    if sources[1].byte_count > project.limits.max_input_bytes {
        return Err(error(
            Code::InvalidInput,
            "manual SRT exceeds selected native input budget",
        ));
    }
    let video_config =
        command::read_document(&native::path(&project.video_configuration)?, 131_072)?;
    if release_cohort::sha256(&video_config) != sources[2].sha256 {
        return Err(error(Code::StalePlan, "selected native recipe changed"));
    }
    let identity = hash(project)?;
    let native_parent = workspace.join("native");
    let text_config = json!({"schema":"renderflow/v2","sources":[{"id":"manual-lyrics","path":utf8(&native::path(&project.manual_srt)?)?,"format":"srt","immutable":true}],"targets":{"profiles":["lyrics"]},"execution":{"network":"deny","ai":"deny","max_parallel":1,"requirements":{"offline":true,"local_only":true},"lyrics":{"source_sha256":sources[1].sha256,"review_state":"unreviewed","language":project.metadata.language,"revision":format!("project-{}",&identity[..24])},"derivative_planning":{"max_hops":4,"max_candidates":128}},"output":{"bundle_root":utf8(&native_parent.join("timed-text/native/output"))?,"naming_template":"{source.id}/{target.id}.{ext}","collision":"error"}});
    let mut configurations = vec![
        ("video.json".into(), video_config),
        ("timed-text.json".into(), bytes(&text_config)?),
    ];
    let mut selected_rows = vec![
        select_row(
            &installation,
            &cohort,
            Tool::Aniflow,
            "music-video-source-v1",
            project.rows.video.as_deref(),
            "video",
        )?,
        select_row(
            &installation,
            &cohort,
            Tool::Renderflow,
            "canonical-build-v1",
            project.rows.timed_text.as_deref(),
            "timed_text",
        )?,
    ];
    if let Some(audio) = &project.audio {
        sources.push(FilePin::observe(
            &audio.path,
            project.limits.max_input_bytes,
            token,
        )?);
        let options = FilePin::observe(&audio.options, 1_048_576, token)?;
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Options {
            schema_version: String,
            targets: Value,
            execution: Value,
        }
        let raw = command::read_document(&native::path(&audio.options)?, 1_048_576)?;
        if release_cohort::sha256(&raw) != options.sha256 {
            return Err(error(Code::StalePlan, "audio options changed"));
        }
        let mut selected: Options = serde_json::from_slice(&raw).map_err(|_| {
            error(
                Code::InvalidInput,
                "closed native audio target/execution options required",
            )
        })?;
        if selected.schema_version != "flow.renderflow-options/v1"
            || !selected.targets.is_object()
            || !selected.execution.is_object()
            || selected
                .execution
                .get("network")
                .is_some_and(|v| v != "deny")
            || selected.execution.get("ai").is_some_and(|v| v != "deny")
        {
            return Err(error(
                Code::InvalidInput,
                "audio options require offline native targets and no network/AI",
            ));
        }
        selected.execution["network"] = json!("deny");
        selected.execution["ai"] = json!("deny");
        if selected.execution.get("requirements").is_some_and(|r| {
            !r.is_object()
                || r.get("offline").is_some_and(|v| v != true)
                || r.get("local_only").is_some_and(|v| v != true)
        }) {
            return Err(error(
                Code::InvalidInput,
                "audio options widen local/offline scope",
            ));
        }
        if selected.execution.get("requirements").is_none() {
            selected.execution["requirements"] = json!({});
        }
        selected.execution["requirements"]["offline"] = json!(true);
        selected.execution["requirements"]["local_only"] = json!(true);
        let config = json!({"schema":"renderflow/v2","sources":[{"id":"song","path":utf8(&native::path(&audio.path)?)?,"format":audio.format,"immutable":true}],"targets":selected.targets,"execution":selected.execution,"output":{"bundle_root":utf8(&native_parent.join("audio/native/output"))?,"naming_template":"{source.id}/{target.id}.{ext}","collision":"error"}});
        configurations.push(("audio.json".into(), bytes(&config)?));
        sources.push(options);
        selected_rows.push(select_row(
            &installation,
            &cohort,
            Tool::Renderflow,
            "canonical-build-v1",
            project.rows.audio.as_deref(),
            "audio",
        )?);
    }
    for source in sources
        .iter()
        .map(|s| &s.path)
        .chain(std::iter::once(&project.installation.root))
        .chain(project.tool_directories.iter())
        .chain(project.preparation.iter().map(|p| &p.path))
    {
        if overlap(&workspace, &native::path(source)?) {
            return Err(error(
                Code::InvalidInput,
                "project workspace overlaps originals, installation, options, preparation or tools",
            ));
        }
    }
    let mut requests = vec![];
    for (index, row) in selected_rows.iter().enumerate() {
        let (name, _) = &configurations[index];
        let role = if row.role == "timed_text" {
            "timed-text"
        } else {
            row.role.as_str()
        };
        let input = match row.role.as_str() {
            "video" => project.video.clone(),
            "timed_text" => project.manual_srt.clone(),
            _ => project
                .audio
                .as_ref()
                .ok_or_else(|| error(Code::InvalidInput, "selected audio missing"))?
                .path
                .clone(),
        };
        requests.push(native::Request {
            schema_version: native::REQUEST_SCHEMA.into(),
            cohort: installation.cohort.path.clone(),
            cohort_sha256: installation.cohort.sha256.clone(),
            row_id: row.row_id.clone(),
            provider_executable: row.provider.path.clone(),
            release_descriptor: row.descriptor.path.clone(),
            inputs: vec![input],
            configuration: NativePath::from_path(&workspace.join("configuration").join(name)),
            run_directory: NativePath::from_path(&native_parent.join(role)),
            tool_directories: project.tool_directories.clone(),
            limits: project.limits.clone(),
            preparation: if row.tool == Tool::Aniflow {
                project.preparation.clone()
            } else {
                None
            },
        });
    }
    let preview = Preview {
        schema_version: "flow.publishing-project-preview/v1".into(),
        project_sha256: identity,
        metadata: project.metadata.clone(),
        sources: sources.clone(),
        cohort_sha256: installation.cohort.sha256.clone(),
        selected_rows,
        destinations: project.destinations.clone(),
        publisher_authenticated_at_install: installation.publisher_authenticated_at_install,
        provider_invoked: false,
        native_qualified: false,
        review_pending: vec![
            "manual lyric words and timing".into(),
            "video, audio and caption readability".into(),
            "selected destination package and final publishing approval".into(),
        ],
    };
    Ok(Draft {
        preview,
        cohort: installation.cohort,
        configurations,
        sources,
        requests,
    })
}
pub fn preview(project: &Project, token: &dyn CancellationSignal) -> Result<Preview, Error> {
    Ok(draft(project, token)?.preview)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema_version: String,
    project: Project,
    project_sha256: String,
    sources: Vec<FilePin>,
    configuration_files: Vec<FilePin>,
    plan: pilot::Plan,
    plan_file: FilePin,
}
#[derive(Debug, Serialize)]
pub struct Prepared {
    pub schema_version: String,
    pub root: NativePath,
    pub project_sha256: String,
    pub record: FilePin,
    pub pilot_plan: FilePin,
    pub requested_destinations: Vec<Destination>,
    pub provider_invoked: bool,
    pub native_qualified: bool,
}
pub fn prepare(project: &Project, token: &dyn CancellationSignal) -> Result<Prepared, Error> {
    let selected = draft(project, token)?;
    let root = native::path(&project.workspace)?;
    create(&root)?;
    let action = (|| {
        create(&root.join("configuration"))?;
        create(&root.join("native"))?;
        let mut configs = vec![];
        for (name, data) in &selected.configurations {
            cancelled(token)?;
            let file = root.join("configuration").join(name);
            write(&file, data)?;
            configs.push(FilePin::observe(
                &NativePath::from_path(&file),
                1_048_576,
                token,
            )?);
        }
        let manual = selected.sources[1].clone();
        let audio_source = project
            .audio
            .as_ref()
            .map(|audio| {
                selected
                    .sources
                    .iter()
                    .find(|s| s.path == audio.path)
                    .cloned()
                    .ok_or_else(|| error(Code::InvalidInput, "prepared audio source missing"))
            })
            .transpose()?;
        let request = pilot::Request {
            schema_version: "flow.manual-srt-pilot-request/v1".into(),
            metadata: project.metadata.clone(),
            manual_srt: manual,
            audio_source,
            video: selected.requests[0].clone(),
            timed_text: selected.requests[1].clone(),
            audio: selected.requests.get(2).cloned(),
            run_directory: NativePath::from_path(&root.join("pilot")),
        };
        let plan = pilot::prepare(&request, token)?;
        let path = root.join("pilot-plan.json");
        write(&path, &bytes(&plan)?)?;
        let plan_file = FilePin::observe(&NativePath::from_path(&path), 16_777_216, token)?;
        for source in &selected.sources {
            source.current(project.limits.max_input_bytes.max(1_048_576), token)?;
        }
        distribution::selected_installation(
            &native::path(&project.installation.root)?,
            &project.installation.receipt_sha256,
            token,
        )?;
        selected.cohort.current(1_048_576, token)?;
        let record = Record {
            schema_version: "flow.prepared-publishing-project/v1".into(),
            project: project.clone(),
            project_sha256: selected.preview.project_sha256.clone(),
            sources: selected.sources,
            configuration_files: configs,
            plan,
            plan_file: plan_file.clone(),
        };
        let path = root.join("prepared-project.json");
        write(&path, &bytes(&record)?)?;
        fs::File::open(&root)
            .and_then(|f| f.sync_all())
            .map_err(|_| error(Code::Io, "project directory synchronization"))?;
        Ok(Prepared {
            schema_version: "flow.publishing-project-preparation/v1".into(),
            root: project.workspace.clone(),
            project_sha256: record.project_sha256,
            record: FilePin::observe(&NativePath::from_path(&path), 16_777_216, token)?,
            pilot_plan: plan_file,
            requested_destinations: project.destinations.clone(),
            provider_invoked: false,
            native_qualified: false,
        })
    })();
    action.map_err(|mut e: Error| {
        e.recovery_directory = Some(NativePath::from_path(&root));
        e
    })
}
pub fn status(
    root: &Path,
    expected_record_sha256: &str,
    token: &dyn CancellationSignal,
) -> Result<Prepared, Error> {
    let file = FilePin::observe(
        &NativePath::from_path(&root.join("prepared-project.json")),
        16_777_216,
        token,
    )?;
    if file.sha256 != expected_record_sha256 {
        return Err(error(Code::StalePlan, "prepared project pin changed"));
    }
    let record: Record = serde_json::from_slice(&command::read_document(
        &root.join("prepared-project.json"),
        16_777_216,
    )?)
    .map_err(|_| {
        error(
            Code::RecoveryRequired,
            "closed prepared project record required",
        )
    })?;
    if record.schema_version != "flow.prepared-publishing-project/v1"
        || hash(&record.project)? != record.project_sha256
        || native::path(&record.project.workspace)? != root
        || native::path(&record.plan_file.path)? != root.join("pilot-plan.json")
    {
        return Err(error(
            Code::RecoveryRequired,
            "prepared project identity/root changed",
        ));
    }
    for source in record
        .sources
        .iter()
        .chain(record.configuration_files.iter())
        .chain(std::iter::once(&record.plan_file))
    {
        source.current(record.project.limits.max_input_bytes.max(16_777_216), token)?;
    }
    record.plan.require_current(token)?;
    let plan = pilot::Plan::from_json_slice(&command::read_document(
        &root.join("pilot-plan.json"),
        16_777_216,
    )?)?;
    if plan != record.plan {
        return Err(error(Code::StalePlan, "prepared plan differs from record"));
    }
    distribution::selected_installation(
        &native::path(&record.project.installation.root)?,
        &record.project.installation.receipt_sha256,
        token,
    )?;
    file.current(16_777_216, token)?;
    Ok(Prepared {
        schema_version: "flow.publishing-project-preparation/v1".into(),
        root: record.project.workspace,
        project_sha256: record.project_sha256,
        record: file,
        pilot_plan: record.plan_file,
        requested_destinations: record.project.destinations,
        provider_invoked: false,
        native_qualified: false,
    })
}
