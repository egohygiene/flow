#!/usr/bin/env python3
"""Check local required validation stays joined to its existing hosted owners.

These observers inspect configuration and mutate test-owned copies. They never
launch a product, build, hosted workflow, or Docker container.
"""
import copy
import json
from pathlib import Path
import shlex
import unittest

import yaml

ROOT = Path(__file__).resolve().parents[1]
VALIDATORS = [
    "contract-schemas", "native-optiflow-receipts", "deterministic-scenario-sources",
    "acceptance-report-regressions", "lifecycle-report-regressions",
    "durable-contract-regressions", "architecture-specifications",
    "repository-skills", "repository-agents",
]
NEW_IDS = ["canonical-format", *VALIDATORS]
EXISTING_REQUIRED = ["declared-compiler-library", "workflow-budget-regressions",
                     "workflow-budget-source-plan", "workflow-budget-act"]


def required_selection(manifest, task, hosted):
    checks = manifest["checks"]
    ids = [row["id"] for row in checks]
    if len(ids) != len(set(ids)):
        raise ValueError("duplicate check ID")
    by_id = {row["id"]: row for row in checks}
    for identifier in [*NEW_IDS, *EXISTING_REQUIRED]:
        if identifier not in by_id or by_id[identifier]["required"] is not True or "skip_reason" in by_id[identifier]:
            raise ValueError("required validation missing, skipped or optional")
    expected_format = ["cargo", "+1.85.0", "fmt", "--all", "--", "--check"]
    formatting = by_id["canonical-format"]
    if formatting["command"] != expected_format or task["tasks"]["format:check"]["cmds"] != [shlex.join(expected_format)]:
        raise ValueError("canonical formatting differs from Task owner")
    rust_steps = hosted["jobs"]["rust"]["steps"]
    if next(step["run"] for step in rust_steps if step.get("name") == "Check canonical formatting") != shlex.join(expected_format):
        raise ValueError("canonical formatting differs from hosted owner")
    hosted_commands = [shlex.split(step["run"]) for step in hosted["jobs"]["repository-validators"]["steps"]
                       if "run" in step and step.get("name") != "Install validator dependencies"]
    if len(hosted_commands) != len(VALIDATORS):
        raise ValueError("hosted validator inventory differs")
    normalized = [[row[0], "-B", *row[1:]] for row in hosted_commands]
    if [by_id[identifier]["command"] for identifier in VALIDATORS] != normalized:
        raise ValueError("local validator command differs from hosted owner")
    if [shlex.split(command) for command in task["tasks"]["ci:validators"]["cmds"]] != normalized:
        raise ValueError("Task validator command differs from hosted owner")
    for identifier in NEW_IDS:
        row = by_id[identifier]
        if row["kind"] != "direct" or type(row["timeout_seconds"]) is not int or row["timeout_seconds"] != 120:
            raise ValueError("selected validation bound differs")
    hosted_clippy = next(step["run"] for step in rust_steps if step.get("name") == "Lint all targets")
    if shlex.split(hosted_clippy) != ["cargo", "clippy", "--all-targets", "--all-features", "--locked", "--", "-D", "warnings"]:
        raise ValueError("required hosted Clippy policy differs")


class SelectionBoundaries(unittest.TestCase):
    def setUp(self):
        self.manifest = json.loads((ROOT / "config/local-ci.json").read_bytes())
        self.task = yaml.safe_load((ROOT / "Taskfile.yml").read_bytes())
        self.hosted = yaml.safe_load((ROOT / ".github/workflows/ci.yml").read_bytes())

    def validate(self):
        required_selection(self.manifest, self.task, self.hosted)

    def test_exact_existing_owners_and_required_bounds(self):
        self.validate()

    def test_removed_optional_skipped_duplicate_and_unbounded_checks_refuse(self):
        original = copy.deepcopy(self.manifest)
        for identifier in NEW_IDS:
            for mutation in ["remove", "optional", "skip", "duplicate", "timeout", "boolean_timeout"]:
                self.manifest = copy.deepcopy(original)
                row = next(row for row in self.manifest["checks"] if row["id"] == identifier)
                if mutation == "remove": self.manifest["checks"].remove(row)
                elif mutation == "optional": row["required"] = False
                elif mutation == "skip": row["skip_reason"] = "silently omitted"
                elif mutation == "duplicate": self.manifest["checks"].append(copy.deepcopy(row))
                elif mutation == "timeout": row["timeout_seconds"] = 7200
                else: row["timeout_seconds"] = True
                with self.subTest(identifier=identifier, mutation=mutation), self.assertRaises(ValueError):
                    self.validate()

    def test_changed_commands_or_generated_source_rewrite_refuse(self):
        original = copy.deepcopy(self.manifest)
        for identifier in NEW_IDS:
            self.manifest = copy.deepcopy(original)
            row = next(row for row in self.manifest["checks"] if row["id"] == identifier)
            row["command"] = ["python3", "-c", "raise SystemExit(0)"]
            with self.subTest(identifier=identifier), self.assertRaises(ValueError): self.validate()
        self.manifest = copy.deepcopy(original)
        row = next(row for row in self.manifest["checks"] if row["id"] == "deterministic-scenario-sources")
        row["command"].remove("--check")
        with self.assertRaises(ValueError): self.validate()

    def test_task_drift_or_hosted_policy_weakening_refuses(self):
        self.task["tasks"]["ci:validators"]["cmds"].pop()
        with self.assertRaises(ValueError): self.validate()
        self.setUp()
        row = next(step for step in self.hosted["jobs"]["rust"]["steps"] if step.get("name") == "Lint all targets")
        row["run"] = row["run"].replace("-D warnings", "-A warnings")
        with self.assertRaises(ValueError): self.validate()


if __name__ == "__main__":
    unittest.main()
