#!/usr/bin/env python3
# Copyright 2026 Ego Hygiene
# SPDX-License-Identifier: MIT

"""Build deterministic release metadata for first-party skill publications.

Usage:
    python3 library/organization/skills/build-release-artifacts.py \
        --release-tag v1.2.3 [--output-directory dist] [--commit-sha <sha>]
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys

from jsonschema import Draft202012Validator

REPO_ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = REPO_ROOT / "catalog" / "first-party" / "catalog.v1.json"
RELEASE_SCHEMA_PATH = REPO_ROOT / "catalog" / "schemas" / "aether.release-manifest.v1.schema.json"
LICENSE_PATH = REPO_ROOT / "LICENSE"
CHANGELOG_PATH = REPO_ROOT / "CHANGELOG.md"


def _canonical_json(data: object) -> str:
    return json.dumps(data, indent=2, sort_keys=True) + "\n"


def _sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def _load_release_records(catalog_path: Path) -> list[dict]:
    catalog = _read_json(catalog_path)
    records = []
    for record in catalog.get("artifacts", []):
        if record.get("kind") != "skill":
            continue
        if ((record.get("lifecycle") or {}).get("state")) != "stable":
            continue
        if not (record.get("release") or {}).get("included"):
            continue
        records.append(record)
    return sorted(records, key=lambda item: item["id"])


def _load_distribution_manifest(output_directory: Path, skill_name: str) -> dict:
    manifest_path = output_directory / "skills" / skill_name / "distribution-manifest.v1.json"
    if not manifest_path.exists():
        raise ValueError(f"missing distribution manifest: {manifest_path}")
    return _read_json(manifest_path)


def _build_release_manifest(release_tag: str, records: list[dict], output_directory: Path) -> dict:
    artifacts = []
    for record in records:
        skill_name = record["id"].split("/", 1)[1]
        dist_manifest = _load_distribution_manifest(output_directory, skill_name)
        if dist_manifest.get("artifact_id") != record["id"]:
            raise ValueError(
                f"distribution artifact mismatch for {record['id']}: "
                f"{dist_manifest.get('artifact_id')!r}"
            )
        if dist_manifest.get("artifact_version") != record["artifact_version"]:
            raise ValueError(
                f"distribution version mismatch for {record['id']}: "
                f"{dist_manifest.get('artifact_version')!r}"
            )
        if (dist_manifest.get("source_digest") or {}).get("value") != (
            (record.get("source_digest") or {}).get("value")
        ):
            raise ValueError(f"distribution source digest mismatch for {record['id']}")
        artifacts.append(
            {
                "artifact_id": record["id"],
                "artifact_version": record["artifact_version"],
                "source_digest": dist_manifest["source_digest"],
            }
        )

    if not artifacts:
        raise ValueError(
            "no release-eligible first-party skills found; "
            "only catalog records with lifecycle.state=stable and release.included=true may be released"
        )

    return {
        "schema_version": "aether.release-manifest/v1",
        "repository_release_tag": release_tag,
        "artifacts": artifacts,
    }


def _validate_release_manifest(release_manifest: dict, schema_path: Path) -> None:
    schema = _read_json(schema_path)
    validator = Draft202012Validator(schema)
    errors = sorted(validator.iter_errors(release_manifest), key=lambda err: list(err.path))
    if errors:
        raise ValueError(f"release manifest failed schema validation: {errors[0].message}")


def _build_checksums(output_directory: Path, release_dir: Path) -> str:
    lines = []
    for path in sorted(output_directory.rglob("*")):
        if not path.is_file():
            continue
        if path == release_dir / "checksums.txt":
            continue
        lines.append(f"{_sha256_file(path)}  {path.relative_to(output_directory).as_posix()}")
    return "\n".join(lines) + "\n"


def _build_license_notices(records: list[dict], license_path: Path) -> str:
    header = [
        "Aether release license notices",
        "",
        "Repository license file: LICENSE",
        "",
        "Included first-party skills:",
        "",
    ]
    for record in records:
        header.append(
            f"- {record['id']} | version {record['artifact_version']} | license {record.get('license', 'UNKNOWN')}"
        )
    header.extend(
        [
            "",
            "--- LICENSE BEGIN ---",
            license_path.read_text(encoding="utf-8").rstrip(),
            "--- LICENSE END ---",
            "",
        ]
    )
    return "\n".join(header)


def _extract_unreleased_changelog(changelog_path: Path) -> str:
    lines = changelog_path.read_text(encoding="utf-8").splitlines()
    capture = []
    active = False
    for line in lines:
        if line == "## Unreleased":
            active = True
            capture.append(line)
            continue
        if active and line.startswith("## "):
            break
        if active:
            capture.append(line)
    return "\n".join(capture).strip()


def _build_release_notes(release_tag: str, release_manifest: dict, changelog_path: Path) -> str:
    artifact_lines = [
        f"- `{artifact['artifact_id']}` @ `{artifact['artifact_version']}`"
        for artifact in release_manifest["artifacts"]
    ]
    changelog_excerpt = _extract_unreleased_changelog(changelog_path)
    lines = [
        f"# Release notes for {release_tag}",
        "",
        "This release is published explicitly from a tagged commit after deterministic validation.",
        "",
        "## Included first-party skills",
        *artifact_lines,
    ]
    if changelog_excerpt:
        lines.extend(["", "## Changelog excerpt", changelog_excerpt])
    lines.append("")
    return "\n".join(lines)


def _build_provenance(
    release_tag: str,
    commit_sha: str,
    release_manifest: dict,
    output_directory: Path,
) -> dict:
    output_files = []
    for path in sorted(output_directory.rglob("*")):
        if path.is_file():
            output_files.append(
                {
                    "path": path.relative_to(output_directory).as_posix(),
                    "sha256": _sha256_file(path),
                }
            )
    return {
        "schema_version": "aether.release-provenance/v1",
        "repository_release_tag": release_tag,
        "commit_sha": commit_sha,
        "artifacts": release_manifest["artifacts"],
        "output_files": output_files,
    }


def build_release_artifacts(
    release_tag: str,
    output_directory: Path,
    commit_sha: str,
) -> Path:
    records = _load_release_records(CATALOG_PATH)
    release_dir = output_directory / "release"
    release_dir.mkdir(parents=True, exist_ok=True)

    release_manifest = _build_release_manifest(release_tag, records, output_directory)
    _validate_release_manifest(release_manifest, RELEASE_SCHEMA_PATH)

    (release_dir / "release-manifest.v1.json").write_text(
        _canonical_json(release_manifest),
        encoding="utf-8",
    )
    (release_dir / "LICENSE.notices.txt").write_text(
        _build_license_notices(records, LICENSE_PATH),
        encoding="utf-8",
    )
    (release_dir / "release-notes.md").write_text(
        _build_release_notes(release_tag, release_manifest, CHANGELOG_PATH),
        encoding="utf-8",
    )
    (release_dir / "checksums.txt").write_text(
        _build_checksums(output_directory, release_dir),
        encoding="utf-8",
    )
    (release_dir / "release-provenance.v1.json").write_text(
        _canonical_json(
            _build_provenance(release_tag, commit_sha, release_manifest, output_directory)
        ),
        encoding="utf-8",
    )
    return release_dir


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release-tag", required=True, help="Repository release tag.")
    parser.add_argument(
        "--output-directory",
        default="dist",
        help="Base output directory containing generated release payloads.",
    )
    parser.add_argument(
        "--commit-sha",
        default="unknown",
        help="Commit SHA used to build the release payload.",
    )
    args = parser.parse_args(argv)

    try:
        release_dir = build_release_artifacts(
            release_tag=args.release_tag,
            output_directory=REPO_ROOT / args.output_directory,
            commit_sha=args.commit_sha,
        )
    except ValueError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    print(f"Wrote release metadata to {release_dir.relative_to(REPO_ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
