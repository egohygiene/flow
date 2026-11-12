//! Explicit effectful evidence delivery. Default doctor/matrix stays read-only.
use super::*;
use crate::{native as provider, release_evidence as evidence};
pub(super) fn execute(args:&Arguments,token:&dyn CancellationSignal)->Result<Option<Envelope>,Error>{
    let special=["release-evidence","approve-released","run-released","status-released","resume-released"].contains(&args.command.as_str());
    if !special&&!(args.command=="release-matrix"&&args.options.contains_key("--evidence-request")){return Ok(None);}
    let request=evidence::Request::from_json_slice(&read_document(&path(args,"--evidence-request")?,1_048_576)?)?;
    if args.command=="release-matrix"{let selected=read_document(&path(args,"--cohort")?,1_048_576)?;if crate::release_cohort::sha256(&selected)!=request.cohort.sha256{return Err(error(Code::InvalidInput,"evidence request selects another cohort"));}}
    let trust=selected_trust(args,token)?;
    let verified=evidence::verify(&request,&trust,&path(args,"--verification-directory")?,args.acknowledge_verification_network,token)?;
    match args.command.as_str(){
        "release-evidence"=>success("release-evidence",verified.summary(),Outcome::Complete).map(Some),
        "release-matrix"=>{let value=verified.matrix(token)?;let outcome=if value.green==value.rows.len(){Outcome::Complete}else{Outcome::Partial};success("release-matrix",value,outcome).map(Some)},
        "approve-released"|"run-released"=>{
            let plan=provider::Plan::from_json_slice(&read_document(&path(args,"--plan")?,16_777_216)?)?;
            if args.command=="approve-released"{let by=args.options.get("--approved-by").and_then(|s|s.to_str()).ok_or_else(||error(Code::Usage,"explicit operator declaration required"))?.to_owned();let value=provider::approve_released(&plan,by,args.acknowledge_trusted_unconfined,&verified,token)?;provider::save_document(&plan,&path(args,"--output")?,&value)?;success("approve-released",value,Outcome::Complete).map(Some)}
            else{let a:provider::ReleasedApproval=serde_json::from_slice(&read_document(&path(args,"--approval")?,65_536)?).map_err(|_|error(Code::InvalidInput,"closed released approval required"))?;success("run-released",provider::run_released(&plan,&a,&verified,token)?,Outcome::Complete).map(Some)}
        },
        "status-released"=>{let value=provider::status_released(&path(args,"--run-directory")?,&verified,token)?;let outcome=if value.native.state==provider::State::Complete{Outcome::Complete}else{Outcome::RecoveryRequired};success("status-released",value,outcome).map(Some)},
        "resume-released"=>success("resume-released",provider::resume_released(&path(args,"--run-directory")?,&path(args,"--native-run-directory")?,&verified,token)?,Outcome::Complete).map(Some),
        _=>Err(error(Code::Usage,"explicit release evidence command required")),
    }
}

pub(super) fn selected_trust(args:&Arguments,token:&dyn CancellationSignal)->Result<evidence::TrustedConfiguration,Error>{
    let host_pin=|file:&str,digest:&str,maximum:u64|->Result<provider::FilePin,Error>{let expected=args.options.get(digest).and_then(|s|s.to_str()).ok_or_else(||error(Code::Usage,"explicit host trust digest required"))?;let actual=provider::FilePin::observe(&NativePath::from_path(&path(args,file)?),maximum,token)?;if actual.sha256!=expected{return Err(error(Code::StalePlan,"host trust file differs from explicit digest"));}Ok(actual)};
    let trust=evidence::TrustedConfiguration::from_host_configuration(host_pin("--verification-policy","--policy-sha256",1_048_576)?,host_pin("--verifier-executable","--verifier-sha256",536_870_912)?)?;
    Ok(trust)
}
