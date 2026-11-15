#!/usr/bin/env python3
"""Read-only native Mac trial preflight; builds/installations keep existing owners.

plan reads only the explicit request. preflight observes clean source/toolchain
prerequisites. observe additionally checks an existing suite-install request's
local Mach-O/cohort/contract bytes, without executing products or installing.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import selectors
import signal
import stat
import subprocess
import sys
import time
import tomllib

SCHEMA = "flow.macos-native-trial-request/v1"
TARGETS = {"aarch64-apple-darwin": 0x100000C, "x86_64-apple-darwin": 0x1000007}
TOOLCHAINS = {"flow": "1.85.0", "renderflow": "1.94.0", "aniflow": "1.85.0"}
OWNERS = {"flow": "scripts/release/package.py", "renderflow": "scripts/release/native_cohort.py", "aniflow": "scripts/release.py"}
CONTRACTS = {"flow": "flow.release-contract/v1", "renderflow": "renderflow.release-contract/v2", "aniflow": "aniflow.release-contract/v1"}
MAX_JSON = 1_048_576
MAX_BINARY = 134_217_728


class Refusal(ValueError):
    pass


def require(condition, code):
    if not condition:
        raise Refusal(code)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def exact_keys(value, required, optional=()):
    require(type(value) is dict and set(required) <= set(value) <= set(required) | set(optional), "document_shape_differs")


def identity(value, length):
    require(type(value) is str and re.fullmatch("[0-9a-f]{" + str(length) + "}", value), "identity_shape_differs")


def path(value):
    require(type(value) is str and "\x00" not in value, "canonical_path_required")
    result = Path(value)
    require(result.is_absolute() and result.resolve() == result, "canonical_path_required")
    return result


def read(filename, maximum=MAX_JSON):
    filename = path(str(filename))
    descriptor = os.open(filename, os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW)
    try:
        before = os.fstat(descriptor)
        require(stat.S_ISREG(before.st_mode) and before.st_nlink == 1 and 0 < before.st_size <= maximum, "bounded_single_link_regular_input_required")
        with os.fdopen(descriptor, "rb", closefd=False) as stream:
            data = stream.read(maximum + 1)
        after = os.fstat(descriptor)
        stamp = lambda m: (m.st_dev, m.st_ino, m.st_mode, m.st_nlink, m.st_size, m.st_mtime_ns, m.st_ctime_ns)
        require(stamp(before) == stamp(after) == stamp(filename.lstat()) and len(data) == before.st_size, "input_changed_during_observation")
        return data
    finally:
        os.close(descriptor)


def document(data):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, "duplicate_json_member")
            result[key] = value
        return result
    return json.loads(data, object_pairs_hook=pairs,
                      parse_constant=lambda _: require(False, "nonfinite_json"))


def filepin(value):
    exact_keys(value, ("path", "sha256", "byte_count"))
    identity(value["sha256"], 64)
    require(type(value["byte_count"]) is int and 0 < value["byte_count"] <= MAX_BINARY, "file_pin_size_differs")
    native = value["path"]
    exact_keys(native, ("encoding", "value"))
    require(native["encoding"] == "utf8", "utf8_native_path_required")
    return path(native["value"])


def pinned(value, maximum=MAX_JSON):
    data = read(filepin(value), maximum)
    require(len(data) == value["byte_count"] and sha(data) == value["sha256"], "selected_file_pin_differs")
    return data


def request(data):
    value = document(data)
    exact_keys(value, ("schema_version", "target", "sources"), ("install_request",))
    require(value["schema_version"] == SCHEMA and value["target"] in TARGETS, "mac_trial_profile_differs")
    exact_keys(value["sources"], TOOLCHAINS)
    for source in value["sources"].values():
        exact_keys(source, ("directory", "revision", "tree", "cargo_lock_sha256"))
        path(source["directory"])
        for name in ("revision", "tree"):
            identity(source[name], 40)
        identity(source["cargo_lock_sha256"], 64)
    require(len({s["directory"] for s in value["sources"].values()}) == 3, "distinct_source_roots_required")
    if "install_request" in value:
        filepin(value["install_request"])
    return value


def command(argv, cwd):
    """Drain one bounded stream; no raw output or environment enters reports."""
    environment = dict(os.environ)
    environment.update({"RUSTUP_AUTO_INSTALL": "0", "CARGO_NET_OFFLINE": "true", "GIT_OPTIONAL_LOCKS": "0", "PYTHONDONTWRITEBYTECODE": "1"})
    with subprocess.Popen(argv, cwd=cwd, env=environment, stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT, start_new_session=True) as process:
        deadline, data = time.monotonic() + 30, bytearray()
        try:
            with selectors.DefaultSelector() as selector:
                selector.register(process.stdout, selectors.EVENT_READ)
                while selector.get_map():
                    require(time.monotonic() < deadline, "prerequisite_query_timeout")
                    for key, _ in selector.select(min(0.2, max(0, deadline - time.monotonic()))):
                        chunk = os.read(key.fileobj.fileno(), 65536)
                        if not chunk:
                            selector.unregister(key.fileobj)
                        data.extend(chunk)
                        require(len(data) <= MAX_JSON, "prerequisite_query_output_bound")
            require(process.wait(timeout=max(0.01, deadline - time.monotonic())) == 0, "prerequisite_query_failed")
            return bytes(data)
        finally:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            if process.poll() is None:
                process.wait()


def source_check(tool, selected, target):
    root = path(selected["directory"])
    git = ["git", "-c", "core.hooksPath=/dev/null", "--no-optional-locks"]
    require(command(git + ["rev-parse", "HEAD"], root).decode().strip() == selected["revision"], "source_revision_differs")
    require(command(git + ["rev-parse", "HEAD^{tree}"], root).decode().strip() == selected["tree"], "source_tree_differs")
    require(not command(git + ["status", "--porcelain", "--untracked-files=all"], root), "source_not_clean")
    require(sha(read(root / "Cargo.lock", 16_777_216)) == selected["cargo_lock_sha256"], "source_cargo_lock_differs")
    read(root / OWNERS[tool])
    cargo = tomllib.loads(read(root / "Cargo.toml").decode())
    package = cargo["workspace"]["package"] if tool == "renderflow" else cargo["package"]
    require(type(package["version"]) is str, "cargo_authority_missing")
    if tool == "aniflow":
        config = document(read(root / ".egohygiene/release-build.json"))
        require(target in {v["triple"] for v in config["targets"]}, "aniflow_target_owner_unavailable")
        require(config["rust_toolchain"] == TOOLCHAINS[tool], "owner_toolchain_differs")
        candidate_path = root / ".egohygiene/release-candidate.json"
        require(candidate_path.is_file(), "aniflow_reviewed_successor_required")
        candidate = document(read(candidate_path))
        require(candidate.get("version") == "v" + package["version"], "aniflow_candidate_cargo_differs")
        # Keep the repository's full reviewed-successor/Cargo/changelog gate.
        result = document(command([sys.executable, "-B", str(root / OWNERS[tool]), "verify", "--version", candidate["version"], "--source-revision", selected["revision"]], root))
        require(result.get("candidate") == candidate and result.get("qualification") == "not-established", "aniflow_owner_verification_differs")
    elif tool == "flow":
        config = document(read(root / ".egohygiene/release-build.json"))
        require(target in config["native_targets"] and config["rust_toolchain"] == TOOLCHAINS[tool], "flow_target_or_toolchain_differs")
    return {"revision": selected["revision"], "tree": selected["tree"], "cargo_lock_sha256": selected["cargo_lock_sha256"], "version": package["version"]}


def mach_o(data, target):
    require(len(data) >= 32 and data[:4] == b"\xcf\xfa\xed\xfe" and int.from_bytes(data[4:8], "little") == TARGETS[target]
            and int.from_bytes(data[12:16], "little") == 2, "native_mach_o_executable_cpu_required")


def observe_install(value):
    """Observe selection bytes only; the product distribution owner admits/install them."""
    selected = document(pinned(value["install_request"]))
    exact_keys(selected, ("schema_version", "cohort", "target", "install_directory", "tools", "timeout_ms", "maximum_total_bytes"))
    require(selected["schema_version"] == "flow.suite-install-request/v1" and selected["target"] == value["target"], "installation_target_differs")
    require(type(selected["timeout_ms"]) is int and 0 < selected["timeout_ms"] <= 3_600_000
            and type(selected["maximum_total_bytes"]) is int and 0 < selected["maximum_total_bytes"] <= 8_589_934_592, "installation_limits_differ")
    deadline = time.monotonic() + min(selected["timeout_ms"] / 1000, 120)
    tools = selected["tools"]
    require(type(tools) is list and len(tools) == 3 and all(type(t) is dict for t in tools), "exact_three_trial_tools_required")
    require({t.get("tool") for t in tools} == set(TOOLCHAINS), "exact_three_trial_tools_required")
    files = [selected["cohort"]]
    for tool in tools:
        exact_keys(tool, ("tool", "row_id", "executable", "contracts", "extras"))
        require(type(tool["contracts"]) is list and len(tool["contracts"]) == 1 and tool["extras"] == [], "exact_compiled_contract_required")
        exact_keys(tool["contracts"][0], ("identifier", "file"))
        files.extend([tool["executable"], tool["contracts"][0]["file"]])
    for pin in files:
        filepin(pin)
    require(sum(pin["byte_count"] for pin in files) <= selected["maximum_total_bytes"], "installation_aggregate_byte_bound")
    cohort = document(pinned(selected["cohort"]))
    require(cohort.get("schema_version") == "flow.release-cohort/v1" and cohort.get("targets") == [value["target"]], "cohort_target_differs")
    rows = cohort.get("rows")
    require(type(rows) is list and len(rows) == 3 and all(type(r) is dict for r in rows), "exact_three_cohort_rows_required")
    require(len({r.get("id") for r in rows}) == 3 and {r.get("tool") for r in rows} == set(TOOLCHAINS)
            and cohort.get("retained_rows") == [r["id"] for r in rows], "cohort_membership_differs")
    result = {}
    for tool in tools:
        require(time.monotonic() < deadline, "installation_observation_deadline")
        exact_keys(tool, ("tool", "row_id", "executable", "contracts", "extras"))
        name = tool["tool"]
        row = next(r for r in rows if r["tool"] == name)
        require(tool["row_id"] == row["id"] and row["target"] == value["target"] and row["interface"] == "standalone_cli"
                and row["implementation"] == "implemented", "selected_cohort_row_differs")
        binary = pinned(tool["executable"], MAX_BINARY)
        mach_o(binary, value["target"])
        artifact = row["artifact"]
        require(artifact["sha256"] == sha(binary) and artifact["byte_count"] == len(binary)
                and artifact["source_revision"] == value["sources"][name]["revision"], "artifact_source_or_bytes_differ")
        require(type(tool["contracts"]) is list and len(tool["contracts"]) == 1 and not tool["extras"], "exact_compiled_contract_required")
        contract = tool["contracts"][0]
        exact_keys(contract, ("identifier", "file"))
        identifier = CONTRACTS[name]
        content = pinned(contract["file"])
        descriptor = document(content)
        require(contract["identifier"] == identifier and descriptor.get("schema_version") == identifier
                and descriptor.get("version") == artifact["version"], "compiled_contract_identity_differs")
        require(row["contracts"] == [{"identifier": identifier, "sha256": sha(content)}], "cohort_contract_pin_differs")
        result[name] = {"binary_sha256": sha(binary), "byte_count": len(binary), "contract_sha256": sha(content), "source_revision": artifact["source_revision"], "version": artifact["version"]}
    require(time.monotonic() < deadline, "installation_observation_deadline")
    return result


def host_target(system, machine):
    if system == "Darwin":
        return {"arm64": "aarch64-apple-darwin", "aarch64": "aarch64-apple-darwin", "x86_64": "x86_64-apple-darwin"}.get(machine)
    return None


def report(action, value, digest):
    host = host_target(platform.system(), platform.machine().lower())
    result = {"schema_version": "flow.macos-native-trial-report/v1", "action": action,
              "request_sha256": digest, "target": value["target"], "host_target": host,
              "state": "planned", "checks": [], "sources": {}, "artifacts": {},
              "source_locks": {name: {field: source[field] for field in ("revision", "tree", "cargo_lock_sha256")} for name, source in value["sources"].items()},
              "native_execution": "not_executed_by_this_helper", "native_qualification": "not_established",
              "build_performed": False, "installation_performed": False, "provider_invoked": False,
              "download_performed": False, "publication_performed": False,
              "retained_targets": list(TARGETS),
              "remaining_unobserved": ["cargo_cache_completeness", "free_build_disk_capacity", "native_external_tools_and_profiles", "build_reproducibility", "installed_owner_status_and_workloads", "publisher_authentication", "native_domain_qualification"]}
    def check(name, operation, action_needed):
        try:
            operation()
            result["checks"].append({"check": name, "state": "passed"})
        except Exception as error:
            code = str(error) if isinstance(error, Refusal) else "prerequisite_unavailable"
            action_needed = {"aniflow_reviewed_successor_required": "prepare_and_review_successor_in_aniflow_then_replace_source_lock",
                             "aniflow_target_owner_unavailable": "review_aniflow_native_target_adapter_before_intel_trial"}.get(code, action_needed)
            result["checks"].append({"check": name, "state": "blocked", "reason": code, "next_action": action_needed})
    if action == "plan":
        result["required_next"] = ["native_matching_mac_host", "clean_exact_source_locks", "installed_offline_native_toolchains_and_dependencies", "reviewed_aniflow_successor_candidate", "explicit_existing_owner_builds", "pinned_three_tool_install_request", "explicit_existing_suite_management_and_workload_proofs"]
        return result
    check("native_host", lambda: require(host == value["target"], "native_matching_mac_required"), "run_on_selected_mac_architecture")
    for tool in TOOLCHAINS:
        def observe_source(tool=tool):
            result["sources"][tool] = source_check(tool, value["sources"][tool], value["target"])
        check(tool + "_source", observe_source, "select_clean_reviewed_source_and_exact_tree_lock_pins")
    if host == value["target"]:
        root = path(value["sources"]["flow"]["directory"])
        check("xcode_command_line_tools", lambda: command(["xcode-select", "--print-path"], root), "prepare_xcode_command_line_tools_outside_this_helper")
        if host == "x86_64-apple-darwin":
            def translated():
                # Missing keys/query failures stay unavailable; never infer Intel
                # hardware readiness from an unsuccessful metadata query.
                observed = command(["/usr/sbin/sysctl", "-n", "sysctl.proc_translated"], root)
                require(observed.strip() != b"1", "rosetta_native_trial_refused")
                require(observed.strip() == b"0", "native_process_architecture_unavailable")
            check("native_process_architecture", translated, "use_native_terminal_and_matching_toolchains")
        for toolchain in sorted(set(TOOLCHAINS.values())):
            def rust(toolchain=toolchain):
                data = command(["rustup", "run", toolchain, "rustc", "--version", "--verbose"], root)
                require(data.startswith(("rustc " + toolchain + " ").encode()) and ("host: " + host + "\n").encode() in data, "native_toolchain_differs")
            check("rust_" + toolchain, rust, "prepare_exact_native_toolchain_and_frozen_cargo_cache_outside_this_helper")
    if action == "observe":
        def artifacts():
            require("install_request" in value, "pinned_install_request_required")
            require(set(result["sources"]) == set(TOOLCHAINS), "source_prerequisites_required_before_artifact_admission")
            result["artifacts"] = observe_install(value)
            require(all(result["artifacts"][name]["version"] == result["sources"][name]["version"] for name in TOOLCHAINS), "candidate_cargo_version_differs")
        check("candidate_selection_bytes", artifacts, "prepare_existing_owner_artifacts_and_exact_three_tool_install_request")
    result["state"] = "blocked" if any(c["state"] == "blocked" for c in result["checks"]) else "prerequisites_observed"
    return result


def write_report(directory, value, selected):
    root = path(str(directory))
    require(root.parent.is_dir() and not root.exists(), "fresh_report_root_required")
    protected = [path(s["directory"]) for s in selected["sources"].values()]
    if "install_request" in selected:
        protected.append(filepin(selected["install_request"]))
    require(all(not root.is_relative_to(p) and not p.is_relative_to(root) for p in protected), "report_input_overlap")
    data = (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()
    require(len(data) <= 262_144, "report_output_bound")
    root.mkdir(mode=0o700)
    descriptor = os.open(root / "report.json", os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "wb") as stream:
        stream.write(data)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("plan", "preflight", "observe"))
    parser.add_argument("--request", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        data = read(args.request)
        selected = request(data)
        result = report(args.action, selected, sha(data))
        require(args.output.absolute() != args.request.absolute(), "report_input_overlap")
        write_report(args.output, result, selected)
        print(json.dumps(result, sort_keys=True))
        return 2 if result["state"] == "blocked" else 0
    except Exception as error:
        reason = str(error) if isinstance(error, Refusal) else "request_or_prerequisite_unavailable"
        print(json.dumps({"schema_version": "flow.macos-native-trial-error/v1", "state": "refused", "reason": reason,
                          "native_execution": "not_executed_by_this_helper"}, sort_keys=True))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
