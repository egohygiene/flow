//! Versioned, deterministic descriptions of Flow orchestration test scenarios.
//!
//! A scenario manifest describes conformance intent. It references immutable
//! provider, artifact, evidence, and transition documents by schema and digest;
//! it does not define a runtime plan, run, checkpoint, or execution engine.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::contracts::{
    EXTENSION_MANIFEST_V1, ValidationError, contract_family_major, is_capability_id,
    is_extension_id,
};

pub const SCENARIO_MANIFEST_V1: &str = "flow.scenario-manifest/v1";
pub const SCENARIO_CANONICALIZATION_V1: &str = "flow.canonical-json/v1";

/// One Flow-owned orchestration conformance scenario.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioManifest {
    pub schema_version: String,
    pub scenario_id: String,
    pub fixture_id: String,
    pub fixture_version: String,
    pub title: String,
    pub description: String,
    pub fixture_class: FixtureClass,
    pub tags: Vec<String>,
    pub inputs: Vec<ScenarioInput>,
    pub providers: Vec<ScenarioProvider>,
    pub stages: Vec<ScenarioStage>,
    pub expectation: ScenarioExpectation,
    pub execution: ScenarioExecution,
    pub coverage: ScenarioCoverage,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FixtureClass {
    Valid,
    Boundary,
    Malformed,
    Unsupported,
    ResourceStress,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioInput {
    pub artifact_id: String,
    pub media_type: String,
    pub artifact_contract: ContractDocumentReference,
    pub source: ImmutableSource,
}

/// A closed reference to another versioned contract document.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContractDocumentReference {
    pub schema_version: String,
    pub digest: String,
}

