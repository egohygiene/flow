//! Root-confined artifact bindings, deterministic host observations, and acceptance.
//!
//! Provider-authored artifact identifiers remain untrusted until they correlate
//! with Flow-owned bindings and Flow-observed filesystem evidence. This module
//! reads local artifacts but does not launch providers, grant filesystem
//! authority, enforce a sandbox, or perform domain-specific validation.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::ResolvedExtension;
use crate::contracts::{EventKind, ExtensionInvocation, Outcome, ValidationError};
use crate::execution::ValidatedExecution;

pub const ARTIFACT_BINDINGS_V1: &str = "flow.artifact-bindings/v1";
pub const ARTIFACT_OBSERVATIONS_V1: &str = "flow.artifact-observations/v1";
pub const DIRECTORY_MANIFEST_V1: &str = "flow.directory-manifest/v1";
pub const SHA256: &str = "sha256";

/// Flow-owned declarations for immutable inputs and candidate outputs.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactBindingSet {
    pub schema_version: String,
    pub binding_set_id: String,
    pub digest_algorithm: String,
    pub inputs: Vec<InputArtifactBinding>,
    pub outputs: Vec<OutputArtifactBinding>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InputArtifactBinding {
    pub artifact_id: String,
    pub port: String,
    pub media_type: String,
    pub kind: ArtifactKind,
    pub locator: String,
    pub expected_digest: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OutputArtifactBinding {
    pub artifact_id: String,
    pub port: String,
    pub media_type: String,
    pub kind: ArtifactKind,
    pub locator: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactKind {
    File,
    Directory,
}

/// Portable filesystem-observation evidence for one complete binding set.
///
/// This serializable document is portable evidence, not proof that observation
/// occurred. Only [`observe_artifacts`] can wrap it as [`ObservedArtifactSet`]
/// for acceptance.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostArtifactObservationSet {
    pub schema_version: String,
    pub binding_set_id: String,
    pub digest_algorithm: String,
    pub directory_manifest_profile: String,
    pub artifacts: Vec<HostArtifactObservation>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostArtifactObservation {
    pub artifact_id: String,
    pub port: String,
    pub media_type: String,
    pub kind: ArtifactKind,
    pub locator: String,
    pub digest: String,
    pub size_bytes: u64,
    pub manifest: Vec<DirectoryManifestEntry>,
}

/// One descendant in a deterministic, flattened directory manifest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DirectoryManifestEntry {
    pub locator: String,
    pub kind: ArtifactKind,
    pub digest: String,
    pub size_bytes: u64,
}

/// Filesystem evidence produced by Flow's root-confined observer.
///
/// The field is private so a deserialized or provider-authored observation
/// document cannot be passed to [`accept_artifacts`] as if Flow had read it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservedArtifactSet {
    evidence: HostArtifactObservationSet,
}

impl ObservedArtifactSet {
    #[must_use]
    pub const fn evidence(&self) -> &HostArtifactObservationSet {
        &self.evidence
    }

    #[must_use]
    pub fn into_evidence(self) -> HostArtifactObservationSet {
        self.evidence
    }
}

/// Artifact evidence that passed binding, host, and provider correlation.
///
/// Fields are private so a provider observation cannot be promoted with a
/// struct literal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedArtifactSet {
    binding_set_id: String,
    inputs: Vec<HostArtifactObservation>,
    outputs: Vec<HostArtifactObservation>,
}

impl AcceptedArtifactSet {
    #[must_use]
    pub fn binding_set_id(&self) -> &str {
        &self.binding_set_id
    }

    #[must_use]
    pub fn inputs(&self) -> &[HostArtifactObservation] {
        &self.inputs
    }

    #[must_use]
    pub fn outputs(&self) -> &[HostArtifactObservation] {
        &self.outputs
    }

    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        String,
        Vec<HostArtifactObservation>,
        Vec<HostArtifactObservation>,
    ) {
        (self.binding_set_id, self.inputs, self.outputs)
    }
}

