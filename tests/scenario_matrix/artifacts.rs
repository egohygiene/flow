use super::*;

// Keep the closed mutation table beside its observation and promotion assertions.
#[allow(clippy::too_many_lines)]
pub(super) fn run(fixture: &KitFixture, recipe: &str) -> (Outcome, Value) {
    let workspace = fixture.root.path().join(WORKSPACE_LOCATOR);
    let mut bindings = fixture.bindings.clone();
    if recipe == "observed-empty" {
        let path = workspace.join("outputs/empty");
        fs::create_dir(&path).unwrap();
        "outputs/empty".clone_into(&mut bindings.outputs[0].locator);
        bindings.outputs[0].kind = ArtifactKind::Directory;
        let observed = observe_artifacts(&workspace, &bindings).unwrap();
        let directory = observed
            .evidence()
            .artifacts
            .iter()
            .find(|a| a.kind == ArtifactKind::Directory)
            .unwrap();
        assert!(directory.manifest.is_empty());
        assert_eq!(directory.size_bytes, 0);
        let mut actual = outcome(
            "observation",
            "observed-empty",
            ScenarioTerminalState::Complete,
        );
        actual.evidence_state = ScenarioEvidenceState::ObservedEmpty;
        return (
            actual,
            json!({"entry_count": directory.manifest.len(), "digest": directory.digest}),
        );
    }
    let prepared = fixture.prepare_lifecycle(CAPABILITIES[0], "success", false);
    let execution = execute(fixture, &prepared).unwrap();
    match recipe {
        "absolute-path" => {
            "/private/FLOW_PRIVATE_CANARY_2604".clone_into(&mut bindings.outputs[0].locator);
        }
        "parent-traversal" => {
            "../FLOW_PRIVATE_CANARY_2604".clone_into(&mut bindings.outputs[0].locator);
        }
        "nested-escape" => {
            "outputs/../../FLOW_PRIVATE_CANARY_2604".clone_into(&mut bindings.outputs[0].locator);
        }
        "windows-path" => {
            "C:/private/FLOW_PRIVATE_CANARY_2604".clone_into(&mut bindings.outputs[0].locator);
        }
        "duplicate-identity" => bindings.outputs[0]
            .artifact_id
            .clone_from(&bindings.inputs[0].artifact_id),
        "duplicate-port" => bindings.outputs[0]
            .port
            .clone_from(&bindings.inputs[0].port),
        "duplicate-locator" => bindings.outputs[0]
            .locator
            .clone_from(&bindings.inputs[0].locator),
        "file-as-directory" => bindings.outputs[0].kind = ArtifactKind::Directory,
        "directory-as-file" => {
            fs::remove_file(fixture.output_path()).unwrap();
            fs::create_dir(fixture.output_path()).unwrap();
        }
        "symlink" => {
            let private = fixture.root.path().join(CANARIES[3]);
            fs::write(&private, CANARIES[3]).unwrap();
            fs::remove_file(fixture.output_path()).unwrap();
            symlink(&private, &fixture.output_path());
        }
        "parent-symlink" => {
            let private = fixture.root.path().join(CANARIES[3]);
            fs::create_dir(&private).unwrap();
            fs::write(private.join("inspection-report.json"), CANARIES[3]).unwrap();
            fs::remove_dir_all(workspace.join("outputs")).unwrap();
            symlink(&private, &workspace.join("outputs"));
        }
        "changed-input-before-observation" => {
            fs::write(workspace.join(INPUT_LOCATOR), b"changed input").unwrap();
        }
        "digest-conflict" => bindings.inputs[0].expected_digest = "a".repeat(64),
        "undeclared-output-type" => "text/plain".clone_into(&mut bindings.outputs[0].media_type),
        "duplicate-event"
        | "stale-invocation"
        | "stale-binding"
        | "changed-output"
        | "removed-output"
        | "changed-input-after-observation"
        | "accepted" => {}
        other => panic!("unknown artifact recipe: {other}"),
    }
    let observed = match observe_artifacts(&workspace, &bindings) {
        Ok(observed) => observed,
        Err(error) => {
            return (
                observation_error(&error),
                json!({"provider_outcome": execution.result().outcome, "provider_exit": 0}),
            );
        }
    };
    let mut invocation = prepared.invocation.clone();
    let execution = if recipe == "duplicate-event" {
        let mut events = execution.events().to_vec();
        let mut duplicate = events[1].clone();
        "event:duplicate-artifact".clone_into(&mut duplicate.event_id);
        duplicate.sequence = 2;
        events.last_mut().unwrap().sequence = 3;
        events.insert(2, duplicate);
        validate_transcript(&prepared, &transcript(&events, execution.result()), &[]).unwrap()
    } else {
        execution
    };
    match recipe {
        "stale-invocation" => "invocation:stale".clone_into(&mut invocation.invocation_id),
        "stale-binding" => "bindings:stale".clone_into(&mut bindings.binding_set_id),
        "changed-output" => fs::write(fixture.output_path(), b"changed output").unwrap(),
        "removed-output" => fs::remove_file(fixture.output_path()).unwrap(),
        "changed-input-after-observation" => {
            fs::write(workspace.join(INPUT_LOCATOR), b"changed input").unwrap();
        }
        _ => {}
    }
    match accept_artifacts(
        prepared.resolved(),
        &invocation,
        &execution,
        &bindings,
        &observed,
    ) {
        Ok(accepted) => accepted_evidence(&accepted),
        Err(error) => (
            acceptance_error(&error),
            json!({"provider_outcome": execution.result().outcome, "provider_exit": 0, "observed_artifacts": observed.evidence().artifacts.len()}),
        ),
    }
}

#[cfg(unix)]
fn symlink(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).unwrap();
}

#[cfg(not(unix))]
fn symlink(_target: &Path, _link: &Path) {
    panic!(
        "the acceptance matrix requires a Unix symlink-capable host; do not count a skipped case as passing"
    );
}
