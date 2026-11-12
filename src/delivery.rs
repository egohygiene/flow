//! Private local delivery packages over exact accepted artifacts. This module
//! copies bytes and records supplied review declarations; it never transforms
//! media, validates current platform requirements, signs or publishes anything.
pub mod audio_readiness;
use std::{collections::{BTreeMap,BTreeSet},fs,path::Path,time::{Duration,Instant}};
use serde::{Serialize,Deserialize};
use serde_json::{json,Value};
use crate::{CancellationSignal,command::{self,Code,Error},distribution,native::{self,FilePin},optiflow::NativePath,pilot,project::{self,Destination},release_cohort};
const MAX_DOC:u64=16_777_216;
fn error(code:Code,text:&str)->Error{Error{code,message:text.into(),recovery_directory:None}}
fn hash<T:Serialize>(v:&T)->Result<String,Error>{let v=serde_json::to_value(v).map_err(|_|error(Code::InvalidInput,"delivery encoding"))?;Ok(release_cohort::sha256(&serde_json::to_vec(&v).map_err(|_|error(Code::InvalidInput,"delivery encoding"))?))}
fn bytes<T:Serialize>(v:&T)->Result<Vec<u8>,Error>{let mut b=serde_json::to_vec_pretty(v).map_err(|_|error(Code::InvalidInput,"delivery document encoding"))?;b.push(b'\n');if b.len() as u64>MAX_DOC{return Err(error(Code::InvalidInput,"delivery document bound"));}Ok(b)}
fn read(pin:&FilePin,maximum:u64,token:&dyn CancellationSignal)->Result<Vec<u8>,Error>{pin.current(maximum,token)?;let b=command::read_document(&native::path(&pin.path)?,maximum)?;if release_cohort::sha256(&b)!=pin.sha256{return Err(error(Code::StalePlan,"selected delivery document changed"));}Ok(b)}
fn object<T:for<'a>Deserialize<'a>>(pin:&FilePin,token:&dyn CancellationSignal)->Result<T,Error>{serde_json::from_slice(&read(pin,MAX_DOC,token)?).map_err(|_|error(Code::InvalidInput,"closed delivery document required"))}
fn token(s:&str)->bool{!s.is_empty()&&s.len()<=96&&s.bytes().all(|b|b.is_ascii_alphanumeric()||b"._-".contains(&b))&&s!="."&&s!=".."}
fn filename(s:&str)->bool{!s.trim().is_empty()&&s.len()<=512&&crate::publisher_report::safe_member(s)&&!s.contains('/')}
fn create(path:&Path)->Result<(),Error>{let parent=path.parent().ok_or_else(||error(Code::InvalidInput,"delivery parent required"))?;if parent.canonicalize().ok().as_deref()!=Some(parent)||!fs::metadata(parent).is_ok_and(|m|m.is_dir()){return Err(error(Code::InvalidInput,"existing canonical delivery parent required"));}let mut b=fs::DirBuilder::new();#[cfg(unix)]{use std::os::unix::fs::DirBuilderExt;b.mode(0o700);}b.create(path).map_err(|_|error(Code::RecoveryRequired,"fresh delivery directory required"))}
fn write(path:&Path,data:&[u8])->Result<(),Error>{use std::io::Write;let mut o=fs::OpenOptions::new();o.create_new(true).write(true);#[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;o.mode(0o600).custom_flags(nix::libc::O_NOFOLLOW|nix::libc::O_CLOEXEC);}let mut f=o.open(path).map_err(|_|error(Code::RecoveryRequired,"new delivery member required"))?;f.write_all(data).and_then(|_|f.sync_all()).map_err(|_|error(Code::Io,"delivery member incomplete; preserve partial package"))}
fn parents(root:&Path,relative:&str)->Result<(),Error>{let mut at=root.to_path_buf();let parent=Path::new(relative).parent().ok_or_else(||error(Code::InvalidInput,"relative delivery member required"))?;for component in parent.components(){let std::path::Component::Normal(name)=component else{return Err(error(Code::InvalidInput,"delivery traversal refused"));};at.push(name);if !at.exists(){create(&at)?;}if at.canonicalize().ok().as_deref()!=Some(at.as_path())||!fs::metadata(&at).is_ok_and(|m|m.is_dir()){return Err(error(Code::RecoveryRequired,"delivery parent identity changed"));}}Ok(())}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Role{CleanMaster,VideoDelivery,OriginalAudio,AudioDerivative,Lyrics,Artwork,SupportingDocument}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum Source{Native{stage:String,artifact_id:String},OriginalAudio,ManualSrt,External{file:FilePin}}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection{pub id:String,pub role:Role,pub source:Source,pub file_name:String,pub destinations:Vec<Destination>}
#[derive(Debug,Clone,Default,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperatorMetadata{pub creators:Vec<String>,pub description:Option<String>,pub tags:Vec<String>,pub destination_fields:Vec<DestinationFields>}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DestinationFields{pub destination:Destination,pub fields:BTreeMap<String,String>}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits{pub timeout_ms:u64,pub maximum_payload_bytes:u64,pub maximum_total_bytes:u64}
impl Limits{fn validate(&self)->Result<(),Error>{if self.timeout_ms==0||self.timeout_ms>86_400_000||self.maximum_payload_bytes==0||self.maximum_payload_bytes>68_719_476_736||self.maximum_total_bytes==0||self.maximum_total_bytes>1_099_511_627_776{return Err(error(Code::InvalidInput,"bounded delivery copy limits required"));}Ok(())}}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request{pub schema_version:String,pub project_record:FilePin,pub handoff:FilePin,pub payloads:Vec<Selection>,pub metadata:OperatorMetadata,pub reviews:Vec<FilePin>,pub evidence_attachments:Vec<FilePin>,pub output_directory:NativePath,pub limits:Limits}
impl Request{
    pub fn from_json_slice(bytes:&[u8])->Result<Self,Error>{if bytes.len()>1_048_576{return Err(error(Code::InvalidInput,"delivery request exceeds 1 MiB"));}let r:Self=serde_json::from_slice(bytes).map_err(|_|error(Code::InvalidInput,"closed delivery request required"))?;r.validate()?;Ok(r)}
    fn validate(&self)->Result<(),Error>{self.limits.validate()?;native::path(&self.output_directory)?;
        if self.schema_version!="flow.delivery-request/v1"||self.payloads.is_empty()||self.payloads.len()>128||self.reviews.len()>5||self.evidence_attachments.len()>32||self.metadata.creators.len()>16||self.metadata.tags.len()>64||self.metadata.destination_fields.len()>4||self.metadata.description.as_ref().is_some_and(|s|s.len()>32_768){return Err(error(Code::InvalidInput,"delivery request identity/inventory bound"));}
        if self.metadata.creators.iter().any(|s|s.trim().is_empty()||s.len()>512||s.chars().any(char::is_control))||self.metadata.tags.iter().any(|s|s.is_empty()||s.len()>128||s.chars().any(char::is_control)){return Err(error(Code::InvalidInput,"bounded explicit creator/tag metadata required"));}
        let mut ids=BTreeSet::new();for p in &self.payloads{if !token(&p.id)||!filename(&p.file_name)||!ids.insert(&p.id)||p.destinations.len()>4||p.destinations.iter().map(|d|format!("{d:?}")).collect::<BTreeSet<_>>().len()!=p.destinations.len(){return Err(error(Code::InvalidInput,"unique safe delivery identity/name/destinations required"));}}
        let mut destinations=BTreeSet::new();for d in &self.metadata.destination_fields{if !destinations.insert(format!("{:?}",d.destination))||d.fields.len()>32||d.fields.iter().any(|(k,v)|k.is_empty()||k.len()>128||k.chars().any(char::is_control)||v.len()>4096){return Err(error(Code::InvalidInput,"bounded unique destination metadata required"));}}Ok(())
    }
}
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum ReviewScope{Words,Timing,Visual,Audio,FinalPackage}
const SCOPES:[ReviewScope;5]=[ReviewScope::Words,ReviewScope::Timing,ReviewScope::Visual,ReviewScope::Audio,ReviewScope::FinalPackage];
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Decision{Accept,Reject,NeedsChanges}
/// A supplied local operator declaration, not an authenticated identity or a
/// machine-generated assertion that human review occurred.
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review{pub schema_version:String,pub package_subject_sha256:String,pub scope:ReviewScope,pub decision:Decision,pub reviewed_by:String,pub reviewed_at:String,pub notes:String}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewSummary{pub state:String,pub pending:Vec<ReviewScope>,pub changes_requested:Vec<ReviewScope>,pub identity_authenticated:bool}
fn reviews(values:&[Review],subject:&str)->Result<ReviewSummary,Error>{
    let mut scopes=BTreeMap::new();for r in values{if r.schema_version!="flow.delivery-review/v1"||r.package_subject_sha256!=subject||r.reviewed_by.trim().is_empty()||r.reviewed_by.len()>512||r.reviewed_by.chars().any(char::is_control)||r.reviewed_at.is_empty()||r.reviewed_at.len()>128||r.notes.len()>8192||scopes.insert(r.scope,r.decision).is_some(){return Err(error(Code::StalePlan,"supplied review is stale, ambiguous or not bound to this exact payload/metadata package"));}}
    let pending=SCOPES.into_iter().filter(|s|!scopes.contains_key(s)).collect::<Vec<_>>();let changes_requested=scopes.iter().filter(|(_,d)|**d!=Decision::Accept).map(|(s,_)|*s).collect::<Vec<_>>();
    Ok(ReviewSummary{state:if !changes_requested.is_empty(){"changes_requested"}else if !pending.is_empty(){"review_pending"}else{"operator_declarations_supplied"}.into(),pending,changes_requested,identity_authenticated:false})
}
#[derive(Debug,Clone,Serialize)]
pub struct Available{pub stage:String,pub artifact_id:String,pub purpose:String,pub file:FilePin}
#[derive(Debug,Clone,Serialize)]
pub struct Catalog{pub schema_version:String,pub plan_sha256:String,pub metadata:pilot::Metadata,pub requested_destinations:Vec<Destination>,pub artifacts:Vec<Available>,pub original_audio:Option<FilePin>,pub manual_srt:FilePin,pub provider_history:String,pub native_release_admission_reverified:bool}
struct Context{catalog:Catalog,plan:pilot::Plan,evidence:Vec<(String,FilePin)>}
fn context(project_record:&FilePin,handoff:&FilePin,token:&dyn CancellationSignal)->Result<Context,Error>{
    let record_path=native::path(&project_record.path)?;if record_path.file_name().and_then(|s|s.to_str())!=Some("prepared-project.json"){return Err(error(Code::InvalidInput,"exact prepared-project.json selection required"));}
    project_record.current(MAX_DOC,token)?;let prepared=project::status(record_path.parent().ok_or_else(||error(Code::InvalidInput,"prepared project parent required"))?,&project_record.sha256,token)?;
    let plan=pilot::Plan::from_json_slice(&read(&prepared.pilot_plan,MAX_DOC,token)?)?;plan.require_current(token)?;
    let raw=read(handoff,MAX_DOC,token)?;let schema:Value=serde_json::from_slice(&raw).map_err(|_|error(Code::InvalidInput,"pilot handoff JSON required"))?;
    let mut evidence=vec![("evidence/project-record.json".into(),project_record.clone()),("evidence/pilot-handoff.json".into(),handoff.clone()),("evidence/manual-source.srt".into(),plan.request.manual_srt.clone())];
    let (metadata,manual,audio,plan_sha,stages,history)=if schema["schema_version"]=="flow.publishing-pilot-handoff/v1"{
        let h:pilot::Handoff=serde_json::from_slice(&raw).map_err(|_|error(Code::InvalidInput,"closed candidate pilot handoff required"))?;
        if !h.review_pending||h.upload_authority||h.release_qualified||native::path(&handoff.path)?!=native::path(&plan.request.run_directory)?.join("handoff.json"){return Err(error(Code::InvalidInput,"candidate handoff scope/path differs"));}
        (h.metadata,h.manual_srt,h.audio_source,h.plan_sha256,h.stages.into_iter().map(|s|(s.role,s.receipt,None)).collect::<Vec<_>>(),"source_candidate_handoff")
    }else if schema["schema_version"]=="flow.released-pilot-handoff/v1"{
        let h:pilot::released::Handoff=serde_json::from_slice(&raw).map_err(|_|error(Code::InvalidInput,"closed released pilot handoff required"))?;
        if !h.creative_review_pending||h.upload_authority||!h.provider_release_qualified||native::path(&handoff.path)?!=native::path(&plan.request.run_directory)?.join("released-handoff.json"){return Err(error(Code::InvalidInput,"released handoff scope/path differs"));}
        (h.metadata,h.manual_srt,h.audio_source,h.plan_sha256,h.stages.into_iter().map(|s|(s.role,s.native_receipt,Some((s.admission,s.release_record)))).collect::<Vec<_>>(),"released_provider_handoff_historical")
    }else{return Err(error(Code::Unavailable,"supported exact pilot handoff required"));};
    if plan_sha!=plan.plan_sha256||metadata!=plan.request.metadata||manual!=plan.request.manual_srt||audio!=plan.request.audio_source||stages.len()!=plan.stages.len(){return Err(error(Code::StalePlan,"project and pilot handoff identity differ"));}
    let mut artifacts=vec![];
    for (index,(stage,(role,receipt,released))) in plan.stages.iter().zip(stages).enumerate(){
        if role!=stage.role{return Err(error(Code::InvalidInput,"pilot stage role differs"));}let root=native::path(&stage.plan.request.run_directory)?;
        let current=native::observe_receipt(&root,&stage.plan.plan_sha256,token)?.receipt.ok_or_else(||error(Code::RecoveryRequired,"complete native acceptance required before delivery"))?;
        if current!=receipt{return Err(error(Code::StalePlan,"retained pilot receipt differs from actual native acceptance"));}
        evidence.push((format!("evidence/native/{index:02}-receipt.json"),FilePin::observe(&NativePath::from_path(&root.join("accepted.json")),MAX_DOC,token)?));
        let acceptance=FilePin::observe(&receipt.acceptance_file,MAX_DOC,token)?;if acceptance.sha256!=receipt.acceptance_sha256{return Err(error(Code::StalePlan,"native acceptance snapshot changed"));}evidence.push((format!("evidence/native/{index:02}-acceptance.json"),acceptance));
        if let Some((admission,record))=released{if native::path(&record.path)?!=root.join("released-acceptance.json"){return Err(error(Code::InvalidInput,"exact native release record required"));}let data:Value=object(&record,token)?;if data["schema_version"]!="flow.released-native-acceptance/v1"||data["native"]!=serde_json::to_value(&receipt).map_err(|_|error(Code::InvalidInput,"native receipt encoding"))?||data["admission"]!=serde_json::to_value(admission).map_err(|_|error(Code::InvalidInput,"admission encoding"))?{return Err(error(Code::StalePlan,"historical release record differs"));}evidence.push((format!("evidence/native/{index:02}-release.json"),record));}
        artifacts.extend(native::artifact_inventory(&root,&stage.plan.plan_sha256,token)?.into_iter().map(|a|Available{stage:role.clone(),artifact_id:a.artifact_id,purpose:a.purpose,file:a.file}));
    }
    handoff.current(MAX_DOC,token)?;
    Ok(Context{catalog:Catalog{schema_version:"flow.delivery-catalog/v1".into(),plan_sha256:plan.plan_sha256.clone(),metadata,requested_destinations:prepared.requested_destinations,artifacts,original_audio:audio,manual_srt:manual,provider_history:history.into(),native_release_admission_reverified:false},plan,evidence})
}
pub fn catalog(project_record:&FilePin,handoff:&FilePin,token:&dyn CancellationSignal)->Result<Catalog,Error>{Ok(context(project_record,handoff,token)?.catalog)}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Payload{pub id:String,pub role:Role,pub relative:String,pub sha256:String,pub byte_count:u64,pub destinations:Vec<Destination>,pub source_state:String}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedSource{pub payload_id:String,pub file:FilePin,pub source_state:String}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preview{pub schema_version:String,pub package_subject_sha256:String,pub plan_sha256:String,pub metadata_sha256:String,pub requested_destinations:Vec<Destination>,pub payloads:Vec<Payload>,pub selected_sources:Vec<SelectedSource>,pub reviews:ReviewSummary,pub missing_destination_payloads:Vec<Destination>,pub missing_metadata:Vec<String>,pub provider_history:String,pub publishing_authority:bool,pub current_platform_compliance_verified:bool}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedFile{pub relative:String,pub sha256:String,pub byte_count:u64}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest{schema_version:String,original_root:NativePath,visibility:String,request_sha256:String,preview:Preview,files:Vec<OwnedFile>,review_files:Vec<String>,proof_state:String,evidence_scope:String}
enum Member{Copy{relative:String,source:FilePin},Generated{relative:String,data:Vec<u8>}}
impl Member{fn owned(&self)->OwnedFile{match self{Self::Copy{relative,source}=>OwnedFile{relative:relative.clone(),sha256:source.sha256.clone(),byte_count:source.byte_count},Self::Generated{relative,data}=>OwnedFile{relative:relative.clone(),sha256:release_cohort::sha256(data),byte_count:data.len() as u64}}}}
struct Build{preview:Preview,members:Vec<Member>,manifest_bytes:Vec<u8>}
fn public_index(preview:&Preview)->Value{json!({"schema_version":"flow.delivery-index/v1","package_subject_sha256":preview.package_subject_sha256,"metadata":"metadata.json","metadata_sha256":preview.metadata_sha256,"requested_destinations":preview.requested_destinations,"payloads":preview.payloads,"review_state":preview.reviews,"missing_destination_payloads":preview.missing_destination_payloads,"missing_metadata":preview.missing_metadata,"platform_preflight":"required_from_domain_owner; not_performed_by_packaging","publishing_authority":false})}
fn build(request:&Request,token:&dyn CancellationSignal)->Result<Build,Error>{
    request.validate()?;let context=context(&request.project_record,&request.handoff,token)?;let mut members=vec![];let mut payloads=vec![];let mut selected_sources=vec![];
    for selected in &request.payloads{
        if selected.destinations.iter().any(|d|!context.catalog.requested_destinations.contains(d)){return Err(error(Code::InvalidInput,"payload destination absent from selected project"));}
        let (source,state)=match &selected.source{
            Source::Native{stage,artifact_id}=>{let a=context.catalog.artifacts.iter().find(|a|&a.stage==stage&& &a.artifact_id==artifact_id).ok_or_else(||error(Code::Unavailable,"exact accepted native artifact ID required"))?;
                let permitted=match selected.role{Role::CleanMaster=>stage=="video"&&a.purpose=="clean_archival_master",Role::VideoDelivery=>stage=="video"&&a.purpose=="destination_copy",Role::AudioDerivative=>stage=="audio",Role::Lyrics=>stage=="timed_text",_=>false};if !permitted{return Err(error(Code::InvalidInput,"payload role differs from accepted owner artifact/stage; no master inferred from filename"));}(a.file.clone(),"accepted_native_artifact")},
            Source::OriginalAudio=>{if selected.role!=Role::OriginalAudio{return Err(error(Code::InvalidInput,"original audio role required"));}(context.catalog.original_audio.clone().ok_or_else(||error(Code::Unavailable,"project has no selected original audio"))?,"unchanged_selected_original_audio")},
            Source::ManualSrt=>{if selected.role!=Role::Lyrics{return Err(error(Code::InvalidInput,"manual SRT requires lyric role"));}(context.catalog.manual_srt.clone(),"unchanged_manual_srt")},
            Source::External{file}=>{if !matches!(selected.role,Role::Artwork|Role::SupportingDocument){return Err(error(Code::InvalidInput,"external files cannot claim accepted native/master roles"));}file.current(request.limits.maximum_payload_bytes,token)?;(file.clone(),"explicit_external_asset; domain_preflight_not_performed")},
        };
        if selected.role==Role::CleanMaster&&selected.destinations.iter().any(|d|*d!=Destination::Archive){return Err(error(Code::InvalidInput,"clean archival master must stay in the archival selection"));}
        if source.byte_count>request.limits.maximum_payload_bytes{return Err(error(Code::InvalidInput,"selected delivery payload exceeds byte budget"));}
        let relative=format!("{}/{}/{}",if selected.role==Role::CleanMaster{"archive"}else{"deliver"},selected.id,selected.file_name);
        selected_sources.push(SelectedSource{payload_id:selected.id.clone(),file:source.clone(),source_state:state.into()});
        payloads.push(Payload{id:selected.id.clone(),role:selected.role,relative:relative.clone(),sha256:source.sha256.clone(),byte_count:source.byte_count,destinations:selected.destinations.clone(),source_state:state.into()});members.push(Member::Copy{relative,source});
    }
    if !payloads.iter().any(|p|p.role==Role::CleanMaster){return Err(error(Code::InvalidInput,"retain an actual accepted clean archival master in the local package"));}
    if request.metadata.destination_fields.iter().any(|m|!context.catalog.requested_destinations.contains(&m.destination)){return Err(error(Code::InvalidInput,"metadata destination absent from project"));}
    let metadata=bytes(&json!({"schema_version":"flow.delivery-metadata/v1","work":context.catalog.metadata,"operator_supplied":request.metadata,"authorship_or_ownership_authenticated":false}))?;
    let metadata_sha256=release_cohort::sha256(&metadata);
    let subject=hash(&("flow.delivery-subject/v1",&request.project_record.sha256,&request.handoff.sha256,&context.catalog.plan_sha256,&metadata_sha256,&context.catalog.requested_destinations,&payloads))?;
    let review_values=request.reviews.iter().map(|r|object::<Review>(r,token)).collect::<Result<Vec<_>,_>>()?;let review_summary=reviews(&review_values,&subject)?;
    let missing_destination_payloads=context.catalog.requested_destinations.iter().filter(|d|!payloads.iter().any(|p|p.destinations.contains(d)||(**d==Destination::Archive&&p.role==Role::CleanMaster))).copied().collect();
    let preview=Preview{schema_version:"flow.delivery-preview/v1".into(),package_subject_sha256:subject,plan_sha256:context.catalog.plan_sha256,metadata_sha256,requested_destinations:context.catalog.requested_destinations,payloads,selected_sources,reviews:review_summary,missing_destination_payloads,missing_metadata:if request.metadata.creators.is_empty(){vec!["creators".into()]}else{vec![]},provider_history:context.catalog.provider_history,publishing_authority:false,current_platform_compliance_verified:false};
    members.extend(context.evidence.into_iter().map(|(relative,source)|Member::Copy{relative,source}));
    let mut review_files=vec![];for (i,r) in request.reviews.iter().enumerate(){let relative=format!("evidence/reviews/{i:02}.json");review_files.push(relative.clone());members.push(Member::Copy{relative,source:r.clone()});}
    for (i,pin) in request.evidence_attachments.iter().enumerate(){pin.current(MAX_DOC,token)?;members.push(Member::Copy{relative:format!("evidence/attachments/{i:02}.data"),source:pin.clone()});}
    let intent=bytes(request)?;let request_sha256=release_cohort::sha256(&intent);
    members.extend([Member::Generated{relative:"intent.json".into(),data:intent},Member::Generated{relative:"metadata.json".into(),data:metadata},Member::Generated{relative:"delivery-index.json".into(),data:bytes(&public_index(&preview))?}]);
    let root=native::path(&request.output_directory)?;
    for stage in &context.plan.stages{let native_root=native::path(&stage.plan.request.run_directory)?;if root.starts_with(&native_root)||native_root.starts_with(&root){return Err(error(Code::InvalidInput,"delivery package overlaps a retained native run"));}}
    let pilot_root=native::path(&context.plan.request.run_directory)?;if root.starts_with(&pilot_root)||pilot_root.starts_with(&root){return Err(error(Code::InvalidInput,"delivery package overlaps the retained pilot run"));}
    for m in &members{if let Member::Copy{source,..}=m{let input=native::path(&source.path)?;if root.starts_with(&input)||input.starts_with(&root){return Err(error(Code::InvalidInput,"delivery output overlaps an original or evidence file"));}}}
    let files=members.iter().map(Member::owned).collect::<Vec<_>>();let mut names=BTreeSet::new();if files.len()>512||files.iter().any(|f|!crate::publisher_report::safe_member(&f.relative)||!names.insert(f.relative.clone())){return Err(error(Code::InvalidInput,"unique bounded delivery member namespace required"));}
    let manifest=Manifest{schema_version:"flow.delivery-package/v1".into(),original_root:request.output_directory.clone(),visibility:"private_review_package".into(),request_sha256,preview:preview.clone(),files,review_files,proof_state:"not_asserted".into(),evidence_scope:"selected native receipts/domain acceptance and supplied records; original native workspaces and external signatures not reproduced or reverified".into()};let manifest_bytes=bytes(&manifest)?;
    let total=manifest.files.iter().try_fold(manifest_bytes.len() as u64,|n,f|n.checked_add(f.byte_count)).ok_or_else(||error(Code::InvalidInput,"delivery total byte overflow"))?;if total>request.limits.maximum_total_bytes{return Err(error(Code::InvalidInput,"complete payload/evidence/metadata/manifest set exceeds delivery byte budget"));}
    Ok(Build{preview,members,manifest_bytes})
}
pub fn preview(request:&Request,token:&dyn CancellationSignal)->Result<Preview,Error>{let started=Instant::now();request.limits.validate()?;let guard=||token.is_cancelled()||started.elapsed()>=Duration::from_millis(request.limits.timeout_ms);Ok(build(request,&guard)?.preview)}
#[derive(Debug,Serialize)]
pub struct Status{pub schema_version:String,pub current_root:NativePath,pub original_root:NativePath,pub manifest:FilePin,pub preview:Preview,pub current_copied_bytes:bool,pub native_release_admission_reverified:bool}
pub fn prepare(request:&Request,token:&dyn CancellationSignal)->Result<Status,Error>{
    let started=Instant::now();request.limits.validate()?;let guard=||token.is_cancelled()||started.elapsed()>=Duration::from_millis(request.limits.timeout_ms);let selected=build(request,&guard)?;let root=native::path(&request.output_directory)?;create(&root)?;
    let action=(||{
        for member in &selected.members{if guard(){return Err(error(Code::Cancelled,"delivery copy cancelled; retain partial directory"));}let expected=member.owned();parents(&root,&expected.relative)?;let file=root.join(&expected.relative);
            match member{Member::Copy{source,..}=>{distribution::copy_bounded(source,&file,false,request.limits.maximum_payload_bytes.max(MAX_DOC),&guard)?;},Member::Generated{data,..}=>write(&file,data)?;}
            FilePin{path:NativePath::from_path(&file),sha256:expected.sha256,byte_count:expected.byte_count}.current(request.limits.maximum_payload_bytes.max(MAX_DOC),&guard)?;
        }
        let current=build(request,&guard)?;if current.preview!=selected.preview||current.manifest_bytes!=selected.manifest_bytes{return Err(error(Code::StalePlan,"delivery source/native/review context changed during copy"));}
        write(&root.join("manifest.json"),&selected.manifest_bytes)?;fs::File::open(&root).and_then(|f|f.sync_all()).map_err(|_|error(Code::Io,"delivery directory synchronization"))?;
        status(&root,&release_cohort::sha256(&selected.manifest_bytes),&guard)
    })();action.map_err(|mut e:Error|{e.recovery_directory=Some(NativePath::from_path(&root));e})
}
fn membership(root:&Path)->Result<BTreeSet<String>,Error>{
    fn visit(root:&Path,at:&Path,depth:usize,result:&mut BTreeSet<String>)->Result<(),Error>{if depth>5||result.len()>1024{return Err(error(Code::RecoveryRequired,"delivery membership bound"));}for e in fs::read_dir(at).map_err(|_|error(Code::Io,"delivery directory unavailable"))?{let path=e.map_err(|_|error(Code::Io,"delivery member unavailable"))?.path();let m=fs::symlink_metadata(&path).map_err(|_|error(Code::Io,"delivery member metadata unavailable"))?;let relative=path.strip_prefix(root).map_err(|_|error(Code::RecoveryRequired,"delivery member escaped"))?.to_str().ok_or_else(||error(Code::RecoveryRequired,"owned delivery member encoding changed"))?.replace(std::path::MAIN_SEPARATOR,"/");if m.file_type().is_symlink(){return Err(error(Code::RecoveryRequired,"delivery symlink refused"));}if m.is_dir(){result.insert(relative+"/");visit(root,&path,depth+1,result)?;}else if m.is_file(){result.insert(relative);}else{return Err(error(Code::RecoveryRequired,"special delivery member refused"));}}Ok(())}
    let mut result=BTreeSet::new();visit(root,root,0,&mut result)?;Ok(result)
}
/// Portable byte/membership readback of the retained package, independent of
/// original host paths. This does not rerun domain or signature verification.
pub fn status(root:&Path,expected_manifest_sha256:&str,token:&dyn CancellationSignal)->Result<Status,Error>{
    if root.canonicalize().ok().as_deref()!=Some(root){return Err(error(Code::InvalidInput,"canonical current package root required"));}
    let manifest_pin=FilePin::observe(&NativePath::from_path(&root.join("manifest.json")),MAX_DOC,token)?;if manifest_pin.sha256!=expected_manifest_sha256{return Err(error(Code::StalePlan,"delivery manifest pin changed"));}
    let manifest:Manifest=object(&manifest_pin,token)?;
    if manifest.schema_version!="flow.delivery-package/v1"||manifest.visibility!="private_review_package"||manifest.proof_state!="not_asserted"||manifest.preview.publishing_authority||manifest.preview.current_platform_compliance_verified||manifest.files.len()>512||manifest.review_files.len()>5{return Err(error(Code::RecoveryRequired,"delivery evidence/authority scope changed"));}
    let intent_file=manifest.files.iter().find(|f|f.relative=="intent.json"&&f.sha256==manifest.request_sha256).ok_or_else(||error(Code::RecoveryRequired,"delivery intent pin absent"))?;
    let raw=command::read_document(&root.join("intent.json"),MAX_DOC)?;if release_cohort::sha256(&raw)!=intent_file.sha256{return Err(error(Code::StalePlan,"delivery intent changed"));}
    let request:Request=serde_json::from_slice(&raw).map_err(|_|error(Code::RecoveryRequired,"retained delivery intent malformed"))?;request.limits.validate()?;
    let started=Instant::now();let guard=||token.is_cancelled()||started.elapsed()>=Duration::from_millis(request.limits.timeout_ms);
    let mut expected=BTreeSet::from(["manifest.json".into()]);let mut total=manifest_pin.byte_count;
    for f in &manifest.files{if !crate::publisher_report::safe_member(&f.relative)||!expected.insert(f.relative.clone()){return Err(error(Code::RecoveryRequired,"delivery member namespace changed"));}FilePin{path:NativePath::from_path(&root.join(&f.relative)),sha256:f.sha256.clone(),byte_count:f.byte_count}.current(request.limits.maximum_payload_bytes.max(MAX_DOC),&guard)?;total=total.checked_add(f.byte_count).ok_or_else(||error(Code::RecoveryRequired,"delivery byte overflow"))?;let mut parent=Path::new(&f.relative).parent();while let Some(p)=parent{if p.as_os_str().is_empty(){break;}expected.insert(p.to_string_lossy().replace(std::path::MAIN_SEPARATOR,"/")+"/");parent=p.parent();}}
    if total>request.limits.maximum_total_bytes||membership(root)?!=expected{return Err(error(Code::RecoveryRequired,"delivery has changed, missing, oversized or unknown members; preserve them"));}
    if !manifest.files.iter().any(|f|f.relative=="metadata.json"&&f.sha256==manifest.preview.metadata_sha256)||manifest.preview.payloads.iter().any(|p|!manifest.files.iter().any(|f|f.relative==p.relative&&f.sha256==p.sha256&&f.byte_count==p.byte_count)){return Err(error(Code::RecoveryRequired,"delivery payload/metadata ledger differs"));}
    let index:Value=serde_json::from_slice(&command::read_document(&root.join("delivery-index.json"),MAX_DOC)?).map_err(|_|error(Code::RecoveryRequired,"delivery index malformed"))?;if index!=public_index(&manifest.preview){return Err(error(Code::RecoveryRequired,"public delivery index differs from private pinned ledger"));}
    let review_values=manifest.review_files.iter().map(|name|{if !manifest.files.iter().any(|f|&f.relative==name){return Err(error(Code::RecoveryRequired,"review outside owned delivery set"));}serde_json::from_slice::<Review>(&command::read_document(&root.join(name),MAX_DOC)?).map_err(|_|error(Code::RecoveryRequired,"retained review malformed"))}).collect::<Result<Vec<_>,_>>()?;
    if reviews(&review_values,&manifest.preview.package_subject_sha256)?!=manifest.preview.reviews{return Err(error(Code::RecoveryRequired,"retained review summary differs"));}manifest_pin.current(MAX_DOC,&guard)?;
    Ok(Status{schema_version:"flow.delivery-package-status/v1".into(),current_root:NativePath::from_path(root),original_root:manifest.original_root,manifest:manifest_pin,preview:manifest.preview,current_copied_bytes:true,native_release_admission_reverified:false})
}

#[cfg(test)]
mod tests;
