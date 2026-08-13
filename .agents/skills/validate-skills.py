#!/usr/bin/env python3
# Copyright 2026 Ego Hygiene
# SPDX-License-Identifier: MIT

"""Validate canonical SKILL.md frontmatter against the Agent Skills specification.

Usage:
    python3 library/organization/skills/validate-skills.py

Exit code 0 when all skills are valid; non-zero otherwise.
"""

from pathlib import Path
import re
import sys

try:
    import yaml
except ImportError:
    print("ERROR: PyYAML is required. Install with: pip install pyyaml", file=sys.stderr)
    sys.exit(2)

BASE = Path(__file__).resolve().parent

ALLOWED_TOP = {"name", "description", "license", "compatibility", "metadata", "allowed-tools"}
REQUIRED = {"name", "description"}
NAME_RE = re.compile(r"^[a-z]([a-z0-9-]*[a-z0-9])?$")
FRONTMATTER_RE = re.compile(r"^---\n(.*?)\n---\n", re.DOTALL)


def find_skills(base):
    return sorted(
        (skill_path.parent.relative_to(base), skill_path) for skill_path in base.rglob("SKILL.md")
    )


def validate_skill(rel, path):
    dir_name = rel.name
    errors = []

    with path.open(encoding="utf-8") as f:
        content = f.read()

    m = FRONTMATTER_RE.match(content)
    if not m:
        return ["no YAML frontmatter block found"]

    try:
        fm = yaml.safe_load(m.group(1))
    except yaml.YAMLError as exc:
        return [f"YAML parse error: {exc}"]

    if not isinstance(fm, dict):
        return ["frontmatter is not a YAML mapping"]

    for req in REQUIRED:
        if req not in fm:
            errors.append(f"missing required field '{req}'")

    if "name" in fm:
        name = str(fm["name"])
        if name != dir_name:
            errors.append(f"name '{name}' does not match parent directory '{dir_name}'")
        if len(name) > 64:
            errors.append(f"name exceeds 64 characters ({len(name)})")
        if not NAME_RE.match(name):
            errors.append(f"name '{name}' must be lowercase kebab-case")

    if "description" in fm:
        desc = str(fm["description"])
        if not desc.strip():
            errors.append("description is empty")
        if len(desc) > 1024:
            errors.append(f"description exceeds 1024 characters ({len(desc)})")

    for key in fm:
        if key not in ALLOWED_TOP:
            errors.append(f"unsupported top-level key '{key}'")

    if "metadata" in fm and not isinstance(fm["metadata"], dict):
        errors.append("metadata must be a YAML mapping")

    return errors


def main():
    skills = find_skills(BASE)
    print(f"Validating {len(skills)} canonical skills\n")

    all_errors = {}
    for rel, path in skills:
        errs = validate_skill(rel, path)
        all_errors[rel] = errs
        status = "\033[32m✓\033[0m" if not errs else "\033[31m✗\033[0m"
        print(f"  {status}  {rel}")
        for e in errs:
            print(f"       ERROR: {e}")

    total_errors = sum(len(v) for v in all_errors.values())
    print()
    if total_errors == 0:
        print(f"All {len(skills)} skills are valid.")
        return 0
    print(f"{total_errors} error(s) across {sum(1 for v in all_errors.values() if v)} skill(s).")
    return 1


if __name__ == "__main__":
    sys.exit(main())
