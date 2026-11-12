//! Main command delivery for existing selected-file AI setup; no inference.
use super::*;
use crate::ai_preparation::{self as ai,setup::{self,documents}};
fn refusal(_:crate::ValidationError)->Error{error(Code::InvalidInput,"local AI setup input/identity refused; use explicit canonical bounded private documents")}
fn outcome(state:ai::State)->Outcome{match state{ai::State::Disabled=>Outcome::Complete,ai::State::PreparedBytes=>Outcome::Partial,_=>Outcome::Unavailable}}
fn separate_output(request:&setup::Request,input:&Path,output:&Path)->Result<(),Error>{
    for p in std::iter::once(input).chain(request.assets.iter().map(|a|a.path.as_path())) {
        if !p.is_absolute()||p.components().any(|c|matches!(c,std::path::Component::ParentDir|std::path::Component::CurDir))||output.starts_with(p)||p.starts_with(output){return Err(error(Code::InvalidInput,"AI setup output must be separate from request and selected asset paths"));}
    }Ok(())
}
pub(super) fn execute(args:&Arguments,token:&dyn CancellationSignal)->Result<Option<Envelope>,Error>{
    let cancel=||token.is_cancelled();
    match args.command.as_str(){
        "ai-prepare"=>{
            let input=path(args,"--request")?;let root=path(args,"--output-directory")?;
            let request=setup::Request::from_json_slice(&documents::read(&input,131_072).map_err(refusal)?).map_err(refusal)?;
            separate_output(&request,&input,&root)?;
            let value=documents::write(&request,&root,&cancel).map_err(|e|{let mut e=refusal(e);e.recovery_directory=Some(NativePath::from_path(&root));e})?;
            let marker=crate::native::FilePin::observe(&NativePath::from_path(&root.join("setup.json")),16_384,token)?;
            let state=outcome(value.current.report.state);
            success(&args.command,serde_json::json!({"schema_version":"flow.ai-setup-command/v1","marker":marker,"setup":value,"runtime_or_service_contacted":false,"invocation_authority":false}),state).map(Some)
        },
        "ai-doctor"=>{
            let profile=ai::Profile::from_json_slice(&documents::read(&path(args,"--profile")?,32_768).map_err(refusal)?).map_err(refusal)?;
            let preparation=if profile.mode==ai::Mode::Disabled{None}else{args.options.get("--preparation").map(|_|path(args,"--preparation").and_then(|p|documents::read(&p,65_536).map_err(refusal)).and_then(|b|ai::LocalPreparation::from_json_slice(&b).map_err(refusal))).transpose()?};
            let value=setup::diagnose(&profile,preparation.as_ref(),&cancel).map_err(refusal)?;let state=outcome(value.report.state);
            success(&args.command,value,state).map(Some)
        },
        "ai-setup-status"=>{
            let root=path(args,"--directory")?;let expected=args.options.get("--marker-sha256").and_then(|v|v.to_str()).ok_or_else(||error(Code::Usage,"exact AI setup marker digest required"))?;
            let pin=crate::native::FilePin::observe(&NativePath::from_path(&root.join("setup.json")),16_384,token)?;
            if pin.sha256!=expected{return Err(error(Code::StalePlan,"AI setup marker differs from selected pin"));}
            let value=documents::status(&root,&cancel).map_err(refusal)?;pin.current(16_384,token)?;let state=outcome(value.current.report.state);
            success(&args.command,serde_json::json!({"schema_version":"flow.ai-setup-command/v1","marker":pin,"setup":value,"runtime_or_service_contacted":false,"invocation_authority":false}),state).map(Some)
        },
        _=>Ok(None),
    }
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn prepared_bytes_remain_partial_and_setup_cannot_enable_network_or_inference(){
        assert_eq!(outcome(ai::State::PreparedBytes),Outcome::Partial);
        let argv=|s:&[&str]|->Vec<OsString>{s.iter().map(|v|OsString::from(*v)).collect()};
        assert!(parse(argv(&["ai-doctor","--profile","private 模型.json"])).is_ok());
        assert!(parse(argv(&["ai-prepare","--request","r","--output-directory","d","--download-model","x"])).is_err());
        assert!(parse(argv(&["ai-doctor","--profile","p","--acknowledge-verification-network"])).is_err());
        assert!(parse(argv(&["ai-setup-status","--directory","d"])).is_err());
    }
}
