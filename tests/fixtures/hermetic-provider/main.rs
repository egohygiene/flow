//! Redistribution-safe synthetic process provider used by Flow conformance tests.
//!
//! This executable deliberately uses only Flow's public contracts. It reads one
//! invocation frame, verifies explicitly bound inputs, writes one deterministic
//! candidate artifact, and emits a JSON Lines event/result transcript.

use std::error::Error;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Component, Path, PathBuf};

use flow::{
    ArtifactBindingSet, ArtifactKind, EXTENSION_EVENT_V1, EXTENSION_RESULT_V1, EventKind,
    EventState, ExtensionEvent, ExtensionInvocation, ExtensionResult, Failure,
    FailureClassification, Outcome, Progress, ProvenanceEvidence, ProvenanceKind,
    ValidationEvidence, ValidationStatus,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const CONFIGURATION_SCHEMA: &str = "flow.hermetic-provider-configuration/v1";
const ARTIFACT_SCHEMA: &str = "flow.hermetic-artifact/v1";
const SUCCESS_MODE: &str = "success";

type ProviderResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
struct Arguments {
    artifact_root: PathBuf,
    artifact_bindings: PathBuf,
}

#[derive(Serialize)]
struct HermeticArtifact<'a> {
    schema_version: &'static str,
    capability_id: &'a str,
    operation: &'static str,
    configuration_digest: &'a str,
    seed: &'a str,
    inputs: Vec<InputEvidence<'a>>,
}

#[derive(Serialize)]
struct InputEvidence<'a> {
    artifact_id: &'a str,
    digest: &'a str,
    size_bytes: u64,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("flow-hermetic-provider: {error}");
        std::process::exit(2);
    }
}

