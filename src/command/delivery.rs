//! Closed argv and presentation delivery over the public command SDK.
use super::*;
use std::{collections::BTreeMap,ffi::OsString};
#[path = "native_delivery.rs"]
mod native_delivery;
#[path = "distribution_delivery.rs"]
mod distribution_delivery;
#[path = "preparation_delivery.rs"]
mod preparation_delivery;
#[path = "pilot_delivery.rs"]
mod pilot_delivery;
#[path = "release_delivery.rs"]
mod release_delivery;
#[path = "publisher_delivery.rs"]
mod publisher_delivery;
#[path = "project_delivery.rs"]
mod project_delivery;
pub const COMMANDS: &[&str] = &["doctor","release-contract","release-matrix","release-evidence","publisher-verify","suite-install-released","approve-released","run-released","status-released","resume-released","prepare-assets","preparation-status","project-preview","project-prepare","project-status","pilot-plan","pilot-approve","pilot-run","pilot-status","pilot-continue","suite-install","suite-bundle","suite-status","suite-activate","suite-rollback","suite-uninstall","inspect","plan","approve","run","status","resume","help","version"];

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Presentation {Human,Json}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Arguments {pub command:String,pub presentation:Presentation,pub options:BTreeMap<String,OsString>,pub acknowledge_trusted_unconfined:bool,pub acknowledge_unqualified_candidate:bool,pub acknowledge_verification_network:bool}
pub fn parse(values:Vec<OsString>)->Result<Arguments,Error>{
    if values.len()>32||values.iter().map(|v|v.len()).sum::<usize>()>65_536{return Err(error(Code::Usage,"command arguments exceed bounds"));}
    let mut values=values.into_iter();let command=match values.next(){None=>"help".into(),Some(value)=>value.into_string().map_err(|_|error(Code::Usage,"recognized UTF-8 command name required"))?};
    let command=match command.as_str(){"--help"|"-h"=>"help".into(),"--version"|"-V"=>"version".into(),_=>command};
    if !COMMANDS.contains(&command.as_str()){return Err(error(Code::Usage,"recognized Flow command required; use help for the command inventory"));}
    let mut options=BTreeMap::new();let mut ack=false;let mut candidate_ack=false;let mut verification_ack=false;let mut presentation=Presentation::Human;let mut format_seen=false;
    while let Some(key)=values.next(){let key=key.into_string().map_err(|_|error(Code::Usage,"recognized UTF-8 option name required"))?;
        if key=="--acknowledge-trusted-unconfined"{if ack||!["approve","pilot-approve","approve-released"].contains(&command.as_str()){return Err(error(Code::Usage,"process acknowledgement belongs once to approve"));}ack=true;continue;}
        if key=="--acknowledge-unqualified-candidate"{if candidate_ack||!["approve","pilot-approve","suite-install"].contains(&command.as_str()){return Err(error(Code::Usage,"candidate acknowledgement belongs once to approve or suite-install"));}candidate_ack=true;continue;}
        if key=="--acknowledge-verification-network"{if verification_ack||!["release-evidence","release-matrix","publisher-verify","suite-install-released","approve-released","run-released","status-released","resume-released"].contains(&command.as_str()){return Err(error(Code::Usage,"verification network acknowledgement belongs once to an explicit release-evidence operation"));}verification_ack=true;continue;}
        let value=values.next().ok_or_else(||error(Code::Usage,"option value required"))?;
        if key=="--format"{if format_seen{return Err(error(Code::Usage,"duplicate presentation option"));}format_seen=true;presentation=match value.to_str(){Some("json")=>Presentation::Json,Some("human")=>Presentation::Human,_=>return Err(error(Code::Usage,"format must be human or json"))};continue;}
        let allowed:&[&str]=match command.as_str(){"project-preview"|"project-prepare"=>&["--project"],"project-status"=>&["--workspace","--record-sha256"],"publisher-verify"=>&["--publisher-request","--verification-directory","--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"],"suite-install-released"=>&["--request","--bundle","--bundle-sha256","--install-directory","--publisher-request","--verification-directory","--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"],"release-evidence"=>&["--evidence-request","--verification-directory","--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"],"approve-released"=>&["--plan","--approved-by","--output","--evidence-request","--verification-directory","--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"],"run-released"=>&["--plan","--approval","--evidence-request","--verification-directory","--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"],"status-released"=>&["--run-directory","--evidence-request","--verification-directory","--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"],"resume-released"=>&["--run-directory","--native-run-directory","--evidence-request","--verification-directory","--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"],"pilot-plan"=>&["--request","--output"],"pilot-approve"=>&["--plan","--approved-by","--output"],"pilot-run"|"pilot-continue"=>&["--plan","--approval"],"pilot-status"=>&["--run-directory"],"prepare-assets"=>&["--request","--output-directory"],"preparation-status"=>&["--preparation-directory"],"suite-install"=>&["--request","--bundle","--bundle-sha256","--install-directory"],"suite-bundle"=>&["--request","--output-directory"],"suite-status"=>&["--installation-root","--receipt-sha256"],"suite-activate"|"suite-rollback"=>&["--installation-root","--receipt-sha256","--selection","--previous-selection-sha256"],"suite-uninstall"=>&["--installation-root","--receipt-sha256","--removal-record"],"doctor"=>&["--provider-executable","--cohort","--installations","--preparation-directory","--installation-root","--receipt-sha256"],"release-matrix"=>&["--cohort","--installations","--evidence-request","--verification-directory","--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"],"inspect"=>&["--request","--input","--profile"],"plan"=>&["--request","--input","--profile","--provider-executable","--run-directory","--output","--native-selection"],"approve"=>&["--plan","--approved-by","--output"],"run"=>&["--plan","--approval"],"status"=>&["--run-directory"],"resume"=>&["--run-directory","--native-run-directory"],_=>&[]};
        if !allowed.contains(&key.as_str())||options.insert(key,value).is_some(){return Err(error(Code::Usage,"unknown or duplicate command option"));}
    }
    let required:&[&str]=match command.as_str(){"project-preview"|"project-prepare"=>&["--project"],"project-status"=>&["--workspace","--record-sha256"],"publisher-verify"=>&["--publisher-request","--verification-directory","--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"],"suite-install-released"=>&["--publisher-request","--verification-directory","--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"],"release-evidence"=>&["--evidence-request","--verification-directory","--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"],"approve-released"=>&["--plan","--approved-by","--output","--evidence-request","--verification-directory","--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"],"run-released"=>&["--plan","--approval","--evidence-request","--verification-directory","--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"],"status-released"=>&["--run-directory","--evidence-request","--verification-directory","--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"],"resume-released"=>&["--run-directory","--native-run-directory","--evidence-request","--verification-directory","--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"],"pilot-plan"=>&["--request"],"pilot-approve"=>&["--plan","--approved-by","--output"],"pilot-run"|"pilot-continue"=>&["--plan","--approval"],"pilot-status"=>&["--run-directory"],"prepare-assets"=>&["--request","--output-directory"],"preparation-status"=>&["--preparation-directory"],"suite-bundle"=>&["--request","--output-directory"],"suite-status"=>&["--installation-root","--receipt-sha256"],"suite-activate"=>&["--installation-root","--receipt-sha256","--selection"],"suite-rollback"=>&["--installation-root","--receipt-sha256","--selection","--previous-selection-sha256"],"suite-uninstall"=>&["--installation-root","--receipt-sha256","--removal-record"],"release-matrix"=>&["--cohort"],"inspect"|"plan"=>&[],"approve"=>&["--plan","--approved-by","--output"],"run"=>&["--plan","--approval"],"status"|"resume"=>&["--run-directory"],_=>&[]};
    if required.iter().any(|k|!options.contains_key(*k)){return Err(error(Code::Usage,"required long-form command option missing"));}
    if ["inspect","plan"].contains(&command.as_str()) {
        if options.contains_key("--request")==options.contains_key("--input"){return Err(error(Code::Usage,"select exactly one --request document or --input path"));}
        if options.contains_key("--request")&&["--profile","--provider-executable","--run-directory","--native-selection"].iter().any(|k|options.contains_key(*k)){return Err(error(Code::Usage,"request document cannot be mixed with local-input overrides"));}
        if options.contains_key("--native-selection")&&["--provider-executable","--run-directory"].iter().any(|k|options.contains_key(*k)){return Err(error(Code::Usage,"native selection already binds provider and output; overrides refused"));}
    }
    if options.contains_key("--installations")&&!options.contains_key("--cohort"){return Err(error(Code::Usage,"installation observations require an explicit cohort"));}
    if ["suite-install","suite-install-released"].contains(&command.as_str()) {let file=options.contains_key("--request");let bundle=options.contains_key("--bundle");if file==bundle||file&&["--bundle-sha256","--install-directory"].iter().any(|k|options.contains_key(*k))||bundle&&["--bundle-sha256","--install-directory"].iter().any(|k|!options.contains_key(*k)){return Err(error(Code::Usage,"suite-install selects an exact request or a pinned bundle plus explicit installation directory"));}}
    if command=="doctor" && options.contains_key("--installation-root") != options.contains_key("--receipt-sha256") {return Err(error(Code::Usage,"doctor installation root and receipt digest belong together"));}
    if command=="release-matrix"&&(options.contains_key("--evidence-request")!=options.contains_key("--verification-directory")||options.contains_key("--evidence-request")&&options.contains_key("--installations")||verification_ack&&!options.contains_key("--evidence-request")){return Err(error(Code::Usage,"authenticated matrix requires its exact evidence request and fresh verification directory; local installation selection is separate"));}
    if options.contains_key("--evidence-request")&&["--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"].iter().any(|k|!options.contains_key(*k)){return Err(error(Code::Usage,"authenticated evidence requires independent root host policy and verifier pins"));}
    if command=="release-matrix"&&!options.contains_key("--evidence-request")&&["--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"].iter().any(|k|options.contains_key(*k)){return Err(error(Code::Usage,"host trust options require an explicit evidence request"));}
    if options.contains_key("--publisher-request")&&["--verification-policy","--policy-sha256","--verifier-executable","--verifier-sha256"].iter().any(|k|!options.contains_key(*k)){return Err(error(Code::Usage,"publisher verification requires independent root host trust pins"));}
    Ok(Arguments {command,presentation,options,acknowledge_trusted_unconfined:ack,acknowledge_unqualified_candidate:candidate_ack,acknowledge_verification_network:verification_ack})
}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Outcome {Complete,Partial,Unavailable,RecoveryRequired,Refused}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {pub schema_version:String,pub command:String,pub outcome:Outcome,pub exit_code:u8,pub result:Option<serde_json::Value>,pub diagnostics:Vec<Error>}
pub fn failure(command:&str,value:Error)->Envelope {Envelope {schema_version:"flow.command-result/v1".into(),command:command.into(),outcome:match value.code {Code::Unavailable=>Outcome::Unavailable,Code::RecoveryRequired=>Outcome::RecoveryRequired,_=>Outcome::Refused},exit_code:value.exit_code(),result:None,diagnostics:vec![value]}}
fn success<T:Serialize>(command:&str,value:T,outcome:Outcome)->Result<Envelope,Error>{Ok(Envelope {schema_version:"flow.command-result/v1".into(),command:command.into(),exit_code:match outcome{Outcome::Complete=>0,Outcome::Partial=>9,Outcome::Unavailable=>4,Outcome::RecoveryRequired=>7,Outcome::Refused=>6},outcome,result:Some(serde_json::to_value(value).map_err(|_|error(Code::Io,"command result cannot be encoded"))?),diagnostics:vec![]})}
fn path(args:&Arguments,name:&str)->Result<PathBuf,Error>{absolute(Path::new(args.options.get(name).ok_or_else(||error(Code::Usage,"required command option missing"))?))}
fn saved_plan(path:&Path)->Result<Plan,Error>{Plan::from_json_slice(&read_document(path,32_768)?)}
fn save<T:Serialize>(plan:&Plan,path:&Path,value:&T)->Result<(),Error>{let source=selected_path(&plan.request.source_root)?;let provider=selected_path(&plan.request.provider_executable)?;if path.starts_with(source)||path==provider{return Err(error(Code::InvalidInput,"operation documents must be separate from source/provider"));}write_document(path,value)}
fn state(value:&Status)->Outcome{match value.state{State::Complete=>Outcome::Complete,State::Partial=>Outcome::Partial,State::RecoveryRequired=>Outcome::RecoveryRequired,State::Refused=>Outcome::Refused}}
fn local_request(args:&Arguments)->Result<crate::local_input::Request,Error>{
    use crate::local_input::{Input,Profile,Limits};
    let input=path(args,"--input")?;let metadata=std::fs::symlink_metadata(&input).map_err(|_|error(Code::InvalidInput,"explicit local input is unavailable"))?;
    let input=if metadata.is_dir(){Input::Directory {path:NativePath::from_path(&input),recursive:true}}else if metadata.is_file(){Input::File {path:NativePath::from_path(&input)}}else{return Err(error(Code::InvalidInput,"regular local file or directory required; links/special nodes refused"))};
    let profile=match args.options.get("--profile").map(|v|v.to_str()){None|Some(Some("inspect-only"))=>Profile::InspectOnly,Some(Some("optiflow-read-only"))=>Profile::OptiflowReadOnly,Some(Some("renderflow-publication"))=>Profile::RenderflowPublication,Some(Some("aniflow-temporal"))=>Profile::AniflowTemporal,_=>return Err(error(Code::Usage,"recognized explicit local profile required"))};
    Ok(crate::local_input::Request {schema_version:"flow.local-input-request/v1".into(),input,profile,provider_executable:args.options.get("--provider-executable").map(|_|path(args,"--provider-executable").map(|p|NativePath::from_path(&p))).transpose()?,run_directory:args.options.get("--run-directory").map(|_|path(args,"--run-directory").map(|p|NativePath::from_path(&p))).transpose()?,limits:Limits::default()})
}
fn local(args:&Arguments,request:crate::local_input::Request,token:&dyn CancellationSignal)->Result<Envelope,Error>{
    if args.command=="inspect" {return success("inspect",crate::local_input::inspect(&request,token)?,Outcome::Complete);}
    let value=crate::local_input::plan(&request,token)?;
    let outcome=match value.availability {crate::local_input::Availability::InspectionOnly=>Outcome::Complete,crate::local_input::Availability::ScopePlanOnly=>Outcome::Partial,crate::local_input::Availability::Unavailable=>Outcome::Unavailable};
    if let Some(output)=args.options.get("--output") {
        let output=absolute(Path::new(output))?;
        if let Some(plan)=&value.command_plan {save(plan,&output,plan)?;}
        else if value.availability==crate::local_input::Availability::InspectionOnly {
            for file in &value.inventory.files {if file.path.path().map_err(native)?==output{return Err(error(Code::InvalidInput,"inspection snapshot output equals a selected source"));}}
            for directory in &value.inventory.directories {if output.starts_with(directory.path.path().map_err(native)?){return Err(error(Code::InvalidInput,"inspection snapshot output is inside selected directory scope"));}}
            write_document(&output,&value)?;
        }
    }
    success("plan",value,outcome)
}

