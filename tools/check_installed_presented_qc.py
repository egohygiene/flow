#!/usr/bin/env python3
"""Actual installed project -> explicitly selected final-video QC observation.

Inputs are redistribution-safe authored video/manual-SRT fixtures. Execution
acknowledgements are mechanical authority, never human review or publication.
"""
import argparse
import copy
from datetime import datetime, timezone
from fractions import Fraction
import math
from pathlib import Path
import struct
import sys

from check_native_publication import pin, put, refusal
from check_installed_music_video import inventory
from check_released_read_only import OWNER_IO, canonical, command, document, envelope, native_path, require, sha
from installed_audio_candidate_subjects import InstalledAudioCandidate
from installed_candidate_subjects import PACKAGE

LIMITS = {"timeout_ms": 120000, "max_input_bytes": 2000000, "max_output_bytes": 16000000,
    "max_total_output_bytes": 32000000, "max_artifacts": 16}
OBSERVER_FILES = ("tools/check_installed_presented_qc.py", "tools/installed_audio_candidate_subjects.py",
    "tools/installed_music_candidate_subjects.py", "tools/installed_candidate_subjects.py",
    "tools/check_installed_music_video.py", "tools/check_native_publication.py",
    "tools/check_released_read_only.py", "scripts/owner_reports.py", "scripts/release/package.py")


def observed_helpers(repository):
    # These two modules use importlib without sys.modules registration. Bind
    # their actual loaded file identities explicitly, alongside ordinary imports.
    modules = list(sys.modules.values()) + [OWNER_IO, PACKAGE]
    return {Path(module.__file__).resolve() for module in modules
        if getattr(module, "__file__", None) and Path(module.__file__).resolve().is_relative_to(repository)
        and Path(module.__file__).suffix == ".py"}


def float_wav(path):
    raw = OWNER_IO.regular(path, 2097152)
    require(raw[:4] == b"RIFF" and raw[8:12] == b"WAVE" and len(raw) >= 44
        and int.from_bytes(raw[4:8], "little") + 8 == len(raw), "complete bounded RIFF envelope required")
    chunks = {}; offset = 12
    while offset < len(raw):
        require(offset + 8 <= len(raw), "truncated RIFF chunk header")
        tag = raw[offset:offset + 4]; length = int.from_bytes(raw[offset + 4:offset + 8], "little")
        end = offset + 8 + length
        require(tag not in chunks and end <= len(raw), "duplicate or truncated RIFF chunk")
        chunks[tag] = raw[offset + 8:end]; offset = end + (length & 1)
    require(offset == len(raw) and set(chunks) == {b"fmt ", b"fact", b"data"}
        and len(chunks[b"fmt "]) in [16, 18] and len(chunks[b"fact"]) == 4
        and (len(chunks[b"fmt "]) == 16 or chunks[b"fmt "][16:] == b"\0\0"),
        "native Float32 WAV chunks required")
    tag, channels, rate, byte_rate, align, bits = struct.unpack("<HHIIHH", chunks[b"fmt "][:16])
    pcm = chunks[b"data"]
    require(tag == 3 and channels in [1, 2] and rate == 48000 and bits == 32
        and align == channels * 4 and byte_rate == rate * align and len(pcm) % align == 0
        and int.from_bytes(chunks[b"fact"], "little") == len(pcm) // align
        and all(math.isfinite(v[0]) for v in struct.iter_unpack("<f", pcm)),
        "finite complete native Float32 PCM/clock required")
    return raw, pcm, len(pcm) // align, channels