impl ArtifactBindingSet {
    /// Validate the closed binding contract and its cross-field invariants.
    ///
    /// # Errors
    ///
    /// Returns the first deterministic contract violation.
    pub fn validate(&self) -> Result<(), ValidationError> {
        expect(
            self.schema_version == ARTIFACT_BINDINGS_V1,
            "bindings.schema_version",
            format!("must equal {ARTIFACT_BINDINGS_V1}"),
        )?;
        expect(
            has_prefixed_id(&self.binding_set_id, "bindings:"),
            "bindings.binding_set_id",
            "must be a lowercase bindings identifier",
        )?;
        expect(
            self.digest_algorithm == SHA256,
            "bindings.digest_algorithm",
            "must equal sha256",
        )?;
        expect(
            !self.inputs.is_empty() || !self.outputs.is_empty(),
            "bindings",
            "must declare at least one input or output",
        )?;

        for (index, binding) in self.inputs.iter().enumerate() {
            validate_binding(
                &format!("bindings.inputs[{index}]"),
                &binding.artifact_id,
                &binding.port,
                &binding.media_type,
                &binding.locator,
            )?;
            validate_digest(
                &format!("bindings.inputs[{index}].expected_digest"),
                &binding.expected_digest,
            )?;
        }
        for (index, binding) in self.outputs.iter().enumerate() {
            validate_binding(
                &format!("bindings.outputs[{index}]"),
                &binding.artifact_id,
                &binding.port,
                &binding.media_type,
                &binding.locator,
            )?;
        }

        let all = self
            .inputs
            .iter()
            .map(BindingView::from)
            .chain(self.outputs.iter().map(BindingView::from))
            .collect::<Vec<_>>();
        unique_field(
            &all,
            |binding| binding.artifact_id,
            "bindings",
            "artifact identities must be unique",
        )?;
        unique_field(
            &all,
            |binding| binding.port,
            "bindings",
            "ports must be unique",
        )?;
        unique_field(
            &all,
            |binding| binding.locator,
            "bindings",
            "locators must be unique",
        )
    }
}

impl HostArtifactObservationSet {
    /// Validate observation shape and deterministic directory-manifest identity.
    ///
    /// # Errors
    ///
    /// Returns the first deterministic contract or manifest violation.
    pub fn validate(&self) -> Result<(), ValidationError> {
        expect(
            self.schema_version == ARTIFACT_OBSERVATIONS_V1,
            "observations.schema_version",
            format!("must equal {ARTIFACT_OBSERVATIONS_V1}"),
        )?;
        expect(
            has_prefixed_id(&self.binding_set_id, "bindings:"),
            "observations.binding_set_id",
            "must be a lowercase bindings identifier",
        )?;
        expect(
            self.digest_algorithm == SHA256,
            "observations.digest_algorithm",
            "must equal sha256",
        )?;
        expect(
            self.directory_manifest_profile == DIRECTORY_MANIFEST_V1,
            "observations.directory_manifest_profile",
            format!("must equal {DIRECTORY_MANIFEST_V1}"),
        )?;
        expect(
            !self.artifacts.is_empty(),
            "observations.artifacts",
            "must contain at least one artifact",
        )?;

        for (index, observation) in self.artifacts.iter().enumerate() {
            let path = format!("observations.artifacts[{index}]");
            validate_binding(
                &path,
                &observation.artifact_id,
                &observation.port,
                &observation.media_type,
                &observation.locator,
            )?;
            validate_digest(&format!("{path}.digest"), &observation.digest)?;
            match observation.kind {
                ArtifactKind::File => expect(
                    observation.manifest.is_empty(),
                    format!("{path}.manifest"),
                    "file observations cannot contain directory entries",
                )?,
                ArtifactKind::Directory => validate_directory_manifest(observation, &path)?,
            }
        }

        unique_field(
            &self.artifacts,
            |observation| observation.artifact_id.as_str(),
            "observations.artifacts",
            "artifact identities must be unique",
        )?;
        unique_field(
            &self.artifacts,
            |observation| observation.port.as_str(),
            "observations.artifacts",
            "ports must be unique",
        )?;
        unique_field(
            &self.artifacts,
            |observation| observation.locator.as_str(),
            "observations.artifacts",
            "locators must be unique",
        )
    }
}

