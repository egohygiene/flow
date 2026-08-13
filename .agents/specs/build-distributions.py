#!/usr/bin/env python3
# Copyright 2026 Ego Hygiene
# SPDX-License-Identifier: MIT

"""Build deterministic portable specification distributions from canonical source.

Usage:
    python3 library/organization/specs/build-distributions.py [--check] [--output-directory dist]
"""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import sys

REPO_ROOT = Path(__file__).resolve().parents[3]
SPECS_DIR = REPO_ROOT / "library" / "organization" / "specs"
FRONTMATTER_RE = re.compile(r"^---\n(.*?)\n---\n", re.DOTALL)


def _normalized_bytes(path: Path) -> bytes:
    text = path.read_text(encoding="utf-8")
    return text.replace("\r\n", "\n").replace("\r", "\n").encode("utf-8")


def _display_path(path: Path) -> str:
    try:
        return path.relative_to(REPO_ROOT).as_posix()
    except ValueError:
        return path.as_posix()


def find_specs() -> list[Path]:
    return sorted(SPECS_DIR.rglob("*.spec.md"))


def _distribution_filename(source_path: Path) -> str:
    text = source_path.read_text(encoding="utf-8").replace("\r\n", "\n").replace("\r", "\n")
    match = FRONTMATTER_RE.match(text)
    if not match:
        raise ValueError(f"missing YAML frontmatter in {source_path}")
    spec_id = None
    for line in match.group(1).splitlines():
        if line.startswith("id:"):
            spec_id = line.split(":", 1)[1].strip()
            break
    if not spec_id:
        raise ValueError(f"missing frontmatter id in {source_path}")
    return f"{spec_id}.spec.md"


def build(check: bool = False, output_directory: Path | None = None) -> int:
    dist_dir = (output_directory or (REPO_ROOT / "dist")) / "specs"
    specs = find_specs()
    if not specs:
        print(
            "WARNING: no *.spec.md files found under library/organization/specs/", file=sys.stderr
        )
        return 0

    drift = 0
    written = 0
    for source_path in specs:
        try:
            out_path = dist_dir / _distribution_filename(source_path)
            content = _normalized_bytes(source_path)
            display_path = _display_path(out_path)
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
        except ValueError as exc:
            print(f"ERROR  {exc}", file=sys.stderr)
            drift += 1

    if check:
        if drift:
            print(
                f"\n{drift} file(s) out of date. Run build-distributions.py to regenerate.",
                file=sys.stderr,
            )
            return 1
        print(f"\nAll {len(specs)} specification distribution(s) are up to date.")
        return 0

    print(f"\n{written} file(s) written for {len(specs)} specification(s).")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Detect drift without writing files")
    parser.add_argument(
        "--output-directory",
        default="dist",
        help="Base output directory for generated distributions.",
    )
    args = parser.parse_args()
    return build(check=args.check, output_directory=(REPO_ROOT / args.output_directory))


if __name__ == "__main__":
    sys.exit(main())
