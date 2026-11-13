//! Bounded explicit local input inspection and honest profile availability.
//! File hashes are observations, not media-format/domain acceptance or a source
//! freeze. The initial native route still grants whole-directory read-only scope.
use std::{path::{Path,PathBuf,Component},fs,io::Read,time::{Duration,Instant}};
use serde::{Serialize,Deserialize};
use sha2::{Sha256,Digest};
use crate::{CancellationSignal,command::{self,Error,Code},optiflow::NativePath};
fn error(code:Code,message:&str)->Error {Error {code,message:message.into(),recovery_directory:None}}
fn path(value:&NativePath)->Result<PathBuf,Error>{let p=value.path().map_err(|_|error(Code::InvalidInput,"native input path cannot be decoded"))?;if !p.is_absolute()||p.components().any(|c|matches!(c,Component::CurDir|Component::ParentDir))||p.as_os_str().len()>4096{return Err(error(Code::InvalidInput,"normalized absolute input path required"));}Ok(p)}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
pub enum Input {File {path:NativePath},Directory {path:NativePath,recursive:bool},OrderedCollection {members:Vec<NativePath>}}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Profile {InspectOnly,OptiflowReadOnly,RenderflowPublication,AniflowTemporal}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {pub maximum_entries:u64,pub maximum_files:u64,pub maximum_depth:u64,pub maximum_file_bytes:u64,pub maximum_total_bytes:u64,pub deadline_ms:u64}
impl Default for Limits {fn default()->Self {Self {maximum_entries:4096,maximum_files:256,maximum_depth:16,maximum_file_bytes:268_435_456,maximum_total_bytes:1_073_741_824,deadline_ms:30_000}}}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {pub schema_version:String,pub input:Input,pub profile:Profile,pub provider_executable:Option<NativePath>,pub run_directory:Option<NativePath>,pub limits:Limits}
impl Request {
    pub fn from_json_slice(bytes:&[u8])->Result<Self,Error>{if bytes.len()>131_072{return Err(error(Code::InvalidInput,"local input request exceeds128KiB"));}let value:Self=serde_json::from_slice(bytes).map_err(|_|error(Code::InvalidInput,"closed local input request required"))?;value.validate()?;Ok(value)}
    pub fn validate(&self)->Result<(),Error>{let l=&self.limits;if self.schema_version!="flow.local-input-request/v1"||!(1..=4096).contains(&l.maximum_entries)||!(1..=256).contains(&l.maximum_files)||l.maximum_files>l.maximum_entries||!(1..=32).contains(&l.maximum_depth)||!(1..=1_073_741_824).contains(&l.maximum_file_bytes)||l.maximum_total_bytes<l.maximum_file_bytes||l.maximum_total_bytes>4_294_967_296||!(1..=300_000).contains(&l.deadline_ms){return Err(error(Code::InvalidInput,"explicit finite local inspection limits required"));}
        match &self.input {Input::File {path:p}|Input::Directory {path:p,..}=>{path(p)?;},Input::OrderedCollection {members}=>{if members.is_empty()||members.len() as u64>l.maximum_files{return Err(error(Code::InvalidInput,"bounded nonempty ordered collection required"));}let mut seen=std::collections::BTreeSet::new();for p in members{if !seen.insert(path(p)?){return Err(error(Code::InvalidInput,"collection paths must be distinct; order is explicit"));}}}}
        for p in self.provider_executable.iter().chain(self.run_directory.iter()){path(p)?;}
        if self.profile==Profile::InspectOnly&&(self.provider_executable.is_some()||self.run_directory.is_some()){return Err(error(Code::InvalidInput,"inspection-only profile cannot hide provider/output selections"));}Ok(())
    }
}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct File {pub ordinal:u64,pub path:NativePath,pub filesystem_id:u64,pub file_id:u64,pub byte_count:u64,pub sha256:String}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Directory {pub path:NativePath,pub filesystem_id:u64,pub file_id:u64}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {pub schema_version:String,pub inventory_sha256:String,pub request_sha256:String,pub files:Vec<File>,pub directories:Vec<Directory>,pub excluded_subtrees:Vec<NativePath>,pub entries_seen:u64,pub total_bytes:u64,pub content_type:String,pub domain_acceptance:bool,pub execution_input_freeze:bool}
fn digest(value:&impl Serialize)->Result<String,Error>{let v=serde_json::to_value(value).map_err(|_|error(Code::InvalidInput,"local input identity cannot be encoded"))?;let b=serde_json::to_vec(&v).map_err(|_|error(Code::InvalidInput,"local input identity unavailable"))?;Ok(command::hash(&b))}
fn identity(metadata:&fs::Metadata)->Result<(u64,u64),Error>{#[cfg(unix)]{use std::os::unix::fs::MetadataExt;Ok((metadata.dev(),metadata.ino()))}#[cfg(not(unix))]{let _=metadata;Err(error(Code::Unavailable,"initial bounded local identity profile requires Unix"))}}
fn links(metadata:&fs::Metadata)->u64{#[cfg(unix)]{use std::os::unix::fs::MetadataExt;metadata.nlink()}#[cfg(not(unix))]{let _=metadata;0}}
struct Guard<'a>{request:&'a Request,token:&'a dyn CancellationSignal,deadline:Instant,entries:u64,bytes:u64,files:Vec<File>,directories:Vec<Directory>,excluded:Vec<NativePath>}
impl Guard<'_>{
    fn check(&self)->Result<(),Error>{if self.token.is_cancelled(){return Err(error(Code::Cancelled,"local input inspection cancelled"));}if Instant::now()>=self.deadline{return Err(error(Code::InvalidInput,"local input inspection deadline exhausted"));}Ok(())}
    fn file(&mut self,p:&Path,device:Option<u64>)->Result<(),Error>{self.check()?;let before=fs::symlink_metadata(p).map_err(|_|error(Code::InvalidInput,"selected regular input is unavailable"))?;
        if !before.is_file()||before.file_type().is_symlink()||links(&before)!=1||p.canonicalize().ok().as_deref()!=Some(p)||before.len()>self.request.limits.maximum_file_bytes||self.files.len() as u64>=self.request.limits.maximum_files{return Err(error(Code::InvalidInput,"input is redirected/aliased/unsupported or exceeds selected bounds"));}
        let id=identity(&before)?;if device.is_some_and(|d|d!=id.0){return Err(error(Code::InvalidInput,"directory input crosses a filesystem boundary"));}
        if self.bytes.checked_add(before.len()).is_none_or(|v|v>self.request.limits.maximum_total_bytes){return Err(error(Code::InvalidInput,"local input total-byte bound exceeded"));}
        let mut options=fs::OpenOptions::new();options.read(true);#[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.custom_flags(nix::libc::O_NOFOLLOW|nix::libc::O_CLOEXEC|nix::libc::O_NONBLOCK);}
        let mut file=options.open(p).map_err(|_|error(Code::InvalidInput,"selected input cannot be opened"))?;let opened=file.metadata().map_err(|_|error(Code::Io,"input identity unavailable"))?;
        if !opened.is_file()||identity(&opened)?!=id||links(&opened)!=1{return Err(error(Code::StalePlan,"input changed while opening"));}
        let mut hasher=Sha256::new();let mut count=0u64;let mut chunk=[0u8;65_536];loop {self.check()?;let n=file.read(&mut chunk).map_err(|_|error(Code::Io,"selected input cannot be read"))?;if n==0{break;}count=count.checked_add(n as u64).ok_or_else(||error(Code::InvalidInput,"input extent overflow"))?;if count>before.len()||count>self.request.limits.maximum_file_bytes{return Err(error(Code::StalePlan,"input changed or exceeded its bound during reading"));}hasher.update(&chunk[..n]);}
        let after_fd=file.metadata().map_err(|_|error(Code::Io,"input identity unavailable"))?;let after=fs::symlink_metadata(p).map_err(|_|error(Code::StalePlan,"input path changed"))?;
        for current in [&opened,&after_fd,&after]{if identity(current)?!=id||!current.is_file()||links(current)!=1||current.len()!=before.len()||current.modified().ok()!=before.modified().ok()||count!=before.len(){return Err(error(Code::StalePlan,"input changed during bounded observation"));}#[cfg(unix)] {use std::os::unix::fs::MetadataExt;if current.ctime()!=before.ctime()||current.ctime_nsec()!=before.ctime_nsec(){return Err(error(Code::StalePlan,"input metadata changed during observation"));}}}
        self.bytes+=count;self.files.push(File {ordinal:self.files.len() as u64,path:NativePath::from_path(p),filesystem_id:id.0,file_id:id.1,byte_count:count,sha256:format!("{:x}",hasher.finalize())});Ok(())
    }
    fn directory(&mut self,p:&Path,recursive:bool,depth:u64,device:u64)->Result<(),Error>{self.check()?;if depth>self.request.limits.maximum_depth{return Err(error(Code::InvalidInput,"directory depth bound exceeded"));}let metadata=fs::symlink_metadata(p).map_err(|_|error(Code::InvalidInput,"selected directory unavailable"))?;
        if !metadata.is_dir()||metadata.file_type().is_symlink()||identity(&metadata)?.0!=device||p.canonicalize().ok().as_deref()!=Some(p){return Err(error(Code::InvalidInput,"directory is redirected or crosses a filesystem boundary"));}
        let id=identity(&metadata)?;self.directories.push(Directory {path:NativePath::from_path(p),filesystem_id:id.0,file_id:id.1});
        let mut children=Vec::new();for item in fs::read_dir(p).map_err(|_|error(Code::InvalidInput,"directory cannot be enumerated"))? {self.check()?;self.entries+=1;if self.entries>self.request.limits.maximum_entries{return Err(error(Code::InvalidInput,"directory entry bound exceeded"));}children.push(item.map_err(|_|error(Code::Io,"directory member unavailable"))?.path());}children.sort();
        for child in children {self.check()?;let meta=fs::symlink_metadata(&child).map_err(|_|error(Code::StalePlan,"directory member changed"))?;if meta.file_type().is_symlink(){return Err(error(Code::InvalidInput,"directory symlink refused"));}if meta.is_dir(){if recursive{self.directory(&child,true,depth+1,device)?;}else{self.excluded.push(NativePath::from_path(&child));}}else{self.file(&child,Some(device))?;}}
        let after=fs::symlink_metadata(p).map_err(|_|error(Code::StalePlan,"directory identity changed"))?;if identity(&after)?!=identity(&metadata)?||after.modified().ok()!=metadata.modified().ok(){return Err(error(Code::StalePlan,"directory changed while enumerating"));}Ok(())
    }
}
fn snapshot(request:&Request,token:&dyn CancellationSignal,deadline:Instant)->Result<Inventory,Error>{let mut guard=Guard {request,token,deadline,entries:0,bytes:0,files:vec![],directories:vec![],excluded:vec![]};match &request.input {
    Input::File {path:p}=>{guard.entries=1;guard.file(&path(p)?,None)?;},
    Input::Directory {path:p,recursive}=>{let p=path(p)?;let metadata=fs::symlink_metadata(&p).map_err(|_|error(Code::InvalidInput,"input directory unavailable"))?;guard.directory(&p,*recursive,0,identity(&metadata)?.0)?;},
    Input::OrderedCollection {members}=>{for p in members{guard.check()?;guard.entries+=1;if guard.entries>request.limits.maximum_entries{return Err(error(Code::InvalidInput,"collection entry bound exceeded"));}guard.file(&path(p)?,None)?;}},
    }let mut result=Inventory {schema_version:"flow.local-input-inventory/v1".into(),inventory_sha256:String::new(),request_sha256:digest(request)?,files:guard.files,directories:guard.directories,excluded_subtrees:guard.excluded,entries_seen:guard.entries,total_bytes:guard.bytes,content_type:"unclassified_octets; extension_is_not_media_proof".into(),domain_acceptance:false,execution_input_freeze:false};result.inventory_sha256=digest(&result)?;Ok(result)}
/// Two bounded observations retain collection order and refuse net input drift.
/// This is not an atomic snapshot, native format validation or execution grant.
pub fn inspect(request:&Request,token:&dyn CancellationSignal)->Result<Inventory,Error>{request.validate()?;let deadline=Instant::now().checked_add(Duration::from_millis(request.limits.deadline_ms)).ok_or_else(||error(Code::InvalidInput,"inspection deadline overflow"))?;let first=snapshot(request,token,deadline)?;let second=snapshot(request,token,deadline)?;if first!=second{return Err(error(Code::StalePlan,"input membership, order or bytes changed during inspection"));}Ok(first)}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Availability {InspectionOnly,ScopePlanOnly,Unavailable}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Planning {pub schema_version:String,pub inventory:Inventory,pub availability:Availability,pub reason:String,pub command_plan:Option<command::Plan>,pub inspection_limits_apply_to_native_execution:bool}
fn planning_scope<T>(token:&dyn CancellationSignal,deadline:Instant,operation:impl FnOnce(&dyn CancellationSignal)->Result<T,Error>)->Result<T,Error>{
    let check=||{if token.is_cancelled(){Err(error(Code::Cancelled,"local input planning cancelled"))}else if Instant::now()>=deadline{Err(error(Code::InvalidInput,"local input planning deadline exhausted"))}else{Ok(())}};
    check()?;
    let guard=||token.is_cancelled()||Instant::now()>=deadline;
    let result=operation(&guard);
    check()?;
    result
}
pub fn plan(request:&Request,token:&dyn CancellationSignal)->Result<Planning,Error>{
    request.validate()?;let deadline=Instant::now().checked_add(Duration::from_millis(request.limits.deadline_ms)).ok_or_else(||error(Code::InvalidInput,"local planning deadline overflow"))?;
    planning_scope(token,deadline,|guard|plan_guarded(request,guard))
}
fn plan_guarded(request:&Request,token:&dyn CancellationSignal)->Result<Planning,Error>{let inventory=inspect(request,token)?;let mut result=Planning {schema_version:"flow.local-input-plan/v1".into(),inventory,availability:Availability::Unavailable,reason:"selected immutable provider route unavailable; no fallback".into(),command_plan:None,inspection_limits_apply_to_native_execution:false};
    if request.profile==Profile::InspectOnly{result.availability=Availability::InspectionOnly;result.reason="filesystem identities only; no media/provider acceptance".into();return Ok(result)}
    if request.profile!=Profile::OptiflowReadOnly{result.reason="native source route requires explicit cohort/artifact/descriptor/configuration selection; use plan_native or --native-selection".into();return Ok(result)}
    let Input::Directory {path:source_root,recursive:true}=&request.input else{result.reason="pinned Optiflow adapter requires whole recursive directory scope; file/ordered/nonrecursive requests cannot be broadened".into();return Ok(result)};
    let (Some(provider_executable),Some(run_directory))=(&request.provider_executable,&request.run_directory)else{result.reason="select an exact released provider executable and separate new run directory".into();return Ok(result)};
    let command=command::Request {schema_version:"flow.command-request/v1".into(),route:command::Route::OptiflowReadOnly,provider_executable:provider_executable.clone(),source_root:source_root.clone(),run_directory:run_directory.clone()};
    let planned=command::plan(&command,token)?;if inspect(request,token)?!=result.inventory{return Err(error(Code::StalePlan,"input preview changed while planning provider scope"));}
    result.availability=Availability::ScopePlanOnly;result.reason="explicit whole-directory read-only scope; preview hashes/limits are not execution input freezing or provider budget enforcement".into();result.command_plan=Some(planned);Ok(result)
}

/// Additive native selection wrapper; the old v1 scope-plan wire contract stays
/// unchanged. A directory cannot be widened into an implicit native collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRequest { pub schema_version: String, pub request: Request, pub selection: crate::native::Request }
#[derive(Debug, Clone, Serialize)]
pub struct NativePlanning { pub schema_version: String, pub inventory: Inventory, pub command_plan: crate::native::Plan, pub release_qualified: bool }
pub fn plan_native(request: &Request, selection: &crate::native::Request, token: &dyn CancellationSignal) -> Result<NativePlanning, Error> {
    request.validate()?;
    let deadline=Instant::now().checked_add(Duration::from_millis(request.limits.deadline_ms)).ok_or_else(||error(Code::InvalidInput,"local planning deadline overflow"))?;
    planning_scope(token,deadline,|guard|plan_native_guarded(request,selection,guard))
}
fn plan_native_guarded(request: &Request, selection: &crate::native::Request, token: &dyn CancellationSignal) -> Result<NativePlanning, Error> {
    if !matches!(request.profile, Profile::AniflowTemporal | Profile::RenderflowPublication) { return Err(error(Code::InvalidInput, "explicit Aniflow or Renderflow local profile required for native selection")); }
    let selected = match &request.input { Input::File { path } => vec![path.clone()], Input::OrderedCollection { members } => members.clone(), Input::Directory { .. } => return Err(error(Code::InvalidInput, "native local route requires a selected file or explicit ordered collection; no directory expansion")) };
    if selected != selection.inputs || request.provider_executable.as_ref().is_some_and(|p| p != &selection.provider_executable) || request.run_directory.as_ref().is_some_and(|p| p != &selection.run_directory) { return Err(error(Code::InvalidInput, "local input and native selection scopes/overrides differ")); }
    let inventory = inspect(request, token)?;
    let plan = crate::native::prepare(selection, token)?;
    let expected = match request.profile { Profile::AniflowTemporal => crate::release_cohort::Tool::Aniflow, _ => crate::release_cohort::Tool::Renderflow };
    if plan.row.tool != expected || inventory.files.len() != plan.sources.len() || inventory.files.iter().zip(&plan.sources).any(|(file, pin)| file.path != pin.path || file.sha256 != pin.sha256 || file.byte_count != pin.byte_count) || inspect(request, token)? != inventory { return Err(error(Code::StalePlan, "native profile or selected input bytes/order differ from local inspection")); }
    Ok(NativePlanning { schema_version: "flow.local-input-native-plan/v1".into(), inventory, command_plan: plan, release_qualified: false })
}

#[cfg(test)] mod tests;
