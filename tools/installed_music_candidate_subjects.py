#!/usr/bin/env python3
"""Closed four-tool local-candidate admission, separate from the three-tool v1.

The unchanged Flow package remains the Flow byte/build subject. A separately
pinned provider supplement selects current Aniflow/Renderflow candidates and the
actual Optiflow release. This observer grants no product or release authority.
"""
from pathlib import Path
import re

import installed_candidate_subjects as previous

cli, PACKAGE = previous.cli, previous.PACKAGE
TARGET = previous.TARGET
RENDER_ROW = previous.RENDER_ROW
ANI_ROW = "aniflow.music-video-source.source-candidate." + TARGET
PROVIDERS = ("aniflow", "renderflow", "optiflow")
FIELDS = {
    "schema_version", "target", "installation_root", "installation_receipt_sha256",
    "flow_package_directory", "flow_package_manifest_sha256", "flow_package_build_observation",
    "flow_package_build_observation_sha256", "flow_source_revision", "flow_source_tree",
    "flow_selected_source_revision", "flow_selected_source_tree",
    "flow_product_equivalence", "flow_product_equivalence_sha256",
    "provider_supplement", "provider_supplement_sha256", "optiflow_row_id", "optiflow_release_bundle",
    *{tool + suffix for tool in ("aniflow", "renderflow") for suffix in (
        "_source_revision", "_source_tree", "_binary_sha256", "_contract_sha256",
        "_candidate_observation", "_candidate_observation_sha256", "_row_id")},
}


def selection(value):
    cli.require(type(value) is dict and set(value) == FIELDS
        and all(type(v) is str and v for v in value.values()), "closed four-tool candidate selection required")
    cli.require(value["schema_version"] == "flow.installed-candidate-selection/v2"
        and value["target"] == TARGET and value["renderflow_row_id"] == RENDER_ROW
        and value["aniflow_row_id"] == ANI_ROW, "explicit four-tool candidate mode/profile required")
    for key, field in value.items():
        if key.endswith("sha256"):
            cli.require(re.fullmatch(r"[0-9a-f]{64}", field) is not None, "full selection digest required")
        elif key.endswith(("revision", "tree")):
            cli.require(re.fullmatch(r"[0-9a-f]{40}", field) is not None, "full selection source identity required")
    return dict(value)


def provider_projection(supplement, installed):
    fields = {"schema_version", "id", "flow_version", "targets", "retained_rows", "rows"}
    cli.require(type(supplement) is dict and type(installed) is dict and set(supplement) == fields == set(installed),
        "closed provider supplement and installed cohort required")
    cli.require(len(supplement["rows"]) == 3 and len(installed["rows"]) == 4
        and [r["tool"] for r in supplement["rows"]] == list(PROVIDERS)
        and [r["tool"] for r in installed["rows"]] == ["flow", *PROVIDERS]
        and len({r["id"] for r in installed["rows"]}) == 4,
        "exact three supplemented providers and one derived Flow row required")
    cli.require(supplement["schema_version"] == installed["schema_version"] == "flow.release-cohort/v1"
        and supplement["flow_version"] == installed["flow_version"]
        and supplement["targets"] == installed["targets"] == [TARGET]
        and installed["rows"][1:] == supplement["rows"]
        and supplement["retained_rows"] == [r["id"] for r in supplement["rows"]]
        and installed["retained_rows"] == [r["id"] for r in installed["rows"]],
        "supplemented provider rows/coverage changed during installation")


