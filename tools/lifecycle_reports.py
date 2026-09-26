"""Strict allowlist and completeness checks for test-owned lifecycle receipts."""
import hashlib
import json
import re

MARKER = "FLOW_LIFECYCLE_RECEIPT="
RECEIPT_FIELDS = {"schema_version", "scenario_id", "recipe_digest", "fixture_identity",
                  "outcome", "recovery", "plan_digest", "history_digest", "artifact_digests"}
IDENTITY_FIELDS = {"package_digest", "executable_digest", "manifest_digest", "input_digest", "bindings_digest"}
RECOVERY = {"operator-decision-required", "continue-ready-work", "reuse-accepted-work",
            "retry-with-acknowledgement", "refuse-retry", "abandon-invalid-evidence",
            "rebuild-current-context", "preserve-and-repair-store"}
BUDGET = {"test_threads": 1, "repetitions": 2, "timeout_ms": 60_000, "max_receipt_bytes": 32_768,
          "max_snapshots": 16, "max_history_bytes": 2_097_152, "max_artifacts": 16,
          "max_artifact_bytes": 1_048_576, "max_fixture_bytes": 134_217_728,
          "max_process_address_space_bytes": 4_294_967_296, "max_process_file_bytes": 16_777_216,
          "max_driver_output_bytes": 4_194_304, "driver_timeout_seconds": 600}


def canonical_digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()).hexdigest()


def require_digest(value):
    if not isinstance(value, str) or not re.fullmatch("[0-9a-f]{64}", value):
        raise ValueError("invalid identity digest")


def verify_receipts(catalog, output, source_identity):
    if set(catalog) != {"schema_version", "fixture_version", "tier", "budget", "known_gaps", "scenarios"}:
        raise ValueError("unknown or missing catalog fields")
    if (catalog["schema_version"] != "flow.lifecycle-scenario-catalog/v1"
            or catalog["fixture_version"] != "1.1.0" or catalog["tier"] != "pull-request-linux"):
        raise ValueError("unsupported lifecycle catalog")
    if catalog["budget"] != BUDGET or not catalog["known_gaps"]:
        raise ValueError("unsupported lifecycle budgets or missing gaps")
    scenarios = catalog["scenarios"]
    expected = {case["scenario_id"]: case for case in scenarios}
    if not expected or len(expected) != len(scenarios) or len({case["recipe"] for case in scenarios}) != len(scenarios):
        raise ValueError("empty or duplicate lifecycle recipes")
    for case in scenarios:
        if set(case) != {"scenario_id", "recipe", "mode", "graph", "recovery", "expected"} or case["recovery"] not in RECOVERY:
            raise ValueError("unsupported lifecycle recipe fields or recovery classification")
    receipts = {}
    for line in output.splitlines():
        if MARKER not in line:
            continue
        encoded = line.split(MARKER, 1)[1]
        if len(encoded.encode()) > BUDGET["max_receipt_bytes"]:
            raise ValueError("lifecycle receipt exceeds budget")
        receipt = json.loads(encoded)
        if set(receipt) != RECEIPT_FIELDS or receipt["schema_version"] != "flow.lifecycle-scenario-receipt/v1":
            raise ValueError("unsupported lifecycle receipt fields or version")
        identifier = receipt["scenario_id"]
        if identifier not in expected or identifier in receipts:
            raise ValueError("unknown or duplicate lifecycle receipt")
        case = expected[identifier]
        if receipt["recipe_digest"] != canonical_digest({"fixture_version": catalog["fixture_version"], "case": case}):
            raise ValueError("lifecycle recipe identity drift")
        if receipt["outcome"] != case["expected"] or receipt["recovery"] != case["recovery"]:
            raise ValueError(f"unexpected lifecycle trace or recovery: {identifier}")
        identities = receipt["fixture_identity"]
        count = 3 if case["graph"] else 1
        if len(identities) != count or len(receipt["artifact_digests"]) != count:
            raise ValueError("incomplete fixture inventory")
        for identity in identities:
            if set(identity) != IDENTITY_FIELDS:
                raise ValueError("unsupported fixture identity fields")
            for value in identity.values():
                require_digest(value)
            if identity["input_digest"] != source_identity["files"]["contracts/fixtures/scenarios/sources/source-text.txt"]:
                raise ValueError("fixture source identity drift")
        require_digest(receipt["plan_digest"])
        require_digest(receipt["history_digest"])
        for value in receipt["artifact_digests"]:
            if value is not None:
                require_digest(value)
        receipts[identifier] = receipt
    if receipts.keys() != expected.keys():
        raise ValueError("missing lifecycle receipts")
    # Every recipe used the same exact compiled provider package. Per-toolchain
    # executables may differ, so pin them in the report rather than in the catalog.
    for field in ["executable_digest", "package_digest", "manifest_digest"]:
        if len({identity[field] for receipt in receipts.values() for identity in receipt["fixture_identity"]}) != 1:
            raise ValueError("provider fixture identity drift")
    return [receipts[key] for key in sorted(receipts)]


def report(catalog, catalog_bytes, receipts, source_identity, toolchain):
    return {"schema_version": "flow.lifecycle-scenario-report/v1", "status": "passed",
            "catalog_digest": hashlib.sha256(catalog_bytes).hexdigest(), "source_identity": source_identity,
            "toolchain": toolchain, "tier": catalog["tier"], "scenario_count": len(receipts),
            "executions_per_scenario": BUDGET["repetitions"], "budgets": BUDGET,
            "known_gaps": catalog["known_gaps"], "receipts": receipts}
