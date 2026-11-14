//! Released authority wraps the existing native engine; it cannot be supplied by
//! deserializing a `verified` result or manufacturing a candidate acknowledgement.
use super::*;
use crate::release_evidence::{Admission, Verification, VerifiedEvidence};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasedApproval {
    pub schema_version: String,
    pub plan_sha256: String,
    pub approved_by: String,
    pub trusted_unconfined: bool,
    pub admission: Admission,
}
impl ReleasedApproval {
    fn validate(
        &self,
        plan: &Plan,
        evidence: &VerifiedEvidence,
        token: &dyn CancellationSignal,
    ) -> Result<(), Error> {
        if self.schema_version != "flow.released-native-approval/v1"
            || self.plan_sha256 != plan.plan_sha256
            || self.approved_by.is_empty()
            || self.approved_by.len() > 256
            || self.approved_by.chars().any(char::is_control)
            || !self.trusted_unconfined
            || self.admission != evidence.admit(plan, token)?
        {
            return Err(error(
                Code::ApprovalRequired,
                "exact released-native approval and current authenticated admission required",
            ));
        }
        Ok(())
    }
}
pub fn approve_released(
    plan: &Plan,
    by: String,
    trusted_unconfined: bool,
    evidence: &VerifiedEvidence,
    token: &dyn CancellationSignal,
) -> Result<ReleasedApproval, Error> {
    let a = ReleasedApproval {
        schema_version: "flow.released-native-approval/v1".into(),
        plan_sha256: plan.plan_sha256.clone(),
        approved_by: by,
        trusted_unconfined,
        admission: evidence.admit(plan, token)?,
    };
    a.validate(plan, evidence, token)?;
    Ok(a)
}
#[derive(Debug, Serialize)]
pub struct ReleasedReceipt {
    pub schema_version: String,
    pub native: Receipt,
    pub admission: Admission,
    pub verification: Verification,
    pub retained_release_record: FilePin,
    pub release_qualified: bool,
    pub creative_approval_inferred: bool,
    pub publication_authority: bool,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReleaseRecord {
    schema_version: String,
    native: Receipt,
    admission: Admission,
    verification_audit: FilePin,
}
fn retained_record(
    root: &Path,
    plan: &Plan,
    native: &Receipt,
    admission: &Admission,
    token: &dyn CancellationSignal,
) -> Result<Option<FilePin>, Error> {
    let file = root.join("released-acceptance.json");
    match fs::symlink_metadata(&file) {
        Ok(_) => {
            let saved: ReleaseRecord = read(&file)?;
            if saved.schema_version != "flow.released-native-acceptance/v1"
                || saved.native != *native
                || saved.admission != *admission
            {
                return Err(error(
                    Code::RecoveryRequired,
                    "retained released acceptance differs from current native and authenticated admission",
                ));
            }
            saved.verification_audit.current(MAX_DOCUMENT, token)?;
            let audit: Value = read(&path(&saved.verification_audit.path)?)?;
            if audit["binding_sha256"] != admission.binding_sha256
                || audit["cohort_sha256"] != plan.request.cohort_sha256
            {
                return Err(error(
                    Code::RecoveryRequired,
                    "retained release audit binding differs",
                ));
            }
            FilePin::observe(&NativePath::from_path(&file), MAX_DOCUMENT, token).map(Some)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(error(Code::Io, "retained release acceptance unavailable")),
    }
}
// Caller holds the native run lock. Opaque evidence is rechecked independently
// of any retained serialized data before this release record is committed.
fn receipt(
    plan: &Plan,
    native: Receipt,
    evidence: &VerifiedEvidence,
    token: &dyn CancellationSignal,
) -> Result<ReleasedReceipt, Error> {
    let root = path(&plan.request.run_directory)?;
    let now = current_status(&root, plan, token)?;
    if now.receipt.as_ref() != Some(&native) {
        return Err(error(
            Code::RecoveryRequired,
            "native acceptance changed before released receipt commit",
        ));
    }
    let admission = evidence.admit(plan, token)?;
    let verification = evidence.summary();
    if retained_record(&root, plan, &native, &admission, token)?.is_none() {
        write(
            &root.join("released-acceptance.json"),
            &ReleaseRecord {
                schema_version: "flow.released-native-acceptance/v1".into(),
                native: native.clone(),
                admission: admission.clone(),
                verification_audit: verification.audit.clone(),
            },
        )?;
    }
    let retained_release_record = retained_record(&root, plan, &native, &admission, token)?
        .ok_or_else(|| error(Code::RecoveryRequired, "release record commit is absent"))?;
    Ok(ReleasedReceipt {
        schema_version: "flow.released-native-receipt/v1".into(),
        native,
        admission,
        verification,
        retained_release_record,
        release_qualified: true,
        creative_approval_inferred: false,
        publication_authority: false,
    })
}
pub fn run_released(
    plan: &Plan,
    approval: &ReleasedApproval,
    evidence: &VerifiedEvidence,
    token: &dyn CancellationSignal,
) -> Result<ReleasedReceipt, Error> {
    plan.require_current(token)?;
    approval.validate(plan, evidence, token)?;
    let native = run_authorized(plan, approval, Some(evidence), token)?;
    let root = path(&plan.request.run_directory)?;
    let _held = lock(&root, false)?;
    receipt(plan, native, evidence, token).map_err(|e| context_error(e, &root))
}
fn basis_released(
    root: &Path,
    evidence: &VerifiedEvidence,
    token: &dyn CancellationSignal,
) -> Result<(Plan, ReleasedApproval), Error> {
    let p = Plan::from_json_slice(&command::read_document(
        &root.join("operation-plan.json"),
        MAX_DOCUMENT,
    )?)?;
    let a: ReleasedApproval = read(&root.join("operation-approval.json"))?;
    a.validate(&p, evidence, token)?;
    if path(&p.request.run_directory)? != root {
        return Err(error(
            Code::RecoveryRequired,
            "released operation moved from approved destination",
        ));
    }
    Ok((p, a))
}
#[derive(Debug, Serialize)]
pub struct ReleasedStatus {
    pub schema_version: String,
    pub native: Status,
    pub admission: Admission,
    pub verification: Verification,
    pub retained_release_record: Option<FilePin>,
}
pub fn status_released(
    root: &Path,
    evidence: &VerifiedEvidence,
    token: &dyn CancellationSignal,
) -> Result<ReleasedStatus, Error> {
    let _held = lock(root, false)?;
    let (p, _) = basis_released(root, evidence, token)?;
    let mut native = current_status(root, &p, token)?;
    let admission = evidence.admit(&p, token)?;
    let retained_release_record = if let Some(r) = &native.receipt {
        retained_record(root, &p, r, &admission, token)?
    } else {
        None
    };
    if retained_release_record.is_none() {
        native.state = State::RecoveryRequired;
        native.next_action="retain native acceptance; explicit resume-released with the accepted native run can finalize release bookkeeping without replay".into();
    }
    Ok(ReleasedStatus {
        schema_version: "flow.released-native-status/v1".into(),
        native,
        admission,
        verification: evidence.summary(),
        retained_release_record,
    })
}
pub fn resume_released(
    root: &Path,
    native_run: &Path,
    evidence: &VerifiedEvidence,
    token: &dyn CancellationSignal,
) -> Result<ReleasedReceipt, Error> {
    let _held = lock(root, false)?;
    let (p, _) = basis_released(root, evidence, token)?;
    let native = resume_authorized(root, native_run, &p, Some(evidence), token)?;
    receipt(&p, native, evidence, token).map_err(|e| context_error(e, root))
}
