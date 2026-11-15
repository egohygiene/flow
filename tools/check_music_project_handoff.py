#!/usr/bin/env python3
"""Existing preset/project/pilot -> private delivery, with all human reviews pending."""
import argparse
import copy
from datetime import datetime, timezone
from pathlib import Path
import os
import shutil
import math

from check_native_publication import pin, put, refusal
from check_installed_music_video import inventory
from check_released_read_only import OWNER_IO, canonical, command, document, envelope, native_path, require, sha
from installed_music_candidate_subjects import InstalledMusicCandidate

LIMITS = {"timeout_ms": 120000, "max_input_bytes": 2000000, "max_output_bytes": 16000000,
    "max_total_output_bytes": 32000000, "max_artifacts": 32}
ANI_TOOLS = {"ffmpeg": "098c4ba3d9d7ca067bc9bfa004a33b5b91cf0913fa9380bcda7f3975c214380b",
    "ffprobe": "31054f1d68b781089f98f594bb304cf5c9ae96dadafafdd021719ab4101a79be"}
RENDER_TOOLS = {"ffmpeg": "94dbc5c9d7602f044b0ec9089cea2085d0f2adc3a2b1612eeec84f8badc6949d",
    "ffprobe": "40d68a7e8fc61e8656be62b12179142124ebd61eea5ed8afefda6e2d68b97a1d"}


def qc_subject(value, artifact, provider_sha):
    signal = value["signal"]; tail = signal["decoded_audio_tail"]
    require(value["schema"] == "aniflow.final-audio-qc/v1"
        and value["intent"]["artifact"]["sha256"] == signal["source_sha256"] == artifact["sha256"]
        and value["intent"]["artifact"]["byte_count"] == signal["source_bytes"] == artifact["byte_count"]
        and signal["provider_sha256"] == provider_sha and signal["ffmpeg_sha256"] == ANI_TOOLS["ffmpeg"]
        and signal["ffprobe_sha256"] == ANI_TOOLS["ffprobe"]
        and signal["frames"] == tail["decoded_sample_frames"] == 96256
        and tail["presented_sample_frames"] == 96000 and tail["nonpresented_tail_sample_frames"] == 256
        and tail["metric_scope"] == "complete_decoded_pcm_including_nonpresented_tail"
        and signal["decoded_pcm_sha256"] == "b70f04f7cd3c64fbb6c80784e69f5d10ef29c5df3643695dfe04be47d0c2a4ee"
        and signal["sample_rate_hz"] == 48000 and signal["channels"] == 2
        and signal["loudness"]["frames"] == value["accepted_pcm"]["source"]["frame_count"] == 96256
        and signal["loudness"]["sample_rate_hz"] == 48000 and signal["loudness"]["channels"] == 2
        and signal["loudness"]["pcm_sha256"] == signal["decoded_pcm_sha256"]
        and signal["loudness"]["runtime_qualified"] is False
        and value["accepted_pcm"]["source"]["artifact"]["sha256"] == signal["decoded_wav_sha256"]
        and value["accepted_pcm"]["source"]["artifact"]["byte_size"] == signal["decoded_wav_bytes"]
        and value["normalization"] == signal["normalization"] == "none"
        and value["runtime_qualified"] is False and signal["runtime_qualified"] is False
        and value["creative_approval"] == "not_performed; measurements are not musical quality review"
        and value["publication_authority"] == "none"
        and all(value["accepted_pcm"][key]["value"] == {"kind": "unavailable", "reason": "unsupported_native_signal_profile"}
            for key in ["integrated_loudness", "true_peak"]),
        "fresh QC does not bind exact final bytes and honest complete-decoded scope")
    measurements = {}
    for key, unit in [("integrated_loudness", "loudness_units_full_scale"), ("true_peak", "decibels_true_peak")]:
        selected = signal["loudness"][key]
        require(selected["unit"] == unit and selected["value"]["kind"] == "measured"
            and type(selected["value"]["value"]) in [int, float] and math.isfinite(selected["value"]["value"]),
            "complete-decoded meter returned missing/nonfinite measurement")
        measurements[key] = selected["value"]["value"]
    return {"source_sha256": artifact["sha256"], "source_bytes": artifact["byte_count"],
        "decoded_sample_frames": 96256, "presented_sample_frames": 96000, "nonpresented_tail_sample_frames": 256,
        "metric_scope": tail["metric_scope"], "measurements": measurements,
        "presented_delivery_loudness_verified": False, "native_meter_qualified": False,
        "normalization": "none", "human_review_performed": False}


