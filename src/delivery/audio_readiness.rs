//! Delivery-file rows over fresh, explicitly approved Aniflow owner checks.
//! No audio math, mastering, review declaration or publication lives in Flow.
use super::*;
use crate::release_evidence::VerifiedEvidence;

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum Authority {SourceCandidate {approval:FilePin},Released {approval:FilePin}}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum Choice {
    Check {plan:FilePin,authority:Authority},
    KeepArchiveMaster {reason:String},
}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {pub payload_id:String,pub destination:Destination,pub choice:Choice}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {pub schema_version:String,pub package_manifest:FilePin,pub selections:Vec<Selection>,pub timeout_ms:u64}
impl Request {
    pub fn from_json_slice(bytes:&[u8])->Result<Self,Error>{if bytes.len()>1_048_576{return Err(error(Code::InvalidInput,"audio readiness request bound"));}let r:Self=serde_json::from_slice(bytes).map_err(|_|error(Code::InvalidInput,"closed audio readiness request required"))?;r.validate()?;Ok(r)}
    fn validate(&self)->Result<(),Error>{if self.schema_version!="flow.audio-readiness-request/v1"||self.selections.len()>128||self.timeout_ms==0||self.timeout_ms>86_400_000||!release_cohort::is_digest(&self.package_manifest.sha256){return Err(error(Code::InvalidInput,"bounded explicit package/check selection required"));}Ok(())}
    pub fn needs_released_evidence(&self)->bool{self.selections.iter().any(|s|matches!(&s.choice,Choice::Check {authority:Authority::Released {..},..}))}
}
fn audio_role(role:Role)->bool{matches!(role,Role::CleanMaster|Role::VideoDelivery|Role::OriginalAudio|Role::AudioDerivative)}
fn key(id:&str,destination:Destination)->String{format!("{id}:{destination:?}")}
fn archive_skip(role:Role,destination:Destination,reason:&str)->bool {
    destination==Destination::Archive&&matches!(role,Role::CleanMaster|Role::OriginalAudio)&&!reason.trim().is_empty()&&reason.len()<=512&&!reason.chars().any(char::is_control)
}

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub payload_id:String,pub destination:Destination,pub relative:String,pub sha256:String,pub byte_count:u64,
    pub state:String,pub policy:Option<Value>,pub integrated_loudness:Option<Value>,pub true_peak:Option<Value>,
    pub matches_selected_policy:Option<bool>,pub meter_qualified:bool,pub provider_release_qualified:bool,
    pub native_record:Option<FilePin>,pub note:String,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Summary {
    pub schema_version:String,pub package_subject_sha256:String,pub package_manifest_sha256:String,
    pub rows:Vec<Row>,pub reviews:ReviewSummary,pub missing_metadata:Vec<String>,pub missing_destination_payloads:Vec<Destination>,
    pub all_selected_audio_goals_match:bool,pub owner_checks_performed:usize,pub qualification_pending:bool,
    pub evidence_scope:String,pub upload_authority:bool,pub current_platform_compliance_verified:bool,
}
struct Prepared {package:Status,rows:Vec<Row>,plans:Vec<(usize,native::Plan,Authority)>}
fn prepare(request:&Request,token:&dyn CancellationSignal)->Result<Prepared,Error>{
    request.validate()?;
    let manifest=native::path(&request.package_manifest.path)?;
    if manifest.file_name().and_then(|n|n.to_str())!=Some("manifest.json"){return Err(error(Code::InvalidInput,"exact delivery manifest.json required"));}
    request.package_manifest.current(MAX_DOC,token)?;
    let root=manifest.parent().ok_or_else(||error(Code::InvalidInput,"delivery package parent required"))?;
    let package=super::status(root,&request.package_manifest.sha256,token)?;
    let required=package.preview.payloads.iter().filter(|p|audio_role(p.role)).flat_map(|p|p.destinations.iter().map(move|d|key(&p.id,*d))).collect::<BTreeSet<_>>();
    let mut chosen=BTreeSet::new();let mut rows=vec![];let mut plans=vec![];let mut outputs:Vec<std::path::PathBuf>=vec![];
    for selection in &request.selections {
        if !chosen.insert(key(&selection.payload_id,selection.destination)){return Err(error(Code::InvalidInput,"duplicate audio readiness payload/destination"));}
        let payload=package.preview.payloads.iter().find(|p|p.id==selection.payload_id&&p.destinations.contains(&selection.destination)&&audio_role(p.role)).ok_or_else(||error(Code::InvalidInput,"audio selection must identify an exact assigned media payload"))?;
        let mut row=Row {payload_id:payload.id.clone(),destination:selection.destination,relative:payload.relative.clone(),sha256:payload.sha256.clone(),byte_count:payload.byte_count,state:"fresh_owner_check_required".into(),policy:None,integrated_loudness:None,true_peak:None,matches_selected_policy:None,meter_qualified:false,provider_release_qualified:false,native_record:None,note:String::new()};
        match &selection.choice {
            Choice::KeepArchiveMaster {reason}=>{if !archive_skip(payload.role,selection.destination,reason){return Err(error(Code::InvalidInput,"only an explicitly unchanged archival master/original audio can skip target processing"));}row.state="archive_master_unchanged".into();row.note=reason.clone();},
            Choice::Check {plan,authority}=>{
                let p=native::Plan::from_json_slice(&super::read(plan,1_048_576,token)?)?;p.require_current(token)?;
                if p.row.tool!=crate::release_cohort::Tool::Aniflow||p.row.profile!="audio-policy-check-v1"||p.sources.len()!=1||p.sources[0].sha256!=payload.sha256||p.sources[0].byte_count!=payload.byte_count{return Err(error(Code::InvalidInput,"audio owner plan must select the exact assigned final media bytes and policy-check profile"));}
                let output=native::path(&p.request.run_directory)?;
                if output.starts_with(root)||root.starts_with(&output)||outputs.iter().any(|p|output.starts_with(p)||p.starts_with(&output)){return Err(error(Code::InvalidInput,"owner check workspaces must be separate from the package and one another"));}
                outputs.push(output);
                let approval=match authority {Authority::SourceCandidate {approval}|Authority::Released {approval}=>approval};approval.current(1_048_576,token)?;
                let config:Value=serde_json::from_slice(&super::read(&p.configuration,131_072,token)?).map_err(|_|error(Code::InvalidInput,"explicit native policy request JSON required"))?;
                row.policy=Some(config["policy"].clone());plans.push((rows.len(),p,authority.clone()));
            },
        }
        rows.push(row);
    }
    if chosen!=required{return Err(error(Code::InvalidInput,"every assigned audio/video payload and destination needs an explicit check or archival-master choice"));}
    Ok(Prepared {package,rows,plans})
}
fn summary(prepared:Prepared,checks:usize,scope:&str)->Summary {
    let all=prepared.rows.iter().all(|r|r.matches_selected_policy==Some(true)||r.state=="archive_master_unchanged");
    let qualification_pending=prepared.rows.iter().any(|r|r.state!="archive_master_unchanged"&&(!r.meter_qualified||!r.provider_release_qualified));
    Summary {schema_version:"flow.audio-readiness-summary/v1".into(),package_subject_sha256:prepared.package.preview.package_subject_sha256,package_manifest_sha256:prepared.package.manifest.sha256,rows:prepared.rows,reviews:prepared.package.preview.reviews,missing_metadata:prepared.package.preview.missing_metadata,missing_destination_payloads:prepared.package.preview.missing_destination_payloads,all_selected_audio_goals_match:all,owner_checks_performed:checks,qualification_pending,evidence_scope:scope.into(),upload_authority:false,current_platform_compliance_verified:false}
}
/// Read-only; no domain process, approval generation or workspace creation.
pub fn preview(request:&Request,token:&dyn CancellationSignal)->Result<Summary,Error>{Ok(summary(prepare(request,token)?,0,"preview_only; current_owner_checks_not_performed"))}

