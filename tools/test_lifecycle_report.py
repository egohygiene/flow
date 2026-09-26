#!/usr/bin/env python3
"""Exercise failure gates independently of the Rust matrix and enforce supervision."""
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import bounded_test_process as bounded
import lifecycle_reports as reports

ROOT = Path(__file__).resolve().parents[1]
CATALOG = json.loads((ROOT / "tests/fixtures/lifecycle-scenarios.v1.json").read_text())
SOURCE = {"files": {"contracts/fixtures/scenarios/sources/source-text.txt": hashlib.sha256(
    (ROOT / "contracts/fixtures/scenarios/sources/source-text.txt").read_bytes()).hexdigest()}}


def receipt(case):
    identity = dict.fromkeys(reports.IDENTITY_FIELDS, "a" * 64)
    identity["input_digest"] = next(iter(SOURCE["files"].values()))
    return dict(schema_version="flow.lifecycle-scenario-receipt/v1", scenario_id=case["scenario_id"],
                recipe_digest=reports.canonical_digest({"fixture_version": CATALOG["fixture_version"], "case": case}),
                fixture_identity=[identity] * (3 if case["graph"] else 1),
                outcome=deepcopy(case["expected"]), recovery=case["recovery"],
                plan_digest="b" * 64, history_digest="c" * 64,
                artifact_digests=[None] * (3 if case["graph"] else 1))


class ReceiptGate(unittest.TestCase):
    def setUp(self):
        self.receipts = [receipt(case) for case in CATALOG["scenarios"]]

    def verify(self, receipts=None, catalog=None, source=None):
        output = "\n".join(reports.MARKER + json.dumps(row) for row in
                           (self.receipts if receipts is None else receipts))
        return reports.verify_receipts(catalog or CATALOG, output, source or SOURCE)

    def test_complete_exact_inventory(self):
        self.assertEqual(len(self.verify()), len(CATALOG["scenarios"]))

    def test_missing_duplicate_unknown(self):
        for rows in [self.receipts[:-1], self.receipts + [self.receipts[0]],
                     [dict(self.receipts[0], scenario_id="scenario:unknown"), *self.receipts[1:]]]:
            with self.subTest(rows=len(rows)), self.assertRaises(ValueError):
                self.verify(rows)

    def test_transition_order_eligibility_and_recovery(self):
        mutations = [
            lambda row: row["outcome"]["history"].reverse(),
            lambda row: row["outcome"]["assessments"][-1]["steps"][0].update(eligibility="reusable"),
            lambda row: row.update(recovery="reuse-accepted-work"),
            lambda row: row["outcome"].update(code="succeeded"),
        ]
        for mutate in mutations:
            rows = deepcopy(self.receipts)
            mutate(rows[0])
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                self.verify(rows)

    def test_versions_identities_and_private_fields(self):
        mutations = [
            lambda row: row.update(schema_version="flow.lifecycle-scenario-receipt/v2"),
            lambda row: row.update(recipe_digest="d" * 64),
            lambda row: row.update(plan_digest=None),
            lambda row: row.update(history_digest="private path"),
            lambda row: row.update(raw_stderr="private provider text"),
            lambda row: row["fixture_identity"][0].update(input_digest="d" * 64),
            lambda row: row["fixture_identity"][0].update(executable_digest="d" * 64),
            lambda row: row["fixture_identity"][0].update(raw_configuration="private"),
            lambda row: row.update(artifact_digests=["private text"]),
        ]
        for mutate in mutations:
            rows = deepcopy(self.receipts)
            mutate(rows[0])
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                self.verify(rows)

    def test_catalog_drift_and_source_drift(self):
        changed = deepcopy(CATALOG)
        changed["scenarios"][0]["mode"] = "different-provider-behavior"
        with self.assertRaisesRegex(ValueError, "recipe identity drift"):
            self.verify(catalog=changed)
        with self.assertRaisesRegex(ValueError, "source identity drift"):
            self.verify(source={"files": dict.fromkeys(SOURCE["files"], "f" * 64)})
        for mutation in [{"fixture_version": "2.0.0"}, {"budget": {}}, {"known_gaps": []}]:
            with self.assertRaises(ValueError):
                self.verify(catalog=dict(CATALOG, **mutation))

    def test_oversized_receipt(self):
        self.receipts[0]["raw_stderr"] = "x" * reports.BUDGET["max_receipt_bytes"]
        with self.assertRaisesRegex(ValueError, "exceeds budget"):
            self.verify()

    def test_effect_authority_and_residual_evidence_cannot_be_changed_or_omitted(self):
        index = next(index for index, row in enumerate(self.receipts)
                     if row["scenario_id"] == "scenario:lifecycle-cleanup-failed")
        mutations = [
            lambda safety: safety["observations"][-1]["counters"][0].update(effects=0),
            lambda safety: safety["authority"][0].update(granted=False),
            lambda safety: safety.update(residual="none"),
            lambda safety: safety.update(refusals=[]),
            lambda safety: safety.update(raw_stderr="FLOW_CLEANUP_PRIVATE_CANARY"),
        ]
        for mutate in mutations:
            rows = deepcopy(self.receipts)
            mutate(rows[index]["outcome"]["safety"])
            with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                self.verify(rows)
        rows = deepcopy(self.receipts)
        del rows[index]["outcome"]["safety"]
        with self.assertRaises(ValueError):
            self.verify(rows)


@unittest.skipUnless(sys.platform == "linux", "Linux resource qualification")
class TestSupervision(unittest.TestCase):
    def test_combines_streams_without_shell(self):
        result = bounded.run([sys.executable, "-c", "import sys; print('out'); print('err', file=sys.stderr)"], cwd=ROOT)
        self.assertEqual(result.returncode, 0)
        self.assertEqual(set(result.stdout.splitlines()), {"out", "err"})

    def test_output_flood_is_stopped_during_capture(self):
        with self.assertRaises(bounded.BudgetExceeded) as caught:
            bounded.run([sys.executable, "-c", "import os\nwhile True: os.write(1, b'x' * 65536)"],
                        cwd=ROOT, timeout=5, max_output=8192)
        self.assertEqual(len(caught.exception.output), 8192)
        self.assertIn("output budget", str(caught.exception))

    def test_hung_command_is_killed_and_reaped(self):
        with self.assertRaisesRegex(bounded.BudgetExceeded, "runtime budget"):
            bounded.run([sys.executable, "-c", "while True: pass"], cwd=ROOT, timeout=0.1)

    def test_kernel_rejects_excess_address_space_and_file_size(self):
        program = """
import errno, mmap, resource
assert resource.getrlimit(resource.RLIMIT_AS) == (4294967296, 4294967296)
assert resource.getrlimit(resource.RLIMIT_FSIZE) == (16777216, 16777216)
try:
    mmap.mmap(-1, 4294967296)
except OSError as error:
    assert error.errno == errno.ENOMEM
else:
    raise AssertionError('memory limit did not apply')
with open('oversized-test-file', 'wb', buffering=0) as stream:
    stream.seek(16777216)
    try:
        stream.write(b'x')
    except OSError as error:
        assert error.errno == errno.EFBIG
    else:
        raise AssertionError('file limit did not apply')
"""
        with tempfile.TemporaryDirectory() as directory:
            result = bounded.run([sys.executable, str(ROOT / "tools/bounded_test_runner.py"),
                                  sys.executable, "-c", program], cwd=directory)
            self.assertEqual(result.returncode, 0, result.stdout)


if __name__ == "__main__":
    unittest.main()
