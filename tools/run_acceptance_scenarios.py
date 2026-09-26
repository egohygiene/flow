#!/usr/bin/env python3
"""Execute Flow #30 recipes and retain only checked, normalized portable receipts.

This is a test driver, not a Flow runtime. Rust owns assertions and public API
execution; this driver checks completeness and writes a bounded coverage report.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "tests/fixtures/acceptance-scenarios.v1.json"
MARKER = "FLOW_ACCEPTANCE_RECEIPT="
FAMILIES = {"resolution", "contract", "artifact", "provider", "privacy"}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def source_identity():
    """Pin the tested recipe, provider, contracts, and implementation bytes."""
    paths = {"Cargo.toml", "Cargo.lock", "LICENSE", "tests/hermetic_provider_kit.rs",
             "tools/run_acceptance_scenarios.py"}
    paths.add("tests/durable_state.rs")
    for directory in ["src", "contracts", "tests/scenario_matrix", "tests/durable_execution", "tests/fixtures", "tests/common"]:
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
    parser.add_argument("--output", type=Path, default=Path("target/acceptance-scenarios.v1.report.json"))
    parser.add_argument("--all-targets", action="store_true", help="Also execute the rest of the required Rust test suite.")
    parser.add_argument("--cargo", default="cargo", help="Cargo executable; dependencies must already be cached.")
    args = parser.parse_args()
    report_path = args.output if args.output.is_absolute() else ROOT / args.output
    # A failed run must never leave an older successful report at this target.
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.unlink(missing_ok=True)
    report_path.with_suffix(".failure.log").unlink(missing_ok=True)
    catalog = json.loads(CATALOG.read_text())
    if catalog["schema_version"] != "flow.acceptance-scenario-catalog/v1":
        raise ValueError("unsupported catalog version")
    if os.name != "posix":
        raise ValueError("the PR matrix requires a Unix symlink-capable host")
    before = source_identity()
    command = [args.cargo, "test", "--locked", "--offline"]
    command += ["--all-targets"] if args.all_targets else ["--test", "hermetic_provider_kit", "scenario_matrix::"]
    command += ["--", "--nocapture"]
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, timeout=600, check=False)
    if result.returncode:
        # Retained locally for diagnosis. Raw test/provider text is not portable.
        log = report_path.with_suffix(".failure.log")
        log.write_text((result.stdout + result.stderr)[-4_194_304:])
        raise ValueError(f"Rust tests failed; inspect local diagnostics at {log}")
    if len(result.stdout.encode()) + len(result.stderr.encode()) > 4_194_304:
        raise ValueError("test output exceeds the 4 MiB driver budget")
    receipts = verify_receipts(catalog, result.stdout)
    after = source_identity()
    if before != after:
        changed = sorted(path for path in before["files"].keys() | after["files"].keys()
                         if before["files"].get(path) != after["files"].get(path))
        raise ValueError(f"source bytes changed during validation: {', '.join(changed)}")
    report = {
        "schema_version": "flow.acceptance-scenario-report/v1",
        "catalog_digest": digest(CATALOG.read_bytes()),
        "source_identity": before,
        "toolchain": subprocess.check_output([args.cargo, "--version"], text=True).strip(),
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
    temporary = report_path.with_suffix(".tmp")
    temporary.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    temporary.replace(report_path)
    print(f"PASS: {len(receipts)} acceptance scenarios, two fresh roots each; {report_path}")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.SubprocessError, KeyError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        sys.exit(1)
