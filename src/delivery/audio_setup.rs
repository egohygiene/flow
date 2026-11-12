//! Installed, read-only preparation of exact delivery audio policy checks.
//! Aniflow retains current QC and audio math; approvals are supplied separately.
use super::*;
use std::path::PathBuf;
use crate::release_cohort::{Cohort,Tool};

pub const REQUEST_SCHEMA:&str="flow.audio-check-setup/v1";
pub const PROFILE:&str="audio-policy-check-v1";

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum Choice {
    /// Existing Aniflow current-final-QC request, including the selected policy.
    Check {owner_request:FilePin,#[serde(default)] row_id:Option<String>},
    KeepArchiveMaster {reason:String},
}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {pub payload_id:String,pub destination:Destination,pub choice:Choice}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version:String,pub package_manifest:FilePin,
    pub installation:Option<project::Installation>,pub workspace:NativePath,
    pub selections:Vec<Selection>,pub tool_directories:Vec<NativePath>,
    pub limits:native::Limits,pub timeout_ms:u64,
}
impl Request {
    pub fn from_json_slice(raw:&[u8])->Result<Self,Error>{
        if raw.len()>1_048_576{return Err(error(Code::InvalidInput,"audio setup request exceeds 1 MiB"));}
        let value:Self=serde_json::from_slice(raw).map_err(|_|error(Code::InvalidInput,"closed audio setup request required"))?;
        value.validate()?;Ok(value)
    }
    fn validate(&self)->Result<(),Error>{
        self.limits.validate()?;native::path(&self.workspace)?;
        if self.schema_version!=REQUEST_SCHEMA||self.selections.len()>128||self.tool_directories.len()>32||self.timeout_ms==0||self.timeout_ms>86_400_000 {
            return Err(error(Code::InvalidInput,"bounded audio setup selection required"));
        }
        if self.selections.iter().any(|s|matches!(&s.choice,Choice::Check {..}))&&self.installation.is_none(){return Err(error(Code::Unavailable,"an explicit installed Aniflow selection is required for checks"));}
        Ok(())
    }
}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum ResolvedChoice {
    Check {
        source:FilePin,owner_request:FilePin,row:project::SelectedRow,policy:Value,
        request:native::Request,plan_file:NativePath,approval_file:NativePath,
    },
    KeepArchiveMaster {reason:String},
}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedSelection {pub payload_id:String,pub destination:Destination,pub choice:ResolvedChoice}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preview {
    pub schema_version:String,pub setup_sha256:String,pub package_subject_sha256:String,
    pub package_manifest:FilePin,pub workspace:NativePath,pub selections:Vec<ResolvedSelection>,
    pub publisher_authenticated_at_install:bool,pub provider_invoked:bool,
    pub approvals_generated:bool,pub native_qualified:bool,pub next_step:String,
}
struct Draft {preview:Preview,package:Status,inputs:Vec<FilePin>,configurations:Vec<(PathBuf,Vec<u8>)>,protected:Vec<PathBuf>}
fn overlaps(a:&Path,b:&Path)->bool{a.starts_with(b)||b.starts_with(a)}
fn ensure_separate(workspace:&Path,selected:&Path)->Result<(),Error>{
    if overlaps(workspace,selected){Err(error(Code::InvalidInput,"audio setup workspace overlaps package, source, installation, tools or QC evidence"))}else{Ok(())}
}
fn validate_coverage(payloads:&[Payload],selections:&[Selection])->Result<(),Error>{
    let expected=payloads.iter().filter(|p|audio_readiness::audio_role(p.role)).flat_map(|p|p.destinations.iter().map(move|d|audio_readiness::key(&p.id,*d))).collect::<BTreeSet<_>>();
    let mut selected=BTreeSet::new();
    for selection in selections {
        let id=audio_readiness::key(&selection.payload_id,selection.destination);
        if !expected.contains(&id)||!selected.insert(id){return Err(error(Code::InvalidInput,"unique exact assigned audio payload/destination required"));}
        if let Choice::KeepArchiveMaster {reason}=&selection.choice {
            let payload=payloads.iter().find(|p|p.id==selection.payload_id).ok_or_else(||error(Code::InvalidInput,"selected payload missing"))?;
            if !audio_readiness::archive_skip(payload.role,selection.destination,reason){return Err(error(Code::InvalidInput,"only an explicitly unchanged archival master/original can skip a policy check"));}
        }
    }
    if selected!=expected{return Err(error(Code::InvalidInput,"audio setup must cover every assigned media payload and destination"));}Ok(())
}
fn draft(request:&Request,token:&dyn CancellationSignal)->Result<Draft,Error>{
    request.validate()?;
    let manifest=native::path(&request.package_manifest.path)?;
    if manifest.file_name().and_then(|n|n.to_str())!=Some("manifest.json"){return Err(error(Code::InvalidInput,"exact delivery manifest.json required"));}
    request.package_manifest.current(MAX_DOC,token)?;
    let package=super::status(manifest.parent().ok_or_else(||error(Code::InvalidInput,"package parent required"))?,&request.package_manifest.sha256,token)?;
    validate_coverage(&package.preview.payloads,&request.selections)?;
    let workspace=native::path(&request.workspace)?;
    let parent=workspace.parent().ok_or_else(||error(Code::InvalidInput,"audio setup parent required"))?;
    if parent.canonicalize().ok().as_deref()!=Some(parent)||!fs::metadata(parent).is_ok_and(|m|m.is_dir()){return Err(error(Code::InvalidInput,"existing canonical audio setup parent required"));}
    let mut protected=vec![native::path(&package.current_root)?];
    let installed=request.installation.as_ref().map(|s|distribution::selected_installation(&native::path(&s.root)?,&s.receipt_sha256,token)).transpose()?;
    let cohort=installed.as_ref().map(|i|Cohort::from_json_slice(&super::read(&i.cohort,1_048_576,token)?).map_err(|_|error(Code::InvalidInput,"installed audio cohort invalid"))).transpose()?;
    if installed.as_ref().is_some_and(|i|Some(i.target.as_str())!=native::host_target())||cohort.as_ref().is_some_and(|c|c.flow_version!=env!("CARGO_PKG_VERSION")){return Err(error(Code::Unavailable,"installed audio cohort targets another host or Flow version"));}
    if let Some(i)=&installed{protected.push(native::path(&i.root)?);}
    for p in &request.tool_directories {let p=native::path(p)?;if p.canonicalize().ok().as_deref()!=Some(p.as_path())||!fs::metadata(&p).is_ok_and(|m|m.is_dir()){return Err(error(Code::InvalidInput,"canonical selected tool directory required"));}protected.push(p);}
    let mut inputs=vec![request.package_manifest.clone()];let mut configurations=vec![];let mut selections=vec![];
    for (index,s) in request.selections.iter().enumerate(){
        if token.is_cancelled(){return Err(error(Code::Cancelled,"audio setup observation cancelled"));}
        let choice=match &s.choice {
            Choice::KeepArchiveMaster {reason}=>ResolvedChoice::KeepArchiveMaster {reason:reason.clone()},
            Choice::Check {owner_request,row_id}=>{
                let payload=package.preview.payloads.iter().find(|p|p.id==s.payload_id).ok_or_else(||error(Code::InvalidInput,"audio payload missing"))?;
                let source=package.preview.selected_sources.iter().find(|p|p.payload_id==s.payload_id).ok_or_else(||error(Code::InvalidInput,"original final-file locator absent from delivery package"))?.file.clone();
                if source.sha256!=payload.sha256||source.byte_count!=payload.byte_count{return Err(error(Code::StalePlan,"original final bytes differ from copied delivery bytes"));}
                source.current(request.limits.max_input_bytes,token)?;
                let config=super::read(owner_request,131_072,token)?;
                let value:Value=serde_json::from_slice(&config).map_err(|_|error(Code::InvalidInput,"explicit Aniflow policy-check JSON required"))?;
                let i=installed.as_ref().ok_or_else(||error(Code::Unavailable,"installed audio provider required"))?;
                let row=project::select_row(i,cohort.as_ref().ok_or_else(||error(Code::Unavailable,"audio cohort required"))?,Tool::Aniflow,PROFILE,row_id.as_deref(),"audio_policy_check")?;
                let descriptor:Value=serde_json::from_slice(&super::read(&row.descriptor,MAX_DOC,token)?).map_err(|_|error(Code::InvalidInput,"installed Aniflow descriptor JSON required"))?;
                if descriptor["schema_version"]!="aniflow.release-contract/v1"||descriptor["tool"]!="aniflow"||!descriptor["profiles"].as_array().is_some_and(|profiles|profiles.iter().any(|p|p["id"]==PROFILE&&p["source_mutation"]==false)){return Err(error(Code::Unavailable,"installed Aniflow descriptor does not advertise the selected read-only policy profile"));}
                let name=format!("check-{index:03}");let configuration=workspace.join("configuration").join(format!("{name}.json"));
                let mut planned=native::Request {schema_version:native::REQUEST_SCHEMA.into(),cohort:i.cohort.path.clone(),cohort_sha256:i.cohort.sha256.clone(),row_id:row.row_id.clone(),provider_executable:row.provider.path.clone(),release_descriptor:row.descriptor.path.clone(),inputs:vec![source.path.clone()],configuration:owner_request.path.clone(),run_directory:NativePath::from_path(&workspace.join("native").join(&name)),tool_directories:request.tool_directories.clone(),limits:request.limits.clone(),preparation:None};
                protected.extend(native::audio_policy_protected_paths(&planned)?);
                protected.extend([native::path(&source.path)?,native::path(&owner_request.path)?]);
                planned.configuration=NativePath::from_path(&configuration);
                configurations.push((configuration,config));inputs.extend([source.clone(),owner_request.clone()]);
                ResolvedChoice::Check {source,owner_request:owner_request.clone(),row,policy:value["policy"].clone(),request:planned,plan_file:NativePath::from_path(&workspace.join("plans").join(format!("{name}.json"))),approval_file:NativePath::from_path(&workspace.join("approvals").join(format!("{name}.json")))}
            },
        };
        selections.push(ResolvedSelection {payload_id:s.payload_id.clone(),destination:s.destination,choice});
    }
    for p in &protected{ensure_separate(&workspace,p)?;}
    let preview=Preview {schema_version:"flow.audio-check-setup-preview/v1".into(),setup_sha256:hash(request)?,package_subject_sha256:package.preview.package_subject_sha256.clone(),package_manifest:request.package_manifest.clone(),workspace:request.workspace.clone(),selections,publisher_authenticated_at_install:installed.as_ref().is_some_and(|i|i.publisher_authenticated_at_install),provider_invoked:false,approvals_generated:false,native_qualified:false,next_step:"Explicit preparation creates native plans only; supply exact approvals separately before a fresh owner check. Preview does not recheck Aniflow QC.".into()};
    Ok(Draft {preview,package,inputs,configurations,protected})
}
/// Reads exact local selections only. No provider process, directory or approval.
pub fn preview(request:&Request,token:&dyn CancellationSignal)->Result<Preview,Error>{
    let start=Instant::now();let guard=||token.is_cancelled()||start.elapsed()>=Duration::from_millis(request.timeout_ms);
    Ok(draft(request,&guard)?.preview)
}

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedPlan {
    pub payload_id:String,pub destination:Destination,pub plan_file:FilePin,
    pub plan:native::Plan,pub approval_file:NativePath,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema_version:String,request:Request,preview:Preview,inputs:Vec<FilePin>,
    intent_file:FilePin,configuration_files:Vec<FilePin>,plans:Vec<PreparedPlan>,
}
#[derive(Debug,Serialize)]
pub struct Prepared {
    pub schema_version:String,pub root:NativePath,pub setup_sha256:String,
    pub package_subject_sha256:String,pub record:FilePin,pub plans:Vec<PreparedPlan>,
    pub provider_invoked:bool,pub approvals_generated:bool,pub native_qualified:bool,
    pub evidence_scope:String,
}
fn prepared(record:&Record,file:FilePin)->Prepared {
    Prepared {schema_version:"flow.audio-check-preparation/v1".into(),root:record.request.workspace.clone(),setup_sha256:record.preview.setup_sha256.clone(),package_subject_sha256:record.preview.package_subject_sha256.clone(),record:file,plans:record.plans.clone(),provider_invoked:false,approvals_generated:false,native_qualified:false,evidence_scope:"local configuration/plan identity only; no provider process, current owner QC, release qualification or approval".into()}
}
/// Explicit local configuration writing; does not execute Aniflow or approve it.
/// The last record is a completion marker. Errors retain the partial root.
pub fn prepare(request:&Request,token:&dyn CancellationSignal)->Result<Prepared,Error>{
    let started=Instant::now();let guard=||token.is_cancelled()||started.elapsed()>=Duration::from_millis(request.timeout_ms);
    let selected=draft(request,&guard)?;let root=native::path(&request.workspace)?;
    create(&root)?;
    let action=(||{
        for name in ["configuration","plans","approvals","native"]{create(&root.join(name))?;}
        let intent=root.join("intent.json");write(&intent,&bytes(request)?)?;
        let intent_file=FilePin::observe(&NativePath::from_path(&intent),MAX_DOC,&guard)?;
        let mut configuration_files=vec![];
        for (path,data) in &selected.configurations {
            if guard(){return Err(error(Code::Cancelled,"audio setup preparation cancelled; preserve partial root"));}
            write(path,data)?;configuration_files.push(FilePin::observe(&NativePath::from_path(path),131_072,&guard)?);
        }
        let mut plans=vec![];
        for s in &selected.preview.selections {
            if let ResolvedChoice::Check {request,plan_file,approval_file,..}=&s.choice {
                let plan=native::prepare(request,&guard)?;
                write(&native::path(plan_file)?,&bytes(&plan)?)?;
                plans.push(PreparedPlan {payload_id:s.payload_id.clone(),destination:s.destination,plan_file:FilePin::observe(plan_file,MAX_DOC,&guard)?,plan,approval_file:approval_file.clone()});
            }
        }
        for pin in selected.inputs.iter().chain(configuration_files.iter()).chain(std::iter::once(&intent_file)){pin.current(request.limits.max_input_bytes.max(MAX_DOC),&guard)?;}
        for p in &plans {p.plan.require_current(&guard)?;p.plan_file.current(MAX_DOC,&guard)?;}
        if let Some(i)=&request.installation{distribution::selected_installation(&native::path(&i.root)?,&i.receipt_sha256,&guard)?;}
        let current=super::status(&native::path(&selected.package.current_root)?,&request.package_manifest.sha256,&guard)?;
        if current.preview!=selected.package.preview{return Err(error(Code::StalePlan,"delivery package changed during check preparation"));}
        let record=Record {schema_version:"flow.prepared-audio-check-setup/v1".into(),request:request.clone(),preview:selected.preview,inputs:selected.inputs,intent_file,configuration_files,plans};
        let path=root.join("prepared-audio-checks.json");write(&path,&bytes(&record)?)?;
        fs::File::open(&root).and_then(|f|f.sync_all()).map_err(|_|error(Code::Io,"audio preparation directory synchronization"))?;
        let file=FilePin::observe(&NativePath::from_path(&path),MAX_DOC,&guard)?;Ok(prepared(&record,file))
    })();
    action.map_err(|mut e:Error|{e.recovery_directory=Some(NativePath::from_path(&root));e})
}

