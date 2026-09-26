#!/usr/bin/env python3
"""Regression checks for incomplete or misleading conformance reports."""

import copy
import json
import unittest

from run_acceptance_scenarios import CATALOG, MARKER, verify_receipts


class ReceiptCompletenessTests(unittest.TestCase):
    def setUp(self):
        self.catalog = json.loads(CATALOG.read_text())
        self.receipts = [
            {"schema_version": "flow.acceptance-scenario-receipt/v1",
             "scenario_id": case["scenario_id"], "outcome": case["expected"],
             "recovery": "Synthetic checker fixture only.",
             "fixture_identity": {"package_digest": "a" * 64}}
            for case in self.catalog["scenarios"]
        ]

    def encode(self, receipts):
        return "\n".join(MARKER + json.dumps(receipt) for receipt in receipts)

    def test_complete_out_of_order_receipts_are_sorted(self):
        result = verify_receipts(self.catalog, self.encode(reversed(self.receipts)))
        self.assertEqual([r["scenario_id"] for r in result],
                         sorted(r["scenario_id"] for r in self.receipts))

    def test_missing_duplicate_and_unknown_receipts_cannot_claim_completion(self):
        unknown = copy.deepcopy(self.receipts[0])
        unknown["scenario_id"] = "scenario:unknown"
        for receipts in [[], self.receipts[:-1], self.receipts + [self.receipts[0]],
                         self.receipts + [unknown]]:
            with self.subTest(count=len(receipts)), self.assertRaises(ValueError):
                verify_receipts(self.catalog, self.encode(receipts))

    def test_false_acceptance_unknown_version_and_missing_evidence_fail(self):
        for mutate in [
            lambda r: r["outcome"].update(accepted=True),
            lambda r: r.update(schema_version="flow.acceptance-scenario-receipt/v99"),
            lambda r: r.update(fixture_identity={}),
            lambda r: r.update(recovery=""),
        ]:
            receipts = copy.deepcopy(self.receipts)
            mutate(receipts[0])
            with self.assertRaises(ValueError):
                verify_receipts(self.catalog, self.encode(receipts))

    def test_duplicate_catalog_ids_and_missing_families_fail(self):
        for scenarios in [self.catalog["scenarios"] + [self.catalog["scenarios"][0]],
                          [c for c in self.catalog["scenarios"] if c["family"] != "privacy"]]:
            catalog = dict(self.catalog, scenarios=scenarios)
            with self.assertRaises(ValueError):
                verify_receipts(catalog, self.encode(self.receipts))

    def test_unbounded_receipt_is_rejected(self):
        self.receipts[0]["recovery"] = "x" * (self.catalog["budget"]["max_receipt_bytes"] + 1)
        with self.assertRaises(ValueError):
            verify_receipts(self.catalog, self.encode(self.receipts))


if __name__ == "__main__":
    unittest.main()
