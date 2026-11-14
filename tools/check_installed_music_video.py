#!/usr/bin/env python3
"""Bounded actual installed Flow -> Aniflow source-candidate observation.

Synthetic ordinary MP4 only. Candidate execution acknowledgements are mechanical
execution choices; they are never human lyric/appearance/publishing approval.
"""
import argparse
import copy
from datetime import datetime, timezone
import os
from pathlib import Path
import signal
import subprocess
import time

from check_native_publication import inventory, pin, put, refusal
from check_released_read_only import OWNER_IO, canonical, command, document, envelope, native_path, require, sha
from installed_music_candidate_subjects import InstalledMusicCandidate

LIMITS = {"timeout_ms": 120000, "max_input_bytes": 2000000, "max_output_bytes": 16000000,
    "max_total_output_bytes": 32000000, "max_artifacts": 8}


def accepted(value, root, source, provider):
    require(value["schema_version"] in ["flow.native-receipt/v1", "flow.native-status/v1"]
        and value["state"] == "complete", "completed native music-video result required")
    receipt = value["receipt"] if value["schema_version"] == "flow.native-status/v1" else value
    native = Path(receipt["native_run"]["value"])
    require(native.is_relative_to(root / "native") and native != root / "native"
        and native.resolve(strict=True) == native and len(receipt["artifacts"]) == 2
        and all(receipt[flag] is False for flag in ["release_qualified", "creative_approval_inferred", "publication_authority"]),
        "confined native candidate completion with two final variants required")
    evidence_path = Path(receipt["acceptance_file"]["value"])
    require(evidence_path.is_relative_to(root) and evidence_path.resolve(strict=True) == evidence_path,
        "acceptance escaped selected run")
    raw = OWNER_IO.regular(evidence_path, 16777216)
    require(sha(raw) == receipt["acceptance_sha256"], "domain acceptance pin differs")
    evidence = document(raw)
    require(evidence["schema_version"] == "flow.aniflow-domain-acceptance/v1"
        and evidence["source_pins"] == [pin(source)] and evidence["native_status"]["complete"] is True
        and evidence["native_status"]["intent_sha256"] == receipt["native_intent_sha256"]
        and evidence["native_completion"]["schema"] == "aniflow.final-video-handoff/v1"
        and evidence["native_completion"]["publication_authority"] == "none"
        and evidence["native_completion"]["proof_state"] == "not_asserted",
        "Aniflow source/status/completion authority join differs")
    intent = evidence["native_plan"]["intent"]
    require(intent["schema"] == "aniflow.source-music-video-intent/v1"
        and sha(canonical(intent["payload"])) == intent["intent_sha256"] == receipt["native_intent_sha256"]
        and intent["payload"]["source_video"] == str(source)
        and intent["payload"]["provider_executable"] == str(provider)
        and document(OWNER_IO.regular(native / "intent.json", 16777216)) == intent
        and evidence["native_status"]["run_directory"] == str(native),
        "accepted native intent does not bind exact installed provider/source/run")
    declared = evidence["native_completion"]["artifacts"]
    require(len(declared) == 2 and {a["variant"] for a in declared} == {"clean_archival_master", "destination_copy"}
        and len({a["artifact_id"] for a in declared}) == 2, "two distinct accepted final roles required")
    for item, artifact in zip(declared, receipt["artifacts"]):
        path = Path(artifact["path"]["value"])
        require(path.is_relative_to(native) and path.resolve(strict=True) == path
            and pin(path) == artifact and item["locator"] == str(path)
            and item["sha256"] == artifact["sha256"] and item["byte_count"] == artifact["byte_count"],
            "completion/receipt/current payload byte join differs")
    require(document(OWNER_IO.regular(root / "accepted.json", 16777216)) == receipt,
        "final accepted record differs from validated receipt")
    return receipt


