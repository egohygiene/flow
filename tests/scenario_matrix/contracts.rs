use super::*;

pub(super) fn run(fixture: &KitFixture, recipe: &str) -> (Outcome, Value) {
    if let Some(document) = recipe.strip_prefix("schema-") {
        return schema_case(fixture, document);
    }
    let mut prepared = fixture.prepare_lifecycle(CAPABILITIES[0], "success", false);
    if recipe == "executable-mismatch" {
        fs::write(
            fixture
                .root
                .path()
                .join(PACKAGE_LOCATOR)
                .join(&fixture.executable_locator),
            b"changed executable",
        )
        .unwrap();
        let error = execute(fixture, &prepared).unwrap_err();
        assert!(matches!(
            error,
            ProcessRunnerError::SubjectObservation { .. }
        ));
        assert!(!fixture.output_path().exists());
        return (
            invalid("preflight", "subject-mismatch"),
            json!({"invoked": false}),
        );
    }
    if let Some(field) = recipe.strip_prefix("invocation-") {
        let invocation = &mut prepared.invocation;
        match field {
            "configuration-schema" => {
                "flow.other-configuration/v1".clone_into(&mut invocation.configuration.schema_id);
            }
            "publisher" => "org.example.other".clone_into(&mut invocation.extension.publisher_id),
            "capability" => "flow/transform-fixture".clone_into(&mut invocation.capability_id),
            "authorization" => {
                "authorization:other".clone_into(&mut invocation.authorization.authorization_id);
            }
            "output" => invocation.expected_output_types = vec!["text/plain".to_owned()],
            _ => panic!("unknown invocation recipe: {field}"),
        }
        let error = Orchestrator::encode_process_request(
            prepared.resolution.resolved().unwrap(),
            invocation,
            &prepared.subject_lock,
            &prepared.subjects,
            &prepared.authority,
        )
        .unwrap_err();
        assert!(!fixture.output_path().exists());
        return (execution_error(&error), json!({"invoked": false}));
    }
    let execution = execute(fixture, &prepared).unwrap();
    let mut events = execution.events().to_vec();
    let mut result = execution.result().clone();
    match recipe {
        "result-configuration" => result.configuration_digest = "a".repeat(64),
        "result-authorization" => "authorization:other".clone_into(&mut result.authorization_id),
        "result-capability" => "flow/transform-fixture".clone_into(&mut result.capability_id),
        "result-version" => "9.0.0".clone_into(&mut result.extension_version),
        "result-integrity" => result.extension_integrity = "b".repeat(64),
        "result-provenance" => result.provenance[0].value = String::new(),
        "result-duplicate-output" => result
            .produced_artifacts
            .push(result.produced_artifacts[0].clone()),
        "result-terminal-contradiction" => events.last_mut().unwrap().state = EventState::Failed,
        other => panic!("unknown contract recipe: {other}"),
    }
    let error = validate_transcript(&prepared, &transcript(&events, &result), &[]).unwrap_err();
    (
        execution_error(&error),
        json!({"provider_outcome": execution.result().outcome, "accepted_execution": false}),
    )
}

fn schema_case(fixture: &KitFixture, document: &str) -> (Outcome, Value) {
    // Baselines pass first; the only mutation is an unsupported schema version.
    macro_rules! reject_version {
        ($value:expr) => {{
            let mut value = $value;
            value.validate().unwrap();
            value.schema_version = "flow.unsupported/v99".to_owned();
            value.validate().unwrap_err();
        }};
    }
    match document {
        "manifest" => reject_version!(fixture.manifest.clone()),
        "lock" => reject_version!(fixture.lock.clone()),
        "bindings" => reject_version!(fixture.bindings.clone()),
        "scenario" => reject_version!(
            serde_json::from_str::<ScenarioManifest>(include_str!(
                "../../contracts/examples/scenario-manifest.v1.example.json"
            ))
            .unwrap()
        ),
        "invocation" | "subjects" => {
            let prepared = fixture.prepare_lifecycle(CAPABILITIES[0], "success", false);
            if document == "invocation" {
                reject_version!(prepared.invocation);
            } else {
                reject_version!(prepared.subject_lock);
            }
        }
        "result" | "event" => {
            let prepared = fixture.prepare_lifecycle(CAPABILITIES[0], "success", false);
            let execution = execute(fixture, &prepared).unwrap();
            if document == "result" {
                reject_version!(execution.result().clone());
            } else {
                reject_version!(execution.events()[0].clone());
            }
        }
        _ => panic!("unknown schema document: {document}"),
    }
    (
        invalid("contract", "unsupported-schema"),
        json!({"document": document, "baseline_valid": true}),
    )
}