#[allow(clippy::too_many_lines)]
fn run() -> ProviderResult<()> {
    let arguments = parse_arguments()?;
    let invocation = read_invocation()?;
    invocation.validate()?;

    ensure(
        invocation.configuration.schema_id == CONFIGURATION_SCHEMA,
        "unsupported hermetic provider configuration schema",
    )?;
    let actual_configuration_digest = digest_json(&invocation.configuration.values)?;
    ensure(
        actual_configuration_digest == invocation.configuration.digest,
        "configuration digest does not match its deterministic values",
    )?;
    let mode = configuration_string(&invocation, "mode")?;
    ensure(
        mode == SUCCESS_MODE,
        "checkpoint one supports only success mode",
    )?;
    let seed = configuration_string(&invocation, "seed")?;
    ensure(
        invocation.configuration.values.len() == 2,
        "configuration contains an unsupported field",
    )?;

    let (operation, expected_output_type) = capability_profile(&invocation.capability_id)?;
    let root_metadata = fs::symlink_metadata(&arguments.artifact_root)?;
    ensure(
        !root_metadata.file_type().is_symlink(),
        "artifact root must not be a symlink",
    )?;
    ensure(root_metadata.is_dir(), "artifact root must be a directory")?;
    let root = fs::canonicalize(&arguments.artifact_root)?;
    let bindings_path = canonical_input_path(&root, &arguments.artifact_bindings)?;
    let bindings: ArtifactBindingSet = serde_json::from_slice(&fs::read(bindings_path)?)?;
    bindings.validate()?;
    ensure(
        bindings.inputs.len() == 1 && bindings.outputs.len() == 1,
        "the hermetic success profile requires exactly one input and one output",
    )?;
    ensure(
        invocation.input_artifacts.len() == 1,
        "the hermetic success profile requires exactly one invocation input",
    )?;
    ensure(
        invocation.expected_output_types == [expected_output_type],
        "invocation output type does not match the selected capability",
    )?;

    let input_binding = &bindings.inputs[0];
    let invocation_input = &invocation.input_artifacts[0];
    ensure(
        input_binding.kind == ArtifactKind::File,
        "the hermetic success input binding must name a file",
    )?;
    ensure(
        input_binding.media_type == "text/plain",
        "the hermetic success input must use text/plain",
    )?;
    ensure(
        invocation_input.artifact_id == input_binding.artifact_id
            && invocation_input.digest == input_binding.expected_digest,
        "invocation input does not match the artifact binding",
    )?;
    let input_path = canonical_input_path(&root, Path::new(&input_binding.locator))?;
    let input_bytes = fs::read(input_path)?;
    let input_digest = digest_bytes(&input_bytes);
    ensure(
        input_digest == input_binding.expected_digest,
        "observed input bytes do not match the immutable binding",
    )?;

    let output_binding = &bindings.outputs[0];
    ensure(
        output_binding.kind == ArtifactKind::File,
        "the hermetic success output binding must name a file",
    )?;
    ensure(
        output_binding.media_type == expected_output_type,
        "output binding type does not match the selected capability",
    )?;
    let output_path = confined_new_output_path(&root, Path::new(&output_binding.locator))?;
    let artifact = HermeticArtifact {
        schema_version: ARTIFACT_SCHEMA,
        capability_id: &invocation.capability_id,
        operation,
        configuration_digest: &invocation.configuration.digest,
        seed,
        inputs: vec![InputEvidence {
            artifact_id: &input_binding.artifact_id,
            digest: &input_digest,
            size_bytes: u64::try_from(input_bytes.len())?,
        }],
    };
    let mut artifact_bytes = serde_json::to_vec(&artifact)?;
    artifact_bytes.push(b'\n');
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output_path)?;
    output.write_all(&artifact_bytes)?;
    output.sync_all()?;
    let output_digest = digest_bytes(&artifact_bytes);

    let consumed_artifacts = vec![input_binding.artifact_id.clone()];
    let produced_artifacts = vec![output_binding.artifact_id.clone()];
    let events = [
        event(
            &invocation,
            "started",
            0,
            EventKind::PhaseStarted,
            EventState::Running,
            Vec::new(),
        ),
        event(
            &invocation,
            "artifact",
            1,
            EventKind::ArtifactProduced,
            EventState::Produced,
            produced_artifacts.clone(),
        ),
        event(
            &invocation,
            "completed",
            2,
            EventKind::PhaseCompleted,
            EventState::Produced,
            Vec::new(),
        ),
    ];
    let result = ExtensionResult {
        schema_version: EXTENSION_RESULT_V1.to_owned(),
        run_id: invocation.run_id.clone(),
        invocation_id: invocation.invocation_id.clone(),
        extension_id: invocation.extension.extension_id.clone(),
        extension_version: invocation.extension.version.clone(),
        extension_integrity: invocation.extension.integrity.clone(),
        capability_id: invocation.capability_id.clone(),
        configuration_digest: invocation.configuration.digest.clone(),
        authorization_id: invocation.authorization.authorization_id.clone(),
        outcome: Outcome::Produced,
        partial_result: false,
        consumed_artifacts,
        produced_artifacts,
        validations: vec![ValidationEvidence {
            validator: "flow/hermetic-provider-success".to_owned(),
            status: ValidationStatus::Passed,
            evidence: "binding-and-input-digest-match".to_owned(),
        }],
        provenance: vec![
            ProvenanceEvidence {
                kind: ProvenanceKind::Extension,
                value: format!(
                    "{}@{}",
                    invocation.extension.extension_id, invocation.extension.version
                ),
            },
            ProvenanceEvidence {
                kind: ProvenanceKind::Configuration,
                value: invocation.configuration.digest.clone(),
            },
            ProvenanceEvidence {
                kind: ProvenanceKind::Artifact,
                value: format!("sha256:{output_digest}"),
            },
        ],
        diagnostics: Vec::new(),
        failure: Failure {
            classification: FailureClassification::None,
            code: String::new(),
            message: String::new(),
            retryable: false,
        },
        checkpoint_refs: Vec::new(),
        explanation: "The hermetic provider produced one deterministic synthetic artifact."
            .to_owned(),
    };

    let stdout = io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());
    for event in &events {
        write_record(&mut stdout, event)?;
    }
    write_record(&mut stdout, &result)?;
    stdout.flush()?;
    Ok(())
}