def qc_subject(receipt, configuration, source, selected):
    root = Path(receipt["native_run"]["value"])
    path = Path(receipt["artifacts"][0]["path"]["value"])
    report = document(OWNER_IO.regular(path, 16777216))
    record_path = Path(receipt["acceptance_file"]["value"])
    accepted = document(OWNER_IO.regular(record_path, 16777216))
    owner, handoff = accepted["native_plan"], accepted["native_handoff"]
    native = owner["payload"]["native"]
    resumable_digest = sha(canonical([native[key] for key in
        ["schema", "request", "native", "components", "projected_working_bytes"]]))
    require(receipt["schema_version"] == "flow.native-receipt/v1" and receipt["state"] == "complete"
        and len(receipt["artifacts"]) == 1 and path == root / "qc.json"
        and pin(path) == receipt["artifacts"][0] and pin(record_path)["sha256"] == receipt["acceptance_sha256"]
        and accepted["schema_version"] == "flow.aniflow-audio-processing-acceptance/v1"
        and accepted["source_pins"] == [pin(source)]
        and owner["payload"]["profile"] == "final-video-audio-qc-v1"
        and owner["payload"]["request"] == configuration
        and owner["payload"]["source_files"] == [{"locator": str(source),
            "sha256": pin(source)["sha256"], "byte_count": pin(source)["byte_count"]}]
        and sha(canonical(owner["payload"])) == owner["plan_sha256"]
        and sha(canonical(handoff["payload"])) == handoff["handoff_sha256"]
        and handoff["payload"]["plan"] == owner
        and handoff["payload"]["native_intent_sha256"] == receipt["native_intent_sha256"]
        and resumable_digest == native["intent_sha256"] == receipt["native_intent_sha256"]
        and handoff["payload"]["report"] == report
        and handoff["payload"]["artifacts"] == [{"id": "final_audio_qc", "purpose": "supporting_document",
            "file": {"locator": str(path), "sha256": pin(path)["sha256"], "byte_count": pin(path)["byte_count"]}}]
        and report["intent"] == native["native"]
        and sha(canonical(report["intent"])) == report["intent_sha256"]
        and report["intent"]["request"] == configuration["qc"]
        and report["intent"]["artifact"]["locator"] == str(source)
        and report["intent"]["artifact"]["sha256"] == pin(source)["sha256"]
        and report["intent"]["artifact"]["byte_count"] == pin(source)["byte_count"]
        and report["runtime_qualified"] is False and report["publication_authority"] == "none"
        and report["normalization"] == "none"
        and all(receipt[key] is False for key in ["release_qualified", "creative_approval_inferred", "publication_authority"]),
        "current Flow/native QC request, source, report or authority join differs")
    signal = report["signal"]; full_wav = root / "signal/selected-audio.wav"
    full_raw, full_pcm, full_frames, channels = float_wav(full_wav)
    tail = signal["decoded_audio_tail"]
    def moment(value): return Fraction(value["numerator"], value["denominator"])
    extent = signal["source_audio_extent"]
    require(extent == report["intent"]["artifact"]["audio"]["interval"]
        and moment(extent["start"]) == 0 and moment(extent["end"]) == 2
        and (moment(extent["end"]) - moment(extent["start"])) * 48000 == 96000,
        "actual source extent differs from the authored two-second fixture")
    require(signal["source_sha256"] == pin(source)["sha256"] and signal["source_bytes"] == pin(source)["byte_count"]
        and signal["decoded_wav_sha256"] == sha(full_raw) and signal["decoded_pcm_sha256"] == sha(full_pcm)
        and signal["frames"] == full_frames == tail["decoded_sample_frames"]
        and signal["channels"] == channels and signal["sample_rate_hz"] == 48000
        and tail["metric_scope"] == "complete_decoded_pcm_including_nonpresented_tail"
        and tail["schema"] == "aniflow.aac-decoded-tail/v1"
        and tail["priming_sample_frames"] == 1024
        and tail["decoded_block_count"] * 1024 == full_frames
        and tail["presented_sample_frames"] == 96000
        and tail["final_presented_sample_frames"] + tail["nonpresented_tail_sample_frames"] == 1024
        and tail["presented_sample_frames"] + tail["nonpresented_tail_sample_frames"] == full_frames
        and moment(tail["decoded_extent"]["start"]) == 0
        and moment(tail["decoded_extent"]["end"]) * 48000 == full_frames
        and signal["loudness"]["frames"] == full_frames and signal["loudness"]["pcm_sha256"] == sha(full_pcm)
        and signal["loudness"]["runtime_qualified"] is False,
        "complete decoded PCM/native full meter changed")
    result = {"source_sha256": pin(source)["sha256"], "source_bytes": pin(source)["byte_count"],
        "complete_sample_frames": full_frames, "presented_sample_frames": tail["presented_sample_frames"],
        "complete_pcm_sha256": sha(full_pcm), "full_meter": signal["loudness"],
        "request_selected_window": selected, "native_meter_qualified": False}
    if selected:
        view = signal["presented_window"]
        raw, pcm, frames, view_channels = float_wav(root / "signal/presented-window/selected-audio.wav")
        presented = tail["presented_sample_frames"]; start = view["start_sample_frame"]
        require(view["schema"] == "aniflow.presented-audio-window/v1"
            and view["metric_scope"] == "exact_presented_audio_window" and start == 0
            and configuration["qc"]["presented_window"] == "exact_presented_audio"
            and view["source_sha256"] == result["source_sha256"] and view["source_bytes"] == result["source_bytes"]
            and view["complete_pcm_sha256"] == sha(full_pcm) and view["complete_sample_frames"] == full_frames
            and view["source_audio_extent"] == extent and view["temporal_sha256"] == signal["temporal_sha256"]
            and view["channels"] == channels and view["sample_rate_hz"] == 48000
            and view["method"] == "exact_float32_sample_bytes_from_complete_pcm; no_decode_gain_resample_or_delivery_edit"
            and view["presented_sample_frames"] == frames == presented and 0 < frames < full_frames
            and view_channels == channels and view["window_wav_sha256"] == sha(raw)
            and view["window_wav_bytes"] == len(raw) and view["window_pcm_sha256"] == sha(pcm)
            and pcm == full_pcm[:frames * channels * 4]
            and view["loudness"]["frames"] == frames and view["loudness"]["pcm_sha256"] == sha(pcm)
            and view["loudness"]["runtime_qualified"] is False and view["runtime_qualified"] is False
            and view["normalization"] == "none", "selected analysis is not the exact retained presented PCM prefix")
        excluded = full_pcm[frames * channels * 4:]
        nonzero = sum(v[0] != 0 for v in struct.iter_unpack("<f", excluded))
        require(nonzero > 0 and len(excluded) == (full_frames - frames) * channels * 4,
            "authored fixture must retain an actual nonzero decoded tail")
        result.update(window_pcm_sha256=sha(pcm), window_wav_sha256=sha(raw),
            window_bytes=len(raw), nonpresented_tail_frames=full_frames - frames,
            nonzero_tail_channel_values=nonzero, measurements=view["loudness"])
    else:
        require("presented_window" not in signal and "presented_window" not in configuration["qc"],
            "omitted selection silently adopted a presented analysis view")
    return report, record_path, root, result