/// Observe every declared artifact beneath one caller-selected root.
///
/// The root and all bound paths are inspected with link-aware metadata. This
/// function rejects links and special nodes; it never follows a symlink. The
/// returned observations remain unaccepted until [`accept_artifacts`] also
/// correlates them with provider evidence.
///
/// # Errors
///
/// Returns a typed error for invalid bindings, unsafe roots/paths, filesystem
/// failures, unsupported nodes, or deterministic-manifest failures.
pub fn observe_artifacts(
    root: &Path,
    bindings: &ArtifactBindingSet,
) -> Result<ObservedArtifactSet, ArtifactObservationError> {
    bindings
        .validate()
        .map_err(|source| ArtifactObservationError::InvalidBindings { source })?;
    let root_metadata =
        fs::symlink_metadata(root).map_err(|source| ArtifactObservationError::Io {
            operation: "inspect artifact root",
            path: display_path(root),
            source,
        })?;
    if root_metadata.file_type().is_symlink() {
        return Err(ArtifactObservationError::RootSymlink {
            path: display_path(root),
        });
    }
    if !root_metadata.is_dir() {
        return Err(ArtifactObservationError::RootNotDirectory {
            path: display_path(root),
        });
    }
    let canonical_root = fs::canonicalize(root).map_err(|source| ArtifactObservationError::Io {
        operation: "canonicalize artifact root",
        path: display_path(root),
        source,
    })?;

    let mut artifacts = Vec::with_capacity(bindings.inputs.len() + bindings.outputs.len());
    for binding in bindings
        .inputs
        .iter()
        .map(BindingView::from)
        .chain(bindings.outputs.iter().map(BindingView::from))
    {
        artifacts.push(observe_binding(root, &canonical_root, binding)?);
    }

    let observations = HostArtifactObservationSet {
        schema_version: ARTIFACT_OBSERVATIONS_V1.to_owned(),
        binding_set_id: bindings.binding_set_id.clone(),
        digest_algorithm: SHA256.to_owned(),
        directory_manifest_profile: DIRECTORY_MANIFEST_V1.to_owned(),
        artifacts,
    };
    observations
        .validate()
        .map_err(|source| ArtifactObservationError::InvalidObservations { source })?;
    Ok(ObservedArtifactSet {
        evidence: observations,
    })
}

/// Correlate declared bindings, host observations, and validated provider evidence.
///
/// # Errors
///
/// Returns an error unless every declared input and candidate output matches
/// the resolved capability, invocation, host observation, provider result, and
/// artifact-produced events exactly.
#[allow(clippy::too_many_lines)]
pub fn accept_artifacts(
    resolved: &ResolvedExtension,
    invocation: &ExtensionInvocation,
    execution: &ValidatedExecution,
    bindings: &ArtifactBindingSet,
    observed: &ObservedArtifactSet,
) -> Result<AcceptedArtifactSet, ArtifactAcceptanceError> {
    let observations = observed.evidence();
    invocation
        .validate()
        .map_err(|source| ArtifactAcceptanceError::InvalidInvocation { source })?;
    bindings
        .validate()
        .map_err(|source| ArtifactAcceptanceError::InvalidBindings { source })?;
    observations
        .validate()
        .map_err(|source| ArtifactAcceptanceError::InvalidObservations { source })?;

    mismatch(
        bindings.binding_set_id == observations.binding_set_id,
        "host observations reference a different binding set",
    )?;
    correlate_execution_context(resolved, invocation, execution)?;
    mismatch(
        matches!(
            execution.result().outcome,
            Outcome::Produced | Outcome::Reused
        ) && !execution.result().partial_result,
        "artifact acceptance requires a complete produced or reused result",
    )?;

    let input_bindings = bindings
        .inputs
        .iter()
        .map(|binding| (binding.artifact_id.as_str(), binding))
        .collect::<HashMap<_, _>>();
    let output_bindings = bindings
        .outputs
        .iter()
        .map(|binding| (binding.artifact_id.as_str(), binding))
        .collect::<HashMap<_, _>>();
    let observed = observations
        .artifacts
        .iter()
        .map(|observation| (observation.artifact_id.as_str(), observation))
        .collect::<HashMap<_, _>>();

    mismatch(
        invocation.input_artifacts.len() == input_bindings.len(),
        "invocation inputs do not match the declared input bindings",
    )?;
    for input in &invocation.input_artifacts {
        let Some(binding) = input_bindings.get(input.artifact_id.as_str()) else {
            return mismatch_error("invocation contains an undeclared input artifact");
        };
        mismatch(
            input.digest == binding.expected_digest,
            "invocation input digest conflicts with its binding",
        )?;
    }

    for binding in &bindings.inputs {
        mismatch(
            media_type_allowed(&resolved.capability().accepts, &binding.media_type),
            "input binding media type is not accepted by the resolved capability",
        )?;
    }
    for binding in &bindings.outputs {
        mismatch(
            media_type_allowed(&resolved.capability().produces, &binding.media_type)
                && media_type_allowed(&invocation.expected_output_types, &binding.media_type),
            "output binding media type is not declared by the invocation and capability",
        )?;
    }
    for expected_type in &invocation.expected_output_types {
        mismatch(
            bindings
                .outputs
                .iter()
                .any(|binding| media_type_matches(expected_type, &binding.media_type)),
            "an expected output media type has no candidate output binding",
        )?;
    }

    exact_identifiers(
        &execution.result().consumed_artifacts,
        input_bindings.keys().copied(),
        "provider-consumed artifacts do not exactly match declared inputs",
    )?;
    exact_identifiers(
        &execution.result().produced_artifacts,
        output_bindings.keys().copied(),
        "provider-produced artifacts do not exactly match declared outputs",
    )?;

    let event_artifacts = execution
        .events()
        .iter()
        .filter(|event| event.kind == EventKind::ArtifactProduced)
        .flat_map(|event| event.artifact_refs.iter().map(String::as_str))
        .collect::<Vec<_>>();
    exact_identifiers(
        &event_artifacts,
        output_bindings.keys().copied(),
        "artifact-produced events do not exactly match declared outputs",
    )?;

    mismatch(
        observed.len() == input_bindings.len() + output_bindings.len(),
        "host observations do not cover every binding exactly",
    )?;

    let mut accepted_inputs = Vec::with_capacity(bindings.inputs.len());
    for binding in &bindings.inputs {
        let observation = observed.get(binding.artifact_id.as_str()).ok_or_else(|| {
            ArtifactAcceptanceError::Mismatch {
                message: "a declared input has no host observation".to_owned(),
            }
        })?;
        correlate_observation(
            &binding.artifact_id,
            &binding.port,
            &binding.media_type,
            binding.kind,
            &binding.locator,
            observation,
        )?;
        mismatch(
            observation.digest == binding.expected_digest,
            "host-observed input digest conflicts with the immutable binding",
        )?;
        accepted_inputs.push((*observation).clone());
    }

    let mut accepted_outputs = Vec::with_capacity(bindings.outputs.len());
    for binding in &bindings.outputs {
        let observation = observed.get(binding.artifact_id.as_str()).ok_or_else(|| {
            ArtifactAcceptanceError::Mismatch {
                message: "a declared output has no host observation".to_owned(),
            }
        })?;
        correlate_observation(
            &binding.artifact_id,
            &binding.port,
            &binding.media_type,
            binding.kind,
            &binding.locator,
            observation,
        )?;
        accepted_outputs.push((*observation).clone());
    }

    Ok(AcceptedArtifactSet {
        binding_set_id: bindings.binding_set_id.clone(),
        inputs: accepted_inputs,
        outputs: accepted_outputs,
    })
}