def captured_provider(observation, tool, value, binary, descriptor, version):
    # This is a supplied local capture record, not source authentication. The
    # actual installed byte identities and native handshake remain independent.
    data = cli.OWNER_IO.regular(binary, 83886080)
    cli.require(len(data) >= 64 and data[:6] == b"\x7fELF\x02\x01" and int.from_bytes(data[18:20], "little") == 62,
        "selected provider ELF CPU/class differs")
    cli.require(observation["schema_version"] == "flow.provider-candidate-capture/v1"
        and observation["tool"] == tool and observation["target"] == TARGET
        and observation["source"]["revision"] == value[tool + "_source_revision"]
        and observation["source"]["tree"] == value[tool + "_source_tree"]
        and observation["binary"] == previous.pin(binary)
        and observation["contract"] == {"identifier": tool + ".release-contract/" + ("v1" if tool == "aniflow" else "v2"),
            **previous.pin(descriptor)}
        and observation["version"] == version and observation["profile"] in ["release", "dev", "optimized-dev"]
        and type(observation["build_command"]) is list and observation["build_command"]
        and all(type(item) is str and item for item in observation["build_command"])
        and type(observation["rustc"]) is str and observation["rustc"].startswith("rustc ")
        and all(observation[flag] is False for flag in ["release_published", "publisher_authenticated", "native_matrix_qualified"]),
        "selected provider capture differs from installed local candidate")


