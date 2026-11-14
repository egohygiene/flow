#!/usr/bin/env python3
"""Run existing tiny publishing/read-only pilots through selected installed bytes.

No build, install, acquisition, inference, signing, hosted CI or publication.
The exact package/candidate/install subjects must already have been prepared.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
import platform
import time

import check_native_publication as native
import check_released_read_only as readonly
import check_static_publication as static
import check_static_staging_budget as staging
from installed_candidate_subjects import InstalledCandidate


def run_proof(selection_path, img2pdf, output):
    subject = InstalledCandidate.load(selection_path)
    native.require(platform.system() == "Linux" and platform.machine() == "x86_64",
                   "installed candidate proof requires the selected Linux x86-64 host")
    native.require(img2pdf.is_absolute() and img2pdf.resolve(strict=True) == img2pdf
        and img2pdf.is_file(), "canonical prepared img2pdf executable required")
    native.require(output.is_absolute() and output.parent.resolve(strict=True) == output.parent,
                   "canonical fresh combined proof parent required")
    protected = [subject.root, subject.package_directory, selection_path,
        Path(subject.selection["renderflow_candidate_proof"]),
        Path(subject.selection["flow_package_build_observation"]),
        subject.optiflow_bundle, img2pdf.parent, Path(__file__).resolve().parents[1]]
    native.require(all(not output.is_relative_to(path) and not path.is_relative_to(output)
        for path in protected), "combined proof root must not overlap selected inputs or checkout")
    output.mkdir(mode=0o700)
    started = time.monotonic()
    admission = subject.observe_status(output, "installation-before")
    direct_root, static_root = output / "direct-publication", output / "static-publication"
    direct = native.run_pilot(subject.flow, subject.provider, subject.descriptor, img2pdf,
        direct_root, None, installed=subject)
    native.put(direct_root / "report.json", direct)
    project = static.run_pilot(subject.flow, subject.root,
        subject.selection["installation_receipt_sha256"], img2pdf, direct_root,
        static_root, None, installed=subject)
    native.put(static_root / "report.json", project)
    budget_root = output / "staging-budget"
    budget = staging.run_check(subject.flow, None, static_root, budget_root, installed=subject)
    native.put(budget_root / "report.json", budget)
    readonly_root = output / "released-optiflow"
    release = readonly.run_pilot(subject.flow, subject.optiflow_bundle,
        readonly_root, None, installed=subject)
    native.put(readonly_root / "report.json", release)
    final = subject.observe_status(output, "installation-after")
    native.require(admission["installation_receipt_sha256"] == final["installation_receipt_sha256"],
                   "selected installed receipt changed")
    reports = [("direct-publication", direct_root, direct), ("static-publication", static_root, project),
        ("staging-budget", budget_root, budget), ("released-optiflow", readonly_root, release)]
    report = {"schema_version": "flow.combined-installed-candidate-proof/v1", "status": "passed",
        "observed_utc": datetime.now(timezone.utc).isoformat(),
        "scope": "tiny synthetic installed-package CLI workloads on this Linux host only",
        "host": {"system": "Linux", "machine": "x86_64", "libc": list(platform.libc_ver())},
        "flow_build": subject.flow_identity, "renderflow_candidate": subject.render_identity,
        "installed_before": admission, "installed_after": final,
        "selection_sha256": subject.selection_sha256,
        "driver_sha256": native.pin(Path(__file__).resolve())["sha256"],
        "subject_admission_sha256": native.pin(Path(InstalledCandidate.load.__func__.__code__.co_filename).resolve())["sha256"],
        "routes": [{"route": name, "report_sha256": native.pin(root / "report.json")["sha256"],
            "execution_subject": result["execution_subject"], "cli_invocations": len(result["commands"]),
            "status": result["status"]} for name, root, result in reports],
        "cli_invocations": sum(len(result["commands"]) for _, _, result in reports) + 2,
        "elapsed_seconds": round(time.monotonic() - started, 6),
        "external_tools": {"img2pdf": {"sha256": native.pin(img2pdf)["sha256"],
            "byte_count": native.pin(img2pdf)["byte_count"]},
            "scope": "already prepared local provider; transitive tool authentication unestablished"},
        "originals_and_selected_bytes_reobserved": True,
        "product_invocation_uses_installed_paths": True, "source_checkout_required_by_product_argv": False,
        "source_mutation_authority": False, "human_creative_approval": False,
        "publisher_authenticated": False, "release_authenticated": False,
        "native_matrix_qualified": False, "hosted_ci_dispatched": False,
        "publication_performed": False,
        "limits": ["point-in-time byte observations and trusted-unconfined execution, not an OS sandbox",
            "Flow release-profile local preparation and Renderflow Ubuntu userland candidate are not published releases",
            "Optiflow is the exact existing v0.1.1 read-only release, with no new signature verification",
            "static artifacts remain partial with creative/locale/rights/accessibility review pending",
            "no other platforms, private media, models, physical volumes or deployed site is qualified"]}
    native.put(output / "report.json", report)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--selection", type=Path, required=True)
    parser.add_argument("--img2pdf", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = run_proof(args.selection, args.img2pdf, args.output)
    print(readonly.canonical({"status": result["status"], "cli_invocations": result["cli_invocations"],
        "report_sha256": native.pin(args.output / "report.json")["sha256"],
        "publication_performed": False}).decode())


if __name__ == "__main__":
    main()