#[derive(Debug, Error)]
pub enum ArtifactObservationError {
    #[error("invalid artifact bindings: {source}")]
    InvalidBindings {
        #[source]
        source: ValidationError,
    },
    #[error("invalid generated artifact observations: {source}")]
    InvalidObservations {
        #[source]
        source: ValidationError,
    },
    #[error("artifact root is a symlink: {path}")]
    RootSymlink { path: String },
    #[error("artifact root is not a directory: {path}")]
    RootNotDirectory { path: String },
    #[error("artifact {artifact_id} path contains a symlink at {path}")]
    Symlink { artifact_id: String, path: String },
    #[error("artifact {artifact_id} is missing at {locator}")]
    Missing {
        artifact_id: String,
        locator: String,
    },
    #[error("artifact {artifact_id} expected {expected:?} but observed {observed:?}")]
    KindMismatch {
        artifact_id: String,
        expected: ArtifactKind,
        observed: ArtifactKind,
    },
    #[error("artifact {artifact_id} contains unsupported filesystem node {path}")]
    UnsupportedNode { artifact_id: String, path: String },
    #[error("artifact {artifact_id} contains a non-UTF-8 path at {path}")]
    NonUtf8Path { artifact_id: String, path: String },
    #[error("artifact {artifact_id} resolved outside the selected root: {locator}")]
    PathEscape {
        artifact_id: String,
        locator: String,
    },
    #[error("failed to {operation} at {path}: {source}")]
    Io {
        operation: &'static str,
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("failed to encode a deterministic directory manifest: {message}")]
    ManifestEncoding { message: String },
    #[error("artifact byte count exceeds the supported u64 range")]
    SizeOverflow,
}