class InstalledMusicCandidate(previous.InstalledCandidate):
    def __init__(self, value):
        self.selection = selection(value)
        self.selection_path = self.selection_sha256 = None
        self.root = previous.directory(value["installation_root"])
        self.flow, self.provider, self.aniflow, self.optiflow = [self.root / "bin" / name
            for name in ("flow", "renderflow", "aniflow", "optiflow")]
        self.cohort_path = self.root / "cohort.json"
        self.package_directory = previous.directory(value["flow_package_directory"])
        self.optiflow_bundle = Path(value["optiflow_release_bundle"])
        self.reobserve()

    def reobserve(self):
        value = self.selection
        if self.selection_path is not None:
            cli.require(previous.pin(self.selection_path, 65536)["sha256"] == self.selection_sha256,
                "admitted selection bytes changed")
        previous.read(self.package_directory / PACKAGE.MANIFEST, value["flow_package_manifest_sha256"])
        manifest = PACKAGE.verify(self.package_directory, value["flow_package_manifest_sha256"])
        cli.require(manifest["source_revision"] == value["flow_source_revision"] and manifest["target"] == TARGET
            and manifest["native_build"]["host_target"] == TARGET
            and type(manifest["native_build"]["isolated_builds"]) is int and manifest["native_build"]["isolated_builds"] == 2
            and manifest["native_build"]["rustc"].startswith("rustc 1.85.0 "), "original Flow package identity differs")
        build = previous.read(Path(value["flow_package_build_observation"]), value["flow_package_build_observation_sha256"])
        cli.require(build["schema_version"] == "flow.combined-package-build-observation/v1"
            and build["source"]["revision"] == value["flow_source_revision"] and build["source"]["tree"] == value["flow_source_tree"]
            and build["source"]["clean_before"] is True and build["source"]["clean_after"] is True
            and build["manifest_sha256"] == value["flow_package_manifest_sha256"]
            and build["native_build"]["profile"] == "release" and build["native_build"]["isolated_builds"] == 2
            and type(build["native_build"]["isolated_builds"]) is int
            and build["native_build"]["rustc"] == manifest["native_build"]["rustc"]
            and build["native_build"]["binary_equal"] is True and build["native_build"]["descriptor_equal"] is True,
            "original package build observation differs")
        equivalent = previous.read(Path(value["flow_product_equivalence"]), value["flow_product_equivalence_sha256"])
        cli.require(equivalent["schema_version"] == "flow.product-input-equivalence/v1"
            and equivalent["build_revision"] == value["flow_source_revision"] and equivalent["build_tree"] == value["flow_source_tree"]
            and equivalent["selected_revision"] == value["flow_selected_source_revision"]
            and equivalent["selected_tree"] == value["flow_selected_source_tree"]
            and equivalent["excluded_prefixes"] == ["docs/", "tools/"] and equivalent["product_changed_paths"] == []
            and equivalent["docs_or_tools_embedded_in_runtime"] is False
            and type(equivalent["changed_paths"]) is list
            and all(type(p) is str and p.startswith(("docs/", "tools/")) and ".." not in Path(p).parts for p in equivalent["changed_paths"]),
            "supplied Flow product-input equivalence differs")
        receipt = previous.read(self.root / "installation.json", value["installation_receipt_sha256"])
        cli.require(receipt["schema_version"] == "flow.suite-installation/v1" and receipt["root"] == cli.native_path(self.root)
            and receipt["target"] == TARGET and receipt["candidate_acknowledged"] is True
            and all(receipt[flag] is False for flag in ["provider_invoked", "publisher_authenticated", "native_qualified"])
            and receipt.get("publisher_admission") is None, "explicit four-tool candidate receipt required")
        intent = previous.read(self.root / "installation-intent.json", receipt["intent_sha256"])
        cli.require(intent["schema_version"] == "flow.suite-install-request/v1" and intent["target"] == TARGET
            and intent["install_directory"] == cli.native_path(self.root), "selected installation intent differs")
        self.cohort = previous.read(self.cohort_path, intent["cohort"]["sha256"], 1048576)
        supplement = previous.read(Path(value["provider_supplement"]), value["provider_supplement_sha256"], 1048576)
        provider_projection(supplement, self.cohort)
        cli.require(self.cohort["flow_version"] == manifest["version"] and len(intent["tools"]) == 4
            and {t["tool"] for t in intent["tools"]} == {"flow", *PROVIDERS}, "exact four-tool installed intent required")
        self.tools = {}
        for tool in intent["tools"]:
            name = tool["tool"]
            row = previous.one(self.cohort["rows"], lambda r: r["id"] == tool["row_id"], "unique installed row required")
            actual = previous.pin(self.root / "bin" / name)
            cli.require(row["tool"] == name and row["target"] == TARGET and row["implementation"] == "implemented"
                and row["interface"] == ("flow_route" if name in ["aniflow", "renderflow"] else "standalone_cli")
                and type(row["artifact"]["byte_count"]) is int and row["artifact"]["byte_count"] > 0
                and type(tool["executable"]["byte_count"]) is int
                and actual == {"sha256": row["artifact"]["sha256"], "byte_count": row["artifact"]["byte_count"]}
                and actual == {"sha256": tool["executable"]["sha256"], "byte_count": tool["executable"]["byte_count"]},
                "installed executable differs from intent/supplement")
            expected_ids = [] if name == "optiflow" else [name + ".release-contract/" + ("v2" if name == "renderflow" else "v1")]
            cli.require([c["identifier"] for c in tool["contracts"]] == expected_ids
                and [c["identifier"] for c in row["contracts"]] == expected_ids, "exact installed contract roles required")
            contracts = {}
            for index, contract in enumerate(tool["contracts"]):
                path = self.root / "contracts" / name / f"{index:02}.json"
                observed = previous.pin(path)
                cli.require(type(contract["file"]["byte_count"]) is int
                    and observed == {"sha256": contract["file"]["sha256"], "byte_count": contract["file"]["byte_count"]}
                    and observed["sha256"] == row["contracts"][index]["sha256"], "installed contract pin differs")
                previous.read(path); contracts[contract["identifier"]] = path
            self.tools[name] = {"row": row, "contracts": contracts}
        flow_asset = previous.one(manifest["assets"], lambda a: a["name"] == manifest["primary_binary"], "one packaged Flow asset required")
        self.flow_pin = previous.pin(self.flow)
        flow_bytes = cli.OWNER_IO.regular(self.flow, 83886080)
        cli.require(len(flow_bytes) >= 64 and flow_bytes[:6] == b"\x7fELF\x02\x01"
            and int.from_bytes(flow_bytes[18:20], "little") == 62, "selected Flow ELF CPU/class differs")
        cli.require(self.flow_pin == {"sha256": flow_asset["sha256"], "byte_count": flow_asset["byte_count"]}, "installed Flow differs from original package")
        flow_row = self.tools["flow"]["row"]
        cli.require(flow_row["artifact"]["source_revision"] == value["flow_source_revision"]
            and flow_row["artifact"]["version"] == manifest["version"]
            and flow_row["artifact"]["release_url"] == "https://github.com/egohygiene/flow/commit/" + value["flow_source_revision"]
            and flow_row["effects"] == ["read_originals", "write_new_outputs"]
            and cli.OWNER_IO.regular(self.tools["flow"]["contracts"]["flow.release-contract/v1"], 16777216)
                == cli.OWNER_IO.regular(self.package_directory / manifest["compiled_descriptor"], 16777216), "derived Flow row is not the original package subject")
        for name, profile, contract_id in [("renderflow", "canonical-build-v1", "renderflow.release-contract/v2"),
                                          ("aniflow", "music-video-source-v1", "aniflow.release-contract/v1")]:
            row = self.tools[name]["row"]; descriptor = self.tools[name]["contracts"][contract_id]
            cli.require(row["id"] == value[name + "_row_id"] and row["profile"] == profile
                and row["artifact"]["source_revision"] == value[name + "_source_revision"]
                and row["artifact"]["sha256"] == value[name + "_binary_sha256"]
                and row["artifact"]["release_url"] == "https://github.com/egohygiene/" + name + "/commit/" + value[name + "_source_revision"]
                and previous.pin(descriptor)["sha256"] == value[name + "_contract_sha256"], "explicit selected provider row differs")
            description = previous.read(descriptor)
            cli.require(description["schema_version"] == contract_id and description["tool"] == name
                and previous.semver(description["version"]) and description["version"] == row["artifact"]["version"]
                and any(p["id"] == profile and p["source_mutation"] is False for p in description["profiles"]), "selected provider profile/descriptor differs")
            observation = previous.read(Path(value[name + "_candidate_observation"]), value[name + "_candidate_observation_sha256"])
            captured_provider(observation, name, value, self.root / "bin" / name, descriptor, description["version"])
        self.descriptor = self.tools["renderflow"]["contracts"]["renderflow.release-contract/v2"]
        self.aniflow_descriptor = self.tools["aniflow"]["contracts"]["aniflow.release-contract/v1"]
        opti = self.tools["optiflow"]["row"]
        cli.require(opti["id"] == value["optiflow_row_id"] and opti["artifact"]["version"] == "0.1.1"
            and opti["artifact"]["source_revision"] == cli.PROVIDER_SOURCE and opti["artifact"]["sha256"] == cli.PROVIDER_SHA
            and previous.pin(self.optiflow)["sha256"] == cli.PROVIDER_SHA
            and previous.pin(self.optiflow_bundle, 33554432)["sha256"] == cli.BUNDLE_SHA, "actual Optiflow release differs")
        self.manifest, self.receipt, self.intent = manifest, receipt, intent
        def identity(name):
            return {"source_revision": value[name + "_source_revision"], "source_tree": value[name + "_source_tree"],
                "sha256": value[name + "_binary_sha256"], "contract_sha256": value[name + "_contract_sha256"],
                "candidate_observation_sha256": value[name + "_candidate_observation_sha256"], "target": TARGET,
                "publication": "not_performed", "qualification": "local candidate only"}
        self.render_identity, self.aniflow_identity = identity("renderflow"), identity("aniflow")
        self.flow_identity = {"schema_version": "flow.installed-package-observation/v1",
            "actual_build_revision": value["flow_source_revision"], "actual_build_tree": value["flow_source_tree"],
            "binary": {"sha256": self.flow_pin["sha256"], "bytes": self.flow_pin["byte_count"]}, "profile": "release",
            "package_manifest_sha256": value["flow_package_manifest_sha256"], "package_build_observation_sha256": value["flow_package_build_observation_sha256"],
            "installation_receipt_sha256": value["installation_receipt_sha256"], "cohort_sha256": intent["cohort"]["sha256"],
            "provider_supplement_sha256": value["provider_supplement_sha256"], "product_equivalence_sha256": value["flow_product_equivalence_sha256"],
            "selected_product_source_revision": value["flow_selected_source_revision"],
            "selected_product_source_tree": value["flow_selected_source_tree"],
            "scope": "unchanged old Flow package bytes with separately selected new provider candidates; no repackaging or qualification transfer"}
        return self
