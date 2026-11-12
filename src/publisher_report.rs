//! Portable publisher declarations. Parsing never authenticates a report or
//! establishes native readiness; only the separate verifier creates authority.
use std::collections::BTreeSet;
use serde::{Deserialize,Serialize};
use crate::{command::{Code,Error},release_cohort::{self,Interface,Row,Tool}};

fn error(text:&str)->Error{Error{code:Code::InvalidInput,message:text.into(),recovery_directory:None}}
pub(crate) fn tool_name(tool:Tool)->&'static str{match tool{Tool::Flow=>"flow",Tool::Aniflow=>"aniflow",Tool::Renderflow=>"renderflow",Tool::Optiflow=>"optiflow"}}
pub(crate) fn safe_member(name:&str)->bool{!name.is_empty()&&name.len()<=4096&&!name.contains('\\')&&!name.chars().any(char::is_control)&&name.split('/').all(|s|!s.is_empty()&&s!="."&&s!="..")}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum AssetKind{TarGz,RawBinary}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Archive{pub sha256:String,pub byte_count:u64,pub name:String}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Publication{pub release_url:String,pub release_id:u64,pub asset_id:u64}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sidecar{pub release_url:String,pub asset_id:u64}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contract{pub identifier:String,pub member:Option<String>,pub sha256:String,pub byte_count:u64,pub publication:Option<Sidecar>}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Member{pub name:String,pub sha256:String,pub byte_count:u64}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report{
    pub schema_version:String,pub tool:Tool,pub cohort_sha256:String,pub row_subject_sha256:String,
    pub asset_kind:AssetKind,pub version:String,pub source_revision:String,pub target:String,
    pub archive:Archive,pub publication:Publication,pub executable_member:Option<String>,
    pub contracts:Vec<Contract>,pub members:Vec<Member>,
}
impl Report{
    pub fn from_json_slice(bytes:&[u8])->Result<Self,Error>{if bytes.len()>1_048_576{return Err(error("publisher report exceeds 1 MiB"));}serde_json::from_slice(bytes).map_err(|_|error("closed portable publisher report required"))}
    pub fn validate_for(&self,row:&Row,cohort_sha256:&str)->Result<(),Error>{
        let artifact=row.artifact.as_ref().ok_or_else(||error("publisher row lacks immutable artifact"))?;
        let subject=release_cohort::subject(row).map_err(|_|error("publisher row subject unavailable"))?;
        let prefix=format!("https://github.com/egohygiene/{}/releases/download/v{}/",tool_name(row.tool),artifact.version);
        if self.schema_version!="flow.publisher-artifact-report/v1"||self.tool!=row.tool||self.cohort_sha256!=cohort_sha256||!release_cohort::is_digest(cohort_sha256)||self.row_subject_sha256!=subject||self.version!=artifact.version||self.source_revision!=artifact.source_revision||self.target!=row.target||row.implementation!=release_cohort::Implementation::Implemented
            ||!release_cohort::is_digest(&self.archive.sha256)||self.archive.byte_count==0||self.archive.byte_count>536_870_912||!safe_member(&self.archive.name)||self.archive.name.contains('/')||self.publication.release_id==0||self.publication.asset_id==0||self.publication.release_url!=artifact.release_url||!self.publication.release_url.starts_with(&prefix)||self.members.is_empty()||self.members.len()>8192||self.contracts.len()!=row.contracts.len(){return Err(error("publisher identity, release or inventory differs from exact row"));}
        let mut names=BTreeSet::new();let mut total=0_u64;
        for member in &self.members{if !safe_member(&member.name)||!names.insert(&member.name)||!release_cohort::is_digest(&member.sha256)||member.byte_count>536_870_912{return Err(error("publisher member identity/bound"));}total=total.checked_add(member.byte_count).ok_or_else(||error("publisher expanded byte overflow"))?;}
        if total>1_073_741_824{return Err(error("publisher expanded byte bound"));}
        match (&self.executable_member,row.interface){
            (None,Interface::RustLibrary)=>{if self.asset_kind!=AssetKind::TarGz||artifact.sha256!=self.archive.sha256||artifact.byte_count!=self.archive.byte_count{return Err(error("library archive differs from cohort"));}},
            (Some(name),interface) if interface!=Interface::RustLibrary=>{
                if !self.members.iter().any(|m|&m.name==name&&m.sha256==artifact.sha256&&m.byte_count==artifact.byte_count)||name.rsplit('/').next()!=Some(tool_name(row.tool)){return Err(error("publisher executable member differs"));}
                if self.asset_kind==AssetKind::RawBinary&&(self.members.len()!=1||name!=tool_name(row.tool)||artifact.sha256!=self.archive.sha256||artifact.byte_count!=self.archive.byte_count){return Err(error("raw binary must bind the complete published asset"));}
            },
            _=>return Err(error("publisher interface/executable binding differs")),
        }
        let mut contracts=BTreeSet::new();
        for contract in &self.contracts{
            if !contracts.insert(&contract.identifier)||contract.byte_count==0||contract.byte_count>1_048_576||!row.contracts.iter().any(|c|c.identifier==contract.identifier&&c.sha256==contract.sha256){return Err(error("publisher contract differs from cohort"));}
            match (&contract.member,&contract.publication){
                (Some(name),None) if self.asset_kind==AssetKind::TarGz=>{if !self.members.iter().any(|m|&m.name==name&&m.sha256==contract.sha256&&m.byte_count==contract.byte_count){return Err(error("publisher contract member differs"));}},
                (None,Some(asset)) if asset.asset_id>0&&asset.release_url.starts_with(&prefix)=>{},
                _=>return Err(error("contract must bind an archive member or a separately published asset")),
            }
        }Ok(())
    }
}
