//! Thin delivery over the independent installation SDK.
use super::*;
use crate::distribution;
fn text<'a>(args:&'a Arguments,key:&str)->Result<&'a str,Error>{args.options.get(key).and_then(|v|v.to_str()).ok_or_else(||error(Code::Usage,"explicit UTF-8 digest required"))}
pub(super) fn execute(args:&Arguments,token:&dyn CancellationSignal)->Result<Option<Envelope>,Error>{
    let value=match args.command.as_str(){
        "suite-install"=>{let request=if args.options.contains_key("--request"){distribution::Request::from_json_slice(&read_document(&path(args,"--request")?,1_048_576)?)?}else{distribution::open_bundle(&path(args,"--bundle")?,text(args,"--bundle-sha256")?,&path(args,"--install-directory")?,token)?};success("suite-install",distribution::install(&request,args.acknowledge_unqualified_candidate,token)?,Outcome::Complete)?},
        "suite-bundle"=>{let request=distribution::Request::from_json_slice(&read_document(&path(args,"--request")?,1_048_576)?)?;success("suite-bundle",distribution::bundle(&request,&path(args,"--output-directory")?,token)?,Outcome::Complete)?},
        "suite-status"=>success("suite-status",distribution::status(&path(args,"--installation-root")?,text(args,"--receipt-sha256")?,token)?,Outcome::Complete)?,
        "suite-activate"|"suite-rollback"=>success(&args.command,distribution::activate(&path(args,"--installation-root")?,text(args,"--receipt-sha256")?,&path(args,"--selection")?,args.options.get("--previous-selection-sha256").map(|_|text(args,"--previous-selection-sha256")).transpose()?,token)?,Outcome::Complete)?,
        "suite-uninstall"=>{distribution::uninstall(&path(args,"--installation-root")?,text(args,"--receipt-sha256")?,&path(args,"--removal-record")?,token)?;success("suite-uninstall",serde_json::json!({"owned_copies_removed":true,"original_assets_removed":false}),Outcome::Complete)?},
        _=>return Ok(None),
    };Ok(Some(value))
}
