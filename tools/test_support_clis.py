"""Negative observers for the explicit local metadata-only CLI driver."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

import check_support_clis as driver


class SupportConformanceObservers(unittest.TestCase):
    def test_panic_and_argument_errors_cannot_count_as_support_refusals(self):
        for code, stderr in [(101, b"thread main panicked"), (2, b"error: unknown argument")]:
            with self.subTest(code=code):
                result = subprocess.CompletedProcess([], code, b"", stderr)
                with self.assertRaises(ValueError):
                    driver.require_refusal("aniflow", "preview", result, driver.INVALID)

    def test_native_refusal_must_match_the_command_and_domain_reason(self):
        value = {"schema_version": 1, "command": "diagnostics_preview", "status": "error",
                 "error": {"category": "input", "message": driver.INVALID}}
        def result():
            return subprocess.CompletedProcess([], 3, b"", json.dumps(value).encode())
        driver.require_refusal("aniflow", "preview", result(), driver.INVALID)
        for key, replacement in [("command", "diagnostics_bundle"), ("status", "success")]:
            changed = dict(value)
            changed[key] = replacement
            with self.subTest(key=key), self.assertRaises(ValueError):
                driver.require_refusal("aniflow", "preview",
                    subprocess.CompletedProcess([], 3, b"", json.dumps(changed).encode()), driver.INVALID)
        value["error"]["message"] = "unrelated failure"
        with self.assertRaises(ValueError):
            driver.require_refusal("aniflow", "preview", result(), driver.INVALID)

    def test_renderflow_native_text_requires_the_exact_domain_refusal(self):
        good = subprocess.CompletedProcess([], 1, b"", ("Error: " + driver.INVALID + "\n").encode())
        driver.require_refusal("renderflow", "inspect", good, driver.INVALID)
        for code, out, err in [(101, b"", good.stderr), (1, b"extra result", good.stderr),
                               (1, b"", b"Error: another failure\n")]:
            with self.subTest(code=code, out=out), self.assertRaises(ValueError):
                driver.require_refusal("renderflow", "inspect",
                    subprocess.CompletedProcess([], code, out, err), driver.INVALID)

    def test_valid_looking_json_cannot_hide_a_crash_or_wrong_typed_category(self):
        import copy
        fixtures = {
            "aniflow": (3, {"schema_version": 1, "command": "diagnostics_preview", "status": "error",
                "error": {"category": "input", "message": driver.INVALID}}),
            "flow": (3, {"schema_version": "flow.command-result/v1", "command": "diagnostics-preview",
                "outcome": "refused", "exit_code": 3, "result": None,
                "diagnostics": [{"code": "invalid_input", "message": driver.INVALID}]}),
            "optiflow": (2, {"schema": "optiflow.command-result.v1", "command": "diagnostics preview",
                "outcome": {"class": "invalid_input", "exit_code": 2}, "result": None,
                "diagnostics": [{"code": "invalid_command_input", "severity": "error",
                    "classification": "input", "impact": "blocks_command", "message": driver.INVALID}]}),
        }
        def result(tool, code, value):
            data = json.dumps(value).encode()
            return subprocess.CompletedProcess([], code, b"" if tool == "aniflow" else data,
                                                data if tool == "aniflow" else b"")
        for tool, (code, value) in fixtures.items():
            driver.require_refusal(tool, "preview", result(tool, code, value), driver.INVALID)
            for wrong in (70, 101):
                with self.subTest(tool=tool, exit=wrong), self.assertRaises(ValueError):
                    driver.require_refusal(tool, "preview", result(tool, wrong, value), driver.INVALID)
            changed = copy.deepcopy(value)
            if tool == "aniflow":
                changed["error"]["category"] = "internal"
            elif tool == "flow":
                changed["diagnostics"][0]["code"] = "io"
            else:
                changed["diagnostics"][0]["classification"] = "internal"
            with self.subTest(tool=tool, wrong_category=True), self.assertRaises(ValueError):
                driver.require_refusal(tool, "preview", result(tool, code, changed), driver.INVALID)

    def test_failure_retains_prior_cases_and_checks_the_other_selected_tools(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "report.json"
            def check(tool, binary, cases):
                cases.append("preview-no-write")
                if tool == "aniflow":
                    raise ValueError("selected fixture failure")
                return {"tool": tool, "state": "passed", "cases": list(cases)}
            argv = ["check_support_clis.py", "--flow", "/fake/flow", "--aniflow", "/fake/aniflow",
                    "--renderflow", "/fake/renderflow", "--output", str(output)]
            with mock.patch.object(sys, "argv", argv), mock.patch.object(driver, "check", side_effect=check) as call:
                self.assertEqual(driver.main(), 1)
            report = json.loads(output.read_text())
            self.assertEqual(call.call_count, 3)
            self.assertFalse(report["complete"])
            self.assertFalse(report["all_selected_passed"])
            self.assertEqual(report["tools"][1]["cases"], [{"id": "preview-no-write", "state": "passed"}])
            self.assertEqual(report["tools"][2]["state"], "passed")

    def test_partial_selection_cannot_claim_complete_coverage(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "report.json"
            argv = ["check_support_clis.py", "--flow", "/fake/flow", "--output", str(output)]
            with mock.patch.object(sys, "argv", argv), mock.patch.object(driver, "check",
                return_value={"tool": "flow", "state": "passed", "cases": []}):
                self.assertEqual(driver.main(), 0)
            report = json.loads(output.read_text())
            self.assertTrue(report["all_selected_passed"])
            self.assertFalse(report["complete"])
            self.assertEqual(report["unobserved_tools"], ["aniflow", "optiflow", "renderflow"])

    def test_existing_report_refuses_before_launching_any_tool(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "report.json"
            output.write_bytes(b"retained receipt")
            argv = ["check_support_clis.py", "--flow", "/fake/flow", "--output", str(output)]
            with mock.patch.object(sys, "argv", argv), mock.patch.object(driver, "check") as call:
                with self.assertRaises(ValueError):
                    driver.main()
                call.assert_not_called()
            self.assertEqual(output.read_bytes(), b"retained receipt")

    @unittest.skipUnless(os.name == "posix", "selected local profile uses Unix symlinks")
    def test_symlink_loop_retains_a_fixed_failure_and_other_selected_tools(self):
        with tempfile.TemporaryDirectory(prefix="PRIVATE_SUPPORT_CANARY_") as directory:
            root = Path(directory)
            first, second = root / "first", root / "second"
            first.symlink_to(second)
            second.symlink_to(first)
            output = root / "report.json"
            argv = ["check_support_clis.py", "--flow", str(first),
                    "--renderflow", str(root / "missing"), "--output", str(output)]
            with mock.patch.object(sys, "argv", argv):
                self.assertEqual(driver.main(), 1)
            raw = output.read_text()
            report = json.loads(raw)
            self.assertEqual([row["tool"] for row in report["tools"]], ["flow", "renderflow"])
            self.assertTrue(all(row["state"] == "failed" for row in report["tools"]))
            self.assertNotIn(str(root), raw)


if __name__ == "__main__":
    unittest.main()
