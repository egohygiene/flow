#!/usr/bin/env python3
"""Observe existing installed audio setup, current checks and snapshot handoff.

Only the selected synthetic CP4 handoff is consumed. Candidate acknowledgements
permit exact unconfined local processes; they are never human or release approval.
"""
import argparse
import copy
from datetime import datetime, timezone
import math
from pathlib import Path
import re
import sys
import time
from types import ModuleType

from check_native_publication import pin, put, refusal
from check_installed_music_video import inventory
from check_released_read_only import OWNER_IO, canonical, command, document, envelope, native_path, require, sha
from installed_audio_candidate_subjects import InstalledAudioCandidate


HANDOFF_FIELDS = {"schema_version", "selection", "context", "project_record", "pilot_handoff",
    "manual_srt", "recipe", "source_video", "final_qc_receipt", "legacy_qc_receipt",
    "final_qc_native_record", "video_native_receipt", "driver", "report", "final_qc_directory",
    "source_run_directory", "artifact_id", "tool_directories", "provider_identities", "current_context"}
PIN_FIELDS = ["selection", "context", "project_record", "pilot_handoff", "manual_srt", "recipe",
    "source_video", "final_qc_receipt", "legacy_qc_receipt", "final_qc_native_record",
    "video_native_receipt", "driver", "report"]
LIMITS = {"timeout_ms": 120000, "max_input_bytes": 2000000, "max_output_bytes": 16000000,
    "max_total_output_bytes": 32000000, "max_artifacts": 8}
REVIEWS = ["words", "timing", "visual", "audio", "final_package"]


def path(value):
    require(type(value) is dict and set(value) == {"encoding", "value"}
        and value["encoding"] == "utf8" and type(value["value"]) is str,
        "explicit UTF-8 native path required")
    result = Path(value["value"])
    require(result.is_absolute() and ".." not in result.parts and result.resolve(strict=True) == result,
        "current canonical selected path required")
    return result


def selected_file(value):
    require(type(value) is dict and set(value) == {"path", "sha256", "byte_count"}
        and type(value["byte_count"]) is int and 0 < value["byte_count"] <= 83886080
        and re.fullmatch(r"[0-9a-f]{64}", value["sha256"]) is not None,
        "closed bounded file pin required")
    selected = path(value["path"])
    require(pin(selected) == value, "selected handoff member changed")
    return selected


def data(selected):
    return document(OWNER_IO.regular(selected, 16777216))


def helper_pins():
    root = Path(__file__).resolve().parents[1]
    # Explicit importlib modules can be held only as another module's attribute
    # (OWNER_IO and PACKAGE), so sys.modules alone is not the helper closure.
    selected = {Path(__file__).resolve()}
    pending = list(sys.modules.values())
    seen = set()
    while pending:
        module = pending.pop()
        if not isinstance(module, ModuleType) or id(module) in seen:
            continue
        seen.add(id(module))
        require(len(seen) <= 1024, "module observation bound exceeded")
        name = getattr(module, "__file__", None)
        if name:
            candidate = Path(name).resolve()
            if candidate.is_relative_to(root) and candidate.suffix == ".py":
                selected.add(candidate)
                pending.extend(value for value in vars(module).values() if isinstance(value, ModuleType))
    require(len(selected) <= 32, "observer helper count bound exceeded")
    return {p.relative_to(root).as_posix(): {k: v for k, v in pin(p).items() if k != "path"}
        for p in sorted(selected)}


def pending(value):
    require(value["reviews"]["state"] == "review_pending"
        and value["reviews"]["pending"] == REVIEWS
        and value["reviews"]["changes_requested"] == []
        and value["reviews"]["identity_authenticated"] is False,
        "human reviews must remain explicitly pending")