def cancel_actual_child(flow, plan, approval, root, logs, cwd):
    """Signal only an owned Flow process after its real transform child exists.

    This observes Flow's cooperative direct-child scope. It does not establish
    shutdown of descendants. Any harness timeout cleanup is stated separately.
    """
    argv = [str(flow), "run", "--plan", str(plan), "--approval", str(approval), "--format", "json"]
    stdout_path, stderr_path = logs / "cancel-run.stdout.json", logs / "cancel-run.stderr"
    deadline = time.monotonic() + 20
    signalled, child_seen, cleanup = False, [], False
    with stdout_path.open("xb") as out, stderr_path.open("xb") as err:
        with subprocess.Popen(argv, cwd=cwd, env={"PATH": "/usr/bin:/bin", "LC_ALL": "C.UTF-8", "NO_COLOR": "1"},
                stdin=subprocess.DEVNULL, stdout=out, stderr=err, start_new_session=True) as process:
            try:
                marker = root / "attempt-0000/transform-started.json"
                while process.poll() is None and time.monotonic() < deadline:
                    require(stdout_path.stat().st_size <= 16777216 and stderr_path.stat().st_size <= 1048576,
                        "bounded cancellation output exceeded")
                    if marker.exists():
                        children = Path(f"/proc/{process.pid}/task/{process.pid}/children")
                        child_seen = [int(n) for n in children.read_text().split()] if children.exists() else []
                        if child_seen:
                            # Require an actual selected Aniflow executable, not
                            # only a transform marker preceding child creation.
                            live = []
                            for pid in child_seen:
                                try:
                                    if Path(f"/proc/{pid}/exe").resolve(strict=True) == flow.parent / "aniflow": live.append(pid)
                                except FileNotFoundError: pass
                            if live:
                                child_seen = live; process.send_signal(signal.SIGINT); signalled = True; break
                    time.sleep(0.002)
                if signalled:
                    process.wait(timeout=max(0.001, deadline - time.monotonic()))
                else:
                    require(False, "actual selected transform child was not observed before completion/deadline")
            finally:
                if process.poll() is None:
                    cleanup = True
                    os.killpg(process.pid, signal.SIGKILL); process.wait()
            code = process.returncode
    require(stdout_path.stat().st_size <= 16777216 and stderr_path.stat().st_size <= 1048576,
        "bounded cancellation output exceeded")
    stdout, stderr = stdout_path.read_bytes(), stderr_path.read_bytes()
    value = envelope(stdout, stderr, code, "run", 130, "refused")
    refusal(value, ("cancelled", "native direct child cancelled; inspect retained attempt before explicit resume", native_path(root)))
    require(signalled and child_seen and not cleanup, "cancellation needed harness kill cleanup")
    return {"case": "cancel-actual-transform-child", "command": "run", "exit_code": code,
        "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr), "actual_selected_child_observed": True,
        "signal": "SIGINT", "harness_forced_cleanup": cleanup, "descendant_shutdown": "not_established"}


