#!/usr/bin/env python3
"""Bounded synthetic Flow -> independently captured Renderflow candidate proof.

Uses the existing candidate adapter, not released execution or human approval.
No acquisition, signing, upload, model or source checkout is needed at runtime.
"""
from __future__ import annotations

import argparse
import copy
import hashlib
from datetime import datetime, timezone
import io
import json
import math
import os
from pathlib import Path
import posixpath
import stat
import struct
import time
import zipfile
import zlib
from xml.etree import ElementTree as ET

from check_released_read_only import (OWNER_IO, build_binary, canonical, command,
    document, envelope, native_path, public_build_identity, require, sha)

TARGET = "x86_64-unknown-linux-gnu"
ROW = "renderflow.canonical-build.source-candidate." + TARGET
PROVIDER_SHA = "b0a55e600e7888428b6971b99dc4566eac285cebf7338b99d6ed75415d1a3a56"
CONTRACT_SHA = "9ff99a410bcf98a82801eeadf3e35fdc40539c36d192ce066365769c98e9aa74"
PROVIDER_SOURCE = "c8f2b6896ec82c12e75ad6b8bcf7e4b319602db2"
LIMITS = {"timeout_ms": 60000, "max_input_bytes": 1000000, "max_output_bytes": 5000000,
          "max_total_output_bytes": 10000000, "max_artifacts": 8}


def put(path, value):
    with path.open("xb") as file:
        file.write(canonical(value) + b"\n")


def pin(path):
    data = OWNER_IO.regular(path, 83886080)
    return {"path": native_path(path), "sha256": sha(data), "byte_count": len(data)}


def inventory(root, *, maximum_member_bytes=16777216, maximum_total_bytes=None, maximum_entries=None):
    result = {}
    total = 0; entries = 0
    for path in root.rglob("*"):
        entries += 1
        require(maximum_entries is None or entries <= maximum_entries, "native state entry bound exceeded")
        before = path.lstat()
        require(not stat.S_ISLNK(before.st_mode), "native state contains a symlink")
        if stat.S_ISDIR(before.st_mode):
            continue
        total += before.st_size
        require(stat.S_ISREG(before.st_mode) and before.st_nlink == 1 and before.st_size <= maximum_member_bytes
                and (maximum_total_bytes is None or total <= maximum_total_bytes),
                "bounded regular native state required")
        # Empty lock/stderr files are valid state. The held descriptor still
        # prevents following a substituted path while comparing content.
        with os.fdopen(os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK), "rb") as stream:
            opened = os.fstat(stream.fileno()); digest = hashlib.sha256(); count = 0
            while True:
                chunk = stream.read(1048576)
                if not chunk: break
                count += len(chunk)
                require(count <= maximum_member_bytes, "native state grew beyond observation bound")
                digest.update(chunk)
            after = os.fstat(stream.fileno())
        identity = lambda s: (s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns, s.st_ctime_ns)
        require(count == before.st_size and identity(before) == identity(opened) == identity(after) == identity(path.lstat()),
                "native state changed during observation")
        result[path.relative_to(root).as_posix()] = digest.hexdigest()
    return result


def public_candidate_build(value, installed=None):
    # Reuse public identity type checks; this local candidate makes no claim
    # that its source tree equals a remote branch.
    if installed is not None:
        from installed_candidate_subjects import InstalledCandidate
        require(type(installed) is InstalledCandidate, "admitted installed candidate required")
        installed.reobserve()
        require(value is None or value == installed.flow_identity, "installed package identity substitution")
        return copy.deepcopy(installed.flow_identity)
    selected = copy.deepcopy(value)
    selected["equivalent_remote_revision"] = selected.get("actual_build_revision")
    result = public_build_identity(selected)
    result.pop("equivalent_remote_revision")
    return result


def png(color):
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xffffffff)
    rows = b"".join(b"\0" + bytes(color) * 100 for _ in range(100))
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 100, 100, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(rows, 9)) + chunk(b"IEND", b""))