/// Retrieval metadata whose identity is always bound to immutable evidence.
///
/// `locator` is a non-authoritative retrieval hint. `revision` and `digest`
/// establish the pinned recipe/revision and exact bytes respectively.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImmutableSource {
    pub kind: SourceKind,
    pub locator: String,
    pub revision: String,
    pub digest: String,
    pub license: String,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_generator",
        skip_serializing_if = "Option::is_none"
    )]
    pub generator: Option<GeneratorProvenance>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    Generated,
    Vendored,
    Released,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratorProvenance {
    pub generator_id: String,
    pub version: String,
    pub seed: String,
    pub parameters_digest: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioProvider {
    pub provider_id: String,
    pub version: String,
    pub interface: ProviderInterface,
    pub manifest: ContractDocumentReference,
    pub package: ImmutableSource,
    pub required_capabilities: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderInterface {
    InProcess,
    Process,
}

/// An ordered, topologically valid scenario step.
///
/// Stage array order is contract-significant and provides a deterministic
/// tie-break between otherwise independent ready stages.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioStage {
    pub stage_id: String,
    pub provider_id: String,
    pub capability_id: String,
    pub depends_on: Vec<String>,
    pub consumes: Vec<String>,
    pub produces: Vec<String>,
    pub required: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioExpectation {
    pub terminal_state: ScenarioTerminalState,
    pub evidence_state: ScenarioEvidenceState,
    pub expected_artifacts: Vec<String>,
    pub expected_diagnostics: Vec<String>,
    pub evidence_refs: Vec<ContractDocumentReference>,
    pub state_trace_refs: Vec<ContractDocumentReference>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScenarioTerminalState {
    Complete,
    Partial,
    Unavailable,
    Unsupported,
    Invalid,
    Failed,
    Interrupted,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScenarioEvidenceState {
    Complete,
    ObservedEmpty,
    Unavailable,
    Incomplete,
    Unsupported,
    Invalid,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioExecution {
    pub tier: ExecutionTier,
    pub clean_room: bool,
    pub network_mode: NetworkMode,
    pub external_services: Vec<String>,
    pub budget: ResourceBudget,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionTier {
    PullRequest,
    Scheduled,
    Release,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkMode {
    Denied,
    Allowlisted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceBudget {
    pub timeout_ms: u64,
    pub max_memory_bytes: u64,
    pub max_stdout_bytes: u64,
    pub max_stderr_bytes: u64,
    pub max_artifact_bytes: u64,
    pub max_artifacts: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioCoverage {
    pub claim: CoverageClaim,
    pub covered_behaviors: Vec<String>,
    pub known_gaps: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoverageClaim {
    ScenarioOnly,
}

/// Canonicalization can fail only when the manifest is invalid or JSON
/// serialization unexpectedly fails.
#[derive(Debug, Error)]
pub enum ScenarioCanonicalizationError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("scenario JSON serialization failed: {0}")]
    Json(#[from] serde_json::Error),
}

impl ScenarioManifest {
    /// Validate schema-level shapes and cross-field scenario invariants.
    ///
    /// # Errors
    ///
    /// Returns the first deterministic schema or cross-field violation.
    #[allow(clippy::too_many_lines)]
    pub fn validate(&self) -> Result<(), ValidationError> {
        expect(
            "scenario.schema_version",
            &self.schema_version,
            SCENARIO_MANIFEST_V1,
        )?;
        prefixed_id("scenario.scenario_id", &self.scenario_id, "scenario:")?;
        prefixed_id("scenario.fixture_id", &self.fixture_id, "fixture:")?;
        strict_version("scenario.fixture_version", &self.fixture_version)?;
        nonempty("scenario.title", &self.title)?;
        nonempty("scenario.description", &self.description)?;
        strings("scenario.tags", &self.tags)?;

        unique_by(
            "scenario.inputs",
            &self.inputs,
            |input| &input.artifact_id,
            "input artifact identifiers must be unique",
        )?;
        for (index, input) in self.inputs.iter().enumerate() {
            identifier(
                &format!("scenario.inputs[{index}].artifact_id"),
                &input.artifact_id,
            )?;
            nonempty(
                format!("scenario.inputs[{index}].media_type"),
                &input.media_type,
            )?;
            expect(
                &format!("scenario.inputs[{index}].artifact_contract.schema_version"),
                &input.artifact_contract.schema_version,
                "flow.artifact/v1",
            )?;
            reference(
                &format!("scenario.inputs[{index}].artifact_contract"),
                &input.artifact_contract,
            )?;
            source(&format!("scenario.inputs[{index}].source"), &input.source)?;
        }

        unique_by(
            "scenario.providers",
            &self.providers,
            |provider| &provider.provider_id,
            "provider identifiers must be unique",
        )?;
        for (index, provider) in self.providers.iter().enumerate() {
            qualified_id(
                &format!("scenario.providers[{index}].provider_id"),
                &provider.provider_id,
            )?;
            strict_version(
                &format!("scenario.providers[{index}].version"),
                &provider.version,
            )?;
            expect(
                &format!("scenario.providers[{index}].manifest.schema_version"),
                &provider.manifest.schema_version,
                EXTENSION_MANIFEST_V1,
            )?;
            reference(
                &format!("scenario.providers[{index}].manifest"),
                &provider.manifest,
            )?;
            source(
                &format!("scenario.providers[{index}].package"),
                &provider.package,
            )?;
            if provider.required_capabilities.is_empty() {
                return Err(ValidationError::new(
                    format!("scenario.providers[{index}].required_capabilities"),
                    "must contain at least one capability",
                ));
            }
            strings(
                &format!("scenario.providers[{index}].required_capabilities"),
                &provider.required_capabilities,
            )?;
            for (capability_index, capability) in provider.required_capabilities.iter().enumerate()
            {
                capability_id(
                    &format!(
                        "scenario.providers[{index}].required_capabilities[{capability_index}]"
                    ),
                    capability,
                )?;
            }
        }

        self.validate_stages()?;
        self.validate_expectation()?;
        self.validate_execution()?;
        self.validate_coverage()
    }

    /// Return canonical UTF-8 JSON bytes for identity and drift checks.
    ///
    /// Object keys and set-like collections are sorted. Stage order remains
    /// unchanged because it is a contract-significant topological tie-break.
    ///
    /// # Errors
    ///
    /// Returns an error when validation or serialization fails.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ScenarioCanonicalizationError> {
        self.validate()?;
        let normalized = self.normalized();
        let value = serde_json::to_value(normalized)?;
        let mut bytes = Vec::new();
        write_canonical_json(&value, &mut bytes)?;
        Ok(bytes)
    }

    /// Return the lowercase SHA-256 digest of [`Self::canonical_bytes`].
    ///
    /// # Errors
    ///
    /// Returns an error when validation or serialization fails.
    pub fn canonical_sha256(&self) -> Result<String, ScenarioCanonicalizationError> {
        let digest = Sha256::digest(self.canonical_bytes()?);
        Ok(format!("{digest:x}"))
    }

    fn normalized(&self) -> Self {
        let mut normalized = self.clone();
        normalized.tags.sort();
        normalized
            .inputs
            .sort_by(|left, right| left.artifact_id.cmp(&right.artifact_id));
        normalized
            .providers
            .sort_by(|left, right| left.provider_id.cmp(&right.provider_id));
        for provider in &mut normalized.providers {
            provider.required_capabilities.sort();
        }
        for stage in &mut normalized.stages {
            stage.depends_on.sort();
            stage.consumes.sort();
            stage.produces.sort();
        }
        normalized.expectation.expected_artifacts.sort();
        normalized.expectation.expected_diagnostics.sort();
        normalized.expectation.evidence_refs.sort();
        normalized.expectation.state_trace_refs.sort();
        normalized.execution.external_services.sort();
        normalized.coverage.covered_behaviors.sort();
        normalized.coverage.known_gaps.sort();
        normalized
    }

    #[allow(clippy::too_many_lines)]
    fn validate_stages(&self) -> Result<(), ValidationError> {
        unique_by(
            "scenario.stages",
            &self.stages,
            |stage| &stage.stage_id,
            "stage identifiers must be unique",
        )?;
        let providers: BTreeMap<&str, &ScenarioProvider> = self
            .providers
            .iter()
            .map(|provider| (provider.provider_id.as_str(), provider))
            .collect();
        let mut known_stages: BTreeSet<&str> = BTreeSet::new();
        let mut ancestors: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        let mut available_artifacts: BTreeMap<&str, Option<&str>> = self
            .inputs
            .iter()
            .map(|input| (input.artifact_id.as_str(), None))
            .collect();

        for (index, stage) in self.stages.iter().enumerate() {
            identifier(
                &format!("scenario.stages[{index}].stage_id"),
                &stage.stage_id,
            )?;
            let provider = providers.get(stage.provider_id.as_str()).ok_or_else(|| {
                ValidationError::new(
                    format!("scenario.stages[{index}].provider_id"),
                    "references an unknown provider",
                )
            })?;
            capability_id(
                &format!("scenario.stages[{index}].capability_id"),
                &stage.capability_id,
            )?;
            if !provider
                .required_capabilities
                .contains(&stage.capability_id)
            {
                return Err(ValidationError::new(
                    format!("scenario.stages[{index}].capability_id"),
                    "must be declared by the selected provider reference",
                ));
            }
            strings(
                &format!("scenario.stages[{index}].depends_on"),
                &stage.depends_on,
            )?;
            strings(
                &format!("scenario.stages[{index}].consumes"),
                &stage.consumes,
            )?;
            strings(
                &format!("scenario.stages[{index}].produces"),
                &stage.produces,
            )?;

            let mut stage_ancestors = BTreeSet::new();
            for dependency in &stage.depends_on {
                if !known_stages.contains(dependency.as_str()) {
                    return Err(ValidationError::new(
                        format!("scenario.stages[{index}].depends_on"),
                        format!("dependency {dependency} must reference an earlier stage"),
                    ));
                }
                stage_ancestors.insert(dependency.as_str());
                if let Some(transitive) = ancestors.get(dependency.as_str()) {
                    stage_ancestors.extend(transitive.iter().copied());
                }
            }

            for artifact_id in &stage.consumes {
                let producer = available_artifacts
                    .get(artifact_id.as_str())
                    .ok_or_else(|| {
                        ValidationError::new(
                            format!("scenario.stages[{index}].consumes"),
                            format!("references unknown artifact {artifact_id}"),
                        )
                    })?;
                if let Some(producer) = producer {
                    if !stage_ancestors.contains(producer) {
                        return Err(ValidationError::new(
                            format!("scenario.stages[{index}].depends_on"),
                            format!(
                                "must depend on producing stage {producer} for artifact {artifact_id}"
                            ),
                        ));
                    }
                }
            }
            for artifact_id in &stage.produces {
                identifier(&format!("scenario.stages[{index}].produces"), artifact_id)?;
                if available_artifacts
                    .insert(artifact_id.as_str(), Some(stage.stage_id.as_str()))
                    .is_some()
                {
                    return Err(ValidationError::new(
                        format!("scenario.stages[{index}].produces"),
                        format!("artifact identifier {artifact_id} is already declared"),
                    ));
                }
            }
            known_stages.insert(stage.stage_id.as_str());
            ancestors.insert(stage.stage_id.as_str(), stage_ancestors);
        }
        Ok(())
    }

    fn validate_expectation(&self) -> Result<(), ValidationError> {
        strings(
            "scenario.expectation.expected_artifacts",
            &self.expectation.expected_artifacts,
        )?;
        strings(
            "scenario.expectation.expected_diagnostics",
            &self.expectation.expected_diagnostics,
        )?;
        let known_artifacts: HashSet<&str> = self
            .inputs
            .iter()
            .map(|input| input.artifact_id.as_str())
            .chain(
                self.stages
                    .iter()
                    .flat_map(|stage| stage.produces.iter().map(String::as_str)),
            )
            .collect();
        for artifact_id in &self.expectation.expected_artifacts {
            if !known_artifacts.contains(artifact_id.as_str()) {
                return Err(ValidationError::new(
                    "scenario.expectation.expected_artifacts",
                    format!("references unknown artifact {artifact_id}"),
                ));
            }
        }
        references(
            "scenario.expectation.evidence_refs",
            &self.expectation.evidence_refs,
        )?;
        references(
            "scenario.expectation.state_trace_refs",
            &self.expectation.state_trace_refs,
        )?;

        let coherent = matches!(
            (
                self.expectation.terminal_state,
                self.expectation.evidence_state
            ),
            (
                ScenarioTerminalState::Complete,
                ScenarioEvidenceState::Complete | ScenarioEvidenceState::ObservedEmpty
            ) | (
                ScenarioTerminalState::Partial | ScenarioTerminalState::Interrupted,
                ScenarioEvidenceState::Incomplete
            ) | (
                ScenarioTerminalState::Unavailable,
                ScenarioEvidenceState::Unavailable
            ) | (
                ScenarioTerminalState::Unsupported,
                ScenarioEvidenceState::Unsupported
            ) | (
                ScenarioTerminalState::Invalid,
                ScenarioEvidenceState::Invalid
            ) | (ScenarioTerminalState::Failed, ScenarioEvidenceState::Failed)
        );
        if !coherent {
            return Err(ValidationError::new(
                "scenario.expectation",
                "terminal and evidence states are contradictory",
            ));
        }
        if self.expectation.evidence_state == ScenarioEvidenceState::ObservedEmpty
            && !self.expectation.expected_artifacts.is_empty()
        {
            return Err(ValidationError::new(
                "scenario.expectation.expected_artifacts",
                "observed-empty evidence cannot claim expected artifacts",
            ));
        }
        Ok(())
    }

    fn validate_execution(&self) -> Result<(), ValidationError> {
        strings(
            "scenario.execution.external_services",
            &self.execution.external_services,
        )?;
        let budget = &self.execution.budget;
        for (field, value) in [
            ("timeout_ms", budget.timeout_ms),
            ("max_memory_bytes", budget.max_memory_bytes),
            ("max_stdout_bytes", budget.max_stdout_bytes),
            ("max_stderr_bytes", budget.max_stderr_bytes),
            ("max_artifact_bytes", budget.max_artifact_bytes),
            ("max_artifacts", budget.max_artifacts),
        ] {
            if value == 0 {
                return Err(ValidationError::new(
                    format!("scenario.execution.budget.{field}"),
                    "must be greater than zero",
                ));
            }
        }
        if self.execution.network_mode == NetworkMode::Denied
            && !self.execution.external_services.is_empty()
        {
            return Err(ValidationError::new(
                "scenario.execution.external_services",
                "network-denied execution cannot declare external services",
            ));
        }
        if self.execution.network_mode == NetworkMode::Allowlisted
            && self.execution.external_services.is_empty()
        {
            return Err(ValidationError::new(
                "scenario.execution.external_services",
                "allowlisted network execution must name its services",
            ));
        }
        if self.execution.clean_room
            && (self.execution.network_mode != NetworkMode::Denied
                || !self.execution.external_services.is_empty())
        {
            return Err(ValidationError::new(
                "scenario.execution.clean_room",
                "clean-room execution must deny network access and external services",
            ));
        }
        if self.execution.tier == ExecutionTier::PullRequest && !self.execution.clean_room {
            return Err(ValidationError::new(
                "scenario.execution.clean_room",
                "pull-request scenarios must be clean-room",
            ));
        }
        Ok(())
    }

    fn validate_coverage(&self) -> Result<(), ValidationError> {
        if self.coverage.covered_behaviors.is_empty() {
            return Err(ValidationError::new(
                "scenario.coverage.covered_behaviors",
                "must contain at least one bounded behavior",
            ));
        }
        strings(
            "scenario.coverage.covered_behaviors",
            &self.coverage.covered_behaviors,
        )?;
        strings("scenario.coverage.known_gaps", &self.coverage.known_gaps)
    }
}

fn source(path: &str, value: &ImmutableSource) -> Result<(), ValidationError> {
    nonempty(format!("{path}.locator"), &value.locator)?;
    immutable_revision(&format!("{path}.revision"), &value.revision)?;
    digest(&format!("{path}.digest"), &value.digest)?;
    nonempty(format!("{path}.license"), &value.license)?;
    match (value.kind, &value.generator) {
        (SourceKind::Generated, Some(generator)) => {
            qualified_id(
                &format!("{path}.generator.generator_id"),
                &generator.generator_id,
            )?;
            strict_version(&format!("{path}.generator.version"), &generator.version)?;
            nonempty(format!("{path}.generator.seed"), &generator.seed)?;
            digest(
                &format!("{path}.generator.parameters_digest"),
                &generator.parameters_digest,
            )
        }
        (SourceKind::Generated, None) => Err(ValidationError::new(
            format!("{path}.generator"),
            "generated sources require generator provenance",
        )),
        (SourceKind::Vendored | SourceKind::Released, Some(_)) => Err(ValidationError::new(
            format!("{path}.generator"),
            "vendored and released sources cannot claim local generator provenance",
        )),
        (SourceKind::Vendored | SourceKind::Released, None) => Ok(()),
    }
}

fn deserialize_optional_generator<'de, D>(
    deserializer: D,
) -> Result<Option<GeneratorProvenance>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    GeneratorProvenance::deserialize(deserializer).map(Some)
}

fn write_canonical_json(
    value: &serde_json::Value,
    output: &mut Vec<u8>,
) -> Result<(), serde_json::Error> {
    match value {
        serde_json::Value::Object(object) => {
            output.push(b'{');
            let mut keys: Vec<&String> = object.keys().collect();
            keys.sort_unstable();
            for (index, key) in keys.into_iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                serde_json::to_writer(&mut *output, key)?;
                output.push(b':');
                let nested = object
                    .get(key)
                    .expect("canonical object key must remain present");
                write_canonical_json(nested, output)?;
            }
            output.push(b'}');
        }
        serde_json::Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                write_canonical_json(value, output)?;
            }
            output.push(b']');
        }
        _ => serde_json::to_writer(output, value)?,
    }
    Ok(())
}

fn reference(path: &str, value: &ContractDocumentReference) -> Result<(), ValidationError> {
    if contract_family_major(&value.schema_version).is_none() {
        return Err(ValidationError::new(
            format!("{path}.schema_version"),
            "must use flow.<family>/v<positive-major> syntax",
        ));
    }
    digest(&format!("{path}.digest"), &value.digest)
}

fn references(path: &str, values: &[ContractDocumentReference]) -> Result<(), ValidationError> {
    let mut seen = HashSet::new();
    for (index, value) in values.iter().enumerate() {
        reference(&format!("{path}[{index}]"), value)?;
        if !seen.insert((value.schema_version.as_str(), value.digest.as_str())) {
            return Err(ValidationError::new(path, "references must be unique"));
        }
    }
    Ok(())
}

fn strict_version(path: &str, value: &str) -> Result<(), ValidationError> {
    let parsed = Version::parse(value).map_err(|error| {
        ValidationError::new(path, format!("invalid semantic version: {error}"))
    })?;
    if parsed.pre.is_empty() && parsed.build.is_empty() && parsed.to_string() == value {
        Ok(())
    } else {
        Err(ValidationError::new(
            path,
            "must use strict major.minor.patch syntax",
        ))
    }
}

fn immutable_revision(path: &str, value: &str) -> Result<(), ValidationError> {
    let valid = value
        .strip_prefix("git:")
        .is_some_and(|revision| is_lower_hex(revision, 40))
        || value
            .strip_prefix("sha256:")
            .is_some_and(|revision| is_lower_hex(revision, 64));
    if valid {
        Ok(())
    } else {
        Err(ValidationError::new(
            path,
            "must be git:<40-lowercase-hex> or sha256:<64-lowercase-hex>",
        ))
    }
}

fn digest(path: &str, value: &str) -> Result<(), ValidationError> {
    if is_lower_hex(value, 64) {
        Ok(())
    } else {
        Err(ValidationError::new(
            path,
            "must be 64 lowercase hexadecimal characters",
        ))
    }
}

fn prefixed_id(path: &str, value: &str, prefix: &str) -> Result<(), ValidationError> {
    let Some(rest) = value.strip_prefix(prefix) else {
        return Err(ValidationError::new(
            path,
            format!("must start with {prefix}"),
        ));
    };
    identifier(path, rest)
}

fn identifier(path: &str, value: &str) -> Result<(), ValidationError> {
    let mut bytes = value.bytes();
    let valid = matches!(bytes.next(), Some(byte) if byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        });
    if valid {
        Ok(())
    } else {
        Err(ValidationError::new(path, "must be a lowercase identifier"))
    }
}

fn qualified_id(path: &str, value: &str) -> Result<(), ValidationError> {
    if is_extension_id(value) {
        Ok(())
    } else {
        Err(ValidationError::new(
            path,
            "must be a lowercase qualified identifier",
        ))
    }
}

fn capability_id(path: &str, value: &str) -> Result<(), ValidationError> {
    if is_capability_id(value) {
        Ok(())
    } else {
        Err(ValidationError::new(
            path,
            "must be a qualified capability identifier",
        ))
    }
}

fn strings(path: &str, values: &[String]) -> Result<(), ValidationError> {
    let unique: HashSet<&String> = values.iter().collect();
    if unique.len() != values.len() {
        return Err(ValidationError::new(path, "items must be unique"));
    }
    for (index, value) in values.iter().enumerate() {
        nonempty(format!("{path}[{index}]"), value)?;
    }
    Ok(())
}

fn unique_by<'a, T, F>(
    path: &str,
    values: &'a [T],
    key: F,
    message: &str,
) -> Result<(), ValidationError>
where
    F: Fn(&'a T) -> &'a String,
{
    let unique: HashSet<&String> = values.iter().map(key).collect();
    if unique.len() == values.len() {
        Ok(())
    } else {
        Err(ValidationError::new(path, message))
    }
}

fn nonempty(path: impl Into<String>, value: &str) -> Result<(), ValidationError> {
    if value.is_empty() {
        Err(ValidationError::new(path, "must not be empty"))
    } else {
        Ok(())
    }
}

fn expect(path: &str, actual: &str, expected: &str) -> Result<(), ValidationError> {
    if actual == expected {
        Ok(())
    } else {
        Err(ValidationError::new(path, format!("must equal {expected}")))
    }
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
