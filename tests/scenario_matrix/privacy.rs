use super::*;
use flow::ExtensionPort;

#[cfg(unix)]
pub(super) fn argv_environment() -> (Outcome, Value) {
    let mut permissions = common::manifest().requested_permissions;
    permissions.environment_read =
        vec!["FLOW_PRIVATE".to_owned(), "FLOW_PROVIDER_STDOUT".to_owned()];
    let fixture = common::process_runner_fixture(
        include_bytes!("../fixtures/privacy-provider.sh"),
        permissions.clone(),
        permissions,
    );
    let resolution = fixture.catalog.resolve(&fixture.request);
    let resolved = resolution.resolved().unwrap();
    let mut invocation = common::invocation(resolved);
    invocation.secret_handles = vec![
        "secret:env.flow_private".to_owned(),
        "secret:env.flow_provider_stdout".to_owned(),
    ];
    let subjects = observe_execution_subjects(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
    )
    .unwrap();
    let mut profile = process_authority_profile(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        ProcessIsolation::TrustedUnconfined,
    );
    profile.requested.argv = vec![CANARIES[0].to_owned()];
    profile.granted.argv = profile.requested.argv.clone();
    let enforcement = process_enforcement_evidence(&profile, &subjects);
    let authority = authorize_process(
        resolved,
        &invocation,
        &fixture.subject_lock,
        &subjects,
        &profile,
        &enforcement,
    )
    .unwrap();
    let provider = flow::HermeticExtension::new(flow::PortIdentity::from_resolved(resolved));
    let mut events = Vec::new();
    let expected = provider.invoke(&invocation, &mut events).unwrap();
    let stdout = String::from_utf8(transcript(&events, &expected)).unwrap();
    let secrets = |handle: &str| match handle {
        "secret:env.flow_provider_stdout" => Some(flow::SecretValue::new(stdout.clone())),
        "secret:env.flow_private" => Some(flow::SecretValue::new(CANARIES[1..].join(" "))),
        _ => panic!("runner requested an unauthorized secret handle"),
    };
    let execution = LocalProcessRunner::run(
        fixture.root.path(),
        resolved,
        &invocation,
        &fixture.subject_lock,
        &authority,
        &secrets,
        &mut Vec::new(),
    )
    .unwrap();
    assert_eq!(execution.events(), events);
    assert_eq!(execution.result(), &expected);
    assert_private_values_absent(&format!("{execution:?}"), fixture.root.path());
    assert!(!format!("{:?}", flow::SecretValue::new(CANARIES[2])).contains(CANARIES[2]));
    (
        outcome(
            "privacy",
            "canaries-absent",
            ScenarioTerminalState::Complete,
        ),
        json!({"canary_count": CANARIES.len(), "execution_outcome": execution.result().outcome, "probe_subjects": subjects.evidence()}),
    )
}

#[cfg(not(unix))]
pub(super) fn argv_environment() -> (Outcome, Value) {
    panic!("the privacy launch probe requires the declared Unix PR host");
}
