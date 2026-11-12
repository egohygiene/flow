//! Thin local catalog/preview/copy/readback delivery; no provider or network call.
use super::*;
use crate::{delivery as package,native};
pub(super) fn execute(args:&Arguments,token:&dyn CancellationSignal)->Result<Option<Envelope>,Error>{
    match args.command.as_str(){
        "delivery-catalog"=>{let pin=|file:&str,digest:&str|->Result<native::FilePin,Error>{let sha=args.options.get(digest).and_then(|v|v.to_str()).ok_or_else(||error(Code::Usage,"exact selected record digest required"))?;let pin=native::FilePin::observe(&NativePath::from_path(&path(args,file)?),16_777_216,token)?;if pin.sha256!=sha{return Err(error(Code::StalePlan,"selected record bytes changed"));}Ok(pin)};
            success("delivery-catalog",package::catalog(&pin("--project-record","--project-record-sha256")?,&pin("--handoff","--handoff-sha256")?,token)?,Outcome::Complete).map(Some)},
        "delivery-preview"|"delivery-prepare"=>{let request=package::Request::from_json_slice(&read_document(&path(args,"--request")?,1_048_576)?)?;
            if args.command=="delivery-preview"{success("delivery-preview",package::preview(&request,token)?,Outcome::Complete).map(Some)}else{success("delivery-prepare",package::prepare(&request,token)?,Outcome::Complete).map(Some)}},
        "delivery-status"=>{let sha=args.options.get("--manifest-sha256").and_then(|s|s.to_str()).ok_or_else(||error(Code::Usage,"exact delivery manifest digest required"))?;success("delivery-status",package::status(&path(args,"--directory")?,sha,token)?,Outcome::Complete).map(Some)},
        _=>Ok(None),
    }
}