def run(selection_path, source_input, recipe_input, output, handoff_path):
    subject = InstalledAudioCandidate.load(selection_path)
    require(output.is_absolute() and output.parent.resolve() == output.parent and not output.exists(),
        "fresh canonical installed-QC evidence root required")
    require(handoff_path.is_absolute() and handoff_path.parent.resolve() == handoff_path.parent
        and not handoff_path.exists(), "fresh canonical external CP4 handoff required")
    output.mkdir(mode=0o700); logs = output / "commands"; logs.mkdir(mode=0o700)
    source, recipe, srt = output / "synthetic source 作品.mp4", output / "source-recipe.json", output / "manual lyrics.srt"
    original = {"source": pin(source_input), "recipe": pin(recipe_input)}
    source.write_bytes(OWNER_IO.regular(source_input, 2000000))
    recipe.write_bytes(OWNER_IO.regular(recipe_input, 131072))
    srt.write_bytes(b"1\n00:00:00,000 --> 00:00:01,000\nAuthored synthetic first line\n\n2\n00:00:01,000 --> 00:00:02,000\nManual timing remains unreviewed\n")
    initial_inputs = {str(p): pin(p) for p in [source, recipe, srt]}
    repository = Path(__file__).resolve().parents[1]
    helper_paths = [repository / name for name in OBSERVER_FILES]
    require(observed_helpers(repository) == set(helper_paths), "complete named observer helper closure differs")
    helper_pins = {p.relative_to(repository).as_posix(): pin(p) for p in helper_paths}; cases = []
    before_suite = subject.observe_status(output, "suite-before")
    tool_directories = [Path(document(OWNER_IO.regular(recipe, 131072))["extraction"]["ffmpeg"]["executable"]).parent]

    def invoke(label, args, code=0, diagnostic=None, outcome=None):
        actual, stdout, stderr = command([str(subject.flow), *map(str, args), "--format", "json"],
            cwd=output, timeout=360, max_output=16777216)
        (logs / (label + ".stdout.json")).write_bytes(stdout)
        (logs / (label + ".stderr")).write_bytes(stderr)
        put(logs / (label + ".exit.json"), {"exit_code": actual})
        expected = outcome or {0: "complete", 3: "refused", 4: "unavailable", 5: "refused", 7: "recovery_required", 9: "partial"}.get(code)
        value = envelope(stdout, stderr, actual, args[0], code, expected)
        if diagnostic: refusal(value, diagnostic)
        else: require(value["diagnostics"] == [] and type(value["result"]) is dict, "unexpected installed-QC diagnostic")
        cases.append({"case": label, "command": args[0], "exit_code": actual, "outcome": expected,
            "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr)})
        return value["result"]

    workspace = output / "project 作品"
    project = {"schema_version": "flow.publishing-project/v1",
        "installation": {"root": native_path(subject.root), "receipt_sha256": subject.selection["installation_receipt_sha256"]},
        "metadata": {"work_id": "synthetic-presented-qc", "title": "Authored installed audio pilot", "language": "en"},
        "video": native_path(source), "manual_srt": native_path(srt), "video_configuration": native_path(recipe),
        "audio": None, "destinations": ["archive", "tiktok"], "workspace": native_path(workspace),
        "tool_directories": [native_path(p) for p in tool_directories], "limits": LIMITS,
        "rows": {"video": subject.selection["aniflow_row_id"], "timed_text": subject.selection["renderflow_row_id"], "audio": None},
        "preparation": None}
    project_path = output / "project.json"; put(project_path, project)
    preview = invoke("project-preview", ["project-preview", "--project", project_path])
    require(not workspace.exists() and preview["provider_invoked"] is False, "project preview invoked provider or wrote workspace")
    prepared = invoke("project-prepare", ["project-prepare", "--project", project_path])
    record = Path(prepared["record"]["path"]["value"]); plan = Path(prepared["pilot_plan"]["path"]["value"])
    require(pin(record) == prepared["record"] and pin(plan) == prepared["pilot_plan"], "prepared project pins differ")
    approval = output / "pilot-approval.json"
    invoke("pilot-approve", ["pilot-approve", "--plan", plan, "--approved-by", "user-authorized synthetic mechanical execution",
        "--acknowledge-trusted-unconfined", "--acknowledge-unqualified-candidate", "--output", approval])
    handoff = invoke("pilot-run", ["pilot-run", "--plan", plan, "--approval", approval])
    handoff_file = workspace / "pilot/handoff.json"
    require(handoff["schema_version"] == "flow.publishing-pilot-handoff/v1"
        and [stage["role"] for stage in handoff["stages"]] == ["video", "timed_text"]
        and handoff["review_pending"] is True and handoff["upload_authority"] is False
        and handoff["release_qualified"] is False and document(OWNER_IO.regular(handoff_file, 16777216)) == handoff,
        "two-stage authored project inferred release/review authority")
    video_receipt = handoff["stages"][0]["receipt"]
    video_record = Path(video_receipt["acceptance_file"]["value"])
    domain = document(OWNER_IO.regular(video_record, 16777216))
    owners = [p for p in domain["native_status"]["phases"] if p["phase"] == "assembly"]
    deliveries = [a for a in domain["native_completion"]["artifacts"] if a["variant"] == "destination_copy"]
    require(len(owners) == len(deliveries) == 1, "one exact accepted assembly/delivery required")
    source_owner, artifact = owners[0]["run_directory"], deliveries[0]
    delivery = Path(artifact["locator"])
    require(pin(delivery)["sha256"] == artifact["sha256"], "new final video changed before QC")
    invoke("project-status", ["project-status", "--workspace", workspace, "--record-sha256", pin(record)["sha256"]])
    invoke("pilot-status", ["pilot-status", "--run-directory", workspace / "pilot"])
    qc_results = {}
    for label, selected in [("selected", True), ("legacy", False)]:
        configuration = {"schema": "aniflow.resumable-final-video-audio-qc-request/v1", "qc": {
            "schema": "aniflow.final-audio-qc-request/v1", "source_run_directory": source_owner,
            "artifact_id": artifact["artifact_id"], "configuration": document(OWNER_IO.regular(recipe, 131072))["recipe"]["assembly"],
            "provider_executable": str(subject.aniflow), "maximum_pcm_bytes": 2097152}}
        if selected: configuration["qc"]["presented_window"] = "exact_presented_audio"
        config_path, root = output / (label + "-qc-request.json"), output / (label + " QC 作品")
        put(config_path, configuration)
        request = {"schema_version": "flow.native-request/v1", "cohort": native_path(subject.cohort_path),
            "cohort_sha256": pin(subject.cohort_path)["sha256"], "row_id": subject.selection["aniflow_final_qc_row_id"],
            "provider_executable": native_path(subject.aniflow), "release_descriptor": native_path(subject.aniflow_descriptor),
            "inputs": [native_path(delivery)], "configuration": native_path(config_path), "run_directory": native_path(root),
            "tool_directories": [native_path(p) for p in tool_directories], "limits": LIMITS}
        request_path, plan_path, approval_path = [output / (label + suffix) for suffix in ["-native-request.json", "-plan.json", "-approval.json"]]
        put(request_path, request)
        planned = invoke(label + "-qc-prepare", ["plan", "--request", request_path, "--output", plan_path])
        require(not root.exists() and planned["request"] == request, "native QC preparation performed execution or changed selection")
        invoke(label + "-qc-approve", ["approve", "--plan", plan_path, "--approved-by", "user-authorized synthetic mechanical execution",
            "--acknowledge-trusted-unconfined", "--acknowledge-unqualified-candidate", "--output", approval_path])
        receipt = invoke(label + "-qc-run", ["run", "--plan", plan_path, "--approval", approval_path])
        report, native_record, qc_root, summary = qc_subject(receipt, configuration, delivery, selected)
        before = inventory(root)
        status = invoke(label + "-qc-status", ["status", "--run-directory", root])
        require(status["receipt"] == receipt, "retained QC status receipt differs")
        resumed = invoke(label + "-qc-completed-resume", ["resume", "--run-directory", root, "--native-run-directory", qc_root])
        require(resumed == receipt and inventory(root) == before, "completed QC status/resume replayed or changed native state")
        qc_results[label] = {"configuration": configuration, "configuration_path": config_path, "root": root,
            "qc_root": qc_root, "receipt": receipt, "native_record": native_record, "report": report,
            "summary": summary, "plan": planned}
    require(qc_results["selected"]["summary"]["complete_pcm_sha256"] == qc_results["legacy"]["summary"]["complete_pcm_sha256"]
        and qc_results["selected"]["summary"]["full_meter"] == qc_results["legacy"]["summary"]["full_meter"]
        and qc_results["selected"]["plan"]["plan_sha256"] != qc_results["legacy"]["plan"]["plan_sha256"],
        "explicit selected window changed full PCM/meter or failed to bind a distinct native plan")
    chosen = qc_results["selected"]
    before = inventory(chosen["root"])
    for label, path, replacement in [
        ("changed-window-selection", chosen["configuration_path"], canonical(qc_results["legacy"]["configuration"]) + b"\n"),
        ("changed-final-video", delivery, OWNER_IO.regular(delivery, 16000000) + b" "),
        ("changed-qc-report", chosen["qc_root"] / "qc.json", OWNER_IO.regular(chosen["qc_root"] / "qc.json", 16777216) + b" ")]:
        saved = OWNER_IO.regular(path, 16777216)
        path.write_bytes(replacement)
        try:
            invoke(label, ["status", "--run-directory", chosen["root"]], 3,
                ("stale_plan", "native pinned bytes changed; create a new reviewed plan", native_path(chosen["root"])))
        finally: path.write_bytes(saved)
        restored = invoke(label + "-restored", ["status", "--run-directory", chosen["root"]])
        require(restored["receipt"] == chosen["receipt"], "restored positive QC readback differs")
    require(inventory(chosen["root"]) == before, "negative controls changed retained completed QC")
    after_suite = subject.observe_status(output, "suite-after")
    require({str(p): pin(p) for p in [source, recipe, srt]} == initial_inputs
        and {"source": pin(source_input), "recipe": pin(recipe_input)} == original,
        "original selected fixtures changed")
    require(observed_helpers(repository) == set(helper_paths)
        and {p.relative_to(repository).as_posix(): pin(p) for p in helper_paths} == helper_pins,
        "observer helper bytes changed during execution")
    provider_identities = {"flow": subject.flow_identity, "aniflow": subject.aniflow_identity,
        "renderflow": subject.render_identity, "optiflow": {"version": "0.1.1", "sha256": pin(subject.optiflow)["sha256"]}}
    report = {"schema_version": "flow.installed-presented-qc-observation/v1", "state": "passed",
        "observed_utc": datetime.now(timezone.utc).isoformat(), "observer_files": helper_pins,
        "selection_sha256": subject.selection_sha256, "providers": provider_identities,
        "current_source_context": subject.current_context, "commands": cases,
        "suite_before": before_suite, "suite_after": after_suite,
        "selected": chosen["summary"], "legacy": qc_results["legacy"]["summary"],
        "full_pcm_and_meter_preserved": True, "completed_resume_reused": True,
        "project_record_sha256": pin(record)["sha256"], "pilot_handoff_sha256": pin(handoff_file)["sha256"],
        "final_qc_receipt_sha256": pin(chosen["qc_root"] / "qc.json")["sha256"],
        "retained_inventory": inventory(output), "human_reviews": "awaiting",
        "provider_release_qualified": False, "meter_qualified": False, "upload_authority": False,
        "scope": "fresh installed source-candidate project and selected native final-file QC; no release/native/human qualification or publication"}
    report_path = output / "report.json"; put(report_path, report)
    handshake = {"schema_version": "flow.installed-presented-qc-handoff/v1", "selection": pin(selection_path),
        "context": pin(Path(subject.selection["flow_context_equivalence"])), "project_record": pin(record),
        "pilot_handoff": pin(handoff_file), "manual_srt": pin(srt), "recipe": pin(recipe), "source_video": pin(source),
        "final_qc_receipt": pin(chosen["qc_root"] / "qc.json"),
        "legacy_qc_receipt": pin(qc_results["legacy"]["qc_root"] / "qc.json"),
        "final_qc_native_record": pin(chosen["native_record"]),
        "video_native_receipt": pin(workspace / "native/video/accepted.json"),
        "driver": pin(Path(__file__).resolve()), "report": pin(report_path),
        "final_qc_directory": str(chosen["qc_root"]), "source_run_directory": source_owner,
        "artifact_id": artifact["artifact_id"], "tool_directories": [native_path(p) for p in tool_directories],
        "provider_identities": provider_identities, "current_context": subject.current_context}
    put(handoff_path, handshake)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ["selection", "source", "recipe", "work-directory", "handoff"]:
        parser.add_argument("--" + option, type=Path, required=True)
    args = parser.parse_args()
    value = run(args.selection, args.source, args.recipe, args.work_directory, args.handoff)
    print(canonical({"state": value["state"], "commands": len(value["commands"]),
        "final_qc_receipt_sha256": value["final_qc_receipt_sha256"]}).decode())
