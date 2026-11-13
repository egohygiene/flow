//! Proposed #80 preparation evidence only. No installer, inference or authority.
use crate::{CancellationSignal, ValidationError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs::File, io::Read, net::SocketAddr, path::PathBuf};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Disabled,
    LocalOnly,
    Remote,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    Text,
    Image,
    Audio,
    Video,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budgets {
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub context_tokens: u64,
    pub output_tokens: u64,
    pub deadline_ms: u64,
    pub attempts: u32,
    pub concurrency: u32,
    pub memory_mib: u64,
    pub working_bytes: u64,
    pub hard_cost_microunits: Option<u64>,
}
/// Nonsecret recipe settings. Deployment addresses/paths/credential values never belong here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub schema_version: String,
    pub profile_id: String,
    pub mode: Mode,
    pub provider_id: String,
    pub model_id: String,
    pub runtime_revision: String,
    pub model_revision: String,
    pub template_sha256: String,
    pub output_schema_sha256: String,
    pub modality: Modality,
    pub structured_output: bool,
    pub budgets: Budgets,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    RuntimeExecutable,
    ModelWeights,
    ModelConfiguration,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub asset_id: String,
    pub kind: AssetKind,
    pub path: PathBuf,
    pub sha256: String,
    pub byte_count: u64,
}
/// Private explicit local deployment; no credential or arbitrary URL field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalPreparation {
    pub schema_version: String,
    pub deployment_id: String,
    pub profile_sha256: String,
    pub endpoint: SocketAddr,
    pub service_egress_disabled_declared: bool,
    pub runtime_license: String,
    pub model_license: String,
    pub quantization: String,
    pub maximum_observation_bytes: u64,
    pub assets: Vec<Asset>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetState {
    MatchedBytes,
    Missing,
    Changed,
    Unavailable,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetObservation {
    pub asset_id: String,
    pub kind: AssetKind,
    pub state: AssetState,
    pub observed_sha256: Option<String>,
    pub observed_bytes: Option<u64>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Disabled,
    Refused,
    Incompatible,
    Missing,
    Unavailable,
    Changed,
    PreparedBytes,
}
/// Portable diagnostic allowlist. It carries no paths, endpoint, licenses, model text or secrets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub schema_version: String,
    pub profile_sha256: String,
    pub preparation_sha256: Option<String>,
    pub state: State,
    pub assets: Vec<AssetObservation>,
    pub reasons: Vec<String>,
    pub runtime_qualified: bool,
    pub offline_enforcement_verified: bool,
    pub invocation_authority: bool,
    pub memory_bound_enforced: bool,
    pub cost_known: bool,
}
fn error(message: &str) -> ValidationError {
    ValidationError::new("ai_preparation", message)
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 96
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && !s.starts_with('/')
        && !s.contains("://")
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-:+/".contains(&c))
}
fn pin(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn text(s: &str) -> bool {
    !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control)
}
fn hash<T: Serialize>(value: &T) -> Result<String, ValidationError> {
    let v = serde_json::to_value(value).map_err(|_| error("canonical value unavailable"))?;
    let bytes = serde_json::to_vec(&v).map_err(|_| error("canonical bytes unavailable"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
impl Profile {
    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, ValidationError> {
        if bytes.len() > 32_768 {
            return Err(error("profile exceeds32KiB"));
        }
        let p: Self = serde_json::from_slice(bytes)
            .map_err(|_| error("closed preparation profile required"))?;
        p.validate()?;
        Ok(p)
    }
    pub fn validate(&self) -> Result<(), ValidationError> {
        let b = &self.budgets;
        if self.schema_version != "flow.ai-preparation-profile/v1"
            || !id(&self.profile_id)
            || ![
                &self.provider_id,
                &self.model_id,
                &self.runtime_revision,
                &self.model_revision,
            ]
            .into_iter()
            .all(|s| name(s))
            || !pin(&self.template_sha256)
            || !pin(&self.output_schema_sha256)
        {
            return Err(error(
                "explicit portable identities and schema/template pins required",
            ));
        }
        if b.input_bytes == 0
            || b.input_bytes > 1_048_576
            || b.output_bytes == 0
            || b.output_bytes > 1_048_576
            || b.context_tokens == 0
            || b.context_tokens > 131_072
            || b.output_tokens == 0
            || b.output_tokens > b.context_tokens
            || b.deadline_ms == 0
            || b.deadline_ms > 300_000
            || b.attempts != 1
            || b.concurrency != 1
            || b.memory_mib == 0
            || b.memory_mib > 1_048_576
            || b.working_bytes == 0
            || b.working_bytes > 137_438_953_472
        {
            return Err(error("unsupported or unbounded selected inference budgets"));
        }
        Ok(())
    }
    pub fn sha256(&self) -> Result<String, ValidationError> {
        self.validate()?;
        hash(self)
    }
}
impl LocalPreparation {
    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, ValidationError> {
        if bytes.len() > 65_536 {
            return Err(error("private preparation exceeds64KiB"));
        }
        let p: Self = serde_json::from_slice(bytes)
            .map_err(|_| error("closed private preparation required"))?;
        p.validate()?;
        Ok(p)
    }
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.schema_version != "flow.ai-local-preparation/v1"
            || !id(&self.deployment_id)
            || !pin(&self.profile_sha256)
            || !self.endpoint.ip().is_loopback()
            || self.endpoint.port() == 0
            || ![
                &self.runtime_license,
                &self.model_license,
                &self.quantization,
            ]
            .into_iter()
            .all(|s| text(s))
            || self.maximum_observation_bytes == 0
            || self.maximum_observation_bytes > 137_438_953_472
            || self.assets.is_empty()
            || self.assets.len() > 32
        {
            return Err(error("explicit bounded local preparation required"));
        }
        let mut ids = BTreeSet::new();
        let mut paths = BTreeSet::new();
        let mut total = 0u64;
        for a in &self.assets {
            if !id(&a.asset_id)
                || !ids.insert(&a.asset_id)
                || !paths.insert(&a.path)
                || !a.path.is_absolute()
                || !pin(&a.sha256)
                || a.byte_count == 0
                || a.byte_count > 8_589_934_592
            {
                return Err(error("unique pinned bounded assets required"));
            }
            total = total
                .checked_add(a.byte_count)
                .ok_or_else(|| error("asset byte overflow"))?;
        }
        if total > self.maximum_observation_bytes
            || self
                .assets
                .iter()
                .filter(|a| a.kind == AssetKind::RuntimeExecutable)
                .count()
                != 1
            || !self
                .assets
                .iter()
                .any(|a| a.kind == AssetKind::ModelWeights)
            || !self
                .assets
                .iter()
                .any(|a| a.kind == AssetKind::ModelConfiguration)
        {
            return Err(error(
                "preparation needs one runtime, weights and model configuration within observed byte bound",
            ));
        }
        Ok(())
    }
}
fn observe(
    a: &Asset,
    cancel: &impl CancellationSignal,
) -> Result<AssetObservation, ValidationError> {
    let mut out = AssetObservation {
        asset_id: a.asset_id.clone(),
        kind: a.kind,
        state: AssetState::Unavailable,
        observed_sha256: None,
        observed_bytes: None,
    };
    let before = match std::fs::symlink_metadata(&a.path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            out.state = AssetState::Missing;
            return Ok(out);
        }
        Err(_) => return Ok(out),
    };
    if !before.file_type().is_file() || a.path.canonicalize().ok().as_ref() != Some(&a.path) {
        return Ok(out);
    }
    if before.len() != a.byte_count {
        out.state = AssetState::Changed;
        out.observed_bytes = Some(before.len());
        return Ok(out);
    }
    let Ok(mut file) = File::open(&a.path) else {
        return Ok(out);
    };
    let mut hasher = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0u8; 65_536];
    loop {
        if cancel.is_cancelled() {
            return Err(error("cancelled; no preparation acceptance"));
        }
        let n = match file.read(&mut buffer) {
            Ok(n) => n,
            Err(_) => return Ok(out),
        };
        if n == 0 {
            break;
        }
        bytes = bytes
            .checked_add(n as u64)
            .ok_or_else(|| error("observed byte overflow"))?;
        if bytes > a.byte_count {
            out.state = AssetState::Changed;
            return Ok(out);
        }
        hasher.update(&buffer[..n]);
    }
    let after = match std::fs::symlink_metadata(&a.path) {
        Ok(m) => m,
        Err(_) => return Ok(out),
    };
    if !after.file_type().is_file()
        || a.path.canonicalize().ok().as_ref() != Some(&a.path)
        || after.len() != before.len()
        || after.modified().ok() != before.modified().ok()
    {
        out.state = AssetState::Changed;
        return Ok(out);
    }
    let sha = format!("{:x}", hasher.finalize());
    out.observed_sha256 = Some(sha.clone());
    out.observed_bytes = Some(bytes);
    out.state = if sha == a.sha256 && bytes == a.byte_count {
        AssetState::MatchedBytes
    } else {
        AssetState::Changed
    };
    Ok(out)
}
/// Explicit read-only byte observation. This never opens a socket, executes a
/// runtime, queries a catalog, downloads, writes, installs or grants launch authority.
pub fn inspect(
    profile: &Profile,
    preparation: Option<&LocalPreparation>,
    cancel: &impl CancellationSignal,
) -> Result<Report, ValidationError> {
    profile.validate()?;
    let profile_sha256 = profile.sha256()?;
    let mut report = Report {
        schema_version: "flow.ai-preparation-report/v1".into(),
        profile_sha256: profile_sha256.clone(),
        preparation_sha256: None,
        state: State::Disabled,
        assets: vec![],
        reasons: vec![],
        runtime_qualified: false,
        offline_enforcement_verified: false,
        invocation_authority: false,
        memory_bound_enforced: false,
        cost_known: false,
    };
    if profile.mode == Mode::Disabled {
        return Ok(report);
    }
    if cancel.is_cancelled() {
        return Err(error("cancelled; no preparation acceptance"));
    }
    if profile.mode == Mode::Remote {
        report.state = State::Refused;
        report
            .reasons
            .push("remote_execution_not_supported_by_local_inspection".into());
        return Ok(report);
    }
    if profile.modality != Modality::Text
        || !profile.structured_output
        || profile.budgets.hard_cost_microunits.is_some()
    {
        report.state = State::Incompatible;
        report
            .reasons
            .push("unsupported_modality_output_or_hard_cost_option".into());
        return Ok(report);
    }
    let Some(p) = preparation else {
        report.state = State::Missing;
        report
            .reasons
            .push("explicit_local_preparation_missing".into());
        return Ok(report);
    };
    p.validate()?;
    if p.profile_sha256 != profile_sha256 {
        return Err(error("preparation belongs to a different exact profile"));
    }
    report.preparation_sha256 = Some(hash(p)?);
    if !p.service_egress_disabled_declared {
        report.state = State::Refused;
        report
            .reasons
            .push("local_service_egress_declaration_required".into());
        return Ok(report);
    }
    for a in &p.assets {
        report.assets.push(observe(a, cancel)?);
    }
    // A second bounded full observation detects ordinary changes across the set.
    // It is not descriptor-bound launch, a snapshot, or authenticated runtime evidence.
    for (a, first) in p.assets.iter().zip(&mut report.assets) {
        if first.state == AssetState::MatchedBytes {
            let second = observe(a, cancel)?;
            if second != *first {
                first.state = AssetState::Changed;
            }
        }
    }
    report.state = if report.assets.iter().any(|a| a.state == AssetState::Missing) {
        State::Missing
    } else if report
        .assets
        .iter()
        .any(|a| a.state == AssetState::Unavailable)
    {
        State::Unavailable
    } else if report.assets.iter().any(|a| a.state == AssetState::Changed) {
        State::Changed
    } else {
        State::PreparedBytes
    };
    report.reasons = vec![
        "runtime_native_model_identity_unverified".into(),
        "hardware_and_memory_estimates_unqualified".into(),
        "egress_assertion_not_enforcement".into(),
        "licenses_declared_not_reviewed".into(),
        "no_inference_or_launch_authority".into(),
    ];
    Ok(report)
}
#[cfg(test)]
mod tests;

pub mod setup;
