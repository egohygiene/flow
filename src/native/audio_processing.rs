//! Translation only. Aniflow retains audio math, native state and domain QC.
use super::*;

pub(super) fn supports(profile:&str)->bool {profile=="fixed-audio-gain-v1"}
fn expected(profile:&str)->Result<(&'static str,&'static str,&'static str,&'static str),Error> {
    match profile {
        "fixed-audio-gain-v1"=>Ok(("aniflow.audio-gain-request/v1","gain_audio","audio_derivative","requires_fresh_exact_file_qc")),
        _=>Err(error(Code::Unavailable,"selected audio-processing route unavailable")),
    }
}
fn configuration(request:&Request)->Result<Value,Error> {
    let value:Value=serde_json::from_slice(&command::read_document(&path(&request.configuration)?,65_536)?).map_err(|_|error(Code::InvalidInput,"explicit native audio-processing JSON required"))?;
    let keys=["schema","check","configuration","provider_executable"];
    if value["schema"]!="aniflow.audio-gain-request/v1"||value.as_object().is_none_or(|v|v.len()!=keys.len()||keys.iter().any(|k|!v.contains_key(*k)))
        ||value["provider_executable"].as_str()!=path(&request.provider_executable)?.to_str()||request.inputs.len()!=1 {
        return Err(error(Code::InvalidInput,"one exact final file and closed explicitly selected gain request required"));
    }
    let output=path(&request.run_directory)?;
    if selected_paths(&value,true)?.iter().any(|p|output.starts_with(p)||p.starts_with(&output)){return Err(error(Code::InvalidInput,"processing workspace overlaps native/QC/source/tool evidence"));}
    Ok(value)
}
fn selected_paths(value:&Value,current:bool)->Result<Vec<PathBuf>,Error> {
    fn visit(value:&Value,depth:usize,current:bool,out:&mut Vec<PathBuf>)->Result<(),Error> {
        if depth>32{return Err(error(Code::InvalidInput,"bounded processing configuration depth required"));}
        match value {
            Value::Object(fields)=>for (key,v) in fields {
                if ["run_directory","source_run_directory","qc_directory","gain_run_directory","gain_qc_directory","signal_run_directory","file","executable","provider_executable"].contains(&key.as_str()) {
                    let text=v.as_str().ok_or_else(||error(Code::InvalidInput,"explicit native path string required"))?;
                    let p=path(&NativePath::from_path(Path::new(text)))?;
                    if current&&p.canonicalize().ok().as_deref()!=Some(p.as_path()){return Err(error(Code::Unavailable,"canonical current processing source/evidence/tool path required"));}
                    out.push(p);
                }else{visit(v,depth+1,current,out)?;}
            },
            Value::Array(items)=>for v in items {visit(v,depth+1,current,out)?;},
            _=>{},
        }Ok(())
    }
    let mut result=vec![];visit(value,0,current,&mut result)?;Ok(result)
}
pub(super) fn validate_configuration(request:&Request)->Result<(),Error>{configuration(request).map(|_|())}

