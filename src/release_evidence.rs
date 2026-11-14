//! Explicit authenticated evidence ingestion. Serialized reports are declarations
//! until a pinned verifier authenticates their exact bytes and current bindings.
use crate::{
    CancellationSignal,
    command::{self, Code, Error},
    native::{self, FilePin},
    optiflow::NativePath,
    release_cohort::{
        self as cohort, Cohort, Contract, Dimension, EvidencePort, InstalledFiles, LocalFiles,
        Observation, Row, State, Tool,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs,
    path::Path,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const MAX: u64 = 16_777_216;
fn error(code: Code, text: &str) -> Error {
    Error {
        code,
        message: text.into(),
        recovery_directory: None,
    }
}
fn hash<T: Serialize>(v: &T) -> Result<String, Error> {
    let v = serde_json::to_value(v).map_err(|_| error(Code::InvalidInput, "evidence encoding"))?;
    Ok(cohort::sha256(&serde_json::to_vec(&v).map_err(|_| {
        error(Code::InvalidInput, "evidence encoding")
    })?))
}
fn read<T: for<'a> Deserialize<'a>>(
    pin: &FilePin,
    limit: u64,
    token: &dyn CancellationSignal,
) -> Result<T, Error> {
    pin.current(limit, token)?;
    let bytes = command::read_document(&native::path(&pin.path)?, limit)?;
    if cohort::sha256(&bytes) != pin.sha256 {
        return Err(error(Code::StalePlan, "evidence document changed"));
    }
    serde_json::from_slice(&bytes)
        .map_err(|_| error(Code::InvalidInput, "closed evidence document required"))
}
fn repo(tool: Tool) -> &'static str {
    match tool {
        Tool::Flow => "egohygiene/flow",
        Tool::Aniflow => "egohygiene/aniflow",
        Tool::Renderflow => "egohygiene/renderflow",
        Tool::Optiflow => "egohygiene/optiflow",
    }
}
fn revision(s: &str) -> bool {
    s.len() == 40
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn safe(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn clock() -> Result<u64, Error> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| error(Code::Unavailable, "evidence clock unavailable"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Owner {
    pub tool: Tool,
    pub repository: String,
    pub signer_workflow: String,
    pub signer_revision: String,
    pub source_revision: String,
    pub source_ref: String,
}
impl Owner {
    fn validate(&self) -> Result<(), Error> {
        let prefix = format!("{}/.github/workflows/", repo(self.tool));
        let name = self
            .signer_workflow
            .strip_prefix(&prefix)
            .ok_or_else(|| error(Code::InvalidInput, "exact owner workflow required"))?;
        if self.repository != repo(self.tool)
            || !safe(name)
            || !name.ends_with(".yml")
            || !revision(&self.signer_revision)
            || !revision(&self.source_revision)
            || !(self.source_ref == "refs/heads/main"
                || self.source_ref.strip_prefix("refs/tags/").is_some_and(safe))
        {
            return Err(error(
                Code::InvalidInput,
                "owner repository/workflow/source policy differs",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub schema_version: String,
    pub owners: Vec<Owner>,
    pub host_identity: FilePin,
    pub maximum_age_seconds: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_credential: Option<FilePin>,
}
impl Policy {
    fn validate(&self) -> Result<(), Error> {
        if self.schema_version != "flow.release-evidence-policy/v1"
            || self.owners.is_empty()
            || self.owners.len() > 4
            || self.maximum_age_seconds == 0
            || self.maximum_age_seconds > 86_400
        {
            return Err(error(
                Code::InvalidInput,
                "bounded explicit evidence policy required",
            ));
        }
        let mut owners = BTreeSet::new();
        for o in &self.owners {
            let prefix = format!("{}/.github/workflows/", repo(o.tool));
            let name = o
                .signer_workflow
                .strip_prefix(&prefix)
                .ok_or_else(|| error(Code::InvalidInput, "exact owner workflow required"))?;
            if o.repository != repo(o.tool)
                || !safe(name)
                || !name.ends_with(".yml")
                || !revision(&o.signer_revision)
                || !revision(&o.source_revision)
                || !owners.insert(o.repository.clone())
                || !(o.source_ref == "refs/heads/main"
                    || o.source_ref.strip_prefix("refs/tags/").is_some_and(safe))
            {
                return Err(error(
                    Code::InvalidInput,
                    "owner repository/workflow/source policy differs",
                ));
            }
        }
        native::path(&self.host_identity.path)?;
        if !cohort::is_digest(&self.host_identity.sha256)
            || self.host_identity.byte_count == 0
            || self.host_identity.byte_count > 65_536
        {
            return Err(error(
                Code::InvalidInput,
                "explicit current host identity pin required",
            ));
        }
        if let Some(pin) = &self.github_credential {
            native::path(&pin.path)?;
            if !cohort::is_digest(&pin.sha256) || pin.byte_count == 0 || pin.byte_count > 4096 {
                return Err(error(
                    Code::InvalidInput,
                    "bounded explicit GitHub credential pin required",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractFile {
    pub identifier: String,
    pub file: FilePin,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub row_id: String,
    pub report: FilePin,
    pub sigstore_bundle: FilePin,
    pub executable: FilePin,
    pub contracts: Vec<ContractFile>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: String,
    pub cohort: FilePin,
    pub selections: Vec<Selection>,
    pub timeout_ms: u64,
}
impl Request {
    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > 1_048_576 {
            return Err(error(Code::InvalidInput, "evidence request bound"));
        }
        let v: Self = serde_json::from_slice(bytes)
            .map_err(|_| error(Code::InvalidInput, "closed evidence request required"))?;
        v.validate()?;
        Ok(v)
    }
    fn validate(&self) -> Result<(), Error> {
        if self.schema_version != "flow.release-evidence-request/v1"
            || self.selections.is_empty()
            || self.selections.len() > 32
            || self.timeout_ms == 0
            || self.timeout_ms > 600_000
        {
            return Err(error(
                Code::InvalidInput,
                "evidence request identity or bounds",
            ));
        }
        let mut rows = BTreeSet::new();
        for s in &self.selections {
            if s.row_id.is_empty()
                || s.row_id.len() > 4096
                || !rows.insert(&s.row_id)
                || s.contracts.len() > 64
            {
                return Err(error(
                    Code::InvalidInput,
                    "duplicate evidence row or contracts bound",
                ));
            }
        }
        for p in self.pins() {
            native::path(&p.path)?;
            if !cohort::is_digest(&p.sha256) || p.byte_count == 0 || p.byte_count > 536_870_912 {
                return Err(error(
                    Code::InvalidInput,
                    "exact bounded evidence file pins required",
                ));
            }
        }
        Ok(())
    }
    fn pins(&self) -> Vec<&FilePin> {
        let mut p = vec![&self.cohort];
        for s in &self.selections {
            p.extend([&s.report, &s.sigstore_bundle, &s.executable]);
            p.extend(s.contracts.iter().map(|c| &c.file));
        }
        p
    }
}

/// Host-selected trust is intentionally outside deserializable pipeline requests.
/// Call only from trusted embedding configuration or explicit root CLI options.
#[derive(Debug, Clone, Serialize)]
pub struct TrustedConfiguration {
    policy: FilePin,
    verifier: FilePin,
}
impl TrustedConfiguration {
    pub fn from_host_configuration(policy: FilePin, verifier: FilePin) -> Result<Self, Error> {
        for (pin, max) in [(&policy, 1_048_576), (&verifier, 536_870_912)] {
            native::path(&pin.path)?;
            if !cohort::is_digest(&pin.sha256) || pin.byte_count == 0 || pin.byte_count > max {
                return Err(error(Code::InvalidInput, "exact host trust pins required"));
            }
        }
        Ok(Self { policy, verifier })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequirementFile {
    pub requirement_id: String,
    pub file: FilePin,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Publication {
    pub release_url: String,
    pub release_id: u64,
    pub asset_id: u64,
    pub published_archive: FilePin,
    pub executable_member_sha256: String,
    pub executable_member_bytes: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Witness {
    pub run_directory: Option<NativePath>,
    pub plan_sha256: String,
    pub acceptance_sha256: String,
    pub accepted_receipt: FilePin,
}
/// A producer-owned positive release gate for standalone/library interfaces.
/// The containing report authenticates this exact file; a unit/refusal report
/// cannot substitute for this closed native evidence vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerGate {
    pub schema_version: String,
    pub row_subject_sha256: String,
    pub artifact_sha256: String,
    pub target: String,
    pub host_identity_sha256: String,
    pub completed_at_unix: u64,
    pub plan_sha256: String,
    pub acceptance_sha256: String,
    pub evidence_kind: String,
    pub accepted_checks: Vec<String>,
    pub failed_checks: Vec<String>,
    pub missing_checks: Vec<String>,
}
/// Positive clean-install smoke acceptance, authenticated through the containing
/// report's exact proof pin. A byte-copy installation receipt has another schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CleanInstall {
    pub schema_version: String,
    pub row_subject_sha256: String,
    pub installed_executable_sha256: String,
    pub contracts: Vec<Contract>,
    pub empty_destination_observed: bool,
    pub positive_native_plan_sha256: String,
    pub positive_acceptance_sha256: String,
}
/// A real native gate record is distinct from refusal/unit/synthetic evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Qualification {
    pub schema_version: String,
    pub row_subject_sha256: String,
    pub evidence_kind: String,
    pub target: String,
    pub host_identity_sha256: String,
    pub completed_at_unix: u64,
    pub native_plan_sha256: String,
    pub acceptance_sha256: String,
    pub missing_checks: Vec<String>,
    pub failed_checks: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub schema_version: String,
    pub cohort_sha256: String,
    pub row_subject_sha256: String,
    pub publication: Publication,
    pub clean_install: FilePin,
    pub qualification: FilePin,
    pub witness: Witness,
    pub requirements: Vec<RequirementFile>,
}

struct Entry {
    selection: Selection,
    report: Report,
    qualification: Qualification,
}
/// Opaque in-memory authority. There is intentionally no deserializer or public
/// constructor accepting a `verified`/`passed` flag or retained verifier output.
pub struct VerifiedEvidence {
    request: Request,
    trust: TrustedConfiguration,
    policy: Policy,
    cohort: Cohort,
    entries: Vec<Entry>,
    binding_sha256: String,
    audit: FilePin,
}
#[derive(Debug, Clone, Serialize)]
pub struct Verification {
    pub schema_version: String,
    pub binding_sha256: String,
    pub cohort_sha256: String,
    pub verified_rows: Vec<String>,
    pub audit: FilePin,
    pub verifier_invoked: bool,
    pub verification_network_permitted: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Admission {
    pub schema_version: String,
    pub binding_sha256: String,
    pub row_subject_sha256: String,
    pub policy_sha256: String,
}

fn remaining(start: Instant, maximum: u64) -> Result<u64, Error> {
    maximum
        .checked_sub(start.elapsed().as_millis().try_into().unwrap_or(u64::MAX))
        .filter(|v| *v > 0)
        .ok_or_else(|| error(Code::Unavailable, "evidence verification deadline exceeded"))
}
fn pins_for<'a>(entry: &'a Entry, policy: &'a Policy) -> Vec<&'a FilePin> {
    let mut p = vec![
        &entry.selection.report,
        &entry.selection.sigstore_bundle,
        &entry.selection.executable,
        &entry.report.publication.published_archive,
        &entry.report.clean_install,
        &entry.report.qualification,
        &entry.report.witness.accepted_receipt,
        &policy.host_identity,
    ];
    if let Some(pin) = &policy.github_credential {
        p.push(pin);
    }
    p.extend(entry.selection.contracts.iter().map(|c| &c.file));
    p.extend(entry.report.requirements.iter().map(|r| &r.file));
    p
}
fn current(
    entry: &Entry,
    row: &Row,
    policy: &Policy,
    token: &dyn CancellationSignal,
) -> Result<(), Error> {
    let now = clock()?;
    let q = &entry.qualification;
    if q.completed_at_unix > now
        || now - q.completed_at_unix > policy.maximum_age_seconds
        || Some(row.target.as_str()) != native::host_target()
    {
        return Err(error(
            Code::Unavailable,
            "native evidence is expired, future-dated or targets another current host",
        ));
    }
    for p in pins_for(entry, policy) {
        p.current(536_870_912, token)?;
    }
    let witness = &entry.report.witness;
    if row.interface == cohort::Interface::FlowRoute
        && matches!(row.tool, Tool::Aniflow | Tool::Renderflow)
    {
        let root = native::path(witness.run_directory.as_ref().ok_or_else(|| {
            error(
                Code::InvalidInput,
                "Flow route requires exact native witness directory",
            )
        })?)?;
        if native::path(&witness.accepted_receipt.path)? != root.join("accepted.json") {
            return Err(error(
                Code::InvalidInput,
                "native witness must pin the exact retained accepted receipt",
            ));
        }
        let status = native::observe_receipt(&root, &witness.plan_sha256, token)?;
        let receipt = status
            .receipt
            .ok_or_else(|| error(Code::Unavailable, "positive native acceptance is absent"))?;
        if status.plan_sha256 != witness.plan_sha256
            || receipt.row_subject_sha256 != entry.report.row_subject_sha256
            || receipt.acceptance_sha256 != witness.acceptance_sha256
            || receipt.state != native::State::Complete
            || receipt.creative_approval_inferred
            || receipt.publication_authority
        {
            return Err(error(
                Code::Unavailable,
                "native witness differs from authenticated positive qualification",
            ));
        }
        let saved: native::Receipt = read(&witness.accepted_receipt, MAX, token)?;
        if saved != receipt {
            return Err(error(
                Code::StalePlan,
                "authenticated native receipt differs from current status",
            ));
        }
    } else {
        if witness.run_directory.is_some() {
            return Err(error(
                Code::InvalidInput,
                "standalone/library gate must use producer-owned native acceptance",
            ));
        }
        let g: OwnerGate = read(&witness.accepted_receipt, 1_048_576, token)?;
        let required: [String; 5] = [
            "native_build",
            "positive_contract_probe",
            "clean_install",
            "positive_smoke",
            "domain_acceptance",
        ]
        .map(str::to_owned);
        let actual: BTreeSet<_> = g.accepted_checks.iter().cloned().collect();
        let expected: BTreeSet<_> = required.into_iter().collect();
        if g.schema_version != "flow.owner-native-release-gate/v1"
            || g.evidence_kind != "real_native_release_gate"
            || g.row_subject_sha256 != entry.report.row_subject_sha256
            || g.artifact_sha256 != entry.selection.executable.sha256
            || g.target != row.target
            || g.host_identity_sha256 != policy.host_identity.sha256
            || g.completed_at_unix != q.completed_at_unix
            || g.plan_sha256 != witness.plan_sha256
            || g.acceptance_sha256 != witness.acceptance_sha256
            || g.accepted_checks.len() != 5
            || actual != expected
            || !g.failed_checks.is_empty()
            || !g.missing_checks.is_empty()
        {
            return Err(error(
                Code::Unavailable,
                "authenticated owner gate lacks exact positive native checks",
            ));
        }
    }
    Ok(())
}
fn entry(
    selection: Selection,
    report: Report,
    row: &Row,
    request: &Request,
    policy: &Policy,
    token: &dyn CancellationSignal,
) -> Result<Entry, Error> {
    let subject =
        cohort::subject(row).map_err(|_| error(Code::InvalidInput, "row subject unavailable"))?;
    let artifact = row
        .artifact
        .as_ref()
        .ok_or_else(|| error(Code::Unavailable, "exact release artifact required"))?;
    let publication = &report.publication;
    if report.schema_version != "flow.authenticated-release-report/v1"
        || report.cohort_sha256 != request.cohort.sha256
        || report.row_subject_sha256 != subject
        || publication.release_url != artifact.release_url
        || !publication.release_url.starts_with(&format!(
            "https://github.com/{}/releases/download/",
            repo(row.tool)
        ))
        || publication.release_id == 0
        || publication.asset_id == 0
        || publication.executable_member_sha256 != artifact.sha256
        || publication.executable_member_bytes != artifact.byte_count
        || selection.executable.sha256 != artifact.sha256
        || selection.executable.byte_count != artifact.byte_count
        || selection.contracts.len() != row.contracts.len()
        || row.contracts.iter().any(|c| {
            selection
                .contracts
                .iter()
                .filter(|f| f.identifier == c.identifier && f.file.sha256 == c.sha256)
                .count()
                != 1
        })
    {
        return Err(error(
            Code::InvalidInput,
            "authenticated publication/artifact/contracts do not bind exact row",
        ));
    }
    let install: CleanInstall = read(&report.clean_install, 1_048_576, token)?;
    let qualification: Qualification = read(&report.qualification, 1_048_576, token)?;
    if install.schema_version != "flow.positive-clean-install/v1"
        || install.row_subject_sha256 != subject
        || install.installed_executable_sha256 != artifact.sha256
        || install.contracts != row.contracts
        || !install.empty_destination_observed
        || install.positive_native_plan_sha256 != report.witness.plan_sha256
        || install.positive_acceptance_sha256 != report.witness.acceptance_sha256
        || qualification.schema_version != "flow.positive-native-qualification/v1"
        || qualification.evidence_kind != "real_native_positive_acceptance"
        || qualification.row_subject_sha256 != subject
        || qualification.target != row.target
        || qualification.host_identity_sha256 != policy.host_identity.sha256
        || qualification.native_plan_sha256 != report.witness.plan_sha256
        || qualification.acceptance_sha256 != report.witness.acceptance_sha256
        || !qualification.missing_checks.is_empty()
        || !qualification.failed_checks.is_empty()
        || !cohort::is_digest(&report.witness.plan_sha256)
        || !cohort::is_digest(&report.witness.acceptance_sha256)
        || report.requirements.len() != row.requirements.len()
        || report.requirements.len() > 256
        || row.requirements.iter().any(|r| {
            report
                .requirements
                .iter()
                .filter(|p| p.requirement_id == r.id)
                .count()
                != 1
        })
    {
        return Err(error(
            Code::Unavailable,
            "positive clean-install/native/domain qualification is incomplete or substituted",
        ));
    }
    let e = Entry {
        selection,
        report,
        qualification,
    };
    current(&e, row, policy, token)?;
    Ok(e)
}

/// Effectful only when explicitly invoked: pinned `gh attestation verify` may
/// retrieve Sigstore trust roots. It never downloads artifacts or invokes media.
/// Caller policy is trusted host configuration, not publisher-supplied authority.
pub fn verify(
    request: &Request,
    trust: &TrustedConfiguration,
    output: &Path,
    allow_verification_network: bool,
    token: &dyn CancellationSignal,
) -> Result<VerifiedEvidence, Error> {
    request.validate()?;
    if !allow_verification_network {
        return Err(error(
            Code::ApprovalRequired,
            "explicit verification-network acknowledgement required",
        ));
    }
    let started = Instant::now();
    let guard =
        || token.is_cancelled() || started.elapsed() >= Duration::from_millis(request.timeout_ms);
    let selected: Cohort = read(&request.cohort, 1_048_576, &guard)?;
    selected
        .validate()
        .map_err(|_| error(Code::InvalidInput, "closed selected cohort required"))?;
    let policy: Policy = read(&trust.policy, 1_048_576, &guard)?;
    policy.validate()?;
    policy.host_identity.current(65_536, &guard)?;
    trust.verifier.current(536_870_912, &guard)?;
    let mut reports = vec![];
    for s in &request.selections {
        let row = selected
            .rows
            .iter()
            .find(|r| r.id == s.row_id)
            .ok_or_else(|| error(Code::InvalidInput, "unknown evidence row"))?;
        let report: Report = read(&s.report, 1_048_576, &guard)?;
        s.sigstore_bundle.current(MAX, &guard)?;
        let owner = policy
            .owners
            .iter()
            .find(|o| o.tool == row.tool)
            .ok_or_else(|| error(Code::Unavailable, "no explicitly trusted owner for row"))?;
        if row
            .artifact
            .as_ref()
            .is_none_or(|a| a.source_revision != owner.source_revision)
        {
            return Err(error(
                Code::InvalidInput,
                "source policy differs from selected release revision",
            ));
        }
        reports.push((s.clone(), report, row.clone(), owner.clone()));
    }
    let output = command::absolute(output)?;
    for p in
        request
            .pins()
            .into_iter()
            .chain([&trust.policy, &trust.verifier, &policy.host_identity])
    {
        let source = native::path(&p.path)?;
        if source.starts_with(&output) || output.starts_with(&source) {
            return Err(error(
                Code::InvalidInput,
                "verification output overlaps selected source",
            ));
        }
    }
    // Validate all retained semantics before creating audit state or launching a verifier.
    let mut entries = vec![];
    for (s, r, row, _) in &reports {
        let e = entry(s.clone(), r.clone(), row, request, &policy, &guard)?;
        for p in pins_for(&e, &policy) {
            let source = native::path(&p.path)?;
            if source.starts_with(&output) || output.starts_with(&source) {
                return Err(error(
                    Code::InvalidInput,
                    "verification output overlaps retained proof",
                ));
            }
        }
        if let Some(w) = &r.witness.run_directory {
            let witness = native::path(w)?;
            if output.starts_with(&witness) || witness.starts_with(&output) {
                return Err(error(
                    Code::InvalidInput,
                    "verification audit must be outside native witness",
                ));
            }
        }
        entries.push(e);
    }
    let parent = output
        .parent()
        .ok_or_else(|| error(Code::InvalidInput, "verification output parent required"))?;
    if parent.canonicalize().ok().as_deref() != Some(parent)
        || !fs::metadata(parent).is_ok_and(|m| m.is_dir())
    {
        return Err(error(
            Code::InvalidInput,
            "existing canonical verification parent required",
        ));
    }
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(&output).map_err(|_| {
        error(
            Code::RecoveryRequired,
            "fresh verification directory required",
        )
    })?;
    let action = (|| {
        command::write_document(&output.join("request.json"), request)?;
        command::write_document(&output.join("policy.json"), &policy)?;
        let mut transcripts = vec![];
        for (index, (s, report, row, owner)) in reports.iter().enumerate() {
            let publication = &report.publication;
            let captured = authenticate(
                trust,
                owner,
                &s.row_id,
                &s.report,
                &s.sigstore_bundle,
                policy.github_credential.as_ref(),
                publication.release_id,
                &output,
                index,
                started,
                request.timeout_ms,
                &guard,
            )?;
            let metadata = captured.document;
            transcripts.extend(captured.receipts);
            let artifact = row
                .artifact
                .as_ref()
                .ok_or_else(|| error(Code::Unavailable, "published artifact pin absent"))?;
            let tag = format!("v{}", artifact.version);
            let archive_digest = format!("sha256:{}", publication.published_archive.sha256);
            if metadata["tag_name"] != tag
                || !metadata["assets"].as_array().is_some_and(|assets| {
                    assets.iter().any(|a| {
                        a["id"].as_u64() == Some(publication.asset_id)
                            && a["state"] == "uploaded"
                            && a["browser_download_url"] == publication.release_url
                            && a["digest"] == archive_digest
                            && a["size"].as_u64() == Some(publication.published_archive.byte_count)
                    })
                })
            {
                return Err(error(
                    Code::Unavailable,
                    "exact published release asset or GitHub content digest is absent",
                ));
            }
        }
        // Effect authority binds the installed/runtime scope, not a positive
        // proof's timestamp or capture directory. A freshly authenticated proof
        // for identical scope may renew readiness without changing plan authority.
        let mut scope = entries
            .iter()
            .map(|e| {
                let mut contracts = e.selection.contracts.clone();
                contracts.sort_by(|a, b| a.identifier.cmp(&b.identifier));
                let mut requirements = e.report.requirements.clone();
                requirements.sort_by(|a, b| a.requirement_id.cmp(&b.requirement_id));
                (
                    e.selection.row_id.clone(),
                    e.report.row_subject_sha256.clone(),
                    e.selection.executable.clone(),
                    contracts,
                    requirements,
                )
            })
            .collect::<Vec<_>>();
        scope.sort_by(|a, b| a.0.cmp(&b.0));
        let binding_sha256 = hash(&(trust, &policy, &request.cohort.sha256, scope))?;
        let audit_path = output.join("verification.json");
        command::write_document(
            &audit_path,
            &serde_json::json!({"schema_version":"flow.authenticated-evidence-audit/v1","binding_sha256":binding_sha256,"request_sha256":hash(request)?,"cohort_sha256":request.cohort.sha256,"transcripts":transcripts,"verification_network_permitted":true,"media_or_provider_invoked":false}),
        )?;
        let audit = FilePin::observe(&NativePath::from_path(&audit_path), MAX, &guard)?;
        let verified = VerifiedEvidence {
            request: request.clone(),
            trust: trust.clone(),
            policy,
            cohort: selected,
            entries,
            binding_sha256,
            audit,
        };
        verified.current(&guard)?;
        Ok(verified)
    })();
    action.map_err(|mut e: Error| {
        e.recovery_directory = Some(NativePath::from_path(&output));
        e
    })
}
impl VerifiedEvidence {
    pub fn summary(&self) -> Verification {
        Verification {
            schema_version: "flow.verified-release-evidence/v1".into(),
            binding_sha256: self.binding_sha256.clone(),
            cohort_sha256: self.request.cohort.sha256.clone(),
            verified_rows: self
                .entries
                .iter()
                .map(|e| e.selection.row_id.clone())
                .collect(),
            audit: self.audit.clone(),
            verifier_invoked: true,
            verification_network_permitted: true,
        }
    }
    fn current_configuration(&self, token: &dyn CancellationSignal) -> Result<(), Error> {
        self.request.cohort.current(1_048_576, token)?;
        self.trust.policy.current(1_048_576, token)?;
        self.trust.verifier.current(536_870_912, token)?;
        self.audit.current(MAX, token)
    }
    fn current(&self, token: &dyn CancellationSignal) -> Result<(), Error> {
        self.current_configuration(token)?;
        for e in &self.entries {
            let row = self
                .cohort
                .rows
                .iter()
                .find(|r| r.id == e.selection.row_id)
                .ok_or_else(|| error(Code::InvalidInput, "verified row absent"))?;
            current(e, row, &self.policy, token)?;
        }
        Ok(())
    }
    pub fn matrix(&self, token: &dyn CancellationSignal) -> Result<cohort::Matrix, Error> {
        self.current_configuration(token)?;
        cohort::project(&self.cohort, self, token).map_err(|_| {
            error(
                Code::Unavailable,
                "authenticated matrix projection unavailable",
            )
        })
    }
    pub fn admit(
        &self,
        plan: &native::Plan,
        token: &dyn CancellationSignal,
    ) -> Result<Admission, Error> {
        plan.require_current(token)?;
        self.current_configuration(token)?;
        let row = &plan.row;
        if plan.request.cohort_sha256 != self.request.cohort.sha256
            || Some(row.target.as_str()) != native::host_target()
            || row.implementation != cohort::Implementation::Implemented
            || row.artifact.is_none()
            || row.contracts.is_empty()
            || !self.cohort.rows.iter().any(|r| r == row)
        {
            return Err(error(
                Code::Unavailable,
                "exact native row lacks current authenticated release admission",
            ));
        }
        let e = self
            .entries
            .iter()
            .find(|e| e.selection.row_id == row.id)
            .ok_or_else(|| {
                error(
                    Code::Unavailable,
                    "no authenticated positive report selected for this native row",
                )
            })?;
        // Construction authenticated every required dimension. Reobserve its exact
        // current bindings without letting unrelated stale rows block this row.
        current(e, row, &self.policy, token)?;
        Ok(Admission {
            schema_version: "flow.verified-release-admission/v1".into(),
            binding_sha256: self.binding_sha256.clone(),
            row_subject_sha256: plan.row_subject_sha256.clone(),
            policy_sha256: self.trust.policy.sha256.clone(),
        })
    }
}
impl EvidencePort for VerifiedEvidence {
    fn observe(
        &self,
        row: &Row,
        dimension: Dimension,
        token: &dyn CancellationSignal,
    ) -> Result<Observation, cohort::Error> {
        if token.is_cancelled() {
            return Err(cohort::Error::Cancelled);
        }
        let Some(e) = self.entries.iter().find(|e| e.selection.row_id == row.id) else {
            return Ok(Observation::unobserved(
                row,
                "no authenticated positive report selected",
            ));
        };
        if !self.cohort.rows.iter().any(|r| r == row) {
            return Err(cohort::Error::Invalid("authenticated row substitution"));
        }
        let configuration = self.current_configuration(token);
        if configuration.is_err() || current(e, row, &self.policy, token).is_err() {
            return Ok(Observation{subject_sha256:cohort::subject(row)?,state:State::Failed,evidence_sha256:Some(e.selection.report.sha256.clone()),reason:"authenticated evidence expired or current host, native witness, installed files or dependencies changed".into()});
        }
        if matches!(dimension, Dimension::Installation | Dimension::Contracts) {
            let files = LocalFiles {
                installations: vec![InstalledFiles {
                    row_id: row.id.clone(),
                    executable: native::path(&e.selection.executable.path)
                        .map_err(|_| cohort::Error::Invalid("installation path"))?,
                    contract_files: e
                        .selection
                        .contracts
                        .iter()
                        .map(|c| native::path(&c.file.path).map(|p| (c.identifier.clone(), p)))
                        .collect::<Result<_, _>>()
                        .map_err(|_| cohort::Error::Invalid("contract path"))?,
                }],
            };
            let observed = files.observe(row, dimension, token)?;
            if observed.state != State::Passed {
                return Ok(observed);
            }
        }
        Ok(Observation{subject_sha256:cohort::subject(row)?,state:State::Passed,evidence_sha256:Some(e.selection.report.sha256.clone()),reason:"exact owner-authenticated report, positive clean-install/native acceptance and current explicit host/dependency/payload bindings".into()})
    }
}

struct AuthenticatedMetadata {
    document: serde_json::Value,
    receipts: Vec<FilePin>,
}
fn authenticate(
    trust: &TrustedConfiguration,
    owner: &Owner,
    row_id: &str,
    report: &FilePin,
    bundle: &FilePin,
    credential: Option<&FilePin>,
    release_id: u64,
    output: &Path,
    index: usize,
    started: Instant,
    timeout_ms: u64,
    token: &dyn CancellationSignal,
) -> Result<AuthenticatedMetadata, Error> {
    let mut transcripts = vec![];
    for p in [&trust.verifier, report, bundle] {
        p.current(536_870_912, token)?;
    }
    let args: Vec<OsString> = vec![
        "attestation".into(),
        "verify".into(),
        native::path(&report.path)?.into_os_string(),
        "--bundle".into(),
        native::path(&bundle.path)?.into_os_string(),
        "--repo".into(),
        owner.repository.clone().into(),
        "--signer-workflow".into(),
        owner.signer_workflow.clone().into(),
        "--signer-digest".into(),
        owner.signer_revision.clone().into(),
        "--source-digest".into(),
        owner.source_revision.clone().into(),
        "--source-ref".into(),
        owner.source_ref.clone().into(),
        "--deny-self-hosted-runners".into(),
        "--format".into(),
        "json".into(),
    ];
    // Empty private HOME contains no inherited credentials or configuration.
    let home = output.join(format!("verifier-home-{index:02}"));
    let mut b = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        b.mode(0o700);
    }
    b.create(&home)
        .map_err(|_| error(Code::Io, "private verifier home unavailable"))?;
    let mut environment = vec![(OsString::from("HOME"), home.into_os_string())];
    if let Some(pin) = credential {
        pin.current(4096, token)?;
        let p = native::path(&pin.path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if fs::metadata(&p)
                .map_err(|_| error(Code::Unavailable, "credential metadata unavailable"))?
                .permissions()
                .mode()
                & 0o077
                != 0
            {
                return Err(error(
                    Code::InvalidInput,
                    "credential must exclude group/other access",
                ));
            }
        }
        let bytes = command::read_document(&p, 4096)?;
        if cohort::sha256(&bytes) != pin.sha256 {
            return Err(error(Code::StalePlan, "credential pin changed"));
        }
        let secret = std::str::from_utf8(&bytes)
            .map_err(|_| error(Code::InvalidInput, "credential encoding"))?
            .trim();
        if secret.is_empty()
            || secret.chars().any(char::is_whitespace)
            || secret.chars().any(char::is_control)
        {
            return Err(error(
                Code::InvalidInput,
                "single GitHub credential required",
            ));
        }
        environment.push((OsString::from("GH_TOKEN"), OsString::from(secret)));
    }
    let captured = crate::runner::capture_native(
        &native::path(&trust.verifier.path)?,
        &args,
        &environment,
        remaining(started, timeout_ms)?,
        MAX,
        65_536,
        token,
    )
    .map_err(|_| {
        error(
            Code::Unavailable,
            "attestation verifier cancelled, unavailable or exceeded bounds",
        )
    })?;
    let document = serde_json::json!({"schema_version":"flow.attestation-verifier-capture/v1","row_id":row_id,"report_sha256":report.sha256,"verifier_sha256":trust.verifier.sha256,"exit_code":captured.code,"stdout":String::from_utf8_lossy(&captured.stdout),"stderr":String::from_utf8_lossy(&captured.stderr)});
    let name = output.join(format!("verification-{index:02}.json"));
    command::write_document(&name, &document)?;
    transcripts.push(FilePin::observe(&NativePath::from_path(&name), MAX, token)?);
    // The pinned trusted verifier authenticates the exact file subject.
    // Native/domain admission remains a separate caller-owned proof boundary.
    let response: serde_json::Value = serde_json::from_slice(&captured.stdout).map_err(|_| {
        error(
            Code::Unavailable,
            "attestation verifier returned invalid JSON",
        )
    })?;
    if captured.code != Some(0)
        || !response.as_array().is_some_and(|v| {
            !v.is_empty() && v.len() <= 32 && v.iter().all(serde_json::Value::is_object)
        })
    {
        return Err(error(
            Code::Unavailable,
            "attestation did not authenticate this exact report",
        ));
    }

    let api_args: Vec<OsString> = vec![
        "api".into(),
        "--hostname".into(),
        "github.com".into(),
        format!("/repos/{}/releases/{}", owner.repository, release_id).into(),
    ];
    let published = crate::runner::capture_native(
        &native::path(&trust.verifier.path)?,
        &api_args,
        &environment,
        remaining(started, timeout_ms)?,
        1_048_576,
        32_768,
        token,
    )
    .map_err(|_| {
        error(
            Code::Unavailable,
            "public release metadata unavailable within bounds",
        )
    })?;
    let metadata: serde_json::Value = serde_json::from_slice(&published.stdout).map_err(|_| {
        error(
            Code::Unavailable,
            "public release metadata JSON unavailable",
        )
    })?;
    if published.code != Some(0)
        || metadata["id"].as_u64() != Some(release_id)
        || metadata["draft"] != false
    {
        return Err(error(
            Code::Unavailable,
            "actual non-draft release metadata required",
        ));
    }
    let public_path = output.join(format!("publication-{index:02}.json"));
    command::write_document(&public_path, &metadata)?;
    transcripts.push(FilePin::observe(
        &NativePath::from_path(&public_path),
        MAX,
        token,
    )?);
    for p in [&trust.verifier, report, bundle] {
        p.current(536_870_912, token)?;
    }
    Ok(AuthenticatedMetadata {
        document: metadata,
        receipts: transcripts,
    })
}

pub mod publisher;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod publisher_tests;
