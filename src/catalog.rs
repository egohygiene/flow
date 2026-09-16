//! Deterministic inspection and resolution of explicitly supplied extensions.

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashSet};

use semver::Version;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::contracts::{
    Capability, Domain, EXTENSION_RESOLUTION_V1, ExecutionMode, ExecutionModeKind, ExtensionLock,
    ExtensionManifest, ExtensionResolution, FallbackPolicy, Integrity, LockedExtension,
    ResolutionCandidate, ResolutionResult, Trust, ValidationError, contract_family_major,
    parse_flow_version_requirement,
};

/// Caller-supplied observation of installed bytes and pre-invocation availability.
///
/// Observations never grant authority. They are compared with both the provider
/// declaration and the operator lock.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionObservation {
    pub extension_id: String,
    pub version: String,
    pub publisher_id: String,
    pub integrity: Integrity,
    pub available: bool,
}

impl ExtensionObservation {
    #[must_use]
    pub fn new(
        extension_id: impl Into<String>,
        version: impl Into<String>,
        publisher_id: impl Into<String>,
        integrity: Integrity,
        available: bool,
    ) -> Self {
        Self {
            extension_id: extension_id.into(),
            version: version.into(),
            publisher_id: publisher_id.into(),
            integrity,
            available,
        }
    }

    fn key(&self) -> IdentityKey<'_> {
        IdentityKey {
            extension_id: &self.extension_id,
            version: &self.version,
            publisher_id: &self.publisher_id,
        }
    }
}

/// A fully explicit resolution request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionRequest {
    pub case_id: String,
    pub capability_id: String,
    pub domain: Domain,
    pub execution_mode_name: String,
    pub execution_mode_kind: ExecutionModeKind,
}

impl ResolutionRequest {
    #[must_use]
    pub fn new(
        case_id: impl Into<String>,
        capability_id: impl Into<String>,
        domain: Domain,
        execution_mode_name: impl Into<String>,
        execution_mode_kind: ExecutionModeKind,
    ) -> Self {
        Self {
            case_id: case_id.into(),
            capability_id: capability_id.into(),
            domain,
            execution_mode_name: execution_mode_name.into(),
            execution_mode_kind,
        }
    }
}

/// An inspected catalog whose inputs have been normalized without executing code.
#[derive(Clone, Debug)]
pub struct ExtensionCatalog {
    manifests: Vec<InspectedManifest>,
    lock: ExtensionLock,
    observations: BTreeMap<OwnedIdentityKey, ExtensionObservation>,
    flow_version: Version,
}

