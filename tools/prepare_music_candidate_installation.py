#!/usr/bin/env python3
"""Prepare a four-tool test installation through the existing suite-* CLI.

This is observation/fixture assembly, not an installer. The old Flow package is
never edited; three separately pinned provider rows form a candidate supplement.
"""
import argparse
import copy
from pathlib import Path

from check_native_publication import pin, put, refusal
from check_released_read_only import OWNER_IO, canonical, command, document, envelope, native_path, require, sha
from installed_music_candidate_subjects import (ANI_ROW, RENDER_ROW, TARGET,
    InstalledMusicCandidate, PROVIDERS, PACKAGE)


def prepare(config, output, *, candidate_type=InstalledMusicCandidate, extra_provider_rows=()):
    require(output.is_absolute() and output.parent.resolve() == output.parent and not output.exists(),
        "fresh canonical installation evidence root required")
    output.mkdir(mode=0o700)
    cwd = output / "empty cwd 作品"; cwd.mkdir(mode=0o700)
    logs = output / "commands"; logs.mkdir(mode=0o700)
    package = Path(config["flow_package_directory"])
    manifest = PACKAGE.verify(package, config["flow_package_manifest_sha256"])
    flow = package / manifest["primary_binary"]
    descriptor = package / manifest["compiled_descriptor"]
    # Data preparation never substitutes a new Flow build for this package.
    original_paths = {"flow": flow, "flow-contract": descriptor}
    original = {name: pin(path) for name, path in original_paths.items()}
    selections = copy.deepcopy(config)
    provider_rows = []
    tools = []
    for name in PROVIDERS:
        row = copy.deepcopy(config["provider_rows"][name])
        binary = Path(config["provider_files"][name]["binary"])
        require({k: row["artifact"][k] for k in ["sha256", "byte_count"]}
                == {k: pin(binary)[k] for k in ["sha256", "byte_count"]}, "provider row/input bytes differ")
        contracts = []
        if name != "optiflow":
            contract = Path(config["provider_files"][name]["contract"])
            contracts = [{"identifier": row["contracts"][0]["identifier"], "file": pin(contract)}]
            original[name + "-contract"] = pin(contract)
            original_paths[name + "-contract"] = contract
        original[name] = pin(binary)
        original_paths[name] = binary
        provider_rows.append(row)
        tools.append({"tool": name, "row_id": row["id"], "executable": pin(binary), "contracts": contracts, "extras": []})
    provider_rows.extend(copy.deepcopy(list(extra_provider_rows)))
    supplement = {"schema_version": "flow.release-cohort/v1", "id": "music-video-candidate-providers",
        "flow_version": manifest["version"], "targets": [TARGET],
        "retained_rows": [row["id"] for row in provider_rows], "rows": provider_rows}
    supplement_path = output / "provider-supplement.json"; put(supplement_path, supplement)
    compiled = document(OWNER_IO.regular(descriptor, 1048576))
    row = {"id": "flow.local-package." + TARGET, "tool": "flow", "interface": "standalone_cli",
        "target": TARGET, "profile": "private-installed-candidate", "input": "explicit local requests",
        "output": "bounded CLI results", "effects": ["read_originals", "write_new_outputs"],
        "implementation": "implemented", "artifact": {"version": compiled["version"],
            "source_revision": config["flow_source_revision"], "name": "flow-" + TARGET,
            "sha256": original["flow"]["sha256"], "byte_count": original["flow"]["byte_count"],
            "release_url": "https://github.com/egohygiene/flow/commit/" + config["flow_source_revision"]},
        "contracts": [{"identifier": "flow.release-contract/v1", "sha256": original["flow-contract"]["sha256"]}],
        "requirements": [], "owner_issue": "https://github.com/egohygiene/flow/issues/71",
        "next_action": "old packaged Flow identity; independently selected candidate providers; unqualified/unpublished"}
    cohort = {**supplement, "id": "installed-music-video-candidate", "rows": [row, *provider_rows],
        "retained_rows": [row["id"], *supplement["retained_rows"]]}
    candidate_type.projection_checker(supplement, cohort)
    cohort_path = output / "selected-cohort.json"; put(cohort_path, cohort)
    tools.insert(0, {"tool": "flow", "row_id": row["id"], "executable": pin(flow),
        "contracts": [{"identifier": "flow.release-contract/v1", "file": pin(descriptor)}], "extras": []})
    installation = output / "installed suite 作品"
    request = {"schema_version": "flow.suite-install-request/v1", "cohort": pin(cohort_path), "target": TARGET,
        "install_directory": native_path(installation), "tools": tools,
        "timeout_ms": 30000, "maximum_total_bytes": 134217728}
    request_path = output / "install-request.json"; put(request_path, request)
    observations = []

    def invoke(label, operation, args, code=0, diagnostic=None, executable=flow):
        actual, stdout, stderr = command([str(executable), operation, *map(str, args), "--format", "json"], cwd=cwd, timeout=40)
        (logs / (label + ".stdout.json")).write_bytes(stdout); (logs / (label + ".stderr")).write_bytes(stderr)
        value = envelope(stdout, stderr, actual, operation, code, "complete" if not code else "refused")
        if diagnostic: refusal(value, diagnostic)
        else: require(value["diagnostics"] == [], "unexpected installation diagnostic")
        observations.append({"case": label, "command": operation, "exit_code": actual, "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr)})
        return value["result"]

    invoke("acknowledgement-required", "suite-install", ["--request", request_path], 5,
        ("approval_required", "explicit unqualified candidate installation acknowledgement required", None))
    require(not installation.exists(), "refused installation created destination")
    result = invoke("install", "suite-install", ["--request", request_path, "--acknowledge-unqualified-candidate"])
    for key in ["provider_rows", "provider_files"]: selections.pop(key)
    selections.update({"schema_version": candidate_type.SELECTION_SCHEMA, "target": TARGET,
        "installation_root": str(installation), "installation_receipt_sha256": result["receipt_sha256"],
        "provider_supplement": str(supplement_path), "provider_supplement_sha256": pin(supplement_path)["sha256"]})
    selection_path = output / ("selection-" + candidate_type.SELECTION_SCHEMA.rsplit("/", 1)[1] + ".json")
    put(selection_path, selections)
    subject = candidate_type.load(selection_path)
    status = subject.observe_status(output, "installed-four-tool-status")
    require(status["current_copied_bytes"] is True, "four-tool installation is not current")
    require(not list(cwd.iterdir()), "installation command working directory changed")
    require({name: pin(path) for name, path in original_paths.items()} == original,
        "installation changed original candidate inputs")
    report = {"schema_version": "flow.music-candidate-installation-observation/v1", "status": "passed",
        "commands": observations, "selection_sha256": pin(selection_path)["sha256"],
        "installation_receipt_sha256": selections["installation_receipt_sha256"],
        "original_flow_package_manifest_sha256": selections["flow_package_manifest_sha256"],
        "provider_supplement_sha256": selections["provider_supplement_sha256"],
        "flow_build": subject.flow_identity, "providers": {"aniflow": subject.aniflow_identity, "renderflow": subject.render_identity},
        "scope": "existing packaged Flow bytes plus separately selected unsealed/unpublished candidate provider supplement; no new Flow package",
        "provider_workload_invoked": False, "release_published": False, "publisher_authenticated": False, "native_matrix_qualified": False}
    put(output / "report.json", report)
    return subject, report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inputs", type=Path, required=True); parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    _, report = prepare(document(OWNER_IO.regular(args.inputs, 1048576)), args.output)
    print(canonical(report).decode())
