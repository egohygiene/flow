#!/usr/bin/env python3
"""Bounded synthetic public-CLI proof for the actual Optiflow v0.1.1 route.

No network, installation, source-media mutation, signing or publication is selected.
The explicitly supplied release bundle is verified before one selected binary
is copied into the fresh private evidence root. This is a development CLI proof,
not qualification of an installed/released Flow suite or a filesystem sandbox.
"""
from __future__ import annotations

import argparse
import copy
from datetime import datetime, timezone
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import platform
import re
import selectors
import signal
import stat
import subprocess
import time
import tarfile

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("flow_owner_report_io", ROOT / "scripts/owner_reports.py")
OWNER_IO = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(OWNER_IO)
BUNDLE_SHA = "f576131f2695a218fabfaa5167fd16f17641c267194254a9e624303a8cf23e07"
ARCHIVE_SHA = "5e8a0eea84f5ab55fe75fc8a8abc2e9d36e3512537b754262b1f34a9c40aae31"
PROVIDER_SHA = "46af9399f3785607f835d9a805a8daa42c93597772c6ee812911615ea4b76280"
PROVIDER_SOURCE = "b82599a2231e997d42fd9f26f4b59587f4ae14cf"
TARGET = "x86_64-unknown-linux-gnu"
ARCHIVE_NAME = "optiflow-v0.1.1-" + TARGET + ".tar.gz"
MAX_OUTPUT = 1_048_576
FIXTURE = {"a 🍃.txt": b"same synthetic bytes\n", "b 🍃.txt": b"same synthetic bytes\n",
           "different.txt": b"other synthetic bytes\n"}
BUILD_COMMAND = ["cargo", "+1.85.0", "build", "--locked", "--offline", "--bin", "flow"]
REFUSALS = {
    "approval-required": ("approval_required", "exact supplied operation approval and unconfined-process acknowledgement required", False),
    "wrong-approval": ("approval_required", "exact supplied operation approval and unconfined-process acknowledgement required", False),
    "existing-run": ("recovery_required", "run output already exists; inspect its status or select a new directory", False),
    "marker-tampered": ("recovery_required", "operation completion differs from retained native acceptance", True),
    "overlap": ("invalid_input", "run output must be canonical and disjoint from source/provider", False),
    "wrong-provider": ("unavailable", "selected native adapter refused; inspect private retained evidence", False),
    "native-tampered": ("provider_refused", "selected native adapter refused; inspect private retained evidence", True),
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(",", ":"), allow_nan=False).encode()


def document(data):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, "duplicate JSON member")
            result[key] = value
        return result
    return json.loads(data, object_pairs_hook=unique,
                      parse_constant=lambda _: require(False, "finite JSON required"))


def native_path(path):
    return {"encoding": "utf8", "value": str(path)}


def public_build_identity(value):
    """Project public, typed identity only; supplied free-form/private data stays local."""
    require(type(value) is dict and value.get("schema_version") == "flow.local-build-observation/v1",
            "known local build-observation schema required")
    binary = value.get("binary")
    require(type(binary) is dict and type(binary.get("sha256")) is str
            and re.fullmatch(r"[0-9a-f]{64}", binary["sha256"]) is not None
            and type(binary.get("bytes")) is int and 0 < binary["bytes"] <= 83_886_080,
            "bounded public binary identity required")
    identity = {}
    for name in ("actual_build_revision", "actual_build_tree", "equivalent_remote_revision"):
        field = value.get(name)
        require(type(field) is str and re.fullmatch(r"[0-9a-f]{40}", field) is not None,
                "full public build/source identity required")
        identity[name] = field
    require(value.get("profile") == "dev" and type(value.get("profile")) is str
            and type(value.get("build_command")) is list and value["build_command"] == BUILD_COMMAND
            and all(type(item) is str for item in value["build_command"]), "selected public build command/profile required")
    return {"schema_version": "flow.local-build-observation/v1",
            "binary": {"sha256": binary["sha256"], "bytes": binary["bytes"]}, **identity,
            "profile": "dev", "build_command": list(BUILD_COMMAND),
            "authority": "supplied local build observation; not signed or independently reproduced"}


