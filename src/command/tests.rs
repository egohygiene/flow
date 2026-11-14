//! Authored/uncompiled/unrun. Synthetic declarations are never release proof.
use super::*;
fn request() -> Request {
    Request {
        schema_version: "flow.command-request/v1".into(),
        route: Route::OptiflowReadOnly,
        provider_executable: NativePath::from_path(Path::new("/synthetic/selected/optiflow")),
        source_root: NativePath::from_path(Path::new("/synthetic/source")),
        run_directory: NativePath::from_path(Path::new("/synthetic/new-run")),
    }
}
fn fixture_plan() -> Plan {
    let request = request();
    let p = optiflow::RELEASE_PROFILES[0];
    let mut plan = Plan {
        schema_version: "flow.command-plan/v1".into(),
        plan_sha256: String::new(),
        source_scope: SourceIdentity {
            root: request.source_root.clone(),
            filesystem_id: 1,
            file_id: 2,
        },
        request,
        provider: ProviderIdentity {
            version: optiflow::PROVIDER_VERSION.into(),
            source_revision: optiflow::PROVIDER_SOURCE_REVISION.into(),
            target: p.target.into(),
            archive_sha256: p.archive_sha256.into(),
            executable_sha256: p.executable_sha256.into(),
            interface: optiflow::COMMAND_RESULT_SCHEMA.into(),
        },
        effects: effects(),
        content_binding: "source_scope_only; native scan binds actual members at execution".into(),
    };
    plan.plan_sha256 = digest(&plan).unwrap();
    plan
}
#[test]
fn approval_cannot_widen_source_effects_or_bypass_selected_plan() {
    let plan = fixture_plan();
    assert!(approve(&plan, "declared operator".into(), false).is_err());
    let approved = approve(&plan, "declared operator".into(), true).unwrap();
    let mut changed = plan.clone();
    changed.request.run_directory = NativePath::from_path(Path::new("/synthetic/different"));
    changed.plan_sha256.clear();
    changed.plan_sha256 = digest(&changed).unwrap();
    assert!(approved.validate(&changed).is_err());
    changed = plan;
    changed.effects.source_mutation_granted = true;
    changed.plan_sha256.clear();
    changed.plan_sha256 = digest(&changed).unwrap();
    assert!(changed.validate().is_err());
}
#[test]
fn unsupported_release_routes_refuse_before_any_selected_path_io() {
    let mut request = request();
    request.route = Route::AniflowTemporal;
    assert_eq!(
        plan(&request, &crate::NeverCancelled).unwrap_err().code,
        Code::Unavailable
    );
    request.route = Route::RenderflowPublication;
    assert_eq!(
        plan(&request, &crate::NeverCancelled).unwrap_err().code,
        Code::Unavailable
    );
}
#[test]
fn cancellation_precedes_source_provider_and_output_access() {
    assert_eq!(
        plan(&request(), &|| true).unwrap_err().code,
        Code::Cancelled
    );
    assert_eq!(
        run(
            &fixture_plan(),
            &approve(&fixture_plan(), "operator".into(), true).unwrap(),
            &|| true
        )
        .unwrap_err()
        .code,
        Code::Cancelled
    );
}
#[test]
fn requests_reject_unknown_fields_parent_traversal_and_implicit_defaults() {
    let mut value = serde_json::to_value(request()).unwrap();
    value["auto_install"] = serde_json::json!(true);
    assert!(Request::from_json_slice(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut request = request();
    request.source_root = NativePath::from_path(Path::new("/synthetic/../escape"));
    assert!(request.validate().is_err());
}
#[test]
fn doctor_without_selected_provider_contacts_nothing() {
    let value = doctor(None, &crate::NeverCancelled).unwrap();
    assert!(value.selected_provider.is_none());
    assert!(!value.provider_invoked && !value.installation_performed && !value.release_qualified);
    assert_eq!(value.unavailable_routes.len(), 2);
}
#[cfg(unix)]
#[test]
fn public_native_path_retains_non_utf8_and_spaces() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let path = PathBuf::from(OsString::from_vec(b"/synthetic/with space-\xff".to_vec()));
    let encoded = NativePath::from_path(&path);
    let decoded: NativePath =
        serde_json::from_slice(&serde_json::to_vec(&encoded).unwrap()).unwrap();
    assert_eq!(decoded.path().unwrap(), path);
}
