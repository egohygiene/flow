# Repository-Local Agent Framework

This directory contains the Aether specifications, focused authoring/quality
skills, agent role definitions, templates, evaluation fixtures, and validators
used by Flow contributors.

It is repository-local guidance. It is not installed as a personal ChatGPT or
Codex skill collection.

## Layout

| Path | Purpose |
| --- | --- |
| `specs/` | Canonical contracts for architecture and other repository artifacts |
| `skills/` | Task-specific workflows, references, templates, and evaluations |
| `agents/` | Role definitions and their skill/spec relationships |

## Validate

Run from the repository root:

```bash
python3 .agents/specs/validate-specs.py
python3 .agents/skills/validate-skills.py
python3 .agents/agents/validate-agents.py
```

The distribution/projection builders are retained from the uploaded Aether
bundle for provenance, but their source-library paths target the upstream
organization layout. Flow's supported initialization surface is `.agents/`
itself plus the validators above; generated distributions are not committed.

## Author architecture

Begin with `skills/architecture/architecture-authoring/SKILL.md`, resolve the
applicable specification under `specs/architecture/`, and then use the focused
document skill and template. Architecture documents live in
`docs/architecture/`; the strategic roadmap is `ROADMAP.md` at the repository
root.

Keep shared policy at suite level. Holon documents refine only the concern owned
by that holon. Record intentional omissions in `docs/architecture/meta/META.md`.
