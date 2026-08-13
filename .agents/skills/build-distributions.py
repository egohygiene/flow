#!/usr/bin/env python3
# Copyright 2026 Ego Hygiene
# SPDX-License-Identifier: MIT

"""Build deterministic portable skill distributions from canonical source.

Usage:
    python3 library/organization/skills/build-distributions.py [--check] [--output-directory dist]

Options:
    --check    Verify that existing distributions are up to date; exit non-zero
               if any drift is detected without writing files.

Distribution layout
-------------------
For each canonical skill at:

    library/organization/skills/<domain>/<skill-name>/

This script writes a self-contained distribution package at:

    dist/skills/<skill-name>/
    ├── SKILL.md                           # copied verbatim from canonical source
    ├── evals/                             # copied verbatim (when present)
    ├── references/                        # copied verbatim (when present)
    ├── templates/                         # copied verbatim (when present)
    └── distribution-manifest.v1.json     # generated provenance manifest

Rules
-----
1. Canonical source is never modified.
2. Generated files are never hand-edited; the manifest header makes this explicit.
3. The source digest covers only the canonical SKILL.md file (UTF-8, LF-normalised).
4. Companion directories (evals/, references/, templates/) are copied verbatim.
5. If ``--check`` is passed the script prints OK/DRIFT lines and exits non-zero
   on any drift without writing anything.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys

try:
    import yaml
except ImportError:
    print("ERROR: PyYAML is required. Install with: pip install pyyaml", file=sys.stderr)
    sys.exit(2)

REPO_ROOT = Path(__file__).resolve().parents[3]
SKILLS_DIR = REPO_ROOT / "library" / "organization" / "skills"
DIST_DIR = REPO_ROOT / "dist" / "skills"

FRONTMATTER_RE = re.compile(r"^---\n(.*?)\n---\n", re.DOTALL)
GENERATOR_ID = "library/organization/skills/build-distributions.py"
COMPANION_DIRS = ("evals", "references", "templates")


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _normalized_bytes(path: Path) -> bytes:
    text = path.read_text(encoding="utf-8")
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    return text.encode("utf-8")


def _source_digest(path: Path) -> str:
    return hashlib.sha256(_normalized_bytes(path)).hexdigest()


def _canonical_json(data: object) -> str:
    return json.dumps(data, sort_keys=True, indent=2)


def _read_frontmatter(path: Path) -> dict:
    text = path.read_text(encoding="utf-8")
    m = FRONTMATTER_RE.match(text)
    if not m:
        raise ValueError(f"no YAML frontmatter in {path}")
    return yaml.safe_load(m.group(1)) or {}


# ---------------------------------------------------------------------------
# Discovery
# ---------------------------------------------------------------------------


def find_skills() -> list[tuple[str, Path]]:
    """Return sorted list of (skill-name, source-dir) for every canonical skill."""
    results = []
    for domain_dir in sorted(SKILLS_DIR.iterdir()):
        if not domain_dir.is_dir():
            continue
        for skill_dir in sorted(domain_dir.iterdir()):
            if not skill_dir.is_dir():
                continue
            skill_md = skill_dir / "SKILL.md"
            if skill_md.exists():
                results.append((skill_dir.name, skill_dir))
    return results


# ---------------------------------------------------------------------------
# Distribution manifest
# ---------------------------------------------------------------------------


def _build_manifest(
    skill_name: str,
    fm: dict,
    source_path: Path,
    generated_paths: list[str],
) -> dict:
    artifact_id = f"skill/{skill_name}"
    artifact_version = (fm.get("metadata") or {}).get("aether-version", "0.0.0")
    compatibility = fm.get("compatibility") or {"required_tools": []}
    if "required_tools" not in compatibility:
        compatibility = {"required_tools": [], **compatibility}

    return {
        "schema_version": "aether.distribution-manifest/v1",
        "distribution_id": f"distribution/{skill_name}",
        "artifact_id": artifact_id,
        "artifact_version": artifact_version,
        "source_digest": {
            "algorithm": "sha256-utf8-lf",
            "value": _source_digest(source_path),
        },
        "generated_paths": generated_paths,
        "generator": GENERATOR_ID,
        "compatibility": compatibility,
    }


# ---------------------------------------------------------------------------
# Per-skill distribution
# ---------------------------------------------------------------------------


def _collect_generated_paths(skill_name: str, source_dir: Path) -> list[str]:
    """Enumerate all paths that will be written to dist/skills/<name>/."""
    paths = [f"dist/skills/{skill_name}/SKILL.md"]
    for companion in COMPANION_DIRS:
        src = source_dir / companion
        if src.is_dir():
            for f in sorted(src.rglob("*")):
                if f.is_file():
                    rel = f.relative_to(source_dir)
                    paths.append(f"dist/skills/{skill_name}/{rel.as_posix()}")
    paths.append(f"dist/skills/{skill_name}/distribution-manifest.v1.json")
    return paths


def _build_skill_dist(skill_name: str, source_dir: Path) -> dict[str, bytes]:
    """Return mapping of relative-to-repo-root path → file bytes for one skill."""
    source_md = source_dir / "SKILL.md"
    fm = _read_frontmatter(source_md)
    generated_paths = _collect_generated_paths(skill_name, source_dir)

    out: dict[str, bytes] = {}

    # SKILL.md — copied verbatim
    out[f"dist/skills/{skill_name}/SKILL.md"] = _normalized_bytes(source_md)

    # companion directories — copied verbatim
    for companion in COMPANION_DIRS:
        src = source_dir / companion
        if src.is_dir():
            for f in sorted(src.rglob("*")):
                if f.is_file():
                    rel = f.relative_to(source_dir)
                    out[f"dist/skills/{skill_name}/{rel.as_posix()}"] = f.read_bytes()

    # distribution manifest
    manifest = _build_manifest(skill_name, fm, source_md, generated_paths)
    manifest_bytes = (_canonical_json(manifest) + "\n").encode("utf-8")
    out[f"dist/skills/{skill_name}/distribution-manifest.v1.json"] = manifest_bytes

    return out


# ---------------------------------------------------------------------------
# Build / check
# ---------------------------------------------------------------------------


def build(check: bool = False, output_directory: Path | None = None) -> int:
    dist_dir = (output_directory or (REPO_ROOT / "dist")) / "skills"
    skills = find_skills()
    if not skills:
        print(
            "WARNING: no SKILL.md files found under library/organization/skills/", file=sys.stderr
        )
        return 0

    drift = 0
    written = 0

    for skill_name, source_dir in skills:
        try:
            file_map = _build_skill_dist(skill_name, source_dir)
        except Exception as exc:  # noqa: BLE001
            print(f"ERROR  {skill_name}: {exc}", file=sys.stderr)
            drift += 1
            continue

        for rel_path, content in file_map.items():
            out_path = dist_dir / Path(rel_path).relative_to("dist/skills")
            display_path = _normalized_display_path(out_path)
            if check:
                if not out_path.exists():
                    print(f"DRIFT  missing: {display_path}")
                    drift += 1
                elif out_path.read_bytes() != content:
                    print(f"DRIFT  stale:   {display_path}")
                    drift += 1
                else:
                    print(f"OK     {display_path}")
            else:
                out_path.parent.mkdir(parents=True, exist_ok=True)
                out_path.write_bytes(content)
                print(f"wrote  {display_path}")
                written += 1

    if check:
        if drift:
            print(
                f"\n{drift} file(s) out of date. Run build-distributions.py to regenerate.",
                file=sys.stderr,
            )
            return 1
        print(f"\nAll {len(skills)} skill distribution(s) are up to date.")
        return 0

    print(f"\n{written} file(s) written for {len(skills)} skill(s).")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="Detect drift without writing files",
    )
    parser.add_argument(
        "--output-directory",
        default="dist",
        help="Base output directory for generated distributions.",
    )
    args = parser.parse_args()
    return build(check=args.check, output_directory=(REPO_ROOT / args.output_directory))


def _normalized_display_path(path: Path) -> str:
    try:
        return path.relative_to(REPO_ROOT).as_posix()
    except ValueError:
        return path.as_posix()


if __name__ == "__main__":
    sys.exit(main())
