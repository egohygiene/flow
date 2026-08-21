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


def main() -> int:
    errors: list[str] = []
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

    expected = {"flow.artifact/v1", "flow.capability/v1", "flow.compatibility/v1"}
    require(seen == expected, f"contract ids must be exactly {sorted(expected)}", errors)

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print(f"Validated {len(seen)} Flow contracts and examples ({manifest['contract_set_version']}, {manifest['status']}).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
