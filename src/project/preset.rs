//! Explicit reusable settings; no new workflow engine or inference defaults.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preset {
    pub schema_version: String,
    pub preset_id: String,
    pub installation: Installation,
    pub video_configuration: NativePath,
    pub destinations: Vec<Destination>,
    pub tool_directories: Vec<NativePath>,
    pub limits: native::Limits,
    pub rows: RowSelection,
    pub preparation: Option<FilePin>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub schema_version: String,
    pub metadata: pilot::Metadata,
    pub video: NativePath,
    pub manual_srt: NativePath,
    pub audio: Option<Audio>,
    pub workspace: NativePath,
    pub destinations: Option<Vec<Destination>>,
    pub rows: Option<RowSelection>,
    pub video_configuration: Option<NativePath>,
}
impl Preset {
    pub fn from_json_slice(raw: &[u8]) -> Result<Self, Error> {
        if raw.len() > 131_072 {
            return Err(error(Code::InvalidInput, "publishing preset exceeds128KiB"));
        }
        let value: Self = serde_json::from_slice(raw)
            .map_err(|_| error(Code::InvalidInput, "closed publishing preset required"))?;
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<(), Error> {
        self.limits.validate()?;
        if self.schema_version != "flow.publishing-project-preset/v1"
            || self.preset_id.is_empty()
            || self.preset_id.len() > 96
            || !self
                .preset_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            || self.destinations.is_empty()
            || self.destinations.len() > 4
            || self
                .destinations
                .iter()
                .map(|d| format!("{d:?}"))
                .collect::<BTreeSet<_>>()
                .len()
                != self.destinations.len()
            || !release_cohort::is_digest(&self.installation.receipt_sha256)
            || self.tool_directories.len() > 32
        {
            return Err(error(
                Code::InvalidInput,
                "explicit bounded publishing preset settings required",
            ));
        }
        for path in [&self.installation.root, &self.video_configuration]
            .into_iter()
            .chain(self.tool_directories.iter())
        {
            native::path(path)?;
        }
        Ok(())
    }
}
impl Selection {
    pub fn from_json_slice(raw: &[u8]) -> Result<Self, Error> {
        if raw.len() > 131_072 {
            return Err(error(
                Code::InvalidInput,
                "publishing selection exceeds128KiB",
            ));
        }
        let value: Self = serde_json::from_slice(raw).map_err(|_| {
            error(
                Code::InvalidInput,
                "closed explicit publishing selection required",
            )
        })?;
        if value.schema_version != "flow.publishing-project-selection/v1" {
            return Err(error(Code::InvalidInput, "unknown publishing selection"));
        }
        Ok(value)
    }
}
#[derive(Debug, Serialize)]
pub struct Resolved {
    pub schema_version: String,
    pub preset_id: String,
    pub preset_sha256: String,
    pub selection_sha256: String,
    pub project_sha256: String,
    pub project: Project,
    pub inherited: Vec<&'static str>,
    pub overridden: Vec<&'static str>,
    pub media_inspected: bool,
    pub provider_invoked: bool,
    pub approvals_generated: bool,
}
/// Semantic configuration identities only. Existing project preview obtains
/// current source/provider pins; existing approval owns any later execution.
pub fn resolve(preset: &Preset, selection: &Selection) -> Result<Resolved, Error> {
    preset.validate()?;
    if selection.schema_version != "flow.publishing-project-selection/v1" {
        return Err(error(Code::InvalidInput, "unknown publishing selection"));
    }
    let srt = native::path(&selection.manual_srt)?;
    if srt
        .extension()
        .and_then(|s| s.to_str())
        .is_none_or(|s| !s.eq_ignore_ascii_case("srt"))
    {
        return Err(error(
            Code::InvalidInput,
            "select supplied manual SRT; no automatic transcription fallback",
        ));
    }
    let project = Project {
        schema_version: "flow.publishing-project/v1".into(),
        metadata: selection.metadata.clone(),
        installation: preset.installation.clone(),
        video: selection.video.clone(),
        manual_srt: selection.manual_srt.clone(),
        audio: selection.audio.clone(),
        workspace: selection.workspace.clone(),
        video_configuration: selection
            .video_configuration
            .clone()
            .unwrap_or_else(|| preset.video_configuration.clone()),
        destinations: selection
            .destinations
            .clone()
            .unwrap_or_else(|| preset.destinations.clone()),
        tool_directories: preset.tool_directories.clone(),
        limits: preset.limits.clone(),
        rows: selection
            .rows
            .clone()
            .unwrap_or_else(|| preset.rows.clone()),
        preparation: preset.preparation.clone(),
    };
    project.validate()?;
    if bytes(&project)?.len() > 131_072 {
        return Err(error(Code::InvalidInput, "resolved project exceeds128KiB"));
    }
    let mut inherited = vec!["installation", "tool_directories", "limits", "preparation"];
    let mut overridden = vec![];
    for (field, selected) in [
        (
            "video_configuration",
            selection.video_configuration.is_some(),
        ),
        ("destinations", selection.destinations.is_some()),
        ("rows", selection.rows.is_some()),
    ] {
        if selected {
            overridden.push(field);
        } else {
            inherited.push(field);
        }
    }
    Ok(Resolved {
        schema_version: "flow.resolved-publishing-project/v1".into(),
        preset_id: preset.preset_id.clone(),
        preset_sha256: hash(preset)?,
        selection_sha256: hash(selection)?,
        project_sha256: hash(&project)?,
        project,
        inherited,
        overridden,
        media_inspected: false,
        provider_invoked: false,
        approvals_generated: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn path(s: &str) -> NativePath {
        NativePath::from_path(Path::new(s))
    }
    fn preset() -> Preset {
        Preset {
            schema_version: "flow.publishing-project-preset/v1".into(),
            preset_id: "manual-amv".into(),
            installation: Installation {
                root: path("/synthetic/install"),
                receipt_sha256: "a".repeat(64),
            },
            video_configuration: path("/synthetic/video.json"),
            destinations: vec![Destination::Archive],
            tool_directories: vec![],
            limits: native::Limits::default(),
            rows: RowSelection {
                video: Some("video-row".into()),
                timed_text: Some("text-row".into()),
                audio: None,
            },
            preparation: None,
        }
    }
    fn selection() -> Selection {
        Selection {
            schema_version: "flow.publishing-project-selection/v1".into(),
            metadata: pilot::Metadata {
                work_id: "akashic-synthetic".into(),
                title: "Synthetic example".into(),
                language: "en".into(),
            },
            video: path("/synthetic/edited video.mp4"),
            manual_srt: path("/synthetic/manual 歌詞.srt"),
            audio: None,
            workspace: path("/synthetic/new-project"),
            destinations: None,
            rows: None,
            video_configuration: None,
        }
    }
    #[test]
    fn reusable_settings_keep_each_project_source_and_identity_explicit() {
        let p = preset();
        let a = selection();
        let first = resolve(&p, &a).unwrap();
        let mut b = a.clone();
        b.metadata.work_id = "second-project".into();
        b.video = path("/synthetic/second.mp4");
        b.workspace = path("/synthetic/second-work");
        let second = resolve(&p, &b).unwrap();
        assert_ne!(first.project_sha256, second.project_sha256);
        assert_eq!(first.preset_sha256, second.preset_sha256);
        assert_eq!(first.project.manual_srt, a.manual_srt);
        assert!(!first.media_inspected && !first.provider_invoked && !first.approvals_generated);
    }
    #[test]
    fn explicit_overrides_replace_whole_settings_without_hidden_merge() {
        let mut selected = selection();
        selected.rows = Some(RowSelection {
            video: Some("other".into()),
            timed_text: None,
            audio: None,
        });
        selected.destinations = Some(vec![Destination::Tiktok]);
        let value = resolve(&preset(), &selected).unwrap();
        assert!(value.project.rows.timed_text.is_none());
        assert!(matches!(
            value.project.destinations.as_slice(),
            [Destination::Tiktok]
        ));
        assert!(value.overridden.contains(&"rows"));
    }
    #[test]
    fn manual_lyrics_and_unknown_effect_fields_cannot_be_defaulted() {
        let mut selected = selection();
        selected.manual_srt = path("/synthetic/auto-transcribe.json");
        assert!(resolve(&preset(), &selected).is_err());
        let mut raw = serde_json::to_value(selection()).unwrap();
        raw.as_object_mut().unwrap().remove("manual_srt");
        assert!(Selection::from_json_slice(&serde_json::to_vec(&raw).unwrap()).is_err());
        let mut raw = serde_json::to_value(preset()).unwrap();
        raw["upload"] = json!(true);
        assert!(Preset::from_json_slice(&serde_json::to_vec(&raw).unwrap()).is_err());
    }
}
