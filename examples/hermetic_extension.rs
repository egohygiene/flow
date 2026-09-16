use std::collections::BTreeMap;

use flow::{
    Authorization, Configuration, Domain, EXTENSION_INVOCATION_V1, ExecutionModeKind,
    ExtensionCatalog, ExtensionInvocation, ExtensionLock, ExtensionManifest, ExtensionObservation,
    HermeticExtension, InvocationExtension, InvocationInterface, InvocationPhase, Orchestrator,
    PortIdentity, ResolutionRequest,
};

const CONFIG_DIGEST: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const GRANTS_DIGEST: &str = "3333333333333333333333333333333333333333333333333333333333333333";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest: ExtensionManifest = serde_json::from_str(MANIFEST)?;
    let lock: ExtensionLock = serde_json::from_str(LOCK)?;
    let observation = ExtensionObservation::new(
        manifest.extension_id.clone(),
        manifest.version.clone(),
        manifest.publisher.id.clone(),
        manifest.integrity.clone(),
        true,
    );
    let catalog = ExtensionCatalog::inspect([manifest], lock, [observation])?;
    let resolution = catalog.resolve(&ResolutionRequest::new(
        "hermetic-example",
        "flow/hermetic-check",
        Domain::Flow,
        "hermetic-library",
        ExecutionModeKind::InProcess,
    ));
    let resolved = resolution
        .resolved()
        .ok_or("the hermetic extension did not resolve")?;
    let invocation = ExtensionInvocation {
        schema_version: EXTENSION_INVOCATION_V1.to_owned(),
        invocation_id: "invocation:hermetic-example".to_owned(),
        run_id: "run:hermetic-example".to_owned(),
        phase: InvocationPhase::Execute,
        extension: InvocationExtension {
            extension_id: resolved.extension_id().to_owned(),
            version: resolved.version().to_owned(),
            publisher_id: resolved.publisher_id().to_owned(),
            integrity: resolved.integrity().value.clone(),
        },
        capability_id: resolved.capability().capability_id.clone(),
        interface: InvocationInterface {
            kind: resolved.execution_mode().kind,
            name: resolved.execution_mode().name.clone(),
            protocol: EXTENSION_INVOCATION_V1.to_owned(),
        },
        input_artifacts: Vec::new(),
        expected_output_types: Vec::new(),
        configuration: Configuration {
            schema_id: resolved.capability().configuration_schema.clone(),
            digest: CONFIG_DIGEST.to_owned(),
            values: BTreeMap::new(),
        },
        authorization: Authorization {
            authorization_id: "authorization:hermetic-example".to_owned(),
            lock_id: resolved.lock_id().to_owned(),
            grants_digest: GRANTS_DIGEST.to_owned(),
        },
        limits: resolved.execution_mode().limits.clone(),
        checkpoint_refs: Vec::new(),
        secret_handles: Vec::new(),
        cancellation_id: "cancel:hermetic-example".to_owned(),
    };
    let port = HermeticExtension::new(PortIdentity::from_resolved(resolved));
    let mut observed = Vec::new();
    let execution = Orchestrator::execute(resolved, &invocation, &port, &mut observed)?;
    println!(
        "{} selected {}; validated {} events with outcome {:?}",
        resolution.evidence().case_id,
        resolved.extension_id(),
        execution.events().len(),
        execution.result().outcome
    );
    Ok(())
}

const MANIFEST: &str = r#"
{
  "schema_version": "flow.extension-manifest/v1",
  "extension_id": "org.egohygiene.hermetic",
  "version": "0.1.0",
  "publisher": { "id": "org.egohygiene", "display_name": "Ego Hygiene" },
  "integrity": { "algorithm": "sha256", "value": "1111111111111111111111111111111111111111111111111111111111111111" },
  "compatibility": {
    "flow_version_requirement": ">=0.1.0, <0.2.0",
    "contract_families": [
      "flow.extension-invocation/v1",
      "flow.extension-event/v1",
      "flow.extension-result/v1"
    ]
  },
  "inspection": {
    "version_source": "manifest",
    "capability_source": "manifest",
    "health_mode": "manifest-only",
    "health_capability_ids": []
  },
  "domain_ownership": ["flow"],
  "capabilities": [{
    "capability_id": "flow/hermetic-check",
    "domain": "flow",
    "configuration_schema": "flow.hermetic/v1",
    "accepts": [],
    "produces": [],
    "preconditions": [],
    "postconditions": [],
    "deterministic": true,
    "cacheable": false,
    "loss": "none",
    "content_changes": false,
    "effects": []
  }],
  "execution_modes": [{
    "name": "hermetic-library",
    "kind": "in-process",
    "protocol": "flow.extension-invocation/v1",
    "entrypoint": "flow::HermeticExtension",
    "limits": {
      "timeout_ms": 1000,
      "max_stdout_bytes": 0,
      "max_stderr_bytes": 0,
      "cancellation_grace_ms": 0
    }
  }],
  "requested_permissions": {
    "filesystem_read": [], "filesystem_write": [], "environment_read": [],
    "subprocesses": [], "network_hosts": [], "ai_providers": [],
    "gpu": false, "source_mutation": false, "destructive": false,
    "sign": false, "publish": false
  },
  "checkpoint": {
    "mode": "none",
    "contract_family": "flow.extension-result/v1",
    "compatibility_keys": []
  },
  "replacement": { "replaces": [], "fallbacks_for": [] },
  "observer_hooks": [],
  "transform_hooks": []
}
"#;

const LOCK: &str = r#"
{
  "schema_version": "flow.extension-lock/v1",
  "flow_version": "0.1.0",
  "lock_id": "lock:hermetic-example",
  "extensions": [{
    "extension_id": "org.egohygiene.hermetic",
    "version": "0.1.0",
    "publisher_id": "org.egohygiene",
    "integrity": { "algorithm": "sha256", "value": "1111111111111111111111111111111111111111111111111111111111111111" },
    "discovery": { "kind": "embedded", "location": "flow::HermeticExtension" },
    "enabled": true,
    "trust": "trusted",
    "granted_permissions": {
      "filesystem_read": [], "filesystem_write": [], "environment_read": [],
      "subprocesses": [], "network_hosts": [], "ai_providers": [],
      "gpu": false, "source_mutation": false, "destructive": false,
      "sign": false, "publish": false
    },
    "precedence": 100
  }],
  "capability_resolution": [{
    "capability_id": "flow/hermetic-check",
    "ordered_extensions": ["org.egohygiene.hermetic"],
    "fallback": "forbidden"
  }]
}
"#;
