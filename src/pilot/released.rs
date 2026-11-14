//! Released authority for the same fixed pilot. Native execution, acceptance and
//! recovery remain in the existing native SDK; no new process engine is added.
use super::*;
use crate::release_evidence::{self, VerifiedEvidence};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Approval {
    pub schema_version: String,
    pub plan_sha256: String,
    pub stages: Vec<native::ReleasedApproval>,
}
impl Approval {
    fn validate(
        &self,
        plan: &Plan,
        evidence: &VerifiedEvidence,
        token: &dyn CancellationSignal,
    ) -> Result<(), Error> {
        plan.require_current(token)?;
        if self.schema_version != "flow.released-pilot-approval/v1"
            || self.plan_sha256 != plan.plan_sha256
            || self.stages.len() != plan.stages.len()
        {
            return Err(error(
                Code::ApprovalRequired,
                "exact released pilot authority required",
            ));
        }
        for (stage, a) in plan.stages.iter().zip(&self.stages) {
            if native::approve_released(
                &stage.plan,
                a.approved_by.clone(),
                a.trusted_unconfined,
                evidence,
                token,
            )? != *a
            {
                return Err(error(
                    Code::ApprovalRequired,
                    "released pilot stage admission differs",
                ));
            }
        }
        Ok(())
    }
}
pub fn approve(
    plan: &Plan,
    by: String,
    trusted_unconfined: bool,
    evidence: &VerifiedEvidence,
    token: &dyn CancellationSignal,
) -> Result<Approval, Error> {
    plan.require_current(token)?;
    let stages = plan
        .stages
        .iter()
        .map(|s| native::approve_released(&s.plan, by.clone(), trusted_unconfined, evidence, token))
        .collect::<Result<_, _>>()?;
    let a = Approval {
        schema_version: "flow.released-pilot-approval/v1".into(),
        plan_sha256: plan.plan_sha256.clone(),
        stages,
    };
    a.validate(plan, evidence, token)?;
    Ok(a)
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletedStage {
    pub role: String,
    pub native_receipt: native::Receipt,
    pub admission: release_evidence::Admission,
    pub release_record: FilePin,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Handoff {
    pub schema_version: String,
    pub plan_sha256: String,
    pub metadata: Metadata,
    pub manual_srt: FilePin,
    pub audio_source: Option<FilePin>,
    pub stages: Vec<CompletedStage>,
    pub verification_binding_sha256: String,
    pub verification_audit: FilePin,
    pub provider_release_qualified: bool,
    pub creative_review_pending: bool,
    pub upload_authority: bool,
}
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StageState {
    NotStarted,
    RecoveryRequired,
    Complete,
}
#[derive(Debug, Clone, Serialize)]
pub struct StageStatus {
    pub role: String,
    pub state: StageState,
    pub completed: Option<CompletedStage>,
}
#[derive(Debug, Serialize)]
pub struct Status {
    pub schema_version: String,
    pub plan_sha256: String,
    pub stages: Vec<StageStatus>,
    pub handoff: Option<Handoff>,
    pub verification: release_evidence::Verification,
    pub next_action: String,
}
fn basis(
    root: &Path,
    evidence: &VerifiedEvidence,
    token: &dyn CancellationSignal,
) -> Result<(Plan, Approval), Error> {
    let p = Plan::from_json_slice(&command::read_document(&root.join("pilot-plan.json"), MAX)?)?;
    let a: Approval = read(&root.join("pilot-approval.json"))?;
    a.validate(&p, evidence, token)?;
    if native::path(&p.request.run_directory)? != root {
        return Err(error(
            Code::RecoveryRequired,
            "released pilot moved from approved destination",
        ));
    }
    Ok((p, a))
}
fn observe(
    stage: &Stage,
    evidence: &VerifiedEvidence,
    token: &dyn CancellationSignal,
) -> Result<StageStatus, Error> {
    cancelled(token)?;
    let root = native::path(&stage.plan.request.run_directory)?;
    match fs::symlink_metadata(&root) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(StageStatus {
            role: stage.role.clone(),
            state: StageState::NotStarted,
            completed: None,
        }),
        Err(_) => Err(error(Code::Io, "released stage state unavailable")),
        Ok(_) => {
            let value = native::status_released(&root, evidence, token)?;
            if value.native.plan_sha256 != stage.plan.plan_sha256 {
                return Err(error(
                    Code::RecoveryRequired,
                    "released stage belongs to another plan",
                ));
            }
            let completed = match (
                value.native.state,
                value.native.receipt,
                value.retained_release_record,
            ) {
                (native::State::Complete, Some(receipt), Some(record)) => Some(CompletedStage {
                    role: stage.role.clone(),
                    native_receipt: receipt,
                    admission: value.admission,
                    release_record: record,
                }),
                _ => None,
            };
            Ok(StageStatus {
                role: stage.role.clone(),
                state: if completed.is_some() {
                    StageState::Complete
                } else {
                    StageState::RecoveryRequired
                },
                completed,
            })
        }
    }
}
fn handoff(plan: &Plan, stages: Vec<CompletedStage>, evidence: &VerifiedEvidence) -> Handoff {
    let verified = evidence.summary();
    Handoff {
        schema_version: "flow.released-pilot-handoff/v1".into(),
        plan_sha256: plan.plan_sha256.clone(),
        metadata: plan.request.metadata.clone(),
        manual_srt: plan.request.manual_srt.clone(),
        audio_source: plan.request.audio_source.clone(),
        stages,
        verification_binding_sha256: verified.binding_sha256,
        verification_audit: verified.audit,
        provider_release_qualified: true,
        creative_review_pending: true,
        upload_authority: false,
    }
}
fn saved(
    root: &Path,
    plan: &Plan,
    expected: &Handoff,
    token: &dyn CancellationSignal,
) -> Result<Option<Handoff>, Error> {
    match fs::symlink_metadata(root.join("released-handoff.json")) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(error(Code::Io, "released pilot handoff unavailable")),
        Ok(_) => {
            let value: Handoff = read(&root.join("released-handoff.json"))?;
            let mut same = expected.clone();
            same.verification_audit = value.verification_audit.clone();
            if value != same {
                return Err(error(
                    Code::RecoveryRequired,
                    "released handoff differs from current native/provider admission",
                ));
            }
            value.verification_audit.current(MAX, token)?;
            let audit: Value = serde_json::from_slice(&command::read_document(
                &native::path(&value.verification_audit.path)?,
                MAX,
            )?)
            .map_err(|_| error(Code::RecoveryRequired, "released handoff audit malformed"))?;
            if audit["binding_sha256"] != value.verification_binding_sha256
                || audit["cohort_sha256"] != plan.request.video.cohort_sha256
            {
                return Err(error(
                    Code::RecoveryRequired,
                    "released handoff audit subject differs",
                ));
            }
            Ok(Some(value))
        }
    }
}
pub fn status(
    root: &Path,
    evidence: &VerifiedEvidence,
    token: &dyn CancellationSignal,
) -> Result<Status, Error> {
    let _held = lock(root, false)?;
    let (plan, _) = basis(root, evidence, token)?;
    let stages = plan
        .stages
        .iter()
        .map(|s| observe(s, evidence, token))
        .collect::<Result<Vec<_>, _>>()?;
    let completed = stages
        .iter()
        .map(|s| s.completed.clone())
        .collect::<Option<Vec<_>>>();
    let handoff = if let Some(completed) = completed {
        saved(root, &plan, &handoff(&plan, completed, evidence), token)?
    } else {
        if fs::symlink_metadata(root.join("released-handoff.json")).is_ok() {
            return Err(error(
                Code::RecoveryRequired,
                "released handoff exists without current complete stages",
            ));
        }
        None
    };
    Ok(Status{schema_version:"flow.released-pilot-status/v1".into(),plan_sha256:plan.plan_sha256,stages,handoff,verification:evidence.summary(),next_action:"use explicit native resume-released for a partial stage or final bookkeeping; then pilot-continue-released with the exact original authority; human review and upload remain separate".into()})
}
/// Reuse accepted stages only; uncertainty never causes a native replay.
pub fn run(
    plan: &Plan,
    approval: &Approval,
    evidence: &VerifiedEvidence,
    continue_existing: bool,
    token: &dyn CancellationSignal,
) -> Result<Handoff, Error> {
    plan.require_current(token)?;
    approval.validate(plan, evidence, token)?;
    let root = native::path(&plan.request.run_directory)?;
    if !continue_existing {
        create(&root)?;
    }
    let _held = lock(&root, !continue_existing)?;
    let action = (|| {
        if continue_existing {
            let (p, a) = basis(&root, evidence, token)?;
            if p != *plan
                || serde_json::to_value(a)
                    .map_err(|_| error(Code::InvalidInput, "released authority encoding"))?
                    != serde_json::to_value(approval)
                        .map_err(|_| error(Code::InvalidInput, "released authority encoding"))?
            {
                return Err(error(
                    Code::RecoveryRequired,
                    "retained released pilot differs from supplied plan/authority",
                ));
            }
        } else {
            command::write_document(&root.join("pilot-plan.json"), plan)?;
            command::write_document(&root.join("pilot-approval.json"), approval)?;
        }
        let mut completed = vec![];
        for (index, (stage, authority)) in plan.stages.iter().zip(&approval.stages).enumerate() {
            cancelled(token)?;
            let native_root = native::path(&stage.plan.request.run_directory)?;
            let started = root.join(format!("stage-{index:02}-started.json"));
            let accepted = match fs::symlink_metadata(&native_root) {
                Ok(_) => {
                    if !continue_existing {
                        return Err(error(
                            Code::RecoveryRequired,
                            "released pilot stage already exists",
                        ));
                    }
                    observe(stage,evidence,token)?.completed.ok_or_else(||error(Code::RecoveryRequired,"partial released stage needs explicit native resume-released; no replay"))?
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    if fs::symlink_metadata(&started).is_ok() {
                        return Err(error(
                            Code::RecoveryRequired,
                            "started released stage lost its root; reconcile without replay",
                        ));
                    }
                    command::write_document(
                        &started,
                        &json!({"schema_version":"flow.released-pilot-stage-start/v1","pilot_plan_sha256":plan.plan_sha256,"stage_plan_sha256":stage.plan.plan_sha256,"admission":authority.admission}),
                    )?;
                    let value = native::run_released(&stage.plan, authority, evidence, token)?;
                    CompletedStage {
                        role: stage.role.clone(),
                        native_receipt: value.native,
                        admission: value.admission,
                        release_record: value.retained_release_record,
                    }
                }
                Err(_) => return Err(error(Code::Io, "released pilot stage state unavailable")),
            };
            completed.push(accepted);
        }
        plan.require_current(token)?;
        for (stage, accepted) in plan.stages.iter().zip(&completed) {
            if observe(stage, evidence, token)?.completed.as_ref() != Some(accepted) {
                return Err(error(
                    Code::RecoveryRequired,
                    "released stage changed before aggregate handoff",
                ));
            }
        }
        let value = handoff(plan, completed, evidence);
        if let Some(saved) = saved(&root, plan, &value, token)? {
            return Ok(saved);
        }
        command::write_document(&root.join("released-handoff.json"), &value)?;
        saved(&root, plan, &value, token)?
            .ok_or_else(|| error(Code::RecoveryRequired, "released handoff commit is absent"))
    })();
    action.map_err(|mut e: Error| {
        e.recovery_directory = Some(NativePath::from_path(&root));
        e
    })
}
