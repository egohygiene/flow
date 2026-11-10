//! Explicit preparation of selected existing local files and portable doctor
//! actions. This is not an installer, runtime launcher or invocation authority.
use super::*;
use std::time::{Duration,Instant};
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {pub asset_id:String,pub kind:AssetKind,pub path:PathBuf,pub expected_sha256:Option<String>}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version:String,pub profile:Profile,pub deployment_id:String,pub endpoint:SocketAddr,
    pub service_egress_disabled_declared:bool,pub runtime_license:String,pub model_license:String,
    pub quantization:String,pub maximum_observation_bytes:u64,pub deadline_ms:u64,pub assets:Vec<Selection>,
}
impl Request {pub fn from_json_slice(bytes:&[u8])->Result<Self,ValidationError>{
    if bytes.len()>131_072{return Err(error("local setup request exceeds128KiB"));}
    let raw:serde_json::Value=serde_json::from_slice(bytes).map_err(|_|error("local setup JSON required"))?;
    if !raw.is_object()||!raw["profile"].is_object()||!raw["profile"]["budgets"].is_object()||raw["assets"].as_array().is_none_or(|v|v.iter().any(|a|!a.is_object())){return Err(error("closed object setup records required"));}
    let r:Self=serde_json::from_slice(bytes).map_err(|_|error("closed local setup request required"))?;r.profile.validate()?;Ok(r)
}}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum ActionKind {PrepareSelectedAssetSeparately,CorrectAssetPin,SelectReadableRegularAsset,ObserveNativeRuntimeIdentity,ReviewSeparateCodeModelLicenses,QualifyOfflineService,ReviewMemoryContextBudget}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {pub kind:ActionKind,pub asset_id:Option<String>,pub automatic:bool}
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Doctor {pub schema_version:String,pub report:Report,pub actions:Vec<Action>,pub selected_logical_bytes:Option<u64>,pub physical_storage_verified:bool,pub runtime_or_service_contacted:bool}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prepared {pub schema_version:String,pub profile:Profile,pub private_preparation:Option<LocalPreparation>,pub doctor:Doctor}
fn doctor(report:Report,total:Option<u64>)->Doctor {
    let mut actions=Vec::new();for asset in &report.assets {let kind=match asset.state {AssetState::Missing=>Some(ActionKind::PrepareSelectedAssetSeparately),AssetState::Changed=>Some(ActionKind::CorrectAssetPin),AssetState::Unavailable=>Some(ActionKind::SelectReadableRegularAsset),AssetState::MatchedBytes=>None};if let Some(kind)=kind{actions.push(Action {kind,asset_id:Some(asset.asset_id.clone()),automatic:false});}}
    if report.state==State::PreparedBytes {for kind in [ActionKind::ObserveNativeRuntimeIdentity,ActionKind::ReviewSeparateCodeModelLicenses,ActionKind::QualifyOfflineService,ActionKind::ReviewMemoryContextBudget]{actions.push(Action {kind,asset_id:None,automatic:false});}}
    Doctor {schema_version:"flow.ai-local-doctor/v1".into(),report,actions,selected_logical_bytes:total,physical_storage_verified:false,runtime_or_service_contacted:false}
}
/// Reuse the existing inspector; diagnostics contain logical asset IDs and
/// fixed actions, never private paths, endpoint, license text or raw errors.
pub fn diagnose(profile:&Profile,preparation:Option<&LocalPreparation>,cancel:&impl CancellationSignal)->Result<Doctor,ValidationError>{let report=inspect(profile,preparation,cancel)?;let total=preparation.filter(|_|report.state==State::PreparedBytes).and_then(|p|p.assets.iter().try_fold(0u64,|sum,a|sum.checked_add(a.byte_count)));Ok(doctor(report,total))}
/// Observe only explicitly selected existing files, derive their byte pins,
/// then require the existing two-observation inspector to confirm the set.
pub fn prepare(request:&Request,cancel:&impl CancellationSignal)->Result<Prepared,ValidationError>{
    request.profile.validate()?;if request.schema_version!="flow.ai-local-setup/v1"{return Err(error("unknown local setup request"));}
    let mut baseline=inspect(&request.profile,None,cancel)?;
    if request.profile.mode!=Mode::LocalOnly||baseline.state==State::Incompatible {return Ok(Prepared {schema_version:"flow.ai-local-setup-result/v1".into(),profile:request.profile.clone(),private_preparation:None,doctor:doctor(baseline,None)});}
    if !id(&request.deployment_id)||!request.endpoint.ip().is_loopback()||request.endpoint.port()==0||!request.service_egress_disabled_declared||![&request.runtime_license,&request.model_license,&request.quantization].into_iter().all(|s|text(s))
        ||request.maximum_observation_bytes==0||request.maximum_observation_bytes>137_438_953_472||!(1..=300_000).contains(&request.deadline_ms)||request.assets.is_empty()||request.assets.len()>32{return Err(error("explicit bounded local deployment/assets required"));}
    let deadline=Instant::now().checked_add(Duration::from_millis(request.deadline_ms)).ok_or_else(||error("setup deadline overflow"))?;
    let guard=||cancel.is_cancelled()||Instant::now()>=deadline;
    let mut ids=BTreeSet::new();let mut paths=BTreeSet::new();let mut selections=Vec::new();let mut total=0u64;
    for selection in &request.assets {
        if guard(){return Err(error("setup cancelled or deadline exhausted"));}
        if !id(&selection.asset_id)||!ids.insert(&selection.asset_id)||!paths.insert(&selection.path)||!selection.path.is_absolute()||selection.expected_sha256.as_ref().is_some_and(|p|!pin(p)){return Err(error("unique explicit asset identities/locators required"));}
        let metadata=std::fs::symlink_metadata(&selection.path);
        let (state,bytes)=match metadata {Ok(m) if m.is_file()&&m.len()>0&&m.len()<=8_589_934_592&&selection.path.canonicalize().ok().as_ref()==Some(&selection.path)=>(AssetState::MatchedBytes,m.len()),Err(e) if e.kind()==std::io::ErrorKind::NotFound=>(AssetState::Missing,0),_=>(AssetState::Unavailable,0)};
        total=total.checked_add(bytes).ok_or_else(||error("selected asset total overflow"))?;if total>request.maximum_observation_bytes{return Err(error("selected assets exceed observation byte bound"));}
        if state!=AssetState::MatchedBytes {baseline.assets.push(AssetObservation {asset_id:selection.asset_id.clone(),kind:selection.kind,state,observed_sha256:None,observed_bytes:None});continue;}
        selections.push(Asset {asset_id:selection.asset_id.clone(),kind:selection.kind,path:selection.path.clone(),sha256:selection.expected_sha256.clone().unwrap_or_else(||"0".repeat(64)),byte_count:bytes});
    }
    if request.assets.iter().filter(|a|a.kind==AssetKind::RuntimeExecutable).count()!=1||!request.assets.iter().any(|a|a.kind==AssetKind::ModelWeights)||!request.assets.iter().any(|a|a.kind==AssetKind::ModelConfiguration){return Err(error("select one runtime plus weights and model configuration"));}
    if !baseline.assets.is_empty(){baseline.state=if baseline.assets.iter().any(|a|a.state==AssetState::Missing){State::Missing}else{State::Unavailable};baseline.reasons=vec!["selected_assets_unavailable_no_install_or_fallback".into()];return Ok(Prepared {schema_version:"flow.ai-local-setup-result/v1".into(),profile:request.profile.clone(),private_preparation:None,doctor:doctor(baseline,None)});}
    let mut assets=Vec::new();for (selection,asset) in request.assets.iter().zip(selections){let mut observed=observe(&asset,&guard)?;
        if observed.state==AssetState::Missing||observed.state==AssetState::Unavailable||selection.expected_sha256.as_ref().is_some_and(|expected|observed.observed_sha256.as_ref()!=Some(expected)){if observed.state==AssetState::MatchedBytes{observed.state=AssetState::Changed;}baseline.assets.push(observed);continue;}
        let actual=observed.observed_sha256.ok_or_else(||error("selected asset did not produce a complete digest"))?;
        let bytes=observed.observed_bytes.ok_or_else(||error("selected asset byte extent unavailable"))?;
        if bytes!=asset.byte_count {baseline.assets.push(AssetObservation {asset_id:asset.asset_id,kind:asset.kind,state:AssetState::Changed,observed_sha256:Some(actual),observed_bytes:Some(bytes)});continue;}
        assets.push(Asset {sha256:actual,..asset});
    }
    if !baseline.assets.is_empty(){baseline.state=if baseline.assets.iter().any(|a|a.state==AssetState::Missing){State::Missing}else if baseline.assets.iter().any(|a|a.state==AssetState::Unavailable){State::Unavailable}else{State::Changed};baseline.reasons=vec!["selected_asset_changed_during_preparation".into()];return Ok(Prepared {schema_version:"flow.ai-local-setup-result/v1".into(),profile:request.profile.clone(),private_preparation:None,doctor:doctor(baseline,None)});}
    let prep=LocalPreparation {schema_version:"flow.ai-local-preparation/v1".into(),deployment_id:request.deployment_id.clone(),profile_sha256:request.profile.sha256()?,endpoint:request.endpoint,service_egress_disabled_declared:request.service_egress_disabled_declared,runtime_license:request.runtime_license.clone(),model_license:request.model_license.clone(),quantization:request.quantization.clone(),maximum_observation_bytes:request.maximum_observation_bytes,assets};
    let report=inspect(&request.profile,Some(&prep),&guard)?;let accepted=report.state==State::PreparedBytes;
    Ok(Prepared {schema_version:"flow.ai-local-setup-result/v1".into(),profile:request.profile.clone(),private_preparation:if accepted{Some(prep)}else{None},doctor:doctor(report,accepted.then_some(total))})
}
#[cfg(test)] mod tests;

pub mod documents;
