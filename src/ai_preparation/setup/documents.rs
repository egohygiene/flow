//! New private setup documents and read-only current diagnostics. Source utility
//! delivery remains separate from the supported orchestration CLI/release.
use super::*;
use std::{fs::{File,OpenOptions},io::{Read,Write},path::Path};
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Member {pub name:String,pub byte_count:u64,pub sha256:String}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Marker {pub schema_version:String,pub profile_sha256:String,pub recorded_state:State,pub members:Vec<Member>}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleStatus {pub schema_version:String,pub profile_sha256:String,pub recorded_state:State,pub current:Doctor,pub documents_match:bool}
fn io_error(_:impl std::fmt::Display)->ValidationError{error("private setup document IO unavailable")}
fn sha(bytes:&[u8])->String{format!("{:x}",Sha256::digest(bytes))}
fn bytes<T:Serialize>(value:&T)->Result<Vec<u8>,ValidationError>{let mut data=serde_json::to_vec_pretty(value).map_err(|_|error("setup document serialization unavailable"))?;data.push(b'\n');if data.len()>131_072{return Err(error("setup document exceeds128KiB"));}Ok(data)}
fn directory(path:&Path)->Result<File,ValidationError>{if !path.is_absolute()||path.canonicalize().ok().as_deref()!=Some(path){return Err(error("canonical absolute private directory required"));}let file=File::open(path).map_err(io_error)?;let metadata=file.metadata().map_err(io_error)?;if !metadata.is_dir(){return Err(error("private setup directory required"));}
    #[cfg(unix)] {use std::os::unix::fs::PermissionsExt;if metadata.permissions().mode()&0o077!=0{return Err(error("setup directory must exclude group/other access"));}}
    Ok(file)
}
pub fn read(path:&Path,maximum:u64)->Result<Vec<u8>,ValidationError>{if maximum==0||maximum>131_072||!path.is_absolute()||path.canonicalize().ok().as_deref()!=Some(path){return Err(error("explicit canonical bounded document required"));}
    let before=std::fs::symlink_metadata(path).map_err(io_error)?;if !before.is_file()||before.len()==0||before.len()>maximum{return Err(error("bounded regular setup document required"));}
    let mut options=OpenOptions::new();options.read(true);#[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.custom_flags(nix::libc::O_NOFOLLOW|nix::libc::O_CLOEXEC);}
    let file=options.open(path).map_err(io_error)?;let opened=file.metadata().map_err(io_error)?;let mut data=Vec::new();file.take(maximum+1).read_to_end(&mut data).map_err(io_error)?;
    let after=std::fs::symlink_metadata(path).map_err(io_error)?;if data.len() as u64>maximum||opened.len()!=before.len()||opened.modified().ok()!=after.modified().ok()||opened.len()!=after.len(){return Err(error("setup document changed during read"));}
    #[cfg(unix)] {use std::os::unix::fs::MetadataExt;if before.dev()!=opened.dev()||before.ino()!=opened.ino()||after.dev()!=opened.dev()||after.ino()!=opened.ino()||opened.nlink()!=1||opened.ctime()!=after.ctime()||opened.ctime_nsec()!=after.ctime_nsec(){return Err(error("setup document identity changed or aliased"));}}
    Ok(data)
}
#[cfg(unix)]
fn write_new(root:&Path,name:&str,data:&[u8])->Result<(),ValidationError>{use std::os::unix::fs::OpenOptionsExt;let mut file=OpenOptions::new().write(true).create_new(true).mode(0o600).custom_flags(nix::libc::O_NOFOLLOW|nix::libc::O_CLOEXEC).open(root.join(name)).map_err(io_error)?;file.write_all(data).map_err(io_error)?;file.sync_all().map_err(io_error)}
#[cfg(not(unix))] fn write_new(_:&Path,_:&str,_:&[u8])->Result<(),ValidationError>{Err(error("private setup publication requires Unix profile"))}
/// A last marker follows all immutable files; partial publication is retained.
pub fn write(request:&Request,root:&Path,cancel:&impl CancellationSignal)->Result<BundleStatus,ValidationError>{
    let held=directory(root)?;if std::fs::read_dir(root).map_err(io_error)?.next().is_some(){return Err(error("setup output directory must be empty"));}
    let prepared=prepare(request,cancel)?;let documents=[("profile.json",bytes(&prepared.profile)?),("preparation.json",bytes(&prepared.private_preparation)?),("doctor.json",bytes(&prepared.doctor)?),("request.json",bytes(request)?)];let mut members=Vec::new();
    for (name,data) in documents {if cancel.is_cancelled(){return Err(error("setup publication cancelled; retain incomplete documents"));}write_new(root,name,&data)?;members.push(Member {name:name.into(),byte_count:data.len() as u64,sha256:sha(&data)});}
    let marker=Marker {schema_version:"flow.ai-local-setup-bundle/v1".into(),profile_sha256:prepared.profile.sha256()?,recorded_state:prepared.doctor.report.state,members};
    write_new(root,"setup.json",&bytes(&marker)?)?;held.sync_all().map_err(io_error)?;status(root,cancel)
}
pub fn status(root:&Path,cancel:&impl CancellationSignal)->Result<BundleStatus,ValidationError>{
    let _held=directory(root)?;let marker:Marker=serde_json::from_slice(&read(&root.join("setup.json"),16_384)?).map_err(|_|error("closed setup marker required"))?;
    let names=["profile.json","preparation.json","doctor.json","request.json"];
    if marker.schema_version!="flow.ai-local-setup-bundle/v1"||!pin(&marker.profile_sha256)||marker.members.len()!=names.len(){return Err(error("setup marker identity differs"));}
    for (expected,m) in names.into_iter().zip(&marker.members){if cancel.is_cancelled(){return Err(error("setup status cancelled"));}if m.name!=expected||m.byte_count==0||m.byte_count>131_072||!pin(&m.sha256){return Err(error("setup member vocabulary or bound differs"));}let actual=read(&root.join(&m.name),131_072)?;if actual.len() as u64!=m.byte_count||sha(&actual)!=m.sha256{return Err(error("saved setup member changed"));}}
    if std::fs::read_dir(root).map_err(io_error)?.take(6).collect::<std::io::Result<Vec<_>>>().map_err(io_error)?.len()!=5{return Err(error("unknown setup member requires reconciliation"));}
    let saved_doctor:Doctor=serde_json::from_slice(&read(&root.join("doctor.json"),131_072)?).map_err(|_|error("closed saved doctor required"))?;
    if saved_doctor.report.state!=marker.recorded_state||saved_doctor.report.profile_sha256!=marker.profile_sha256||saved_doctor.runtime_or_service_contacted||saved_doctor.physical_storage_verified
        ||saved_doctor.report.invocation_authority||saved_doctor.report.runtime_qualified||saved_doctor.report.offline_enforcement_verified||saved_doctor.report.memory_bound_enforced||saved_doctor.report.cost_known||saved_doctor.actions.iter().any(|a|a.automatic){return Err(error("saved doctor claims unsupported readiness/authority"));}
    let profile=Profile::from_json_slice(&read(&root.join("profile.json"),32_768)?)?;if profile.sha256()?!=marker.profile_sha256{return Err(error("setup profile differs from marker"));}
    let prep_bytes=read(&root.join("preparation.json"),65_536)?;let prep=if prep_bytes.iter().copied().filter(|b|!b.is_ascii_whitespace()).eq(b"null".iter().copied()){None}else{Some(LocalPreparation::from_json_slice(&prep_bytes)?)};
    let request=Request::from_json_slice(&read(&root.join("request.json"),131_072)?)?;if request.profile!=profile{return Err(error("setup request/profile basis differs"));}
    let deadline=Instant::now().checked_add(Duration::from_millis(request.deadline_ms.max(1).min(300_000))).ok_or_else(||error("status deadline overflow"))?;let guard=||cancel.is_cancelled()||Instant::now()>=deadline;
    let current=diagnose(&profile,prep.as_ref(),&guard)?;Ok(BundleStatus {schema_version:"flow.ai-local-setup-status/v1".into(),profile_sha256:marker.profile_sha256,recorded_state:marker.recorded_state,current,documents_match:true})
}
