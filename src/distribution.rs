//! Private, pinned software copies. No provider launch, downloads or qualification.
use std::{collections::BTreeSet, fs::{self, File, OpenOptions}, io::{Read, Write}, path::{Path, PathBuf}, time::Instant};
use serde::{Deserialize, Serialize};
use fs2::FileExt;
use sha2::{Digest, Sha256};
use crate::{CancellationSignal, command::{self, Code, Error}, native::{self, FilePin}, optiflow::NativePath, release_cohort::{self, Cohort, Interface, Tool}};
pub const REQUEST_SCHEMA: &str = "flow.suite-install-request/v1";
const RECEIPT: &str = "installation.json";
const MAX_FILE: u64 = 536_870_912;
const MAX_JSON: u64 = 16_777_216;
fn error(code: Code, message: &str) -> Error { Error { code, message: message.into(), recovery_directory: None } }
fn path(value: &NativePath) -> Result<PathBuf, Error> { native::path(value) }
fn hash(bytes: &[u8]) -> String { release_cohort::sha256(bytes) }
fn name(tool: Tool) -> &'static str { match tool { Tool::Flow => "flow", Tool::Aniflow => "aniflow", Tool::Optiflow => "optiflow", Tool::Renderflow => "renderflow" } }
fn directory(root: &Path) -> Result<(), Error> { if root.is_absolute() && root.canonicalize().ok().as_deref() == Some(root) && fs::metadata(root).is_ok_and(|m|m.is_dir()) { Ok(()) } else { Err(error(Code::InvalidInput,"canonical private directory required")) } }
fn create(root: &Path) -> Result<(), Error> {
    directory(root.parent().ok_or_else(||error(Code::InvalidInput,"existing parent required"))?)?;
    let mut builder=fs::DirBuilder::new(); #[cfg(unix)] { use std::os::unix::fs::DirBuilderExt; builder.mode(0o700); }
    builder.create(root).map_err(|_|error(Code::RecoveryRequired,"fresh owned directory required; preserve existing state"))?; directory(root)
}
fn write(path: &Path, bytes: &[u8], executable: bool) -> Result<(), Error> {
    directory(path.parent().ok_or_else(||error(Code::InvalidInput,"existing record parent required"))?)?;
    let mut options=OpenOptions::new(); options.write(true).create_new(true);
    #[cfg(unix)] { use std::os::unix::fs::OpenOptionsExt; options.mode(if executable {0o700}else{0o600}).custom_flags(nix::libc::O_NOFOLLOW|nix::libc::O_CLOEXEC); }
    let mut file=options.open(path).map_err(|_|error(Code::RecoveryRequired,"owned output already exists or cannot be reserved"))?;
    file.write_all(bytes).and_then(|_|file.sync_all()).map_err(|_|error(Code::Io,"owned output write incomplete; preserve it"))?;
    File::open(path.parent().ok_or_else(||error(Code::Io,"record parent"))?).and_then(|f|f.sync_all()).map_err(|_|error(Code::Io,"record directory synchronization"))
}
fn json<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> { let mut data=serde_json::to_vec_pretty(value).map_err(|_|error(Code::InvalidInput,"installation record encoding"))?; data.push(b'\n'); if data.len() as u64>MAX_JSON {return Err(error(Code::InvalidInput,"installation record bound"));} Ok(data) }
fn read<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, Error> { serde_json::from_slice(&command::read_document(path,MAX_JSON)?).map_err(|_|error(Code::RecoveryRequired,"closed installation record required")) }
fn pin(pin: &FilePin) -> Result<(), Error> { path(&pin.path)?; if !release_cohort::is_digest(&pin.sha256)||pin.byte_count==0||pin.byte_count>MAX_FILE {return Err(error(Code::InvalidInput,"bounded exact source pin required"));} Ok(()) }
fn safe(value: &str) -> bool { !value.is_empty()&&value.len()<=128&&value.bytes().all(|b|b.is_ascii_alphanumeric()||b"._-".contains(&b))&&value!="."&&value!=".." }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contract { pub identifier: String, pub file: FilePin }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Extra { pub name: String, pub file: FilePin }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedTool { pub tool: Tool, pub row_id: String, pub executable: FilePin, pub contracts: Vec<Contract>, pub extras: Vec<Extra> }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request { pub schema_version: String, pub cohort: FilePin, pub target: String, pub install_directory: NativePath, pub tools: Vec<SelectedTool>, pub timeout_ms: u64, pub maximum_total_bytes: u64 }
impl Request {
    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, Error> { if bytes.len()>1_048_576 {return Err(error(Code::InvalidInput,"suite request bound"));} let value: Self=serde_json::from_slice(bytes).map_err(|_|error(Code::InvalidInput,"closed suite install request required"))?; value.validate()?; Ok(value) }
    fn validate(&self) -> Result<(), Error> {
        pin(&self.cohort)?; let root=path(&self.install_directory)?;
        if self.schema_version!=REQUEST_SCHEMA||self.tools.is_empty()||self.tools.len()>4||self.timeout_ms==0||self.timeout_ms>3_600_000||self.maximum_total_bytes==0||self.maximum_total_bytes>8_589_934_592 {return Err(error(Code::InvalidInput,"suite request identity/limits"));}
        let mut tools=BTreeSet::new(); let mut total=0_u64;
        for tool in &self.tools {
            if !tools.insert(name(tool.tool))||tool.row_id.is_empty()||tool.row_id.len()>4096||tool.contracts.len()>64||tool.extras.len()>16 {return Err(error(Code::InvalidInput,"suite tool/contract identity or bounds"));}
            let mut identifiers=BTreeSet::new(); for contract in &tool.contracts {if contract.identifier.is_empty()||contract.identifier.len()>256||!identifiers.insert(&contract.identifier) {return Err(error(Code::InvalidInput,"suite contract identity"));}}
            let mut extras=BTreeSet::new(); for extra in &tool.extras {if !safe(&extra.name)||!extras.insert(&extra.name) {return Err(error(Code::InvalidInput,"suite support asset name"));}}
        }
        for file in self.sources() {pin(file)?;let selected=path(&file.path)?;if root.starts_with(&selected)||selected.starts_with(&root){return Err(error(Code::InvalidInput,"suite output overlaps selected original assets"));}total=total.checked_add(file.byte_count).ok_or_else(||error(Code::InvalidInput,"suite byte count overflow"))?;}
        if total>self.maximum_total_bytes {return Err(error(Code::InvalidInput,"suite total logical-byte bound"));} Ok(())
    }
    fn sources(&self) -> Vec<&FilePin> { let mut files=vec![&self.cohort]; for tool in &self.tools {files.push(&tool.executable); files.extend(tool.contracts.iter().map(|c|&c.file)); files.extend(tool.extras.iter().map(|e|&e.file));} files }
    fn entries(&self) -> Vec<(String,&FilePin)> { let mut entries=vec![("cohort.json".into(),&self.cohort)]; for tool in &self.tools {entries.push((format!("bin/{}",name(tool.tool)),&tool.executable));for (index,contract) in tool.contracts.iter().enumerate(){entries.push((format!("contracts/{}/{index:02}.json",name(tool.tool)),&contract.file));}for extra in &tool.extras {entries.push((format!("assets/{}/{}",name(tool.tool),extra.name),&extra.file));}} entries }
}
fn validate_cohort(request: &Request, token: &dyn CancellationSignal) -> Result<Cohort, Error> {
    request.cohort.current(1_048_576,token)?;
    let cohort=Cohort::from_json_slice(&command::read_document(&path(&request.cohort.path)?,1_048_576)?).map_err(|_|error(Code::InvalidInput,"suite cohort invalid"))?;
    for tool in &request.tools {
        let row=cohort.rows.iter().find(|r|r.id==tool.row_id).ok_or_else(||error(Code::InvalidInput,"selected suite row missing"))?;
        let artifact=row.artifact.as_ref().ok_or_else(||error(Code::Unavailable,"selected suite row has no immutable executable pin"))?;
        if row.tool!=tool.tool||row.target!=request.target||row.interface==Interface::RustLibrary||artifact.sha256!=tool.executable.sha256||artifact.byte_count!=tool.executable.byte_count||row.contracts.len()!=tool.contracts.len()
            ||row.contracts.iter().any(|pin|!tool.contracts.iter().any(|c|c.identifier==pin.identifier&&c.file.sha256==pin.sha256)) {return Err(error(Code::InvalidInput,"suite executable/target/contracts differ from cohort"));}
    }
    Ok(cohort)
}
fn parents(root: &Path, relative: &str) -> Result<(), Error> {
    let parent=Path::new(relative).parent().ok_or_else(||error(Code::InvalidInput,"owned relative output required"))?;
    let mut current=root.to_path_buf(); for component in parent.components() {let std::path::Component::Normal(part)=component else{return Err(error(Code::InvalidInput,"owned output traversal"));};current.push(part);match fs::symlink_metadata(&current){Ok(_)=>directory(&current)?,Err(e) if e.kind()==std::io::ErrorKind::NotFound=>create(&current)?,Err(_)=>return Err(error(Code::Io,"owned output parent unavailable"))}}
    Ok(())
}
fn copy(source: &FilePin, output: &Path, executable: bool, token: &dyn CancellationSignal) -> Result<FilePin, Error> {
    source.current(MAX_FILE,token)?;
    let input=path(&source.path)?;
    let mut options=OpenOptions::new();options.read(true);
    #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.custom_flags(nix::libc::O_NOFOLLOW|nix::libc::O_CLOEXEC|nix::libc::O_NONBLOCK);}
    let mut reader=options.open(&input).map_err(|_|error(Code::Io,"selected asset could not be opened"))?;
    if !reader.metadata().is_ok_and(|m|m.is_file()&&m.len()==source.byte_count){return Err(error(Code::StalePlan,"selected asset changed"));}
    let mut options=OpenOptions::new();options.write(true).create_new(true);
    #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.mode(if executable{0o700}else{0o600}).custom_flags(nix::libc::O_NOFOLLOW|nix::libc::O_CLOEXEC);}
    let mut writer=options.open(output).map_err(|_|error(Code::RecoveryRequired,"owned asset output already exists or cannot be created"))?;
    let mut hash=Sha256::new();let mut count=0_u64;let mut buffer=[0_u8;8192];
    loop {if token.is_cancelled(){return Err(error(Code::Cancelled,"asset copy cancelled; preserve incomplete installation"));}let size=reader.read(&mut buffer).map_err(|_|error(Code::Io,"asset copy read"))?;if size==0{break;}count=count.saturating_add(size as u64);if count>source.byte_count{return Err(error(Code::StalePlan,"asset size changed during copy"));}writer.write_all(&buffer[..size]).map_err(|_|error(Code::Io,"asset copy write"))?;hash.update(&buffer[..size]);}
    if count!=source.byte_count||format!("{:x}",hash.finalize())!=source.sha256{return Err(error(Code::StalePlan,"copied asset bytes differ from pin"));}
    writer.sync_all().map_err(|_|error(Code::Io,"installed asset synchronization"))?;source.current(MAX_FILE,token)?;
    FilePin::observe(&NativePath::from_path(output),MAX_FILE,token)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstalledFile { pub relative: String, pub sha256: String, pub byte_count: u64 }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt { pub schema_version: String, pub root: NativePath, pub target: String, pub intent_sha256: String, pub files: Vec<InstalledFile>, pub candidate_acknowledged: bool, pub provider_invoked: bool, pub publisher_authenticated: bool, pub native_qualified: bool }
#[derive(Debug, Clone, Serialize)]
pub struct Observation { pub schema_version: String, pub receipt: Receipt, pub receipt_sha256: String, pub current_copied_bytes: bool, pub required_runtime_evidence: String }
pub fn install(request: &Request, candidate_acknowledged: bool, token: &dyn CancellationSignal) -> Result<Observation, Error> {
    request.validate()?;
    if !candidate_acknowledged{return Err(error(Code::ApprovalRequired,"explicit unqualified candidate installation acknowledgement required"));}
    if Some(request.target.as_str())!=native::host_target(){return Err(error(Code::Unavailable,"native install target does not match this host"));}
    let started=Instant::now();let budget=||token.is_cancelled()||started.elapsed().as_millis()>=u128::from(request.timeout_ms);
    validate_cohort(request,&budget)?;for source in request.sources(){source.current(MAX_FILE,&budget)?;}
    let root=path(&request.install_directory)?;create(&root)?;
    let action=(|| {
        let intent=json(request)?;write(&root.join("installation-intent.json"),&intent,false)?;
        let mut files=vec![];for (relative,source) in request.entries(){parents(&root,&relative)?;let observed=copy(source,&root.join(&relative),relative.starts_with("bin/"),&budget)?;files.push(InstalledFile {relative,sha256:observed.sha256,byte_count:observed.byte_count});}
        let receipt=Receipt {schema_version:"flow.suite-installation/v1".into(),root:NativePath::from_path(&root),target:request.target.clone(),intent_sha256:hash(&intent),files,candidate_acknowledged:true,provider_invoked:false,publisher_authenticated:false,native_qualified:false};
        let data=json(&receipt)?;write(&root.join(RECEIPT),&data,false)?;
        status(&root,&hash(&data),&budget)
    })();action.map_err(|mut e: Error|{e.recovery_directory=Some(NativePath::from_path(&root));if !token.is_cancelled()&&started.elapsed().as_millis()>=u128::from(request.timeout_ms){e.code=Code::RecoveryRequired;e.message="installation observation/copy deadline exceeded; preserve partial root".into();}e})
}
fn membership(root: &Path) -> Result<BTreeSet<String>,Error> {
    fn visit(root: &Path, current: &Path, depth: usize, names: &mut BTreeSet<String>) -> Result<(),Error> {
        if depth>4||names.len()>512{return Err(error(Code::RecoveryRequired,"owned installation membership bound"));}
        for entry in fs::read_dir(current).map_err(|_|error(Code::Io,"installation membership unavailable"))? {let path=entry.map_err(|_|error(Code::Io,"installation entry unavailable"))?.path();let metadata=fs::symlink_metadata(&path).map_err(|_|error(Code::Io,"installation entry metadata"))?;if metadata.file_type().is_symlink(){return Err(error(Code::RecoveryRequired,"installation contains an unexpected link"));}let relative=path.strip_prefix(root).map_err(|_|error(Code::RecoveryRequired,"installation path escaped"))?.to_str().ok_or_else(||error(Code::RecoveryRequired,"owned path encoding changed"))?.replace(std::path::MAIN_SEPARATOR,"/");if metadata.is_dir(){directory(&path)?;names.insert(relative+"/");visit(root,&path,depth+1,names)?;}else if metadata.is_file(){names.insert(relative);}else{return Err(error(Code::RecoveryRequired,"installation contains a special node"));}}
        Ok(())
    }
    let mut names=BTreeSet::new();visit(root,root,0,&mut names)?;Ok(names)
}
pub fn status(root: &Path, expected_receipt_sha256: &str, token: &dyn CancellationSignal) -> Result<Observation,Error> {
    directory(root)?;let data=command::read_document(&root.join(RECEIPT),MAX_JSON)?;
    if hash(&data)!=expected_receipt_sha256{return Err(error(Code::StalePlan,"installation receipt pin differs"));}
    let receipt: Receipt=serde_json::from_slice(&data).map_err(|_|error(Code::RecoveryRequired,"installation receipt malformed"))?;
    let intent=command::read_document(&root.join("installation-intent.json"),1_048_576)?;let request=Request::from_json_slice(&intent)?;
    if receipt.schema_version!="flow.suite-installation/v1"||path(&receipt.root)?!=root||request.install_directory!=receipt.root||receipt.target!=request.target||hash(&intent)!=receipt.intent_sha256||!receipt.candidate_acknowledged||receipt.provider_invoked||receipt.publisher_authenticated||receipt.native_qualified{return Err(error(Code::RecoveryRequired,"installation identity or evidence scope changed"));}
    let entries=request.entries();if entries.len()!=receipt.files.len(){return Err(error(Code::RecoveryRequired,"installation copied membership differs"));}
    let mut expected=BTreeSet::from([RECEIPT.to_owned(),"installation-intent.json".into()]);
    for ((relative,pin),file) in entries.iter().zip(&receipt.files){if relative!=&file.relative||pin.sha256!=file.sha256||pin.byte_count!=file.byte_count||!expected.insert(relative.clone()){return Err(error(Code::RecoveryRequired,"installation copied identity differs from intent"));}FilePin {path:NativePath::from_path(&root.join(relative)),sha256:file.sha256.clone(),byte_count:file.byte_count}.current(MAX_FILE,token)?;}
    for (relative,_) in &entries {let mut parent=Path::new(relative).parent();while let Some(dir)=parent{if dir.as_os_str().is_empty(){break;}expected.insert(dir.to_string_lossy().replace(std::path::MAIN_SEPARATOR,"/")+"/");parent=dir.parent();}}
    if membership(root)?!=expected{return Err(error(Code::RecoveryRequired,"installation has missing or unknown files/directories; preserve them"));}
    Ok(Observation {schema_version:"flow.suite-installation-status/v1".into(),receipt,receipt_sha256:expected_receipt_sha256.into(),current_copied_bytes:true,required_runtime_evidence:"publisher/release, dependency preparation and native profile qualification remain independent; no provider was invoked".into()})
}
pub fn bundle(request: &Request, output: &Path, token: &dyn CancellationSignal) -> Result<FilePin,Error> {
    request.validate()?;validate_cohort(request,token)?;
    for source in request.sources(){let source=path(&source.path)?;if output.starts_with(&source)||source.starts_with(output){return Err(error(Code::InvalidInput,"offline bundle overlaps original assets"));}}
    let install=path(&request.install_directory)?;if output.starts_with(&install)||install.starts_with(output){return Err(error(Code::InvalidInput,"offline bundle overlaps installation destination"));}
    create(output)?;create(&output.join("files"))?;
    let mut bundled=request.clone();let mut ordinal=0_usize;
    let mut selected=|source:&FilePin|->Result<FilePin,Error>{let destination=output.join("files").join(format!("{ordinal:04}"));ordinal+=1;copy(source,&destination,false,token)};
    bundled.cohort=selected(&request.cohort)?;
    for tool in &mut bundled.tools {tool.executable=selected(&tool.executable)?;for contract in &mut tool.contracts{contract.file=selected(&contract.file)?;}for extra in &mut tool.extras{extra.file=selected(&extra.file)?;}}
    let locator=output.join("install-request.json");let data=json(&bundled)?;write(&locator,&data,false)?;
    let receipt=BundleReceipt {schema_version:"flow.suite-bundle/v1".into(),original_root:NativePath::from_path(output),request_sha256:hash(&data),file_count:ordinal};
    let locator=output.join("bundle.json");write(&locator,&json(&receipt)?,false)?;FilePin::observe(&NativePath::from_path(&locator),16_384,token)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleReceipt { schema_version: String, original_root: NativePath, request_sha256: String, file_count: usize }
/// Rebase only the exact ordinal file namespace of a pinned portable bundle.
/// The original bundle documents remain unchanged; the install intent is new.
pub fn open_bundle(root: &Path, expected_bundle_sha256: &str, install_directory: &Path, token: &dyn CancellationSignal) -> Result<Request,Error> {
    directory(root)?;let bytes=command::read_document(&root.join("bundle.json"),16_384)?;
    if hash(&bytes)!=expected_bundle_sha256{return Err(error(Code::StalePlan,"offline bundle receipt pin differs"));}
    let receipt:BundleReceipt=serde_json::from_slice(&bytes).map_err(|_|error(Code::InvalidInput,"closed offline bundle receipt required"))?;
    if receipt.schema_version!="flow.suite-bundle/v1"||receipt.file_count==0||receipt.file_count>328{return Err(error(Code::InvalidInput,"offline bundle identity/bounds"));}
    let bytes=command::read_document(&root.join("install-request.json"),1_048_576)?;
    if hash(&bytes)!=receipt.request_sha256{return Err(error(Code::StalePlan,"offline bundle request changed"));}
    let mut request=Request::from_json_slice(&bytes)?;let old=path(&receipt.original_root)?.join("files");let mut ordinal=0;
    let mut rebase=|file:&mut FilePin|->Result<(),Error>{let prior=path(&file.path)?;if prior!=old.join(format!("{ordinal:04}")){return Err(error(Code::InvalidInput,"offline bundle file namespace/order differs"));}file.path=NativePath::from_path(&root.join("files").join(format!("{ordinal:04}")));ordinal+=1;file.current(MAX_FILE,token)};
    rebase(&mut request.cohort)?;for tool in &mut request.tools{rebase(&mut tool.executable)?;for contract in &mut tool.contracts{rebase(&mut contract.file)?;}for extra in &mut tool.extras{rebase(&mut extra.file)?;}}
    if ordinal!=receipt.file_count{return Err(error(Code::InvalidInput,"offline bundle member count differs"));}
    let mut expected=BTreeSet::from(["bundle.json".into(),"install-request.json".into(),"files/".into()]);for index in 0..ordinal{expected.insert(format!("files/{index:04}"));}
    if membership(root)?!=expected{return Err(error(Code::InvalidInput,"offline bundle contains missing/unknown entries"));}
    request.install_directory=NativePath::from_path(install_directory);request.validate()?;validate_cohort(&request,token)?;Ok(request)
}
fn separate_record(root: &Path, output: &Path) -> Result<(),Error> {
    let request:Request=read(&root.join("installation-intent.json"))?;
    if output.starts_with(root)||root.starts_with(output){return Err(error(Code::InvalidInput,"management records must be outside immutable installation"));}
    for source in request.sources(){let source=path(&source.path)?;if output.starts_with(&source)||source.starts_with(output){return Err(error(Code::InvalidInput,"management record overlaps an original selected asset"));}}
    Ok(())
}
struct SelectionLock(File);
impl Drop for SelectionLock {fn drop(&mut self){let _=FileExt::unlock(&self.0);}}
fn selection_lock(root:&Path,selection:&Path)->Result<SelectionLock,Error>{
    let lock=selection.with_extension("flow-selection-lock");separate_record(root,&lock)?;
    let mut options=OpenOptions::new();options.read(true).write(true).create_new(true);
    #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.mode(0o600).custom_flags(nix::libc::O_NOFOLLOW|nix::libc::O_CLOEXEC|nix::libc::O_NONBLOCK);}
    let marker=b"flow.suite-selection-lock/v1\n";
    let file=match options.open(&lock){Ok(mut file)=>{file.write_all(marker).and_then(|_|file.sync_all()).map_err(|_|error(Code::RecoveryRequired,"selection lock initialization interrupted"))?;file},Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>{let mut options=OpenOptions::new();options.read(true).write(true);#[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.custom_flags(nix::libc::O_NOFOLLOW|nix::libc::O_CLOEXEC|nix::libc::O_NONBLOCK);}options.open(&lock).map_err(|_|error(Code::RecoveryRequired,"selection lock unavailable"))?},Err(_)=>return Err(error(Code::RecoveryRequired,"selection lock could not be reserved"))};
    file.try_lock_exclusive().map_err(|_|error(Code::RecoveryRequired,"another selection update is active"))?;
    let guard=SelectionLock(file);if command::read_document(&lock,1024)?.as_slice()!=&marker[..]{return Err(error(Code::RecoveryRequired,"selection lock is not an owned marker"));}Ok(guard)
}
/// A new selection is the activation boundary. Rollback selects an earlier exact
/// retained root. Existing pointer replacement requires its external digest.
pub fn activate(root: &Path, receipt_sha256: &str, selection: &Path, previous_sha256: Option<&str>, token: &dyn CancellationSignal) -> Result<FilePin,Error> {
    let observed=status(root,receipt_sha256,token)?;
    separate_record(root,selection)?;
    directory(selection.parent().ok_or_else(||error(Code::InvalidInput,"activation pointer parent"))?)?;
    let _lock=selection_lock(root,selection)?;
    let data=json(&serde_json::json!({"schema_version":"flow.suite-selection/v1","installation":NativePath::from_path(root),"receipt_sha256":observed.receipt_sha256,"native_qualified":false}))?;
    match previous_sha256 {
        None=>write(selection,&data,false)?,
        Some(previous)=>{let current=command::read_document(selection,16_384)?;if hash(&current)!=previous{return Err(error(Code::StalePlan,"activation changed; explicit prior pointer digest required"));}let old:serde_json::Value=serde_json::from_slice(&current).map_err(|_|error(Code::InvalidInput,"only a suite selection pointer can be replaced"))?;if old["schema_version"]!="flow.suite-selection/v1"||old["native_qualified"]!=false{return Err(error(Code::InvalidInput,"existing selection record differs"));}let pending=selection.with_extension("flow-selection-pending");separate_record(root,&pending)?;write(&pending,&data,false)?;if hash(&command::read_document(selection,16_384)?)!=previous{return Err(error(Code::StalePlan,"activation changed before replacement; preserve pending record"));}fs::rename(&pending,selection).map_err(|_|error(Code::Io,"activation replacement failed; preserve pending record"))?;File::open(selection.parent().ok_or_else(||error(Code::Io,"selection parent"))?).and_then(|f|f.sync_all()).map_err(|_|error(Code::Io,"activation directory synchronization"))?;}
    }
    FilePin::observe(&NativePath::from_path(selection),16_384,token)
}
/// Remove only exact unchanged owned copies, after a complete readback and a
/// separate create-new removal record. Unknown/changed material refuses removal.
pub fn uninstall(root: &Path, receipt_sha256: &str, removal_record: &Path, token: &dyn CancellationSignal) -> Result<(),Error> {
    let observed=status(root,receipt_sha256,token)?;
    separate_record(root,removal_record)?;
    let mut files=observed.receipt.files.iter().map(|f|root.join(&f.relative)).collect::<Vec<_>>();files.extend([root.join("installation-intent.json"),root.join(RECEIPT)]);
    let mut directories=BTreeSet::new();for file in &files{let mut parent=file.parent();while let Some(dir)=parent{if !dir.starts_with(root){break;}directories.insert(dir.to_path_buf());if dir==root{break;}parent=dir.parent();}}
    write(removal_record,&json(&serde_json::json!({"schema_version":"flow.suite-removal/v1","installation":NativePath::from_path(root),"receipt_sha256":receipt_sha256,"state":"prepared; completion requires observed root absence","original_assets_removed":false}))?,false)?;
    for file in files {if token.is_cancelled(){return Err(error(Code::Cancelled,"owned removal interrupted; inspect external record and remaining copies"));}let relative=file.strip_prefix(root).map_err(|_|error(Code::RecoveryRequired,"owned removal scope"))?.to_str().ok_or_else(||error(Code::RecoveryRequired,"owned removal path encoding"))?;let expected=if relative==RECEIPT{receipt_sha256.to_owned()}else if relative=="installation-intent.json"{observed.receipt.intent_sha256.clone()}else{observed.receipt.files.iter().find(|entry|entry.relative==relative).ok_or_else(||error(Code::RecoveryRequired,"owned removal membership"))?.sha256.clone()};let current=FilePin::observe(&NativePath::from_path(&file),MAX_FILE,token)?;if current.sha256!=expected{return Err(error(Code::RecoveryRequired,"owned copy changed immediately before removal; preserve it"));}fs::remove_file(file).map_err(|_|error(Code::RecoveryRequired,"owned removal interrupted; preserve external record and remaining copies"))?;}
    let mut directories=directories.into_iter().collect::<Vec<_>>();directories.sort_by_key(|p|std::cmp::Reverse(p.components().count()));for directory in directories{fs::remove_dir(directory).map_err(|_|error(Code::RecoveryRequired,"owned removal found remaining/unknown directory contents; preserve them"))?;}
    if root.try_exists().map_err(|_|error(Code::RecoveryRequired,"removed root state unavailable"))?{return Err(error(Code::RecoveryRequired,"new material appeared at the removed root; preserve it"));}Ok(())
}

#[cfg(test)]
mod tests;