#[derive(Clone, Debug)]
struct InspectedManifest {
    manifest: ExtensionManifest,
    validation_error: Option<ValidationError>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct OwnedIdentityKey {
    extension_id: String,
    version: String,
    publisher_id: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct IdentityKey<'a> {
    extension_id: &'a str,
    version: &'a str,
    publisher_id: &'a str,
}

impl From<IdentityKey<'_>> for OwnedIdentityKey {
    fn from(key: IdentityKey<'_>) -> Self {
        Self {
            extension_id: key.extension_id.to_owned(),
            version: key.version.to_owned(),
            publisher_id: key.publisher_id.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum CatalogError {
    #[error("invalid extension lock: {0}")]
    InvalidLock(ValidationError),
    #[error("duplicate extension manifest identity: {identity}")]
    DuplicateManifest { identity: String },
    #[error("duplicate extension observation identity: {identity}")]
    DuplicateObservation { identity: String },
}

impl ExtensionCatalog {
    /// Inspect inert inputs. This function never loads or invokes provider code.
    ///
    /// # Errors
    ///
    /// Returns an error when the operator lock is invalid, an extension ID has
    /// more than one manifest, or an exact observation identity is repeated.
    pub fn inspect(
        manifests: impl IntoIterator<Item = ExtensionManifest>,
        lock: ExtensionLock,
        observations: impl IntoIterator<Item = ExtensionObservation>,
    ) -> Result<Self, CatalogError> {
        lock.validate().map_err(CatalogError::InvalidLock)?;
        let flow_version = Version::parse(&lock.flow_version).map_err(|error| {
            CatalogError::InvalidLock(ValidationError::new(
                "lock.flow_version",
                format!("invalid semantic version: {error}"),
            ))
        })?;

        let mut manifests: Vec<ExtensionManifest> = manifests.into_iter().collect();
        manifests.sort_by(|left, right| manifest_sort_key(left).cmp(&manifest_sort_key(right)));
        for pair in manifests.windows(2) {
            if pair[0].extension_id == pair[1].extension_id {
                return Err(CatalogError::DuplicateManifest {
                    identity: pair[0].extension_id.clone(),
                });
            }
        }
        let manifests = manifests
            .into_iter()
            .map(|manifest| InspectedManifest {
                validation_error: manifest.validate().err(),
                manifest,
            })
            .collect();

        let mut observations: Vec<ExtensionObservation> = observations.into_iter().collect();
        observations.sort_by(|left, right| left.key().cmp(&right.key()));
        for pair in observations.windows(2) {
            if pair[0].key() == pair[1].key() {
                return Err(CatalogError::DuplicateObservation {
                    identity: display_identity(
                        &pair[0].extension_id,
                        &pair[0].version,
                        &pair[0].publisher_id,
                    ),
                });
            }
        }
        let normalized_observations = observations
            .into_iter()
            .map(|observation| {
                let key = OwnedIdentityKey::from(observation.key());
                (key, observation)
            })
            .collect();

        Ok(Self {
            manifests,
            lock,
            observations: normalized_observations,
            flow_version,
        })
    }

    /// Resolve one capability and retain complete, versioned decision evidence.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn resolve(&self, request: &ResolutionRequest) -> ResolutionOutcome {
        if request.case_id.is_empty()
            || request.capability_id.is_empty()
            || request.execution_mode_name.is_empty()
        {
            return ResolutionOutcome::rejected(ExtensionResolution {
                schema_version: EXTENSION_RESOLUTION_V1.to_owned(),
                case_id: if request.case_id.is_empty() {
                    "invalid-request".to_owned()
                } else {
                    request.case_id.clone()
                },
                requested_capability: if request.capability_id.is_empty() {
                    "invalid-capability".to_owned()
                } else {
                    request.capability_id.clone()
                },
                candidates: Vec::new(),
                result: ResolutionResult::Malformed,
                selected_extension_ids: Vec::new(),
                fallback_order: Vec::new(),
                reasons: vec!["The resolution request is missing a required identity.".to_owned()],
            });
        }

        let malformed: Vec<(&ExtensionManifest, &ValidationError)> = self
            .manifests
            .iter()
            .filter_map(|candidate| {
                candidate
                    .validation_error
                    .as_ref()
                    .map(|error| (&candidate.manifest, error))
            })
            .collect();
        if !malformed.is_empty() {
            let reasons = malformed
                .into_iter()
                .enumerate()
                .map(|(index, (_, error))| {
                    format!(
                        "Catalog resolution is atomic because resolution-v1 cannot represent a malformed candidate without fabricating evidence; candidate ordinal {} failed semantic validation at {}.",
                        index + 1,
                        error.path
                    )
                })
                .collect();
            return ResolutionOutcome::rejected(ExtensionResolution {
                schema_version: EXTENSION_RESOLUTION_V1.to_owned(),
                case_id: request.case_id.clone(),
                requested_capability: request.capability_id.clone(),
                candidates: Vec::new(),
                result: ResolutionResult::Malformed,
                selected_extension_ids: Vec::new(),
                fallback_order: Vec::new(),
                reasons,
            });
        }

        let policy = self.lock.resolution_for(&request.capability_id);
        if self.policy_is_partially_materialized(policy) {
            return ResolutionOutcome::rejected(ExtensionResolution {
                schema_version: EXTENSION_RESOLUTION_V1.to_owned(),
                case_id: request.case_id.clone(),
                requested_capability: request.capability_id.clone(),
                candidates: Vec::new(),
                result: ResolutionResult::Malformed,
                selected_extension_ids: Vec::new(),
                fallback_order: Vec::new(),
                reasons: vec![
                    "The requested operator policy is only partially represented by supplied manifests."
                        .to_owned(),
                ],
            });
        }
        let mut assessed: Vec<AssessedCandidate<'_>> = self
            .manifests
            .iter()
            .map(|candidate| self.assess(&candidate.manifest, request, policy))
            .collect();
        assessed.sort_by(|left, right| assessment_sort_key(left).cmp(&assessment_sort_key(right)));

        let fallback_precedence = fallback_precedence(policy, &assessed);
        let is_fallback = fallback_precedence.is_some();
        let fallback_allowed =
            policy.is_some_and(|policy| policy.fallback == FallbackPolicy::BeforeEffects);

        if is_fallback && !fallback_allowed {
            if let Some(unavailable_precedence) = fallback_precedence {
                for candidate in &mut assessed {
                    if candidate.eligible()
                        && candidate.evidence.precedence <= unavailable_precedence
                    {
                        candidate.evidence.authorized = false;
                        candidate.evidence.reasons.push(
                            "Operator fallback policy forbids selecting this candidate after pre-invocation unavailability."
                                .to_owned(),
                        );
                    }
                }
            }
        }

        let eligible: Vec<&AssessedCandidate<'_>> = assessed
            .iter()
            .filter(|candidate| candidate.eligible())
            .collect();
        let max_precedence = eligible
            .iter()
            .map(|candidate| candidate.evidence.precedence)
            .max();
        let leaders: Vec<&AssessedCandidate<'_>> =
            max_precedence.map_or_else(Vec::new, |maximum| {
                eligible
                    .iter()
                    .copied()
                    .filter(|candidate| candidate.evidence.precedence == maximum)
                    .collect()
            });

        let fallback_order = if is_fallback && fallback_allowed && leaders.len() == 1 {
            fallback_order(policy, &assessed, &leaders[0].evidence.extension_id)
        } else {
            Vec::new()
        };

        let evidence_candidates = assessed
            .iter()
            .map(|candidate| candidate.evidence.clone())
            .collect::<Vec<_>>();

        if leaders.len() > 1 {
            return ResolutionOutcome::rejected(ExtensionResolution {
                schema_version: EXTENSION_RESOLUTION_V1.to_owned(),
                case_id: request.case_id.clone(),
                requested_capability: request.capability_id.clone(),
                candidates: evidence_candidates,
                result: ResolutionResult::Conflict,
                selected_extension_ids: Vec::new(),
                fallback_order: Vec::new(),
                reasons: vec![
                    "Multiple eligible candidates share the highest operator precedence."
                        .to_owned(),
                ],
            });
        }

        if let Some(selected) = leaders.first() {
            let (Some(locked), Some(capability), Some(execution_mode)) = (
                selected.locked,
                selected.capability,
                selected.execution_mode,
            ) else {
                return ResolutionOutcome::rejected(ExtensionResolution {
                    schema_version: EXTENSION_RESOLUTION_V1.to_owned(),
                    case_id: request.case_id.clone(),
                    requested_capability: request.capability_id.clone(),
                    candidates: Vec::new(),
                    result: ResolutionResult::Malformed,
                    selected_extension_ids: Vec::new(),
                    fallback_order: Vec::new(),
                    reasons: vec![
                        "An eligible candidate lacks required inspected resolution state."
                            .to_owned(),
                    ],
                });
            };
            let resolved = ResolvedExtension {
                manifest: selected.manifest.clone(),
                locked: locked.clone(),
                capability: capability.clone(),
                execution_mode: execution_mode.clone(),
                lock_id: self.lock.lock_id.clone(),
                fallback_policy: policy.map_or(FallbackPolicy::Forbidden, |entry| entry.fallback),
            };
            let reason = if fallback_order.is_empty() {
                "Selected the unique eligible candidate with highest operator precedence."
            } else {
                "The preferred provider was unavailable before invocation; selected the authorized fallback."
            };
            return ResolutionOutcome::selected(
                ExtensionResolution {
                    schema_version: EXTENSION_RESOLUTION_V1.to_owned(),
                    case_id: request.case_id.clone(),
                    requested_capability: request.capability_id.clone(),
                    candidates: evidence_candidates,
                    result: ResolutionResult::Selected,
                    selected_extension_ids: vec![resolved.extension_id().to_owned()],
                    fallback_order,
                    reasons: vec![reason.to_owned()],
                },
                resolved,
            );
        }

        let any_compatible = assessed
            .iter()
            .any(|candidate| candidate.evidence.compatible);
        let (result, reason) = if any_compatible {
            (
                ResolutionResult::Blocked,
                "Compatible candidates exist, but none is authorized and available under operator policy.",
            )
        } else if assessed.is_empty() {
            (
                ResolutionResult::NoCompatibleProvider,
                "No manifest candidates were supplied for the requested capability.",
            )
        } else {
            (
                ResolutionResult::NoCompatibleProvider,
                "No candidate satisfies the requested capability, domain, contract, and execution mode.",
            )
        };
        ResolutionOutcome::rejected(ExtensionResolution {
            schema_version: EXTENSION_RESOLUTION_V1.to_owned(),
            case_id: request.case_id.clone(),
            requested_capability: request.capability_id.clone(),
            candidates: evidence_candidates,
            result,
            selected_extension_ids: Vec::new(),
            fallback_order: Vec::new(),
            reasons: vec![reason.to_owned()],
        })
    }

    fn policy_is_partially_materialized(
        &self,
        policy: Option<&crate::contracts::CapabilityResolution>,
    ) -> bool {
        if self.manifests.is_empty() {
            return false;
        }
        let Some(policy) = policy else {
            return false;
        };
        let required_entries: Vec<&LockedExtension> = policy
            .ordered_extensions
            .iter()
            .filter_map(|extension_id| self.lock.extension(extension_id))
            .filter(|entry| entry.enabled && entry.trust == Trust::Trusted)
            .collect();
        let represented = required_entries
            .iter()
            .filter(|entry| {
                self.manifests.iter().any(|candidate| {
                    candidate.manifest.extension_id == entry.extension_id
                        && candidate.manifest.version == entry.version
                        && candidate.manifest.publisher.id == entry.publisher_id
                        && candidate.manifest.integrity == entry.integrity
                })
            })
            .count();
        represented > 0 && represented < required_entries.len()
    }

    #[allow(clippy::too_many_lines)]
    fn assess<'a>(
        &'a self,
        manifest: &'a ExtensionManifest,
        request: &ResolutionRequest,
        policy: Option<&crate::contracts::CapabilityResolution>,
    ) -> AssessedCandidate<'a> {
        let locked = self.lock.extension(&manifest.extension_id);
        let observation = self
            .observations
            .get(&OwnedIdentityKey::from(manifest_identity(manifest)));
        let mut compatible = true;
        let mut authorized = true;
        let mut available = true;
        let mut reasons = Vec::new();

        let flow_requirement =
            parse_flow_version_requirement(&manifest.compatibility.flow_version_requirement);
        match flow_requirement {
            Ok(requirement) if requirement.matches(&self.flow_version) => {}
            Ok(_) => {
                compatible = false;
                reasons.push(format!(
                    "Flow {} does not satisfy {}.",
                    self.flow_version, manifest.compatibility.flow_version_requirement
                ));
            }
            Err(error) => {
                compatible = false;
                reasons.push(format!("Invalid Flow version requirement: {error}."));
            }
        }
        for family in &manifest.compatibility.contract_families {
            match contract_family_major(family) {
                Some(1) if is_known_contract_family(family) => {}
                Some(major) => {
                    compatible = false;
                    reasons.push(format!(
                        "Unsupported or unknown contract family {family} (major {major})."
                    ));
                }
                None => {
                    compatible = false;
                    reasons.push(format!("Malformed contract family {family}."));
                }
            }
        }
        for required_family in [
            "flow.extension-invocation/v1",
            "flow.extension-event/v1",
            "flow.extension-result/v1",
        ] {
            if !manifest
                .compatibility
                .contract_families
                .iter()
                .any(|family| family == required_family)
            {
                compatible = false;
                reasons.push(format!(
                    "Missing required runtime contract family {required_family}."
                ));
            }
        }

        let capability = manifest.capability(&request.capability_id);
        if let Some(capability) = capability {
            if capability.domain != request.domain {
                compatible = false;
                reasons.push(format!(
                    "Capability domain {} does not match requested domain {}.",
                    capability.domain.as_str(),
                    request.domain.as_str()
                ));
            }
        } else {
            compatible = false;
            reasons.push("The manifest does not declare the requested capability.".to_owned());
        }
        let execution_mode =
            manifest.execution_mode(&request.execution_mode_name, request.execution_mode_kind);
        if execution_mode.is_none() {
            compatible = false;
            reasons.push("The requested execution mode name and kind are not declared.".to_owned());
        }

        if let Some(entry) = locked {
            if entry.version != manifest.version {
                compatible = false;
                authorized = false;
                reasons.push("Manifest and lock versions do not agree.".to_owned());
            }
            if entry.publisher_id != manifest.publisher.id {
                compatible = false;
                authorized = false;
                reasons.push("Manifest and lock publishers do not agree.".to_owned());
            }
            if entry.integrity != manifest.integrity {
                compatible = false;
                authorized = false;
                available = false;
                reasons.push("Manifest and lock integrity values do not agree.".to_owned());
            }
            if !entry.enabled {
                authorized = false;
                available = false;
                reasons.push("The operator lock disables this extension.".to_owned());
            }
            match entry.trust {
                Trust::Trusted => {}
                Trust::Sandboxed => {
                    authorized = false;
                    available = false;
                    reasons.push(
                        "Sandboxed trust requires an enforceable backend that this checkpoint does not provide."
                            .to_owned(),
                    );
                }
                Trust::Disabled => {
                    authorized = false;
                    available = false;
                    reasons.push("The operator trust mode disables this extension.".to_owned());
                }
            }
            if !entry
                .granted_permissions
                .allows(&manifest.requested_permissions)
            {
                authorized = false;
                reasons
                    .push("Requested permissions exceed the explicit operator grants.".to_owned());
            }
            if let Some(capability) = capability {
                if capability
                    .effects
                    .iter()
                    .any(|effect| !entry.granted_permissions.allows_effect(*effect))
                {
                    authorized = false;
                    reasons
                        .push("Capability effects exceed the explicit operator grants.".to_owned());
                }
            }
        } else {
            authorized = false;
            available = false;
            reasons.push("No operator lock entry authorizes this extension.".to_owned());
        }

        match policy {
            Some(policy) if policy.ordered_extensions.contains(&manifest.extension_id) => {}
            Some(_) => {
                authorized = false;
                reasons.push(
                    "The operator capability policy does not list this extension.".to_owned(),
                );
            }
            None => {
                authorized = false;
                reasons.push("No operator capability policy exists for this request.".to_owned());
            }
        }

        if let Some(observation) = observation {
            if observation.integrity != manifest.integrity
                || locked.is_some_and(|entry| observation.integrity != entry.integrity)
            {
                available = false;
                reasons.push(
                    "Observed integrity does not match the manifest and operator lock.".to_owned(),
                );
            }
            if !observation.available {
                available = false;
                reasons.push(
                    "The caller reports the extension unavailable before invocation.".to_owned(),
                );
            }
        } else {
            available = false;
            reasons.push(
                "No caller-supplied availability and integrity observation matches this extension."
                    .to_owned(),
            );
        }

        let precedence = locked.map_or(0, |entry| entry.precedence);
        AssessedCandidate {
            manifest,
            locked,
            capability,
            execution_mode,
            evidence: ResolutionCandidate {
                extension_id: manifest.extension_id.clone(),
                version: manifest.version.clone(),
                publisher_id: manifest.publisher.id.clone(),
                integrity: manifest.integrity.value.clone(),
                precedence,
                compatible,
                authorized,
                available,
                reasons,
            },
        }
    }
}

