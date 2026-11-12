//! Local setup operations over the existing public delivery SDK; no native run.
use super::*;
use crate::delivery::audio_setup;

fn record_sha(args:&Arguments)->Result<&str,Error>{
    args.options.get("--record-sha256").and_then(|s|s.to_str()).ok_or_else(||error(Code::Usage,"exact audio setup record digest required"))
}
pub(super) fn execute(args:&Arguments,token:&dyn CancellationSignal)->Result<Option<Envelope>,Error>{
    match args.command.as_str(){
        "audio-setup-preview"|"audio-setup-prepare"=>{
            let request=audio_setup::Request::from_json_slice(&read_document(&path(args,"--request")?,1_048_576)?)?;
            if args.command=="audio-setup-preview"{success(&args.command,audio_setup::preview(&request,token)?,Outcome::Complete).map(Some)}
            else{success(&args.command,audio_setup::prepare(&request,token)?,Outcome::Complete).map(Some)}
        },
        "audio-setup-status"=>success(&args.command,audio_setup::status(&path(args,"--workspace")?,record_sha(args)?,token)?,Outcome::Complete).map(Some),
        "audio-setup-bind"=>{
            let approvals=audio_setup::ApprovalSet::from_json_slice(&read_document(&path(args,"--approvals")?,1_048_576)?)?;
            let value=audio_setup::save_binding(&path(args,"--workspace")?,record_sha(args)?,&approvals,&path(args,"--output")?,token)?;
            success(&args.command,value,Outcome::Complete).map(Some)
        },
        _=>Ok(None),
    }
}

#[cfg(test)]mod tests {
    use super::*;
    fn argv(items:&[&str])->Vec<OsString>{items.iter().map(|v|OsString::from(*v)).collect()}
    #[test]fn setup_requires_exact_inputs_and_cannot_create_or_execute_approval(){
        assert!(parse(argv(&["audio-setup-preview","--request","private 曲/setup.json"])).is_ok());
        assert!(parse(argv(&["audio-setup-prepare","--request","r","--acknowledge-trusted-unconfined"])).is_err());
        assert!(parse(argv(&["audio-setup-prepare","--request","r","--approved-by","operator"])).is_err());
        assert!(parse(argv(&["audio-setup-status","--workspace","w"])).is_err());
        assert!(parse(argv(&["audio-setup-bind","--workspace","w","--record-sha256","s","--approvals","a"])).is_err());
        assert!(parse(argv(&["audio-setup-bind","--workspace","w","--record-sha256","s","--approvals","a","--output","out.json"])).is_ok());
        assert!(parse(argv(&["audio-setup-bind","--workspace","w","--record-sha256","s","--approvals","a","--output","o","--acknowledge-verification-network"])).is_err());
    }
}