def build_binary(path):
    """Observe a compiler artifact, including Cargo's legitimate output hardlinks.

    The supplied bytes are copied into a fresh singly-linked file before exec;
    owner document admission remains stricter and is not changed by this helper.
    """
    require(path.is_absolute() and path.resolve(strict=True) == path, "canonical build artifact required")
    before = path.lstat()
    require(stat.S_ISREG(before.st_mode) and 0 < before.st_size <= 83_886_080, "bounded regular build artifact required")
    flags = os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK
    with os.fdopen(os.open(path, flags), "rb") as stream:
        opened = os.fstat(stream.fileno())
        data = stream.read(83_886_081)
        after_fd = os.fstat(stream.fileno())
    after = path.lstat()
    key = lambda value: (value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns,
                         value.st_ctime_ns, value.st_nlink, value.st_mode)
    require(len(data) == before.st_size and all(key(item) == key(before) for item in (opened, after_fd, after)),
            "build artifact changed while being copied")
    return data


def command(command, *, cwd, timeout=30, max_output=MAX_OUTPUT):
    """Separate bounded streams; kill/reap the selected process group on refusal."""
    require(os.name == "posix", "this local process supervision requires Unix")
    output = {"stdout": bytearray(), "stderr": bytearray()}
    deadline = time.monotonic() + timeout
    env = {"PATH": "/usr/bin:/bin", "LC_ALL": "C.UTF-8", "NO_COLOR": "1"}
    with subprocess.Popen(command, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True) as child:
        try:
            with selectors.DefaultSelector() as selector:
                selector.register(child.stdout, selectors.EVENT_READ, "stdout")
                selector.register(child.stderr, selectors.EVENT_READ, "stderr")
                while selector.get_map():
                    remaining = deadline - time.monotonic()
                    require(remaining > 0, "CLI deadline exceeded")
                    for key, _ in selector.select(min(remaining, 0.1)):
                        chunk = os.read(key.fileobj.fileno(), 65_536)
                        if not chunk:
                            selector.unregister(key.fileobj)
                            continue
                        require(len(output[key.data]) + len(chunk) <= max_output, "CLI output budget exceeded")
                        output[key.data].extend(chunk)
            code = child.wait(timeout=max(0.001, deadline - time.monotonic()))
            return code, bytes(output["stdout"]), bytes(output["stderr"])
        except BaseException:
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            child.wait()
            raise


def envelope(raw, stderr, code, expected_command, expected_code, expected_outcome):
    require(not stderr, "machine CLI emitted stderr")
    require(b"\x1b" not in raw, "machine CLI emitted ANSI")
    value = document(raw)
    require(type(value) is dict and set(value) == {"schema_version", "command", "outcome", "exit_code", "result", "diagnostics"},
            "closed command envelope required")
    require(value["schema_version"] == "flow.command-result/v1" and value["command"] == expected_command,
            "native command identity differs")
    require(type(value["exit_code"]) is int and code == expected_code == value["exit_code"] and value["outcome"] == expected_outcome,
            "native exit/outcome differs from the selected case")
    require(type(value["diagnostics"]) is list, "native diagnostics list required")
    return value


def case_diagnostics(value, case, run):
    """A same-exit unrelated failure cannot satisfy a named refusal control."""
    require(value["schema_version"] == "flow.command-result/v1", "Flow diagnostic domain required")
    if case not in REFUSALS:
        require(value["diagnostics"] == [] and type(value["result"]) is dict,
                "successful/typed-partial case returned unrelated diagnostics")
        return
    code, message, has_recovery = REFUSALS[case]
    diagnostics = value["diagnostics"]
    require(value["result"] is None and len(diagnostics) == 1 and type(diagnostics[0]) is dict
            and set(diagnostics[0]) == {"code", "message", "recovery_directory"}, "one closed Flow refusal diagnostic required")
    observed = diagnostics[0]
    require(observed["code"] == code and observed["message"] == message
            and observed["recovery_directory"] == (native_path(run) if has_recovery else None),
            "named refusal diagnostic code/reason/recovery subject differs")