def run(selection_path, source_input, recipe_input, tool_directories, output):
    subject = InstalledMusicCandidate.load(selection_path)
    require(output.is_absolute() and output.parent.resolve() == output.parent and not output.exists(), "fresh canonical proof output required")
    output.mkdir(mode=0o700); logs = output / "commands"; logs.mkdir(mode=0o700)
    originals = {"source": pin(source_input), "recipe": pin(recipe_input)}
    source, recipe = output / "ordinary source 作品.mp4", output / "source-recipe.json"
    source.write_bytes(OWNER_IO.regular(source_input, 2000000)); recipe.write_bytes(OWNER_IO.regular(recipe_input, 131072))
    observation_before = subject.observe_status(output, "suite-before")
    cases = []

    def invoke(label, args, code=0, diagnostic=None, outcome=None):
        actual, stdout, stderr = command([str(subject.flow), *map(str, args), "--format", "json"], cwd=output,
            timeout=145, max_output=16777216)
        (logs / (label + ".stdout.json")).write_bytes(stdout); (logs / (label + ".stderr")).write_bytes(stderr)
        value = envelope(stdout, stderr, actual, args[0], code, outcome or ("complete" if code == 0 else "recovery_required" if code == 7 else "refused"))
        if diagnostic: refusal(value, diagnostic)
        else: require(not value["diagnostics"] and type(value["result"]) is dict, "unexpected native music-video diagnostic")
        cases.append({"case": label, "command": args[0], "exit_code": actual, "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr)})
        return value["result"]

    def planning(name):
        native_root = output / (name + "-run")
        request = {"schema_version": "flow.native-request/v1", "cohort": native_path(subject.cohort_path),
            "cohort_sha256": pin(subject.cohort_path)["sha256"], "row_id": subject.selection["aniflow_row_id"],
            "provider_executable": native_path(subject.aniflow), "release_descriptor": native_path(subject.aniflow_descriptor),
            "inputs": [native_path(source)], "configuration": native_path(recipe), "run_directory": native_path(native_root),
            "tool_directories": [native_path(p) for p in tool_directories], "limits": LIMITS}
        request_path, plan, approval = [output / (name + suffix + ".json") for suffix in ["-request", "-plan", "-approval"]]
        put(request_path, request)
        result = invoke(name + "-plan", ["plan", "--request", request_path, "--output", plan])
        require(not native_root.exists() and result["release_evidence"] == "operator_pinned_source_candidate; publication_and_qualification_not_established",
            "planning executed or promoted candidate")
        invoke(name + "-approve", ["approve", "--plan", plan, "--approved-by", "synthetic local execution observer",
            "--acknowledge-trusted-unconfined", "--acknowledge-unqualified-candidate", "--output", approval])
        return native_root, plan, approval

    root, plan, approval = planning("video")
    receipt = accepted(invoke("native-run", ["run", "--plan", plan, "--approval", approval]), root, source, subject.aniflow)
    before = inventory(root)
    require(accepted(invoke("native-status", ["status", "--run-directory", root]), root, source, subject.aniflow) == receipt, "status changed accepted video")
    native = Path(receipt["native_run"]["value"])
    require(accepted(invoke("completed-resume", ["resume", "--run-directory", root, "--native-run-directory", native]), root, source, subject.aniflow) == receipt,
        "completed resume changed receipt")
    require(inventory(root) == before, "completed resume/status replayed native work")
    invoke("existing-run-refused", ["run", "--plan", plan, "--approval", approval], 7,
        ("recovery_required", "fresh native output directory required; preserve existing state", None))
    invoke("wrong-resume-refused", ["resume", "--run-directory", root, "--native-run-directory", root / "native/not-selected"], 7,
        ("recovery_required", "resume native run differs from accepted operation", None))
    for label, path in [("source", source), ("recipe", recipe), ("accepted-payload", Path(receipt["artifacts"][0]["path"]["value"]))]:
        raw = OWNER_IO.regular(path, 16000000); path.write_bytes(raw + b" ")
        invoke("changed-" + label, ["status", "--run-directory", root], 3,
            ("stale_plan", "native pinned bytes changed; create a new reviewed plan", native_path(root)))
        path.write_bytes(raw)
    saved = source.with_suffix(".saved"); source.rename(saved)
    invoke("missing-source", ["status", "--run-directory", root], 3,
        ("stale_plan", "pinned file or release cohort could not be reobserved", native_path(root)))
    saved.rename(source)
    require(inventory(root) == before, "restored controls changed completed native state")
    cancel_root, cancel_plan, cancel_approval = planning("cancel")
    cases.append(cancel_actual_child(subject.flow, cancel_plan, cancel_approval, cancel_root, logs, output))
    partial = invoke("cancelled-retained-status", ["status", "--run-directory", cancel_root], outcome="recovery_required")
    require(partial["state"] == "recovery_required" and partial["receipt"] is None
        and partial["domain_evidence"] == "no_committed_domain_acceptance", "cancelled state claimed native completion")
    require(accepted(invoke("final-complete-status", ["status", "--run-directory", root]), root, source, subject.aniflow) == receipt,
        "negative controls changed retained successful completion")
    observation_after = subject.observe_status(output, "suite-after")
    require({"source": pin(source_input), "recipe": pin(recipe_input)} == originals, "original inputs changed")
    report = {"schema_version": "flow.installed-music-video-candidate-proof/v1", "status": "passed",
        "observed_utc": datetime.now(timezone.utc).isoformat(), "driver_sha256": pin(Path(__file__).resolve())["sha256"],
        "selection_sha256": subject.selection_sha256, "flow_build": subject.flow_identity,
        "aniflow_candidate": subject.aniflow_identity, "commands": cases, "suite_before": observation_before,
        "suite_after": observation_after, "original_inputs": {k: {field: value[field] for field in ["sha256", "byte_count"]} for k, value in originals.items()},
        "accepted_receipt_sha256": pin(root / "accepted.json")["sha256"],
        "artifacts": [{"sha256": a["sha256"], "byte_count": a["byte_count"]} for a in receipt["artifacts"]],
        "completed_work_reused": True, "human_reviews": "awaiting", "descendant_shutdown": "not_established",
        "release_published": False, "publisher_authenticated": False, "native_matrix_qualified": False,
        "publication_authority": False, "hosted_ci_dispatched": False}
    put(output / "report.json", report)
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["selection", "source", "recipe", "output"]: parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--tool-directory", action="append", type=Path, required=True)
    args = parser.parse_args()
    print(canonical(run(args.selection, args.source, args.recipe, args.tool_directory, args.output)).decode())
