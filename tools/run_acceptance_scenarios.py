#!/usr/bin/env python3
"""Execute Flow #30/#65 recipes and retain checked, normalized portable receipts.

This is a test driver, not a Flow runtime. Rust owns assertions and public API
execution; this driver checks completeness and writes a bounded coverage report.
"""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import bounded_test_process
import lifecycle_reports

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "tests/fixtures/acceptance-scenarios.v1.json"
LIFECYCLE_CATALOG = ROOT / "tests/fixtures/lifecycle-scenarios.v1.json"
MARKER = "FLOW_ACCEPTANCE_RECEIPT="
FAMILIES = {"resolution", "contract", "artifact", "provider", "privacy"}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def source_identity():
    """Pin the tested recipe, provider, contracts, and implementation bytes."""
    paths = {"Cargo.toml", "Cargo.lock", "LICENSE", "tests/hermetic_provider_kit.rs",
             "tools/run_acceptance_scenarios.py", "tools/bounded_test_process.py",
             "tools/bounded_test_runner.py", "tools/lifecycle_reports.py",
             "tools/test_lifecycle_report.py", ".github/workflows/ci.yml"}
    paths.add("tests/durable_state.rs")
    for directory in ["src", "contracts", "tests/scenario_matrix", "tests/durable_execution", "tests/graph_recovery", "tests/lifecycle_matrix", "tests/fixtures", "tests/common"]:
        paths.update(str(path.relative_to(ROOT)) for path in (ROOT / directory).rglob("*")
                     if path.is_file() and "__pycache__" not in path.parts)
    entries = {path: digest((ROOT / path).read_bytes()) for path in sorted(paths)}
    return {"algorithm": "sha256", "files": entries,
            "digest": digest(json.dumps(entries, sort_keys=True, separators=(",", ":")).encode())}


