use super::*;
use crate::delivery::audio_readiness;

pub(super) fn execute(args:&Arguments,token:&dyn CancellationSignal)->Result<Option<Envelope>,Error>{
    if !["audio-readiness-preview","audio-readiness-run"].contains(&args.command.as_str()){return Ok(None);}
    let request=audio_readiness::Request::from_json_slice(&read_document(&path(args,"--request")?,1_048_576)?)?;
    if args.options.contains_key("--output"){audio_readiness::validate_output_path(&request,&path(args,"--output")?,token)?;}
    let value=if args.command=="audio-readiness-preview"{audio_readiness::preview(&request,token)?}else{
        let verified=if request.needs_released_evidence(){
            let verification=path(args,"--verification-directory")?;audio_readiness::validate_output_path(&request,&verification,token)?;
            let evidence=crate::release_evidence::Request::from_json_slice(&read_document(&path(args,"--evidence-request")?,1_048_576)?)?;
            let trust=release_delivery::selected_trust(args,token)?;
            Some(crate::release_evidence::verify(&evidence,&trust,&verification,args.acknowledge_verification_network,token)?)
        }else{None};
        audio_readiness::run(&request,verified.as_ref(),token)?
    };
    if args.options.contains_key("--output"){audio_readiness::save(&request,&path(args,"--output")?,&value,token)?;}
    let outcome=if args.command=="audio-readiness-preview"||value.all_selected_audio_goals_match&&!value.qualification_pending&&value.reviews.state=="operator_declarations_supplied"&&value.missing_metadata.is_empty()&&value.missing_destination_payloads.is_empty(){Outcome::Complete}else{Outcome::Partial};
    success(&args.command,value,outcome).map(Some)
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]fn preview_is_read_only_and_run_requires_a_selected_request() {
        assert!(parse(["audio-readiness-run"].into_iter().map(OsString::from).collect()).is_err());
        assert!(parse(["audio-readiness-preview","--request","private request.json"].into_iter().map(OsString::from).collect()).is_ok());
        assert!(parse(["audio-readiness-preview","--request","r.json","--evidence-request","release.json"].into_iter().map(OsString::from).collect()).is_err());
        assert!(parse(["audio-readiness-run","--request","r.json","--acknowledge-verification-network"].into_iter().map(OsString::from).collect()).is_ok());
    }
}