def completed_status(value):
    require(value["schema_version"] == "flow.command-status/v1" and value["state"] == "complete"
            and value["operation_record_complete"] is True and value["source_content_reobserved"] is False
            and value["source_mutation_authority"] is False, "completed historical read-only status required")
    receipt = value["receipt"]
    require(receipt["schema_version"] == "flow.optiflow-read-only-receipt/v1" and receipt["status"] == "complete"
            and receipt["recovery"] == "none" and receipt["approval"] is None and receipt["physical_savings_bytes"] is None,
            "native receipt overstates authority or physical savings")
    require(receipt["provider"]["version"] == "0.1.1" and receipt["provider"]["source_revision"] == PROVIDER_SOURCE
            and receipt["provider"]["executable_sha256"] == PROVIDER_SHA
            and receipt["provider"]["archive_sha256"] == ARCHIVE_SHA and receipt["provider"]["target"] == TARGET,
            "native receipt is not the selected immutable release")
    require([entry["command"] for entry in receipt["commands"]] == ["scan", "report", "plan"]
            and all(entry["coverage"] == "complete" and entry["outcome"] == "success" for entry in receipt["commands"]),
            "native workload prefix is missing or incomplete")
    require(receipt["duplicate_logical_bytes"] == len(FIXTURE["a 🍃.txt"]), "known duplicate logical-byte result differs")
    require(receipt["environment"] == "cleared; no config; no media probe; no network grant", "native environment authority differs")
    for name in ("dry-run", "quarantine", "restore", "finalize"):
        require(receipt["capabilities"][name] == "unsupported-by-pinned-release", "unreleased mutation capability became available")
    return receipt


def retained_receipt(value, root, expected):
    """Bind the final locator to this run and the already validated decoded receipt."""
    require(canonical(completed_status(value)) == canonical(expected), "final receipt claim changed")
    locator = value.get("receipt_path")
    require(type(locator) is dict and set(locator) == {"encoding", "value"}
            and locator["encoding"] == "utf8" and type(locator["value"]) is str,
            "closed selected UTF-8 receipt locator required")
    path = Path(locator["value"])
    require(path.is_absolute() and ".." not in path.parts and path.is_relative_to(root)
            and path.parent == root / "flow-optiflow-v0.1.1" and path.suffix == ".json",
            "receipt locator escaped selected run/namespace")
    raw = OWNER_IO.regular(path, 16_777_216)
    require(canonical(document(raw)) == canonical(expected), "retained receipt bytes decode to a different native receipt")
    return {"sha256": sha(raw), "bytes": len(raw), "relative_path": path.relative_to(root).as_posix()}


def native_artifacts(receipt, root):
    """Rehash actual provider files independently of the CLI's completion claim."""
    observed = {}
    expected = {"scan": {"effective_policy", "report", "run", "artifact_set"},
                "report": {"effective_policy", "report", "run", "artifact_set"},
                "plan": {"plan", "artifact_set"}}
    for entry in receipt["commands"]:
        records = entry["artifacts"]
        require(len(records) == len(expected[entry["command"]])
                and {record["kind"] for record in records} == expected[entry["command"]],
                "native artifact roles are missing or duplicated")
        for record in records:
            path = record["path"]
            require(path.get("encoding") == "utf8", "selected fixture uses exact UTF-8 artifact paths")
            path = Path(path["value"])
            require(path.is_relative_to(root), "native artifact escaped fixture state")
            data = OWNER_IO.regular(path, 16_777_216)
            require(len(data) == record["size_bytes"] and sha(data) == record["sha256"],
                    "actual native artifact differs from the claimed bytes")
            relative = path.relative_to(root).as_posix()
            pin = {"sha256": sha(data), "bytes": len(data)}
            require(relative not in observed or observed[relative] == pin, "native artifact claims conflict")
            observed[relative] = pin
    return observed