#[cfg(test)]mod tests {
    use super::*;
    fn payload()->Payload {Payload {id:"video".into(),role:Role::VideoDelivery,relative:"payload/video.mp4".into(),sha256:"a".repeat(64),byte_count:100,destinations:vec![Destination::Tiktok],source_state:"accepted_native_artifact".into()}}
    #[test]fn coverage_cannot_omit_duplicate_or_exempt_a_platform_copy(){
        let p=payload();assert!(validate_coverage(&[p.clone()],&[]).is_err());
        let s=Selection {payload_id:p.id.clone(),destination:Destination::Tiktok,choice:Choice::KeepArchiveMaster {reason:"do not process".into()}};
        assert!(validate_coverage(&[p.clone()],&[s]).is_err());
        let mut master=p;master.role=Role::CleanMaster;master.destinations=vec![Destination::Archive];
        let s=Selection {payload_id:master.id.clone(),destination:Destination::Archive,choice:Choice::KeepArchiveMaster {reason:"Preserve selected approved master".into()}};
        validate_coverage(&[master.clone()],&[s.clone()]).unwrap();assert!(validate_coverage(&[master],&[s.clone(),s]).is_err());
    }
    #[test]fn scope_refuses_both_ancestor_directions_without_creating_paths(){
        let root=Path::new("/synthetic/QC 音");assert!(ensure_separate(&root.join("setup"),root).is_err());assert!(ensure_separate(root,&root.join("checks")).is_err());assert!(ensure_separate(Path::new("/synthetic/setup"),root).is_ok());
    }
}