class Observer:
    def __init__(self, subject, root):
        self.subject, self.root = subject, root
        self.logs = root / "commands"
        self.logs.mkdir(mode=0o700)
        self.cases = []
        self.started = time.monotonic()

    def invoke(self, label, args, code=0, diagnostic=None, outcome=None):
        require(re.fullmatch(r"[a-z0-9-]+", label) is not None, "simple unique case label required")
        selected_outcome = outcome or ("complete" if code == 0 else "recovery_required" if code == 7 else "refused")
        remaining = 900 - (time.monotonic() - self.started)
        require(remaining > 0, "audio-readiness observation deadline exceeded")
        argv = [str(self.subject.flow), *map(str, args), "--format", "json"]
        put(self.logs / (label + ".invocation.json"), {"argv": argv, "expected_exit": code,
            "expected_outcome": selected_outcome})
        actual, stdout, stderr = command(argv, cwd=self.root, timeout=min(145, remaining), max_output=16777216)
        (self.logs / (label + ".stdout.json")).write_bytes(stdout)
        (self.logs / (label + ".stderr")).write_bytes(stderr)
        value = envelope(stdout, stderr, actual, args[0], code, selected_outcome)
        if diagnostic is not None:
            refusal(value, diagnostic)
        else:
            require(value["diagnostics"] == [] and type(value["result"]) is dict,
                "unexpected audio-readiness diagnostic")
        self.cases.append({"case": label, "command": args[0], "exit_code": actual,
            "outcome": selected_outcome, "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr),
            "stdout_bytes": len(stdout), "stderr_bytes": len(stderr),
            "diagnostic_code": diagnostic[0] if diagnostic else None})
        return value["result"]


def scope_summary(value, expected_match, checks, expected_policy):
    require(value["schema_version"] == "flow.audio-readiness-summary/v1"
        and value["all_selected_audio_goals_match"] is expected_match
        and value["owner_checks_performed"] == checks
        and value["qualification_pending"] is True
        and value["evidence_scope"] == ("preview_only; current_owner_checks_not_performed" if checks == 0
            else "owner_rechecked_during_this_attempt; not_a_permanent_or_platform_certificate")
        and value["upload_authority"] is False and value["current_platform_compliance_verified"] is False,
        "numeric policy evidence must not claim qualification or publication")
    pending(value)
    rows = {(r["payload_id"], r["destination"]): r for r in value["rows"]}
    require(len(value["rows"]) == len(rows) == 3
        and set(rows) == {("clean-master", "archive"), ("delivery-video", "archive"),
        ("delivery-video", "tiktok")}, "all and only assigned media/destination pairs required")
    archived = rows[("clean-master", "archive")]
    require(archived["state"] == "archive_master_unchanged"
        and all(archived[k] is None for k in ["policy", "integrated_loudness", "true_peak",
            "matches_selected_policy", "native_record"])
        and archived["meter_qualified"] is archived["provider_release_qualified"] is False,
        "archive preservation cannot manufacture numeric evidence or authority")
    for destination in ["archive", "tiktok"]:
        row = rows[("delivery-video", destination)]
        require(row["policy"] == expected_policy and row["meter_qualified"] is False
            and row["provider_release_qualified"] is False, "selected policy or false qualification changed")
        if checks == 0:
            require(row["state"] == "fresh_owner_check_required"
                and all(row[k] is None for k in ["integrated_loudness", "true_peak", "matches_selected_policy", "native_record"]),
                "read-only preview inferred measured evidence")
        else:
            require(row["state"] == ("matches_selected_policy" if expected_match else "outside_target")
                and row["matches_selected_policy"] is expected_match and type(row["native_record"]) is dict,
                "actual owner check did not retain its exact numeric state")
    return rows


def report_scope(value, summary=None):
    require(value["schema_version"] == "flow.publishing-readiness/v1"
        and value["current_copied_bytes"] is True
        and value["evidence_scope"] == "current local copied bytes and correlated historical declarations/snapshots only; not current QC, authenticated human review, runtime/release qualification or publication authority"
        and all(value[k] is False for k in ["current_owner_checks_performed", "release_admission_reverified",
            "current_platform_compliance_verified", "upload_authority"]),
        "historical publishing report inferred current native/release/upload authority")
    pending(value)
    require(any(a["kind"] == "review" for a in value["actions"])
        and any(a["kind"] == "verify_qualification" for a in value["actions"]),
        "publishing report lost pending review or qualification work")
    if summary is not None:
        expected = {(r["payload_id"], r["destination"]): r for r in summary["rows"]}
        observed = {(r["payload_id"], r["destination"]): r for r in value["recorded_audio"]}
        require(len(value["recorded_audio"]) == len(observed) == len(expected) == 3
            and set(observed) == set(expected), "publishing snapshot lost or duplicated audio coverage")
        for key, row in observed.items():
            source = expected[key]
            require(row["recorded_state"] == source["state"]
                and row["matches_selected_policy_in_record"] == source["matches_selected_policy"]
                and row["native_record"] == source["native_record"]
                and row["meter_qualified_in_record"] is False
                and row["provider_release_qualified_in_summary"] is False,
                "publishing report differs from exact recorded owner evidence")