fn parse_arguments() -> ProviderResult<Arguments> {
    let mut artifact_root = None;
    let mut artifact_bindings = None;
    let mut arguments = std::env::args_os().skip(1);
    while let Some(flag) = arguments.next() {
        match flag.to_str() {
            Some("--artifact-root") => set_once(
                &mut artifact_root,
                arguments
                    .next()
                    .ok_or_else(|| invalid_input("--artifact-root requires a value"))?,
                "--artifact-root",
            )?,
            Some("--artifact-bindings") => set_once(
                &mut artifact_bindings,
                arguments
                    .next()
                    .ok_or_else(|| invalid_input("--artifact-bindings requires a value"))?,
                "--artifact-bindings",
            )?,
            Some(other) => {
                return Err(invalid_input(format!("unsupported argument {other}")).into());
            }
            None => return Err(invalid_input("arguments must be valid UTF-8").into()),
        }
    }
    let artifact_root = artifact_root
        .map(PathBuf::from)
        .ok_or_else(|| invalid_input("missing --artifact-root"))?;
    let artifact_bindings = artifact_bindings
        .map(PathBuf::from)
        .ok_or_else(|| invalid_input("missing --artifact-bindings"))?;
    ensure_portable_locator(&artifact_bindings)?;
    Ok(Arguments {
        artifact_root,
        artifact_bindings,
    })
}

fn set_once(target: &mut Option<OsString>, value: OsString, flag: &str) -> ProviderResult<()> {
    ensure(target.replace(value).is_none(), format!("duplicate {flag}"))
}

fn read_invocation() -> ProviderResult<ExtensionInvocation> {
    let mut stdin = BufReader::new(io::stdin().lock());
    let mut frame = Vec::new();
    stdin.read_until(b'\n', &mut frame)?;
    ensure(
        frame.last() == Some(&b'\n'),
        "invocation frame must end with one line feed",
    )?;
    frame.pop();
    ensure(!frame.is_empty(), "invocation frame must not be empty")?;
    let mut trailing = Vec::new();
    stdin.read_to_end(&mut trailing)?;
    ensure(
        trailing.is_empty(),
        "invocation stream contains trailing bytes",
    )?;
    Ok(serde_json::from_slice(&frame)?)
}

fn capability_profile(capability_id: &str) -> ProviderResult<(&'static str, &'static str)> {
    match capability_id {
        "flow/inspect-fixture" => {
            Ok(("inspection", "application/vnd.flow.fixture-inspection+json"))
        }
        "flow/transform-fixture" => Ok((
            "transformation",
            "application/vnd.flow.fixture-transformation+json",
        )),
        "flow/validate-fixture" => {
            Ok(("validation", "application/vnd.flow.fixture-validation+json"))
        }
        "flow/observe-fixture" => Ok((
            "read-only-observation",
            "application/vnd.flow.fixture-observation+json",
        )),
        _ => Err(invalid_input("unsupported hermetic capability").into()),
    }
}

fn configuration_string<'a>(
    invocation: &'a ExtensionInvocation,
    key: &str,
) -> ProviderResult<&'a str> {
    invocation
        .configuration
        .values
        .get(key)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid_input(format!("configuration.{key} must be a nonempty string")))
        .map_err(Into::into)
}

fn canonical_input_path(root: &Path, locator: &Path) -> ProviderResult<PathBuf> {
    ensure_portable_locator(locator)?;
    let mut path = root.to_path_buf();
    let mut components = locator.components().peekable();
    while let Some(Component::Normal(component)) = components.next() {
        path.push(component);
        let metadata = fs::symlink_metadata(&path)?;
        ensure(
            !metadata.file_type().is_symlink(),
            "input path contains a symlink",
        )?;
        if components.peek().is_some() {
            ensure(metadata.is_dir(), "input path parent must be a directory")?;
        } else {
            ensure(metadata.is_file(), "input path must be a regular file")?;
        }
    }
    let path = fs::canonicalize(path)?;
    ensure(
        path.starts_with(root),
        "input path escaped the artifact root",
    )?;
    Ok(path)
}