#[derive(Clone, Debug)]
struct AssessedCandidate<'a> {
    manifest: &'a ExtensionManifest,
    locked: Option<&'a LockedExtension>,
    capability: Option<&'a Capability>,
    execution_mode: Option<&'a ExecutionMode>,
    evidence: ResolutionCandidate,
}

impl AssessedCandidate<'_> {
    fn eligible(&self) -> bool {
        self.evidence.compatible && self.evidence.authorized && self.evidence.available
    }
}

/// A successful decision token. It cannot be deserialized or constructed by callers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedExtension {
    manifest: ExtensionManifest,
    locked: LockedExtension,
    capability: Capability,
    execution_mode: ExecutionMode,
    lock_id: String,
    fallback_policy: FallbackPolicy,
}

impl ResolvedExtension {
    #[must_use]
    pub fn extension_id(&self) -> &str {
        &self.manifest.extension_id
    }

    #[must_use]
    pub fn version(&self) -> &str {
        &self.manifest.version
    }

    #[must_use]
    pub fn publisher_id(&self) -> &str {
        &self.manifest.publisher.id
    }

    #[must_use]
    pub fn integrity(&self) -> &Integrity {
        &self.manifest.integrity
    }

    #[must_use]
    pub fn capability(&self) -> &Capability {
        &self.capability
    }

