//! Thin local diagnostics delivery. No provider launch or telemetry bootstrap.
use super::*;
use crate::diagnostics;
pub(super) fn execute(args:&Arguments,token:&dyn CancellationSignal)->Result<Option<Envelope>,Error>{
    match args.command.as_str(){
        "diagnostics-preview"|"diagnostics-bundle"=>{
            let request=diagnostics::Request::from_json_slice(&read_document(&path(args,"--request")?,4096)?)?;
            if args.command=="diagnostics-preview"{success(&args.command,diagnostics::preview(&request,token)?,Outcome::Complete).map(Some)}
            else{success(&args.command,diagnostics::bundle(&request,&path(args,"--output")?,token)?,Outcome::Complete).map(Some)}
        },
        "diagnostics-inspect"=>success(&args.command,diagnostics::inspect(&path(args,"--bundle")?,token)?,Outcome::Complete).map(Some),
        "doctor-observability"=>success(&args.command,serde_json::json!({
            "schema_version":"flow.observability-health/v1","scope":"flow_owned_operational_signals",
            "instrumentation":"not_implemented","exporters":"not_implemented","collector_reachability":"not_observed",
            "queue_drops":null,"shutdown_flush":"not_observed","network_contacted":false,"domain_evidence":false,
            "next_action":"use explicit generated support metadata; telemetry adoption and native conformance remain separate"
        }),Outcome::Partial).map(Some),
        _=>Ok(None),
    }
}
#[cfg(test)]mod tests{
    use super::*;
    fn argv(values:&[&str])->Vec<OsString>{values.iter().map(|value|OsString::from(*value)).collect()}
    #[test]fn support_commands_cannot_silently_gain_write_process_or_network_authority(){
        assert!(parse(argv(&["diagnostics-preview","--request","private 作品.json","--format","json"])).is_ok());
        assert!(parse(argv(&["diagnostics-bundle","--request","r"])).is_err());
        assert!(parse(argv(&["diagnostics-preview","--request","r","--output","new"])).is_err());
        assert!(parse(argv(&["diagnostics-inspect","--bundle","b","--acknowledge-verification-network"])).is_err());
        assert!(parse(argv(&["diagnostics-bundle","--request","r","--output","b","--approved-by","someone"])).is_err());
    }
    #[test]fn unobserved_health_stays_partial_without_a_collector_probe(){
        let args=parse(argv(&["doctor-observability","--format","json"])).unwrap();
        let value=execute(&args,&crate::NeverCancelled).unwrap().unwrap();assert_eq!(value.outcome,Outcome::Partial);
        let result=value.result.unwrap();assert_eq!(result["network_contacted"],false);assert_eq!(result["queue_drops"],serde_json::Value::Null);assert_eq!(result["domain_evidence"],false);
    }
}
