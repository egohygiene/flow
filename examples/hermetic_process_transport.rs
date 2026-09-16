use std::collections::BTreeMap;

use flow::{
    Authorization, Configuration, Domain, EXTENSION_INVOCATION_V1, ExecutionModeKind,
    ExtensionCatalog, ExtensionInvocation, ExtensionLock, ExtensionManifest, ExtensionObservation,
    ExtensionPort, HermeticExtension, InvocationExtension, InvocationInterface, InvocationPhase,
    Orchestrator, PortIdentity, ProcessCompletion, ProcessTranscript, ResolutionRequest, Trust,
};

const CONFIG_DIGEST: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const GRANTS_DIGEST: &str = "3333333333333333333333333333333333333333333333333333333333333333";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest: ExtensionManifest = serde_json::from_str(include_str!(
        "../contracts/examples/extension-manifest.v1.example.json"
    ))?;
    let mut lock: ExtensionLock = serde_json::from_str(include_str!(
        "../contracts/examples/extension-lock.v1.example.json"
    ))?;
    lock.extensions[0].trust = Trust::Trusted;

    let observation = ExtensionObservation::new(
        manifest.extension_id.clone(),
        manifest.version.clone(),
        manifest.publisher.id.clone(),
        manifest.integrity.clone(),
        true,
    );
    let catalog = ExtensionCatalog::inspect([manifest], lock, [observation])?;
    let resolution = catalog.resolve(&ResolutionRequest::new(
        "hermetic-process-example",
        "optiflow/inspect-collection",
        Domain::Optiflow,
        "synthetic-process",
        ExecutionModeKind::Process,
    ));
    let resolved = resolution
        .resolved()
        .ok_or("the hermetic process interface did not resolve")?;
    let invocation = ExtensionInvocation {
        schema_version: EXTENSION_INVOCATION_V1.to_owned(),
        invocation_id: "invocation:hermetic-process-example".to_owned(),
        run_id: "run:hermetic-process-example".to_owned(),
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
            protocol: resolved.execution_mode().protocol.clone(),
        },
        input_artifacts: Vec::new(),
        expected_output_types: resolved.capability().produces.clone(),
        configuration: Configuration {
            schema_id: resolved.capability().configuration_schema.clone(),
            digest: CONFIG_DIGEST.to_owned(),
            values: BTreeMap::new(),
        },
        authorization: Authorization {
            authorization_id: "authorization:hermetic-process-example".to_owned(),
            lock_id: resolved.lock_id().to_owned(),
            grants_digest: GRANTS_DIGEST.to_owned(),
        },
        limits: resolved.execution_mode().limits.clone(),
        checkpoint_refs: Vec::new(),
        secret_handles: Vec::new(),
        cancellation_id: "cancel:hermetic-process-example".to_owned(),
    };

    let request = Orchestrator::encode_process_request(resolved, &invocation)?;

    // The hermetic port only manufactures deterministic fixture evidence. No
    // operating-system child is launched by this example or the Flow library.
    let fixture = HermeticExtension::new(PortIdentity::from_resolved(resolved));
    let mut provider_events = Vec::new();
    let provider_result = fixture.invoke(&invocation, &mut provider_events)?;
    let mut provider_stdout = Vec::new();
    for event in &provider_events {
        serde_json::to_writer(&mut provider_stdout, event)?;
        provider_stdout.push(b'\n');
    }
    serde_json::to_writer(&mut provider_stdout, &provider_result)?;
    provider_stdout.push(b'\n');

    let mut observed = Vec::new();
    let execution = Orchestrator::validate_process_transcript(
        resolved,
        &invocation,
        ProcessTranscript::new(
            ProcessCompletion::Exited { code: Some(0) },
            &provider_stdout,
            &[],
        ),
        &mut observed,
    )?;

    println!(
        "{} encoded {} request bytes and validated {} process events with outcome {:?}",
        resolution.evidence().case_id,
        request.len(),
        execution.events().len(),
        execution.result().outcome
    );
    Ok(())
}