def inventory(root, *, omit=()):
    result = {}
    entries = sorted(root.rglob("*"))
    require(len(entries) <= 128, "synthetic evidence entry budget exceeded")
    total = 0
    for path in entries:
        relative = path.relative_to(root).as_posix()
        if relative in omit:
            continue
        metadata = path.lstat()
        require(not stat.S_ISLNK(metadata.st_mode), "synthetic evidence contains an alias")
        if path.is_dir():
            continue
        data = OWNER_IO.regular(path, 16_777_216)
        total += len(data)
        require(total <= 33_554_432, "synthetic evidence byte budget exceeded")
        result[relative] = {"sha256": sha(data), "bytes": len(data), "device": metadata.st_dev,
                            "inode": metadata.st_ino, "mtime_ns": metadata.st_mtime_ns}
    return result


def selected_member(data, name, maximum, maximum_members):
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as archive:
        entries = archive.getmembers()
        require(len(entries) <= maximum_members, "release archive inventory exceeded")
        selected = [entry for entry in entries if entry.name.removeprefix("./") == name]
        require(len(selected) == 1 and selected[0].isfile() and 0 < selected[0].size <= maximum,
                "selected release member missing, linked, duplicated or oversized")
        with archive.extractfile(selected[0]) as stream:
            result = stream.read(maximum + 1)
        require(len(result) == selected[0].size, "truncated selected release member")
        return result


def prepare_release(bundle, destination):
    data = OWNER_IO.regular(bundle, 16_777_216)
    require(sha(data) == BUNDLE_SHA, "published release bundle digest differs")
    archive = selected_member(data, ARCHIVE_NAME, 8_388_608, 32)
    require(sha(archive) == ARCHIVE_SHA, "published target archive digest differs")
    provenance = document(selected_member(data, "provenance.json", 65_536, 32))
    build = provenance["predicate"]["buildDefinition"]
    require(build["externalParameters"]["releaseVersion"] == "v0.1.1"
            and any(item["uri"] == "git+https://github.com/egohygiene/optiflow.git"
                    and item["digest"].get("gitCommit") == PROVIDER_SOURCE for item in build["resolvedDependencies"])
            and any(item["name"] == ARCHIVE_NAME and item["digest"].get("sha256") == ARCHIVE_SHA for item in provenance["subject"]),
            "embedded source/target provenance declarations differ")
    binary = selected_member(archive, "optiflow", 16_777_216, 8)
    require(sha(binary) == PROVIDER_SHA, "released executable digest differs")
    require(binary[:6] == b"\x7fELF\x02\x01" and int.from_bytes(binary[18:20], "little") == 62, "Linux x86-64 ELF required")
    destination.mkdir(mode=0o700)
    executable = destination / "optiflow"
    with executable.open("xb") as stream:
        stream.write(binary)
    executable.chmod(0o700)
    return executable, {"version": "0.1.1", "source_revision": PROVIDER_SOURCE, "target": TARGET,
                         "bundle_sha256": BUNDLE_SHA, "archive_sha256": ARCHIVE_SHA,
                         "executable_sha256": PROVIDER_SHA, "executable_bytes": len(binary),
                         "embedded_provenance_checked": True, "signature_verification_performed": False}


