"""Explicit local synthetic #21 conformance; no media, model, collector or CI.

Select exact independently built binaries explicitly; the driver never builds
or installs them. Each child receives literal argv and synthetic canaries,
never the parent's credentials or complete environment. Results describe only
the recorded local selection, not telemetry adoption or release qualification.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

REQUEST = {"schema_version": "egohygiene.diagnostics-request/v1", "categories": ["build", "platform", "contracts", "health"]}
CANARY = "PRIVATE_SUPPORT_CANARY_DO_NOT_COPY"
FIXTURES = Path(__file__).resolve().parents[1] / "docs/observability/fixtures/support-conformance.v1.json"
COMMANDS = {
    "flow": lambda operation: ["doctor-observability", "--format", "json"] if operation == "doctor" else ["diagnostics-" + operation, "--format", "json"],
    "aniflow": lambda operation: ["--output", "json", "diagnostics", operation],
    "optiflow": lambda operation: ["--json", "diagnostics", operation],
    "renderflow": lambda operation: ["diagnostics", operation],
}
INVALID = "invalid or oversized support metadata"
EXISTS = "support destination already exists; preserve it"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def exact_binary(path):
    require(path.is_absolute() and path.resolve(strict=True) == path and path.is_file() and not path.is_symlink(), "explicit canonical built binary required")
    digest = hashlib.sha256()
    with path.open("rb") as file:
        for chunk in iter(lambda: file.read(1_048_576), b""):
            digest.update(chunk)
    return {"sha256": digest.hexdigest(), "byte_count": path.stat().st_size}


def require_refusal(tool, operation, result, expected):
    """A crash, parser error or unrelated refusal is not a conformance pass."""
    require(expected in (INVALID, EXISTS), "unknown support-refusal expectation")
    if tool == "renderflow":
        require(result.returncode == 1 and not result.stdout
                and result.stderr == ("Error: " + expected + "\n").encode(),
                "Renderflow did not return the expected native support refusal")
        return
    raw = result.stderr if tool == "aniflow" else result.stdout
    other = result.stdout if tool == "aniflow" else result.stderr
    require(not other, "support refusal contaminated its other native stream")
    value = json.loads(raw)
    require(isinstance(value, dict) and result.returncode > 0,
            "support refusal lacks a native result")
    if tool == "aniflow":
        require(result.returncode == 3
                and value.get("schema_version") == 1
                and value.get("command") == "diagnostics_" + operation
                and value.get("status") == "error"
                and value.get("error", {}).get("category") == "input"
                and value.get("error", {}).get("message") == expected,
                "Aniflow did not return the expected native support refusal")
    elif tool == "flow":
        code, outcome, exit_code = (("invalid_input", "refused", 3) if expected == INVALID
                                    else ("recovery_required", "recovery_required", 7))
        require(result.returncode == exit_code
                and value.get("schema_version") == "flow.command-result/v1"
                and value.get("command") == "diagnostics-" + operation
                and value.get("exit_code") == result.returncode
                and value.get("outcome") == outcome
                and value.get("result") is None
                and len(value.get("diagnostics", [])) == 1
                and value["diagnostics"][0].get("code") == code
                and value["diagnostics"][0].get("message") == expected,
                "Flow did not return the expected native support refusal")
    else:
        require(result.returncode == 2
                and value.get("schema") == "optiflow.command-result.v1"
                and value.get("command") == "diagnostics " + operation
                and value.get("outcome", {}).get("exit_code") == result.returncode
                and value.get("outcome", {}).get("class") == "invalid_input"
                and value.get("result") is None
                and len(value.get("diagnostics", [])) == 1
                and value["diagnostics"][0].get("code") == "invalid_command_input"
                and value["diagnostics"][0].get("classification") == "input"
                and value["diagnostics"][0].get("severity") == "error"
                and value["diagnostics"][0].get("impact") == "blocks_command"
                and value["diagnostics"][0].get("message") == expected,
                "Optiflow did not return the expected native support refusal")


def invoke(tool, binary, operation, options, success, expected_error=INVALID):
    environment = {"PATH": "/usr/bin:/bin", "LC_ALL": "C", "NO_COLOR": "1",
                   "SUPPORT_SECRET_CANARY": CANARY, "OTEL_EXPORTER_OTLP_HEADERS": "authorization=" + CANARY,
                   "OTEL_EXPORTER_OTLP_ENDPOINT": "http://127.0.0.1:9"}
    result = subprocess.run([str(binary), *COMMANDS[tool](operation), *options], stdin=subprocess.DEVNULL,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=15, check=False, env=environment)
    require(len(result.stdout) <= 131_072 and len(result.stderr) <= 131_072, "support output bound exceeded")
    require(CANARY.encode() not in result.stdout + result.stderr, "private canary leaked into command output")
    require(b"\x1b" not in result.stdout, "machine result contains ANSI")
    require((result.returncode == 0) is success,
            f"{tool} {operation} returned unexpected native exit {result.returncode}")
    if not success:
        require_refusal(tool, operation, result, expected_error)
        return None
    require(not result.stderr, "successful support operation emitted extra stderr")
    value = json.loads(result.stdout)
    if tool != "renderflow":
        require(type(value) is dict and "result" in value, "native result envelope required")
        value = value["result"]
    require(type(value) is dict and value.get("schema_version", "").startswith("egohygiene."), "support result schema missing")
    require(value.get("domain_evidence") is False and value.get("network_contacted") is False, "support result claims domain/network authority")
    return value


def check(tool, binary, cases=None):
    identity = exact_binary(binary)
    cases = [] if cases is None else cases
    with tempfile.TemporaryDirectory(prefix="support-" + CANARY + "-") as temporary:
        root = Path(temporary).resolve(); request = root / "request.json"; bundle = root / "bundle.json"
        request.write_text(json.dumps(REQUEST))
        before = set(root.iterdir())
        preview = invoke(tool, binary, "preview", ["--request", str(request)], True)
        require(set(root.iterdir()) == before and preview["files_written"] is False and preview["member_count"] == 4, "preview wrote or miscounted members")
        cases.append("preview-no-write")
        output_flag = "--output-file" if tool == "aniflow" else "--output"
        created = invoke(tool, binary, "bundle", ["--request", str(request), output_flag, str(bundle)], True)
        raw = bundle.read_bytes(); require(CANARY.encode() not in raw, "canary leaked into an archive member")
        if os.name == "posix": require(bundle.stat().st_mode & 0o777 == 0o600, "bundle permissions are not private")
        archive = json.loads(raw); require(len(archive["members"]) == 4 and archive["domain_evidence"] is False, "bundle membership/authority differs")
        cases.append("bundle-generated-allowlist")
        inspected = invoke(tool, binary, "inspect", ["--bundle", str(bundle)], True)
        require(inspected["claims_authenticated"] is False and inspected["digest"] == created["digest"], "inspection promoted or changed the bundle")
        cases.append("inspect-unsigned")
        invoke(tool, binary, "bundle", ["--request", str(request), output_flag, str(bundle)], False, EXISTS)
        require(bundle.read_bytes() == raw, "existing support output was overwritten"); cases.append("no-clobber")
        request.write_text(json.dumps({**REQUEST, "raw_logs": CANARY}))
        invoke(tool, binary, "preview", ["--request", str(request)], False); cases.append("unknown-input-refusal")
        for name, change in [
            ("tampered-member-refusal", lambda value: value["members"][0]["digest"].update(value="0" * 64)),
            ("path-traversal-refusal", lambda value: value["members"][0].update(name="../private")),
            ("duplicate-member-refusal", lambda value: value["members"].__setitem__(1, value["members"][0])),
        ]:
            value = json.loads(raw); change(value); altered = root / (name + ".json"); altered.write_text(json.dumps(value))
            invoke(tool, binary, "inspect", ["--bundle", str(altered)], False); cases.append(name)
        oversized = root / "oversized.json"; oversized.write_bytes(b" " * 65_537)
        invoke(tool, binary, "inspect", ["--bundle", str(oversized)], False); cases.append("oversize-refusal")
        if os.name == "posix":
            alias = root / "alias.json"; alias.symlink_to(bundle)
            invoke(tool, binary, "inspect", ["--bundle", str(alias)], False); cases.append("alias-refusal")
        health = invoke(tool, binary, "doctor", [], True)
        require(health["queue_records"] is None and health["dropped_records"] is None and health["exporters_compiled"] is False
                and health["logging_runtime_observed"] is False and health["subscriber_initialized_by_diagnostics"] is False,
                "unknown/unsupported health became measured or initialized")
        cases.append("health-unknown-not-zero")
        require(exact_binary(binary) == identity, "selected binary changed during conformance")
    fixture = json.loads(FIXTURES.read_text())
    expected = fixture["cases"] if os.name == "posix" else [case for case in fixture["cases"] if case != "alias-refusal"]
    require(cases == expected, "executed cases differ from the shared fixture inventory")
    return {"tool": tool, "state": "passed", "binary": identity,
            "cases": [{"id": name, "state": "passed"} for name in cases]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for tool in COMMANDS:
        parser.add_argument("--" + tool, type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    require(not args.output.exists() and not args.output.is_symlink()
            and args.output.parent.is_dir(), "support report requires a fresh path in an existing directory")
    selected = [(tool, getattr(args, tool)) for tool in COMMANDS if getattr(args, tool) is not None]
    require(selected, "select at least one exact independently built CLI")
    rows = []
    for tool, binary in selected:
        cases = []
        try:
            rows.append(check(tool, binary, cases))
        except (ValueError, OSError, RuntimeError, TypeError, KeyError, AttributeError, subprocess.TimeoutExpired) as error:
            # Keep only fixed diagnostics. Native captures and paths stay private.
            reason = (str(error) if isinstance(error, ValueError)
                      else "native support I/O, input resolution or process deadline failed")
            rows.append({"tool": tool, "state": "failed", "failure": reason,
                         "cases": [{"id": name, "state": "passed"} for name in cases]})
    all_selected_passed = all(row["state"] == "passed" for row in rows)
    report = {"schema_version": "egohygiene.diagnostics-conformance-report/v1", "observed_unix_seconds": int(time.time()),
              "scope": "local metadata-only synthetic CLI profile", "tools": rows,
              "unobserved_tools": [tool for tool in COMMANDS if getattr(args, tool) is None],
              "all_selected_passed": all_selected_passed,
              "complete": all_selected_passed and len(selected) == len(COMMANDS),
              "native_media_qualified": False, "telemetry_profile_adopted": False, "release_qualified": False}
    with args.output.open("x", encoding="utf8") as file:
        json.dump(report, file, indent=2); file.write("\n")
    return 0 if all_selected_passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
