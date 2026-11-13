//! Thin delivery for the product-owned metadata-only support profile.
use super::*;
use crate::diagnostics;
fn mapped(error:diagnostics::Error)->Error{let code=match error{diagnostics::Error::Unsupported=>Code::Unavailable,diagnostics::Error::Cancelled=>Code::Cancelled,diagnostics::Error::Changed=>Code::StalePlan,diagnostics::Error::Exists=>Code::RecoveryRequired,diagnostics::Error::Io=>Code::Io,_=>Code::InvalidInput};super::error(code,&error.to_string())}
pub(super) fn execute(args:&Arguments,token:&dyn CancellationSignal)->Result<Option<Envelope>,Error>{
    let owner=diagnostics::owner();let cancel=||token.is_cancelled();
    match args.command.as_str(){
        "diagnostics-preview"|"diagnostics-bundle"=>{
            let request=owner.read_request(&path(args,"--request")?,&cancel).map_err(mapped)?;
            if args.command=="diagnostics-preview"{success(&args.command,owner.preview(&request,&cancel).map_err(mapped)?,Outcome::Complete).map(Some)}
            else{success(&args.command,owner.create_bundle(&request,&path(args,"--output")?,&cancel).map_err(mapped)?,Outcome::Complete).map(Some)}
        },
        "diagnostics-inspect"=>success(&args.command,owner.inspect_bundle(&path(args,"--bundle")?,&cancel).map_err(mapped)?,Outcome::Complete).map(Some),
        "doctor-observability"=>success(&args.command,owner.health(),Outcome::Complete).map(Some),
        _=>Ok(None),
    }
}
#[cfg(test)]mod tests{
    use super::*;
    fn argv(v:&[&str])->Vec<OsString>{v.iter().map(|s|OsString::from(*s)).collect()}
    #[test]fn diagnostics_write_network_and_approval_options_stay_separate(){
        assert!(parse(argv(&["diagnostics-preview","--request","private 作品.json"])).is_ok());
        assert!(parse(argv(&["diagnostics-bundle","--request","r"])).is_err());
        assert!(parse(argv(&["diagnostics-preview","--request","r","--output","new"])).is_err());
        assert!(parse(argv(&["diagnostics-inspect","--bundle","b","--acknowledge-verification-network"])).is_err());
    }
}
