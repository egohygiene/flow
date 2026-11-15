"""Bounded fixture refusals. These cases never establish native Mac execution."""
import copy
from contextlib import contextmanager
import json
import os
from pathlib import Path
import stat
import struct
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import macos_native_trial as trial


class MacTrialTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.sources = {}
        for name in trial.TOOLCHAINS:
            directory = self.root / name
            directory.mkdir()
            self.sources[name] = {"directory": str(directory), "revision": "1" * 40,
                                  "tree": "2" * 40, "cargo_lock_sha256": "3" * 64}
        self.value = {"schema_version": trial.SCHEMA, "target": "aarch64-apple-darwin", "sources": self.sources}

    @contextmanager
    def refuses(self, reason):
        with self.assertRaises(trial.Refusal) as caught:
            yield
        self.assertIs(type(caught.exception), trial.Refusal)
        self.assertEqual(str(caught.exception), reason)

    def put(self, name, data):
        target = self.root / name
        target.write_bytes(data)
        return {"path": {"encoding": "utf8", "value": str(target)}, "sha256": trial.sha(data), "byte_count": len(data)}

    def json_pin(self, name, value):
        return self.put(name, json.dumps(value).encode())

    def selection(self):
        rows, tools = [], []
        binary = struct.pack("<8I", 0xFEEDFACF, 0x100000C, 0, 2, 0, 0, 0, 0)
        for name in trial.TOOLCHAINS:
            executable = self.put(name + "-binary", binary)
            descriptor = self.json_pin(name + "-contract", {"schema_version": trial.CONTRACTS[name], "version": "0.1.0"})
            contracts = [{"identifier": trial.CONTRACTS[name], "sha256": descriptor["sha256"]}]
            rows.append({"id": name, "tool": name, "interface": "standalone_cli", "target": self.value["target"],
                         "implementation": "implemented", "artifact": {"version": "0.1.0", "source_revision": self.sources[name]["revision"],
                         "sha256": executable["sha256"], "byte_count": executable["byte_count"]}, "contracts": contracts})
            tools.append({"tool": name, "row_id": name, "executable": executable,
                          "contracts": [{"identifier": trial.CONTRACTS[name], "file": descriptor}], "extras": []})
        cohort = {"schema_version": "flow.release-cohort/v1", "targets": [self.value["target"]],
                  "retained_rows": [r["id"] for r in rows], "rows": rows}
        selected = {"schema_version": "flow.suite-install-request/v1", "cohort": self.json_pin("cohort", cohort),
                    "target": self.value["target"], "install_directory": {"encoding": "utf8", "value": str(self.root / "fresh-install")},
                    "tools": tools, "timeout_ms": 30000, "maximum_total_bytes": 1000000}
        return selected, cohort

    def bind_selection(self, selected):
        target = self.root / "install-request"
        if target.exists():
            target.unlink()
        self.value["install_request"] = self.json_pin("install-request", selected)

    def test_plan_never_invokes_program_or_qualifies(self):
        with patch.object(trial, "command", side_effect=AssertionError("no command")):
            report = trial.report("plan", trial.request(json.dumps(self.value).encode()), "0" * 64)
        self.assertEqual(report["state"], "planned")
        self.assertFalse(report["build_performed"])
        self.assertEqual(report["native_execution"], "not_executed_by_this_helper")
        self.assertNotIn(str(self.root), json.dumps(report))

    def test_duplicate_request_member_refuses(self):
        with self.refuses("duplicate_json_member"):
            trial.document(b'{"target":1,"target":2}')

    def test_request_rejects_extra_authority_or_host(self):
        for key in ("host_target", "approved", "environment"):
            value = {**self.value, key: "pretend"}
            with self.refuses("document_shape_differs"):
                trial.request(json.dumps(value).encode())

    def test_linux_cannot_pass_mac_preflight(self):
        with patch.object(trial.platform, "system", return_value="Linux"), patch.object(trial.platform, "machine", return_value="x86_64"), patch.object(trial, "source_check", side_effect=trial.Refusal("fixture_unavailable")), patch.object(trial, "command", side_effect=AssertionError("no host query")):
            result = trial.report("preflight", self.value, "0" * 64)
        self.assertEqual(result["state"], "blocked")
        self.assertIsNone(result["host_target"])
        self.assertEqual(result["checks"][0]["reason"], "native_matching_mac_required")
        self.assertEqual(result["native_qualification"], "not_established")

    def test_unknown_hardware_query_cannot_pass_intel(self):
        self.value["target"] = "x86_64-apple-darwin"
        def query(argv, _):
            if argv[0] == "/usr/sbin/sysctl":
                return b"arbitrary failure text /private/secret"
            if argv[0] == "rustup":
                return ("rustc " + argv[2] + " (fixture)\nhost: x86_64-apple-darwin\n").encode()
            return b"/private/secret\n"
        with patch.object(trial.platform, "system", return_value="Darwin"), patch.object(trial.platform, "machine", return_value="x86_64"), patch.object(trial, "source_check", return_value={"version": "fixture"}), patch.object(trial, "command", side_effect=query):
            result = trial.report("preflight", self.value, "0" * 64)
        self.assertEqual(result["state"], "blocked")
        check = next(c for c in result["checks"] if c["check"] == "native_process_architecture")
        self.assertEqual(check["reason"], "native_process_architecture_unavailable")
        self.assertNotIn("/private/secret", json.dumps(result))

    def test_native_errors_are_not_exported(self):
        with patch.object(trial, "source_check", side_effect=ValueError("private source path and secret")):
            result = trial.report("preflight", self.value, "0" * 64)
        self.assertNotIn("private source", json.dumps(result))
        self.assertEqual(next(c for c in result["checks"] if c["check"] == "flow_source")["reason"], "prerequisite_unavailable")

    def test_exact_mach_o_selection_observes_only_bytes(self):
        selected, _ = self.selection()
        self.bind_selection(selected)
        result = trial.observe_install(self.value)
        self.assertEqual(set(result), set(trial.TOOLCHAINS))
        self.assertNotIn(str(self.root), json.dumps(result))
        self.assertFalse((self.root / "fresh-install").exists())

    def test_elf_cannot_be_relabelled_mac(self):
        selected, _ = self.selection()
        selected["tools"][0]["executable"] = self.put("elf", b"\x7fELF" + bytes(60))
        self.bind_selection(selected)
        with self.refuses("native_mach_o_executable_cpu_required"):
            trial.observe_install(self.value)

    def test_other_cpu_or_shared_library_refuses(self):
        for cpu, kind in ((0x1000007, 2), (0x100000C, 6)):
            binary = struct.pack("<8I", 0xFEEDFACF, cpu, 0, kind, 0, 0, 0, 0)
            with self.refuses("native_mach_o_executable_cpu_required"):
                trial.mach_o(binary, "aarch64-apple-darwin")

    def test_duplicate_or_omitted_tool_refuses(self):
        selected, _ = self.selection()
        for tools in ([selected["tools"][0]] * 3, selected["tools"][:2]):
            changed = {**selected, "tools": tools}
            self.bind_selection(changed)
            with self.refuses("exact_three_trial_tools_required"):
                trial.observe_install(self.value)

    def test_changed_source_and_contract_cohort_refuse(self):
        selected, cohort = self.selection()
        for field, changed in (("source_revision", "f" * 40), ("sha256", "f" * 64)):
            edited = copy.deepcopy(cohort)
            edited["rows"][0]["artifact"][field] = changed
            selected["cohort"] = self.json_pin("cohort-" + field, edited)
            self.bind_selection(selected)
            with self.refuses("artifact_source_or_bytes_differ"):
                trial.observe_install(self.value)
        edited = copy.deepcopy(cohort)
        edited["rows"][0]["contracts"][0]["sha256"] = "f" * 64
        selected["cohort"] = self.json_pin("cohort-contract", edited)
        self.bind_selection(selected)
        with self.refuses("cohort_contract_pin_differs"):
            trial.observe_install(self.value)

    def test_cohort_target_and_membership_refuse(self):
        selected, cohort = self.selection()
        cases = [("target", {**cohort, "targets": ["x86_64-apple-darwin"]}, "cohort_target_differs"),
                 ("members", {**cohort, "retained_rows": ["flow"]}, "cohort_membership_differs")]
        for name, changed, reason in cases:
            selected["cohort"] = self.json_pin("cohort-" + name, changed)
            self.bind_selection(selected)
            with self.refuses(reason):
                trial.observe_install(self.value)

    def test_install_limits_and_aggregate_refuse_before_binary_read(self):
        selected, _ = self.selection()
        for field, value, reason in (("timeout_ms", True, "installation_limits_differ"),
                                     ("maximum_total_bytes", 0, "installation_limits_differ"),
                                     ("maximum_total_bytes", 1, "installation_aggregate_byte_bound")):
            changed = {**selected, field: value}
            self.bind_selection(changed)
            with patch.object(trial, "read", wraps=trial.read) as reads:
                with self.refuses(reason):
                    trial.observe_install(self.value)
                self.assertFalse(any(len(c.args) > 1 and c.args[1] == trial.MAX_BINARY for c in reads.call_args_list))

    def test_aniflow_candidate_and_intel_owner_gates(self):
        root = Path(self.sources["aniflow"]["directory"])
        (root / "scripts").mkdir()
        (root / "scripts/release.py").write_text("# fixture; must not launch\n")
        (root / "Cargo.lock").write_text("# fixture\n")
        self.sources["aniflow"]["cargo_lock_sha256"] = trial.sha((root / "Cargo.lock").read_bytes())
        (root / "Cargo.toml").write_text('[package]\nname = "aniflow"\nversion = "0.1.0"\n')
        (root / ".egohygiene").mkdir()
        (root / ".egohygiene/release-build.json").write_text(json.dumps({"targets": [{"triple": "aarch64-apple-darwin"}], "rust_toolchain": "1.85.0"}))
        def git(argv, _):
            if argv[-1] == "HEAD":
                return b"1" * 40 + b"\n"
            if argv[-1] == "HEAD^{tree}":
                return b"2" * 40 + b"\n"
            return b""
        with patch.object(trial, "command", side_effect=git):
            with self.refuses("aniflow_reviewed_successor_required"):
                trial.source_check("aniflow", self.sources["aniflow"], "aarch64-apple-darwin")
            with self.refuses("aniflow_target_owner_unavailable"):
                trial.source_check("aniflow", self.sources["aniflow"], "x86_64-apple-darwin")

    def test_changed_and_linked_files_refuse(self):
        selected = self.put("pin", b"exact")
        (self.root / "pin").write_bytes(b"other")
        with self.refuses("selected_file_pin_differs"):
            trial.pinned(selected)
        os.link(self.root / "pin", self.root / "alias")
        with self.refuses("bounded_single_link_regular_input_required"):
            trial.read(self.root / "pin")
        (self.root / "symlink").symlink_to(self.root / "pin")
        with self.refuses("canonical_path_required"):
            trial.read(self.root / "symlink")

    def test_report_is_private_fresh_and_redacted(self):
        report = trial.report("plan", self.value, "0" * 64)
        output = self.root / "private-report"
        trial.write_report(output, report, self.value)
        self.assertEqual(stat.S_IMODE(output.stat().st_mode), 0o700)
        self.assertEqual(stat.S_IMODE((output / "report.json").stat().st_mode), 0o600)
        self.assertEqual(set(p.name for p in output.iterdir()), {"report.json"})
        self.assertNotIn(str(self.root), (output / "report.json").read_text())
        with self.refuses("fresh_report_root_required"):
            trial.write_report(output, report, self.value)
        with self.refuses("report_input_overlap"):
            trial.write_report(Path(self.sources["flow"]["directory"]) / "report", report, self.value)

    def test_source_head_tree_lock_and_cleanliness_bound(self):
        root = Path(self.sources["flow"]["directory"])
        (root / "scripts/release").mkdir(parents=True)
        (root / "scripts/release/package.py").write_text("# owner fixture\n")
        (root / ".egohygiene").mkdir()
        (root / ".egohygiene/release-build.json").write_text(json.dumps({"native_targets": list(trial.TARGETS), "rust_toolchain": "1.85.0"}))
        (root / "Cargo.toml").write_text('[package]\nname = "flow"\nversion = "0.1.0"\n')
        (root / "Cargo.lock").write_text("# fixture\n")
        git = ["git", "-c", "core.hooksPath=/dev/null", "-C", str(root)]
        subprocess.run(git + ["init", "--quiet"], check=True)
        subprocess.run(git + ["add", "."], check=True)
        subprocess.run(git + ["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--quiet", "-m", "fixture"], check=True)
        selected = {"directory": str(root), "revision": subprocess.check_output(git + ["rev-parse", "HEAD"]).decode().strip(),
                    "tree": subprocess.check_output(git + ["rev-parse", "HEAD^{tree}"]).decode().strip(), "cargo_lock_sha256": trial.sha((root / "Cargo.lock").read_bytes())}
        self.assertEqual(trial.source_check("flow", selected, self.value["target"])["version"], "0.1.0")
        for field, reason in (("revision", "source_revision_differs"), ("tree", "source_tree_differs"), ("cargo_lock_sha256", "source_cargo_lock_differs")):
            changed = {**selected, field: "f" * len(selected[field])}
            with self.refuses(reason):
                trial.source_check("flow", changed, self.value["target"])
        (root / "unreviewed").write_text("preserve")
        with self.refuses("source_not_clean"):
            trial.source_check("flow", selected, self.value["target"])


if __name__ == "__main__":
    unittest.main()
