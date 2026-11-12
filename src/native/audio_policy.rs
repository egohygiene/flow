//! Concrete read-only Aniflow policy adapter over the existing native runner.
//! Flow checks identity/framing, while Aniflow owns loudness math and QC truth.
use super::*;
pub(super) const PROFILE:&str="audio-policy-check-v1";

fn configuration(request:&Request)->Result<Value,Error>{
    let value:Value=serde_json::from_slice(&command::read_document(&path(&request.configuration)?,131_072)?).map_err(|_|error(Code::InvalidInput,"audio policy request JSON required"))?;
    if request.inputs.len()!=1||path(&request.inputs[0])?.to_str().is_none()||value["schema"]!="aniflow.audio-policy-check-request/v1"||value["policy"]["schema"]!="aniflow.audio-delivery-policy/v1"
        ||!matches!(value["input"]["kind"].as_str(),Some("accepted_video_with_qc"|"accepted_audio_with_qc"))
        ||value.as_object().is_none_or(|v|v.len()!=3||!["schema","input","policy"].iter().all(|k|v.contains_key(*k))){return Err(error(Code::InvalidInput,"one explicit final media file and closed Aniflow current-QC policy request required"));}Ok(value)
}
pub(super) fn validate_configuration(request:&Request)->Result<(),Error>{configuration(request).map(|_|())}

fn validate_assessment(plan:&Plan,value:&Value)->Result<(),Error>{
    let config=configuration(&plan.request)?;let source=&plan.sources[0];
    validate_subject(&config,source,value)
}
fn validate_subject(config:&Value,source:&FilePin,value:&Value)->Result<(),Error>{
    let scope=&value["evaluation"];
    let expected_kind=match config["input"]["kind"].as_str(){Some("accepted_video_with_qc")=>"video_with_qc",Some("accepted_audio_with_qc")=>"audio_with_qc",_=>return Err(error(Code::InvalidInput,"final-file QC input required"))};
    if value["subject"]["facts"]["kind"]!=expected_kind{return Err(error(Code::ProviderRefused,"owner QC facts differ from the selected final-file kind"));}
    let states=[scope["integrated_loudness"]["state"].as_str(),scope["true_peak"]["state"].as_str()];
    let channels=match value["subject"]["facts"]["kind"].as_str(){Some("video_with_qc")=>value["subject"]["facts"]["qc"]["signal"]["channels"].as_u64(),Some("audio_with_qc")=>value["subject"]["facts"]["qc"]["accepted_pcm"]["source"]["channels"].as_u64(),_=>None}.filter(|n|matches!(*n,1|2)).ok_or_else(||error(Code::ProviderRefused,"complete mono/stereo owner QC scope required"))?;
    for (key,unit) in [("integrated_loudness","loudness_units_full_scale"),("true_peak","decibels_true_peak")] {
        let check=&scope[key];let observed=&check["observed"];let measurement=&observed["value"];
        let expected=(0..channels).map(Value::from).collect::<Vec<_>>();
        let available=match measurement["kind"].as_str(){Some("measured")=>measurement["value"].as_f64().is_some_and(f64::is_finite)&&check["state"]!="missing_evidence",Some("unavailable")=>measurement["reason"].as_str().is_some_and(|s|!s.is_empty()&&s.len()<=96)&&check["state"]=="missing_evidence",_=>false};
        if observed["unit"]!=unit||observed["scope"]["channels"].as_array()!=Some(&expected)||!observed["scope"].get("stem_id").is_some_and(Value::is_null)||!available||!check["maximum"].as_f64().is_some_and(f64::is_finite)
            ||(key=="integrated_loudness"&&!check["minimum"].as_f64().is_some_and(f64::is_finite))||(key=="true_peak"&&!check.get("minimum").is_some_and(Value::is_null)) {return Err(error(Code::ProviderRefused,"audio owner quantities have invalid units, scope, availability or bounds"));}
    }
    if value["schema"]!="aniflow.audio-policy-assessment/v1"||&value["request"]!=config||value["subject"]["input"]!=config["input"]
        ||value["subject"]["file"]["locator"].as_str()!=path(&source.path)?.to_str()||value["subject"]["file"]["sha256"]!=source.sha256||value["subject"]["file"]["byte_count"].as_u64()!=Some(source.byte_count)
        ||scope["schema"]!="aniflow.audio-policy-evaluation/v1"||scope["policy"]!=config["policy"]||scope["policy_sha256"]!=digest(&config["policy"])?
        ||scope["runtime_qualified"]!=false||scope["creative_approval"]!="not_performed"||scope["publication_authority"]!="none"
        ||states.iter().any(|s|!matches!(s,Some("matches"|"outside_target"|"missing_evidence")))
        ||scope["matches_selected_policy"].as_bool()!=Some(states==[Some("matches"),Some("matches")])
        ||value["assessment_sha256"]!=digest(&("aniflow.audio-policy-assessment/v1",&value["request"],&value["subject"],scope))? {
        return Err(error(Code::ProviderRefused,"Aniflow audio assessment framing/source/policy/authority binding differs"));
    }
    Ok(())
}

