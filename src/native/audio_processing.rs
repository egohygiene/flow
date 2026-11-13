//! Translation only. Aniflow retains audio math, native state and domain QC.
use super::*;

pub(super) fn supports(profile:&str)->bool {matches!(profile,"fixed-audio-gain-v1"|"final-lossless-audio-qc-v1"|"final-video-audio-qc-v1"|"gain-video-v1")}
fn expected(profile:&str)->Result<(&'static str,&'static str,&'static str,&'static str),Error> {
    match profile {
        "fixed-audio-gain-v1"=>Ok(("aniflow.audio-gain-request/v1","gain_audio","audio_derivative","requires_fresh_exact_file_qc")),
        "final-lossless-audio-qc-v1"=>Ok(("aniflow.resumable-final-audio-qc-request/v1","final_audio_qc","supporting_document","measurements_only_selected_policy_check_required")),
        "final-video-audio-qc-v1"=>Ok(("aniflow.resumable-final-video-audio-qc-request/v1","final_audio_qc","supporting_document","measurements_only_selected_policy_check_required")),
        "gain-video-v1"=>Ok(("aniflow.gain-video-request/v1","gain_video","video_delivery","requires_fresh_encoded_file_qc")),
        _=>Err(error(Code::Unavailable,"selected audio-processing route unavailable")),
    }
}
fn configuration(request:&Request,profile:&str)->Result<Value,Error> {
    let value:Value=serde_json::from_slice(&command::read_document(&path(&request.configuration)?,65_536)?).map_err(|_|error(Code::InvalidInput,"explicit native audio-processing JSON required"))?;
    let (schema,_,_,_)=expected(profile)?;
    let (keys,provider):(&[&str],&Value)=match profile {
        "fixed-audio-gain-v1"=>(&["schema","check","configuration","provider_executable"],&value["provider_executable"]),
        "gain-video-v1"=>(&["schema","gain_run_directory","gain_qc_directory","gain_qc_receipt_sha256","settings","audio_bitrate_kbps","provider_executable"],&value["provider_executable"]),
        _=>(&["schema","qc"],&value["qc"]["provider_executable"]),
    };
    let required_inputs=if profile=="gain-video-v1"{2}else{1};
    if value["schema"]!=schema||value.as_object().is_none_or(|v|v.len()!=keys.len()||keys.iter().any(|k|!v.contains_key(*k)))
        ||provider.as_str()!=path(&request.provider_executable)?.to_str()||request.inputs.len()!=required_inputs {
        return Err(error(Code::InvalidInput,"exact source membership/order and matching closed explicitly selected native request required"));
    }
    let output=path(&request.run_directory)?;
    if selected_paths(&value,true)?.iter().any(|p|output.starts_with(p)||p.starts_with(&output)){return Err(error(Code::InvalidInput,"processing workspace overlaps native/QC/source/tool evidence"));}
    Ok(value)
}

