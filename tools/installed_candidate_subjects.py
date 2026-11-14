#!/usr/bin/env python3
"""Admit explicit local installed candidates for the existing proof drivers.

This observer grants no product, publisher, release or native-matrix authority.
The package verifier and suite-status remain their existing independent owners.
"""
from __future__ import annotations

import importlib.util
from pathlib import Path
import re

import check_released_read_only as cli

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("flow_package_proof_owner", ROOT / "scripts/release/package.py")
PACKAGE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PACKAGE)
TARGET = "x86_64-unknown-linux-gnu"
RENDER_ROW = "renderflow.canonical-build.source-candidate." + TARGET
IMAGE = "ghcr.io/catthehacker/ubuntu:act-24.04@sha256:19619a010b204f44c5340bff7d39b68551109fe6a56a53f75201ea91e05d4270"
FIELDS = {"schema_version", "target", "installation_root", "installation_receipt_sha256",
    "flow_package_directory", "flow_package_manifest_sha256", "flow_package_build_observation",
    "flow_package_build_observation_sha256", "flow_source_revision", "flow_source_tree",
    "renderflow_source_revision", "renderflow_source_tree", "renderflow_binary_sha256",
    "renderflow_contract_sha256", "renderflow_candidate_proof", "renderflow_candidate_proof_sha256",
    "renderflow_row_id", "optiflow_row_id", "optiflow_release_bundle"}


def selection(value):
    cli.require(type(value) is dict and set(value) == FIELDS,
                "closed installed-candidate selection required")
    cli.require(all(type(v) is str and v for v in value.values()), "string selection fields required")
    cli.require(value["schema_version"] == "flow.installed-candidate-selection/v1"
        and value["target"] == TARGET and value["renderflow_row_id"] == RENDER_ROW,
        "selected installed candidate profile differs")
    for name, field in value.items():
        if name.endswith("sha256"):
            cli.require(re.fullmatch(r"[0-9a-f]{64}", field) is not None, "full selection digest required")
        elif name.endswith(("revision", "tree")):
            cli.require(re.fullmatch(r"[0-9a-f]{40}", field) is not None, "full selection source identity required")
    return dict(value)


def directory(value):
    path = Path(value)
    cli.require(path.is_absolute() and path.resolve(strict=True) == path and path.is_dir(),
                "canonical existing selected directory required")
    return path


def pin(path, maximum=536_870_912):
    data = cli.OWNER_IO.regular(path, maximum)
    return {"sha256": cli.sha(data), "byte_count": len(data)}


def read(path, expected=None, maximum=16_777_216):
    raw = cli.OWNER_IO.regular(path, maximum)
    cli.require(expected is None or cli.sha(raw) == expected, "selected document digest differs")
    return cli.document(raw)


def one(values, predicate, message):
    rows = [value for value in values if predicate(value)]
    cli.require(len(rows) == 1, message)
    return rows[0]


def semver(value):
    return type(value) is str and len(value) <= 96 and re.fullmatch(
        r"[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?", value) is not None


def provider_projection(packaged, installed):
    """The existing installer adds one Flow self row after package preparation."""
    keys = {"schema_version", "id", "flow_version", "targets", "retained_rows", "rows"}
    cli.require(type(packaged) is dict and type(installed) is dict
        and set(packaged) == keys == set(installed), "closed package/installation cohorts required")
    cli.require(len(packaged["rows"]) == 2
        and {row["tool"] for row in packaged["rows"]} == {"renderflow", "optiflow"}
        and len(installed["rows"]) == 3
        and {row["tool"] for row in installed["rows"]} == {"flow", "renderflow", "optiflow"},
        "exact two packaged providers and one derived Flow row required")
    cli.require(packaged["schema_version"] == installed["schema_version"] == "flow.release-cohort/v1"
        and packaged["flow_version"] == installed["flow_version"]
        and packaged["targets"] == installed["targets"] == [TARGET]
        and [row for row in installed["rows"] if row["tool"] != "flow"] == packaged["rows"]
        and packaged["retained_rows"] == [row["id"] for row in packaged["rows"]]
        and installed["retained_rows"] == [row["id"] for row in installed["rows"]],
        "installed provider rows/coverage differ from package projection")