def cohort(provider, descriptor, flow_version="0.1.0"):
    return {"schema_version": "flow.release-cohort/v1", "id": "local-renderflow-candidate",
        "flow_version": flow_version, "targets": [TARGET], "retained_rows": [ROW], "rows": [{
        "id": ROW, "tool": "renderflow", "interface": "flow_route", "target": TARGET,
        "profile": "canonical-build-v1", "input": "explicit ordered PNG collection",
        "output": "selected fixed-layout EPUB or print-interior PDF",
        "effects": ["read_originals", "write_new_outputs", "direct_provider", "trusted_unconfined"],
        "implementation": "implemented", "artifact": {"version": "0.3.0-rc.1",
        "source_revision": PROVIDER_SOURCE, "name": "renderflow-x86_64-unknown-linux-gnu",
        "sha256": provider["sha256"], "byte_count": provider["byte_count"],
        # A real source URL, deliberately not a fabricated release-download URL.
        "release_url": "https://github.com/egohygiene/renderflow/commit/" + PROVIDER_SOURCE},
        "contracts": [{"identifier": "renderflow.release-contract/v2", "sha256": descriptor["sha256"]}],
        "requirements": [], "owner_issue": "https://github.com/egohygiene/flow/issues/52",
        "next_action": "operator-pinned local source candidate; publication and native qualification unestablished"}]}


def publication(sources, root=None):
    return {"publication": "Synthetic Flow candidate", "issue_id": "local-pilot",
        "title": "Two synthetic pages", "contributors": [{"name": "Flow synthetic fixture", "role": "author"}],
        "publication_date": "2026-11-14", "language": "en-US",
        "geometry": {"width": 90, "height": 90, "unit": "mm"},
        "artwork": [{"role": "page", "path": str(p.relative_to(root) if root else p), "alt_text": "Synthetic colored geometric page " + str(i)}
                    for i, p in enumerate(sources, 1)],
        "rights": {"license": "CC0-1.0", "rights_holder": "Flow synthetic fixture"},
        "accessibility": {"summary": "Two synthetic colored pages with descriptions.",
                          "access_modes": ["visual"], "hazards": ["none"]}}


def configuration(sources, run, fmt, img2pdf):
    declarations = [{"id": "source.page%03d" % i, "path": str(p.relative_to(run.parent)), "format": "png", "media_type": "image/png",
        "sha256": pin(p)["sha256"], "geometry": {"width": 90, "height": 90, "unit": "mm", **({"bleed": 5} if fmt == "pdf" else {})},
        "immutable": True} for i, p in enumerate(sources, 1)]
    declarations.append({"id": "source.pages", "kind": "collection", "members": [s["id"] for s in declarations]})
    role = "ebook" if fmt == "epub" else "interior"
    execution = {"network": "deny", "ai": "deny", "requirements": {"offline": True, "local_only": True}}
    if fmt == "epub":
        execution["fixed_layout_epub"] = {"page_progression_direction": "ltr", "spread": "none", "cover_member_id": "source.page001",
            "max_pages": 2, "max_input_bytes": 1000000, "max_output_bytes": 5000000}
    else:
        execution["print_pdf_interior"] = {"executable": str(img2pdf), "provider_version": "0.6.3", "box_policy": "media_bleed_trim_inset",
            "rotation": "none", "scaling": "fit", "color_policy": "preserve_rgb_gray", "max_pages": 2,
            "max_input_bytes": 1000000, "max_output_bytes": 5000000, "timeout_seconds": 15}
    return {"schema": "renderflow/v2", "sources": declarations, **({"publication": publication(sources, run.parent)} if fmt == "epub" else {}),
        "targets": {"exact": [{"id": "target." + role, "role": role, "format": fmt, "requirement": "required"}]},
        "execution": execution, "output": {"bundle_root": str(run / "native/output"), "collision": "error",
        "naming_template": "{source.id}/{target.role}.{ext}"}}