enum Loaded {Candidate(native::Approval),Released(native::ReleasedApproval)}
/// Only an explicit call with existing exact native approvals can launch checks.
/// Released choices also require the existing opaque verified-evidence authority.
pub fn run(request:&Request,evidence:Option<&VerifiedEvidence>,token:&dyn CancellationSignal)->Result<Summary,Error>{
    let started=Instant::now();let guard=||token.is_cancelled()||started.elapsed()>=Duration::from_millis(request.timeout_ms);
    let mut prepared=prepare(request,&guard)?;
    let mut loaded=vec![];
    for (_,plan,authority) in &prepared.plans {
        if fs::symlink_metadata(native::path(&plan.request.run_directory)?).is_ok(){return Err(error(Code::RecoveryRequired,"a fresh current owner-check workspace is required; retained snapshots cannot substitute"));}
        let a=match authority {
            Authority::SourceCandidate {approval}=>{
                let value:native::Approval=serde_json::from_slice(&super::read(approval,1_048_576,&guard)?).map_err(|_|error(Code::InvalidInput,"closed candidate approval required"))?;
                if native::approve(plan,value.approved_by.clone(),value.trusted_unconfined,value.unqualified_candidate)?!=value{return Err(error(Code::ApprovalRequired,"exact existing candidate approval required"));}Loaded::Candidate(value)
            },
            Authority::Released {approval}=>{
                let value:native::ReleasedApproval=serde_json::from_slice(&super::read(approval,1_048_576,&guard)?).map_err(|_|error(Code::InvalidInput,"closed released approval required"))?;
                let evidence=evidence.ok_or_else(||error(Code::ApprovalRequired,"released checks require current opaque release evidence"))?;
                if native::approve_released(plan,value.approved_by.clone(),value.trusted_unconfined,evidence,&guard)?!=value{return Err(error(Code::ApprovalRequired,"exact existing released approval required"));}Loaded::Released(value)
            },
        };loaded.push(a);
    }
    for ((index,plan,_),authority) in prepared.plans.iter().zip(loaded) {
        let (receipt,released)=match authority {
            Loaded::Candidate(a)=>(native::run(plan,&a,&guard)?,false),
            Loaded::Released(a)=>{let value=native::run_released(plan,&a,evidence.ok_or_else(||error(Code::ApprovalRequired,"release authority missing"))?,&guard)?;
                if !value.release_qualified||value.creative_approval_inferred||value.publication_authority{return Err(error(Code::ProviderRefused,"released owner-check authority scope differs"));}(value.native,value.release_qualified)
            },
        };
        let assessment=native::audio_policy_snapshot(&native::path(&plan.request.run_directory)?,&plan.plan_sha256,&guard)?;
        let evaluation=&assessment["evaluation"];let row=&mut prepared.rows[*index];
        row.integrated_loudness=Some(evaluation["integrated_loudness"].clone());row.true_peak=Some(evaluation["true_peak"].clone());row.matches_selected_policy=evaluation["matches_selected_policy"].as_bool();row.meter_qualified=evaluation["runtime_qualified"].as_bool().unwrap_or(false);row.provider_release_qualified=released;
        row.state=if row.matches_selected_policy==Some(true){"matches_selected_policy"}else if evaluation["integrated_loudness"]["state"]=="missing_evidence"||evaluation["true_peak"]["state"]=="missing_evidence"{"missing_evidence"}else{"outside_target"}.into();
        row.native_record=Some(FilePin::observe(&receipt.acceptance_file,MAX_DOC,&guard)?);
        row.note="Owner checked exact final-file QC in this attempt; retained status requires another explicit check for current QC. Numeric matching does not qualify the meter or approve music.".into();
    }
    for (_,plan,_) in &prepared.plans {plan.require_current(&guard)?;}
    let current=super::status(&native::path(&prepared.package.current_root)?,&request.package_manifest.sha256,&guard)?;
    if current.preview!=prepared.package.preview{return Err(error(Code::StalePlan,"delivery metadata/payload/review context changed during owner checks"));}
    let count=prepared.plans.len();Ok(summary(prepared,count,"owner_rechecked_during_this_attempt; not_a_permanent_or_platform_certificate"))
}