#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeFile {locator:PathBuf,sha256:String,byte_count:u64}
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnerPayload {profile:String,request:Value,source_files:Vec<NativeFile>,native:Value}
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnerPlan {schema:String,payload:OwnerPayload,plan_sha256:String}
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Artifact {id:String,purpose:String,file:NativeFile}
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct HandoffPayload {
    plan:OwnerPlan,run_directory:PathBuf,native_intent_sha256:String,artifacts:Vec<Artifact>,report:Value,
    output_state:String,runtime_qualified:bool,creative_approval:String,publication_authority:String,
}
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Handoff {schema:String,payload:HandoffPayload,handoff_sha256:String}
#[derive(Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {schema_version:String,native_plan:OwnerPlan,native_handoff:Handoff,source_pins:Vec<FilePin>,scope:String}
const RECORD_SCHEMA:&str="flow.aniflow-audio-processing-acceptance/v1";
const RECORD_SCOPE:&str="current_owner_readback_at_acceptance; retained_status_does_not_refresh_QC";
fn decode<T:for<'de> Deserialize<'de>>(v:Value)->Result<T,Error>{serde_json::from_value(v).map_err(|_|error(Code::ProviderRefused,"closed Aniflow audio-processing response required"))}
fn native_intent(plan:&OwnerPlan)->Result<&str,Error>{
    plan.payload.native["native_plan"]["plan_sha256"].as_str().filter(|s|release_cohort::is_digest(s)).ok_or_else(||error(Code::ProviderRefused,"native processing intent missing"))
}
fn validate_owner(profile:&str,config:&Value,sources:&[FilePin],owner:&OwnerPlan)->Result<(),Error>{
    let (schema,_,_,_)=expected(profile)?;
    if owner.schema!="aniflow.audio-processing-plan/v1"||owner.payload.profile!=profile||owner.payload.request!=*config||config["schema"]!=schema
        ||owner.plan_sha256!=digest(&owner.payload)?||owner.payload.source_files.len()!=sources.len() {
        return Err(error(Code::ProviderRefused,"owner processing plan request/profile/source membership differs"));
    }
    native_intent(owner)?;
    for (a,b) in owner.payload.source_files.iter().zip(sources){
        if a.locator!=path(&b.path)?||a.sha256!=b.sha256||a.byte_count!=b.byte_count{return Err(error(Code::ProviderRefused,"owner processing plan selected different source bytes/order"));}
    }Ok(())
}
fn validate_handoff(owner:&OwnerPlan,value:&Handoff)->Result<(),Error>{
    let (_,id,purpose,state)=expected(&owner.payload.profile)?;let p=&value.payload;
    if value.schema!="aniflow.audio-processing-handoff/v1"||value.handoff_sha256!=digest(p)?||p.plan!=*owner||p.native_intent_sha256!=native_intent(owner)?
        ||p.runtime_qualified||p.creative_approval!="not_performed"||p.publication_authority!="none"||p.output_state!=state||p.artifacts.len()!=1 {
        return Err(error(Code::ProviderRefused,"native processing handoff intent/state/authority differs"));
    }
    path(&NativePath::from_path(&p.run_directory))?;
    let a=&p.artifacts[0];path(&NativePath::from_path(&a.file.locator))?;
    if a.id!=id||a.purpose!=purpose||!release_cohort::is_digest(&a.file.sha256)||a.file.byte_count==0||!a.file.locator.starts_with(&p.run_directory)
        ||p.report["schema"]!="aniflow.audio-gain-completion/v1"||p.report["file"]!=serde_json::to_value(&a.file).map_err(|_|error(Code::InvalidInput,"gain file encoding"))? {
        return Err(error(Code::ProviderRefused,"native processing artifact/report membership differs"));
    }Ok(())
}
fn args(command:&str,request:&Request)->Result<Vec<OsString>,Error>{
    let mut args=["--output","json","delivery","audio-processing",command,"--request"].into_iter().map(OsString::from).collect::<Vec<_>>();
    args.push(path(&request.configuration)?.into_os_string());Ok(args)
}
pub(super) fn execute(context:&Context<'_>,resume:Option<&Path>)->Result<Receipt,Error>{
    let config=configuration(&context.plan.request)?;let profile=&context.plan.row.profile;
    expected(profile)?;
    let descriptor=aniflow::invoke(context,"contract","release_contract",&["--output".into(),"json".into(),"release-contract".into()])?;
    if descriptor!=read::<Value>(&path(&context.plan.descriptor.path)?)?{return Err(error(Code::Unavailable,"running Aniflow processing contract differs from exact descriptor"));}
    let owner:OwnerPlan=decode(aniflow::invoke(context,"plan","delivery_audio_processing_plan",&args("plan",&context.plan.request)?)?)?;
    validate_owner(profile,&config,&context.plan.sources,&owner)?;
    write(&context.evidence.join("native-plan.json"),&owner)?;
    let mut invocation=args(if resume.is_some(){"resume"}else{"run"},&context.plan.request)?;
    if let Some(root)=resume {
        context.require_native_run(root)?;
        if read::<OwnerPlan>(&context.root.join("attempt-0000/native-plan.json"))?!=owner{return Err(error(Code::RecoveryRequired,"processing recovery intent differs from the original admitted attempt"));}
        invocation.extend(["--run-directory".into(),root.as_os_str().to_owned()]);
    }else{invocation.extend(["--output-directory".into(),context.native_root().into_os_string()]);}
    write(&context.evidence.join("transform-started.json"),&serde_json::json!({"schema_version":"flow.native-transform-attempt/v1","plan_sha256":context.plan.plan_sha256,"native_intent_sha256":native_intent(&owner)?,"resume":resume.map(NativePath::from_path)}))?;
    let command=if resume.is_some(){"delivery_audio_processing_resume"}else{"delivery_audio_processing_run"};
    let outcome:Handoff=decode(aniflow::invoke(context,"run",command,&invocation)?)?;validate_handoff(&owner,&outcome)?;
    let root=&outcome.payload.run_directory;context.require_native_run(root)?;
    if resume.is_some_and(|r|r!=root){return Err(error(Code::ProviderRefused,"processing resumed a different native run"));}
    let mut status_args=args("status",&context.plan.request)?;status_args.extend(["--run-directory".into(),root.as_os_str().to_owned()]);
    let current:Handoff=decode(aniflow::invoke(context,"completion","delivery_audio_processing_status",&status_args)?)?;
    validate_handoff(&owner,&current)?;
    if current!=outcome{return Err(error(Code::RecoveryRequired,"processing owner changed during completion readback"));}
    let outputs=context.observe_artifacts(current.payload.artifacts.iter().map(|a|(a.file.locator.clone(),a.file.sha256.clone(),a.file.byte_count)).collect())?;
    let record=Record {schema_version:RECORD_SCHEMA.into(),native_plan:owner,native_handoff:current.clone(),source_pins:context.plan.sources.clone(),scope:RECORD_SCOPE.into()};
    context.finish(root,&current.payload.native_intent_sha256,&serde_json::to_value(record).map_err(|_|error(Code::InvalidInput,"processing acceptance encoding"))?,outputs)
}
pub(super) fn validate_retained(plan:&Plan,receipt:&Receipt,value:&Value)->Result<(),Error>{
    let record:Record=decode(value.clone())?;
    validate_owner(&plan.row.profile,&configuration(&plan.request)?,&plan.sources,&record.native_plan)?;
    validate_handoff(&record.native_plan,&record.native_handoff)?;
    if record.schema_version!=RECORD_SCHEMA||record.scope!=RECORD_SCOPE||record.source_pins!=plan.sources
        ||record.native_handoff.payload.native_intent_sha256!=receipt.native_intent_sha256||record.native_handoff.payload.run_directory!=path(&receipt.native_run)?
        ||read::<OwnerPlan>(&path(&plan.request.run_directory)?.join("attempt-0000/native-plan.json"))?!=record.native_plan {
        return Err(error(Code::RecoveryRequired,"retained processing request/intent/scope differs"));
    }
    inventory(receipt,value).map(|_|())
}
pub(super) fn inventory(receipt:&Receipt,value:&Value)->Result<Vec<ArtifactDescription>,Error>{
    let record:Record=decode(value.clone())?;let artifacts=&record.native_handoff.payload.artifacts;
    if artifacts.len()!=receipt.artifacts.len(){return Err(error(Code::RecoveryRequired,"processing accepted artifact membership differs"));}
    artifacts.iter().zip(&receipt.artifacts).map(|(a,p)|{
        if a.file.locator!=path(&p.path)?||a.file.sha256!=p.sha256||a.file.byte_count!=p.byte_count{return Err(error(Code::RecoveryRequired,"processing artifact differs from committed byte pin"));}
        Ok(ArtifactDescription {artifact_id:a.id.clone(),purpose:a.purpose.clone(),file:p.clone()})
    }).collect()
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]fn a_rehashed_foreign_source_cannot_replace_the_approved_gain_input() {
        let source=FilePin {path:NativePath::from_path(Path::new("/synthetic/曲 final.mp4")),sha256:"a".repeat(64),byte_count:100};
        let config=serde_json::json!({"schema":"aniflow.audio-gain-request/v1"});
        let mut payload=OwnerPayload {profile:"fixed-audio-gain-v1".into(),request:config.clone(),source_files:vec![NativeFile {locator:path(&source.path).unwrap(),sha256:source.sha256.clone(),byte_count:100}],native:serde_json::json!({"native_plan":{"plan_sha256":"b".repeat(64)}})};
        let make=|p:&OwnerPayload|OwnerPlan {schema:"aniflow.audio-processing-plan/v1".into(),payload:p.clone(),plan_sha256:digest(p).unwrap()};
        assert!(validate_owner("fixed-audio-gain-v1",&config,&[source.clone()],&make(&payload)).is_ok());
        payload.source_files[0].sha256="c".repeat(64);
        assert!(validate_owner("fixed-audio-gain-v1",&config,&[source],&make(&payload)).is_err());
    }
    #[test]fn nested_qc_and_tool_scopes_are_protected_even_below_other_objects() {
        let config=serde_json::json!({"check":{"input":{"qc_directory":"/synthetic/qc"}},"configuration":{"tools":{"ffmpeg":{"executable":"/synthetic/tools/ffmpeg"}}}});
        let paths=selected_paths(&config,false).unwrap();assert_eq!(paths.len(),2);
        assert!(paths.contains(&PathBuf::from("/synthetic/qc")));
        assert!(selected_paths(&serde_json::json!({"check":{"input":{"qc_directory":"../escape"}}}),false).is_err());
    }
}
