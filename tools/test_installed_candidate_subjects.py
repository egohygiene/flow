#!/usr/bin/env python3
"""Synthetic admission observers only; no package build or executable is run.

The real package verifier reads a tiny test-owned inventory. The immutable Opti
hashes are replaced only inside these fixtures, whose bytes are never executed.
"""
import copy
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import installed_candidate_subjects as subject

cli = subject.cli
PRODUCTION_PROVIDER_SHA, PRODUCTION_BUNDLE_SHA = cli.PROVIDER_SHA, cli.BUNDLE_SHA
OPTI = b"synthetic immutable Optiflow fixture, never executable\n"
BUNDLE = b"synthetic immutable release fixture, never extracted\n"
FLOW_REV, FLOW_TREE = "1" * 40, "2" * 40
RENDER_REV, RENDER_TREE = "3" * 40, "4" * 40


def encoded(value):
    return cli.canonical(value) + b"\n"


def elf(label):
    data = bytearray(128)
    data[:6] = b"\x7fELF\x02\x01"
    data[18:20] = (62).to_bytes(2, "little")
    return bytes(data) + label.encode()


def identity(data):
    return {"sha256": cli.sha(data), "byte_count": len(data)}


class Fixture:
    def __init__(self, base):
        self.base = base
        self.root, self.package = base / "installed 作品", base / "package"
        for path in (self.root / "bin", self.package):
            path.mkdir(parents=True)
        self.bytes = {"flow": elf("Flow"), "renderflow": elf("Render"), "optiflow": OPTI}
        self.contracts = {
            "flow": {"schema_version": "flow.release-contract/v1", "version": "0.1.0", "primary_binary": "flow"},
            "renderflow": {"schema_version": "renderflow.release-contract/v2", "tool": "renderflow", "version": "0.3.0-rc.1",
                "profiles": [{"id": "canonical-build-v1", "source_mutation": False}]},
            "optiflow": {"schema_version": "optiflow.fixture-contract/v1"},
        }
        self.contract_ids = {name: value["schema_version"] for name, value in self.contracts.items()}
        self.cohort = {"schema_version": "flow.release-cohort/v1", "id": "synthetic-fixture", "flow_version": "0.1.0",
            "targets": [subject.TARGET], "retained_rows": [], "rows": []}
        self.intent = {"schema_version": "flow.suite-install-request/v1", "target": subject.TARGET,
            "install_directory": cli.native_path(self.root), "tools": [], "timeout_ms": 1000, "maximum_total_bytes": 1000000}
        for name in self.bytes:
            row_id = subject.RENDER_ROW if name == "renderflow" else name + ".fixture." + subject.TARGET
            version = "0.1.1" if name == "optiflow" else self.contracts[name]["version"]
            revision = {"flow": FLOW_REV, "renderflow": RENDER_REV, "optiflow": cli.PROVIDER_SOURCE}[name]
            row = {"id": row_id, "tool": name, "target": subject.TARGET,
                "interface": "flow_route" if name == "renderflow" else "standalone_cli",
                "implementation": "implemented", "profile": "canonical-build-v1" if name == "renderflow" else "fixture",
                "effects": ["read_originals", "write_new_outputs"],
                "artifact": {"version": version, "source_revision": revision, "name": name + "-" + subject.TARGET,
                    **identity(self.bytes[name]), "release_url": "https://github.com/egohygiene/" + name + "/commit/" + revision},
                "contracts": []}
            self.cohort["retained_rows"].append(row_id)
            self.cohort["rows"].append(row)
            self.intent["tools"].append({"tool": name, "row_id": row_id,
                "executable": {"path": cli.native_path(base / "originals" / name), **identity(self.bytes[name])},
                "contracts": [], "extras": []})
            (self.root / "bin" / name).write_bytes(self.bytes[name])
            self.set_contract(name, self.contracts[name])
        self.receipt = {"schema_version": "flow.suite-installation/v1", "root": cli.native_path(self.root),
            "target": subject.TARGET, "files": [], "candidate_acknowledged": True,
            "provider_invoked": False, "publisher_authenticated": False, "native_qualified": False}
        self.manifest = {"schema_version": "flow.release-manifest/v1", "repository": "egohygiene/flow", "version": "0.1.0",
            "source_revision": FLOW_REV, "target": subject.TARGET, "primary_binary": "flow-" + subject.TARGET,
            "compiled_descriptor": "flow-release-contract-v1.json", "provider_cohort": "provider-cohort.json",
            "library_source": "flow-0.1.0-source.tar.gz", "qualification": "not_established_by_preparation",
            "publication": "not_performed", "registry_channel": "unavailable",
            "native_build": {"host_target": subject.TARGET, "rustc": "rustc 1.85.0 (synthetic fixture)", "isolated_builds": 2}}
        self.build = {"schema_version": "flow.combined-package-build-observation/v1",
            "source": {"revision": FLOW_REV, "tree": FLOW_TREE, "clean_before": True, "clean_after": True},
            "native_build": {"profile": "release", "isolated_builds": 2, "rustc": self.manifest["native_build"]["rustc"],
                "binary_equal": True, "descriptor_equal": True}}
        render_pin = identity(self.bytes["renderflow"])
        contract_pin = identity(self.contract_bytes("renderflow"))
        self.proof = {"schema_version": "renderflow.ubuntu-candidate-summary/v1",
            "source": {"revision": RENDER_REV, "tree": RENDER_TREE}, "target": subject.TARGET,
            "image": {"reference": subject.IMAGE}, "baseline": {"libc": "glibc 2.39", "rustc": "rustc 1.94.0 (synthetic fixture)"},
            "release_authenticated": False, "supported_matrix_qualified": False, "publication_performed": False,
            "assets": [{"name": "renderflow-" + subject.TARGET, "sha256": render_pin["sha256"], "bytes": render_pin["byte_count"]},
                {"name": "renderflow-" + subject.TARGET + ".contract.json", "sha256": contract_pin["sha256"], "bytes": contract_pin["byte_count"]}],
            "local_smoke": {"schema_version": "renderflow.local-candidate-smoke/v1", "qualification": "local_captured_binary_smoke_only",
                "binary": {"sha256": render_pin["sha256"], "bytes": render_pin["byte_count"], "target": subject.TARGET},
                "checks": {"version": "0.3.0-rc.1", "version_help_doctor": "passed", "spec_plan_dry_run": "passed", "missing_config_refused": True},
                **{flag: False for flag in ["source_authenticated", "release_authenticated", "supported_host_qualified",
                    "native_matrix_qualified", "publication_performed"]}}}
        self.value = {"schema_version": "flow.installed-candidate-selection/v1", "target": subject.TARGET,
            "installation_root": str(self.root), "flow_package_directory": str(self.package),
            "flow_source_revision": FLOW_REV, "flow_source_tree": FLOW_TREE,
            "flow_package_build_observation": str(base / "build.json"),
            "renderflow_source_revision": RENDER_REV, "renderflow_source_tree": RENDER_TREE,
            "renderflow_binary_sha256": render_pin["sha256"], "renderflow_contract_sha256": contract_pin["sha256"],
            "renderflow_candidate_proof": str(base / "render-proof.json"), "renderflow_row_id": subject.RENDER_ROW,
            "optiflow_row_id": self.row("optiflow")["id"], "optiflow_release_bundle": str(base / "opti-bundle")}
        (base / "opti-bundle").write_bytes(BUNDLE)
        self.emit()

    def row(self, name):
        return next(row for row in self.cohort["rows"] if row["tool"] == name)

    def tool(self, name):
        return next(tool for tool in self.intent["tools"] if tool["tool"] == name)

    def contract_bytes(self, name):
        value = self.contracts[name]
        return value if isinstance(value, bytes) else encoded(value)

    def set_contract(self, name, value):
        self.contracts[name] = value
        data = self.contract_bytes(name)
        self.row(name)["contracts"] = [{"identifier": self.contract_ids[name], "sha256": cli.sha(data)}]
        self.tool(name)["contracts"] = [{"identifier": self.contract_ids[name],
            "file": {"path": cli.native_path(self.base / "originals" / (name + ".json")), **identity(data)}}]
        if name == "renderflow" and hasattr(self, "proof"):
            self.proof["assets"][1].update(sha256=cli.sha(data), bytes=len(data))
            self.value["renderflow_contract_sha256"] = cli.sha(data)

    def emit(self):
        for name in self.bytes:
            directory = self.root / "contracts" / name
            directory.mkdir(parents=True, exist_ok=True)
            (directory / "00.json").write_bytes(self.contract_bytes(name))
        cohort = encoded(self.cohort)
        (self.root / "cohort.json").write_bytes(cohort)
        self.intent["cohort"] = {"path": cli.native_path(self.base / "originals/cohort.json"), **identity(cohort)}
        intent = encoded(self.intent)
        (self.root / "installation-intent.json").write_bytes(intent)
        self.receipt["intent_sha256"] = cli.sha(intent)
        receipt = encoded(self.receipt)
        (self.root / "installation.json").write_bytes(receipt)
        self.value["installation_receipt_sha256"] = cli.sha(receipt)
        assets = {self.manifest["primary_binary"]: (self.bytes["flow"], "cli-binary"),
            self.manifest["compiled_descriptor"]: (self.contract_bytes("flow"), "compiled-contract"),
            self.manifest["provider_cohort"]: (encoded(self.packaged_cohort()), "provider-cohort"),
            self.manifest["library_source"]: (b"synthetic source archive, never extracted\n", "library-source")}
        self.manifest["assets"] = []
        for name, (data, kind) in assets.items():
            (self.package / name).write_bytes(data)
            self.manifest["assets"].append({"name": name, "kind": kind, **identity(data)})
        raw = encoded(self.manifest)
        (self.package / subject.PACKAGE.MANIFEST).write_bytes(raw)
        self.value["flow_package_manifest_sha256"] = cli.sha(raw)
        sums = {name: cli.sha(data) for name, (data, _) in assets.items()}
        sums[subject.PACKAGE.MANIFEST] = cli.sha(raw)
        (self.package / "SHA256SUMS").write_text("".join(sums[name] + "  " + name + "\n" for name in sorted(sums)))
        self.build["manifest_sha256"] = cli.sha(raw)
        for path_key, sha_key, value in [("flow_package_build_observation", "flow_package_build_observation_sha256", self.build),
                ("renderflow_candidate_proof", "renderflow_candidate_proof_sha256", self.proof)]:
            data = encoded(value)
            Path(self.value[path_key]).write_bytes(data)
            self.value[sha_key] = cli.sha(data)

    def admit(self):
        return subject.InstalledCandidate(self.value)

    def packaged_cohort(self):
        cohort = copy.deepcopy(self.cohort)
        cohort["rows"] = [row for row in cohort["rows"] if row["tool"] != "flow"]
        cohort["retained_rows"] = [row["id"] for row in cohort["rows"]]
        return cohort