def native_record_joins(summary, qc, delivery):
    result = []
    for row in summary["rows"]:
        if row["native_record"] is None:
            continue
        record_path = selected_file(row["native_record"])
        record = data(record_path)
        require(record["schema_version"] == "flow.aniflow-audio-policy-acceptance/v1"
            and record["scope"] == "owner_rechecked_during_this_attempt; retained_status_requires_a_fresh_owner_check_for_current_QC"
            and record["source_pins"] == [delivery],
            "native policy snapshot is not this exact final video")
        assessment = record["assessment"]
        evidence = assessment["subject"]["facts"]["qc"]
        window = evidence["signal"]["presented_window"]
        require(assessment["subject"]["file"] == {"locator": delivery["path"]["value"],
                "sha256": delivery["sha256"], "byte_count": delivery["byte_count"]}
            and evidence == qc and window == qc["signal"]["presented_window"]
            and assessment["evaluation"]["policy"] == row["policy"]
            and assessment["evaluation"]["matches_selected_policy"] is row["matches_selected_policy"]
            and assessment["evaluation"]["runtime_qualified"] is False
            and assessment["evaluation"]["creative_approval"] == "not_performed"
            and assessment["evaluation"]["publication_authority"] == "none",
            "owner policy snapshot did not reobserve the exact selected presented-window QC")
        for name in ["integrated_loudness", "true_peak"]:
            require(row[name]["observed"] == window["loudness"][name]
                and row[name] == assessment["evaluation"][name],
                "reported quantities differ from current exact presented-window meter")
        evaluation = assessment["evaluation"]
        require(assessment["assessment_sha256"] == sha(canonical([assessment["schema"],
            assessment["request"], assessment["subject"], evaluation])),
            "owner assessment digest differs from independently canonicalized observed facts")
        policy = row["policy"]
        loudness = row["integrated_loudness"]["observed"]["value"]
        peak = row["true_peak"]["observed"]["value"]
        require(loudness["kind"] == peak["kind"] == "measured"
            and all(type(v["value"]) in [int, float] and math.isfinite(v["value"]) for v in [loudness, peak]),
            "independent comparison requires actual finite measurements")
        minimum = (policy["target_milli_lufs"] - policy["tolerance_milli_lu"]) / 1000
        maximum = (policy["target_milli_lufs"] + policy["tolerance_milli_lu"]) / 1000
        peak_maximum = policy["maximum_true_peak_milli_dbtp"] / 1000
        level_matches = minimum <= loudness["value"] <= maximum
        peak_matches = peak["value"] <= peak_maximum
        require(row["integrated_loudness"]["minimum"] == minimum
            and row["integrated_loudness"]["maximum"] == maximum
            and row["true_peak"]["minimum"] is None and row["true_peak"]["maximum"] == peak_maximum
            and row["integrated_loudness"]["state"] == ("matches" if level_matches else "outside_target")
            and row["true_peak"]["state"] == ("matches" if peak_matches else "outside_target")
            and row["matches_selected_policy"] is (level_matches and peak_matches),
            "actual measured values do not independently satisfy the explicit selected policy bounds")
        result.append({"destination": row["destination"], "record_sha256": row["native_record"]["sha256"],
            "assessment_sha256": assessment["assessment_sha256"], "source_sha256": delivery["sha256"],
            "window_pcm_sha256": window["window_pcm_sha256"],
            "matches_selected_policy": row["matches_selected_policy"]})
    return result


def inner_refusal(native_root, message):
    files = sorted(native_root.glob("attempt-*/audio-check.stderr"))
    require(len(files) == 1, "exact attempted owner-check stderr required")
    selected = data(files[0])
    require(selected == {"schema_version": 1, "command": "delivery_audio_policy_check", "status": "error",
        "error": {"category": "state", "message": message}},
        "negative owner check reached an unrelated inner refusal")
    code = data(files[0].with_name("audio-check.exit.json"))
    require(code == {"code": 8} and not (native_root / "accepted.json").exists(),
        "failed owner check was accepted or had an unrelated exit")
    return {"sha256": pin(files[0])["sha256"], "native_exit_code": 8,
        "command": "delivery_audio_policy_check", "category": "state", "message": message}


