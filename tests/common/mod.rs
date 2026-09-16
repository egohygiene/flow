#![allow(dead_code)]

use std::collections::BTreeMap;

use flow::{
    Authorization, Configuration, Domain, EXTENSION_INVOCATION_V1, ExecutionModeKind,
    ExtensionCatalog, ExtensionInvocation, ExtensionLock, ExtensionManifest, ExtensionObservation,
    InvocationExtension, InvocationInterface, InvocationPhase, ResolutionRequest,
    ResolvedExtension,
};

pub const CONFIG_DIGEST: &str = "3333333333333333333333333333333333333333333333333333333333333333";
pub const GRANTS_DIGEST: &str = "4444444444444444444444444444444444444444444444444444444444444444";

pub fn manifest() -> ExtensionManifest {
    serde_json::from_str(include_str!(
        "../../contracts/examples/extension-manifest.v1.example.json"
    ))
    .expect("checked-in manifest example must deserialize")
}

pub fn lock() -> ExtensionLock {
    let mut lock: ExtensionLock = serde_json::from_str(include_str!(
        "../../contracts/examples/extension-lock.v1.example.json"
    ))
    .expect("checked-in lock example must deserialize");
    lock.extensions[0].trust = flow::Trust::Trusted;
    lock
}

pub fn observation(manifest: &ExtensionManifest, available: bool) -> ExtensionObservation {
    ExtensionObservation::new(
        manifest.extension_id.clone(),
        manifest.version.clone(),
        manifest.publisher.id.clone(),
        manifest.integrity.clone(),
        available,
    )
}

pub fn request() -> ResolutionRequest {
    ResolutionRequest::new(
        "test-case",
        "optiflow/inspect-collection",
        Domain::Optiflow,
        "synthetic-library",
        ExecutionModeKind::InProcess,
    )
}

pub fn process_request() -> ResolutionRequest {
    ResolutionRequest::new(
        "test-case-process",
        "optiflow/inspect-collection",
        Domain::Optiflow,
        "synthetic-process",
        ExecutionModeKind::Process,
    )
}

pub fn resolved_fixture() -> (ExtensionCatalog, ResolutionRequest) {
    let manifest = manifest();
    let observation = observation(&manifest, true);
    let catalog = ExtensionCatalog::inspect([manifest], lock(), [observation])
        .expect("fixture catalog must inspect");
    (catalog, request())
}

pub fn resolved_process_fixture() -> (ExtensionCatalog, ResolutionRequest) {
    let manifest = manifest();
    let observation = observation(&manifest, true);
    let catalog = ExtensionCatalog::inspect([manifest], lock(), [observation])
        .expect("fixture catalog must inspect");
    (catalog, process_request())
}

pub fn invocation(resolved: &ResolvedExtension) -> ExtensionInvocation {
    ExtensionInvocation {
        schema_version: EXTENSION_INVOCATION_V1.to_owned(),
        invocation_id: "invocation:test".to_owned(),
        run_id: "run:test".to_owned(),
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
        expected_output_types: resolved.capability().produces.clone(),
        configuration: Configuration {
            schema_id: resolved.capability().configuration_schema.clone(),
            digest: CONFIG_DIGEST.to_owned(),
            values: BTreeMap::new(),
        },
        authorization: Authorization {
            authorization_id: "authorization:test".to_owned(),
            lock_id: resolved.lock_id().to_owned(),
            grants_digest: GRANTS_DIGEST.to_owned(),
        },
        limits: resolved.execution_mode().limits.clone(),
        checkpoint_refs: Vec::new(),
        secret_handles: Vec::new(),
        cancellation_id: "cancel:test".to_owned(),
    }
}

pub fn variant(
    base: &ExtensionManifest,
    extension_id: &str,
    digest_character: char,
) -> ExtensionManifest {
    let mut variant = base.clone();
    extension_id.clone_into(&mut variant.extension_id);
    variant.integrity.value = digest_character.to_string().repeat(64);
    variant
}