def request(cohort_path, provider, descriptor, sources, config, run, tools):
    return {"schema_version": "flow.native-request/v1", "cohort": native_path(cohort_path),
        "cohort_sha256": pin(cohort_path)["sha256"], "row_id": ROW, "provider_executable": native_path(provider),
        "release_descriptor": native_path(descriptor), "inputs": [native_path(p) for p in sources],
        "configuration": native_path(config), "run_directory": native_path(run),
        "tool_directories": [native_path(p) for p in tools], "limits": LIMITS}


def accepted(value, run, fmt, sources):
    require(value["schema_version"] in ["flow.native-status/v1", "flow.native-receipt/v1"] and value["state"] == "complete", "native completion required")
    receipt = value["receipt"] if value["schema_version"] == "flow.native-status/v1" else value
    require(receipt["schema_version"] == "flow.native-receipt/v1" and receipt["native_run"] == native_path(run / "native/output")
            and len(receipt["artifacts"]) == 1 and receipt["release_qualified"] is False
            and receipt["creative_approval_inferred"] is False and receipt["publication_authority"] is False,
            "exact unqualified native run and one requested artifact required")
    artifact = receipt["artifacts"][0]
    path = Path(artifact["path"]["value"])
    require(path.is_relative_to(run / "native/output") and path.resolve(strict=True) == path
            and path.suffix == "." + fmt, "confined requested publication required")
    require(pin(path) == artifact, "accepted publication current bytes differ")
    acceptance = Path(receipt["acceptance_file"]["value"])
    require(acceptance.is_relative_to(run) and acceptance.resolve(strict=True) == acceptance,
            "native acceptance path escaped run")
    raw = OWNER_IO.regular(acceptance, 16777216)
    require(sha(raw) == receipt["acceptance_sha256"], "retained acceptance bytes differ")
    evidence = document(raw)
    require(evidence["schema_version"] == "flow.renderflow-domain-acceptance/v1"
            and evidence["source_pins"] == [pin(p) for p in sources]
            and evidence["native_publications"]["verified_publication_count"] == 1,
            "native source/domain/publication evidence required")
    manifest_bytes = OWNER_IO.regular(run / "native/output/renderflow-run.json", 16777216)
    report = evidence["native_publications"]
    selected = [branch for branch in evidence["native_plan"]["artifact_forest"]["branches"] if branch["state"] == "selected"]
    require(len(selected) == 1 and selected[0]["format"] == fmt
        and selected[0]["role"] == ("ebook" if fmt == "epub" else "interior")
        and selected[0]["requirement"] == "required", "requested native target differs")
    expected_target = selected[0]["target_id"]
    verified = report["verified_publications"]
    require(evidence["native_manifest_sha256"] == sha(manifest_bytes)
        and evidence["native_manifest"] == document(manifest_bytes)
        and evidence["native_manifest"]["state"] == "complete"
        and evidence["native_plan"]["artifact_forest"]["identity"] == "sha256:" + receipt["native_intent_sha256"]
        and report["schema_version"] == "renderflow.publication-verification/v1"
        and report["run_state"] == "complete" and report["has_publications"] is True
        and report["receipt_sha256"] == evidence["native_manifest_sha256"]
        and len(verified) == 1 and verified[0]["target_id"] == expected_target
        and verified[0]["sha256"] == artifact["sha256"] and verified[0]["size_bytes"] == artifact["byte_count"]
        and verified[0]["locator"] == "bundle:" + path.relative_to(run / "native/output").as_posix()
        and verified[0]["validation"] in ["valid", "valid_with_warnings"],
        "native receipt/forest/requested publication join differs")
    if fmt == "epub":
        epub_source_images(OWNER_IO.regular(path, 5000000), sources)
    return receipt, {"format": fmt, "sha256": artifact["sha256"], "bytes": artifact["byte_count"],
        "native_manifest_sha256": evidence["native_manifest_sha256"],
        "forest": evidence["native_plan"]["artifact_forest"]["identity"]}


def refusal(value, diagnostic):
    require(value["result"] is None and len(value["diagnostics"]) == 1
        and value["diagnostics"][0] == {"code": diagnostic[0], "message": diagnostic[1],
            "recovery_directory": diagnostic[2]}, "named refusal domain/code/reason/recovery differs")


