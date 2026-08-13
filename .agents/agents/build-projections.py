#!/usr/bin/env python3
# Copyright 2026 Ego Hygiene
# SPDX-License-Identifier: MIT

"""Build GitHub repository and organization agent projections from canonical source.

Usage:
    python3 library/organization/agents/build-projections.py [--check] [--output-directory dist]

Options:
    --check    Verify that existing projections are up to date; exit non-zero if drift detected.

Canonical source: library/organization/agents/<id>/AGENT.md
Repository projection: dist/github/repository/.github/agents/<id>.agent.md
Organization projection: dist/github/organization/agents/<id>.agent.md

Projection rules
----------------
1. The ``aether-id`` frontmatter field is removed.
2. The entire ``metadata`` block is removed.
3. Internal skill links (``../skills/<domain>/<skill>/SKILL.md``) are rewritten
   to the consumer-local path ``.agents/skills/<skill>/SKILL.md``.
4. Internal spec links (``../specs/<path>/<file>.spec.md``) are rewritten to
   ``.github/specs/<path>/<file>.spec.md`` for repository projections and
   ``specs/<path>/<file>.spec.md`` for organization projections.
5. The output file is named ``<aether-id>.agent.md``.
"""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import sys

try:
    import yaml
except ImportError:
    print("ERROR: PyYAML is required. Install with: pip install pyyaml", file=sys.stderr)
    sys.exit(2)

REPO_ROOT = Path(__file__).resolve().parents[3]
AGENTS_DIR = REPO_ROOT / "library" / "organization" / "agents"
REPO_PROJ = REPO_ROOT / "dist" / "github" / "repository" / ".github" / "agents"
ORG_PROJ = REPO_ROOT / "dist" / "github" / "organization" / "agents"

FRONTMATTER_RE = re.compile(r"^---\n(.*?)\n---\n", re.DOTALL)

# Rewrite ``../../skills/<domain>/<skill>/SKILL.md``  →  ``.agents/skills/<skill>/SKILL.md``
_SKILL_LINK_RE = re.compile(r"(?:\.\./){2}skills/[^/]+/([^/]+)/SKILL\.md")

# Rewrite ``../../specs/<path>`` — capture remaining path after ``../../specs/``
_SPEC_LINK_RE = re.compile(r"(?:\.\./){2}specs/([^\s\)\"']+)")

STRIP_FIELDS = {"aether-id", "metadata"}


def _rewrite_links(body: str, spec_prefix: str) -> str:
    """Rewrite source-relative library links to consumer-local paths."""
    body = _SKILL_LINK_RE.sub(lambda m: f".agents/skills/{m.group(1)}/SKILL.md", body)
    return _SPEC_LINK_RE.sub(lambda m: f"{spec_prefix}/{m.group(1)}", body)


def _build_frontmatter(fm: dict) -> str:
    """Emit minimal GitHub-compatible frontmatter (name, description, tools only)."""
    out = {"name": fm["name"], "description": fm["description"], "tools": fm["tools"]}
    return yaml.dump(out, default_flow_style=False, allow_unicode=True, sort_keys=False)


def project_agent(source: Path, spec_prefix: str) -> str:
    """Return the projected file content for an agent."""
    text = source.read_text(encoding="utf-8")
    m = FRONTMATTER_RE.match(text)
    if not m:
        raise ValueError(f"no frontmatter in {source}")
    fm = yaml.safe_load(m.group(1)) or {}
    body = text[m.end() :]
    body = _rewrite_links(body, spec_prefix)
    header = f"---\n{_build_frontmatter(fm)}---\n"
    return header + body


def find_agents() -> list[tuple[str, Path]]:
    results = []
    for child in sorted(AGENTS_DIR.iterdir()):
        if child.is_dir():
            source = child / "AGENT.md"
            if source.exists():
                results.append((child.name, source))
    return results


def build(check: bool = False, output_directory: Path | None = None) -> int:
    base_output = output_directory or (REPO_ROOT / "dist")
    repo_proj = base_output / "github" / "repository" / ".github" / "agents"
    org_proj = base_output / "github" / "organization" / "agents"
    agents = find_agents()
    if not agents:
        print("WARNING: no AGENT.md files found", file=sys.stderr)
        return 0

    repo_proj.mkdir(parents=True, exist_ok=True)
    org_proj.mkdir(parents=True, exist_ok=True)

    drift = 0

    for agent_id, source in agents:
        filename = f"{agent_id}.agent.md"

        repo_content = project_agent(source, ".github/specs")
        org_content = project_agent(source, "specs")

        for out_path, content in [
            (repo_proj / filename, repo_content),
            (org_proj / filename, org_content),
        ]:
            rel = _normalized_display_path(out_path)
            if check:
                if not out_path.exists():
                    print(f"DRIFT  missing projection: {rel}")
                    drift += 1
                elif out_path.read_text(encoding="utf-8") != content:
                    print(f"DRIFT  stale projection: {rel}")
                    drift += 1
                else:
                    print(f"OK     {rel}")
            else:
                out_path.write_text(content, encoding="utf-8")
                print(f"wrote  {rel}")

    if check and drift:
        print(
            f"\n{drift} projection(s) out of date. Run build-projections.py to regenerate.",
            file=sys.stderr,
        )
        return 1

    if not check:
        total = len(agents) * 2
        print(f"\n{total} projection(s) written for {len(agents)} agent(s).")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check", action="store_true", help="Check for drift without writing files"
    )
    parser.add_argument(
        "--output-directory",
        default="dist",
        help="Base output directory for generated projections.",
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