    #[must_use]
    pub fn execution_mode(&self) -> &ExecutionMode {
        &self.execution_mode
    }

    #[must_use]
    pub fn lock_id(&self) -> &str {
        &self.lock_id
    }

    #[must_use]
    pub const fn fallback_policy(&self) -> FallbackPolicy {
        self.fallback_policy
    }

    #[must_use]
    pub fn granted_permissions(&self) -> &crate::contracts::Permissions {
        &self.locked.granted_permissions
    }
}

/// Versioned resolution evidence plus an opaque successful decision, if any.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionOutcome {
    evidence: ExtensionResolution,
    resolved: Option<ResolvedExtension>,
}

impl ResolutionOutcome {
    fn selected(evidence: ExtensionResolution, resolved: ResolvedExtension) -> Self {
        debug_assert!(evidence.validate().is_ok());
        Self {
            evidence,
            resolved: Some(resolved),
        }
    }

    fn rejected(evidence: ExtensionResolution) -> Self {
        debug_assert!(evidence.validate().is_ok());
        Self {
            evidence,
            resolved: None,
        }
    }

    #[must_use]
    pub fn evidence(&self) -> &ExtensionResolution {
        &self.evidence
    }

    #[must_use]
    pub fn resolved(&self) -> Option<&ResolvedExtension> {
        self.resolved.as_ref()
    }