class AdmissionCases(unittest.TestCase):
    def setUp(self):
        for field, data in [("PROVIDER_SHA", OPTI), ("BUNDLE_SHA", BUNDLE)]:
            selected = patch.object(cli, field, cli.sha(data))
            selected.start()
            self.addCleanup(selected.stop)

    def fixture(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        return Fixture(Path(directory.name))

    def test_production_opti_pins_are_immutable(self):
        self.assertEqual(PRODUCTION_PROVIDER_SHA, "46af9399f3785607f835d9a805a8daa42c93597772c6ee812911615ea4b76280")
        self.assertEqual(PRODUCTION_BUNDLE_SHA, "f576131f2695a218fabfaa5167fd16f17641c267194254a9e624303a8cf23e07")

    def test_closed_selection_rejects_missing_extra_boolean_and_malformed_fields(self):
        fixture = self.fixture()
        changes = [{"private_path": "/secret"}, {"target": "aarch64-apple-darwin"}, {"renderflow_row_id": "other"},
            {"flow_source_revision": "main"}, {"flow_source_tree": True}, {"renderflow_binary_sha256": "A" * 64},
            {"flow_package_manifest_sha256": False}, {"installation_root": ""}]
        for change in changes:
            with self.subTest(change=change), self.assertRaises(ValueError):
                subject.selection({**fixture.value, **change})
        missing = dict(fixture.value)
        missing.pop("flow_source_tree")
        with self.assertRaisesRegex(ValueError, "closed"):
            subject.selection(missing)

    def test_loaded_selection_is_frozen_and_reobserved(self):
        fixture = self.fixture()
        path = fixture.base / "selection.json"
        raw = encoded(fixture.value)
        path.write_bytes(raw)
        candidate = subject.InstalledCandidate.load(path)
        self.assertEqual(candidate.selection_sha256, cli.sha(raw))
        path.write_bytes(raw + b" ")
        with self.assertRaisesRegex(ValueError, "selection bytes changed"):
            candidate.reobserve()

    def test_intent_cannot_select_another_installation_root(self):
        fixture = self.fixture()
        fixture.intent["install_directory"] = cli.native_path(fixture.base)
        fixture.emit()
        with self.assertRaisesRegex(ValueError, "intent differs"):
            fixture.admit()

    def test_admitted_subject_uses_installed_layout_and_projects_only_public_identity(self):
        fixture = self.fixture()
        secret = "/private/source-checkout/secret-metadata"
        fixture.manifest["native_build"]["rustc"] += "\n" + secret
        fixture.build["native_build"]["rustc"] = fixture.manifest["native_build"]["rustc"]
        for value in (fixture.manifest, fixture.build, fixture.proof):
            value["private_freeform"] = secret
        fixture.emit()
        candidate = fixture.admit()
        self.assertEqual(candidate.flow, fixture.root / "bin/flow")
        self.assertEqual(candidate.provider, fixture.root / "bin/renderflow")
        self.assertEqual(candidate.optiflow, fixture.root / "bin/optiflow")
        self.assertEqual(candidate.descriptor, fixture.root / "contracts/renderflow/00.json")
        public = json.dumps([candidate.flow_identity, candidate.render_identity])
        self.assertNotIn(secret, public)
        self.assertNotIn(str(fixture.base), public)
        self.assertEqual(candidate.flow_identity["actual_build_tree"], FLOW_TREE)

    def test_source_tree_and_build_claims_must_join_explicit_selection(self):
        mutations = [lambda f: f.manifest.update(source_revision="9" * 40),
            lambda f: f.build["source"].update(revision="9" * 40), lambda f: f.build["source"].update(tree="9" * 40),
            lambda f: f.proof["source"].update(revision="9" * 40), lambda f: f.proof["source"].update(tree="9" * 40),
            lambda f: f.build["source"].update(clean_before=1), lambda f: f.build["source"].update(clean_after=False),
            lambda f: f.build["native_build"].update(profile="dev"), lambda f: f.build["native_build"].update(binary_equal=1),
            lambda f: f.build["native_build"].update(descriptor_equal=False),
            lambda f: f.manifest["native_build"].update(isolated_builds=2.0),
            lambda f: f.build["native_build"].update(isolated_builds=True)]
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                fixture = self.fixture()
                mutate(fixture)
                fixture.emit()
                with self.assertRaises(ValueError):
                    fixture.admit()

    def test_explicit_manifest_proof_provider_and_contract_digests_refuse_substitution(self):
        for field in ["flow_package_manifest_sha256", "flow_package_build_observation_sha256", "installation_receipt_sha256",
                "renderflow_candidate_proof_sha256", "renderflow_binary_sha256", "renderflow_contract_sha256"]:
            with self.subTest(field=field):
                fixture = self.fixture()
                fixture.value[field] = "0" * 64
                with self.assertRaises(ValueError):
                    fixture.admit()
        for name in ("flow", "renderflow", "optiflow"):
            with self.subTest(installed=name):
                fixture = self.fixture()
                path = fixture.root / "bin" / name
                path.write_bytes(path.read_bytes() + b"changed")
                with self.assertRaises(ValueError):
                    fixture.admit()
        fixture = self.fixture()
        Path(fixture.value["optiflow_release_bundle"]).write_bytes(b"wrong bundle")
        with self.assertRaisesRegex(ValueError, "immutable Optiflow"):
            fixture.admit()

    def test_coherent_cohort_changes_cannot_relabel_selected_subjects(self):
        changes = [("flow", "interface", "rust_library"), ("optiflow", "interface", "rust_library"),
            ("renderflow", "implementation", "planned"), ("renderflow", "profile", "other"),
            ("renderflow", "target", "other"), ("renderflow", "artifact.source_revision", "9" * 40),
            ("flow", "artifact.source_revision", "9" * 40), ("optiflow", "artifact.source_revision", "9" * 40),
            ("renderflow", "artifact.byte_count", True), ("renderflow", "artifact.release_url", "https://example.com/release")]
        for name, field, value in changes:
            with self.subTest(name=name, field=field):
                fixture = self.fixture()
                row = fixture.row(name)
                if field.startswith("artifact."):
                    row["artifact"][field.split(".")[1]] = value
                else:
                    row[field] = value
                fixture.emit()
                with self.assertRaises(ValueError):
                    fixture.admit()
        fixture = self.fixture()
        fixture.cohort["rows"].append(copy.deepcopy(fixture.row("renderflow")))
        fixture.emit()
        with self.assertRaisesRegex(ValueError, "row|duplicate"):
            fixture.admit()

    def test_provider_projection_preserves_all_provider_rows_and_coverage_in_order(self):
        fixture = self.fixture()
        packaged, installed = fixture.packaged_cohort(), fixture.cohort
        subject.provider_projection(packaged, installed)
        mutations = [lambda c: c["rows"].pop(), lambda c: c["rows"].append(copy.deepcopy(c["rows"][-1])),
            lambda c: c["rows"][1].update(next_action="different provider declaration"),
            lambda c: c["rows"].reverse(), lambda c: c["retained_rows"].reverse(),
            lambda c: c["targets"].append("aarch64-apple-darwin")]
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                changed = copy.deepcopy(installed)
                mutate(changed)
                with self.assertRaises(ValueError):
                    subject.provider_projection(packaged, changed)

    def test_strict_manifest_and_every_selected_contract_reject_duplicate_json(self):
        fixture = self.fixture()
        path = fixture.package / subject.PACKAGE.MANIFEST
        raw = path.read_bytes().rstrip()[:-1] + b',"source_revision":"' + FLOW_REV.encode() + b'"}\n'
        path.write_bytes(raw)
        fixture.value["flow_package_manifest_sha256"] = cli.sha(raw)
        with self.assertRaisesRegex(ValueError, "duplicate JSON member"):
            fixture.admit()
        for name in ("flow", "renderflow", "optiflow"):
            with self.subTest(contract=name):
                fixture = self.fixture()
                raw = fixture.contract_bytes(name).rstrip()[:-1] + b',"schema_version":' + json.dumps(fixture.contract_ids[name]).encode() + b'}\n'
                fixture.set_contract(name, raw)
                fixture.emit()
                with self.assertRaisesRegex(ValueError, "duplicate JSON member"):
                    fixture.admit()
        fixture = self.fixture()
        fixture.set_contract("optiflow", b'{"measurement":NaN}\n')
        fixture.emit()
        with self.assertRaisesRegex(ValueError, "finite JSON"):
            fixture.admit()

    def test_render_descriptor_and_ubuntu_subject_remain_unqualified(self):
        mutations = [lambda f: f.proof.update(release_authenticated=True), lambda f: f.proof.update(supported_matrix_qualified=True),
            lambda f: f.proof.update(publication_performed=True), lambda f: f.proof["baseline"].update(libc="glibc 2.41"),
            lambda f: f.proof["image"].update(reference="ubuntu:latest"),
            lambda f: f.proof["assets"][0].update(bytes=True),
            lambda f: f.proof["local_smoke"].update(native_matrix_qualified=True),
            lambda f: f.proof["local_smoke"]["checks"].update(missing_config_refused=1),
            lambda f: f.proof["local_smoke"]["binary"].update(target="other")]
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                fixture = self.fixture()
                mutate(fixture)
                fixture.emit()
                with self.assertRaises(ValueError):
                    fixture.admit()
        for change in ({"version": "/private/path"}, {"tool": "other"}, {"profiles": [{"id": "canonical-build-v1", "source_mutation": True}]}):
            fixture = self.fixture()
            fixture.set_contract("renderflow", {**fixture.contracts["renderflow"], **change})
            fixture.emit()
            with self.assertRaises(ValueError):
                fixture.admit()

    def test_installation_authority_flags_cannot_be_promoted(self):
        for field, value in [("candidate_acknowledged", False), ("candidate_acknowledged", 1), ("provider_invoked", True),
                ("publisher_authenticated", True), ("native_qualified", True), ("publisher_admission", {"claim": "passed"})]:
            with self.subTest(field=field):
                fixture = self.fixture()
                fixture.receipt[field] = value
                fixture.emit()
                with self.assertRaisesRegex(ValueError, "unqualified installation"):
                    fixture.admit()

    def test_status_uses_installed_executable_and_binds_actual_envelope_and_receipt(self):
        fixture = self.fixture()
        candidate = fixture.admit()
        output = fixture.base / "observations"
        output.mkdir()
        result = {"schema_version": "flow.suite-installation-status/v1", "current_copied_bytes": True, "receipt": candidate.receipt,
            "receipt_sha256": fixture.value["installation_receipt_sha256"]}
        envelope = {"schema_version": "flow.command-result/v1", "command": "suite-status", "outcome": "complete",
            "exit_code": 0, "result": result, "diagnostics": []}
        with patch.object(cli, "command", return_value=(0, encoded(envelope), b"")) as command:
            candidate.observe_status(output, "before")
        self.assertEqual(command.call_args.args[0][0], str(candidate.flow))
        self.assertEqual(command.call_args.kwargs["cwd"], output)
        self.assertFalse((output / "flow").exists())
        for change in ({"command": "version"}, {"outcome": "partial"}, {"exit_code": True},
                {"result": {**result, "schema_version": "other/v1"}},
                {"result": {**result, "current_copied_bytes": 1}},
                {"result": {**result, "receipt_sha256": "0" * 64}},
                {"result": {**result, "receipt": {**candidate.receipt, "native_qualified": True}}}):
            with self.subTest(change=change), patch.object(cli, "command", return_value=(0, encoded({**envelope, **change}), b"")):
                with self.assertRaises(ValueError):
                    candidate.observe_status(output, "refused")

    def test_direct_driver_never_recopies_or_manufactures_installed_subject(self):
        import check_native_publication as native
        fixture = self.fixture()
        candidate = fixture.admit()
        output = fixture.base / "direct-observer"

        class StopBeforeProduct(Exception):
            pass

        with patch.object(native, "command", side_effect=StopBeforeProduct) as command:
            with self.assertRaises(StopBeforeProduct):
                native.run_pilot(candidate.flow, candidate.provider, candidate.descriptor,
                    fixture.base / "providers/img2pdf", output, None, installed=candidate)
        self.assertEqual(command.call_args.args[0][0], str(candidate.flow))
        self.assertFalse((output / "flow").exists())
        self.assertFalse((output / "cohort.json").exists())
        request = cli.document((output / "epub-request.json").read_bytes())
        self.assertEqual(request["cohort"], cli.native_path(candidate.cohort_path))
        self.assertEqual(request["provider_executable"], cli.native_path(candidate.provider))
        self.assertEqual(request["release_descriptor"], cli.native_path(candidate.descriptor))

    def test_read_only_driver_selects_installed_opti_after_release_byte_comparison(self):
        fixture = self.fixture()
        candidate = fixture.admit()
        output = fixture.base / "read-only-observer"
        extracted = fixture.base / "verified-release-copy"
        extracted.write_bytes(OPTI)

        class StopBeforeProduct(Exception):
            pass

        # Model only the bundle verifier's output and stop at the first process
        # boundary. The driver's installed-vs-extracted byte comparison is real.
        with patch.object(cli, "prepare_release", return_value=(extracted, {})), \
                patch.object(cli, "command", side_effect=StopBeforeProduct) as command:
            with self.assertRaises(StopBeforeProduct):
                cli.run_pilot(candidate.flow, candidate.optiflow_bundle, output, None, installed=candidate)
        argv = command.call_args.args[0]
        self.assertEqual(argv[0], str(candidate.flow))
        self.assertEqual(argv[argv.index("--provider-executable") + 1], str(candidate.optiflow))
        self.assertNotIn(str(extracted), argv)
        self.assertFalse((output / "flow").exists())


if __name__ == "__main__":
    unittest.main()