def run(handoff_path, handoff_sha256, output):
    require(re.fullmatch(r"[0-9a-f]{64}", handoff_sha256) is not None, "external handoff digest required")
    handoff_pin = pin(handoff_path)
    require(handoff_pin["sha256"] == handoff_sha256, "CP4 handoff external digest differs")
    handoff = data(handoff_path)
    require(type(handoff) is dict and set(handoff) == HANDOFF_FIELDS
        and handoff["schema_version"] == "flow.installed-presented-qc-handoff/v1",
        "closed installed presented-QC handoff required")
    members = {name: selected_file(handoff[name]) for name in PIN_FIELDS}
    subject = InstalledAudioCandidate.load(members["selection"])
    require(handoff["context"] == pin(Path(subject.selection["flow_context_equivalence"]))
        and handoff["current_context"] == subject.current_context, "separate current source-input context changed")
    tools = [path(p) for p in handoff["tool_directories"]]
    qc_directory = Path(handoff["final_qc_directory"])
    source_owner = Path(handoff["source_run_directory"])
    require(qc_directory.resolve(strict=True) == qc_directory and source_owner.resolve(strict=True) == source_owner
        and members["final_qc_receipt"] == qc_directory / "qc.json", "current owner/QC roots differ")
    qc = data(members["final_qc_receipt"])
    window = qc["signal"]["presented_window"]
    require(qc["intent"]["request"]["presented_window"] == "exact_presented_audio"
        and qc["intent"]["request"]["source_run_directory"] == str(source_owner)
        and qc["intent"]["request"]["artifact_id"] == handoff["artifact_id"]
        and window["metric_scope"] == "exact_presented_audio_window"
        and window["source_sha256"] == qc["signal"]["source_sha256"]
        and window["runtime_qualified"] is False, "CP4 did not retain current selected presented-window QC")
    require(output.is_absolute() and output.parent.resolve(strict=True) == output.parent and not output.exists(),
        "fresh canonical observation root required")
    legacy_qc_directory = members["legacy_qc_receipt"].parent
    legacy_qc = data(members["legacy_qc_receipt"])
    require(legacy_qc["signal"].get("presented_window") is None
        and legacy_qc["signal"]["source_sha256"] == qc["signal"]["source_sha256"]
        and legacy_qc["signal"]["decoded_pcm_sha256"] == qc["signal"]["decoded_pcm_sha256"],
        "omitted-selection QC must retain the same complete PCM without a presented view")
    protected_roots = {subject.root, members["project_record"].parent, qc_directory, source_owner, legacy_qc_directory}
    require(all(not output.is_relative_to(p) and not p.is_relative_to(output) for p in protected_roots),
        "observation output overlaps retained source/install/QC roots")
    source_before = {str(p): inventory(p) for p in protected_roots}
    helper_before = helper_pins()
    output.mkdir(mode=0o700)
    observer = Observer(subject, output)
    suite_before = subject.observe_status(output, "suite-before")
    invoke = observer.invoke

    catalog = invoke("delivery-catalog", ["delivery-catalog", "--project-record", members["project_record"],
        "--project-record-sha256", handoff["project_record"]["sha256"], "--handoff", members["pilot_handoff"],
        "--handoff-sha256", handoff["pilot_handoff"]["sha256"]])
    masters = [a for a in catalog["artifacts"] if a["stage"] == "video" and a["purpose"] == "clean_archival_master"]
    deliveries = [a for a in catalog["artifacts"] if a["stage"] == "video" and a["artifact_id"] == handoff["artifact_id"]]
    require(len(masters) == len(deliveries) == 1, "exact accepted clean master and QC-bound delivery required")
    master, delivery = masters[0], deliveries[0]
    require(delivery["file"]["sha256"] == window["source_sha256"]
        and delivery["file"]["byte_count"] == window["source_bytes"], "catalog delivery and presented QC subject differ")
    payloads = [{"id": "clean-master", "role": "clean_master", "source": {"kind": "native", "stage": "video", "artifact_id": master["artifact_id"]},
        "file_name": "clean-master" + Path(master["file"]["path"]["value"]).suffix, "destinations": ["archive"]},
        {"id": "delivery-video", "role": "video_delivery", "source": {"kind": "native", "stage": "video", "artifact_id": delivery["artifact_id"]},
        "file_name": "delivery-video" + Path(delivery["file"]["path"]["value"]).suffix, "destinations": ["archive", "tiktok"]},
        {"id": "manual-srt", "role": "lyrics", "source": {"kind": "manual_srt"}, "file_name": "manual-lyrics.srt", "destinations": ["archive"]}]
    package = output / "private delivery 作品"
    delivery_request = {"schema_version": "flow.delivery-request/v1", "project_record": handoff["project_record"],
        "handoff": handoff["pilot_handoff"], "payloads": payloads,
        "metadata": {"creators": [], "description": "Synthetic operator path; no human reviews supplied", "tags": ["synthetic"], "destination_fields": []},
        "reviews": [], "evidence_attachments": [handoff["final_qc_receipt"]], "output_directory": native_path(package),
        "limits": {"timeout_ms": 60000, "maximum_payload_bytes": 16000000, "maximum_total_bytes": 64000000}}
    request_path = output / "delivery-request.json"
    put(request_path, delivery_request)
    preview = invoke("delivery-preview", ["delivery-preview", "--request", request_path]); pending(preview)
    require(not package.exists() and preview["publishing_authority"] is False, "delivery preview wrote or approved package")
    copied = invoke("delivery-prepare", ["delivery-prepare", "--request", request_path]); pending(copied["preview"])
    manifest = pin(package / "manifest.json")
    require(copied["manifest"] == manifest and copied["current_copied_bytes"] is True, "new package manifest differs")
    package_before = inventory(package)
    for payload in copied["preview"]["payloads"]:
        current = pin(package / payload["relative"])
        require(current["sha256"] == payload["sha256"] and current["byte_count"] == payload["byte_count"],
            "copied package payload differs from selected bytes")
    invoke("delivery-status", ["delivery-status", "--directory", package, "--manifest-sha256", manifest["sha256"]])

    meter = window["loudness"]
    level = meter["integrated_loudness"]["value"]
    peak = meter["true_peak"]["value"]
    require(level["kind"] == peak["kind"] == "measured" and all(math.isfinite(v["value"]) for v in [level, peak]),
        "actual finite presented-window measurements required")
    target = round(level["value"] * 1000)
    maximum_peak = min(0, round(peak["value"] * 1000) + 1000)
    require(-70000 <= target <= -10000 and peak["value"] < 0, "bounded quiet synthetic fixture required")
    base_policy = {"schema": "aniflow.audio-delivery-policy/v1", "id": "synthetic_observed_comparison", "revision": "fixture.v1",
        "purpose": "Explicit synthetic numeric comparison only; no platform requirement, native meter or musical approval",
        "target_milli_lufs": target, "tolerance_milli_lu": 500, "maximum_true_peak_milli_dbtp": maximum_peak}
    native_input = {"kind": "accepted_video_with_qc", "run_directory": str(source_owner),
        "artifact_id": handoff["artifact_id"], "qc_directory": str(qc_directory), "qc_receipt_sha256": handoff["final_qc_receipt"]["sha256"]}
    configs = {}
    for name, change in [("match", {}), ("outside", {"target_milli_lufs": target + 10000}),
        ("stale-qc", {}), ("wrong-artifact", {}), ("legacy-qc", {})]:
        chosen_input = copy.deepcopy(native_input)
        if name == "stale-qc": chosen_input["qc_receipt_sha256"] = "0" * 64
        if name == "wrong-artifact": chosen_input["artifact_id"] = master["artifact_id"]
        if name == "legacy-qc":
            chosen_input.update(qc_directory=str(legacy_qc_directory), qc_receipt_sha256=handoff["legacy_qc_receipt"]["sha256"])
        selected = output / (name + "-owner-request.json")
        put(selected, {"schema": "aniflow.audio-policy-check-request/v1", "input": chosen_input,
            "policy": {**base_policy, **change}})
        configs[name] = pin(selected)

    def setup_request(label, config):
        return {"schema_version": "flow.audio-check-setup/v1", "package_manifest": manifest,
            "installation": {"root": native_path(subject.root), "receipt_sha256": subject.selection["installation_receipt_sha256"]},
            "workspace": native_path(output / (label + " setup 作品")),
            "selections": [{"payload_id": "clean-master", "destination": "archive", "choice": {"kind": "keep_archive_master",
                "reason": "Preserve the exact accepted archival master without a target-processing claim"}},
                *[{"payload_id": "delivery-video", "destination": destination,
                    "choice": {"kind": "check", "owner_request": config, "row_id": subject.selection["aniflow_policy_row_id"]}}
                    for destination in ["archive", "tiktok"]]],
            "tool_directories": handoff["tool_directories"], "limits": LIMITS, "timeout_ms": 240000}

    template = setup_request("match", configs["match"])
    controls = [("missing-coverage", lambda r: r["selections"].pop(), "audio setup must cover every assigned media payload and destination"),
        ("duplicate-coverage", lambda r: r["selections"].append(copy.deepcopy(r["selections"][-1])), "unique exact assigned audio payload/destination required"),
        ("unassigned-target", lambda r: r["selections"][-1].update(destination="soundcloud"), "unique exact assigned audio payload/destination required"),
        ("derivative-cannot-skip", lambda r: r["selections"][-1].update(choice={"kind": "keep_archive_master", "reason": "Not an archival master"}),
            "only an explicitly unchanged archival master/original can skip a policy check")]
    for label, mutate, message in controls:
        request = copy.deepcopy(template); mutate(request)
        request["workspace"] = native_path(output / (label + " setup"))
        file = output / (label + "-setup.json"); put(file, request)
        invoke(label, ["audio-setup-preview", "--request", file], 3, ("invalid_input", message, None))
        require(not Path(request["workspace"]["value"]).exists(), "refused setup preview wrote a workspace")

    def prepare(label, config):
        request = setup_request(label, config); file = output / (label + "-setup-request.json"); put(file, request)
        root = Path(request["workspace"]["value"])
        view = invoke(label + "-setup-preview", ["audio-setup-preview", "--request", file])
        require(not root.exists() and all(view[k] is False for k in ["provider_invoked", "approvals_generated", "native_qualified"]),
            "setup preview performed work or invented approval")
        prepared = invoke(label + "-setup-prepare", ["audio-setup-prepare", "--request", file])
        require(all(prepared[k] is False for k in ["provider_invoked", "approvals_generated", "native_qualified"])
            and len(prepared["plans"]) == 2 and not list((root / "approvals").iterdir())
            and not list((root / "native").iterdir()), "preparation invoked checks or created approvals")
        record_sha = prepared["record"]["sha256"]
        status = invoke(label + "-setup-status", ["audio-setup-status", "--workspace", root, "--record-sha256", record_sha])
        require(status == prepared, "read-only setup status differs")
        return root, record_sha, prepared

    def approvals(label, prepared, wrong=False):
        values = []
        for index, chosen in enumerate(prepared["plans"]):
            selected = prepared["plans"][0] if wrong else chosen
            file = Path(chosen["approval_file"]["value"])
            invoke(label + "-approve-" + str(index), ["approve", "--plan", selected["plan_file"]["path"]["value"],
                "--approved-by", "synthetic operator path execution acknowledgement",
                "--acknowledge-trusted-unconfined", "--acknowledge-unqualified-candidate", "--output", file])
            values.append({"payload_id": chosen["payload_id"], "destination": chosen["destination"],
                "authority": {"kind": "source_candidate", "approval": pin(file)}})
        return {"schema_version": "flow.audio-check-approvals/v1", "approvals": values}

    def bind(label, root, record_sha, supplied, code=0, diagnostic=None):
        selected, result = output / (label + "-approvals.json"), output / (label + "-readiness-request.json")
        put(selected, supplied)
        value = invoke(label + "-setup-bind", ["audio-setup-bind", "--workspace", root, "--record-sha256", record_sha,
            "--approvals", selected, "--output", result], code, diagnostic)
        if code:
            require(not result.exists(), "refused approval binding wrote readiness request")
        else:
            require(data(result) == value, "binding output is not its actual raw request")
        return result, value

    match_root, match_sha, prepared = prepare("match", configs["match"])
    empty = {"schema_version": "flow.audio-check-approvals/v1", "approvals": []}
    bind("missing-approval", match_root, match_sha, empty, 5,
        ("approval_required", "every prepared check requires a separately supplied exact approval", None))
    supplied = approvals("match", prepared)
    extra = copy.deepcopy(supplied); extra["approvals"].append({**extra["approvals"][0], "payload_id": "unassigned"})
    bind("extra-approval", match_root, match_sha, extra, 3,
        ("invalid_input", "supplied approval is not assigned to a prepared check", None))
    wrong_path = copy.deepcopy(supplied)
    external = output / "same-approval-wrong-path.json"
    external.write_bytes(OWNER_IO.regular(path(wrong_path["approvals"][0]["authority"]["approval"]["path"]), 1048576))
    wrong_path["approvals"][0]["authority"]["approval"] = pin(external)
    bind("wrong-path-approval", match_root, match_sha, wrong_path, 5,
        ("approval_required", "supplied approval must use the exact reserved path for this prepared check", None))
    match_request, _ = bind("match", match_root, match_sha, supplied)
    before_checks = inventory(match_root)
    preview = invoke("match-readiness-preview", ["audio-readiness-preview", "--request", match_request])
    scope_summary(preview, False, 0, base_policy)
    require(inventory(match_root) == before_checks, "readiness preview changed prepared state")
    match_summary_path = output / "match-audio-summary.json"
    match_summary = invoke("match-readiness-run", ["audio-readiness-run", "--request", match_request,
        "--output", match_summary_path], 9, outcome="partial")
    require(data(match_summary_path) == match_summary, "saved summary is not raw actual command result")
    scope_summary(match_summary, True, 2, base_policy)
    native_joins = native_record_joins(match_summary, qc, delivery["file"])
    completed_checks = inventory(match_root)
    invoke("repeat-current-attempt-refused", ["audio-readiness-run", "--request", match_request], 7,
        ("recovery_required", "a fresh current owner-check workspace is required; retained snapshots cannot substitute", None))
    require(inventory(match_root) == completed_checks, "refused repeated checks changed existing attempts")

    def publishing(label, summary_file, selected_manifest=manifest, code=9, diagnostic=None):
        file, result = output / (label + "-publishing-request.json"), output / (label + "-publishing-report.json")
        put(file, {"schema_version": "flow.publishing-readiness-request/v1", "package_manifest": selected_manifest,
            "audio_summary": pin(summary_file), "timeout_ms": 60000})
        value = invoke(label + "-publishing-readiness", ["publishing-readiness", "--request", file, "--output", result], code,
            diagnostic, "partial" if code == 9 else None)
        if diagnostic: require(not result.exists(), "refused publishing report wrote output")
        else: require(data(result) == value, "saved publishing report differs from actual result")
        return value

    matched_report = publishing("match", match_summary_path); report_scope(matched_report, match_summary)
    outside_root, outside_sha, outside_prepared = prepare("outside", configs["outside"])
    outside_request, _ = bind("outside", outside_root, outside_sha, approvals("outside", outside_prepared))
    outside_summary_path = output / "outside-audio-summary.json"
    outside_summary = invoke("outside-readiness-run", ["audio-readiness-run", "--request", outside_request,
        "--output", outside_summary_path], 9, outcome="partial")
    scope_summary(outside_summary, False, 2, {**base_policy, "target_milli_lufs": target + 10000})
    native_joins.extend(native_record_joins(outside_summary, qc, delivery["file"]))
    outside_report = publishing("outside", outside_summary_path); report_scope(outside_report, outside_summary)
    require(any(a["kind"] == "review_audio_goal" for a in outside_report["actions"]),
        "legitimate outside-target comparison lost its operator action")

    wrong_root, wrong_sha, wrong_prepared = prepare("wrong-plan", configs["match"])
    bind("wrong-plan", wrong_root, wrong_sha, approvals("wrong-plan", wrong_prepared, wrong=True), 5,
        ("approval_required", "supplied candidate approval belongs to another exact plan", None))
    inner = []
    for label, reason in [("stale-qc", "video QC receipt differs from external pin"),
        ("wrong-artifact", "final-file QC must bind the exact selected producer/artifact"),
        ("legacy-qc", "presented delivery loudness is unverified: current meter covers complete decoded AAC including nonpresented tail")]:
        root, record_sha, prep = prepare(label, configs[label])
        selected_request, _ = bind(label, root, record_sha, approvals(label, prep))
        native_root = Path(prep["plans"][0]["plan"]["request"]["run_directory"]["value"])
        invoke(label + "-owner-refusal", ["audio-readiness-run", "--request", selected_request], 6,
            ("provider_refused", "Aniflow native envelope is malformed", native_path(native_root)))
        require(not Path(prep["plans"][1]["plan"]["request"]["run_directory"]["value"]).exists(),
            "failed first owner check continued to a later payload/destination")
        inner.append(inner_refusal(native_root, reason))

    changed = copy.deepcopy(match_summary)
    next(row for row in changed["rows"] if row["payload_id"] == "delivery-video")["sha256"] = "0" * 64
    changed_file = output / "stale-file-summary.json"; put(changed_file, changed)
    publishing("stale-file-summary", changed_file, code=3,
        diagnostic=("stale_plan", "audio summary belongs to different exact delivery bytes", None))
    changed = copy.deepcopy(match_summary)
    next(row for row in changed["rows"] if row["payload_id"] == "delivery-video")["policy"]["target_milli_lufs"] += 1000
    changed_file = output / "stale-policy-summary.json"; put(changed_file, changed)
    publishing("stale-policy-summary", changed_file, code=3,
        diagnostic=("stale_plan", "audio summary differs from its exact retained owner evaluation", None))
    alternate = copy.deepcopy(delivery_request)
    alternate["metadata"]["description"] = "A second exact package context; prior summary cannot be reused"
    alternate["output_directory"] = native_path(output / "second private package 作品")
    alternate_file = output / "second-delivery-request.json"; put(alternate_file, alternate)
    second = invoke("second-delivery-prepare", ["delivery-prepare", "--request", alternate_file])
    publishing("different-package-summary", match_summary_path, second["manifest"], 3,
        ("stale_plan", "audio summary is not bound to the exact current package and declared snapshot scope", None))

    invoke("match-setup-status-after-checks", ["audio-setup-status", "--workspace", match_root, "--record-sha256", match_sha])
    require(inventory(package) == package_before, "audio setup/check/readback changed the copied package")
    require(all(inventory(p) == previous for p, previous in [(Path(k), v) for k, v in source_before.items()]),
        "original CP4 project/install/native/QC bytes changed during the operator path")
    require(pin(handoff_path) == handoff_pin and all(pin(members[name]) == handoff[name] for name in PIN_FIELDS),
        "CP4 handoff or selected immutable member changed")
    suite_after = subject.observe_status(output, "suite-after")
    require(suite_after == suite_before and helper_pins() == helper_before, "installed subjects or executed observer/helpers changed")
    captured_inventory = inventory(output)
    portable = {"schema_version": "flow.installed-audio-readiness-observation/v1", "status": "passed",
        "executed_at_utc": datetime.now(timezone.utc).isoformat(), "handoff_sha256": handoff_sha256,
        "observer_sources": helper_before, "flow_measured_build": subject.flow_identity,
        "current_source_context": subject.current_context,
        "providers": {tool: {"source_revision": subject.selection[tool + "_source_revision"],
            "source_tree": subject.selection[tool + "_source_tree"], "binary_sha256": subject.selection[tool + "_binary_sha256"],
            "contract_sha256": subject.selection[tool + "_contract_sha256"]} for tool in ["aniflow", "renderflow"]},
        "commands": observer.cases, "command_count": len(observer.cases), "suite_status_calls": 2,
        "exact_refusal_controls": len([c for c in observer.cases if c["diagnostic_code"]]),
        "payloads": [{k: item[k] for k in ["id", "role", "relative", "sha256", "byte_count", "destinations"]}
            for item in copied["preview"]["payloads"]],
        "source_sha256": delivery["file"]["sha256"], "window_pcm_sha256": window["window_pcm_sha256"],
        "complete_sample_frames": window["complete_sample_frames"], "presented_sample_frames": window["presented_sample_frames"],
        "actual_matching_checks": 2, "actual_outside_target_checks": 2, "native_record_joins": native_joins,
        "legacy_full_meter_policy_refused": True,
        "inner_refusals": inner, "package_sha256": manifest["sha256"],
        "matching_summary_sha256": pin(match_summary_path)["sha256"],
        "outside_summary_sha256": pin(outside_summary_path)["sha256"],
        "all_human_review_scopes_awaiting": REVIEWS, "meter_qualified": False,
        "provider_release_qualified": False, "current_platform_compliance_verified": False,
        "upload_authority": False, "publication_performed": False,
        "originals_and_installed_subjects_unchanged": True,
        "evidence_scope": "Actual selected current owner checks; later publishing report is a retained snapshot, not fresh QC, release admission, mastering or human approval",
        "inventory": captured_inventory}
    put(output / "portable-report.json", portable)
    put(output / "report.json", {"schema_version": "flow.installed-audio-readiness-private/v1", "portable": pin(output / "portable-report.json"),
        "handoff": handoff_pin, "work_directory": str(output), "matching_summary": pin(match_summary_path),
        "outside_summary": pin(outside_summary_path), "selected_files": {name: handoff[name] for name in PIN_FIELDS},
        "retained_original_inventories": source_before})
    return portable


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--handoff", required=True, type=Path)
    parser.add_argument("--handoff-sha256", required=True)
    parser.add_argument("--work-directory", required=True, type=Path)
    args = parser.parse_args()
    result = run(args.handoff, args.handoff_sha256, args.work_directory)
    print("installed_audio_readiness_passed commands=" + str(result["command_count"]) + " refusals=" + str(result["exact_refusal_controls"]))


if __name__ == "__main__":
    main()