#[derive(Debug, Error)]
pub enum ArtifactAcceptanceError {
    #[error("invalid invocation: {source}")]
    InvalidInvocation {
        #[source]
        source: ValidationError,
    },
    #[error("invalid artifact bindings: {source}")]
    InvalidBindings {
        #[source]
        source: ValidationError,
    },
    #[error("invalid host artifact observations: {source}")]
    InvalidObservations {
        #[source]
        source: ValidationError,
    },
    #[error("artifact acceptance mismatch: {message}")]
    Mismatch { message: String },
}

#[derive(Clone, Copy)]
struct BindingView<'a> {
    artifact_id: &'a str,
    port: &'a str,
    media_type: &'a str,
    kind: ArtifactKind,
    locator: &'a str,
}

impl<'a> From<&'a InputArtifactBinding> for BindingView<'a> {
    fn from(binding: &'a InputArtifactBinding) -> Self {
        Self {
            artifact_id: &binding.artifact_id,
            port: &binding.port,
            media_type: &binding.media_type,
            kind: binding.kind,
            locator: &binding.locator,
        }
    }
}

impl<'a> From<&'a OutputArtifactBinding> for BindingView<'a> {
    fn from(binding: &'a OutputArtifactBinding) -> Self {
        Self {
            artifact_id: &binding.artifact_id,
            port: &binding.port,
            media_type: &binding.media_type,
            kind: binding.kind,
            locator: &binding.locator,
        }
    }
}

#[derive(Serialize)]
struct CanonicalDirectoryChild<'a> {
    name: &'a str,
    kind: ArtifactKind,
    digest: &'a str,
    size_bytes: u64,
}

struct ObservedNode {
    kind: ArtifactKind,
    digest: String,
    size_bytes: u64,
    manifest: Vec<DirectoryManifestEntry>,
}

fn observe_binding(
    root: &Path,
    canonical_root: &Path,
    binding: BindingView<'_>,
) -> Result<HostArtifactObservation, ArtifactObservationError> {
    let target = checked_target(root, binding)?;
    let canonical_target = fs::canonicalize(&target).map_err(|source| {
        if source.kind() == io::ErrorKind::NotFound {
            ArtifactObservationError::Missing {
                artifact_id: binding.artifact_id.to_owned(),
                locator: binding.locator.to_owned(),
            }
        } else {
            ArtifactObservationError::Io {
                operation: "canonicalize artifact",
                path: display_path(&target),
                source,
            }
        }
    })?;
    if !canonical_target.starts_with(canonical_root) {
        return Err(ArtifactObservationError::PathEscape {
            artifact_id: binding.artifact_id.to_owned(),
            locator: binding.locator.to_owned(),
        });
    }

    let node = observe_node(&target, "", binding.artifact_id)?;
    if node.kind != binding.kind {
        return Err(ArtifactObservationError::KindMismatch {
            artifact_id: binding.artifact_id.to_owned(),
            expected: binding.kind,
            observed: node.kind,
        });
    }
    Ok(HostArtifactObservation {
        artifact_id: binding.artifact_id.to_owned(),
        port: binding.port.to_owned(),
        media_type: binding.media_type.to_owned(),
        kind: node.kind,
        locator: binding.locator.to_owned(),
        digest: node.digest,
        size_bytes: node.size_bytes,
        manifest: node.manifest,
    })
}

fn checked_target(
    root: &Path,
    binding: BindingView<'_>,
) -> Result<PathBuf, ArtifactObservationError> {
    let mut target = root.to_path_buf();
    for segment in binding.locator.split('/') {
        target.push(segment);
        let metadata = fs::symlink_metadata(&target).map_err(|source| {
            if source.kind() == io::ErrorKind::NotFound {
                ArtifactObservationError::Missing {
                    artifact_id: binding.artifact_id.to_owned(),
                    locator: binding.locator.to_owned(),
                }
            } else {
                ArtifactObservationError::Io {
                    operation: "inspect artifact path",
                    path: display_path(&target),
                    source,
                }
            }
        })?;
        if metadata.file_type().is_symlink() {
            return Err(ArtifactObservationError::Symlink {
                artifact_id: binding.artifact_id.to_owned(),
                path: display_path(&target),
            });
        }
    }
    Ok(target)
}

