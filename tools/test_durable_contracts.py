#!/usr/bin/env python3
"""Regression checks for nested durable schemas and offline reference resolution."""

import copy
import unittest

from validate_contracts import CONTRACTS, load_object, validate_instance


class DurableContracts(unittest.TestCase):
    def validate(self, name, value):
        errors = []
        schema = load_object(CONTRACTS / "schemas" / f"{name}.v1.schema.json")
        validate_instance(value, schema, name, errors)
        return errors

    def test_nested_completed_example_and_null_pending_fields(self):
        for path in ["examples/run-state.v1.example.json", "fixtures/state/completed.v1.fixture.json"]:
            self.assertEqual(self.validate("run-state", load_object(CONTRACTS / path)), [])

    def test_nested_future_version_unknown_field_and_bad_optional_value(self):
        original = load_object(CONTRACTS / "fixtures/state/completed.v1.fixture.json")
        for mutation in ["version", "private", "optional"]:
            value = copy.deepcopy(original)
            if mutation == "version":
                value["steps"][0]["checkpoint"]["schema_version"] = "flow.run-checkpoint/v2"
            elif mutation == "private":
                value["plan"]["steps"][0]["context"]["configuration_values"] = {"secret": "canary"}
            else:
                value["steps"][0]["failure"] = {}
            self.assertTrue(self.validate("run-state", value), mutation)

    def test_recovery_requires_boolean_acknowledgement(self):
        value = load_object(CONTRACTS / "examples/run-recovery.v1.example.json")
        for acknowledgement in [False, 1, "true", None]:
            value["acknowledged_uncertain_effects"] = acknowledgement
            self.assertTrue(self.validate("run-recovery", value))

    def test_references_never_read_remote_or_escaping_paths(self):
        for reference in ["https://example.com/schema.json", "../outside.schema.json", "/tmp/schema.json", "missing.schema.json"]:
            errors = []
            validate_instance({}, {"$ref": reference}, "fixture", errors)
            self.assertTrue(errors, reference)

    def test_external_reference_preserves_closed_nested_objects(self):
        value = load_object(CONTRACTS / "examples/run-artifact.v1.example.json")
        value["observation"]["source_bytes"] = "private"
        self.assertTrue(self.validate("run-artifact", value))


if __name__ == "__main__":
    unittest.main()
