#!/usr/bin/env python3
"""Generate or check the tiny deterministic inputs used by scenario manifests."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
GENERATOR_ID = "org.egohygiene.fixture-generator"
GENERATOR_VERSION = "1.0.0"


@dataclass(frozen=True)
class Recipe:
    seed: str
    parameters: dict[str, Any]

    def parameter_bytes(self) -> bytes:
        return json.dumps(
            self.parameters,
            ensure_ascii=False,
            separators=(",", ":"),
            sort_keys=True,
        ).encode("utf-8")

    def parameters_digest(self) -> str:
        return hashlib.sha256(self.parameter_bytes()).hexdigest()

    def source_bytes(self) -> bytes:
        if set(self.parameters) == {"encoding", "text"}:
            assert self.parameters["encoding"] == "utf-8"
            return str(self.parameters["text"]).encode("utf-8")
        return self.parameter_bytes() + b"\n"


RECIPES = {
    "contracts/fixtures/scenarios/sources/source-text.txt": Recipe(
        seed="single-provider-success-v1",
        parameters={"encoding": "utf-8", "text": "Flow scenario fixture.\n"},
    ),
    "contracts/fixtures/scenarios/sources/synthetic-tone.json": Recipe(
        seed="interrupted-after-output-v1",
        parameters={
            "duration_ms": 1000,
            "frequency_hz": 440,
            "kind": "synthetic-tone",
        },
    ),
    "contracts/fixtures/scenarios/sources/synthetic-document.json": Recipe(
        seed="multi-provider-handoff-v1",
        parameters={
            "kind": "synthetic-document",
            "pages": [{"text": "Flow fixture"}],
        },
    ),
    "contracts/fixtures/scenarios/sources/empty-collection.json": Recipe(
        seed="observed-empty-v1",
        parameters={"items": []},
    ),
    "contracts/fixtures/scenarios/sources/synthetic-request.json": Recipe(
        seed="required-provider-unavailable-v1",
        parameters={"operation": "inspect"},
    ),
}

SCENARIOS = [
    ROOT / "contracts" / "examples" / "scenario-manifest.v1.example.json",
    ROOT / "contracts" / "fixtures" / "scenarios" / "interrupted.v1.fixture.json",
    ROOT / "contracts" / "fixtures" / "scenarios" / "multi-provider.v1.fixture.json",
    ROOT / "contracts" / "fixtures" / "scenarios" / "observed-empty.v1.fixture.json",
    ROOT / "contracts" / "fixtures" / "scenarios" / "unavailable.v1.fixture.json",
]


def check_provenance(errors: list[str]) -> None:
    observed: set[str] = set()
    for scenario_path in SCENARIOS:
        scenario = json.loads(scenario_path.read_text(encoding="utf-8"))
        for input_value in scenario["inputs"]:
            source = input_value["source"]
            locator = source["locator"]
            recipe = RECIPES.get(locator)
            if recipe is None:
                errors.append(f"{scenario_path.name}: unknown source recipe {locator}")
                continue
            observed.add(locator)
            generator = source.get("generator", {})
            expected = {
                "generator_id": GENERATOR_ID,
                "version": GENERATOR_VERSION,
                "seed": recipe.seed,
                "parameters_digest": recipe.parameters_digest(),
            }
            if generator != expected:
                errors.append(f"{scenario_path.name}: generator provenance drift for {locator}")
            digest = hashlib.sha256(recipe.source_bytes()).hexdigest()
            if source.get("digest") != digest or source.get("revision") != f"sha256:{digest}":
                errors.append(f"{scenario_path.name}: immutable source identity drift for {locator}")
    if observed != set(RECIPES):
        errors.append("scenario source recipes and positive manifests must cover each other exactly")


def run_check() -> list[str]:
    errors: list[str] = []
    check_provenance(errors)
    for relative, recipe in RECIPES.items():
        path = ROOT / relative
        if not path.is_file() or path.is_symlink():
            errors.append(f"{relative}: generated source is missing or not a regular file")
            continue
        if path.read_bytes() != recipe.source_bytes():
            errors.append(f"{relative}: generated source drift")
    return errors


def write_sources() -> None:
    for relative, recipe in RECIPES.items():
        path = ROOT / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(recipe.source_bytes())


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="check without writing")
    mode.add_argument("--write", action="store_true", help="rewrite generated source files")
    arguments = parser.parse_args()

    if arguments.write:
        write_sources()

    errors = run_check()
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print(f"Validated {len(RECIPES)} deterministic scenario source recipes.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