fn observe_node(
    path: &Path,
    relative: &str,
    artifact_id: &str,
) -> Result<ObservedNode, ArtifactObservationError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| ArtifactObservationError::Io {
        operation: "inspect artifact node",
        path: display_path(path),
        source,
    })?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        return Err(ArtifactObservationError::Symlink {
            artifact_id: artifact_id.to_owned(),
            path: display_path(path),
        });
    }
    if file_type.is_file() {
        return Ok(ObservedNode {
            kind: ArtifactKind::File,
            digest: digest_file(path)?,
            size_bytes: metadata.len(),
            manifest: Vec::new(),
        });
    }
    if !file_type.is_dir() {
        return Err(ArtifactObservationError::UnsupportedNode {
            artifact_id: artifact_id.to_owned(),
            path: display_path(path),
        });
    }

    let mut children = fs::read_dir(path)
        .map_err(|source| ArtifactObservationError::Io {
            operation: "read artifact directory",
            path: display_path(path),
            source,
        })?
        .map(|entry| {
            entry.map_err(|source| ArtifactObservationError::Io {
                operation: "read artifact directory entry",
                path: display_path(path),
                source,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    children.sort_by_key(fs::DirEntry::file_name);

    let mut direct_entries = Vec::with_capacity(children.len());
    let mut manifest = Vec::new();
    let mut size_bytes = 0_u64;
    for child in children {
        let file_name = child.file_name().into_string().map_err(|name| {
            ArtifactObservationError::NonUtf8Path {
                artifact_id: artifact_id.to_owned(),
                path: display_path(&path.join(name)),
            }
        })?;
        let child_relative = if relative.is_empty() {
            file_name.clone()
        } else {
            format!("{relative}/{file_name}")
        };
        validate_locator("directory manifest locator", &child_relative)
            .map_err(|source| ArtifactObservationError::InvalidObservations { source })?;
        let observed = observe_node(&child.path(), &child_relative, artifact_id)?;
        size_bytes = size_bytes
            .checked_add(observed.size_bytes)
            .ok_or(ArtifactObservationError::SizeOverflow)?;
        direct_entries.push(DirectoryManifestEntry {
            locator: file_name,
            kind: observed.kind,
            digest: observed.digest.clone(),
            size_bytes: observed.size_bytes,
        });
        manifest.push(DirectoryManifestEntry {
            locator: child_relative,
            kind: observed.kind,
            digest: observed.digest,
            size_bytes: observed.size_bytes,
        });
        manifest.extend(observed.manifest);
    }
    manifest.sort_by(|left, right| left.locator.cmp(&right.locator));
    let digest = digest_directory(&direct_entries)?;
    Ok(ObservedNode {
        kind: ArtifactKind::Directory,
        digest,
        size_bytes,
        manifest,
    })
}

fn digest_file(path: &Path) -> Result<String, ArtifactObservationError> {
    let mut file = File::open(path).map_err(|source| ArtifactObservationError::Io {
        operation: "open artifact file",
        path: display_path(path),
        source,
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|source| ArtifactObservationError::Io {
                operation: "read artifact file",
                path: display_path(path),
                source,
            })?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn digest_directory(
    entries: &[DirectoryManifestEntry],
) -> Result<String, ArtifactObservationError> {
    let mut entries = entries.iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| left.locator.cmp(&right.locator));
    let canonical = entries
        .iter()
        .map(|entry| CanonicalDirectoryChild {
            name: &entry.locator,
            kind: entry.kind,
            digest: &entry.digest,
            size_bytes: entry.size_bytes,
        })
        .collect::<Vec<_>>();
    let bytes = serde_json::to_vec(&canonical).map_err(|error| {
        ArtifactObservationError::ManifestEncoding {
            message: error.to_string(),
        }
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn validate_directory_manifest(
    observation: &HostArtifactObservation,
    path: &str,
) -> Result<(), ValidationError> {
    let mut previous = None;
    let mut entries = BTreeMap::new();
    for (index, entry) in observation.manifest.iter().enumerate() {
        let entry_path = format!("{path}.manifest[{index}]");
        validate_locator(&format!("{entry_path}.locator"), &entry.locator)?;
        validate_digest(&format!("{entry_path}.digest"), &entry.digest)?;
        if let Some(previous) = previous {
            expect(
                previous < entry.locator.as_str(),
                format!("{path}.manifest"),
                "entries must be unique and sorted by locator",
            )?;
        }
        previous = Some(entry.locator.as_str());
        entries.insert(entry.locator.as_str(), entry);
    }
    for (locator, entry) in &entries {
        if let Some(parent) = parent_locator(locator) {
            let Some(parent_entry) = entries.get(parent) else {
                return Err(ValidationError::new(
                    format!("{path}.manifest"),
                    "every nested entry must name a declared parent directory",
                ));
            };
            expect(
                parent_entry.kind == ArtifactKind::Directory,
                format!("{path}.manifest"),
                "nested entries require a directory parent",
            )?;
        }
        if entry.kind == ArtifactKind::Directory {
            let (digest, size) = manifest_directory_identity(locator, &entries)
                .map_err(|message| ValidationError::new(format!("{path}.manifest"), message))?;
            expect(
                entry.digest == digest && entry.size_bytes == size,
                format!("{path}.manifest"),
                "directory entry digest or size conflicts with its children",
            )?;
        }
    }
    let (digest, size) = manifest_directory_identity("", &entries)
        .map_err(|message| ValidationError::new(format!("{path}.manifest"), message))?;
    expect(
        observation.digest == digest && observation.size_bytes == size,
        path,
        "directory observation digest or size conflicts with its manifest",
    )
}

fn manifest_directory_identity(
    directory: &str,
    entries: &BTreeMap<&str, &DirectoryManifestEntry>,
) -> Result<(String, u64), String> {
    let mut direct = Vec::new();
    let mut size = 0_u64;
    for (locator, entry) in entries {
        if parent_locator(locator).unwrap_or("") == directory {
            let name = locator.rsplit('/').next().unwrap_or(locator);
            direct.push(DirectoryManifestEntry {
                locator: name.to_owned(),
                kind: entry.kind,
                digest: entry.digest.clone(),
                size_bytes: entry.size_bytes,
            });
            size = size
                .checked_add(entry.size_bytes)
                .ok_or_else(|| "directory size exceeds u64".to_owned())?;
        }
    }
    digest_directory(&direct)
        .map(|digest| (digest, size))
        .map_err(|error| error.to_string())
}

fn correlate_observation(
    artifact_id: &str,
    port: &str,
    media_type: &str,
    kind: ArtifactKind,
    locator: &str,
    observation: &HostArtifactObservation,
) -> Result<(), ArtifactAcceptanceError> {
    mismatch(
        observation.artifact_id == artifact_id
            && observation.port == port
            && observation.media_type == media_type
            && observation.kind == kind
            && observation.locator == locator,
        "host observation identity, port, type, kind, or locator conflicts with its binding",
    )
}

fn correlate_execution_context(
    resolved: &ResolvedExtension,
    invocation: &ExtensionInvocation,
    execution: &ValidatedExecution,
) -> Result<(), ArtifactAcceptanceError> {
    let result = execution.result();
    mismatch(
        invocation.extension.extension_id == resolved.extension_id()
            && invocation.extension.version == resolved.version()
            && invocation.extension.publisher_id == resolved.publisher_id()
            && invocation.extension.integrity == resolved.integrity().value
            && invocation.capability_id == resolved.capability().capability_id
            && invocation.interface.name == resolved.execution_mode().name
            && invocation.interface.kind == resolved.execution_mode().kind
            && invocation.interface.protocol == resolved.execution_mode().protocol
            && invocation.authorization.lock_id == resolved.lock_id()
            && invocation.configuration.schema_id == resolved.capability().configuration_schema
            && invocation.expected_output_types == resolved.capability().produces
            && invocation.limits == resolved.execution_mode().limits,
        "invocation context does not match the resolved extension",
    )?;
    mismatch(
        result.run_id == invocation.run_id
            && result.invocation_id == invocation.invocation_id
            && result.extension_id == invocation.extension.extension_id
            && result.extension_version == invocation.extension.version
            && result.extension_integrity == invocation.extension.integrity
            && result.capability_id == invocation.capability_id
            && result.configuration_digest == invocation.configuration.digest
            && result.authorization_id == invocation.authorization.authorization_id,
        "validated execution context does not match the supplied invocation",
    )
}

fn validate_binding(
    path: &str,
    artifact_id: &str,
    port: &str,
    media_type: &str,
    locator: &str,
) -> Result<(), ValidationError> {
    expect(
        has_prefixed_id(artifact_id, "artifact:"),
        format!("{path}.artifact_id"),
        "must be a lowercase artifact identifier",
    )?;
    expect(
        has_prefixed_id(port, "port:"),
        format!("{path}.port"),
        "must be a lowercase port identifier",
    )?;
    validate_media_type(&format!("{path}.media_type"), media_type)?;
    validate_locator(&format!("{path}.locator"), locator)
}

fn validate_media_type(path: &str, value: &str) -> Result<(), ValidationError> {
    let Some((top, subtype)) = value.split_once('/') else {
        return Err(ValidationError::new(
            path,
            "must contain one type separator",
        ));
    };
    expect(
        !top.is_empty()
            && !subtype.is_empty()
            && !subtype.contains('/')
            && !value.contains('*')
            && !value.chars().any(char::is_whitespace),
        path,
        "must be a concrete nonempty type/subtype without whitespace",
    )
}

pub(crate) fn validate_locator(path: &str, locator: &str) -> Result<(), ValidationError> {
    expect(!locator.is_empty(), path, "must not be empty")?;
    expect(
        !locator.starts_with('/') && !locator.contains('\\') && !locator.contains(':'),
        path,
        "must be a portable root-relative locator",
    )?;
    expect(
        locator.split('/').all(is_portable_segment),
        path,
        "contains an invalid or host-ambiguous path segment",
    )?;
    let components_are_normal = Path::new(locator)
        .components()
        .all(|component| matches!(component, Component::Normal(_)));
    expect(
        !Path::new(locator).is_absolute() && components_are_normal,
        path,
        "must contain only normal relative path components",
    )
}

fn is_portable_segment(segment: &str) -> bool {
    if segment.is_empty()
        || segment == "."
        || segment == ".."
        || segment.ends_with(' ')
        || segment.ends_with('.')
        || segment.chars().any(|character| {
            character.is_control() || matches!(character, '"' | '*' | '<' | '>' | '?' | '|')
        })
    {
        return false;
    }
    let stem = segment
        .split_once('.')
        .map_or(segment, |(stem, _suffix)| stem);
    !is_windows_device_name(stem)
}

fn is_windows_device_name(stem: &str) -> bool {
    let upper = stem.to_ascii_uppercase();
    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || upper
            .strip_prefix("COM")
            .or_else(|| upper.strip_prefix("LPT"))
            .is_some_and(|suffix| suffix.len() == 1 && matches!(suffix.as_bytes()[0], b'1'..=b'9'))
}

pub(crate) fn validate_digest(path: &str, digest: &str) -> Result<(), ValidationError> {
    expect(
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        path,
        "must be 64 lowercase hexadecimal characters",
    )
}

fn unique_field<T, F>(
    values: &[T],
    field: F,
    path: &str,
    message: &str,
) -> Result<(), ValidationError>
where
    F: Fn(&T) -> &str,
{
    let unique = values.iter().map(field).collect::<HashSet<_>>();
    expect(unique.len() == values.len(), path, message)
}

pub(crate) fn has_prefixed_id(value: &str, prefix: &str) -> bool {
    let Some(identifier) = value.strip_prefix(prefix) else {
        return false;
    };
    let mut bytes = identifier.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes.all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || byte == b'.'
                || byte == b'_'
                || byte == b'-'
        })
}

fn media_type_allowed(patterns: &[String], media_type: &str) -> bool {
    patterns
        .iter()
        .any(|pattern| media_type_matches(pattern, media_type))
}

fn media_type_matches(pattern: &str, media_type: &str) -> bool {
    pattern == media_type
        || pattern
            .strip_suffix("/*")
            .is_some_and(|top| media_type.starts_with(&format!("{top}/")))
}

fn exact_identifiers<'a, T, I>(
    actual: &[T],
    expected: I,
    message: &str,
) -> Result<(), ArtifactAcceptanceError>
where
    T: AsRef<str>,
    I: IntoIterator<Item = &'a str>,
{
    let actual = actual.iter().map(AsRef::as_ref).collect::<Vec<_>>();
    let expected = expected.into_iter().collect::<HashSet<_>>();
    let actual_unique = actual.iter().copied().collect::<HashSet<_>>();
    mismatch(
        actual.len() == expected.len() && actual_unique == expected,
        message,
    )
}

fn mismatch(condition: bool, message: &str) -> Result<(), ArtifactAcceptanceError> {
    if condition {
        Ok(())
    } else {
        mismatch_error(message)
    }
}

fn mismatch_error<T>(message: &str) -> Result<T, ArtifactAcceptanceError> {
    Err(ArtifactAcceptanceError::Mismatch {
        message: message.to_owned(),
    })
}

fn expect(
    condition: bool,
    path: impl Into<String>,
    message: impl Into<String>,
) -> Result<(), ValidationError> {
    if condition {
        Ok(())
    } else {
        Err(ValidationError::new(path, message))
    }
}

fn parent_locator(locator: &str) -> Option<&str> {
    locator.rsplit_once('/').map(|(parent, _)| parent)
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