class InstalledCandidate:
    """Captured exact local selection; reobservation precedes each proof route."""
    def __init__(self, value):
        self.selection = selection(value)
        self.selection_path = None
        self.selection_sha256 = None
        self.root = directory(value["installation_root"])
        self.flow = self.root / "bin/flow"
        self.provider = self.root / "bin/renderflow"
        self.optiflow = self.root / "bin/optiflow"
        self.cohort_path = self.root / "cohort.json"
        self.package_directory = directory(value["flow_package_directory"])
        self.optiflow_bundle = Path(value["optiflow_release_bundle"])
        self.reobserve()

    @classmethod
    def load(cls, path):
        raw = cli.OWNER_IO.regular(path, 65_536)
        candidate = cls(cli.document(raw))
        candidate.selection_path = path
        candidate.selection_sha256 = cli.sha(raw)
        candidate.reobserve()
        return candidate

    def reobserve(self):
        value = self.selection
        if self.selection_path is not None:
            cli.require(pin(self.selection_path, 65_536)["sha256"] == self.selection_sha256,
                        "admitted selection bytes changed")
        read(self.package_directory / PACKAGE.MANIFEST, value["flow_package_manifest_sha256"])
        manifest = PACKAGE.verify(self.package_directory, value["flow_package_manifest_sha256"])
        cli.require(manifest["target"] == TARGET and manifest["source_revision"] == value["flow_source_revision"]
            and semver(manifest["version"]) and manifest["native_build"]["host_target"] == TARGET
            and type(manifest["native_build"]["isolated_builds"]) is int
            and manifest["native_build"]["isolated_builds"] == 2
            and manifest["native_build"]["rustc"].startswith("rustc 1.85.0 "),
            "selected exact-source Flow release preparation differs")
        build = read(Path(value["flow_package_build_observation"]), value["flow_package_build_observation_sha256"])
        cli.require(build["schema_version"] == "flow.combined-package-build-observation/v1"
            and build["source"]["revision"] == value["flow_source_revision"]
            and build["source"]["tree"] == value["flow_source_tree"]
            and build["source"]["clean_before"] is True and build["source"]["clean_after"] is True
            and build["manifest_sha256"] == value["flow_package_manifest_sha256"]
            and build["native_build"]["profile"] == "release"
            and type(build["native_build"]["isolated_builds"]) is int and build["native_build"]["isolated_builds"] == 2
            and build["native_build"]["rustc"] == manifest["native_build"]["rustc"]
            and build["native_build"]["binary_equal"] is True
            and build["native_build"]["descriptor_equal"] is True,
            "explicit package build observation differs from selected source/tree/manifest")
        assets = manifest["assets"]
        flow_asset = one(assets, lambda a: a["name"] == manifest["primary_binary"], "one Flow binary asset required")
        self.flow_pin = pin(self.flow)
        flow_bytes = cli.OWNER_IO.regular(self.flow, 83_886_080)
        cli.require(len(flow_bytes) >= 64 and flow_bytes[:6] == b"\x7fELF\x02\x01"
            and int.from_bytes(flow_bytes[18:20], "little") == 62,
            "selected installed Flow ELF CPU/class differs")
        cli.require(self.flow_pin == {"sha256": flow_asset["sha256"], "byte_count": flow_asset["byte_count"]},
                    "installed Flow differs from selected package")
        receipt = read(self.root / "installation.json", value["installation_receipt_sha256"])
        cli.require(receipt["schema_version"] == "flow.suite-installation/v1"
            and receipt["root"] == cli.native_path(self.root) and receipt["target"] == TARGET
            and receipt["candidate_acknowledged"] is True and receipt["provider_invoked"] is False
            and receipt["publisher_authenticated"] is False and receipt["native_qualified"] is False
            and receipt.get("publisher_admission") is None, "explicit unqualified installation receipt required")
        intent = read(self.root / "installation-intent.json", receipt["intent_sha256"])
        cli.require(intent["schema_version"] == "flow.suite-install-request/v1" and intent["target"] == TARGET
            and intent["install_directory"] == cli.native_path(self.root),
                    "selected installation intent differs")
        self.cohort = read(self.cohort_path, intent["cohort"]["sha256"], 1_048_576)
        packaged_cohort = read(self.package_directory / manifest["provider_cohort"], maximum=1_048_576)
        provider_projection(packaged_cohort, self.cohort)
        cli.require(self.cohort["schema_version"] == "flow.release-cohort/v1"
            and self.cohort["flow_version"] == manifest["version"], "installed cohort version differs from package")
        tools = intent["tools"]
        cli.require(len(tools) == 3 and {t["tool"] for t in tools} == {"flow", "renderflow", "optiflow"},
                    "exact three-tool installed selection required")
        cli.require(len({row["id"] for row in self.cohort["rows"]}) == len(self.cohort["rows"]),
                    "duplicate installed cohort row")
        self.tools = {}
        for tool in tools:
            name = tool["tool"]
            row = one(self.cohort["rows"], lambda r: r["id"] == tool["row_id"], "unique selected cohort row required")
            observed = pin(self.root / "bin" / name)
            cli.require(row["tool"] == name and row["target"] == TARGET
                and row["implementation"] == "implemented"
                and row["interface"] == ("flow_route" if name == "renderflow" else "standalone_cli")
                and type(row["artifact"]["byte_count"]) is int and row["artifact"]["byte_count"] > 0
                and type(tool["executable"]["byte_count"]) is int
                and observed == {"sha256": row["artifact"]["sha256"], "byte_count": row["artifact"]["byte_count"]}
                and observed == {"sha256": tool["executable"]["sha256"], "byte_count": tool["executable"]["byte_count"]},
                "installed executable/intent/cohort differ")
            contracts = {}
            cli.require(len(tool["contracts"]) == len(row["contracts"]), "installed contract cardinality differs")
            for index, contract in enumerate(tool["contracts"]):
                path = self.root / "contracts" / name / f"{index:02}.json"
                observed_contract = pin(path, 16_777_216)
                read(path)
                declared = one(row["contracts"], lambda c: c["identifier"] == contract["identifier"],
                               "unique cohort contract required")
                cli.require(contract["identifier"] not in contracts
                    and type(contract["file"]["byte_count"]) is int
                    and observed_contract == {"sha256": contract["file"]["sha256"], "byte_count": contract["file"]["byte_count"]}
                    and observed_contract["sha256"] == declared["sha256"], "installed contract pin differs")
                contracts[contract["identifier"]] = path
            self.tools[name] = {"row": row, "contracts": contracts}
        flow_row = self.tools["flow"]["row"]
        cli.require(flow_row["artifact"]["source_revision"] == value["flow_source_revision"]
            and flow_row["artifact"]["version"] == manifest["version"]
            and flow_row["artifact"]["release_url"] == "https://github.com/egohygiene/flow/commit/" + value["flow_source_revision"]
            and flow_row["effects"] == ["read_originals", "write_new_outputs"], "Flow cohort source/version/effects differ")
        flow_descriptor = self.tools["flow"]["contracts"]["flow.release-contract/v1"]
        cli.require(cli.OWNER_IO.regular(flow_descriptor, 16_777_216)
            == cli.OWNER_IO.regular(self.package_directory / manifest["compiled_descriptor"], 16_777_216),
            "installed Flow descriptor differs from package")
        render = self.tools["renderflow"]
        row = render["row"]
        cli.require(row["id"] == value["renderflow_row_id"] and row["interface"] == "flow_route"
            and row["profile"] == "canonical-build-v1"
            and row["artifact"]["source_revision"] == value["renderflow_source_revision"]
            and row["artifact"]["sha256"] == value["renderflow_binary_sha256"]
            and row["artifact"]["release_url"] == "https://github.com/egohygiene/renderflow/commit/" + value["renderflow_source_revision"],
            "explicit installed Renderflow candidate row differs")
        self.descriptor = render["contracts"]["renderflow.release-contract/v2"]
        cli.require(pin(self.descriptor)["sha256"] == value["renderflow_contract_sha256"],
                    "selected Renderflow contract differs")
        descriptor = read(self.descriptor)
        cli.require(descriptor["schema_version"] == "renderflow.release-contract/v2" and descriptor["tool"] == "renderflow"
            and semver(descriptor["version"])
            and descriptor["version"] == row["artifact"]["version"]
            and any(p["id"] == "canonical-build-v1" and p["source_mutation"] is False for p in descriptor["profiles"]),
            "native Renderflow contract/profile differs")
        proof = read(Path(value["renderflow_candidate_proof"]), value["renderflow_candidate_proof_sha256"])
        cli.require(proof["schema_version"] == "renderflow.ubuntu-candidate-summary/v1"
            and proof["source"]["revision"] == value["renderflow_source_revision"]
            and proof["source"]["tree"] == value["renderflow_source_tree"] and proof["target"] == TARGET
            and proof["image"]["reference"] == IMAGE and proof["baseline"]["libc"] == "glibc 2.39"
            and proof["baseline"]["rustc"].startswith("rustc 1.94.0 ")
            and proof["release_authenticated"] is False and proof["supported_matrix_qualified"] is False
            and proof["publication_performed"] is False, "selected Ubuntu candidate observation differs")
        binary_asset = one(proof["assets"], lambda a: a["name"] == "renderflow-" + TARGET, "one Ubuntu CLI asset required")
        contract_asset = one(proof["assets"], lambda a: a["name"] == "renderflow-" + TARGET + ".contract.json",
                             "one Ubuntu contract asset required")
        cli.require(binary_asset["sha256"] == value["renderflow_binary_sha256"]
            and type(binary_asset["bytes"]) is int and type(contract_asset["bytes"]) is int
            and binary_asset["bytes"] == row["artifact"]["byte_count"]
            and contract_asset["sha256"] == value["renderflow_contract_sha256"]
            and contract_asset["bytes"] == pin(self.descriptor)["byte_count"]
            and proof["local_smoke"]["schema_version"] == "renderflow.local-candidate-smoke/v1"
            and proof["local_smoke"]["qualification"] == "local_captured_binary_smoke_only"
            and proof["local_smoke"]["binary"] == {"sha256": value["renderflow_binary_sha256"],
                "bytes": binary_asset["bytes"], "target": TARGET}
            and proof["local_smoke"]["checks"]["version"] == descriptor["version"]
            and proof["local_smoke"]["checks"]["version_help_doctor"] == "passed"
            and proof["local_smoke"]["checks"]["spec_plan_dry_run"] == "passed"
            and proof["local_smoke"]["checks"]["missing_config_refused"] is True
            and all(proof["local_smoke"][flag] is False for flag in ["source_authenticated",
                "release_authenticated", "supported_host_qualified", "native_matrix_qualified", "publication_performed"]),
            "installed Renderflow differs from the selected Ubuntu bytes")
        opti = self.tools["optiflow"]["row"]
        cli.require(opti["id"] == value["optiflow_row_id"] and opti["artifact"]["version"] == "0.1.1"
            and opti["artifact"]["source_revision"] == cli.PROVIDER_SOURCE
            and opti["artifact"]["sha256"] == cli.PROVIDER_SHA
            and pin(self.optiflow)["sha256"] == cli.PROVIDER_SHA
            and pin(self.optiflow_bundle, 33_554_432)["sha256"] == cli.BUNDLE_SHA,
            "actual immutable Optiflow release selection differs")
        self.manifest = manifest
        self.receipt = receipt
        self.intent = intent
        self.render_identity = {"version": descriptor["version"], "source_revision": value["renderflow_source_revision"],
            "source_tree": value["renderflow_source_tree"], "sha256": value["renderflow_binary_sha256"],
            "contract_sha256": value["renderflow_contract_sha256"], "target": TARGET,
            "candidate_proof_sha256": value["renderflow_candidate_proof_sha256"], "publication": "not_performed"}
        self.flow_identity = {"schema_version": "flow.installed-package-observation/v1",
            "actual_build_revision": value["flow_source_revision"], "actual_build_tree": value["flow_source_tree"],
            "binary": {"sha256": self.flow_pin["sha256"], "bytes": self.flow_pin["byte_count"]}, "profile": "release",
            "normalized_build_command": ["cargo", "build", "--frozen", "--release", "--bin", "flow", "--target", TARGET],
            "rust_toolchain": "1.85.0", "host_target": TARGET, "isolated_native_builds": 2,
            "package_manifest_sha256": value["flow_package_manifest_sha256"],
            "package_build_observation_sha256": value["flow_package_build_observation_sha256"],
            "installation_receipt_sha256": value["installation_receipt_sha256"],
            "cohort_sha256": intent["cohort"]["sha256"],
            "packaged_provider_cohort_sha256": pin(self.package_directory / manifest["provider_cohort"])["sha256"],
            "authority": "local package and installed-byte observation; publisher/release/native qualification unestablished"}
        return self

    def observe_status(self, output, label):
        self.reobserve()
        code, stdout, stderr = cli.command([str(self.flow), "suite-status", "--installation-root", str(self.root),
            "--receipt-sha256", self.selection["installation_receipt_sha256"], "--format", "json"], cwd=output, timeout=60,
            max_output=16_777_216)
        value = cli.envelope(stdout, stderr, code, "suite-status", 0, "complete")
        result = value["result"]
        cli.require(value["diagnostics"] == [] and result["schema_version"] == "flow.suite-installation-status/v1"
            and result["current_copied_bytes"] is True
            and result["receipt"] == self.receipt
            and result["receipt_sha256"] == self.selection["installation_receipt_sha256"],
            "installed suite status does not reobserve selected bytes")
        (output / (label + ".stdout.json")).write_bytes(stdout)
        (output / (label + ".stderr")).write_bytes(stderr)
        return {"command": "suite-status", "exit_code": 0, "current_copied_bytes": True,
                "stdout_sha256": cli.sha(stdout), "installation_receipt_sha256": self.selection["installation_receipt_sha256"]}
