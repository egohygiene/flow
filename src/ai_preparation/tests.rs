//! Authored/uncompiled/unrun. No runtime/provider fixture is executed.
use super::*;
use std::sync::atomic::{AtomicU64,Ordering};
fn profile()->Profile{Profile {schema_version:"flow.ai-preparation-profile/v1".into(),profile_id:"example".into(),mode:Mode::LocalOnly,provider_id:"ollama".into(),model_id:"synthetic".into(),runtime_revision:"declared-1".into(),model_revision:"declared-2".into(),template_sha256:"a".repeat(64),output_schema_sha256:"b".repeat(64),modality:Modality::Text,structured_output:true,budgets:Budgets {input_bytes:1024,output_bytes:4096,context_tokens:4096,output_tokens:512,deadline_ms:1000,attempts:1,concurrency:1,memory_mib:8192,working_bytes:16777216,hard_cost_microunits:None}}}
struct Fixture(PathBuf);impl Drop for Fixture{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
fn fixture(p:&Profile)->(Fixture,LocalPreparation){static NEXT:AtomicU64=AtomicU64::new(0);let root=std::env::temp_dir().join(format!("flow-ai-preparation-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));std::fs::create_dir(&root).unwrap();let root=root.canonicalize().unwrap();let mut assets=Vec::new();for (id,kind) in [("runtime",AssetKind::RuntimeExecutable),("weights",AssetKind::ModelWeights),("config",AssetKind::ModelConfiguration)]{let path=root.join(id);let data=id.as_bytes();std::fs::write(&path,data).unwrap();assets.push(Asset {asset_id:id.into(),kind,path,sha256:format!("{:x}",Sha256::digest(data)),byte_count:data.len() as u64});}(Fixture(root),LocalPreparation {schema_version:"flow.ai-local-preparation/v1".into(),deployment_id:"synthetic".into(),profile_sha256:p.sha256().unwrap(),endpoint:"127.0.0.1:11434".parse().unwrap(),service_egress_disabled_declared:true,runtime_license:"declared-unreviewed".into(),model_license:"declared-unreviewed".into(),quantization:"synthetic".into(),maximum_observation_bytes:1024,assets})}
#[test] fn disabled_does_not_require_or_inspect_invalid_preparation(){let mut p=profile();let (_f,mut prep)=fixture(&p);p.mode=Mode::Disabled;prep.assets[0].path="/never/read/this".into();prep.endpoint="203.0.113.1:1".parse().unwrap();let r=inspect(&p,Some(&prep),&crate::NeverCancelled).unwrap();assert_eq!(r.state,State::Disabled);assert!(r.assets.is_empty());}
#[test] fn matched_files_prove_only_bytes_and_report_is_redacted(){let p=profile();let (_f,prep)=fixture(&p);let r=inspect(&p,Some(&prep),&crate::NeverCancelled).unwrap();assert_eq!(r.state,State::PreparedBytes);assert!(!r.runtime_qualified&&!r.offline_enforcement_verified&&!r.invocation_authority&&!r.memory_bound_enforced&&!r.cost_known);let s=serde_json::to_string(&r).unwrap();assert!(!s.contains(prep.assets[0].path.to_str().unwrap()));assert!(!s.contains("127.0.0.1"));}
#[test] fn changed_and_missing_local_files_cannot_trigger_fallback(){let p=profile();let (_f,prep)=fixture(&p);std::fs::write(&prep.assets[0].path,b"different").unwrap();assert_eq!(inspect(&p,Some(&prep),&crate::NeverCancelled).unwrap().state,State::Changed);std::fs::remove_file(&prep.assets[1].path).unwrap();assert_eq!(inspect(&p,Some(&prep),&crate::NeverCancelled).unwrap().state,State::Missing);}
#[test] fn absent_preparation_and_remote_profiles_remain_inert(){let mut p=profile();assert_eq!(inspect(&p,None,&crate::NeverCancelled).unwrap().state,State::Missing);p.mode=Mode::Remote;assert_eq!(inspect(&p,None,&crate::NeverCancelled).unwrap().state,State::Refused);}
#[test] fn incompatible_modality_hard_cost_or_egress_policy_is_explicit(){let mut p=profile();p.modality=Modality::Video;assert_eq!(inspect(&p,None,&crate::NeverCancelled).unwrap().state,State::Incompatible);p.modality=Modality::Text;p.budgets.hard_cost_microunits=Some(0);assert_eq!(inspect(&p,None,&crate::NeverCancelled).unwrap().state,State::Incompatible);p.budgets.hard_cost_microunits=None;let (_f,mut prep)=fixture(&p);prep.service_egress_disabled_declared=false;assert_eq!(inspect(&p,Some(&prep),&crate::NeverCancelled).unwrap().state,State::Refused);}
#[test] fn profile_changes_invalidate_preparation_and_unknown_fields_refuse(){let mut p=profile();let (_f,prep)=fixture(&p);p.model_revision="changed".into();assert!(inspect(&p,Some(&prep),&crate::NeverCancelled).is_err());let mut v=serde_json::to_value(&p).unwrap();v["credential"]=serde_json::json!("never accept raw keys");assert!(Profile::from_json_slice(&serde_json::to_vec(&v).unwrap()).is_err());}
#[test] fn duplicate_assets_unbounded_attempts_and_cancellation_refuse(){let mut p=profile();let (_f,mut prep)=fixture(&p);prep.assets.push(prep.assets[0].clone());assert!(prep.validate().is_err());p.budgets.attempts=2;assert!(p.validate().is_err());p.budgets.attempts=1;struct Cancelled;impl CancellationSignal for Cancelled{fn is_cancelled(&self)->bool{true}}assert!(inspect(&p,None,&Cancelled).is_err());}

#[cfg(unix)]
#[test] fn hard_linked_assets_are_not_prepared_even_when_bytes_match(){
    let p=profile();let (f,prep)=fixture(&p);
    std::fs::hard_link(&prep.assets[0].path,f.0.join("runtime-alias")).unwrap();
    let report=inspect(&p,Some(&prep),&crate::NeverCancelled).unwrap();
    assert_eq!(report.state,State::Unavailable);assert!(!report.invocation_authority);
}
#[cfg(unix)]
#[test] fn equal_bytes_and_mtime_cannot_hide_a_replaced_asset_path(){
    use std::cell::Cell;
    let p=profile();let (f,prep)=fixture(&p);let asset=&prep.assets[0];
    let before=std::fs::metadata(&asset.path).unwrap();let changed=Cell::new(false);
    let token=||{if !changed.replace(true){
        std::fs::rename(&asset.path,f.0.join("held-runtime")).unwrap();
        std::fs::write(&asset.path,b"runtime").unwrap();
        std::fs::File::open(&asset.path).unwrap().set_times(std::fs::FileTimes::new().set_modified(before.modified().unwrap())).unwrap();
    }false};
    assert_eq!(observe(asset,&token).unwrap().state,AssetState::Changed);
}
