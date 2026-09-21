#!/usr/bin/env python3
"""Validate the Flow contract manifest, schemas, and examples."""

from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "contracts"
MANIFEST = CONTRACTS / "contract-set.v1.json"
SCENARIO_DIGESTS = CONTRACTS / "fixtures" / "scenarios" / "canonical-digests.v1.json"
SEMVER = re.compile(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")
CONTRACT_FAMILY = re.compile(r"^flow\.[a-z-]+/v([1-9][0-9]{0,19})$")
LOWER_SHA256 = re.compile(r"^[a-f0-9]{64}$")
U64_MAX = 18_446_744_073_709_551_615


def load_object(path: Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as handle:
        value = json.load(handle)
    if not isinstance(value, dict):
        raise ValueError(f"{path.relative_to(ROOT)} must contain a JSON object")
    return value


def require(condition: bool, message: str, errors: list[str]) -> None:
    if not condition:
        errors.append(message)


def is_strict_semver(value: Any) -> bool:
    if not isinstance(value, str) or SEMVER.fullmatch(value) is None:
        return False
    return all(int(component) <= U64_MAX for component in value.split("."))


def is_contract_family(value: Any) -> bool:
    if not isinstance(value, str):
        return False
    match = CONTRACT_FAMILY.fullmatch(value)
    return match is not None and int(match.group(1)) <= U64_MAX


def is_portable_locator(value: Any) -> bool:
    if not isinstance(value, str) or not value:
        return False
    if value.startswith("/") or "\\" in value or ":" in value:
        return False
    return all(is_portable_segment(segment) for segment in value.split("/"))


def is_portable_segment(segment: str) -> bool:
    if segment in {"", ".", ".."} or segment.endswith((" ", ".")):
        return False
    if any(ord(character) < 32 or ord(character) == 127 for character in segment):
        return False
    if any(character in '"*<>?|' for character in segment):
        return False
    stem = segment.split(".", maxsplit=1)[0].upper()
    if stem in {"CON", "PRN", "AUX", "NUL"}:
        return False
    return not (
        len(stem) == 4
        and stem[:3] in {"COM", "LPT"}
        and stem[3] in "123456789"
    )


def parent_locator(locator: str) -> str | None:
    return locator.rsplit("/", maxsplit=1)[0] if "/" in locator else None


def directory_manifest_identity(
    directory: str,
    entries: dict[str, dict[str, Any]],
) -> tuple[str, int] | None:
    children: list[dict[str, Any]] = []
    size_bytes = 0
    for locator, entry in entries.items():
        if (parent_locator(locator) or "") != directory:
            continue
        kind = entry.get("kind")
        digest = entry.get("digest")
        entry_size = entry.get("size_bytes")
        if (
            kind not in {"file", "directory"}
            or not isinstance(digest, str)
            or not isinstance(entry_size, int)
            or isinstance(entry_size, bool)
            or entry_size < 0
            or entry_size > U64_MAX
        ):
            return None
        size_bytes += entry_size
        if size_bytes > U64_MAX:
            return None
        children.append(
            {
                "name": locator.rsplit("/", maxsplit=1)[-1],
                "kind": kind,
                "digest": digest,
                "size_bytes": entry_size,
            }
        )
    children.sort(key=lambda child: child["name"])
    encoded = json.dumps(children, ensure_ascii=False, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest(), size_bytes


def validate_instance(
    instance: Any,
    schema: dict[str, Any],
    location: str,
    errors: list[str],
) -> None:
    """Validate the JSON Schema keywords used by the v1 contract set."""
    expected_type = schema.get("type")
    type_checks = {
        "object": lambda value: isinstance(value, dict),
        "array": lambda value: isinstance(value, list),
        "string": lambda value: isinstance(value, str),
        "integer": lambda value: isinstance(value, int) and not isinstance(value, bool),
        "boolean": lambda value: isinstance(value, bool),
    }
    if expected_type in type_checks and not type_checks[expected_type](instance):
        errors.append(f"{location}: expected {expected_type}")
        return

    if "const" in schema:
        require(instance == schema["const"], f"{location}: does not match const", errors)
    if "enum" in schema:
        require(instance in schema["enum"], f"{location}: not in enum", errors)
    if isinstance(instance, str):
        if "minLength" in schema:
            require(len(instance) >= schema["minLength"], f"{location}: string is too short", errors)
        if "pattern" in schema:
            require(bool(re.search(schema["pattern"], instance)), f"{location}: does not match pattern", errors)
    if isinstance(instance, int) and not isinstance(instance, bool) and "minimum" in schema:
        require(instance >= schema["minimum"], f"{location}: below minimum", errors)
    if isinstance(instance, int) and not isinstance(instance, bool) and "maximum" in schema:
        require(instance <= schema["maximum"], f"{location}: above maximum", errors)

    if isinstance(instance, dict):
        required = schema.get("required", [])
        for field in required if isinstance(required, list) else []:
            require(field in instance, f"{location}: missing required field {field}", errors)
        properties = schema.get("properties", {})
        if isinstance(properties, dict):
            if schema.get("additionalProperties") is False:
                unknown = sorted(set(instance) - set(properties))
                require(not unknown, f"{location}: unknown fields {unknown}", errors)
            for field, value in instance.items():
                child_schema = properties.get(field)
                if isinstance(child_schema, dict):
                    validate_instance(value, child_schema, f"{location}.{field}", errors)

    if isinstance(instance, list):
        if "minItems" in schema:
            require(len(instance) >= schema["minItems"], f"{location}: too few items", errors)
        if "maxItems" in schema:
            require(len(instance) <= schema["maxItems"], f"{location}: too many items", errors)
        if schema.get("uniqueItems"):
            normalized = [json.dumps(value, sort_keys=True) for value in instance]
            require(len(normalized) == len(set(normalized)), f"{location}: items are not unique", errors)
        item_schema = schema.get("items")
        if isinstance(item_schema, dict):
            for index, value in enumerate(instance):
                validate_instance(value, item_schema, f"{location}[{index}]", errors)


def validate_extension_manifest(
    manifest: dict[str, Any],
    location: str,
    errors: list[str],
) -> None:
    """Validate extension invariants that JSON Schema cannot express concisely."""
    capabilities = manifest.get("capabilities", [])
    capability_ids = [
        capability.get("capability_id")
        for capability in capabilities
        if isinstance(capability, dict)
        and isinstance(capability.get("capability_id"), str)
    ]
    require(
        len(capability_ids) == len(set(capability_ids)),
        f"{location}: capability identifiers must be unique",
        errors,
    )

    owned_domains = {
        domain
        for domain in manifest.get("domain_ownership", [])
        if isinstance(domain, str)
    }
    for capability in capabilities if isinstance(capabilities, list) else []:
        if not isinstance(capability, dict):
            continue
        require(
            capability.get("domain") in owned_domains,
            f"{location}: capability {capability.get('capability_id')} uses an undeclared domain",
            errors,
        )

    modes = manifest.get("execution_modes", [])
    mode_names = [
        mode.get("name")
        for mode in modes
        if isinstance(mode, dict)
        and isinstance(mode.get("name"), str)
    ]
    require(
        len(mode_names) == len(set(mode_names)),
        f"{location}: execution mode names must be unique",
        errors,
    )

    capability_by_id = {
        capability.get("capability_id"): capability
        for capability in capabilities
        if isinstance(capability, dict)
        and isinstance(capability.get("capability_id"), str)
    }
    inspection = manifest.get("inspection", {})
    if isinstance(inspection, dict):
        health_mode = inspection.get("health_mode")
        health_capability_ids = inspection.get("health_capability_ids", [])
        if health_mode == "manifest-only":
            require(
                health_capability_ids == [],
                f"{location}: manifest-only health cannot invoke capabilities",
                errors,
            )
        elif health_mode == "invocation":
            require(
                isinstance(health_capability_ids, list)
                and bool(health_capability_ids)
                and all(
                    capability_id in capability_by_id
                    for capability_id in health_capability_ids
                ),
                f"{location}: invocation health must reference declared capabilities",
                errors,
            )
    hook_ids: list[Any] = []
    for collection_name in ("observer_hooks", "transform_hooks"):
        hooks = manifest.get(collection_name, [])
        hook_ids.extend(
            hook.get("hook_id")
            for hook in hooks
            if isinstance(hook, dict)
            and isinstance(hook.get("hook_id"), str)
        )
    require(
        len(hook_ids) == len(set(hook_ids)),
        f"{location}: hook identifiers must be unique",
        errors,
    )

    transform_hooks = manifest.get("transform_hooks", [])
    for hook in transform_hooks if isinstance(transform_hooks, list) else []:
        if not isinstance(hook, dict):
            continue
        capability = capability_by_id.get(hook.get("capability_id"))
        require(
            isinstance(capability, dict),
            f"{location}: transform hook references an unknown capability",
            errors,
        )
        if isinstance(capability, dict):
            require(
                capability.get("content_changes") is True,
                f"{location}: transform hook capability must declare content changes",
                errors,
            )
            require(
                hook.get("accepts") == capability.get("accepts")
                and hook.get("produces") == capability.get("produces"),
                f"{location}: transform hook artifact types must match its capability",
                errors,
            )

    requested = manifest.get("requested_permissions", {})
    effect_requirements = {
        "filesystem-read": "filesystem_read",
        "filesystem-write": "filesystem_write",
        "environment-read": "environment_read",
        "subprocess": "subprocesses",
        "network": "network_hosts",
        "ai": "ai_providers",
        "gpu": "gpu",
        "source-mutation": "source_mutation",
        "destructive": "destructive",
        "sign": "sign",
        "publish": "publish",
    }
    for capability in capabilities if isinstance(capabilities, list) else []:
        if not isinstance(capability, dict):
            continue
        for effect in capability.get("effects", []):
            permission = effect_requirements.get(effect)
            if permission is None or not isinstance(requested, dict):
                continue
            value = requested.get(permission)
            require(
                value is True or (isinstance(value, list) and bool(value)),
                f"{location}: effect {effect} has no matching permission request",
                errors,
            )


def validate_extension_lock(
    lock: dict[str, Any],
    location: str,
    errors: list[str],
) -> None:
    extensions = lock.get("extensions", [])
    extension_ids = [
        extension.get("extension_id")
        for extension in extensions
        if isinstance(extension, dict)
        and isinstance(extension.get("extension_id"), str)
    ]
    require(
        len(extension_ids) == len(set(extension_ids)),
        f"{location}: locked extension identifiers must be unique",
        errors,
    )
    known = set(extension_ids)
    resolutions = lock.get("capability_resolution", [])
    for resolution in resolutions if isinstance(resolutions, list) else []:
        if not isinstance(resolution, dict):
            continue
        for extension_id in resolution.get("ordered_extensions", []):
            require(
                extension_id in known,
                f"{location}: capability resolution references unlocked extension {extension_id}",
                errors,
            )


def validate_extension_result(
    result: dict[str, Any],
    location: str,
    errors: list[str],
) -> None:
    failure = result.get("failure", {})
    classification = failure.get("classification") if isinstance(failure, dict) else None
    if result.get("outcome") in {"produced", "reused"}:
        require(
            classification == "none",
            f"{location}: successful outcomes cannot carry a failure classification",
            errors,
        )
    if result.get("outcome") == "failed":
        require(
            classification not in {None, "none"},
            f"{location}: failed outcomes require a failure classification",
            errors,
        )


def validate_artifact_bindings(
    bindings: dict[str, Any],
    location: str,
    errors: list[str],
) -> None:
    inputs = [
        value
        for value in bindings.get("inputs", [])
        if isinstance(value, dict)
    ]
    outputs = [
        value
        for value in bindings.get("outputs", [])
        if isinstance(value, dict)
    ]
    all_bindings = [*inputs, *outputs]
    require(
        bool(all_bindings),
        f"{location}: bindings must not be empty",
        errors,
    )
    for field in ("artifact_id", "port", "locator"):
        values = [binding.get(field) for binding in all_bindings]
        require(
            all(isinstance(value, str) for value in values)
            and len(values) == len(set(values)),
            f"{location}: {field} values must be unique",
            errors,
        )
    for binding in all_bindings:
        require(
            is_portable_locator(binding.get("locator")),
            f"{location}: binding locators must be portable and root-relative",
            errors,
        )


def validate_artifact_observations(
    observations: dict[str, Any],
    location: str,
    errors: list[str],
) -> None:
    artifacts = [
        value
        for value in observations.get("artifacts", [])
        if isinstance(value, dict)
    ]
    require(
        bool(artifacts),
        f"{location}: observations must contain at least one artifact",
        errors,
    )
    for field in ("artifact_id", "port", "locator"):
        values = [artifact.get(field) for artifact in artifacts]
        require(
            all(isinstance(value, str) for value in values)
            and len(values) == len(set(values)),
            f"{location}: observed {field} values must be unique",
            errors,
        )
    for artifact in artifacts:
        require(
            is_portable_locator(artifact.get("locator")),
            f"{location}: observation locators must be portable and root-relative",
            errors,
        )
        manifest = artifact.get("manifest", [])
        if artifact.get("kind") == "file":
            require(
                manifest == [],
                f"{location}: file observations cannot contain directory entries",
                errors,
            )
        if not isinstance(manifest, list):
            continue
        locators = [
            entry.get("locator")
            for entry in manifest
            if isinstance(entry, dict)
        ]
        require(
            all(is_portable_locator(locator) for locator in locators),
            f"{location}: manifest locators must be portable and root-relative",
            errors,
        )
        require(
            locators == sorted(set(locators)),
            f"{location}: manifest locators must be unique and sorted",
            errors,
        )
        if artifact.get("kind") != "directory" or len(locators) != len(manifest):
            continue
        entries = {
            entry["locator"]: entry
            for entry in manifest
            if isinstance(entry, dict) and isinstance(entry.get("locator"), str)
        }
        for locator, entry in entries.items():
            parent = parent_locator(locator)
            if parent is not None:
                require(
                    parent in entries and entries[parent].get("kind") == "directory",
                    f"{location}: nested manifest entries require a directory parent",
                    errors,
                )
            if entry.get("kind") == "directory":
                identity = directory_manifest_identity(locator, entries)
                require(
                    identity is not None
                    and identity == (entry.get("digest"), entry.get("size_bytes")),
                    f"{location}: directory entry identity conflicts with its children",
                    errors,
                )
        identity = directory_manifest_identity("", entries)
        require(
            identity is not None
            and identity == (artifact.get("digest"), artifact.get("size_bytes")),
            f"{location}: directory observation identity conflicts with its manifest",
            errors,
        )


def validate_execution_subject_lock(
    lock: dict[str, Any],
    location: str,
    errors: list[str],
) -> None:
    extension = lock.get("extension", {})
    package = lock.get("package", {})
    executable = lock.get("executable", {})
    if isinstance(extension, dict):
        require(
            is_strict_semver(extension.get("version")),
            f"{location}: extension version must use strict semantic versioning",
            errors,
        )
    if not isinstance(package, dict) or not isinstance(executable, dict):
        return
    require(
        package.get("subject_id") != executable.get("subject_id"),
        f"{location}: package and executable subjects must be distinct",
        errors,
    )
    require(
        is_portable_locator(package.get("locator")),
        f"{location}: package locator must be portable and root-relative",
        errors,
    )
    require(
        is_portable_locator(executable.get("locator")),
        f"{location}: executable locator must be portable and package-relative",
        errors,
    )
    package_digest = package.get("digest", {})
    package_digest_value = (
        package_digest.get("value") if isinstance(package_digest, dict) else None
    )
    require(
        isinstance(extension, dict)
        and package_digest_value == extension.get("integrity"),
        f"{location}: package digest must equal extension integrity",
        errors,
    )
    package_locator = package.get("locator")
    executable_locator = executable.get("locator")
    if isinstance(package_locator, str) and isinstance(executable_locator, str):
        require(
            is_portable_locator(f"{package_locator}/{executable_locator}"),
            f"{location}: joined executable locator must remain portable",
            errors,
        )


def validate_execution_subject_observations(
    observations: dict[str, Any],
    location: str,
    errors: list[str],
) -> None:
    raw_subjects = observations.get("subjects", [])
    subjects = (
        [subject for subject in raw_subjects if isinstance(subject, dict)]
        if isinstance(raw_subjects, list)
        else []
    )
    require(
        len(subjects) == 2,
        f"{location}: exactly one package and one executable are required",
        errors,
    )
    if len(subjects) != 2:
        return
    package, executable = subjects
    require(
        package.get("role") == "package" and executable.get("role") == "executable",
        f"{location}: subjects must use canonical package-then-executable order",
        errors,
    )
    require(
        package.get("kind") == "directory" and executable.get("kind") == "file",
        f"{location}: package must be a directory and executable must be a file",
        errors,
    )
    require(
        isinstance(package.get("subject_id"), str)
        and package["subject_id"].startswith("package:")
        and isinstance(executable.get("subject_id"), str)
        and executable["subject_id"].startswith("executable:"),
        f"{location}: subject identifiers must match their roles",
        errors,
    )
    for field in ("subject_id", "locator"):
        values = [subject.get(field) for subject in subjects]
        require(
            all(isinstance(value, str) for value in values)
            and len(values) == len(set(values)),
            f"{location}: observed {field} values must be unique",
            errors,
        )

    package_locator = package.get("locator")
    executable_locator = executable.get("locator")
    package_digest = package.get("digest", {})
    extension = observations.get("extension", {})
    require(
        isinstance(package_digest, dict)
        and isinstance(extension, dict)
        and package_digest.get("value") == extension.get("integrity"),
        f"{location}: package digest must equal extension integrity",
        errors,
    )
    relative_executable: str | None = None
    if isinstance(package_locator, str) and isinstance(executable_locator, str):
        prefix = f"{package_locator}/"
        require(
            executable_locator.startswith(prefix),
            f"{location}: executable must be beneath the package locator",
            errors,
        )
        if executable_locator.startswith(prefix):
            relative_executable = executable_locator[len(prefix) :]

    artifacts: list[dict[str, Any]] = []
    for index, subject in enumerate(subjects):
        digest = subject.get("digest", {})
        artifacts.append(
            {
                "artifact_id": f"artifact:execution-subject-{index}",
                "port": f"port:execution-subject-{index}",
                "media_type": "application/vnd.flow.execution-subject",
                "kind": subject.get("kind"),
                "locator": subject.get("locator"),
                "digest": digest.get("value") if isinstance(digest, dict) else None,
                "size_bytes": subject.get("size_bytes"),
                "manifest": subject.get("manifest"),
            }
        )
    validate_artifact_observations({"artifacts": artifacts}, location, errors)

    package_manifest = package.get("manifest", [])
    executable_digest = executable.get("digest", {})
    matching_entries = (
        [
            entry
            for entry in package_manifest
            if isinstance(entry, dict) and entry.get("locator") == relative_executable
        ]
        if isinstance(package_manifest, list) and relative_executable is not None
        else []
    )
    require(
        len(matching_entries) == 1,
        f"{location}: package manifest must contain the executable exactly once",
        errors,
    )
    if len(matching_entries) == 1:
        entry = matching_entries[0]
        require(
            entry.get("kind") == "file"
            and isinstance(executable_digest, dict)
            and entry.get("digest") == executable_digest.get("value")
            and entry.get("size_bytes") == executable.get("size_bytes"),
            f"{location}: package and executable observations contradict",
            errors,
        )

    claims = observations.get("claims", {})
    if isinstance(claims, dict):
        publisher = claims.get("publisher_identity", {})
        require(
            isinstance(extension, dict)
            and isinstance(publisher, dict)
            and publisher.get("publisher_id") == extension.get("publisher_id"),
            f"{location}: publisher claim must correlate the extension declaration",
            errors,
        )
        for claim_name in ("cryptographic_verification", "transparency_log"):
            claim = claims.get(claim_name, {})
            require(
                isinstance(claim, dict) and claim.get("evidence") == [],
                f"{location}: v1 does not support {claim_name} evidence",
                errors,
            )


def validate_extension_resolution(
    resolution: dict[str, Any],
    location: str,
    errors: list[str],
) -> None:
    candidates = [
        candidate
        for candidate in resolution.get("candidates", [])
        if isinstance(candidate, dict)
    ]
    eligible = [
        candidate
        for candidate in candidates
        if candidate.get("compatible")
        and candidate.get("authorized")
        and candidate.get("available")
    ]
    selected = resolution.get("selected_extension_ids", [])
    result = resolution.get("result")

    if result == "selected":
        require(
            isinstance(selected, list) and len(selected) == 1,
            f"{location}: selected resolution must name exactly one extension",
            errors,
        )
        eligible_ids = {candidate.get("extension_id") for candidate in eligible}
        require(
            bool(selected) and selected[0] in eligible_ids,
            f"{location}: selected extension must be compatible, authorized, and available",
            errors,
        )
    else:
        require(
            selected == [],
            f"{location}: non-selected resolution cannot name a selected extension",
            errors,
        )

    if result == "no-compatible-provider":
        require(
            not any(candidate.get("compatible") for candidate in candidates),
            f"{location}: no-compatible-provider requires every candidate to be incompatible",
            errors,
        )
    elif result == "blocked":
        require(
            any(candidate.get("compatible") for candidate in candidates)
            and not eligible,
            f"{location}: blocked requires compatible but ineligible candidates",
            errors,
        )
    elif result == "conflict":
        eligible_precedence = [
            candidate.get("precedence")
            for candidate in eligible
            if isinstance(candidate.get("precedence"), int)
        ]
        highest = max(eligible_precedence) if eligible_precedence else None
        require(
            highest is not None
            and eligible_precedence.count(highest) > 1,
            f"{location}: conflict requires equal-precedence eligible candidates",
            errors,
        )
    elif result == "malformed":
        require(
            not candidates,
            f"{location}: malformed fixture must not expose executable candidates",
            errors,
        )


def validate_scenario_manifest(
    scenario: dict[str, Any],
    location: str,
    errors: list[str],
) -> None:
    """Validate scenario graph, evidence, and execution invariants."""
    require(
        is_strict_semver(scenario.get("fixture_version")),
        f"{location}: fixture_version exceeds strict semantic-version bounds",
        errors,
    )
    raw_inputs = scenario.get("inputs", [])
    raw_providers = scenario.get("providers", [])
    raw_stages = scenario.get("stages", [])
    inputs = [
        value
        for value in (raw_inputs if isinstance(raw_inputs, list) else [])
        if isinstance(value, dict)
    ]
    providers = [
        value
        for value in (raw_providers if isinstance(raw_providers, list) else [])
        if isinstance(value, dict)
    ]
    stages = [
        value
        for value in (raw_stages if isinstance(raw_stages, list) else [])
        if isinstance(value, dict)
    ]

    input_ids = [
        value.get("artifact_id")
        for value in inputs
        if isinstance(value.get("artifact_id"), str)
    ]
    provider_ids = [
        value.get("provider_id")
        for value in providers
        if isinstance(value.get("provider_id"), str)
    ]
    stage_ids = [
        value.get("stage_id")
        for value in stages
        if isinstance(value.get("stage_id"), str)
    ]
    require(
        len(input_ids) == len(set(input_ids)),
        f"{location}: input artifact identifiers must be unique",
        errors,
    )
    require(
        len(provider_ids) == len(set(provider_ids)),
        f"{location}: provider identifiers must be unique",
        errors,
    )
    require(
        len(stage_ids) == len(set(stage_ids)),
        f"{location}: stage identifiers must be unique",
        errors,
    )

    for owner, source in [
        *[(f"input {value.get('artifact_id')}", value.get("source")) for value in inputs],
        *[(f"provider {value.get('provider_id')}", value.get("package")) for value in providers],
    ]:
        if not isinstance(source, dict):
            continue
        kind = source.get("kind")
        generator = source.get("generator")
        require(
            (kind == "generated" and isinstance(generator, dict))
            or (
                isinstance(kind, str)
                and kind in {"vendored", "released"}
                and generator is None
            ),
            f"{location}: {owner} source kind and generator provenance disagree",
            errors,
        )
        if isinstance(generator, dict):
            require(
                is_strict_semver(generator.get("version")),
                f"{location}: {owner} generator version exceeds semantic-version bounds",
                errors,
            )

    for provider in providers:
        require(
            is_strict_semver(provider.get("version")),
            f"{location}: provider version exceeds strict semantic-version bounds",
            errors,
        )

    providers_by_id = {
        provider.get("provider_id"): provider
        for provider in providers
        if isinstance(provider.get("provider_id"), str)
    }
    known_stages: set[str] = set()
    ancestors: dict[str, set[str]] = {}
    available_artifacts: dict[str, str | None] = {
        artifact_id: None for artifact_id in input_ids if isinstance(artifact_id, str)
    }
    for index, stage in enumerate(stages):
        stage_id = stage.get("stage_id")
        provider_id = stage.get("provider_id")
        capability_id = stage.get("capability_id")
        provider = (
            providers_by_id.get(provider_id)
            if isinstance(provider_id, str)
            else None
        )
        require(
            provider is not None,
            f"{location}: stage {stage_id} references unknown provider {provider_id}",
            errors,
        )
        if isinstance(provider, dict):
            required_capabilities = provider.get("required_capabilities", [])
            require(
                isinstance(required_capabilities, list)
                and capability_id in required_capabilities,
                f"{location}: stage {stage_id} capability is not declared by its provider",
                errors,
            )

        stage_ancestors: set[str] = set()
        dependencies = stage.get("depends_on", [])
        for dependency in dependencies if isinstance(dependencies, list) else []:
            if not isinstance(dependency, str):
                continue
            require(
                dependency in known_stages,
                f"{location}: stage {stage_id} dependency {dependency} must be earlier",
                errors,
            )
            if dependency in known_stages:
                stage_ancestors.add(dependency)
                stage_ancestors.update(ancestors.get(dependency, set()))

        consumed = stage.get("consumes", [])
        for artifact_id in consumed if isinstance(consumed, list) else []:
            if not isinstance(artifact_id, str):
                continue
            require(
                artifact_id in available_artifacts,
                f"{location}: stage {stage_id} consumes unknown artifact {artifact_id}",
                errors,
            )
            producer = available_artifacts.get(artifact_id)
            require(
                producer is None or producer in stage_ancestors,
                f"{location}: stage {stage_id} must depend on producer {producer}",
                errors,
            )
        produced = stage.get("produces", [])
        for artifact_id in produced if isinstance(produced, list) else []:
            if not isinstance(artifact_id, str):
                continue
            require(
                artifact_id not in available_artifacts,
                f"{location}: artifact {artifact_id} is declared more than once",
                errors,
            )
            if isinstance(artifact_id, str):
                available_artifacts[artifact_id] = stage_id
        if isinstance(stage_id, str):
            known_stages.add(stage_id)
            ancestors[stage_id] = stage_ancestors

    expectation = scenario.get("expectation", {})
    if isinstance(expectation, dict):
        terminal_state = expectation.get("terminal_state")
        evidence_state = expectation.get("evidence_state")
        allowed_states = {
            "complete": {"complete", "observed-empty"},
            "partial": {"incomplete"},
            "unavailable": {"unavailable"},
            "unsupported": {"unsupported"},
            "invalid": {"invalid"},
            "failed": {"failed"},
            "interrupted": {"incomplete"},
        }
        require(
            evidence_state
            in (
                allowed_states.get(terminal_state, set())
                if isinstance(terminal_state, str)
                else set()
            ),
            f"{location}: terminal and evidence states are contradictory",
            errors,
        )
        raw_expected_artifacts = expectation.get("expected_artifacts", [])
        expected_artifacts = (
            raw_expected_artifacts
            if isinstance(raw_expected_artifacts, list)
            else []
        )
        require(
            all(
                isinstance(artifact_id, str) and artifact_id in available_artifacts
                for artifact_id in expected_artifacts
            ),
            f"{location}: expected artifacts must reference declared artifacts",
            errors,
        )
        require(
            evidence_state != "observed-empty" or expected_artifacts == [],
            f"{location}: observed-empty evidence cannot claim artifacts",
            errors,
        )
        for field in ("evidence_refs", "state_trace_refs"):
            raw_references = expectation.get(field, [])
            for reference in raw_references if isinstance(raw_references, list) else []:
                if isinstance(reference, dict):
                    require(
                        is_contract_family(reference.get("schema_version")),
                        f"{location}: {field} contains an invalid contract family",
                        errors,
                    )

    execution = scenario.get("execution", {})
    if isinstance(execution, dict):
        network_mode = execution.get("network_mode")
        external_services = execution.get("external_services", [])
        clean_room = execution.get("clean_room")
        require(
            network_mode != "denied" or external_services == [],
            f"{location}: network-denied execution cannot declare external services",
            errors,
        )
        require(
            network_mode != "allowlisted" or bool(external_services),
            f"{location}: allowlisted network execution must name services",
            errors,
        )
        require(
            not clean_room or (network_mode == "denied" and external_services == []),
            f"{location}: clean-room execution must deny network and services",
            errors,
        )
        require(
            execution.get("tier") != "pull-request" or clean_room is True,
            f"{location}: pull-request scenarios must be clean-room",
            errors,
        )

    coverage = scenario.get("coverage", {})
    if isinstance(coverage, dict):
        require(
            bool(coverage.get("covered_behaviors")),
            f"{location}: covered_behaviors must not be empty",
            errors,
        )


def normalize_scenario(scenario: dict[str, Any]) -> dict[str, Any]:
    """Normalize set-like arrays while preserving contract-significant stages."""
    normalized = json.loads(json.dumps(scenario))
    normalized["tags"] = sorted(normalized.get("tags", []))
    normalized["inputs"] = sorted(
        normalized.get("inputs", []), key=lambda value: value.get("artifact_id", "")
    )
    normalized["providers"] = sorted(
        normalized.get("providers", []), key=lambda value: value.get("provider_id", "")
    )
    for provider in normalized.get("providers", []):
        provider["required_capabilities"] = sorted(
            provider.get("required_capabilities", [])
        )
    for stage in normalized.get("stages", []):
        for field in ("depends_on", "consumes", "produces"):
            stage[field] = sorted(stage.get(field, []))
    expectation = normalized.get("expectation", {})
    for field in ("expected_artifacts", "expected_diagnostics"):
        expectation[field] = sorted(expectation.get(field, []))
    for field in ("evidence_refs", "state_trace_refs"):
        expectation[field] = sorted(
            expectation.get(field, []),
            key=lambda value: (value.get("schema_version", ""), value.get("digest", "")),
        )
    execution = normalized.get("execution", {})
    execution["external_services"] = sorted(execution.get("external_services", []))
    coverage = normalized.get("coverage", {})
    coverage["covered_behaviors"] = sorted(coverage.get("covered_behaviors", []))
    coverage["known_gaps"] = sorted(coverage.get("known_gaps", []))
    return normalized


def canonical_scenario_digest(scenario: dict[str, Any]) -> str:
    payload = json.dumps(
        normalize_scenario(scenario),
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def canonical_flow_json_digest(value: dict[str, Any]) -> str:
    payload = json.dumps(
        value,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def validate_execution_subject_examples(
    examples: dict[str, dict[str, Any]],
    errors: list[str],
) -> None:
    lock = examples.get("flow.execution-subject-lock/v1")
    observations = examples.get("flow.execution-subject-observations/v1")
    if not isinstance(lock, dict) or not isinstance(observations, dict):
        return

    for field in (
        "subject_lock_id",
        "extension_lock_id",
        "extension",
        "capability_id",
        "interface",
        "declared_entrypoint",
    ):
        require(
            observations.get(field) == lock.get(field),
            f"execution-subject examples disagree on {field}",
            errors,
        )

    claims = observations.get("claims", {})
    equality = claims.get("lock_equality", {}) if isinstance(claims, dict) else {}
    require(
        isinstance(equality, dict)
        and equality.get("subject_lock_digest") == canonical_flow_json_digest(lock),
        "execution-subject observation does not identify the canonical lock bytes",
        errors,
    )

    subjects = observations.get("subjects", [])
    package = lock.get("package", {})
    executable = lock.get("executable", {})
    if (
        not isinstance(subjects, list)
        or len(subjects) != 2
        or not isinstance(subjects[0], dict)
        or not isinstance(subjects[1], dict)
        or not isinstance(package, dict)
        or not isinstance(executable, dict)
    ):
        return
    require(
        all(
            subjects[0].get(field) == package.get(field)
            for field in ("subject_id", "kind", "locator", "digest")
        ),
        "execution-subject package example does not match its lock",
        errors,
    )
    expected_executable_locator = (
        f"{package.get('locator')}/{executable.get('locator')}"
    )
    require(
        subjects[1].get("subject_id") == executable.get("subject_id")
        and subjects[1].get("kind") == executable.get("kind")
        and subjects[1].get("locator") == expected_executable_locator
        and subjects[1].get("digest") == executable.get("digest"),
        "execution-subject executable example does not match its lock",
        errors,
    )


def validate_scenario_digests(
    scenario_paths: list[Path],
    errors: list[str],
) -> None:
    require(SCENARIO_DIGESTS.is_file(), "missing scenario canonical digest catalog", errors)
    if not SCENARIO_DIGESTS.is_file():
        return
    catalog = load_object(SCENARIO_DIGESTS)
    require(
        set(catalog) == {"schema_version", "canonicalization", "algorithm", "entries"},
        "scenario canonical digest catalog has unexpected fields",
        errors,
    )
    require(
        catalog.get("schema_version") == "flow.scenario-digests/v1",
        "scenario canonical digest catalog has the wrong schema version",
        errors,
    )
    require(
        catalog.get("canonicalization") == "flow.canonical-json/v1",
        "scenario canonical digest catalog has the wrong canonicalization",
        errors,
    )
    require(catalog.get("algorithm") == "sha256", "scenario digests must use sha256", errors)
    entries = catalog.get("entries", [])
    require(isinstance(entries, list), "scenario digest entries must be an array", errors)
    entry_values = entries if isinstance(entries, list) else []
    expected_paths = {str(path.relative_to(CONTRACTS)) for path in scenario_paths}
    entry_paths = [
        entry.get("path")
        for entry in entry_values
        if isinstance(entry, dict) and isinstance(entry.get("path"), str)
    ]
    actual_paths = set(entry_paths)
    require(
        len(entry_paths) == len(actual_paths),
        "scenario canonical digest catalog cannot contain duplicate paths",
        errors,
    )
    require(
        actual_paths == expected_paths,
        "scenario canonical digest catalog must cover every positive scenario exactly",
        errors,
    )
    for entry in entry_values:
        if not isinstance(entry, dict):
            errors.append("every scenario digest entry must be an object")
            continue
        require(
            set(entry) == {"path", "digest"},
            "scenario digest entry has unexpected fields",
            errors,
        )
        relative = entry.get("path")
        digest = entry.get("digest")
        require(
            isinstance(digest, str) and LOWER_SHA256.fullmatch(digest) is not None,
            "scenario digest must be 64 lowercase hexadecimal characters",
            errors,
        )
        if not isinstance(relative, str) or relative not in expected_paths:
            continue
        actual = canonical_scenario_digest(load_object(CONTRACTS / relative))
        require(
            digest == actual,
            f"{relative}: canonical digest drift (expected {digest}, actual {actual})",
            errors,
        )


def validate_contract_semantics(
    contract_id: str,
    instance: dict[str, Any],
    location: str,
    errors: list[str],
) -> None:
    validators = {
        "flow.artifact-bindings/v1": validate_artifact_bindings,
        "flow.artifact-observations/v1": validate_artifact_observations,
        "flow.execution-subject-lock/v1": validate_execution_subject_lock,
        "flow.execution-subject-observations/v1": validate_execution_subject_observations,
        "flow.extension-manifest/v1": validate_extension_manifest,
        "flow.extension-lock/v1": validate_extension_lock,
        "flow.extension-result/v1": validate_extension_result,
        "flow.extension-resolution/v1": validate_extension_resolution,
        "flow.scenario-manifest/v1": validate_scenario_manifest,
    }
    validator = validators.get(contract_id)
    if validator is not None:
        validator(instance, location, errors)


def main() -> int:
    errors: list[str] = []
    positive_instances = 0
    rejected_invalid_instances = 0
    scenario_paths: list[Path] = []
    examples_by_id: dict[str, dict[str, Any]] = {}
    manifest = load_object(MANIFEST)
    require(manifest.get("schema_version") == "flow.contract-set/v1", "manifest schema_version must be flow.contract-set/v1", errors)
    require(bool(SEMVER.fullmatch(str(manifest.get("contract_set_version", "")))), "contract_set_version must be semantic versioning", errors)
    require(manifest.get("status") == "provisional", "initial contract set must remain explicitly provisional", errors)

    entries = manifest.get("contracts")
    require(isinstance(entries, list) and bool(entries), "manifest contracts must be a non-empty array", errors)
    seen: set[str] = set()
    for entry in entries if isinstance(entries, list) else []:
        if not isinstance(entry, dict):
            errors.append("every contract entry must be an object")
            continue
        contract_id = entry.get("id")
        require(isinstance(contract_id, str), "contract id must be a string", errors)
        if isinstance(contract_id, str):
            require(contract_id not in seen, f"duplicate contract id: {contract_id}", errors)
            seen.add(contract_id)

        schema_path = CONTRACTS / str(entry.get("schema", ""))
        example_path = CONTRACTS / str(entry.get("example", ""))
        require(schema_path.is_file(), f"missing schema: {schema_path.relative_to(ROOT)}", errors)
        require(example_path.is_file(), f"missing example: {example_path.relative_to(ROOT)}", errors)
        if not schema_path.is_file() or not example_path.is_file():
            continue

        schema = load_object(schema_path)
        example = load_object(example_path)
        if isinstance(contract_id, str):
            examples_by_id[contract_id] = example
        require(schema.get("$schema") == "https://json-schema.org/draft/2020-12/schema", f"{schema_path.name}: unsupported JSON Schema draft", errors)
        require(schema.get("type") == "object", f"{schema_path.name}: root type must be object", errors)
        require(schema.get("additionalProperties") is False, f"{schema_path.name}: root must reject unknown properties", errors)
        errors_before_instance = len(errors)
        require(example.get("schema_version") == contract_id, f"{example_path.name}: schema_version must match {contract_id}", errors)
        validate_instance(example, schema, example_path.name, errors)
        validate_contract_semantics(contract_id, example, example_path.name, errors)
        if (
            contract_id == "flow.scenario-manifest/v1"
            and len(errors) == errors_before_instance
        ):
            scenario_paths.append(example_path)
        positive_instances += 1

        fixtures = entry.get("fixtures", [])
        require(
            isinstance(fixtures, list),
            f"{contract_id}: fixtures must be an array",
            errors,
        )
        for fixture_name in fixtures if isinstance(fixtures, list) else []:
            fixture_path = CONTRACTS / str(fixture_name)
            require(
                fixture_path.is_file(),
                f"missing fixture: {fixture_path.relative_to(ROOT)}",
                errors,
            )
            if not fixture_path.is_file():
                continue
            fixture = load_object(fixture_path)
            errors_before_instance = len(errors)
            require(
                fixture.get("schema_version") == contract_id,
                f"{fixture_path.name}: schema_version must match {contract_id}",
                errors,
            )
            validate_instance(fixture, schema, fixture_path.name, errors)
            validate_contract_semantics(
                contract_id,
                fixture,
                fixture_path.name,
                errors,
            )
            if (
                contract_id == "flow.scenario-manifest/v1"
                and len(errors) == errors_before_instance
            ):
                scenario_paths.append(fixture_path)
            positive_instances += 1

        invalid_examples = entry.get("invalid_examples", [])
        require(
            isinstance(invalid_examples, list),
            f"{contract_id}: invalid_examples must be an array",
            errors,
        )
        for invalid_name in invalid_examples if isinstance(invalid_examples, list) else []:
            invalid_path = CONTRACTS / str(invalid_name)
            require(
                invalid_path.is_file(),
                f"missing invalid example: {invalid_path.relative_to(ROOT)}",
                errors,
            )
            if not invalid_path.is_file():
                continue
            invalid = load_object(invalid_path)
            invalid_errors: list[str] = []
            validate_instance(invalid, schema, invalid_path.name, invalid_errors)
            validate_contract_semantics(
                contract_id,
                invalid,
                invalid_path.name,
                invalid_errors,
            )
            require(
                bool(invalid_errors),
                f"{invalid_path.name}: expected schema or semantic validation to fail",
                errors,
            )
            if invalid_errors:
                rejected_invalid_instances += 1

    expected = {
        "flow.artifact-bindings/v1",
        "flow.artifact-observations/v1",
        "flow.artifact/v1",
        "flow.capability/v1",
        "flow.compatibility/v1",
        "flow.execution-subject-lock/v1",
        "flow.execution-subject-observations/v1",
        "flow.extension-event/v1",
        "flow.extension-invocation/v1",
        "flow.extension-lock/v1",
        "flow.extension-manifest/v1",
        "flow.extension-resolution/v1",
        "flow.extension-result/v1",
        "flow.scenario-manifest/v1",
    }
    require(seen == expected, f"contract ids must be exactly {sorted(expected)}", errors)
    validate_execution_subject_examples(examples_by_id, errors)
    validate_scenario_digests(scenario_paths, errors)

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    invalid_label = (
        "instance" if rejected_invalid_instances == 1 else "instances"
    )
    print(
        f"Validated {len(seen)} Flow contracts, {positive_instances} positive "
        f"instances, and {rejected_invalid_instances} rejected invalid "
        f"{invalid_label} "
        f"({manifest['contract_set_version']}, {manifest['status']})."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