def verify_receipts(catalog, output):
    scenarios = catalog["scenarios"]
    expected = {case["scenario_id"]: case for case in scenarios}
    if len(expected) != len(scenarios) or not scenarios:
        raise ValueError("empty catalog or duplicate scenario ID")
    if {case["family"] for case in scenarios} != FAMILIES:
        raise ValueError("missing or unknown scenario family")
    receipts = {}
    for line in output.splitlines():
        if MARKER not in line:
            continue
        encoded = line.split(MARKER, 1)[1]
        if len(encoded.encode()) > catalog["budget"]["max_receipt_bytes"]:
            raise ValueError("receipt exceeds catalog budget")
        receipt = json.loads(encoded)
        identifier = receipt["scenario_id"]
        if identifier not in expected or identifier in receipts:
            raise ValueError("unknown or duplicate receipt")
        if receipt["schema_version"] != "flow.acceptance-scenario-receipt/v1":
            raise ValueError("unsupported receipt version")
        if receipt["outcome"] != expected[identifier]["expected"]:
            raise ValueError(f"unexpected outcome: {identifier}")
        if not receipt["recovery"] or not receipt["fixture_identity"]:
            raise ValueError("receipt lacks recovery or fixture identity")
        receipts[identifier] = receipt
    if set(receipts) != set(expected):
        raise ValueError(f"missing {len(set(expected) - set(receipts))} scenario receipts")
    return [receipts[key] for key in sorted(receipts)]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, help="Primary report path (defaults to target/<selected>-scenarios.v1.report.json).")
    selection = parser.add_mutually_exclusive_group()
    selection.add_argument("--all-targets", action="store_true", help="Execute all Rust targets and verify both acceptance and lifecycle coverage.")
    selection.add_argument("--lifecycle-only", action="store_true", help="Execute and verify the durable lifecycle corpus only.")
    parser.add_argument("--lifecycle-output", type=Path, default=Path("target/lifecycle-scenarios.v1.report.json"), help="Additional lifecycle report path with --all-targets.")
    parser.add_argument("--cargo", default="cargo", help="Cargo executable; dependencies must already be cached.")
    args = parser.parse_args()
    selected = "lifecycle" if args.lifecycle_only else "acceptance"
    output = args.output or Path(f"target/{selected}-scenarios.v1.report.json")
    report_path = output if output.is_absolute() else ROOT / output
    lifecycle_path = args.lifecycle_output if args.lifecycle_output.is_absolute() else ROOT / args.lifecycle_output
    if args.all_targets and lifecycle_path.resolve() == report_path.resolve():
        raise ValueError("acceptance and lifecycle reports require distinct paths")
    # A failed run must never leave an older successful report at this target.
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.unlink(missing_ok=True)
    report_path.with_suffix(".failure.log").unlink(missing_ok=True)
    if args.all_targets:
        lifecycle_path.parent.mkdir(parents=True, exist_ok=True)
        lifecycle_path.unlink(missing_ok=True)
    catalog = json.loads(CATALOG.read_text())
    if catalog["schema_version"] != "flow.acceptance-scenario-catalog/v1":
        raise ValueError("unsupported catalog version")
    before = source_identity()
    command = bounded_test_process.cargo_command(args.cargo, ROOT)
    command += ["--all-targets"] if args.all_targets else ["--test", "hermetic_provider_kit", "lifecycle_matrix::executable_lifecycle_matrix" if args.lifecycle_only else "scenario_matrix::"]
    # A concurrent fork briefly inherits other workers' flock descriptors before
    # exec. Isolate test-owned run workspaces; explicit contention tests still run.
    command += ["--", "--nocapture", "--test-threads=1"]
    try:
        result = bounded_test_process.run(command, cwd=ROOT)
    except bounded_test_process.BudgetExceeded as error:
        report_path.with_suffix(".failure.log").write_bytes(error.output)
        raise
    if result.returncode:
        # Retained locally for diagnosis. Raw test/provider text is not portable.
        log = report_path.with_suffix(".failure.log")
        log.write_text(result.stdout)
        raise ValueError(f"Rust tests failed; inspect local diagnostics at {log}")
    receipts = [] if args.lifecycle_only else verify_receipts(catalog, result.stdout)
    lifecycle_catalog = json.loads(LIFECYCLE_CATALOG.read_text())
    lifecycle_receipts = lifecycle_reports.verify_receipts(lifecycle_catalog, result.stdout, before) if args.all_targets or args.lifecycle_only else []
    after = source_identity()
    if before != after:
        changed = sorted(path for path in before["files"].keys() | after["files"].keys()
                         if before["files"].get(path) != after["files"].get(path))
        raise ValueError(f"source bytes changed during validation: {', '.join(changed)}")
    toolchain = subprocess.check_output([args.cargo, "--version"], text=True, timeout=10).strip()
    report = {
        "schema_version": "flow.acceptance-scenario-report/v1",
        "catalog_digest": digest(CATALOG.read_bytes()),
        "source_identity": before,
        "toolchain": toolchain,
        "tier": catalog["tier"],
        "status": "passed",
        "scenario_count": len(receipts),
        "executions_per_scenario": catalog["budget"]["repetitions"],
        "coverage": {family: sum(case["family"] == family for case in catalog["scenarios"])
                     for family in sorted(FAMILIES)},
        "budgets": catalog["budget"],
        "known_gaps": catalog["known_gaps"],
        "receipts": receipts,
    }
    if args.lifecycle_only:
        report = lifecycle_reports.report(lifecycle_catalog, LIFECYCLE_CATALOG.read_bytes(), lifecycle_receipts, before, toolchain)
    write_report(report_path, report)
    print(f"PASS: {report['scenario_count']} {selected} scenarios, two fresh roots each; {report_path}")
    if args.all_targets:
        write_report(lifecycle_path, lifecycle_reports.report(lifecycle_catalog, LIFECYCLE_CATALOG.read_bytes(), lifecycle_receipts, before, toolchain))
        print(f"PASS: {len(lifecycle_receipts)} lifecycle scenarios, two fresh roots each; {lifecycle_path}")


def write_report(path, report):
    temporary = path.with_suffix(".tmp")
    temporary.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    temporary.replace(path)


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.SubprocessError, KeyError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        sys.exit(1)
