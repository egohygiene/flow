#!/usr/bin/env python3
"""Actual single-locale static projects over installed candidate Renderflow.

Reuses native plan/approval/run/status and the existing installation receipt.
Each format is a separate native policy; no unsupported combined route is invented.
"""
import argparse
import copy
from datetime import datetime, timezone
import json
from pathlib import Path
import time

import check_native_publication as native


def run_pilot(flow, installation, receipt_sha, img2pdf, seed, output, provenance):
    provenance = native.public_candidate_build(provenance)
    data = native.build_binary(flow)
    native.require({"sha256": native.sha(data), "bytes": len(data)} == provenance["binary"], "Flow build differs")
    native.require(output.is_absolute() and output.parent.resolve(strict=True) == output.parent, "canonical fresh parent required")
    output.mkdir(mode=0o700)
    copied = output / "flow"; copied.write_bytes(data); copied.chmod(0o700); flow = copied
    logs = output / "commands"; logs.mkdir(mode=0o700)
    source = output / "source 作品 with spaces"; source.mkdir(mode=0o700)
    sources = [source / "page 001.png", source / "page 002.png"]
    for p, color in zip(sources, [(30, 70, 130), (180, 90, 40)]): p.write_bytes(native.png(color))
    originals = [native.pin(p) for p in sources]
    seed_report = native.document(native.OWNER_IO.regular(seed / "report.json", 1048576))
    native.require(seed_report["schema_version"] == "flow.native-publication-cli-pilot/v1"
        and seed_report["status"] == "passed" and seed_report["provider"]["sha256"] == native.PROVIDER_SHA,
        "selected prior direct candidate evidence required")
    observations, branches = [], []
    started = time.monotonic()
    def invoke(label, args, code=0, outcome="complete", diagnostic=None):
        remaining = 240 - (time.monotonic() - started)
        native.require(remaining > 0, "static pilot aggregate deadline exceeded")
        actual, stdout, stderr = native.command([str(flow), *args, "--format", "json"], cwd=output,
            timeout=min(75, remaining), max_output=16777216)
        (logs / (label + ".stdout.json")).write_bytes(stdout); (logs / (label + ".stderr")).write_bytes(stderr)
        value = native.envelope(stdout, stderr, actual, args[0], code, outcome)
        if diagnostic: native.refusal(value, diagnostic)
        else: native.require(value["diagnostics"] == [] and isinstance(value["result"], dict), "unexpected static diagnostic")
        observations.append({"case": label, "command": args[0], "exit_code": actual, "outcome": outcome,
            "stdout_sha256": native.sha(stdout), "stdout_bytes": len(stdout), "stderr_bytes": len(stderr)})
        return value["result"]
    installed = invoke("installation-current", ["suite-status", "--installation-root", str(installation), "--receipt-sha256", receipt_sha])
    native.require(installed["current_copied_bytes"] is True and installed["receipt"]["candidate_acknowledged"] is True
        and installed["receipt"]["publisher_authenticated"] is False and installed["receipt"]["native_qualified"] is False,
        "candidate install became runtime/release qualification")
    for fmt in ["epub", "pdf"]:
        seed_plan = native.document(native.OWNER_IO.regular(seed / (fmt + "-run/attempt-0000/native-plan.json"), 16777216))
        targets = [b for b in seed_plan["artifact_forest"]["branches"] if b["state"] == "selected"]
        native.require(len(targets) == 1 and targets[0]["format"] == fmt, "prior native target selection differs")
        target_id = targets[0]["target_id"]
        workspace = output / (fmt + "-project")
        config = native.configuration(sources, output / (fmt + "-unused"), fmt, img2pdf)
        if fmt == "epub":
            config["execution"]["fixed_layout_epub"]["cover_member_id"] = "page001"
        options = {"schema_version": "flow.renderflow-options/v1", "targets": config["targets"], "execution": config["execution"]}
        if fmt == "epub": options["publication"] = native.publication(sources)
        options_path = output / (fmt + "-options.json"); native.put(options_path, options)
        project = {"schema_version": "flow.static-publication-project/v1", "metadata": {
            "work_id": "synthetic-static", "title": "Two synthetic pages", "language": "en-US"},
            "installation": {"root": native.native_path(installation), "receipt_sha256": receipt_sha},
            "workspace": native.native_path(workspace), "row_id": native.ROW,
            "branches": [{"id": "en-US", "locale": "en-US", "sources": [{"id": "page%03d" % i,
                "file": native.pin(p), "format": "png", "media_type": "image/png", "geometry": config["sources"][i-1]["geometry"]}
                for i, p in enumerate(sources, 1)], "options": native.pin(options_path), "required_artifacts": [target_id]}],
            "tool_directories": [] if fmt == "epub" else [native.native_path(img2pdf.parent)],
            "limits": native.LIMITS, "timeout_ms": 60000}
        project_path = output / (fmt + "-project.json"); native.put(project_path, project)
        preview = invoke(fmt + "-preview", ["publication-project-preview", "--project", str(project_path)])
        native.require(not workspace.exists() and preview["provider_invoked"] is False and preview["native_qualified"] is False,
            "preview wrote private copies or qualified provider")
        bad = copy.deepcopy(project); bad["branches"][0]["sources"][0]["geometry"]["width"] = -1
        bad_path = output / (fmt + "-bad-geometry.json"); native.put(bad_path, bad)
        invoke(fmt + "-bad-geometry", ["publication-project-preview", "--project", str(bad_path)], 3, "refused",
            ("invalid_input", "unique ordered source IDs/paths and explicit native formats required", None))
        bad = copy.deepcopy(project); bad["branches"][0]["sources"][0].pop("media_type")
        bad_path = output / (fmt + "-missing-media.json"); native.put(bad_path, bad)
        invoke(fmt + "-missing-media", ["publication-project-preview", "--project", str(bad_path)], 3, "refused",
            ("invalid_input", "unique ordered source IDs/paths and explicit native formats required", None))
        if fmt == "epub":
            unselected = copy.deepcopy(options); unselected["publication"]["artwork"][0]["path"] = str(output / "unselected.png")
            bad_options = output / "unselected-artwork-options.json"; native.put(bad_options, unselected)
            bad = copy.deepcopy(project); bad["branches"][0]["options"] = native.pin(bad_options)
            bad_path = output / "unselected-artwork-project.json"; native.put(bad_path, bad)
            invoke("unselected-artwork", ["publication-project-preview", "--project", str(bad_path)], 3, "refused",
                ("invalid_input", "publication artwork must name an explicitly selected source", None))
        native.require(not workspace.exists(), "refused preview wrote a workspace")
        prepared = invoke(fmt + "-prepare", ["publication-project-prepare", "--project", str(project_path)])
        native.require(prepared["provider_invoked"] is False and prepared["approvals_generated"] is False
            and prepared["native_qualified"] is False and len(prepared["branches"]) == 1, "preparation gained execution authority")
        branch = prepared["branches"][0]; plan = branch["plan"]
        staged = [Path(p["value"]) for p in plan["request"]["inputs"]]
        native.require(all(p.is_relative_to(workspace / "configuration") for p in staged)
            and [(native.pin(p)["sha256"], native.pin(p)["byte_count"]) for p in staged]
                == [(p["sha256"], p["byte_count"]) for p in originals], "staged member bytes/order differ")
        configuration = native.document(native.OWNER_IO.regular(Path(plan["request"]["configuration"]["value"]), 1048576))
        native.require([s["geometry"] for s in configuration["sources"][:2]] == [s["geometry"] for s in project["branches"][0]["sources"]]
            and all(s["media_type"] == "image/png" for s in configuration["sources"][:2]), "native geometry/media declarations lost")
        if fmt == "epub":
            native.require([a["path"] for a in configuration["publication"]["artwork"]]
                == [s["path"] for s in configuration["sources"][:2]], "artwork locators were not rebound to selected copies")
            native.require(configuration["publication"]["rights"] == options["publication"]["rights"]
                and configuration["publication"]["accessibility"] == options["publication"]["accessibility"], "supplied metadata changed")
        run = Path(plan["request"]["run_directory"]["value"])
        plan_path = Path(branch["plan_file"]["path"]["value"]); approval = output / (fmt + "-approval.json")
        invoke(fmt + "-approve", ["approve", "--plan", str(plan_path), "--approved-by", "synthetic installed publication operator",
            "--acknowledge-trusted-unconfined", "--acknowledge-unqualified-candidate", "--output", str(approval)])
        value = invoke(fmt + "-run", ["run", "--plan", str(plan_path), "--approval", str(approval)])
        receipt, summary = native.accepted(value, run, fmt, staged)
        if fmt == "pdf":
            artifact = Path(receipt["artifacts"][0]["path"]["value"])
            code, stdout, stderr = native.command([str(img2pdf.parent / "python"), "-c",
                "import json,pikepdf,sys; p=pikepdf.open(sys.argv[1]); print(json.dumps({'pages':len(p.pages),'boxes':[[float(v) for v in page.MediaBox] for page in p.pages]}))",
                str(artifact)], cwd=output, timeout=15)
            native.require(code == 0 and stderr == b"", "independent PDF inspector failed")
            native.pdf_geometry(native.document(stdout)); summary["inspection_stdout_sha256"] = native.sha(stdout)
        args = ["--workspace", str(workspace), "--record-sha256", prepared["record"]["sha256"]]
        status = invoke(fmt + "-status", ["publication-project-status", *args], 9, "partial")
        native.require(status["all_required_artifacts_observed"] is True and status["creative_review_pending"] is True
            and status["release_admission_reverified"] is False and status["upload_authority"] is False
            and status["branches"][0]["state"] == "native_accepted" and status["branches"][0]["missing_required_artifacts"] == [],
            "accepted native branch became human/release authority or missed target")
        catalog = invoke(fmt + "-catalog", ["publication-project-catalog", *args], 9, "partial")
        native.require(catalog["locale_content_reviewed"] is False and catalog["upload_authority"] is False
            and len(catalog["artifacts"]) == 1 and catalog["artifacts"][0]["required"] is True
            and catalog["artifacts"][0]["file"] == receipt["artifacts"][0], "catalog does not join accepted target/current bytes")
        original = native.OWNER_IO.regular(staged[0], 1000000); staged[0].write_bytes(original + b" ")
        invoke(fmt + "-stale-copy", ["publication-project-status", *args], 3, "refused",
            ("stale_plan", "native pinned bytes changed; create a new reviewed plan", None))
        staged[0].write_bytes(original)
        restored = invoke(fmt + "-final-status", ["publication-project-status", *args], 9, "partial")
        native.require(restored == status and [native.pin(p) for p in sources] == originals, "restoration changed source or project state")
        summary.update(locale="en-US", required_target_id=target_id, original_source_bytes_unchanged=True,
            staged_sources_match=True, creative_review_pending=True, status_outcome="partial")
        branches.append(summary)
    return {"schema_version": "flow.static-publication-cli-pilot/v1", "status": "passed",
        "observed_utc": datetime.now(timezone.utc).isoformat(), "flow_build": provenance,
        "driver_sha256": native.pin(Path(__file__).resolve())["sha256"],
        "observer_dependencies": {"native_publication_sha256": native.pin(Path(native.__file__).resolve())["sha256"],
            "shared_cli_observer_sha256": native.pin(Path(native.public_build_identity.__code__.co_filename).resolve())["sha256"],
            "owner_io_sha256": native.pin(Path(native.OWNER_IO.__file__).resolve())["sha256"]},
        "seed_report_sha256": native.pin(seed / "report.json")["sha256"], "installation_receipt_sha256": receipt_sha,
        "commands": observations, "branches": branches, "elapsed_seconds": round(time.monotonic()-started, 6),
        "human_creative_approval": False, "release_authenticated": False, "native_matrix_qualified": False,
        "hosted_ci_dispatched": False, "scope": "two separate single-locale synthetic candidate policies; not released-suite or human qualification"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("flow", "installation", "img2pdf", "native-pilot", "flow-build-evidence", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--receipt-sha256", required=True)
    args = parser.parse_args()
    result = run_pilot(args.flow, args.installation, args.receipt_sha256, args.img2pdf, args.native_pilot,
        args.output, native.document(native.OWNER_IO.regular(args.flow_build_evidence, 65536)))
    native.put(args.output / "report.json", result)
    print(json.dumps({"status": result["status"], "branches": result["branches"], "seconds": result["elapsed_seconds"]}))


if __name__ == "__main__": main()
