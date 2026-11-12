//! Fixed manual-lyrics publishing pilot. Domain work uses existing native SDKs.
use std::{fs::{self, File}, path::Path};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use crate::{CancellationSignal, command::{self, Code, Error}, native::{self, FilePin}, optiflow::NativePath, release_cohort::{self, Tool}};

const MAX: u64 = 16_777_216;
fn error(code: Code, text: &str) -> Error { Error {code, message:text.into(), recovery_directory:None} }
fn cancelled(token: &dyn CancellationSignal) -> Result<(), Error> { if token.is_cancelled() {Err(error(Code::Cancelled,"pilot cancelled; retain all stage evidence"))} else {Ok(())} }
fn sha<T: Serialize>(value: &T) -> Result<String, Error> { let v=serde_json::to_value(value).map_err(|_|error(Code::InvalidInput,"pilot encoding"))?; Ok(release_cohort::sha256(&serde_json::to_vec(&v).map_err(|_|error(Code::InvalidInput,"pilot encoding"))?)) }
fn read<T: for<'a> Deserialize<'a>>(path:&Path) -> Result<T,Error> { serde_json::from_slice(&command::read_document(path,MAX)?).map_err(|_|error(Code::RecoveryRequired,"closed retained pilot document required")) }
fn overlap(a:&Path,b:&Path)->bool {a.starts_with(b)||b.starts_with(a)}
fn directory(path:&Path)->Result<(),Error> {if path.canonicalize().ok().as_deref()!=Some(path)||!fs::metadata(path).is_ok_and(|m|m.is_dir()){return Err(error(Code::InvalidInput,"canonical pilot directory required"));}Ok(())}
fn create(path:&Path)->Result<(),Error> {directory(path.parent().ok_or_else(||error(Code::InvalidInput,"existing pilot parent required"))?)?;let mut b=fs::DirBuilder::new();#[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;b.mode(0o700);}b.create(path).map_err(|_|error(Code::RecoveryRequired,"fresh pilot output required"))?;directory(path)}
struct Held(File);
impl Drop for Held {fn drop(&mut self){let _=FileExt::unlock(&self.0);}}
fn lock(root:&Path,first:bool)->Result<Held,Error>{directory(root)?;let mut o=fs::OpenOptions::new();o.read(true).write(true).create_new(first);#[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;o.mode(0o600).custom_flags(nix::libc::O_NOFOLLOW|nix::libc::O_CLOEXEC);}let f=o.open(root.join("pilot.lock")).map_err(|_|error(Code::RecoveryRequired,"retained pilot lock required"))?;if !f.metadata().map_err(|_|error(Code::Io,"pilot lock metadata"))?.is_file(){return Err(error(Code::RecoveryRequired,"regular pilot lock required"));}f.try_lock_exclusive().map_err(|_|error(Code::RecoveryRequired,"pilot already active"))?;Ok(Held(f))}

#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {pub work_id:String,pub title:String,pub language:String}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version:String,pub metadata:Metadata,pub manual_srt:FilePin,
    pub audio_source:Option<FilePin>,pub video:native::Request,
    pub timed_text:native::Request,pub audio:Option<native::Request>,pub run_directory:NativePath,
}
impl Request {
    pub fn from_json_slice(bytes:&[u8])->Result<Self,Error>{if bytes.len()>1_048_576{return Err(error(Code::InvalidInput,"pilot request bound"));}let value:Self=serde_json::from_slice(bytes).map_err(|_|error(Code::InvalidInput,"closed pilot request required"))?;value.validate()?;Ok(value)}
    fn stages(&self)->Vec<(&'static str,&native::Request)>{let mut v=vec![("video",&self.video),("timed_text",&self.timed_text)];if let Some(a)=&self.audio{v.push(("audio",a));}v}
    fn validate(&self)->Result<(),Error>{
        if self.schema_version!="flow.manual-srt-pilot-request/v1"||[&self.metadata.work_id,&self.metadata.title,&self.metadata.language].into_iter().any(|s|s.is_empty()||s.len()>512||s.chars().any(char::is_control))||self.audio.is_some()!=self.audio_source.is_some(){return Err(error(Code::InvalidInput,"pilot metadata/audio selection"));}
        let lyrics=native::path(&self.manual_srt.path)?;if lyrics.extension().and_then(|s|s.to_str()).is_none_or(|s|!s.eq_ignore_ascii_case("srt"))||!release_cohort::is_digest(&self.manual_srt.sha256)||self.manual_srt.byte_count==0||self.manual_srt.byte_count>16_777_216{return Err(error(Code::InvalidInput,"exact manually supplied SRT pin required"));}
        if self.timed_text.inputs!=vec![self.manual_srt.path.clone()]||self.video.inputs.len()!=1 {return Err(error(Code::InvalidInput,"pilot video and manual SRT selections must be exact"));}
        if let (Some(audio),Some(source))=(&self.audio,&self.audio_source){if audio.inputs!=vec![source.path.clone()]||!release_cohort::is_digest(&source.sha256)||source.byte_count==0{return Err(error(Code::InvalidInput,"exact supplied audio selection required"));}}
        let root=native::path(&self.run_directory)?;let stages=self.stages();
        let mut outputs=vec![root];
        for (_,stage) in &stages {outputs.push(native::path(&stage.run_directory)?);}
        for (i,output) in outputs.iter().enumerate(){
            for other in outputs.iter().skip(i+1){if overlap(output,other){return Err(error(Code::InvalidInput,"pilot and native outputs must be disjoint sibling trees"));}}
            for (_,stage) in &stages {
                for source in [&stage.cohort,&stage.provider_executable,&stage.release_descriptor,&stage.configuration].into_iter().chain(stage.inputs.iter()).chain(stage.tool_directories.iter()) {
                    if overlap(output,&native::path(source)?){return Err(error(Code::InvalidInput,"pilot outputs overlap selected originals/contracts/tools"));}
                }
                if let Some(pin)=&stage.preparation {let p=native::path(&pin.path)?;if overlap(output,p.parent().ok_or_else(||error(Code::InvalidInput,"preparation root required"))?){return Err(error(Code::InvalidInput,"pilot output overlaps preparation bundle"));}}
            }
        }
        Ok(())
    }
}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {pub role:String,pub plan:native::Plan}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {pub schema_version:String,pub plan_sha256:String,pub request:Request,pub stages:Vec<Stage>}
impl Plan {
    pub fn from_json_slice(bytes:&[u8])->Result<Self,Error>{if bytes.len() as u64>MAX{return Err(error(Code::InvalidInput,"pilot plan bound"));}let value:Self=serde_json::from_slice(bytes).map_err(|_|error(Code::InvalidInput,"closed pilot plan required"))?;value.validate()?;Ok(value)}
    pub fn validate(&self)->Result<(),Error>{
        self.request.validate()?;let mut unsigned=self.clone();unsigned.plan_sha256.clear();
        if self.schema_version!="flow.manual-srt-pilot-plan/v1"||sha(&unsigned)?!=self.plan_sha256||self.stages.len()!=self.request.stages().len(){return Err(error(Code::InvalidInput,"pilot plan identity"));}
        for (stage,(role,selected)) in self.stages.iter().zip(self.request.stages()) {
            stage.plan.validate()?;
            let expected=if role=="video"{Tool::Aniflow}else{Tool::Renderflow};
            if stage.role!=role||stage.plan.request!=*selected||stage.plan.row.tool!=expected||stage.plan.request.cohort_sha256!=self.request.video.cohort_sha256||stage.plan.row.target!=self.stages[0].plan.row.target{return Err(error(Code::InvalidInput,"pilot stage/cohort/target binding"));}
            if role=="timed_text"&&stage.plan.sources!=vec![self.request.manual_srt.clone()]{return Err(error(Code::InvalidInput,"pilot timed-text bytes differ from supplied manual SRT"));}
            if role=="audio"&&Some(stage.plan.sources[0].clone())!=self.request.audio_source{return Err(error(Code::InvalidInput,"pilot audio bytes differ from selected original"));}
        }Ok(())
    }
    pub(crate) fn require_current(&self,token:&dyn CancellationSignal)->Result<(),Error>{self.validate()?;cancelled(token)?;for stage in &self.stages{stage.plan.require_current(token)?;}Ok(())}
}
/// Read-only composition. No domain program, automatic transcription or workspace creation.
pub fn prepare(request:&Request,token:&dyn CancellationSignal)->Result<Plan,Error>{
    request.validate()?;let root=native::path(&request.run_directory)?;directory(root.parent().ok_or_else(||error(Code::InvalidInput,"pilot parent required"))?)?;
    if fs::symlink_metadata(&root).is_ok(){return Err(error(Code::RecoveryRequired,"fresh pilot root required"));}
    request.manual_srt.current(16_777_216,token)?;
    if let Some(source)=&request.audio_source{source.current(68_719_476_736,token)?;}
    let mut stages=vec![];for (role,selected) in request.stages(){stages.push(Stage {role:role.into(),plan:native::prepare(selected,token)?});}
    // Native configuration validation owns conversion algorithms. This preset binds manual source kind.
    let config:serde_json::Value=read(&native::path(&request.timed_text.configuration)?)?;
    if config["sources"].as_array().is_none_or(|s|s.len()!=1||s[0]["format"]!="srt") {return Err(error(Code::InvalidInput,"pilot timed-text configuration must explicitly select supplied SRT"));}
    if config["execution"]["lyrics"].is_object() && config["execution"]["lyrics"]["source_sha256"]!=request.manual_srt.sha256{return Err(error(Code::InvalidInput,"native lyric profile must pin this supplied SRT"));}
    let mut plan=Plan {schema_version:"flow.manual-srt-pilot-plan/v1".into(),plan_sha256:String::new(),request:request.clone(),stages};plan.plan_sha256=sha(&plan)?;plan.require_current(token)?;Ok(plan)
}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Approval {pub schema_version:String,pub plan_sha256:String,pub stages:Vec<native::Approval>}
impl Approval {fn validate(&self,plan:&Plan)->Result<(),Error>{
    if self.schema_version!="flow.manual-srt-pilot-approval/v1"||self.plan_sha256!=plan.plan_sha256||self.stages.len()!=plan.stages.len(){return Err(error(Code::ApprovalRequired,"exact pilot approval required"));}
    for (a,s) in self.stages.iter().zip(&plan.stages){if native::approve(&s.plan,a.approved_by.clone(),a.trusted_unconfined,a.unqualified_candidate)?!=*a{return Err(error(Code::ApprovalRequired,"pilot stage approval differs"));}}Ok(())
}}
pub fn approve(plan:&Plan,by:String,trusted_unconfined:bool,unqualified_candidate:bool)->Result<Approval,Error>{plan.validate()?;let stages=plan.stages.iter().map(|s|native::approve(&s.plan,by.clone(),trusted_unconfined,unqualified_candidate)).collect::<Result<_,_>>()?;Ok(Approval{schema_version:"flow.manual-srt-pilot-approval/v1".into(),plan_sha256:plan.plan_sha256.clone(),stages})}
pub fn save<T:Serialize>(plan:&Plan,output:&Path,value:&T)->Result<(),Error>{
    plan.validate()?;let output=command::absolute(output)?;
    for (_,r) in plan.request.stages(){for source in [&r.cohort,&r.provider_executable,&r.release_descriptor,&r.configuration,&r.run_directory,&plan.request.run_directory].into_iter().chain(r.inputs.iter()).chain(r.tool_directories.iter()){if overlap(&output,&native::path(source)?){return Err(error(Code::InvalidInput,"pilot documents must be outside selected sources and reserved outputs"));}}if let Some(pin)=&r.preparation{let p=native::path(&pin.path)?;if output.starts_with(p.parent().ok_or_else(||error(Code::InvalidInput,"preparation root required"))?){return Err(error(Code::InvalidInput,"pilot document overlaps preparation"));}}}
    command::write_document(&output,value)
}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletedStage {pub role:String,pub receipt:native::Receipt}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Handoff {pub schema_version:String,pub plan_sha256:String,pub metadata:Metadata,pub manual_srt:FilePin,pub audio_source:Option<FilePin>,pub stages:Vec<CompletedStage>,pub review_pending:bool,pub upload_authority:bool,pub release_qualified:bool}
fn handoff(plan:&Plan,stages:Vec<CompletedStage>)->Handoff {Handoff {schema_version:"flow.publishing-pilot-handoff/v1".into(),plan_sha256:plan.plan_sha256.clone(),metadata:plan.request.metadata.clone(),manual_srt:plan.request.manual_srt.clone(),audio_source:plan.request.audio_source.clone(),stages,review_pending:true,upload_authority:false,release_qualified:false}}
#[derive(Debug,Serialize)]
pub struct Status {pub schema_version:String,pub plan_sha256:String,pub stages:Vec<(String,Option<native::Receipt>)>,pub handoff:Option<Handoff>,pub next_action:String}
fn basis(root:&Path)->Result<(Plan,Approval),Error>{let p=Plan::from_json_slice(&command::read_document(&root.join("pilot-plan.json"),MAX)?)?;let a:Approval=read(&root.join("pilot-approval.json"))?;a.validate(&p)?;if native::path(&p.request.run_directory)?!=root{return Err(error(Code::RecoveryRequired,"pilot moved from approved destination"));}Ok((p,a))}
fn retained(plan:&Plan,token:&dyn CancellationSignal)->Result<Vec<(String,Option<native::Receipt>)>,Error>{
    let mut stages=vec![];for stage in &plan.stages{cancelled(token)?;let root=native::path(&stage.plan.request.run_directory)?;let receipt=match fs::symlink_metadata(&root){Ok(_)=>{let value=native::status(&root,token)?;if value.plan_sha256!=stage.plan.plan_sha256{return Err(error(Code::RecoveryRequired,"pilot stage belongs to another plan"));}value.receipt},Err(e) if e.kind()==std::io::ErrorKind::NotFound=>None,Err(_)=>return Err(error(Code::Io,"pilot stage state unavailable"))};stages.push((stage.role.clone(),receipt));}Ok(stages)
}
/// Read-only native receipt and current artifact readback, no provider contact.
pub fn status(root:&Path,token:&dyn CancellationSignal)->Result<Status,Error>{
    let _held=lock(root,false)?;let (plan,_)=basis(root)?;plan.require_current(token)?;let stages=retained(&plan,token)?;let expected=if stages.iter().all(|(_,r)|r.is_some()){Some(handoff(&plan,stages.iter().map(|(role,r)|CompletedStage{role:role.clone(),receipt:r.clone().expect("checked")}).collect()))}else{None};
    let saved=match fs::symlink_metadata(root.join("handoff.json")){Ok(_)=>{let h:Handoff=read(&root.join("handoff.json"))?;if Some(&h)!=expected.as_ref(){return Err(error(Code::RecoveryRequired,"pilot handoff differs from current accepted stage artifacts"));}Some(h)},Err(e) if e.kind()==std::io::ErrorKind::NotFound=>None,Err(_)=>return Err(error(Code::Io,"pilot handoff state unavailable"))};
    Ok(Status{schema_version:"flow.manual-srt-pilot-status/v1".into(),plan_sha256:plan.plan_sha256,stages,handoff:saved,next_action:"review retained native evidence; resume an explicitly identified partial native run, then pilot-continue; never infer creative or publishing approval".into()})
}
/// Sequential orchestration uses the existing native supervisor and recovery.
/// Continue only reuses accepted stages; an uncertain started stage is never replayed.
pub fn run(plan:&Plan,approval:&Approval,continue_existing:bool,token:&dyn CancellationSignal)->Result<Handoff,Error>{
    plan.require_current(token)?;approval.validate(plan)?;let root=native::path(&plan.request.run_directory)?;
    if !continue_existing{create(&root)?;}let _held=lock(&root,!continue_existing)?;
    let action=(||{
        if continue_existing{let (p,a)=basis(&root)?;if p!=*plan||a!=*approval{return Err(error(Code::RecoveryRequired,"retained pilot differs from approved plan"));}}
        else{command::write_document(&root.join("pilot-plan.json"),plan)?;command::write_document(&root.join("pilot-approval.json"),approval)?;}
        let mut completed=vec![];
        for (index,(stage,authority)) in plan.stages.iter().zip(&approval.stages).enumerate(){
            cancelled(token)?;let stage_root=native::path(&stage.plan.request.run_directory)?;let started=root.join(format!("stage-{index:02}-started.json"));
            let receipt=match fs::symlink_metadata(&stage_root){
                Ok(_)=>{if !continue_existing{return Err(error(Code::RecoveryRequired,"pilot stage already exists"));}let value=native::status(&stage_root,token)?;if value.plan_sha256!=stage.plan.plan_sha256{return Err(error(Code::RecoveryRequired,"retained stage plan differs"));}value.receipt.ok_or_else(||error(Code::RecoveryRequired,"partial stage requires explicit native resume before pilot-continue"))?},
                Err(e) if e.kind()==std::io::ErrorKind::NotFound=>{if fs::symlink_metadata(&started).is_ok(){return Err(error(Code::RecoveryRequired,"started stage lost its directory; reconcile, never replay"));}command::write_document(&started,&serde_json::json!({"schema_version":"flow.pilot-stage-start/v1","pilot_plan_sha256":plan.plan_sha256,"stage_plan_sha256":stage.plan.plan_sha256,"role":stage.role}))?;native::run(&stage.plan,authority,token)?},
                Err(_)=>return Err(error(Code::Io,"pilot stage state unavailable")),
            };completed.push(CompletedStage{role:stage.role.clone(),receipt});
        }
        // Later stages cannot turn an earlier changed artifact into a complete handoff.
        plan.require_current(token)?;
        let now=retained(plan,token)?;
        if now.iter().zip(&completed).any(|((role,r),old)|role!=&old.role||r.as_ref()!=Some(&old.receipt)){return Err(error(Code::RecoveryRequired,"accepted pilot stage changed before handoff"));}
        let value=handoff(plan,completed);let output=root.join("handoff.json");
        match fs::symlink_metadata(&output){Ok(_)=>{if read::<Handoff>(&output)?!=value{return Err(error(Code::RecoveryRequired,"existing pilot handoff differs"));}},Err(e) if e.kind()==std::io::ErrorKind::NotFound=>command::write_document(&output,&value)?,Err(_)=>return Err(error(Code::Io,"pilot handoff unavailable"))};Ok(value)
    })();action.map_err(|mut e:Error|{e.recovery_directory=Some(NativePath::from_path(&root));e})
}