def exact_final_qc(subject, receipt, recipe_path, output, logs):
    domain = document(OWNER_IO.regular(Path(receipt["acceptance_file"]["value"]), 16777216))
    assembly = [p for p in domain["native_status"]["phases"] if p["phase"] == "assembly"]
    artifacts = [a for a in domain["native_completion"]["artifacts"] if a["variant"] == "destination_copy"]
    require(len(assembly) == len(artifacts) == 1, "one accepted assembly/destination required for fresh final QC")
    source_owner, artifact = assembly[0]["run_directory"], artifacts[0]
    require(pin(Path(artifact["locator"]))["sha256"] == artifact["sha256"], "exact final payload changed before QC")
    request = {"schema": "aniflow.final-audio-qc-request/v1", "source_run_directory": source_owner,
        "artifact_id": artifact["artifact_id"], "configuration": document(OWNER_IO.regular(recipe_path, 131072))["recipe"]["assembly"],
        "provider_executable": str(subject.aniflow), "maximum_pcm_bytes": 2097152}
    request_path, qc = output / "final-qc-request.json", output / "exact final QC 作品"
    put(request_path, request); cases = []

    def native(label, command_name, args, code=0, message=None):
        actual, stdout, stderr = command([str(subject.aniflow), "--output", "json", *map(str, args)], cwd=output, timeout=145)
        (logs / (label + ".stdout.json")).write_bytes(stdout); (logs / (label + ".stderr.json")).write_bytes(stderr)
        if code:
            value = document(stderr)
            require(actual == code and stdout == b"" and value == {"schema_version": 1, "command": command_name,
                "status": "error", "error": {"category": "state", "message": message}}, "exact native QC refusal domain/code/reason differs")
            result = None
        else:
            value = document(stdout)
            require(actual == 0 and not stderr and value["schema_version"] == 1 and value["command"] == command_name
                and value["status"] == "success" and type(value["result"]) is dict and value.get("error") is None,
                "native exact-final QC envelope differs")
            result = value["result"]
        cases.append({"case": label, "command": command_name, "exit_code": actual,
            "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr), "tool": "installed_aniflow"})
        return result

    native("exact-final-qc-plan", "delivery_audio_qc_plan", ["delivery", "audio-qc", "plan", "--request", request_path])
    require(not qc.exists(), "QC planning created runtime directory")
    result = native("exact-final-qc-run", "delivery_audio_qc_run", ["delivery", "audio-qc", "run", "--request", request_path, "--output-directory", qc])
    qc_path = qc / "qc.json"
    saved = document(OWNER_IO.regular(qc_path, 16777216))
    require(saved == result and saved["intent"]["request"] == request, "saved fresh QC/request differs")
    measured = qc_subject(saved, artifact, subject.selection["aniflow_binary_sha256"])
    readback = native("exact-final-qc-status", "delivery_audio_qc_status", ["delivery", "audio-qc", "status", "--directory", qc,
        "--receipt-sha256", pin(qc_path)["sha256"]])
    require(readback == result, "fresh exact-final QC readback differs")
    native("wrong-final-qc-pin", "delivery_audio_qc_status", ["delivery", "audio-qc", "status", "--directory", qc,
        "--receipt-sha256", "0"*64], 8, "final audio QC receipt differs from the external digest")
    policy_path = output / "presented-policy-request.json"
    put(policy_path, {"schema": "aniflow.audio-policy-check-request/v1", "input": {"kind": "accepted_video_with_qc",
        "run_directory": source_owner, "artifact_id": artifact["artifact_id"], "qc_directory": str(qc), "qc_receipt_sha256": pin(qc_path)["sha256"]},
        "policy": {"schema": "aniflow.audio-delivery-policy/v1", "id": "synthetic_scope_only", "revision": "fixture.v1",
            "purpose": "Synthetic explicit comparison only; no platform requirement or musical approval", "target_milli_lufs": -14000,
            "tolerance_milli_lu": 500, "maximum_true_peak_milli_dbtp": -1000}})
    native("presented-policy-scope-refusal", "delivery_audio_policy_check", ["delivery", "audio-policy", "check", "--request", policy_path],
        8, "presented delivery loudness is unverified: current meter covers complete decoded AAC including nonpresented tail")
    require(pin(Path(artifact["locator"]))["sha256"] == artifact["sha256"], "QC changed exact accepted final payload")
    return qc_path, measured, cases


def tool_selection(recipe_path, directories, output, label):
    selected = document(OWNER_IO.regular(recipe_path, 131072))
    configured = [selected["extraction"], selected["recipe"]["assembly"]]
    configured.append(selected["recipe"]["repair"]["import"])
    for item in configured:
        for name in ["ffmpeg", "ffprobe"]:
            if name not in item: continue
            declared = item[name]
            require(declared["sha256"] == ANI_TOOLS[name] and declared["version"] == "8.0.1"
                and Path(declared["executable"]).is_absolute()
                and pin(Path(declared["executable"]))["sha256"] == ANI_TOOLS[name],
                "Ani recipe selected a different tool variant")
    search = os.pathsep.join(map(str, directories)); resolved = {}
    for name in ["ffmpeg", "ffprobe"]:
        found = shutil.which(name, path=search)
        require(found is not None and pin(Path(found))["sha256"] == RENDER_TOOLS[name],
            "Render audio PATH first-match selected a different tool variant")
        code, stdout, stderr = command([found, "-version"], cwd=output, timeout=10)
        require(code == 0 and not stderr and stdout.startswith((name + " version 8.0.1 ").encode()), "exact selected audio tool version differs")
        (output / (label + "-" + name + "-version.stdout")).write_bytes(stdout)
        resolved[name] = {"sha256": pin(Path(found))["sha256"], "byte_count": pin(Path(found))["byte_count"], "version_stdout_sha256": sha(stdout)}
    found = shutil.which("ffmpeg", path=search)
    code, stdout, stderr = command([found, "-hide_banner", "-encoders"], cwd=output, timeout=10)
    require(code == 0 and not stderr and all(any(line.split()[1:2] == [codec] for line in stdout.decode().splitlines())
        for codec in ["flac", "alac", "libmp3lame", "aac"]), "selected Render audio features differ")
    (output / (label + "-render-audio-encoders.stdout")).write_bytes(stdout)
    return {"aniflow_recipe_absolute_executable_sha256": ANI_TOOLS, "render_audio_first_path_match": resolved,
        "render_encoder_inventory_sha256": sha(stdout), "scope": "explicit absolute Ani configuration and separately resolved Render PATH; no process-image sampling claim"}


def pending(value):
    require(value["reviews"]["state"] == "review_pending"
        and value["reviews"]["pending"] == ["words", "timing", "visual", "audio", "final_package"]
        and value["reviews"]["changes_requested"] == []
        and value["reviews"]["identity_authenticated"] is False,
        "private package must preserve missing human reviews")
    require(value["publishing_authority"] is False and value["current_platform_compliance_verified"] is False,
        "private byte package cannot authorize publication/compliance")


def run(selection_path, video, srt, recipe, audio, audio_options, tool_directories, output):
    subject = InstalledMusicCandidate.load(selection_path)
    require(output.is_absolute() and output.parent.resolve() == output.parent and not output.exists(), "fresh canonical project proof root required")
    output.mkdir(mode=0o700); logs = output / "commands"; logs.mkdir(mode=0o700)
    inputs = dict(video=video, manual_srt=srt, recipe=recipe, audio=audio, audio_options=audio_options)
    original = {k: pin(p) for k, p in inputs.items()}
    selected_tools = tool_selection(recipe, tool_directories, output, "before")
    subject.observe_status(output, "suite-before")
    cases = []

    def invoke(label, args, code=0, diagnostic=None, outcome="complete"):
        actual, stdout, stderr = command([str(subject.flow), *map(str, args), "--format", "json"], cwd=output,
            timeout=360, max_output=16777216)
        (logs / (label + ".stdout.json")).write_bytes(stdout); (logs / (label + ".stderr")).write_bytes(stderr)
        value = envelope(stdout, stderr, actual, args[0], code, outcome)
        if diagnostic: refusal(value, diagnostic)
        else: require(not value["diagnostics"] and type(value["result"]) is dict, "unexpected project/delivery diagnostic")
        cases.append({"case": label, "command": args[0], "exit_code": actual, "outcome": outcome,
            "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr)})
        return value["result"]

    preset = {"schema_version": "flow.publishing-project-preset/v1", "preset_id": "synthetic-manual-amv",
        "installation": {"root": native_path(subject.root), "receipt_sha256": subject.selection["installation_receipt_sha256"]},
        "video_configuration": native_path(recipe), "destinations": ["archive", "tiktok"],
        "tool_directories": [native_path(p) for p in tool_directories], "limits": LIMITS,
        "rows": {"video": subject.selection["aniflow_row_id"], "timed_text": subject.selection["renderflow_row_id"],
            "audio": subject.selection["renderflow_row_id"]}, "preparation": None}
    selected = {"schema_version": "flow.publishing-project-selection/v1",
        "metadata": {"work_id": "synthetic-akashic-pilot", "title": "Synthetic ordinary-input AMV", "language": "en"},
        "video": native_path(video), "manual_srt": native_path(srt),
        "audio": {"path": native_path(audio), "format": "wav", "options": native_path(audio_options)},
        "workspace": native_path(output / "project work 作品"), "destinations": None, "rows": None, "video_configuration": None}
    preset_path, selection, project = [output / name for name in ["preset.json", "song-selection.json", "project.json"]]
    put(preset_path, preset); put(selection, selected)
    resolution = invoke("resolve-inherited", ["project-resolve", "--preset", preset_path, "--selection", selection, "--output", project])
    resolved = resolution["resolution"]
    require(resolution["project_file"] == pin(project), "resolved project file pin differs")
    require(resolved["media_inspected"] is False and resolved["provider_invoked"] is False and resolved["approvals_generated"] is False,
        "preset resolution inspected/invoked/approved media")
    override = copy.deepcopy(selected); override["destinations"] = ["archive", "soundcloud", "distrokid", "tiktok"]
    override_path, effective = output / "song-selection-override.json", output / "effective-project.json"
    put(override_path, override)
    changed_command = invoke("resolve-whole-destinations-override", ["project-resolve", "--preset", preset_path, "--selection", override_path, "--output", effective])
    changed = changed_command["resolution"]
    require(changed_command["project_file"] == pin(effective), "overridden project file pin differs")
    require(changed["project_sha256"] != resolved["project_sha256"] and changed["project"]["destinations"] == override["destinations"]
        and changed["project"]["rows"] == preset["rows"] and changed["overridden"] == ["destinations"],
        "whole-field override identity/inheritance differs")
    preview = invoke("project-preview", ["project-preview", "--project", effective])
    workspace = Path(override["workspace"]["value"])
    require(not workspace.exists() and preview["provider_invoked"] is False and preview["native_qualified"] is False
        and preview["review_pending"] and [r["role"] for r in preview["selected_rows"]] == ["video", "timed_text", "audio"],
        "read-only project preview scope differs")
    prepared = invoke("project-prepare", ["project-prepare", "--project", effective])
    require(prepared["provider_invoked"] is False and prepared["native_qualified"] is False,
        "project preparation invoked/qualified provider")
    record = Path(prepared["record"]["path"]["value"]); plan = Path(prepared["pilot_plan"]["path"]["value"])
    require(pin(record) == prepared["record"] and pin(plan) == prepared["pilot_plan"], "prepared file pins differ")
    invoke("project-status", ["project-status", "--workspace", workspace, "--record-sha256", prepared["record"]["sha256"]])
    approval = output / "pilot-approval.json"
    invoke("pilot-approve", ["pilot-approve", "--plan", plan, "--approved-by", "synthetic local execution observer",
        "--acknowledge-trusted-unconfined", "--acknowledge-unqualified-candidate", "--output", approval])
    handoff = invoke("pilot-run", ["pilot-run", "--plan", plan, "--approval", approval])
    require(handoff["schema_version"] == "flow.publishing-pilot-handoff/v1" and handoff["review_pending"] is True
        and handoff["upload_authority"] is False and handoff["release_qualified"] is False
        and [s["role"] for s in handoff["stages"]] == ["video", "timed_text", "audio"],
        "pilot did not retain three unreviewed candidate stages")
    handoff_path = workspace / "pilot/handoff.json"
    require(document(OWNER_IO.regular(handoff_path, 16777216)) == handoff, "saved pilot handoff differs")
    for stage in handoff["stages"]:
        receipt = stage["receipt"]
        native_run = Path(receipt["native_run"]["value"])
        stage_root = workspace / "native" / ("timed-text" if stage["role"] == "timed_text" else stage["role"])
        require(receipt["state"] == "complete" and native_run.is_relative_to(stage_root / "native")
            and all(receipt[flag] is False for flag in ["release_qualified", "creative_approval_inferred", "publication_authority"]),
            "stage completion scope/authority differs")
        current = invoke("stage-status-" + stage["role"], ["status", "--run-directory", stage_root])
        require(current["receipt"] == receipt, "stage status does not match pilot handoff")
        for artifact in receipt["artifacts"]:
            path = Path(artifact["path"]["value"])
            require(path.is_relative_to(native_run) and path.resolve(strict=True) == path and pin(path) == artifact,
                "stage handoff/current payload pin differs")
        if stage["role"] == "video":
            domain = document(OWNER_IO.regular(Path(receipt["acceptance_file"]["value"]), 16777216))
            intent_config = domain["native_plan"]["intent"]["payload"]["configuration"]
            selected_recipe = document(OWNER_IO.regular(recipe, 131072))
            for kind in ["extraction", "assembly"]:
                observed = intent_config[kind] if kind == "extraction" else intent_config["recipe"][kind]
                supplied = selected_recipe[kind] if kind == "extraction" else selected_recipe["recipe"][kind]
                require(all(observed[name] == supplied[name] for name in ["ffmpeg", "ffprobe"]),
                    "actual native Ani intent changed explicit tool pins")
    before = inventory(workspace)
    status = invoke("pilot-status", ["pilot-status", "--run-directory", workspace / "pilot"])
    require(status["handoff"] == handoff, "pilot status changed accepted handoff")
    continued = invoke("pilot-continue-complete", ["pilot-continue", "--plan", plan, "--approval", approval])
    require(continued == handoff and inventory(workspace) == before, "completed pilot-continue replayed/changed stages")
    video_receipt = next(stage["receipt"] for stage in handoff["stages"] if stage["role"] == "video")
    qc_path, measurement, native_cases = exact_final_qc(subject, video_receipt, recipe, output, logs)
    catalog = invoke("delivery-catalog", ["delivery-catalog", "--project-record", record,
        "--project-record-sha256", pin(record)["sha256"], "--handoff", handoff_path, "--handoff-sha256", pin(handoff_path)["sha256"]])
    artifacts = catalog["artifacts"]
    require(catalog["native_release_admission_reverified"] is False
        and len([a for a in artifacts if a["purpose"] == "clean_archival_master"]) == 1,
        "delivery catalog inferred/requalified native identities")
    payloads = []
    for item in artifacts:
        stage, purpose = item["stage"], item["purpose"]
        if stage == "video":
            role = "clean_master" if purpose == "clean_archival_master" else "video_delivery"
            destinations = ["archive"] if role == "clean_master" else ["tiktok"]
        elif stage == "audio": role, destinations = "audio_derivative", ["archive", "soundcloud", "distrokid"]
        else: role, destinations = "lyrics", ["archive"]
        payloads.append({"id": "native-" + str(len(payloads) + 1), "role": role,
            "source": {"kind": "native", "stage": stage, "artifact_id": item["artifact_id"]},
            "file_name": stage + "-" + str(len(payloads) + 1) + Path(item["file"]["path"]["value"]).suffix,
            "destinations": destinations})
    payloads.extend([
        {"id": "original-audio", "role": "original_audio", "source": {"kind": "original_audio"}, "file_name": "original-mix.wav", "destinations": ["archive"]},
        {"id": "manual-srt", "role": "lyrics", "source": {"kind": "manual_srt"}, "file_name": "manual-lyrics.srt", "destinations": ["archive"]}])
    package = output / "private delivery 作品"
    request = {"schema_version": "flow.delivery-request/v1", "project_record": pin(record), "handoff": pin(handoff_path),
        "payloads": payloads, "metadata": {"creators": [], "description": "Synthetic mechanical pilot; human reviews pending", "tags": ["synthetic"], "destination_fields": []},
        "reviews": [], "evidence_attachments": [pin(qc_path)], "output_directory": native_path(package),
        "limits": {"timeout_ms": 60000, "maximum_payload_bytes": 16000000, "maximum_total_bytes": 64000000}}
    request_path = output / "delivery-request.json"; put(request_path, request)
    delivery_preview = invoke("delivery-preview", ["delivery-preview", "--request", request_path]); pending(delivery_preview)
    require(not package.exists(), "delivery preview created package")
    copied = invoke("delivery-prepare", ["delivery-prepare", "--request", request_path]); pending(copied["preview"])
    require(copied["current_copied_bytes"] is True, "new package bytes are not current")
    manifest_sha = copied["manifest"]["sha256"]
    for item in copied["preview"]["payloads"]:
        actual = pin(package / item["relative"])
        require(actual["sha256"] == item["sha256"] and actual["byte_count"] == item["byte_count"], "copied delivery payload differs")
    invoke("delivery-status", ["delivery-status", "--directory", package, "--manifest-sha256", manifest_sha])
    readiness_request = output / "readiness-request.json"
    put(readiness_request, {"schema_version": "flow.publishing-readiness-request/v1", "package_manifest": pin(package / "manifest.json"), "audio_summary": None, "timeout_ms": 60000})
    readiness = invoke("publishing-readiness", ["publishing-readiness", "--request", readiness_request], code=9, outcome="partial")
    require(readiness["reviews"]["state"] == "review_pending"
        and readiness["reviews"]["pending"] == ["words", "timing", "visual", "audio", "final_package"]
        and readiness["upload_authority"] is False
        and readiness["current_platform_compliance_verified"] is False and readiness["current_owner_checks_performed"] is False
        and readiness["actions"] and any(a["kind"] == "review" for a in readiness["actions"]),
        "readiness promoted missing human review or final destination QC")
    missing_checks = output / "missing-audio-checks.json"
    put(missing_checks, {"schema_version": "flow.audio-readiness-request/v1", "package_manifest": pin(package / "manifest.json"), "selections": [], "timeout_ms": 60000})
    invoke("audio-readiness-needs-exact-selections", ["audio-readiness-preview", "--request", missing_checks], 3,
        ("invalid_input", "every assigned audio/video payload and destination needs an explicit check or archival-master choice", None), "refused")
    moved = output / "moved private delivery 作品"; package.rename(moved)
    moved_status = invoke("moved-delivery-status", ["delivery-status", "--directory", moved, "--manifest-sha256", manifest_sha])
    require(moved_status["current_copied_bytes"] is True and moved_status["preview"] == copied["preview"], "moved private package failed relative readback")
    attached = moved / "evidence/attachments/00.data"
    require(OWNER_IO.regular(attached, 16777216) == OWNER_IO.regular(qc_path, 16777216)
        and pin(attached)["sha256"] == pin(qc_path)["sha256"], "copied QC attachment differs from exact fresh receipt")
    delivered = [p for p in moved_status["preview"]["payloads"] if p["role"] == "video_delivery"]
    require(len(delivered) == 1 and delivered[0]["sha256"] == measurement["source_sha256"]
        and delivered[0]["byte_count"] == measurement["source_bytes"]
        and pin(moved / delivered[0]["relative"])["sha256"] == measurement["source_sha256"],
        "private video delivery differs from fresh exact-QC subject")
    require(inventory(workspace) == before and {k: pin(p) for k, p in inputs.items()} == original, "project/originals changed during packaging/readiness")
    subject.observe_status(output, "suite-after")
    require(tool_selection(recipe, tool_directories, output, "after") == selected_tools,
        "tool variant resolution/features changed during the actual project")
    report = {"schema_version": "flow.installed-music-project-handoff-proof/v1", "status": "passed",
        "observed_utc": datetime.now(timezone.utc).isoformat(), "driver_sha256": pin(Path(__file__).resolve())["sha256"],
        "selection_sha256": subject.selection_sha256, "flow_build": subject.flow_identity, "aniflow_candidate": subject.aniflow_identity,
        "renderflow_candidate": subject.render_identity, "commands": cases,
        "installed_aniflow_commands": native_cases, "fresh_exact_final_qc": measurement,
        "fresh_qc_receipt_sha256": pin(qc_path)["sha256"],
        "tool_selection": selected_tools,
        "preset_sha256": resolved["preset_sha256"], "base_project_sha256": resolved["project_sha256"],
        "effective_project_sha256": changed["project_sha256"], "prepared_record_sha256": pin(record)["sha256"],
        "handoff_sha256": pin(handoff_path)["sha256"], "delivery_manifest_sha256": manifest_sha,
        "payload_count": len(payloads), "completed_stages_reused": True, "moved_package_current": True,
        "originals_preserved": True, "human_reviews": "awaiting", "readiness": "partial; final destination QC and human review remain required",
        "release_published": False, "publisher_authenticated": False, "native_matrix_qualified": False,
        "publication_authority": False, "hosted_ci_dispatched": False}
    put(output / "report.json", report)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["selection", "video", "srt", "recipe", "audio", "audio-options", "output"]: parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--tool-directory", type=Path, action="append", required=True)
    args = parser.parse_args()
    print(canonical(run(args.selection, args.video, args.srt, args.recipe, args.audio, args.audio_options, args.tool_directory, args.output)).decode())