    #[must_use]
    pub fn into_resolved(self) -> Option<ResolvedExtension> {
        self.resolved
    }
}

fn manifest_identity(manifest: &ExtensionManifest) -> IdentityKey<'_> {
    IdentityKey {
        extension_id: &manifest.extension_id,
        version: &manifest.version,
        publisher_id: &manifest.publisher.id,
    }
}

fn manifest_sort_key(manifest: &ExtensionManifest) -> (&str, &str, &str, &str) {
    (
        &manifest.extension_id,
        &manifest.version,
        &manifest.publisher.id,
        &manifest.integrity.value,
    )
}

fn assessment_sort_key<'a>(
    candidate: &'a AssessedCandidate<'_>,
) -> (Reverse<u64>, &'a str, &'a str, &'a str, &'a str) {
    (
        Reverse(candidate.evidence.precedence),
        &candidate.evidence.extension_id,
        &candidate.evidence.version,
        &candidate.evidence.publisher_id,
        &candidate.evidence.integrity,
    )
}

fn display_identity(extension_id: &str, version: &str, publisher_id: &str) -> String {
    format!("{extension_id}@{version} by {publisher_id}")
}

fn fallback_order(
    policy: Option<&crate::contracts::CapabilityResolution>,
    assessed: &[AssessedCandidate<'_>],
    selected_extension_id: &str,
) -> Vec<String> {
    let Some(policy) = policy else {
        return Vec::new();
    };
    let by_id: BTreeMap<&str, &AssessedCandidate<'_>> = assessed
        .iter()
        .map(|candidate| (candidate.manifest.extension_id.as_str(), candidate))
        .collect();
    let mut ids = Vec::new();
    let mut seen = HashSet::new();
    for extension_id in &policy.ordered_extensions {
        if let Some(candidate) = by_id.get(extension_id.as_str()) {
            if candidate.evidence.compatible
                && candidate.evidence.authorized
                && seen.insert(candidate.manifest.extension_id.as_str())
            {
                ids.push(candidate.manifest.extension_id.clone());
            }
        }
        if extension_id == selected_extension_id {
            break;
        }
    }
    ids
}

fn fallback_precedence(
    policy: Option<&crate::contracts::CapabilityResolution>,
    assessed: &[AssessedCandidate<'_>],
) -> Option<u64> {
    let policy = policy?;
    let highest_available = assessed
        .iter()
        .filter(|candidate| candidate.eligible())
        .map(|candidate| candidate.evidence.precedence)
        .max()?;
    let leader_ids: HashSet<&str> = assessed
        .iter()
        .filter(|candidate| {
            candidate.eligible() && candidate.evidence.precedence == highest_available
        })
        .map(|candidate| candidate.evidence.extension_id.as_str())
        .collect();
    let by_id: BTreeMap<&str, &AssessedCandidate<'_>> = assessed
        .iter()
        .map(|candidate| (candidate.manifest.extension_id.as_str(), candidate))
        .collect();
    let first_leader = policy
        .ordered_extensions
        .iter()
        .position(|extension_id| leader_ids.contains(extension_id.as_str()))?;
    policy.ordered_extensions[..first_leader]
        .iter()
        .filter_map(|extension_id| by_id.get(extension_id.as_str()).copied())
        .filter(|candidate| {
            candidate.evidence.compatible
                && candidate.evidence.authorized
                && !candidate.evidence.available
                && candidate.evidence.precedence >= highest_available
        })
        .map(|candidate| candidate.evidence.precedence)
        .max()
}

fn is_known_contract_family(family: &str) -> bool {
    matches!(
        family,
        "flow.artifact/v1"
            | "flow.capability/v1"
            | "flow.compatibility/v1"
            | "flow.extension-manifest/v1"
            | "flow.extension-lock/v1"
            | "flow.extension-invocation/v1"
            | "flow.extension-event/v1"
            | "flow.extension-result/v1"
            | "flow.extension-resolution/v1"
    )
}