def pdf_geometry(observed):
    expected = [0.0, 0.0, 283.46456692913387, 283.46456692913387]
    require(type(observed) is dict and observed.get("pages") == 2
        and type(observed.get("pages")) is int and type(observed.get("boxes")) is list
        and len(observed["boxes"]) == 2
        and all(type(box) is list and len(box) == 4
            and all(type(coordinate) in [float, int] and math.isfinite(coordinate)
                and abs(coordinate - wanted) < 0.001 for coordinate, wanted in zip(box, expected))
            for box in observed["boxes"]), "independent PDF page count/media geometry differs")


def epub_source_images(data, sources):
    def member(base, reference):
        require(type(reference) is str and reference and not any(c in reference for c in [":", "\\", "?", "#", "\0"])
            and not reference.startswith("/"), "local EPUB member reference required")
        joined = posixpath.normpath(posixpath.join(base, reference))
        require(joined not in [".", ".."] and not joined.startswith("../"), "EPUB member escaped archive")
        return joined
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        names = archive.namelist()
        require(len(names) == len(set(names)) and len(names) <= 128
            and sum(item.file_size for item in archive.infolist()) <= 10000000,
            "bounded unique EPUB members required")
        def xml(name):
            info = archive.getinfo(name); require(info.file_size <= 1048576, "EPUB XML bound")
            raw = archive.read(name)
            require(b"<!DOCTYPE" not in raw and b"<!ENTITY" not in raw, "EPUB XML declarations refused")
            return ET.fromstring(raw)
        require(archive.read("mimetype") == b"application/epub+zip", "EPUB MIME differs")
        roots = xml("META-INF/container.xml").findall(".//{urn:oasis:names:tc:opendocument:xmlns:container}rootfile")
        require(len(roots) == 1, "one EPUB package required")
        package = member("", roots[0].get("full-path")); root = xml(package)
        opf = "{http://www.idpf.org/2007/opf}"
        items = root.findall(opf + "manifest/" + opf + "item")
        manifest = {item.get("id"): item for item in items}
        require(len(manifest) == len(items), "duplicate EPUB manifest identity")
        spine = root.findall(opf + "spine/" + opf + "itemref")
        require(len(spine) == len(sources) == 2, "EPUB spine count differs")
        hashes = []
        for reference in spine:
            page = manifest.get(reference.get("idref")); require(page is not None, "EPUB spine page absent")
            page_path = member(posixpath.dirname(package), page.get("href"))
            images = xml(page_path).findall(".//{http://www.w3.org/1999/xhtml}img")
            require(len(images) == 1, "one explicit image per EPUB page required")
            image_path = member(posixpath.dirname(page_path), images[0].get("src"))
            require(archive.getinfo(image_path).file_size <= 1000000, "EPUB source image bound")
            hashes.append(sha(archive.read(image_path)))
        require(hashes == [pin(source)["sha256"] for source in sources], "EPUB spine image bytes/order differ from originals")
        require(len([name for name in names if name.endswith(".png")]) == 2, "unexpected EPUB page image membership")
        return hashes


