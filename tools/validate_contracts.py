#!/usr/bin/env python3
"""Validate the Flow contract manifest, schemas, and examples."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "contracts"
MANIFEST = CONTRACTS / "contract-set.v1.json"
SEMVER = re.compile(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")


def load_object(path: Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as handle:
        value = json.load(handle)
    if not isinstance(value, dict):
        raise ValueError(f"{path.relative_to(ROOT)} must contain a JSON object")
    return value


def require(condition: bool, message: str, errors: list[str]) -> None:
    if not condition:
        errors.append(message)


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


def validate_contract_semantics(
    contract_id: str,
    instance: dict[str, Any],
    location: str,
    errors: list[str],
) -> None:
    validators = {
        "flow.extension-manifest/v1": validate_extension_manifest,
        "flow.extension-lock/v1": validate_extension_lock,
        "flow.extension-result/v1": validate_extension_result,
        "flow.extension-resolution/v1": validate_extension_resolution,
    }
    validator = validators.get(contract_id)
    if validator is not None:
        validator(instance, location, errors)


def main() -> int:
    errors: list[str] = []
    positive_instances = 0
    rejected_invalid_instances = 0
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
        require(schema.get("$schema") == "https://json-schema.org/draft/2020-12/schema", f"{schema_path.name}: unsupported JSON Schema draft", errors)
        require(schema.get("type") == "object", f"{schema_path.name}: root type must be object", errors)
        require(schema.get("additionalProperties") is False, f"{schema_path.name}: root must reject unknown properties", errors)
        require(example.get("schema_version") == contract_id, f"{example_path.name}: schema_version must match {contract_id}", errors)
        validate_instance(example, schema, example_path.name, errors)
        validate_contract_semantics(contract_id, example, example_path.name, errors)
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
        "flow.artifact/v1",
        "flow.capability/v1",
        "flow.compatibility/v1",
        "flow.extension-event/v1",
        "flow.extension-invocation/v1",
        "flow.extension-lock/v1",
        "flow.extension-manifest/v1",
        "flow.extension-resolution/v1",
        "flow.extension-result/v1",
    }
    require(seen == expected, f"contract ids must be exactly {sorted(expected)}", errors)

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
