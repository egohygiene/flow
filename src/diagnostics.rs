//! Local generated metadata only. No environment, content, log reader or network.
use std::{collections::BTreeSet,path::Path};
use serde::{Serialize,Deserialize};
use crate::{CancellationSignal,command::{self,Code,Error},release_cohort};

const MAX:u64=65_536;
const CONTRACTS:&[&str]=&["flow.command-result/v1","flow.command-request/v1","flow.release-matrix/v1"];
const EXCLUSIONS:&[&str]=&["media_and_documents","prompts_lyrics_captions_transcripts","environment_and_credentials","argv_and_child_captures","paths_and_hostnames","raw_logs_traces_metrics_backtraces","durable_run_artifacts"];
fn error(code:Code,s:&str)->Error{Error {code,message:s.into(),recovery_directory:None}}
fn check(token:&dyn CancellationSignal)->Result<(),Error>{if token.is_cancelled(){Err(error(Code::Cancelled,"diagnostics operation cancelled; preserve any created bundle"))}else{Ok(())}}
fn encoded<T:Serialize>(value:&T)->Result<Vec<u8>,Error>{serde_json::to_vec(value).map_err(|_|error(Code::InvalidInput,"diagnostics metadata encoding"))}
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Category {Build,Platform,Contracts}
impl Category{fn member(self)->&'static str{match self{Self::Build=>"build.json",Self::Platform=>"platform.json",Self::Contracts=>"contracts.json"}}}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request{pub schema_version:String,pub categories:Vec<Category>}
impl Request{
    pub fn from_json_slice(bytes:&[u8])->Result<Self,Error>{
        if bytes.len()>4096{return Err(error(Code::InvalidInput,"diagnostic request exceeds4KiB"));}
        let r:Self=serde_json::from_slice(bytes).map_err(|_|error(Code::InvalidInput,"closed diagnostic category request required"))?;r.validate()?;Ok(r)
    }
    fn validate(&self)->Result<(),Error>{if self.schema_version!="flow.diagnostics-request/v1"||self.categories.is_empty()||self.categories.len()>3||self.categories.iter().collect::<BTreeSet<_>>().len()!=self.categories.len(){return Err(error(Code::InvalidInput,"select one to three unique generated metadata categories"));}Ok(())}
}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(tag="category",rename_all="snake_case",deny_unknown_fields)]
enum Payload{Build{tool:String,version:String},Platform{compiled_os:String,compiled_arch:String},Contracts{source_contract_ids:Vec<String>}}
impl Payload{
    fn category(&self)->Category{match self{Self::Build{..}=>Category::Build,Self::Platform{..}=>Category::Platform,Self::Contracts{..}=>Category::Contracts}}
    fn validate(&self)->Result<(),Error>{
        let atom=|s:&str|!s.is_empty()&&s.len()<=64&&s.bytes().all(|b|b.is_ascii_alphanumeric()||b"_.-".contains(&b));
        let valid=match self{
            Self::Build{tool,version}=>tool=="flow"&&version.len()<=64&&semver::Version::parse(version).is_ok(),
            Self::Platform{compiled_os,compiled_arch}=>atom(compiled_os)&&atom(compiled_arch),
            Self::Contracts{source_contract_ids}=>source_contract_ids.iter().map(String::as_str).eq(CONTRACTS.iter().copied()),
        };
        if valid{Ok(())}else{Err(error(Code::InvalidInput,"unsupported support metadata payload"))}
    }
}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Member{name:String,sha256:String,byte_count:u64,payload:Payload}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Archive{schema_version:String,redaction_policy:String,domain_evidence:bool,members:Vec<Member>}
impl Archive{
    fn validate(&self)->Result<(),Error>{
        if self.schema_version!="flow.diagnostics-bundle/v1"||self.redaction_policy!="generated_metadata_only.v1"||self.domain_evidence||self.members.is_empty()||self.members.len()>3{return Err(error(Code::InvalidInput,"unsupported or unbounded diagnostics bundle"));}
        let mut categories=BTreeSet::new();
        for member in &self.members{
            member.payload.validate()?;let data=encoded(&member.payload)?;
            if member.name!=member.payload.category().member()||!categories.insert(member.payload.category())
                ||data.len()>4096||member.byte_count!=data.len() as u64||member.sha256!=release_cohort::sha256(&data){return Err(error(Code::InvalidInput,"diagnostic member identity, path or bound differs"));}
        }Ok(())
    }
}
fn archive(request:&Request)->Result<Archive,Error>{
    request.validate()?;let mut members=vec![];
    for category in &request.categories{
        let payload=match category{
            Category::Build=>Payload::Build {tool:"flow".into(),version:env!("CARGO_PKG_VERSION").into()},
            Category::Platform=>Payload::Platform {compiled_os:std::env::consts::OS.into(),compiled_arch:std::env::consts::ARCH.into()},
            Category::Contracts=>Payload::Contracts {source_contract_ids:CONTRACTS.iter().map(|s|(*s).into()).collect()},
        };
        let data=encoded(&payload)?;members.push(Member{name:category.member().into(),sha256:release_cohort::sha256(&data),byte_count:data.len() as u64,payload});
    }
    let value=Archive{schema_version:"flow.diagnostics-bundle/v1".into(),redaction_policy:"generated_metadata_only.v1".into(),domain_evidence:false,members};value.validate()?;Ok(value)
}
fn document(value:&Archive)->Result<Vec<u8>,Error>{let mut data=serde_json::to_vec_pretty(value).map_err(|_|error(Code::InvalidInput,"diagnostic bundle encoding"))?;data.push(b'\n');if data.len() as u64>MAX{return Err(error(Code::InvalidInput,"diagnostic bundle exceeds64KiB"));}Ok(data)}
#[derive(Debug,Serialize)]
pub struct Preview{pub schema_version:String,pub categories:Vec<Category>,pub member_count:usize,pub archive_bytes:u64,pub exclusions:Vec<&'static str>,pub files_written:bool,pub network_contacted:bool,pub domain_evidence:bool}
pub fn preview(request:&Request,token:&dyn CancellationSignal)->Result<Preview,Error>{check(token)?;let value=archive(request)?;let size=document(&value)?.len() as u64;check(token)?;Ok(Preview{schema_version:"flow.diagnostics-preview/v1".into(),categories:request.categories.clone(),member_count:value.members.len(),archive_bytes:size,exclusions:EXCLUSIONS.to_vec(),files_written:false,network_contacted:false,domain_evidence:false})}
#[derive(Debug,Serialize)]
pub struct Inspection{pub schema_version:String,pub archive_sha256:String,pub archive_bytes:u64,pub categories:Vec<Category>,pub member_count:usize,pub metadata_claims_authenticated:bool,pub domain_evidence:bool,pub network_contacted:bool}
/// Parse a bounded container without extraction, embedded execution or trust in
/// supplied values. The summary does not echo untrusted payload text or paths.
pub fn inspect(path:&Path,token:&dyn CancellationSignal)->Result<Inspection,Error>{
    check(token)?;let data=command::read_document(path,MAX)?;
    let value:Archive=serde_json::from_slice(&data).map_err(|_|error(Code::InvalidInput,"closed diagnostic bundle required"))?;value.validate()?;check(token)?;
    Ok(Inspection {schema_version:"flow.diagnostics-inspection/v1".into(),archive_sha256:release_cohort::sha256(&data),archive_bytes:data.len() as u64,categories:value.members.iter().map(|m|m.payload.category()).collect(),member_count:value.members.len(),metadata_claims_authenticated:false,domain_evidence:false,network_contacted:false})
}
/// Explicit local no-clobber creation. A failed write leaves its partial file;
/// it is never adopted, overwritten, uploaded or used as domain run evidence.
pub fn bundle(request:&Request,output:&Path,token:&dyn CancellationSignal)->Result<Inspection,Error>{
    check(token)?;let value=archive(request)?;let expected=document(&value)?;check(token)?;
    command::write_document(output,&value)?;let observed=inspect(output,token)?;
    if observed.archive_sha256!=release_cohort::sha256(&expected){return Err(error(Code::StalePlan,"created diagnostic bundle differs"));}Ok(observed)
}

#[cfg(test)]mod tests{
    use super::*;
    fn request()->Request{Request {schema_version:"flow.diagnostics-request/v1".into(),categories:vec![Category::Build,Category::Platform,Category::Contracts]}}
    #[test]fn unselected_private_content_has_no_input_surface(){
        let mut raw=serde_json::to_value(request()).unwrap();raw["raw_logs"]=serde_json::json!("PRIVATE_CANARY_NEVER_RETAINED");
        assert!(Request::from_json_slice(&serde_json::to_vec(&raw).unwrap()).is_err());
        let data=document(&archive(&request()).unwrap()).unwrap();assert!(!String::from_utf8(data).unwrap().contains("PRIVATE_CANARY_NEVER_RETAINED"));
        let value=preview(&request(),&crate::NeverCancelled).unwrap();assert!(!value.files_written&&!value.network_contacted&&!value.domain_evidence);
    }
    #[test]fn tampered_members_traversal_duplicates_and_authority_claims_refuse(){
        let value=archive(&request()).unwrap();let mut changed=value.clone();changed.members[0].sha256="a".repeat(64);assert!(changed.validate().is_err());
        let mut changed=value.clone();changed.members[0].name="../outside".into();assert!(changed.validate().is_err());
        let mut changed=value.clone();changed.members[1]=changed.members[0].clone();assert!(changed.validate().is_err());
        let mut changed=value;changed.domain_evidence=true;assert!(changed.validate().is_err());
    }
    #[test]fn explicitly_selected_output_is_never_overwritten(){
        let root=std::env::temp_dir().canonicalize().unwrap();let path=root.join(format!("flow-diagnostics-{}-{}.json",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::write(&path,b"retain existing bytes").unwrap();let result=bundle(&request(),&path,&crate::NeverCancelled);
        assert!(result.is_err());assert_eq!(std::fs::read(&path).unwrap(),b"retain existing bytes");std::fs::remove_file(path).unwrap();
    }
}