fn confined_new_output_path(root: &Path, locator: &Path) -> ProviderResult<PathBuf> {
    ensure_portable_locator(locator)?;
    let mut parent = root.to_path_buf();
    let mut components = locator.components().peekable();
    let mut file_name = None;
    while let Some(Component::Normal(component)) = components.next() {
        if components.peek().is_none() {
            file_name = Some(component.to_owned());
            break;
        }
        parent.push(component);
        let metadata = fs::symlink_metadata(&parent)?;
        ensure(
            !metadata.file_type().is_symlink(),
            "output path contains a symlink",
        )?;
        ensure(metadata.is_dir(), "output path parent must be a directory")?;
    }
    let parent = fs::canonicalize(parent)?;
    ensure(
        parent.starts_with(root),
        "output parent escaped the artifact root",
    )?;
    let file_name =
        file_name.ok_or_else(|| invalid_input("output locator requires a file name"))?;
    let output = parent.join(file_name);
    match fs::symlink_metadata(&output) {
        Ok(_) => return Err(invalid_input("output path already exists").into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    Ok(output)
}

fn ensure_portable_locator(locator: &Path) -> ProviderResult<()> {
    ensure(!locator.as_os_str().is_empty(), "locator must not be empty")?;
    ensure(!locator.is_absolute(), "locator must be relative")?;
    ensure(
        locator
            .components()
            .all(|component| matches!(component, Component::Normal(_))),
        "locator must contain only normal path components",
    )?;
    let locator = locator
        .to_str()
        .ok_or_else(|| invalid_input("locator must be valid UTF-8"))?;
    ensure(
        !locator.contains(['\\', ':']),
        "locator must use portable separators",
    )
}

fn event(
    invocation: &ExtensionInvocation,
    suffix: &str,
    sequence: u64,
    kind: EventKind,
    state: EventState,
    artifact_refs: Vec<String>,
) -> ExtensionEvent {
    ExtensionEvent {
        schema_version: EXTENSION_EVENT_V1.to_owned(),
        event_id: format!("event:{}-{suffix}", invocation.invocation_id),
        run_id: invocation.run_id.clone(),
        invocation_id: invocation.invocation_id.clone(),
        sequence,
        phase: invocation.phase.lifecycle_point(),
        kind,
        state,
        progress: Progress {
            completed: u64::from(kind != EventKind::PhaseStarted),
            total: 1,
            unit: "artifact".to_owned(),
        },
        diagnostics: Vec::new(),
        artifact_refs,
        checkpoint_refs: Vec::new(),
    }
}

fn write_record(writer: &mut impl Write, value: &impl Serialize) -> ProviderResult<()> {
    serde_json::to_writer(&mut *writer, value)?;
    writer.write_all(b"\n")?;
    Ok(())
}

fn digest_json(value: &impl Serialize) -> ProviderResult<String> {
    Ok(digest_bytes(&serde_json::to_vec(value)?))
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn ensure(condition: bool, message: impl Into<String>) -> ProviderResult<()> {
    if condition {
        Ok(())
    } else {
        Err(invalid_input(message).into())
    }
}

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::symlink;

    use super::*;

    #[test]
    fn output_path_rejects_an_intermediate_symlink_before_writing() {
        let root = std::env::temp_dir().join(format!(
            "flow-hermetic-provider-path-test-{}",
            std::process::id()
        ));
        let _stale_cleanup = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("inputs")).unwrap();
        fs::create_dir_all(root.join("outputs")).unwrap();
        symlink("../inputs", root.join("outputs/link")).unwrap();
        let canonical_root = fs::canonicalize(&root).unwrap();

        let error = confined_new_output_path(
            &canonical_root,
            Path::new("outputs/link/undeclared-source.json"),
        )
        .unwrap_err();

        assert!(error.to_string().contains("symlink"));
        assert!(!root.join("inputs/undeclared-source.json").exists());
        fs::remove_dir_all(root).unwrap();
    }
}