#[cfg(test)]mod video_tests {
    use super::*;
    #[test]fn adjusted_video_retains_both_input_identities_and_cannot_become_a_clean_master() {
        let input=|name:&str,sha:&str|NativeFile {locator:PathBuf::from(name),sha256:sha.repeat(64),byte_count:100};
        let sources=vec![input("/synthetic/original.mp4","a"),input("/synthetic/gain.wav","b")];
        let payload=OwnerPayload {profile:"gain-video-v1".into(),request:serde_json::json!({"schema":"aniflow.gain-video-request/v1"}),source_files:sources.clone(),native:serde_json::json!({"plan_sha256":"c".repeat(64)})};
        let owner=OwnerPlan {schema:"aniflow.audio-processing-plan/v1".into(),plan_sha256:digest(&payload).unwrap(),payload};
        let file=input("/synthetic/run/video.mp4","d");
        let report=serde_json::json!({"schema":"aniflow.final-video-handoff/v1","plan_sha256":"c".repeat(64),"publication_authority":"none","proof_state":"not_asserted","artifacts":[{"artifact_id":"gain_video","locator":file.locator,"sha256":file.sha256,"byte_count":100,"variant":"clean_archival_master"}],"source_lineage":[{"id":"gain_video_source","sha256":sources[0].sha256,"byte_count":100},{"id":"gain_video_audio","sha256":sources[1].sha256,"byte_count":100}]});
        let payload=HandoffPayload {plan:owner.clone(),run_directory:PathBuf::from("/synthetic/run"),native_intent_sha256:"c".repeat(64),artifacts:vec![Artifact {id:"gain_video".into(),purpose:"video_delivery".into(),file}],report,output_state:"requires_fresh_encoded_file_qc".into(),runtime_qualified:false,creative_approval:"not_performed".into(),publication_authority:"none".into()};
        let valid=Handoff {schema:"aniflow.audio-processing-handoff/v1".into(),handoff_sha256:digest(&payload).unwrap(),payload};
        assert!(validate_handoff(&owner,&valid).is_ok());
        let mut promoted=valid.clone();promoted.payload.artifacts[0].purpose="clean_archival_master".into();promoted.handoff_sha256=digest(&promoted.payload).unwrap();assert!(validate_handoff(&owner,&promoted).is_err());
        let mut swapped_owner=owner;swapped_owner.payload.source_files.swap(0,1);swapped_owner.plan_sha256=digest(&swapped_owner.payload).unwrap();
        let mut swapped=valid;swapped.payload.plan=swapped_owner.clone();swapped.handoff_sha256=digest(&swapped.payload).unwrap();assert!(validate_handoff(&swapped_owner,&swapped).is_err());
    }
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
pub(super) fn validate_configuration(request:&Request,profile:&str)->Result<(),Error>{configuration(request,profile).map(|_|())}
pub(super) fn protected_paths(request:&Request,profile:&str)->Result<Vec<PathBuf>,Error>{selected_paths(&configuration(request,profile)?,true)}

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
    let value=match plan.payload.profile.as_str(){"fixed-audio-gain-v1"=>&plan.payload.native["native_plan"]["plan_sha256"],"gain-video-v1"=>&plan.payload.native["plan_sha256"],_=>&plan.payload.native["intent_sha256"]};
    value.as_str().filter(|s|release_cohort::is_digest(s)).ok_or_else(||error(Code::ProviderRefused,"native processing intent missing"))
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
    {
        return Err(error(Code::ProviderRefused,"native processing artifact/report membership differs"));
    }
    if owner.payload.profile=="fixed-audio-gain-v1" {
        if p.report["schema"]!="aniflow.audio-gain-completion/v1"||p.report["file"]!=serde_json::to_value(&a.file).map_err(|_|error(Code::InvalidInput,"gain file encoding"))?{return Err(error(Code::ProviderRefused,"gain completion file differs"));}
    }else if owner.payload.profile=="gain-video-v1"{validate_video_report(owner,p)?;}else{validate_qc_report(owner,p)?;}
    Ok(())
}
fn validate_video_report(owner:&OwnerPlan,p:&HandoffPayload)->Result<(),Error>{
    let r=&p.report;let a=&p.artifacts[0];
    let artifacts=r["artifacts"].as_array().filter(|a|a.len()==1).ok_or_else(||error(Code::ProviderRefused,"one exact gain-video artifact required"))?;
    let native=&artifacts[0];
    if r["schema"]!="aniflow.final-video-handoff/v1"||r["plan_sha256"]!=p.native_intent_sha256||r["publication_authority"]!="none"||r["proof_state"]!="not_asserted"
        ||native["artifact_id"]!=a.id||native["locator"].as_str()!=a.file.locator.to_str()||native["sha256"]!=a.file.sha256||native["byte_count"].as_u64()!=Some(a.file.byte_count) {
        return Err(error(Code::ProviderRefused,"selected gain-video completion identity/authority differs"));
    }
    let lineage=r["source_lineage"].as_array().filter(|a|a.len()==2).ok_or_else(||error(Code::ProviderRefused,"exact video and adjusted-audio lineage required"))?;
    if owner.payload.source_files.len()!=2{return Err(error(Code::ProviderRefused,"two ordered gain-video inputs required"));}
    for (id,file) in ["gain_video_source","gain_video_audio"].iter().zip(&owner.payload.source_files) {
        if lineage.iter().filter(|v|v["id"]==*id&&v["sha256"]==file.sha256&&v["byte_count"].as_u64()==Some(file.byte_count)).count()!=1{return Err(error(Code::ProviderRefused,"gain-video lineage differs from the exact selected input order/bytes"));}
    }Ok(())
}
fn validate_qc_report(owner:&OwnerPlan,p:&HandoffPayload)->Result<(),Error>{
    let lossless=owner.payload.profile=="final-lossless-audio-qc-v1";
    let schema=if lossless{"aniflow.final-lossless-audio-qc/v2"}else{"aniflow.final-audio-qc/v1"};
    let r=&p.report;let source=owner.payload.source_files.first().ok_or_else(||error(Code::ProviderRefused,"exact QC subject missing"))?;
    let file=if lossless{&r["intent"]["source"]["file"]}else{&r["intent"]["artifact"]};
    if r["schema"]!=schema||r["intent"]!=owner.payload.native["native"]||r["intent_sha256"]!=digest(&r["intent"])?
        ||r["runtime_qualified"]!=false||r["publication_authority"]!="none"||r["normalization"]!="none"
        ||file["locator"].as_str()!=source.locator.to_str()||file["sha256"]!=source.sha256||file["byte_count"].as_u64()!=Some(source.byte_count)
        ||p.artifacts[0].file.locator!=p.run_directory.join("qc.json") {
        return Err(error(Code::ProviderRefused,"fresh QC report belongs to different final bytes, native intent or authority"));
    }Ok(())
}