def run_pilot(flow, provider, descriptor, img2pdf, output, provenance, installed=None):
    provenance = public_candidate_build(provenance, installed)
    data = build_binary(flow)
    require({"sha256": sha(data), "bytes": len(data)} == provenance["binary"], "Flow supplied build differs")
    provider_pin, descriptor_pin = pin(provider), pin(descriptor)
    selected_provider = ({"version": "0.3.0-rc.1", "source_revision": PROVIDER_SOURCE,
        "sha256": PROVIDER_SHA, "contract_sha256": CONTRACT_SHA, "target": TARGET,
        "publication": "not_performed"} if installed is None else installed.render_identity)
    require(provider_pin["sha256"] == selected_provider["sha256"]
            and descriptor_pin["sha256"] == selected_provider["contract_sha256"],
            "selected captured candidate differs")
    if installed is not None:
        require(flow == installed.flow and provider == installed.provider and descriptor == installed.descriptor,
                "proof must execute the selected installed paths")
    require(output.is_absolute() and output.parent.resolve(strict=True) == output.parent, "canonical fresh evidence parent required")
    output.mkdir(mode=0o700)
    if installed is None:
        copied = output / "flow"; copied.write_bytes(data); copied.chmod(0o700); flow = copied
    logs = output / "commands"; logs.mkdir(mode=0o700)
    source = output / "source 作品 with spaces"; source.mkdir(mode=0o700)
    sources = [source / "page 001.png", source / "page 002.png"]
    for path, color in zip(sources, [(30, 70, 130), (180, 90, 40)]):
        path.write_bytes(png(color))
    originals = [pin(p) for p in sources]
    if installed is None:
        cohort_path = output / "cohort.json"; put(cohort_path, cohort(provider_pin, descriptor_pin))
    else:
        cohort_path = installed.cohort_path
    observations, publications = [], []
    started = time.monotonic()
    def invoke(label, args, expected_code=0, expected_outcome="complete", diagnostic=None):
        remaining = 180 - (time.monotonic() - started)
        require(remaining > 0, "aggregate pilot deadline exceeded")
        code, stdout, stderr = command([str(flow), *args, "--format", "json"], cwd=output,
            timeout=min(75, remaining), max_output=16777216)
        (logs / (label + ".stdout.json")).write_bytes(stdout)
        (logs / (label + ".stderr")).write_bytes(stderr)
        value = envelope(stdout, stderr, code, args[0], expected_code, expected_outcome)
        if diagnostic:
            refusal(value, diagnostic)
        else:
            require(value["diagnostics"] == [] and isinstance(value["result"], dict), "unexpected diagnostic")
        observations.append({"case": label, "command": args[0], "exit_code": code, "outcome": value["outcome"],
            "stdout_sha256": sha(stdout), "stdout_bytes": len(stdout), "stderr_bytes": len(stderr)})
        return value["result"]
    for fmt in ["epub", "pdf"]:
        run = output / (fmt + "-run")
        config_path = output / (fmt + "-config.json"); put(config_path, configuration(sources, run, fmt, img2pdf))
        request_path = output / (fmt + "-request.json")
        tools = [] if fmt == "epub" else [img2pdf.parent]
        put(request_path, request(cohort_path, provider, descriptor, sources, config_path, run, tools))
        plan_path, approval_path = output / (fmt + "-plan.json"), output / (fmt + "-approval.json")
        plan = invoke(fmt + "-plan", ["plan", "--request", str(request_path), "--output", str(plan_path)])
        require(not run.exists() and plan["release_evidence"] == "operator_pinned_source_candidate; publication_and_qualification_not_established",
                "prepare invoked provider or promoted release")
        invoke(fmt + "-ack-required", ["approve", "--plan", str(plan_path), "--approved-by", "synthetic operator",
            "--acknowledge-trusted-unconfined", "--output", str(approval_path)], 5, "refused",
            ("approval_required", "exact plan approval must acknowledge trusted-unconfined, unqualified native candidate execution", None))
        require(not approval_path.exists() and not run.exists(), "missing acknowledgement wrote state")
        invoke(fmt + "-approve", ["approve", "--plan", str(plan_path), "--approved-by", "synthetic local publication operator",
            "--acknowledge-trusted-unconfined", "--acknowledge-unqualified-candidate", "--output", str(approval_path)])
        value = invoke(fmt + "-run", ["run", "--plan", str(plan_path), "--approval", str(approval_path)])
        receipt, summary = accepted(value, run, fmt, sources); publications.append(summary)
        before = inventory(run)
        require(accepted(invoke(fmt + "-status", ["status", "--run-directory", str(run)]), run, fmt, sources)[0] == receipt,
                "status changed native receipt")
        require(accepted(invoke(fmt + "-resume", ["resume", "--run-directory", str(run), "--native-run-directory", str(run / "native/output")]),
                run, fmt, sources)[0] == receipt, "completed resume changed native receipt")
        require(inventory(run) == before, "completed status/resume replayed work")
        invoke(fmt + "-existing-run", ["run", "--plan", str(plan_path), "--approval", str(approval_path)], 7, "recovery_required",
            ("recovery_required", "fresh native output directory required; preserve existing state", None))
        require(inventory(run) == before, "existing native run was overwritten")
        invoke(fmt + "-wrong-resume", ["resume", "--run-directory", str(run), "--native-run-directory", str(run / "native/not-selected")],
            7, "recovery_required", ("recovery_required", "resume native run differs from accepted operation", None))
        raw = OWNER_IO.regular(config_path, 1048576)
        config_path.write_bytes(raw + b" ")
        invoke(fmt + "-changed-config", ["status", "--run-directory", str(run)], 3, "refused",
            ("stale_plan", "native pinned bytes changed; create a new reviewed plan", native_path(run)))
        config_path.write_bytes(raw)
        artifact = Path(receipt["artifacts"][0]["path"]["value"])
        raw = OWNER_IO.regular(artifact, 5000000); artifact.write_bytes(raw + b" ")
        invoke(fmt + "-changed-publication", ["status", "--run-directory", str(run)], 3, "refused",
            ("stale_plan", "native pinned bytes changed; create a new reviewed plan", native_path(run)))
        artifact.write_bytes(raw)
        require(inventory(run) == before, "refusal controls changed restored native state")
        wrong = document(OWNER_IO.regular(request_path, 1048576)); wrong["inputs"].reverse()
        wrong_request = output / (fmt + "-wrong-order.json"); put(wrong_request, wrong)
        invoke(fmt + "-wrong-order", ["plan", "--request", str(wrong_request), "--output", str(output / (fmt + "-wrong-order-plan.json"))],
            3, "refused", ("invalid_input", "native ordered source path differs from selected input", None))
        require(accepted(invoke(fmt + "-final-status", ["status", "--run-directory", str(run)]), run, fmt, sources)[0] == receipt,
                "restored native evidence no longer validates")
        if fmt == "pdf":
            inspection, stdout, stderr = command([str(img2pdf.parent / "python"), "-c",
                "import json,pikepdf,sys; p=pikepdf.open(sys.argv[1]); print(json.dumps({'pages':len(p.pages),'boxes':[[float(v) for v in page.MediaBox] for page in p.pages]}))",
                str(artifact)], cwd=output, timeout=15)
            require(inspection == 0 and stderr == b"", "independent PDF inspector failed")
            observed = document(stdout)
            pdf_geometry(observed)
            summary["independent_inspection"] = "pikepdf two-page media geometry"
            summary["inspection_stdout_sha256"] = sha(stdout)
    require([pin(p) for p in sources] == originals and pin(provider) == provider_pin and pin(descriptor) == descriptor_pin,
            "source/candidate bytes changed")
    require(build_binary(flow) == data, "selected Flow bytes changed")
    if installed is not None:
        installed.reobserve()
    return {"schema_version": "flow.native-publication-cli-pilot/v1", "status": "passed",
        "observed_utc": datetime.now(timezone.utc).isoformat(), "driver_sha256": pin(Path(__file__).resolve())["sha256"],
        "flow_build": provenance, "provider": copy.deepcopy(selected_provider),
        "commands": observations, "publications": publications, "fixture": {"files": 2,
        "logical_bytes": sum(p["byte_count"] for p in originals), "originals_unchanged": True},
        "elapsed_seconds": round(time.monotonic() - started, 6), "source_mutation_authority": False,
        "human_creative_approval": False, "release_authenticated": False, "native_matrix_qualified": False,
        "hosted_ci_dispatched": False,
        "execution_subject": "captured_dev_cli" if installed is None else "installed_packaged_cli",
        "scope": "synthetic public CLI candidate proof; no release or full installed-suite qualification"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("flow", "provider", "descriptor", "img2pdf", "flow-build-evidence", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    report = run_pilot(args.flow, args.provider, args.descriptor, args.img2pdf, args.output,
        document(OWNER_IO.regular(args.flow_build_evidence, 65536)))
    put(args.output / "report.json", report)
    print(json.dumps({"status": report["status"], "publications": report["publications"], "seconds": report["elapsed_seconds"]}))


if __name__ == "__main__":
    main()