def run_pilot(flow, bundle, output, flow_provenance):
    require(platform.system() == "Linux" and platform.machine() in {"x86_64", "amd64"}, "selected real release requires Linux x86-64")
    require(output.is_absolute() and output.parent.resolve(strict=True) == output.parent, "canonical fresh evidence parent required")
    flow_provenance = public_build_identity(flow_provenance)
    supplied_flow = flow
    flow_bytes = build_binary(flow)
    flow_identity = {"sha256": sha(flow_bytes), "bytes": len(flow_bytes)}
    require(flow_identity == flow_provenance["binary"], "Flow binary differs from supplied build evidence")
    output.mkdir(mode=0o700)
    logs = output / "commands"; logs.mkdir(mode=0o700)
    flow = output / "flow"
    with flow.open("xb") as stream:
        stream.write(flow_bytes)
    flow.chmod(0o700)
    provider, release = prepare_release(bundle, output / "provider")
    source = output / "source 作品 with spaces"; source.mkdir(mode=0o700)
    for name, data in FIXTURE.items():
        (source / name).write_bytes(data)
    originals = inventory(source)
    cases, observations = [], []
    started = time.monotonic()

    def invoke(case, args, code=0, outcome="complete"):
        remaining = 150 - (time.monotonic() - started)
        require(remaining > 0, "pilot aggregate deadline exceeded")
        result, stdout, stderr = command([str(flow), *args, "--format", "json"], cwd=output, timeout=min(30, remaining))
        (logs / (case + ".stdout.json")).write_bytes(stdout)
        (logs / (case + ".stderr")).write_bytes(stderr)
        value = envelope(stdout, stderr, result, args[0], code, outcome)
        case_diagnostics(value, case, output / "run")
        observations.append({"case": case, "command": args[0], "exit_code": result, "outcome": outcome,
                             "stdout_sha256": sha(stdout), "stdout_bytes": len(stdout), "stderr_bytes": len(stderr),
                             "diagnostic_codes": [item["code"] for item in value["diagnostics"]]})
        return value["result"]

    doctor = invoke("doctor-release", ["doctor", "--provider-executable", str(provider)], 9, "partial")
    require(doctor["release_qualified"] is False and doctor["provider_invoked"] is False
            and doctor["released_optiflow"]["selected_provider"]["executable_sha256"] == PROVIDER_SHA,
            "doctor did not retain matched bytes/unqualified suite distinction")
    cases.append("doctor-matched-release-suite-partial")
    observed = invoke("inspect", ["inspect", "--input", str(source)])
    require(observed["domain_acceptance"] is False and observed["execution_input_freeze"] is False
            and observed["total_bytes"] == sum(map(len, FIXTURE.values()))
            and len(observed["files"]) == 3, "inspection promoted octet evidence or missed fixture inputs")
    expected = {(len(data), sha(data)) for data in FIXTURE.values()}
    require({(entry["byte_count"], entry["sha256"]) for entry in observed["files"]} == expected,
            "inspection byte identities differ")
    cases.append("unicode-spaces-exact-inspection")
    plan_path, approval_path, run = output / "plan.json", output / "approval.json", output / "run"
    planned = invoke("plan", ["plan", "--input", str(source), "--profile", "optiflow-read-only",
                              "--provider-executable", str(provider), "--run-directory", str(run), "--output", str(plan_path)], 9, "partial")
    require(planned["schema_version"] == "flow.local-input-plan/v1" and planned["availability"] == "scope_plan_only"
            and planned["reason"] == "explicit whole-directory read-only scope; preview hashes/limits are not execution input freezing or provider budget enforcement"
            and planned["inspection_limits_apply_to_native_execution"] is False
            and planned["command_plan"] == document(OWNER_IO.regular(plan_path, 32_768)) and not run.exists(),
            "scope plan became execution or an input-freezing claim")
    cases.append("partial-scope-plan-no-provider-work")
    invoke("approval-required", ["approve", "--plan", str(plan_path), "--approved-by", "synthetic read-only operator",
                                 "--output", str(approval_path)], 5, "refused")
    require(not approval_path.exists() and not run.exists(), "missing acknowledgement wrote approval/run state")
    cases.append("missing-acknowledgement-refusal")
    approval = invoke("approve", ["approve", "--plan", str(plan_path), "--approved-by", "synthetic read-only operator",
                                  "--acknowledge-trusted-unconfined", "--output", str(approval_path)])
    require(approval["scope"] == "selected_read_only_scan_and_local_evidence_only", "approval widens the selected read-only scope")
    cases.append("explicit-synthetic-read-only-approval")
    wrong = copy.deepcopy(approval); wrong["plan_sha256"] = "0" * 64
    wrong_path = output / "wrong-approval.json"; wrong_path.write_bytes(canonical(wrong))
    invoke("wrong-approval", ["run", "--plan", str(plan_path), "--approval", str(wrong_path)], 5, "refused")
    require(not run.exists(), "mismatched approval created an attempt")
    cases.append("mismatched-plan-approval-refusal")
    completed = invoke("run", ["run", "--plan", str(plan_path), "--approval", str(approval_path)])
    receipt = completed_status(completed)
    receipt_pin = retained_receipt(completed, run, receipt)
    accepted_artifacts = native_artifacts(receipt, run)
    cases.append("real-native-scan-report-plan")
    before = inventory(run)
    require(completed_status(invoke("status", ["status", "--run-directory", str(run)])) == receipt,
            "status changed the retained native attempt")
    require(completed_status(invoke("resume-complete", ["resume", "--run-directory", str(run)])) == receipt
            and inventory(run) == before, "completed resume changed native evidence or replayed work")
    cases.append("status-completed-resume-no-replay")
    invoke("existing-run", ["run", "--plan", str(plan_path), "--approval", str(approval_path)], 7, "recovery_required")
    require(inventory(run) == before, "existing run was overwritten")
    cases.append("existing-run-no-clobber")
    marker = run / "operation-result.json"; marker_bytes = OWNER_IO.regular(marker, 16_384)
    marker.rename(output / "retained-marker.json")
    native_before = inventory(run)
    incomplete = invoke("marker-missing", ["status", "--run-directory", str(run)], 7, "recovery_required")
    require(incomplete["schema_version"] == "flow.command-status/v1" and incomplete["state"] == "recovery_required"
            and incomplete["operation_record_complete"] is False and incomplete["source_mutation_authority"] is False
            and incomplete["source_content_reobserved"] is False and incomplete["receipt"] == receipt
            and incomplete["next_action"] == "inspect retained native state; no automatic replay",
            "missing outer marker became completed")
    require(completed_status(invoke("marker-reconcile", ["resume", "--run-directory", str(run)])) == receipt
            and OWNER_IO.regular(marker, 16_384) == marker_bytes and inventory(run, omit=("operation-result.json",)) == native_before,
            "metadata reconciliation replayed native work or changed the completion subject")
    cases.append("missing-marker-reconciliation-without-native-replay")
    altered = document(marker_bytes); altered["receipt_sha256"] = "0" * 64
    marker.write_bytes(canonical(altered))
    invoke("marker-tampered", ["status", "--run-directory", str(run)], 7, "recovery_required")
    marker.write_bytes(marker_bytes)
    cases.append("changed-completion-marker-refusal")
    request = copy.deepcopy(planned["command_plan"]["request"])
    request["run_directory"] = native_path(source / "forbidden-state")
    request_path = output / "overlap.json"; request_path.write_bytes(canonical(request))
    overlap_plan = output / "overlap-plan.json"
    invoke("overlap", ["plan", "--request", str(request_path), "--output", str(overlap_plan)], 3, "refused")
    require(not (source / "forbidden-state").exists() and not overlap_plan.exists(), "overlapping state was prepared")
    cases.append("source-state-overlap-refusal")
    bogus = output / "unreleased-provider"; bogus.write_bytes(b"not the pinned Optiflow release\n")
    request["provider_executable"] = native_path(bogus); request["run_directory"] = native_path(output / "wrong-provider-run")
    request_path = output / "wrong-provider.json"; request_path.write_bytes(canonical(request))
    refused_plan = output / "wrong-provider-plan.json"
    invoke("wrong-provider", ["plan", "--request", str(request_path), "--output", str(refused_plan)], 4, "unavailable")
    require(not refused_plan.exists(), "incompatible provider created a runnable plan")
    cases.append("incompatible-provider-refusal")
    unbroadened = output / "single-file-plan.json"
    single = invoke("single-file", ["plan", "--input", str(source / "a 🍃.txt"), "--profile", "optiflow-read-only",
                                   "--provider-executable", str(provider), "--run-directory", str(output / "single-run"),
                                   "--output", str(unbroadened)], 4, "unavailable")
    require(single["schema_version"] == "flow.local-input-plan/v1" and single["availability"] == "unavailable"
            and single["reason"] == "pinned Optiflow adapter requires whole recursive directory scope; file/ordered/nonrecursive requests cannot be broadened"
            and single["command_plan"] is None and not unbroadened.exists(),
            "single file silently broadened into recursive provider scope")
    cases.append("single-file-no-silent-scope-broadening")
    native_plan = Path(receipt["commands"][2]["artifacts"][0]["path"]["value"])
    require(native_plan.is_relative_to(run), "selected native plan escaped fixture state")
    raw = OWNER_IO.regular(native_plan, 16_777_216)
    native_plan.write_bytes(raw + b" ")
    invoke("native-tampered", ["status", "--run-directory", str(run)], 6, "refused")
    native_plan.write_bytes(raw)
    cases.append("changed-native-artifact-refusal")
    require(inventory(source) == originals, "synthetic originals changed")
    final_status = invoke("final-status", ["status", "--run-directory", str(run)])
    require(completed_status(final_status) == receipt,
            "restored exact fixture evidence no longer validates")
    require(retained_receipt(final_status, run, receipt) == receipt_pin, "final selected receipt identity changed")
    require(native_artifacts(receipt, run) == accepted_artifacts, "accepted native artifact bytes changed")
    require(sha(OWNER_IO.regular(flow, 83_886_080)) == flow_identity["sha256"]
            and sha(build_binary(supplied_flow)) == flow_identity["sha256"]
            and sha(OWNER_IO.regular(provider, 16_777_216)) == PROVIDER_SHA, "selected executable changed during the pilot")
    cases.append("originals-and-selected-binaries-unchanged")
    return {"schema_version": "flow.released-read-only-cli-pilot/v1", "status": "passed",
            "observed_utc": datetime.now(timezone.utc).isoformat(),
            "scope": "Linux synthetic read-only public CLI route; not installed-suite qualification",
            "flow_build": flow_provenance, "provider_release": release,
            "driver_sha256": sha(OWNER_IO.regular(Path(__file__).resolve(), 131_072)),
            "elapsed_seconds": round(time.monotonic() - started, 6),
            "cases": [{"id": case, "state": "passed"} for case in cases], "commands": observations,
            "fixture": {"files": len(FIXTURE), "logical_bytes": sum(map(len, FIXTURE.values())),
                        "duplicate_logical_bytes": len(FIXTURE["a 🍃.txt"]), "originals_unchanged": True},
            "native_receipt_sha256": receipt_pin["sha256"],
            "native_plan_sha256": receipt["plan_sha256"], "physical_savings_bytes": None,
            "accepted_native_artifacts": accepted_artifacts,
            "source_mutation_authority": False, "human_creative_approval": False,
            "flow_release_qualified": False, "full_native_platform_qualified": False,
            "signature_verification_performed": False, "hosted_ci_dispatched": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--flow", type=Path, required=True)
    parser.add_argument("--provider-bundle", type=Path, required=True)
    parser.add_argument("--flow-build-evidence", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    provenance = document(OWNER_IO.regular(args.flow_build_evidence, 65_536))
    result = run_pilot(args.flow, args.provider_bundle, args.output, provenance)
    with (args.output / "report.json").open("x", encoding="utf-8") as file:
        json.dump(result, file, ensure_ascii=False, indent=2, allow_nan=False); file.write("\n")
    print(json.dumps({"status": "passed", "cases": len(result["cases"]), "report": str(args.output / "report.json")}))


if __name__ == "__main__":
    main()