/// Keep explicit report/verification output outside selected originals, package,
/// native/QC evidence, configuration, authorities and provider/tool bytes.
pub fn validate_output_path(request:&Request,output:&Path,token:&dyn CancellationSignal)->Result<(),Error>{
    let prepared=prepare(request,token)?;let output=command::absolute(output)?;
    let package=native::path(&prepared.package.current_root)?;
    let overlaps=|p:&Path|output.starts_with(p)||p.starts_with(&output);
    if overlaps(&package){return Err(error(Code::InvalidInput,"audio readiness output must be outside the delivery package"));}
    for selection in &request.selections {if let Choice::Check {plan,authority}=&selection.choice {
        let approval=match authority {Authority::SourceCandidate {approval}|Authority::Released {approval}=>approval};
        for pin in [plan,approval]{if overlaps(&native::path(&pin.path)?){return Err(error(Code::InvalidInput,"audio readiness output overlaps selected plan/authority"));}}
    }}
    for (_,plan,_) in &prepared.plans {
        for p in native::audio_policy_protected_paths(&plan.request)? {if overlaps(&p){return Err(error(Code::InvalidInput,"audio readiness output overlaps native/QC owner evidence"));}}
        for p in [&plan.request.run_directory,&plan.request.provider_executable,&plan.request.release_descriptor,&plan.request.cohort,&plan.request.configuration].into_iter().chain(plan.request.inputs.iter()).chain(plan.request.tool_directories.iter()) {
            if overlaps(&native::path(p)?){return Err(error(Code::InvalidInput,"audio readiness output overlaps source/provider/native context"));}
        }
        if let Some(pin)=&plan.request.preparation {let p=native::path(&pin.path)?;if overlaps(p.parent().ok_or_else(||error(Code::InvalidInput,"preparation parent required"))?){return Err(error(Code::InvalidInput,"audio readiness output overlaps preparation"));}}
    }Ok(())
}
pub fn save(request:&Request,output:&Path,value:&Summary,token:&dyn CancellationSignal)->Result<(),Error>{validate_output_path(request,output,token)?;command::write_document(&command::absolute(output)?,value)}

#[cfg(test)]mod tests {
    use super::*;
    #[test]fn archival_choice_cannot_skip_a_platform_copy_or_derivative(){assert!(archive_skip(Role::CleanMaster,Destination::Archive,"Preserve approved master"));assert!(!archive_skip(Role::VideoDelivery,Destination::Tiktok,"skip"));assert!(!archive_skip(Role::AudioDerivative,Destination::Archive,"skip"));assert!(!archive_skip(Role::OriginalAudio,Destination::Archive,""));}
}
