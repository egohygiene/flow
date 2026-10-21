#!/usr/bin/env python3
"""Check the shared receipt corpus with the independent Draft 2020-12 validator."""

import copy
import json
from pathlib import Path
import unittest

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests/fixtures/optiflow-receipt-parity.json"
SCHEMA = ROOT / "schemas/optiflow-read-only-receipt-v1.schema.json"


def apply_operations(document, operations):
    """Apply the fixture's bounded existing-key replace/remove operations."""
    for operation in operations:
        assert operation["op"] in {"replace", "remove"}
        expected_keys = {"op", "path", "value"} if operation["op"] == "replace" else {"op", "path"}
        assert set(operation) == expected_keys
        assert operation["path"].startswith("/")
        parts = [part.replace("~1", "/").replace("~0", "~") for part in operation["path"][1:].split("/")]
        parent = document
        for part in parts[:-1]:
            parent = parent[int(part)] if isinstance(parent, list) else parent[part]
        key = int(parts[-1]) if isinstance(parent, list) else parts[-1]
        if isinstance(parent, list):
            assert 0 <= key < len(parent)
        else:
            assert key in parent
        if operation["op"] == "replace":
            parent[key] = copy.deepcopy(operation["value"])
        else:
            del parent[key]
    return document


class OptiflowReceiptSchemaParity(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fixture = json.loads(FIXTURE.read_text(encoding="utf-8"))
        schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
        Draft202012Validator.check_schema(schema)
        cls.validator = Draft202012Validator(schema)

    def test_fixture_contract_and_positive_status_coverage(self):
        self.assertEqual(self.fixture["schema_version"], "flow.optiflow-receipt-parity-tests/v1")
        self.assertTrue(self.validator.is_valid(self.fixture["base"]))
        identities = [case["id"] for case in self.fixture["cases"]]
        self.assertEqual(len(identities), len(set(identities)))
        positive_statuses = set()
        refused_prefixes = set()
        for case in self.fixture["cases"]:
            self.assertIsInstance(case["schema_valid"], bool)
            self.assertIsInstance(case["runtime_valid"], bool)
            self.assertFalse(case["runtime_valid"] and not case["schema_valid"], case["id"])
            value = apply_operations(copy.deepcopy(self.fixture["base"]), case["operations"])
            if case["runtime_valid"]:
                positive_statuses.add(value["status"])
                if value["status"] == "refused":
                    refused_prefixes.add(len(value["commands"]))
        self.assertEqual(positive_statuses, {"scanning", "reporting", "planning", "complete", "partial", "refused"})
        self.assertEqual(refused_prefixes, {0, 1, 2, 3})

    def test_shared_receipt_cases(self):
        for case in self.fixture["cases"]:
            with self.subTest(case=case["id"]):
                value = apply_operations(copy.deepcopy(self.fixture["base"]), case["operations"])
                errors = list(self.validator.iter_errors(value))
                self.assertEqual(not errors, case["schema_valid"], "\n".join(error.message for error in errors))


if __name__ == "__main__":
    unittest.main()
