"""Independent negative controls for the local read-only pilot observers."""
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

SPEC = importlib.util.spec_from_file_location("read_only_pilot", Path(__file__).with_name("check_released_read_only.py"))
pilot = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(pilot)


def status():
    return {"schema_version": "flow.command-status/v1", "state": "complete", "operation_record_complete": True,
            "source_content_reobserved": False, "source_mutation_authority": False,
            "receipt": {"schema_version": "flow.optiflow-read-only-receipt/v1", "status": "complete", "recovery": "none",
                        "approval": None, "physical_savings_bytes": None,
                        "provider": {"version": "0.1.1", "source_revision": pilot.PROVIDER_SOURCE,
                                     "executable_sha256": pilot.PROVIDER_SHA, "archive_sha256": pilot.ARCHIVE_SHA,
                                     "target": pilot.TARGET},
                        "commands": [{"command": name, "outcome": "success", "coverage": "complete"}
                                     for name in ("scan", "report", "plan")],
                        "duplicate_logical_bytes": 21,
                        "environment": "cleared; no config; no media probe; no network grant",
                        "capabilities": {name: "unsupported-by-pinned-release" for name in ("dry-run", "quarantine", "restore", "finalize")}}}


class ObserverNegativeTests(unittest.TestCase):
    def build_observation(self):
        return {"schema_version": "flow.local-build-observation/v1",
                "binary": {"sha256": "a" * 64, "bytes": 55_666_864},
                "actual_build_revision": "b" * 40, "actual_build_tree": "c" * 40,
                "equivalent_remote_revision": "d" * 40, "profile": "dev", "build_command": list(pilot.BUILD_COMMAND)}

    def test_public_build_projection_drops_unknown_private_fields_recursively(self):
        value = self.build_observation()
        canary = "PRIVATE_BUILD_NOTE_PATH_TOKEN"
        value.update(private_notes=canary, credentials={"token": canary}, build_observed_by=canary)
        value["binary"]["private_path"] = canary
        projected = pilot.public_build_identity(value)
        self.assertNotIn(canary, json.dumps(projected))
        self.assertEqual(set(projected), {"schema_version", "binary", "actual_build_revision", "actual_build_tree",
                                         "equivalent_remote_revision", "profile", "build_command", "authority"})
        self.assertEqual(set(projected["binary"]), {"sha256", "bytes"})

    def test_public_build_identity_types_and_grammar_cannot_carry_private_text(self):
        for key, replacement in (("actual_build_revision", "/private/source"), ("actual_build_tree", "f" * 39),
                                 ("equivalent_remote_revision", 123), ("profile", "private profile"),
                                 ("build_command", ["cargo", "--config", "/private/config"])):
            value = self.build_observation(); value[key] = replacement
            with self.subTest(key=key), self.assertRaises(ValueError):
                pilot.public_build_identity(value)
        for key, replacement in (("sha256", "PRIVATE_TOKEN"), ("bytes", True), ("bytes", "55666864")):
            value = self.build_observation(); value["binary"][key] = replacement
            with self.subTest(binary_key=key), self.assertRaises(ValueError):
                pilot.public_build_identity(value)

    def test_same_exit_unrelated_refusal_cannot_satisfy_named_negative_control(self):
        root = Path("/synthetic/selected-run")
        value = {"schema_version": "flow.command-result/v1", "command": "plan", "outcome": "refused", "exit_code": 3,
                 "result": None, "diagnostics": [{"code": "invalid_input", "message": "run output must be canonical and disjoint from source/provider",
                                                    "recovery_directory": None}]}
        for key, replacement in (("code", "unavailable"), ("message", "selected document unavailable"),
                                 ("recovery_directory", pilot.native_path(root))):
            changed = copy.deepcopy(value); changed["diagnostics"][0][key] = replacement
            observed = pilot.envelope(json.dumps(changed).encode(), b"", 3, "plan", 3, "refused")
            with self.subTest(key=key), self.assertRaises(ValueError):
                pilot.case_diagnostics(observed, "overlap", root)
        changed = copy.deepcopy(value); changed["schema_version"] = "aniflow.command-result/v1"
        with self.assertRaisesRegex(ValueError, "domain"):
            pilot.case_diagnostics(changed, "overlap", root)
        changed = copy.deepcopy(value); changed["diagnostics"][0]["private_detail"] = "unexpected"
        with self.assertRaisesRegex(ValueError, "closed"):
            pilot.case_diagnostics(changed, "overlap", root)

    def test_each_named_refusal_requires_its_exact_reason_and_recovery_subject(self):
        root = Path("/synthetic/selected-run")
        for case, (code, message, recovery) in pilot.REFUSALS.items():
            value = {"schema_version": "flow.command-result/v1", "result": None,
                     "diagnostics": [{"code": code, "message": message,
                                      "recovery_directory": pilot.native_path(root) if recovery else None}]}
            pilot.case_diagnostics(value, case, root)
            value["diagnostics"][0]["message"] = "same exit, unrelated failure"
            with self.subTest(case=case), self.assertRaises(ValueError):
                pilot.case_diagnostics(value, case, root)

    def receipt_fixture(self, root):
        namespace = root / "flow-optiflow-v0.1.1"; namespace.mkdir()
        selected = status(); selected["receipt_path"] = pilot.native_path(namespace / "attempt.json")
        (namespace / "attempt.json").write_text(json.dumps(selected["receipt"]))
        return selected

    def test_final_receipt_locator_cannot_escape_or_read_an_unrelated_private_file(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve() / "run"; root.mkdir(); value = self.receipt_fixture(root)
            expected = copy.deepcopy(value["receipt"])
            self.assertEqual(pilot.retained_receipt(value, root, expected)["relative_path"], "flow-optiflow-v0.1.1/attempt.json")
            for path in (root.parent / "private.json", root / ".." / "private.json", root / "unrelated.json"):
                changed = copy.deepcopy(value); changed["receipt_path"] = pilot.native_path(path)
                with mock.patch.object(pilot.OWNER_IO, "regular", side_effect=AssertionError("outside receipt must not be read")):
                    with self.subTest(path=str(path)), self.assertRaisesRegex(ValueError, "escaped"):
                        pilot.retained_receipt(changed, root, expected)

    def test_final_receipt_file_must_decode_to_the_already_validated_receipt(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve(); value = self.receipt_fixture(root)
            expected = copy.deepcopy(value["receipt"])
            for claimed in (999, 21.0):
                wrong = copy.deepcopy(expected); wrong["duplicate_logical_bytes"] = claimed
                (root / "flow-optiflow-v0.1.1/attempt.json").write_text(json.dumps(wrong))
                with self.subTest(claimed=claimed), self.assertRaisesRegex(ValueError, "different native receipt"):
                    pilot.retained_receipt(value, root, expected)

    def test_multiple_documents_duplicate_keys_and_nonfinite_values_are_refused(self):
        for raw in (b'{} {}', b'{"status":1,"status":2}', b'{"status":NaN}', b'{"status":Infinity}'):
            with self.subTest(raw=raw), self.assertRaises((ValueError, json.JSONDecodeError)):
                pilot.document(raw)

    def test_machine_stream_and_exit_forgery_are_refused(self):
        value = {"schema_version": "flow.command-result/v1", "command": "status", "outcome": "complete",
                 "exit_code": 0, "result": status(), "diagnostics": []}
        trials = [(json.dumps(value).encode(), b"unexpected log", 0), (b'\x1b' + json.dumps(value).encode(), b"", 0),
                  (json.dumps(value).encode(), b"", 9)]
        for raw, stderr, code in trials:
            with self.subTest(stderr=stderr, code=code), self.assertRaises(ValueError):
                pilot.envelope(raw, stderr, code, "status", 0, "complete")
        for key, replacement in (("command", "run"), ("outcome", "partial"), ("exit_code", False), ("extra", "unknown")):
            changed = copy.deepcopy(value); changed[key] = replacement
            with self.subTest(key=key), self.assertRaises(ValueError):
                pilot.envelope(json.dumps(changed).encode(), b"", 0, "status", 0, "complete")

    def test_incomplete_or_mutating_status_cannot_pass_as_read_only_completion(self):
        for key, value in (("operation_record_complete", False), ("state", "recovery_required"),
                           ("source_mutation_authority", True), ("source_content_reobserved", True)):
            changed = status(); changed[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                pilot.completed_status(changed)

    def test_native_receipt_cannot_invent_savings_approval_or_finished_recovery(self):
        for key, value in (("physical_savings_bytes", 0), ("approval", {}), ("status", "partial"),
                           ("recovery", "required"), ("duplicate_logical_bytes", 0), ("environment", "inherited")):
            changed = status(); changed["receipt"][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                pilot.completed_status(changed)

    def test_wrong_release_or_unfinished_native_workload_is_refused(self):
        for key, value in (("version", "development"), ("source_revision", "0" * 40),
                           ("executable_sha256", "0" * 64), ("archive_sha256", "0" * 64), ("target", "native-macos")):
            changed = status(); changed["receipt"]["provider"][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                pilot.completed_status(changed)
        for alter in (lambda x: x["commands"].pop(), lambda x: x["commands"][0].update(coverage="partial"),
                      lambda x: x["commands"][2].update(outcome="failure"),
                      lambda x: x["capabilities"].update(quarantine="available")):
            changed = status(); alter(changed["receipt"])
            with self.assertRaises(ValueError):
                pilot.completed_status(changed)

    def artifacts(self, root):
        records = {}
        for name in ("effective_policy", "report", "run", "artifact_set", "plan", "plan_set"):
            path = root / (name + ".json"); data = (name + "-synthetic").encode(); path.write_bytes(data)
            records[name] = {"kind": name if name != "plan_set" else "artifact_set", "path": pilot.native_path(path),
                             "size_bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
        return {"commands": [{"command": name, "artifacts": [records[k] for k in keys]}
                             for name, keys in (("scan", ("effective_policy", "report", "run", "artifact_set")),
                                                ("report", ("effective_policy", "report", "run", "artifact_set")),
                                                ("plan", ("plan", "plan_set")))]}

    def test_actual_artifact_bytes_are_rehashed_independently(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve(); receipt = self.artifacts(root)
            self.assertEqual(len(pilot.native_artifacts(receipt, root)), 6)
            (root / "plan.json").write_bytes(b"changed; CLI report remains unchanged")
            with self.assertRaisesRegex(ValueError, "actual native artifact"):
                pilot.native_artifacts(receipt, root)

    def test_artifact_roles_and_fixture_confinement_cannot_be_forged(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve(); receipt = self.artifacts(root)
            receipt["commands"][0]["artifacts"][0]["kind"] = "plan"
            with self.assertRaisesRegex(ValueError, "roles"):
                pilot.native_artifacts(receipt, root)
            receipt = self.artifacts(root)
            receipt["commands"][0]["artifacts"][0]["path"] = pilot.native_path(root.parent / "outside.json")
            with self.assertRaisesRegex(ValueError, "escaped"):
                pilot.native_artifacts(receipt, root)

    def test_alias_source_inventory_is_refused(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve(); (root / "data").write_bytes(b"source")
            (root / "alias").symlink_to(root / "data")
            with self.assertRaises(ValueError):
                pilot.inventory(root)

    def test_bogus_release_is_refused_before_executable_creation(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve(); bundle = root / "bundle.tar.gz"; bundle.write_bytes(b"not the pinned release")
            with self.assertRaisesRegex(ValueError, "bundle digest"):
                pilot.prepare_release(bundle, root / "provider")
            self.assertFalse((root / "provider").exists())

    def test_cargo_artifact_hardlinks_do_not_weaken_owner_document_admission(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve(); binary = root / "build"; binary.write_bytes(b"synthetic build artifact")
            os.link(binary, root / "cargo-output-alias")
            self.assertEqual(pilot.build_binary(binary), b"synthetic build artifact")
            with self.assertRaises(ValueError):
                pilot.OWNER_IO.regular(binary, 1024)
            alias = root / "symlink"; alias.symlink_to(binary)
            with self.assertRaises(ValueError):
                pilot.build_binary(alias)

    def test_subprocess_output_flood_is_refused(self):
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaisesRegex(ValueError, "output budget"):
                pilot.command([sys.executable, "-c", "import sys; sys.stdout.buffer.write(b'x'*4096); sys.stdout.flush()"],
                              cwd=Path(temp), max_output=128, timeout=2)

    def test_subprocess_timeout_is_refused(self):
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaisesRegex(ValueError, "deadline"):
                pilot.command([sys.executable, "-c", "import time; time.sleep(2)"], cwd=Path(temp), timeout=0.05)


if __name__ == "__main__":
    unittest.main()
