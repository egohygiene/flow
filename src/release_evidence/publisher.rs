//! Publisher authentication before installation. This opaque authority permits
//! only exact file copies; it does not implement native release admission.
use super::*;
use crate::{
    distribution,
    publisher_report::{self, Report as PublisherReport},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublisherPolicy {
    pub schema_version: String,
    pub owners: Vec<Owner>,
    pub maximum_age_seconds: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_credential: Option<FilePin>,
}
impl PublisherPolicy {
    fn validate(&self) -> Result<(), Error> {
        if self.schema_version != "flow.publisher-policy/v1"
            || self.owners.is_empty()
            || self.owners.len() > 4
            || self.maximum_age_seconds == 0
            || self.maximum_age_seconds > 3600
        {
            return Err(error(
                Code::InvalidInput,
                "bounded independent publisher policy required",
            ));
        }
        let mut owners = BTreeSet::new();
        for owner in &self.owners {
            owner.validate()?;
            if !owners.insert(owner.repository.clone()) {
                return Err(error(Code::InvalidInput, "duplicate publisher owner"));
            }
        }
        if let Some(pin) = &self.github_credential {
            native::path(&pin.path)?;
            if !cohort::is_digest(&pin.sha256) || pin.byte_count == 0 || pin.byte_count > 4096 {
                return Err(error(
                    Code::InvalidInput,
                    "bounded explicit publisher credential required",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub row_id: String,
    pub report: FilePin,
    pub sigstore_bundle: FilePin,
    pub archive: FilePin,
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
            return Err(error(Code::InvalidInput, "publisher request exceeds 1 MiB"));
        }
        let v: Self = serde_json::from_slice(bytes)
            .map_err(|_| error(Code::InvalidInput, "closed publisher request required"))?;
        v.validate()?;
        Ok(v)
    }
    fn pins(&self) -> Vec<&FilePin> {
        let mut p = vec![&self.cohort];
        for s in &self.selections {
            p.extend([&s.report, &s.sigstore_bundle, &s.archive]);
        }
        p
    }
    fn validate(&self) -> Result<(), Error> {
        if self.schema_version != "flow.publisher-verification-request/v1"
            || self.selections.is_empty()
            || self.selections.len() > 32
            || self.timeout_ms == 0
            || self.timeout_ms > 600_000
        {
            return Err(error(
                Code::InvalidInput,
                "publisher request identity/budget",
            ));
        }
        let mut rows = BTreeSet::new();
        for s in &self.selections {
            if s.row_id.is_empty() || s.row_id.len() > 4096 || !rows.insert(&s.row_id) {
                return Err(error(Code::InvalidInput, "unique publisher rows required"));
            }
        }
        for p in self.pins() {
            native::path(&p.path)?;
            if !cohort::is_digest(&p.sha256) || p.byte_count == 0 || p.byte_count > 536_870_912 {
                return Err(error(
                    Code::InvalidInput,
                    "bounded exact publisher input pins required",
                ));
            }
        }
        Ok(())
    }
}
struct Entry {
    selection: Selection,
    report: PublisherReport,
    row: Row,
}
/// Created only by live verification against independent host trust. There is no
/// JSON deserializer and this type cannot grant native execution permission.
pub struct VerifiedPublisher {
    request: Request,
    trust: TrustedConfiguration,
    policy: PublisherPolicy,
    cohort: Cohort,
    entries: Vec<Entry>,
    captures: Vec<FilePin>,
    audit: FilePin,
    binding_sha256: String,
    started: Instant,
}
#[derive(Debug, Clone, Serialize)]
pub struct Verification {
    pub schema_version: String,
    pub binding_sha256: String,
    pub cohort_sha256: String,
    pub verified_rows: Vec<String>,
    pub audit: FilePin,
    pub publisher_authenticated: bool,
    pub native_qualified: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorizedRow {
    pub row_id: String,
    pub row_subject_sha256: String,
    pub report_sha256: String,
    pub archive_sha256: String,
}
/// A retained observation, never an input constructor for VerifiedPublisher.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallAdmission {
    pub schema_version: String,
    pub install_request_sha256: String,
    pub cohort_sha256: String,
    pub policy_sha256: String,
    pub verifier_sha256: String,
    pub verification_binding_sha256: String,
    pub audit: FilePin,
    pub rows: Vec<AuthorizedRow>,
}
pub(crate) fn install_subject(request: &distribution::Request) -> Result<String, Error> {
    hash(request)
}

fn metadata_matches(report: &PublisherReport, metadata: &serde_json::Value) -> bool {
    if metadata["id"].as_u64() != Some(report.publication.release_id)
        || metadata["draft"] != false
        || metadata["tag_name"] != format!("v{}", report.version)
    {
        return false;
    }
    let Some(assets) = metadata["assets"].as_array() else {
        return false;
    };
    let present = |id: u64, url: &str, sha: &str, bytes: u64, name: Option<&str>| {
        assets
            .iter()
            .filter(|a| {
                a["id"].as_u64() == Some(id)
                    && a["state"] == "uploaded"
                    && a["browser_download_url"] == url
                    && a["digest"] == format!("sha256:{sha}")
                    && a["size"].as_u64() == Some(bytes)
                    && name.is_none_or(|name| a["name"] == name)
            })
            .count()
            == 1
    };
    if !present(
        report.publication.asset_id,
        &report.publication.release_url,
        &report.archive.sha256,
        report.archive.byte_count,
        Some(&report.archive.name),
    ) {
        return false;
    }
    report.contracts.iter().all(|c| {
        c.publication
            .as_ref()
            .is_none_or(|p| present(p.asset_id, &p.release_url, &c.sha256, c.byte_count, None))
    })
}
/// Explicit signature/publication verification. No installed binary, local
/// native witness, model, runtime probe or provider invocation is required.
pub fn verify(
    request: &Request,
    trust: &TrustedConfiguration,
    output: &Path,
    allow_network: bool,
    token: &dyn CancellationSignal,
) -> Result<VerifiedPublisher, Error> {
    request.validate()?;
    if !allow_network {
        return Err(error(
            Code::ApprovalRequired,
            "explicit publisher verification networking acknowledgement required",
        ));
    }
    let started = Instant::now();
    let guard =
        || token.is_cancelled() || started.elapsed() >= Duration::from_millis(request.timeout_ms);
    let selected: Cohort = read(&request.cohort, 1_048_576, &guard)?;
    selected
        .validate()
        .map_err(|_| error(Code::InvalidInput, "publisher cohort invalid"))?;
    let policy: PublisherPolicy = read(&trust.policy, 1_048_576, &guard)?;
    policy.validate()?;
    trust.verifier.current(536_870_912, &guard)?;
    let output = command::absolute(output)?;
    let mut entries = vec![];
    for s in &request.selections {
        let row = selected
            .rows
            .iter()
            .find(|r| r.id == s.row_id)
            .ok_or_else(|| error(Code::InvalidInput, "unknown publisher row"))?
            .clone();
        let report: PublisherReport = read(&s.report, 1_048_576, &guard)?;
        report.validate_for(&row, &request.cohort.sha256)?;
        s.sigstore_bundle.current(MAX, &guard)?;
        s.archive.current(536_870_912, &guard)?;
        if s.archive.sha256 != report.archive.sha256
            || s.archive.byte_count != report.archive.byte_count
        {
            return Err(error(
                Code::InvalidInput,
                "selected archive differs from authenticated report subject",
            ));
        }
        let owner = policy
            .owners
            .iter()
            .find(|o| o.tool == row.tool)
            .ok_or_else(|| {
                error(
                    Code::Unavailable,
                    "no independent trusted publisher owner selected",
                )
            })?;
        if owner.source_revision != report.source_revision {
            return Err(error(Code::InvalidInput, "publisher source policy differs"));
        }
        entries.push(Entry {
            selection: s.clone(),
            report,
            row,
        });
    }
    for p in request
        .pins()
        .into_iter()
        .chain([&trust.policy, &trust.verifier])
        .chain(policy.github_credential.iter())
    {
        let source = native::path(&p.path)?;
        if source.starts_with(&output) || output.starts_with(&source) {
            return Err(error(
                Code::InvalidInput,
                "publisher audit overlaps selected source",
            ));
        }
    }
    let parent = output
        .parent()
        .ok_or_else(|| error(Code::InvalidInput, "publisher audit parent required"))?;
    if parent.canonicalize().ok().as_deref() != Some(parent)
        || !fs::metadata(parent).is_ok_and(|m| m.is_dir())
    {
        return Err(error(
            Code::InvalidInput,
            "existing canonical publisher audit parent required",
        ));
    }
    let mut b = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        b.mode(0o700);
    }
    b.create(&output).map_err(|_| {
        error(
            Code::RecoveryRequired,
            "fresh publisher audit directory required",
        )
    })?;
    let action = (|| {
        command::write_document(&output.join("request.json"), request)?;
        command::write_document(&output.join("policy.json"), &policy)?;
        let mut captures = vec![];
        for (index, e) in entries.iter().enumerate() {
            let owner = policy
                .owners
                .iter()
                .find(|o| o.tool == e.row.tool)
                .ok_or_else(|| error(Code::Unavailable, "trusted publisher owner missing"))?;
            let capture = authenticate(
                trust,
                owner,
                &e.row.id,
                &e.selection.report,
                &e.selection.sigstore_bundle,
                policy.github_credential.as_ref(),
                e.report.publication.release_id,
                &output,
                index,
                started,
                request.timeout_ms,
                &guard,
            )?;
            if !metadata_matches(&e.report, &capture.document) {
                return Err(error(
                    Code::Unavailable,
                    "actual published asset/contract metadata differs",
                ));
            }
            captures.extend(capture.receipts);
        }
        let binding_sha256 = hash(&(request, trust, &policy))?;
        let file = output.join("publisher-verification.json");
        command::write_document(
            &file,
            &serde_json::json!({"schema_version":"flow.publisher-verification-audit/v1","binding_sha256":binding_sha256,"cohort_sha256":request.cohort.sha256,"captures":captures,"native_qualified":false,"provider_invoked":false}),
        )?;
        let audit = FilePin::observe(&NativePath::from_path(&file), MAX, &guard)?;
        let verified = VerifiedPublisher {
            request: request.clone(),
            trust: trust.clone(),
            policy,
            cohort: selected,
            entries,
            captures,
            audit,
            binding_sha256,
            started,
        };
        verified.current(&guard)?;
        Ok(verified)
    })();
    action.map_err(|mut e: Error| {
        e.recovery_directory = Some(NativePath::from_path(&output));
        e
    })
}
impl VerifiedPublisher {
    pub fn summary(&self) -> Verification {
        Verification {
            schema_version: "flow.verified-publisher/v1".into(),
            binding_sha256: self.binding_sha256.clone(),
            cohort_sha256: self.request.cohort.sha256.clone(),
            verified_rows: self.entries.iter().map(|e| e.row.id.clone()).collect(),
            audit: self.audit.clone(),
            publisher_authenticated: true,
            native_qualified: false,
        }
    }
    pub(crate) fn current(&self, token: &dyn CancellationSignal) -> Result<(), Error> {
        if self.started.elapsed() >= Duration::from_secs(self.policy.maximum_age_seconds) {
            return Err(error(
                Code::Unavailable,
                "publisher observation expired; explicitly verify again",
            ));
        }
        for p in self
            .request
            .pins()
            .into_iter()
            .chain([&self.trust.policy, &self.trust.verifier, &self.audit])
            .chain(self.policy.github_credential.iter())
            .chain(self.captures.iter())
        {
            p.current(536_870_912, token)?;
        }
        Ok(())
    }
    pub fn admit_install(
        &self,
        request: &distribution::Request,
        token: &dyn CancellationSignal,
    ) -> Result<InstallAdmission, Error> {
        self.current(token)?;
        if request.cohort.sha256 != self.request.cohort.sha256
            || request.cohort.byte_count != self.request.cohort.byte_count
            || request.tools.is_empty()
            || request.tools.len() > 4
        {
            return Err(error(
                Code::InvalidInput,
                "installation selects another cohort",
            ));
        }
        let root = native::path(&request.install_directory)?;
        for pin in self
            .request
            .pins()
            .into_iter()
            .chain([&self.trust.policy, &self.trust.verifier, &self.audit])
            .chain(self.policy.github_credential.iter())
        {
            let source = native::path(&pin.path)?;
            if source.starts_with(&root) || root.starts_with(&source) {
                return Err(error(
                    Code::InvalidInput,
                    "installation destination overlaps publisher evidence",
                ));
            }
        }
        if self
            .audit
            .path
            .path()
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .is_some_and(|p| root.starts_with(&p) || p.starts_with(&root))
        {
            return Err(error(
                Code::InvalidInput,
                "installation overlaps publisher audit tree",
            ));
        }
        let mut rows = vec![];
        let mut tools = BTreeSet::new();
        for tool in &request.tools {
            let entry = self
                .entries
                .iter()
                .find(|e| e.row.id == tool.row_id)
                .ok_or_else(|| {
                    error(
                        Code::Unavailable,
                        "installation row lacks publisher authentication",
                    )
                })?;
            let artifact = entry
                .row
                .artifact
                .as_ref()
                .ok_or_else(|| error(Code::InvalidInput, "publisher artifact missing"))?;
            if !self.cohort.rows.iter().any(|r| r == &entry.row)
                || !tools.insert(publisher_report::tool_name(tool.tool))
                || entry.row.tool != tool.tool
                || entry.row.target != request.target
                || entry.row.interface == cohort::Interface::RustLibrary
                || tool.executable.sha256 != artifact.sha256
                || tool.executable.byte_count != artifact.byte_count
                || tool.contracts.len() != entry.report.contracts.len()
                || entry.report.contracts.iter().any(|c| {
                    tool.contracts
                        .iter()
                        .filter(|f| {
                            f.identifier == c.identifier
                                && f.file.sha256 == c.sha256
                                && f.file.byte_count == c.byte_count
                        })
                        .count()
                        != 1
                })
                || tool.extras.iter().any(|extra| {
                    !entry.report.members.iter().any(|m| {
                        m.sha256 == extra.file.sha256 && m.byte_count == extra.file.byte_count
                    })
                })
            {
                return Err(error(
                    Code::InvalidInput,
                    "installed executable/contracts/support bytes differ from authenticated owner report",
                ));
            }
            rows.push(AuthorizedRow {
                row_id: entry.row.id.clone(),
                row_subject_sha256: entry.report.row_subject_sha256.clone(),
                report_sha256: entry.selection.report.sha256.clone(),
                archive_sha256: entry.report.archive.sha256.clone(),
            });
        }
        Ok(InstallAdmission {
            schema_version: "flow.publisher-install-admission/v1".into(),
            install_request_sha256: install_subject(request)?,
            cohort_sha256: self.request.cohort.sha256.clone(),
            policy_sha256: self.trust.policy.sha256.clone(),
            verifier_sha256: self.trust.verifier.sha256.clone(),
            verification_binding_sha256: self.binding_sha256.clone(),
            audit: self.audit.clone(),
            rows,
        })
    }
}
pub(crate) fn validate_retained(
    request: &distribution::Request,
    admission: &InstallAdmission,
    cohort: &Cohort,
) -> Result<(), Error> {
    if admission.schema_version != "flow.publisher-install-admission/v1"
        || admission.install_request_sha256 != install_subject(request)?
        || admission.cohort_sha256 != request.cohort.sha256
        || admission.rows.len() != request.tools.len()
        || ![
            &admission.policy_sha256,
            &admission.verifier_sha256,
            &admission.verification_binding_sha256,
            &admission.audit.sha256,
        ]
        .into_iter()
        .all(|s| cohort::is_digest(s))
    {
        return Err(error(
            Code::RecoveryRequired,
            "retained publisher install binding differs",
        ));
    }
    let mut seen = BTreeSet::new();
    for (authorized, tool) in admission.rows.iter().zip(&request.tools) {
        let row = cohort
            .rows
            .iter()
            .find(|r| r.id == tool.row_id)
            .ok_or_else(|| error(Code::RecoveryRequired, "retained installation row missing"))?;
        if !seen.insert(&authorized.row_id)
            || authorized.row_id != tool.row_id
            || cohort::subject(row)
                .map_err(|_| error(Code::InvalidInput, "retained row subject unavailable"))?
                != authorized.row_subject_sha256
            || !cohort::is_digest(&authorized.report_sha256)
            || !cohort::is_digest(&authorized.archive_sha256)
        {
            return Err(error(
                Code::RecoveryRequired,
                "retained publisher row evidence differs",
            ));
        }
    }
    Ok(())
}