pub(super) fn execute(context:&Context<'_>,resume:Option<&Path>)->Result<Receipt,Error>{
    validate_configuration(&context.plan.request)?;
    let descriptor=super::aniflow::invoke(context,"contract","release_contract",&["--output".into(),"json".into(),"release-contract".into()])?;
    let pinned:Value=read(&path(&context.plan.descriptor.path)?)?;
    if descriptor!=pinned{return Err(error(Code::Unavailable,"running Aniflow audio-check contract differs from exact descriptor"));}
    let root=context.native_root().join("audio-policy");
    let config=configuration(&context.plan.request)?;
    if let Some(selected)=resume {
        context.require_native_run(selected)?;
        if selected!=root||read::<Value>(&root.join("request.json"))?!=config{return Err(error(Code::RecoveryRequired,"only the exact recognized read-only audio check can resume"));}
        for e in fs::read_dir(&root).map_err(|_|error(Code::Io,"audio check members unavailable"))? {
            let e=e.map_err(|_|error(Code::Io,"audio check member unavailable"))?;
            if !e.file_type().is_ok_and(|t|t.is_file())||!["request.json","assessment.json"].iter().any(|n|e.file_name()==*n){return Err(error(Code::RecoveryRequired,"unknown/symbolic audio check members retained"));}
        }
    }else{create_directory(&root)?;write(&root.join("request.json"),&config)?;}
    let mut args:Vec<OsString>=["--output","json","delivery","audio-policy","check","--request"].into_iter().map(OsString::from).collect();args.push(path(&context.plan.configuration.path)?.into_os_string());
    let assessment=super::aniflow::invoke(context,"audio-check","delivery_audio_policy_check",&args)?;
    validate_assessment(context.plan,&assessment)?;
    let record=root.join("assessment.json");
    if fs::symlink_metadata(&record).is_ok(){if resume.is_none()||read::<Value>(&record)?!=assessment{return Err(error(Code::RecoveryRequired,"retained audio check changed; start a new check rather than adopt/overwrite it"));}}
    else{write(&record,&assessment)?;}
    let observed=FilePin::observe(&NativePath::from_path(&record),context.plan.request.limits.max_output_bytes,context.token)?;
    let artifacts=context.observe_artifacts(vec![(record,observed.sha256,observed.byte_count)])?;
    context.finish(&root,assessment["assessment_sha256"].as_str().ok_or_else(||error(Code::ProviderRefused,"audio assessment digest missing"))?,&serde_json::json!({"schema_version":"flow.aniflow-audio-policy-acceptance/v1","assessment":assessment,"source_pins":context.plan.sources,"scope":"owner_rechecked_during_this_attempt; retained_status_requires_a_fresh_owner_check_for_current_QC"}),artifacts)
}

pub(super) fn validate_retained(plan:&Plan,receipt:&Receipt,value:&Value)->Result<(),Error>{
    let assessment=&value["assessment"];validate_assessment(plan,assessment)?;
    if value["schema_version"]!="flow.aniflow-audio-policy-acceptance/v1"||value["source_pins"]!=serde_json::to_value(&plan.sources).map_err(|_|error(Code::InvalidInput,"audio source pins encoding"))?
        ||assessment["assessment_sha256"]!=receipt.native_intent_sha256||receipt.artifacts.len()!=1
        ||path(&receipt.artifacts[0].path)?!=path(&receipt.native_run)?.join("assessment.json")
        ||read::<Value>(&path(&receipt.native_run)?.join("request.json"))?!=configuration(&plan.request)?
        ||read::<Value>(&path(&receipt.artifacts[0].path)?)?!=*assessment {
        return Err(error(Code::RecoveryRequired,"retained audio assessment/source/subject differs"));
    }
    Ok(())
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]fn wrong_final_bytes_or_inferred_qualification_cannot_be_admitted() {
        let source=FilePin {path:NativePath::from_path(Path::new("/synthetic/final 音.mp4")),sha256:"a".repeat(64),byte_count:100};
        let config=serde_json::json!({"schema":"aniflow.audio-policy-check-request/v1","input":{"kind":"accepted_video_with_qc"},"policy":{"schema":"aniflow.audio-delivery-policy/v1","id":"synthetic"}});
        let quantity=|unit:&str,value:f64,minimum:Option<f64>,maximum:f64|serde_json::json!({"state":"matches","minimum":minimum,"maximum":maximum,"observed":{"unit":unit,"scope":{"channels":[0,1],"stem_id":null},"value":{"kind":"measured","value":value}}});
        let mut evaluation=serde_json::json!({"schema":"aniflow.audio-policy-evaluation/v1","policy":config["policy"],"policy_sha256":digest(&config["policy"]).unwrap(),"integrated_loudness":quantity("loudness_units_full_scale",-14.0,Some(-14.5),-13.5),"true_peak":quantity("decibels_true_peak",-2.0,None,-1.0),"matches_selected_policy":true,"runtime_qualified":false,"creative_approval":"not_performed","publication_authority":"none"});
        let subject=serde_json::json!({"input":config["input"],"file":{"locator":"/synthetic/final 音.mp4","sha256":source.sha256,"byte_count":100},"facts":{"kind":"video_with_qc","qc":{"signal":{"channels":2}}}});
        let make=|e:&Value|serde_json::json!({"schema":"aniflow.audio-policy-assessment/v1","request":config,"subject":subject,"evaluation":e,"assessment_sha256":digest(&("aniflow.audio-policy-assessment/v1",&config,&subject,e)).unwrap()});
        assert!(validate_subject(&config,&source,&make(&evaluation)).is_ok());
        let mut stale=make(&evaluation);stale["subject"]["file"]["sha256"]=Value::String("b".repeat(64));assert!(validate_subject(&config,&source,&stale).is_err());
        evaluation["runtime_qualified"]=Value::Bool(true);assert!(validate_subject(&config,&source,&make(&evaluation)).is_err());
    }
}
