#!/usr/bin/env python3
# Copyright 2026 Ego Hygiene
# SPDX-License-Identifier: MIT

"""Validate canonical AGENT.md frontmatter against the Aether agent contract.

Usage:
    python3 library/organization/agents/validate-agents.py

Exit code 0 when all agents are valid; non-zero otherwise.
"""

import os
from pathlib import Path
import re
import sys

try:
    import yaml
except ImportError:
    print("ERROR: PyYAML is required. Install with: pip install pyyaml", file=sys.stderr)
    sys.exit(2)

BASE = Path(__file__).resolve().parent
SKILLS_DIR = BASE.parent / "skills"
SPECS_DIR = BASE.parent / "specs"

REQUIRED = {"aether-id", "name", "description", "tools", "metadata"}
ALLOWED_TOP = {"aether-id", "name", "description", "tools", "metadata"}
ID_RE = re.compile(r"^[a-z]([a-z0-9-]*[a-z0-9])?$")
REQUIRED_META = {
    "aether-version",
    "aether-status",
    "aether-scope",
    "aether-domain",
    "aether-owners",
    "aether-created",
    "aether-updated",
}
ALLOWED_STATUSES = {"draft", "stable", "deprecated", "retired"}
ALLOWED_TOOLS = {"read", "search", "edit", "execute", "web"}
FRONTMATTER_RE = re.compile(r"^---\n(.*?)\n---\n", re.DOTALL)
REQUIRED_SECTIONS = [
    "## Mission",
    "## Operating contract",
    "## Workflow",
    "## Boundaries",
    "## Completion",
]


def find_agents(base: Path):
    results = []
    for child in sorted(base.iterdir()):
        if child.is_dir():
            agent_file = child / "AGENT.md"
            if agent_file.exists():
                results.append((child.name, agent_file))
    return results


def load_known_ids():
    """Return (skill_ids, spec_ids) from canonical library."""
    skill_ids = set()
    for root, _dirs, files in os.walk(SKILLS_DIR):
        if "SKILL.md" in files:
            skill_ids.add(Path(root).name)

    spec_ids = set()
    for spec_file in SPECS_DIR.rglob("*.spec.md"):
        text = spec_file.read_text(encoding="utf-8")
        m = FRONTMATTER_RE.match(text)
        if m:
            try:
                fm = yaml.safe_load(m.group(1)) or {}
                if "id" in fm:
                    spec_ids.add(fm["id"])
            except yaml.YAMLError:
                pass

    return skill_ids, spec_ids


def validate_agent(dir_name: str, path: Path, skill_ids: set, spec_ids: set):
    errors = []
    text = path.read_text(encoding="utf-8")

    m = FRONTMATTER_RE.match(text)
    if not m:
        return ["no YAML frontmatter block found"]

    try:
        fm = yaml.safe_load(m.group(1)) or {}
    except yaml.YAMLError as exc:
        return [f"YAML parse error: {exc}"]

    if not isinstance(fm, dict):
        return ["frontmatter is not a YAML mapping"]

    # Required fields
    for field in REQUIRED:
        if field not in fm:
            errors.append(f"missing required field: {field}")

    # aether-id must match directory name
    aether_id = fm.get("aether-id", "")
    if aether_id != dir_name:
        errors.append(f"aether-id '{aether_id}' does not match directory name '{dir_name}'")
    if not ID_RE.match(str(aether_id)):
        errors.append(f"aether-id '{aether_id}' is not valid kebab-case")
    elif len(str(aether_id)) > 64:
        errors.append(f"aether-id '{aether_id}' exceeds 64-character maximum")

    # tools must be explicit list of allowed values
    tools = fm.get("tools")
    if tools is None:
        errors.append(
            "tools field is required (must be explicit; omitting implies broad access on some hosts)"
        )
    elif not isinstance(tools, list):
        errors.append("tools must be a list")
    elif not tools:
        errors.append(
            "tools list must not be empty; use ['read'] for read-only roles instead of []"
        )
    else:
        for t in tools:
            if t not in ALLOWED_TOOLS:
                errors.append(f"unknown tool '{t}'; allowed: {sorted(ALLOWED_TOOLS)}")

    # metadata block
    meta = fm.get("metadata")
    if meta is not None:
        if not isinstance(meta, dict):
            errors.append("metadata must be a YAML mapping")
        else:
            for key in REQUIRED_META:
                if key not in meta:
                    errors.append(f"missing required metadata key: {key}")

            status = meta.get("aether-status", "")
            if status not in ALLOWED_STATUSES:
                errors.append(f"aether-status '{status}' not in {sorted(ALLOWED_STATUSES)}")

            # Validate skill dependencies resolve
            for skill_id in meta.get("aether-skills", []):
                if skill_id not in skill_ids:
                    errors.append(f"aether-skills references unknown skill id '{skill_id}'")

            # Validate spec dependencies resolve
            for spec_id in meta.get("aether-specs", []):
                if spec_id not in spec_ids:
                    errors.append(f"aether-specs references unknown spec id '{spec_id}'")

    # Body section validation
    body = text[m.end() :]
    for section in REQUIRED_SECTIONS:
        if section not in body:
            errors.append(f"body is missing required section: {section!r}")

    return errors


def main():
    agents = find_agents(BASE)
    if not agents:
        print("WARNING: no AGENT.md files found", file=sys.stderr)
        return 0

    skill_ids, spec_ids = load_known_ids()

    all_ids = []
    total_errors = 0

    for dir_name, path in agents:
        errors = validate_agent(dir_name, path, skill_ids, spec_ids)
        rel = path.relative_to(BASE.parent.parent.parent)
        if errors:
            for e in errors:
                print(f"ERROR [{rel}]: {e}")
            total_errors += len(errors)
        else:
            print(f"OK    [{rel}]")
        all_ids.append(dir_name)

    # Uniqueness check
    if len(all_ids) != len(set(all_ids)):
        print("ERROR: duplicate aether-id values detected")
        total_errors += 1

    if total_errors:
        print(f"\n{total_errors} error(s) found in {len(agents)} agent(s).", file=sys.stderr)
        return 1

    print(f"\n{len(agents)} agent(s) validated successfully.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
