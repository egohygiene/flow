//! A release/capability inventory and a projection of independently verified facts.
//!
//! Cohort JSON is an operator-selected declaration, never qualification evidence.
//! The evidence port is a trusted host boundary: implementations must verify the
//! exact row subject, not copy a provider's `available`/`passed` assertion. The
//! built-in file observer establishes installed bytes only. It neither contacts a
//! release service nor invokes a provider, downloads dependencies or qualifies it.
use std::{collections::BTreeSet, fs, io::Read, path::{Path, PathBuf}};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::CancellationSignal;

pub const COHORT_SCHEMA: &str = "flow.release-cohort/v1";
pub const MATRIX_SCHEMA: &str = "flow.release-matrix/v1";
pub mod coverage;
const MAX_DOCUMENT: usize = 1_048_576;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid or oversized release cohort: {0}")]
    Invalid(&'static str),
    #[error("release observation cancelled")]
    Cancelled,
    #[error("release file observation failed")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tool { Flow, Aniflow, Renderflow, Optiflow }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Interface { StandaloneCli, RustLibrary, FlowRoute }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Implementation { Implemented, Planned, Unsupported }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub version: String,
    pub source_revision: String,
    pub name: String,
    pub sha256: String,
    pub byte_count: u64,
    /// A declaration of location, not authenticated publication evidence.
    pub release_url: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    pub identifier: String,
    pub sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    pub id: String,
    pub kind: String,
    pub constraint: String,
    pub preparation: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub id: String,
    pub tool: Tool,
    pub interface: Interface,
    pub target: String,
    pub profile: String,
    pub input: String,
    pub output: String,
    pub effects: Vec<String>,
    pub implementation: Implementation,
    pub artifact: Option<Artifact>,
    pub contracts: Vec<Contract>,
    pub requirements: Vec<Requirement>,
    pub owner_issue: String,
    pub next_action: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cohort {
    pub schema_version: String,
    pub id: String,
    pub flow_version: String,
    /// Coverage inventory is retained even when no artifact exists for a target.
    pub targets: Vec<String>,
    pub retained_rows: Vec<String>,
    pub rows: Vec<Row>,
}
fn text(value: &str) -> bool { !value.is_empty() && value.len() <= 4096 && !value.chars().any(char::is_control) }
pub(crate) fn sha256(value: &[u8]) -> String { format!("{:x}", Sha256::digest(value)) }
pub(crate) fn is_digest(value: &str) -> bool { value.len() == 64 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) }
impl Cohort {
    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_DOCUMENT { return Err(Error::Invalid("document bound")); }
        let value: Self = serde_json::from_slice(bytes).map_err(|_| Error::Invalid("closed JSON"))?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<(), Error> {
        if self.schema_version != COHORT_SCHEMA || !text(&self.id) || semver::Version::parse(&self.flow_version).is_err()
            || self.rows.is_empty() || self.rows.len() > 4096 || self.targets.is_empty() || self.targets.len() > 64
            || self.targets.iter().any(|v| !text(v)) { return Err(Error::Invalid("cohort identity or bounds")); }
        let targets: BTreeSet<_> = self.targets.iter().cloned().collect();
        let retained: BTreeSet<_> = self.retained_rows.iter().cloned().collect();
        let ids: BTreeSet<_> = self.rows.iter().map(|r| r.id.clone()).collect();
        if targets.len() != self.targets.len() || retained.len() != self.retained_rows.len() || ids.len() != self.rows.len() || ids != retained {
            return Err(Error::Invalid("duplicate or omitted retained coverage"));
        }
        for row in &self.rows {
            if [&row.id, &row.target, &row.profile, &row.input, &row.output, &row.owner_issue, &row.next_action].iter().any(|v| !text(v))
                || !targets.contains(&row.target) || row.effects.len() > 32 || row.effects.iter().any(|v| !text(v))
                || row.contracts.len() > 64 || row.requirements.len() > 256 { return Err(Error::Invalid("row bounds or coverage")); }
            if !row.owner_issue.starts_with("https://github.com/egohygiene/") { return Err(Error::Invalid("owning issue")); }
            let mut contracts = BTreeSet::new();
            for contract in &row.contracts {
                if !text(&contract.identifier) || !is_digest(&contract.sha256) || !contracts.insert(&contract.identifier) { return Err(Error::Invalid("contract pin")); }
            }
            let mut requirements = BTreeSet::new();
            for requirement in &row.requirements {
                if [&requirement.id, &requirement.kind, &requirement.constraint, &requirement.preparation].iter().any(|v| !text(v)) || !requirements.insert(&requirement.id) {
                    return Err(Error::Invalid("requirement identity"));
                }
            }
            if let Some(artifact) = &row.artifact {
                if semver::Version::parse(&artifact.version).is_err() || !is_digest(&artifact.sha256) || artifact.byte_count == 0
                    || artifact.source_revision.len() != 40 || !artifact.source_revision.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    || !text(&artifact.name) || !artifact.release_url.starts_with("https://github.com/egohygiene/") {
                    return Err(Error::Invalid("immutable artifact pin"));
                }
            }
        }
        Ok(())
    }
}
/// Binding covers the complete direction, effects, requirements and release tuple.
pub fn subject(row: &Row) -> Result<String, Error> {
    let value = serde_json::to_value(row).map_err(|_| Error::Invalid("row encoding"))?;
    Ok(sha256(&serde_json::to_vec(&value).map_err(|_| Error::Invalid("row encoding"))?))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Dimension { Contracts, Publication, Installation, Runtime, Qualification }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum State { Unobserved, Failed, Passed }
/// Host observations deliberately have no JSON deserializer. This port is an
/// authority boundary, not another manifest input or provider result envelope.
#[derive(Debug, Clone, Serialize)]
pub struct Observation {
    pub subject_sha256: String,
    pub state: State,
    pub evidence_sha256: Option<String>,
    pub reason: String,
}
impl Observation {
    pub fn unobserved(row: &Row, reason: &str) -> Self {
        Self { subject_sha256: subject(row).unwrap_or_default(), state: State::Unobserved, evidence_sha256: None, reason: reason.into() }
    }
}
/// Implement only over independently verified current evidence. Publication must
/// verify the exact release asset; qualification must verify positive native,
/// installation and domain acceptance evidence for this exact target/profile.
pub trait EvidencePort {
    fn observe(&self, row: &Row, dimension: Dimension, cancellation: &dyn CancellationSignal) -> Result<Observation, Error>;
}
#[derive(Debug, Clone, Serialize)]
pub struct Cell { pub dimension: Dimension, pub observation: Observation }
#[derive(Debug, Clone, Serialize)]
pub struct MatrixRow { pub declaration: Row, pub subject_sha256: String, pub green: bool, pub cells: Vec<Cell> }
#[derive(Debug, Clone, Serialize)]
pub struct Matrix {
    pub schema_version: String,
    pub cohort_id: String,
    pub rows: Vec<MatrixRow>,
    pub green: usize,
    pub blocked: usize,
    pub unverified: usize,
}
impl Matrix {
    pub fn human(&self) -> String {
        let mut view = format!("Cohort {}: {} green, {} blocked, {} unverified\n", self.cohort_id, self.green, self.blocked, self.unverified);
        for row in &self.rows {
            view.push_str(&format!("{}: {}\n", row.declaration.id, if row.green { "green" } else { "not qualified" }));
            for cell in &row.cells { view.push_str(&format!("  {:?}: {:?} — {}\n", cell.dimension, cell.observation.state, cell.observation.reason)); }
            view.push_str(&format!("  Next: {} ({})\n", row.declaration.next_action, row.declaration.owner_issue));
        }
        view
    }
}
pub fn project(cohort: &Cohort, evidence: &dyn EvidencePort, cancellation: &dyn CancellationSignal) -> Result<Matrix, Error> {
    cohort.validate()?;
    let mut matrix = Matrix { schema_version: MATRIX_SCHEMA.into(), cohort_id: cohort.id.clone(), rows: vec![], green: 0, blocked: 0, unverified: 0 };
    for row in &cohort.rows {
        let binding = subject(row)?;
        let mut cells = vec![];
        for dimension in [Dimension::Contracts, Dimension::Publication, Dimension::Installation, Dimension::Runtime, Dimension::Qualification] {
            if cancellation.is_cancelled() { return Err(Error::Cancelled); }
            let observation = evidence.observe(row, dimension, cancellation)?;
            if observation.subject_sha256 != binding || !text(&observation.reason)
                || observation.evidence_sha256.as_ref().is_some_and(|v| !is_digest(v))
                || (observation.state == State::Passed && observation.evidence_sha256.is_none()) {
                return Err(Error::Invalid("unbound observation or missing evidence pin"));
            }
            cells.push(Cell { dimension, observation });
        }
        let green = row.implementation == Implementation::Implemented && row.artifact.is_some() && !row.contracts.is_empty()
            && cells.iter().all(|c| c.observation.state == State::Passed);
        if green { matrix.green += 1; }
        else if row.implementation != Implementation::Implemented || row.artifact.is_none() || cells.iter().any(|c| c.observation.state == State::Failed) { matrix.blocked += 1; }
        else { matrix.unverified += 1; }
        matrix.rows.push(MatrixRow { declaration: row.clone(), subject_sha256: binding, green, cells });
    }
    Ok(matrix)
}

/// Explicit installed paths; no PATH lookup, source checkout or automatic setup.
#[derive(Debug, Clone)]
pub struct InstalledFiles {
    pub row_id: String,
    pub executable: PathBuf,
    pub contract_files: Vec<(String, PathBuf)>,
}
pub struct LocalFiles { pub installations: Vec<InstalledFiles> }
impl EvidencePort for LocalFiles {
    fn observe(&self, row: &Row, dimension: Dimension, token: &dyn CancellationSignal) -> Result<Observation, Error> {
        if token.is_cancelled() { return Err(Error::Cancelled); }
        let Some(files) = self.installations.iter().find(|v| v.row_id == row.id) else { return Ok(Observation::unobserved(row, "no explicit local installation selected")); };
        let artifact = match &row.artifact { Some(v) => v, None => return Ok(Observation::unobserved(row, "no immutable artifact pin in cohort")) };
        let (passed, evidence) = match dimension {
            Dimension::Installation => {
                let observed = match observe_file(&files.executable, 536_870_912, token) {
                    Ok(observed) => observed,
                    Err(Error::Cancelled) => return Err(Error::Cancelled),
                    Err(_) => return Ok(Observation { subject_sha256: subject(row)?, state: State::Failed, evidence_sha256: None, reason: "explicit installed artifact is missing, unreadable or outside file bounds".into() }),
                };
                (observed.0 == artifact.byte_count && observed.1 == artifact.sha256, observed.1)
            }
            Dimension::Contracts if !row.contracts.is_empty() => {
                let mut identities = vec![];
                for pin in &row.contracts {
                    let Some((_, path)) = files.contract_files.iter().find(|(id, _)| id == &pin.identifier) else { return Ok(Observation::unobserved(row, "a pinned contract file is missing")); };
                    let observed = match observe_file(path, MAX_DOCUMENT as u64, token) {
                        Ok(observed) => observed,
                        Err(Error::Cancelled) => return Err(Error::Cancelled),
                        Err(_) => return Ok(Observation { subject_sha256: subject(row)?, state: State::Failed, evidence_sha256: None, reason: "explicit installed contract file is missing, unreadable or outside file bounds".into() }),
                    };
                    if observed.1 != pin.sha256 { return Ok(Observation { subject_sha256: subject(row)?, state: State::Failed, evidence_sha256: Some(observed.1), reason: "installed contract bytes differ".into() }); }
                    identities.push((pin.identifier.clone(), observed.1));
                }
                (true, sha256(&serde_json::to_vec(&identities).map_err(|_| Error::Invalid("contract encoding"))?))
            }
            _ => return Ok(Observation::unobserved(row, "local byte observation cannot establish publication, runtime availability or native qualification")),
        };
        Ok(Observation { subject_sha256: subject(row)?, state: if passed { State::Passed } else { State::Failed }, evidence_sha256: Some(evidence), reason: if passed { "current installed bytes match the operator pin" } else { "current installed bytes differ from the operator pin" }.into() })
    }
}
pub(crate) fn observe_file(path: &Path, limit: u64, token: &dyn CancellationSignal) -> Result<(u64, String), Error> {
    if !path.is_absolute() || path.canonicalize().ok().as_deref() != Some(path) { return Err(Error::Invalid("canonical absolute regular file required")); }
    let before = fs::symlink_metadata(path)?;
    if !before.is_file() || before.file_type().is_symlink() || before.len() > limit { return Err(Error::Invalid("regular file bound")); }
    let mut options = fs::OpenOptions::new(); options.read(true);
    #[cfg(unix)] { use std::os::unix::fs::OpenOptionsExt; options.custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_CLOEXEC | nix::libc::O_NONBLOCK); }
    let mut file = options.open(path)?;
    fn same(a: &fs::Metadata, b: &fs::Metadata) -> bool {
        if !b.is_file() || a.len() != b.len() || a.modified().ok() != b.modified().ok() { return false; }
        #[cfg(unix)] { use std::os::unix::fs::MetadataExt; if a.dev() != b.dev() || a.ino() != b.ino() || a.ctime() != b.ctime() || a.ctime_nsec() != b.ctime_nsec() || b.nlink() != 1 { return false; } }
        true
    }
    if !same(&before, &file.metadata()?) { return Err(Error::Invalid("changed file")); }
    let mut hasher = Sha256::new(); let mut total = 0_u64; let mut buffer = [0_u8; 8192];
    loop {
        if token.is_cancelled() { return Err(Error::Cancelled); }
        let count = file.read(&mut buffer)?; if count == 0 { break; }
        total = total.saturating_add(count as u64); if total > before.len() || total > limit { return Err(Error::Invalid("changed file bound")); }
        hasher.update(&buffer[..count]);
    }
    if token.is_cancelled() { return Err(Error::Cancelled); }
    if total != before.len() || !same(&before, &file.metadata()?) || !same(&before, &fs::symlink_metadata(path)?) { return Err(Error::Invalid("changed file")); }
    Ok((total, format!("{:x}", hasher.finalize())))
}
