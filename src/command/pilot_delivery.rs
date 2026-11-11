//! Thin fixed-preset delivery; the public pilot SDK owns composition.
use super::*;
use crate::pilot;
pub(super) fn execute(args:&Arguments,token:&dyn CancellationSignal)->Result<Option<Envelope>,Error>{
    if !args.command.starts_with("pilot-"){return Ok(None);}
    if args.command=="pilot-status"{let value=pilot::status(&path(args,"--run-directory")?,token)?;let outcome=if value.handoff.is_some(){Outcome::Complete}else{Outcome::RecoveryRequired};return success("pilot-status",value,outcome).map(Some);}
    if args.command=="pilot-plan"{let request=pilot::Request::from_json_slice(&read_document(&path(args,"--request")?,1_048_576)?)?;let value=pilot::prepare(&request,token)?;if args.options.contains_key("--output"){pilot::save(&value,&path(args,"--output")?,&value)?;}return success("pilot-plan",value,Outcome::Complete).map(Some);}
    let plan=pilot::Plan::from_json_slice(&read_document(&path(args,"--plan")?,16_777_216)?)?;
    if args.command=="pilot-approve"{let by=args.options.get("--approved-by").and_then(|s|s.to_str()).ok_or_else(||error(Code::Usage,"explicit operator declaration required"))?.to_owned();let value=pilot::approve(&plan,by,args.acknowledge_trusted_unconfined,args.acknowledge_unqualified_candidate)?;pilot::save(&plan,&path(args,"--output")?,&value)?;return success("pilot-approve",value,Outcome::Complete).map(Some);}
    let approval:pilot::Approval=serde_json::from_slice(&read_document(&path(args,"--approval")?,65_536)?).map_err(|_|error(Code::InvalidInput,"closed pilot approval required"))?;
    success(&args.command,pilot::run(&plan,&approval,args.command=="pilot-continue",token)?,Outcome::Complete).map(Some)
}