/// Ordinary invocation only. Planning and diagnostics never invoke a provider;
/// explicit run/resume retain the SDK's authority and recovery boundaries.
pub fn execute(args:&Arguments,token:&dyn CancellationSignal)->Envelope {
    let run=(||->Result<Envelope,Error>{cancelled(token)?;if let Some(value)=project_delivery::execute(args,token)?{return Ok(value);}if let Some(value)=publisher_delivery::execute(args,token)?{return Ok(value);}if let Some(value)=release_delivery::execute(args,token)?{return Ok(value);}if let Some(value)=pilot_delivery::execute(args,token)?{return Ok(value);}if let Some(value)=preparation_delivery::execute(args,token)?{return Ok(value);}if let Some(value)=distribution_delivery::execute(args,token)?{return Ok(value);}if let Some(value)=native_delivery::execute(args,token)?{return Ok(value);}if args.acknowledge_unqualified_candidate{return Err(error(Code::Usage,"candidate acknowledgement belongs to a native candidate plan"));}match args.command.as_str(){
        "help"=>success("help",serde_json::json!({"usage":"flow COMMAND [long-form options] [--format human|json]","commands":COMMANDS,"released_route":"Optiflow v0.1.1 read-only scan/report/review-plan","candidate_routes":crate::native::SOURCE_ROUTES,"qualification":"source checkpoint; installed/published/native evidence remains separate"}),Outcome::Complete),
        "version"=>success("version",serde_json::json!({"package":env!("CARGO_PKG_VERSION"),"command_contract":"flow.command-result/v1","release_qualified":false}),Outcome::Complete),
        "release-contract"=>success("release-contract",crate::release::describe(),Outcome::Complete),
        "doctor"=>{let executable=args.options.get("--provider-executable").map(|_|path(args,"--provider-executable")).transpose()?;let value=doctor(executable.as_deref(),token)?;let outcome=if value.selected_provider.is_some(){Outcome::Complete}else{Outcome::Unavailable};success("doctor",value,outcome)},
        "inspect"|"plan"=>{
            if args.options.contains_key("--input"){return local(args,local_request(args)?,token);}
            let bytes=read_document(&path(args,"--request")?,131_072)?;
            let value:serde_json::Value=serde_json::from_slice(&bytes).map_err(|_|error(Code::InvalidInput,"closed input request JSON required"))?;
            if value["schema_version"]=="flow.local-input-request/v1"{return local(args,crate::local_input::Request::from_json_slice(&bytes)?,token);}
            let request=Request::from_json_slice(&bytes)?;let value=plan(&request,token)?;if args.options.contains_key("--output"){save(&value,&path(args,"--output")?,&value)?;}success(&args.command,value,Outcome::Complete)
        },
        "approve"=>{let plan=saved_plan(&path(args,"--plan")?)?;let by=args.options.get("--approved-by").and_then(|v|v.to_str()).ok_or_else(||error(Code::Usage,"bounded UTF-8 operator declaration required"))?.to_owned();let value=approve(&plan,by,args.acknowledge_trusted_unconfined)?;save(&plan,&path(args,"--output")?,&value)?;success("approve",value,Outcome::Complete)},
        "run"=>{let plan=saved_plan(&path(args,"--plan")?)?;let approval:Approval=serde_json::from_slice(&read_document(&path(args,"--approval")?,16_384)?).map_err(|_|error(Code::InvalidInput,"closed supplied operation approval required"))?;let value=super::run(&plan,&approval,token)?;let outcome=state(&value);success("run",value,outcome)},
        "status"|"resume"=>{if args.options.contains_key("--native-run-directory"){return Err(error(Code::Usage,"native run selection belongs to a native candidate operation"));}let root=path(args,"--run-directory")?;let value=(if args.command=="status"{status(&root,token)}else{resume(&root,token)}).map_err(|mut e|{e.recovery_directory=Some(NativePath::from_path(&root));e})?;let outcome=state(&value);success(&args.command,value,outcome)},
        _=>Err(error(Code::Usage,"unknown command")),
    }})();match run{Ok(value)=>value,Err(value)=>failure(&args.command,value)}
}
pub fn render(value:&Envelope,presentation:Presentation)->Result<Vec<u8>,Error>{let mut bytes=match presentation {
    Presentation::Json=>serde_json::to_vec(value).map_err(|_|error(Code::Io,"machine result unavailable"))?,
    Presentation::Human=>{let mut text=format!("{}: {:?}\n",value.command,value.outcome);if value.command=="help"{text.push_str(&format!("flow COMMAND\nCommands: {}\nUse --format json for one versioned result. Source checkpoint; release acceptance remains open.\n",COMMANDS.join(", ")));}for diagnostic in &value.diagnostics{text.push_str(&format!("{:?}: {}\n",diagnostic.code,diagnostic.message));}if let Some(result)=&value.result{if result["schema_version"]=="flow.release-matrix/v1"{text.push_str(&native_delivery::human_matrix(result));}else{text.push_str(&serde_json::to_string_pretty(result).map_err(|_|error(Code::Io,"human result unavailable"))?);}}text.into_bytes()},
    };if bytes.len()>33_554_432{return Err(error(Code::Io,"command result exceeds32MiB"));}bytes.push(b'\n');Ok(bytes)}