#[cfg(test)]mod qc_tests {
    use super::*;
    fn fixture(profile:&str)->(OwnerPlan,Handoff) {
        let file=NativeFile {locator:PathBuf::from("/synthetic/final 曲"),sha256:"a".repeat(64),byte_count:100};
        let source=serde_json::to_value(&file).unwrap();let lossless=profile=="final-lossless-audio-qc-v1";
        let intent=if lossless{serde_json::json!({"source":{"file":source}})}else{serde_json::json!({"artifact":source})};
        let payload=OwnerPayload {profile:profile.into(),request:serde_json::json!({"schema":expected(profile).unwrap().0}),source_files:vec![file],native:serde_json::json!({"intent_sha256":"b".repeat(64),"native":intent})};
        let owner=OwnerPlan {schema:"aniflow.audio-processing-plan/v1".into(),plan_sha256:digest(&payload).unwrap(),payload};
        let report=serde_json::json!({"schema":if lossless{"aniflow.final-lossless-audio-qc/v2"}else{"aniflow.final-audio-qc/v1"},"intent":intent,"intent_sha256":digest(&intent).unwrap(),"runtime_qualified":false,"normalization":"none","publication_authority":"none"});
        let payload=HandoffPayload {plan:owner.clone(),run_directory:PathBuf::from("/synthetic/qc"),native_intent_sha256:"b".repeat(64),artifacts:vec![Artifact {id:"final_audio_qc".into(),purpose:"supporting_document".into(),file:NativeFile {locator:PathBuf::from("/synthetic/qc/qc.json"),sha256:"c".repeat(64),byte_count:1000}}],report,output_state:"measurements_only_selected_policy_check_required".into(),runtime_qualified:false,creative_approval:"not_performed".into(),publication_authority:"none".into()};
        let handoff=Handoff {schema:"aniflow.audio-processing-handoff/v1".into(),handoff_sha256:digest(&payload).unwrap(),payload};(owner,handoff)
    }
    #[test]fn rehashed_qc_cannot_promote_measurements_into_goal_or_runtime_acceptance() {
        for profile in ["final-lossless-audio-qc-v1","final-video-audio-qc-v1"] {
            let (owner,valid)=fixture(profile);assert!(validate_handoff(&owner,&valid).is_ok());
            let mut changed=valid.clone();changed.payload.output_state="matches_selected_policy".into();changed.handoff_sha256=digest(&changed.payload).unwrap();assert!(validate_handoff(&owner,&changed).is_err());
            let mut changed=valid;changed.payload.report["runtime_qualified"]=Value::Bool(true);changed.handoff_sha256=digest(&changed.payload).unwrap();assert!(validate_handoff(&owner,&changed).is_err());
        }
    }
    #[test]fn wav_qc_is_not_a_substitute_for_encoded_video_qc() {
        let (_,wav)=fixture("final-lossless-audio-qc-v1");let (video_owner,_)=fixture("final-video-audio-qc-v1");
        assert!(validate_handoff(&video_owner,&wav).is_err());
        let (owner,mut video)=fixture("final-video-audio-qc-v1");
        video.payload.report["intent"]["artifact"]["sha256"]=Value::String("d".repeat(64));
        video.payload.report["intent_sha256"]=Value::String(digest(&video.payload.report["intent"]).unwrap());
        video.handoff_sha256=digest(&video.payload).unwrap();assert!(validate_handoff(&owner,&video).is_err());
    }
}
fn args(command:&str,request:&Request)->Result<Vec<OsString>,Error>{
    let mut args=["--output","json","delivery","audio-processing",command,"--request"].into_iter().map(OsString::from).collect::<Vec<_>>();
    args.push(path(&request.configuration)?.into_os_string());Ok(args)
}
pub(super) fn execute(context:&Context<'_>,resume:Option<&Path>)->Result<Receipt,Error>{
    let profile=&context.plan.row.profile;let config=configuration(&context.plan.request,profile)?;
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
    validate_owner(&plan.row.profile,&configuration(&plan.request,&plan.row.profile)?,&plan.sources,&record.native_plan)?;
    validate_handoff(&record.native_plan,&record.native_handoff)?;
    if record.schema_version!=RECORD_SCHEMA||record.scope!=RECORD_SCOPE||record.source_pins!=plan.sources
        ||record.native_handoff.payload.native_intent_sha256!=receipt.native_intent_sha256||record.native_handoff.payload.run_directory!=path(&receipt.native_run)?
        ||read::<OwnerPlan>(&path(&plan.request.run_directory)?.join("attempt-0000/native-plan.json"))?!=record.native_plan {
        return Err(error(Code::RecoveryRequired,"retained processing request/intent/scope differs"));
    }
    inventory(receipt,value).map(|_|())
}
/// Portable framing of a captured owner record. No current owner, source path,
/// tool, release or approval is observed by this function.
pub(super) fn retained_record(plan:&Plan,receipt:&Receipt,config:&Value,value:&Value)->Result<Value,Error>{
    let record:Record=decode(value.clone())?;
    validate_owner(&plan.row.profile,config,&plan.sources,&record.native_plan)?;
    validate_handoff(&record.native_plan,&record.native_handoff)?;
    let mut unsigned=plan.clone();unsigned.plan_sha256.clear();
    if plan.row.tool!=Tool::Aniflow||plan.schema_version!=PLAN_SCHEMA||digest(&unsigned)?!=plan.plan_sha256
        ||record.schema_version!=RECORD_SCHEMA||record.scope!=RECORD_SCOPE||record.source_pins!=plan.sources
        ||receipt.schema_version!=RECEIPT_SCHEMA||receipt.state!=State::Complete||receipt.plan_sha256!=plan.plan_sha256||receipt.row_subject_sha256!=plan.row_subject_sha256
        ||receipt.release_qualified||receipt.creative_approval_inferred||receipt.publication_authority
        ||record.native_handoff.payload.native_intent_sha256!=receipt.native_intent_sha256||record.native_handoff.payload.run_directory!=path(&receipt.native_run)? {
        return Err(error(Code::RecoveryRequired,"portable processing history framing differs"));
    }
    let artifacts=&record.native_handoff.payload.artifacts;
    if artifacts.len()!=receipt.artifacts.len(){return Err(error(Code::RecoveryRequired,"portable processing artifact membership differs"));}
    for (a,p) in artifacts.iter().zip(&receipt.artifacts){if a.file.locator!=path(&p.path)?||a.file.sha256!=p.sha256||a.file.byte_count!=p.byte_count{return Err(error(Code::RecoveryRequired,"portable processing artifact identity differs"));}}
    serde_json::to_value(record.native_handoff).map_err(|_|error(Code::InvalidInput,"retained processing handoff encoding"))
}
pub(super) fn inventory(receipt:&Receipt,value:&Value)->Result<Vec<ArtifactDescription>,Error>{
    let record:Record=decode(value.clone())?;let artifacts=&record.native_handoff.payload.artifacts;
    if artifacts.len()!=receipt.artifacts.len(){return Err(error(Code::RecoveryRequired,"processing accepted artifact membership differs"));}
    artifacts.iter().zip(&receipt.artifacts).map(|(a,p)|{
        if a.file.locator!=path(&p.path)?||a.file.sha256!=p.sha256||a.file.byte_count!=p.byte_count{return Err(error(Code::RecoveryRequired,"processing artifact differs from committed byte pin"));}
        if a.purpose=="supporting_document"&&read::<Value>(&a.file.locator)?!=record.native_handoff.payload.report{return Err(error(Code::RecoveryRequired,"QC file content differs from the captured owner report"));}
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
