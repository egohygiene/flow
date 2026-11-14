#!/usr/bin/env python3
"""Focused private installation proof through existing suite-* CLI owners.

Only fresh test-owned copies are modified. No installer implementation,
publisher verification, provider workload, global activation or publication.
"""
from __future__ import annotations
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import platform

from check_released_read_only import (OWNER_IO, canonical, command, document,
    envelope, native_path, require, sha)

TARGET = "x86_64-unknown-linux-gnu"


def put(path, value):
    with path.open("xb") as writer:
        writer.write(canonical(value) + b"\n")


def pin(path):
    data = OWNER_IO.regular(path, 83_886_080)
    return {"path": native_path(path), "sha256": sha(data), "byte_count": len(data)}


def run_proof(args):
    root = args.output.absolute()
    require(root.parent.resolve() == root.parent and not root.exists(), "fresh canonical proof root required")
    root.mkdir(mode=0o700)
    cwd = root / "empty cwd 作品"; cwd.mkdir(mode=0o700)
    logs = root / "commands"; logs.mkdir(mode=0o700)
    sources = root / "sources"; sources.mkdir(mode=0o700)
    selected = {"flow": args.flow, "flow-contract": args.descriptor,
                "renderflow": args.renderflow, "renderflow-contract": args.render_descriptor,
                "optiflow": args.optiflow}
    original = {name: pin(path) for name, path in selected.items()}
    require(original["flow"]["sha256"] == args.flow_sha256, "explicit Flow package pin differs")
    copies = {}
    for name, path in selected.items():
        data = OWNER_IO.regular(path, 83_886_080)
        destination = sources / name
        with destination.open("xb") as writer: writer.write(data)
        destination.chmod(0o700 if name in {"flow", "renderflow", "optiflow"} else 0o600)
        copies[name] = destination
    cohort = document(OWNER_IO.regular(args.provider_cohort, 1_048_576))
    descriptor = document(OWNER_IO.regular(copies["flow-contract"], 1_048_576))
    require(descriptor["schema_version"] == "flow.release-contract/v1"
            and descriptor["qualification"] == "not_established_by_contract_description", "unqualified compiled Flow descriptor required")
    require(type(cohort["rows"]) is list and len(cohort["rows"]) == 2
            and len({row["tool"] for row in cohort["rows"]}) == 2
            and len({row["id"] for row in cohort["rows"]}) == 2,
            "exact two distinct provider rows required before projection")
    rows = {row["tool"]: row for row in cohort["rows"]}
    require(set(rows) == {"renderflow", "optiflow"}, "exact selected provider rows required")
    for tool in rows:
        require(rows[tool]["artifact"]["sha256"] == pin(copies[tool])["sha256"]
                and rows[tool]["artifact"]["byte_count"] == pin(copies[tool])["byte_count"], "provider cohort byte binding differs")
    rows["flow"] = {"id": "flow.local-package." + TARGET, "tool": "flow", "interface": "standalone_cli",
        "target": TARGET, "profile": "private-installed-candidate", "input": "explicit local requests",
        "output": "bounded CLI results", "effects": ["read_originals", "write_new_outputs"],
        "implementation": "implemented", "artifact": {"version": descriptor["version"],
        "source_revision": args.source_revision, "name": "flow-" + TARGET,
        "sha256": original["flow"]["sha256"], "byte_count": original["flow"]["byte_count"],
        "release_url": "https://github.com/egohygiene/flow/commit/" + args.source_revision},
        "contracts": [{"identifier": "flow.release-contract/v1", "sha256": original["flow-contract"]["sha256"]}],
        "requirements": [], "owner_issue": "https://github.com/egohygiene/flow/issues/71",
        "next_action": "local package copy only; publication authentication/native matrix remain unestablished"}
    cohort = {"schema_version": "flow.release-cohort/v1", "id": "isolated-suite-proof",
        "flow_version": descriptor["version"], "targets": [TARGET],
        "retained_rows": [rows[name]["id"] for name in ("flow", "renderflow", "optiflow")],
        "rows": [rows[name] for name in ("flow", "renderflow", "optiflow")]}
    cohort_path = sources / "cohort.json"; put(cohort_path, cohort)
    selection = root / "selected-suite.json"
    a, b = root / "installed suite A 作品", root / "installed suite B 作品"
    tools = []
    for name in ("flow", "renderflow", "optiflow"):
        contracts = [] if name == "optiflow" else [{"identifier": rows[name]["contracts"][0]["identifier"],
            "file": pin(copies[name + "-contract"])}]
        tools.append({"tool": name, "row_id": rows[name]["id"], "executable": pin(copies[name]),
                      "contracts": contracts, "extras": []})
    request = {"schema_version": "flow.suite-install-request/v1", "cohort": pin(cohort_path),
        "target": TARGET, "install_directory": native_path(a), "tools": tools,
        "timeout_ms": 30_000, "maximum_total_bytes": 50_000_000}
    request_path = root / "install-request.json"; put(request_path, request)
    invoker = copies["flow"]; cases = []

    def call(case, operation, options, code=0, diagnostic=None, message=None, executable=None):
        process_code, stdout, stderr = command([str(executable or invoker), operation, *map(str, options),
            "--format", "json"], cwd=cwd, timeout=40)
        (logs / (case + ".stdout.json")).write_bytes(stdout)
        (logs / (case + ".stderr")).write_bytes(stderr)
        outcome = "complete" if code == 0 else ("recovery_required" if code == 7 else "refused")
        value = envelope(stdout, stderr, process_code, operation, code, outcome)
        if code:
            require(value["result"] is None and len(value["diagnostics"]) == 1
                    and value["diagnostics"][0]["code"] == diagnostic
                    and value["diagnostics"][0]["message"] == message,
                    "exact refusal code/reason differs: " + case)
        else:
            require(value["diagnostics"] == [], "successful command has diagnostics")
        cases.append({"case": case, "command": operation, "exit_code": code,
                      "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr)})
        return value["result"]

    call("candidate-ack-required", "suite-install", ["--request", request_path], 5, "approval_required",
         "explicit unqualified candidate installation acknowledgement required")
    require(not a.exists(), "refused install created a destination")
    first = call("install-a", "suite-install", ["--request", request_path, "--acknowledge-unqualified-candidate"])
    receipt_a = first["receipt_sha256"]
    require(first["current_copied_bytes"] and first["receipt"]["candidate_acknowledged"] is True
            and all(first["receipt"][field] is False for field in ("provider_invoked", "native_qualified", "publisher_authenticated")),
            "installation scope was promoted")
    invoker = a / "bin/flow"
    status_options = lambda install, digest: ["--installation-root", install, "--receipt-sha256", digest]
    call("installed-status-a", "suite-status", status_options(a, receipt_a))
    call("receipt-pin-refusal", "suite-status", status_options(a, "0" * 64), 3, "stale_plan", "installation receipt pin differs")
    require(call("installed-contract", "release-contract", []) == descriptor, "packaged/installed descriptor differs")
    call("installed-help", "help", []); call("installed-version", "version", [])
    for name, expected_version in (("renderflow", b"renderflow 0.3.0-rc.1\n"), ("optiflow", b"optiflow 0.1.1\n")):
        code, stdout, stderr = command([str(a / "bin" / name), "--version"], cwd=cwd)
        require(code == 0 and not stderr and stdout == expected_version, "exact installed provider version differs")
        (logs / (name + "-version.stdout")).write_bytes(stdout)
    bundle = root / "offline-bundle"
    bundle_pin = call("offline-bundle", "suite-bundle", ["--request", request_path, "--output-directory", bundle])
    moved = root / "moved offline bundle 作品"; bundle.rename(moved)
    second = call("install-moved-bundle-b", "suite-install", ["--bundle", moved,
        "--bundle-sha256", bundle_pin["sha256"], "--install-directory", b, "--acknowledge-unqualified-candidate"])
    receipt_b = second["receipt_sha256"]
    selected_a = call("activate-a", "suite-activate", [*status_options(a, receipt_a), "--selection", selection])
    selected_b = call("activate-b", "suite-activate", [*status_options(b, receipt_b), "--selection", selection,
        "--previous-selection-sha256", selected_a["sha256"]])
    call("stale-selection-refusal", "suite-rollback", [*status_options(a, receipt_a), "--selection", selection,
        "--previous-selection-sha256", selected_a["sha256"]], 3, "stale_plan",
        "activation changed; explicit prior pointer digest required")
    require(pin(selection)["sha256"] == selected_b["sha256"], "stale rollback changed selection")
    restored = call("rollback-a", "suite-rollback", [*status_options(a, receipt_a), "--selection", selection,
        "--previous-selection-sha256", selected_b["sha256"]])
    require(restored["sha256"] == selected_a["sha256"], "rollback did not restore exact selected root")
    target = b / "bin/renderflow"; original_copy = OWNER_IO.regular(target, 83_886_080)
    altered = bytearray(original_copy); altered[-1] ^= 1; target.write_bytes(altered)
    call("changed-copy-refusal", "suite-status", status_options(b, receipt_b), 3, "stale_plan",
        "native pinned bytes changed; create a new reviewed plan")
    # Explicit test fixture restoration only; the product performs no repair.
    target.write_bytes(original_copy)
    unknown = b / "operator-note.txt"; unknown.write_bytes(b"synthetic unknown material; preserve\n")
    removal = root / "remove-b.json"
    call("unknown-member-refusal", "suite-uninstall", [*status_options(b, receipt_b), "--removal-record", removal],
         7, "recovery_required", "installation has missing or unknown files/directories; preserve them")
    require(unknown.read_bytes().startswith(b"synthetic unknown") and not removal.exists()
            and pin(target)["sha256"] == original["renderflow"]["sha256"], "refusal removed/changed material")
    unknown.unlink()  # Delete only the test harness's own generated canary.
    call("owned-only-uninstall-b", "suite-uninstall", [*status_options(b, receipt_b), "--removal-record", removal])
    require(not b.exists() and removal.exists() and a.is_dir(), "owned uninstall affected the wrong scope")
    call("retained-status-a", "suite-status", status_options(a, receipt_a))
    accepted = {name: a / "bin" / name for name in ("flow", "renderflow", "optiflow")}
    accepted.update({"flow-contract": a / "contracts/flow/00.json",
                     "renderflow-contract": a / "contracts/renderflow/00.json"})
    require(all(pin(path)["sha256"] == original[name]["sha256"]
                and pin(path)["byte_count"] == original[name]["byte_count"]
                for name, path in accepted.items()), "retained installed binaries/contracts differ")
    require(list(cwd.iterdir()) == [], "empty outside-checkout working directory changed")
    require({name: pin(path) for name, path in selected.items()} == original, "original candidate inputs changed")
    report = {"schema_version": "flow.isolated-suite-installation-proof/v1", "status": "passed",
        "observed_at": datetime.now(timezone.utc).isoformat(), "flow_source_revision": args.source_revision,
        "target": TARGET, "host": {"system": platform.system(), "machine": platform.machine(), "libc": platform.libc_ver()},
        "inputs": {name: {"sha256": p["sha256"], "byte_count": p["byte_count"]} for name, p in original.items()},
        "cases": cases, "receipt_a_sha256": receipt_a, "receipt_b_sha256": receipt_b,
        "selection_restored": True, "offline_bundle_moved": True, "owned_b_removed": True,
        "a_retained": True, "retained_binaries_and_contracts_rehashed": True,
        "empty_command_cwd_preserved": True, "original_assets_preserved": True, "command_cwd_outside_checkout": True,
        "release_published": False, "publisher_authenticated": False, "native_matrix_qualified": False,
        "provider_workload_invoked": False, "hosted_ci_dispatched": False,
        "limits": "same-version candidate root selection/rollback; no OS integration, authenticated installation or media qualification"}
    put(root / "report.json", report)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("flow", "descriptor", "renderflow", "render-descriptor", "optiflow", "provider-cohort", "output"):
        parser.add_argument("--" + name, required=True, type=Path)
    parser.add_argument("--flow-sha256", required=True)
    parser.add_argument("--source-revision", required=True)
    args = parser.parse_args()
    print(json.dumps(run_proof(args), sort_keys=True))


if __name__ == "__main__":
    main()