#[cfg(test)] mod tests {
    use super::*;
    #[cfg(unix)] #[test] fn invalid_command_bytes_are_usage_errors_without_losing_native_input_paths(){use std::os::unix::ffi::OsStringExt;let bad=OsString::from_vec(vec![0xff]);assert_eq!(parse(vec![bad.clone()]).unwrap_err().code,Code::Usage);let parsed=parse(vec!["inspect".into(),"--input".into(),bad.clone()]).unwrap();assert_eq!(parsed.options["--input"],bad);}
    #[test] fn parser_requires_supplied_plan_approval_and_explicit_acknowledgement_surface(){assert!(parse(["run","--plan","p"].into_iter().map(Into::into).collect()).is_err());assert!(parse(["run","--plan","p","--approval","a","--format","json"].into_iter().map(Into::into).collect()).is_ok());assert!(parse(["run","--plan","p","--approval","a","--acknowledge-trusted-unconfined"].into_iter().map(Into::into).collect()).is_err());}
    #[test] fn planning_refuses_implicit_execution_install_and_duplicate_options(){for flag in ["--execute","--install","--download","--post"]{assert!(parse(["plan","--request","r",flag,"x"].into_iter().map(Into::into).collect()).is_err());}assert!(parse(["plan","--request","r","--request","s"].into_iter().map(Into::into).collect()).is_err());}
    #[test] fn machine_failure_is_one_closed_result_without_private_error_text(){let value=failure("run",error(Code::Unavailable,"selected provider unavailable"));let bytes=render(&value,Presentation::Json).unwrap();let json:serde_json::Value=serde_json::from_slice(&bytes).unwrap();assert_eq!(json["schema_version"],"flow.command-result/v1");assert_eq!(json["outcome"],"unavailable");assert!(!String::from_utf8(bytes).unwrap().contains("/private"));}
    #[test] fn ordinary_input_flags_do_not_mix_with_request_or_hidden_overrides(){assert!(parse(["inspect","--input","file with spaces"].into_iter().map(Into::into).collect()).is_ok());assert!(parse(["plan","--input","folder","--profile","optiflow-read-only","--provider-executable","/selected/opti","--run-directory","/separate/new"].into_iter().map(Into::into).collect()).is_ok());assert!(parse(["inspect","--input","file","--request","r"].into_iter().map(Into::into).collect()).is_err());assert!(parse(["plan","--request","r","--provider-executable","/override"].into_iter().map(Into::into).collect()).is_err());}
}
