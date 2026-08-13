---
name: create-roadmap-document
description: Creates or updates ROADMAP.md from repository evidence. Use when a project needs to define, repair, or review its strategic capability evolution and sequencing across time horizons.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "draft"
  aether-spec-id: "architecture-roadmap"
  aether-scope: "organization"
  aether-domain: "architecture"
  aether-owners: "egohygiene"
  aether-created: "2026-08-01"
  aether-updated: "2026-08-08"
---

# Create Roadmap Document

## Purpose

Create, update, or validate `ROADMAP.md` in conformance with
`architecture-roadmap`.

Primary question:

> How should this project evolve over time?

## Use This Skill When

- the canonical document is missing
- strategic direction is scattered across issues, chat, or ad hoc notes
- capability evolution needs now/next/later/maybe framing
- major initiatives must be sequenced without overspecifying tasks or dates
- the roadmap needs review after purpose, vision, architecture, or methodology changed

## Do Not Use This Skill For

- turning every future idea into a committed obligation
- building an issue queue, sprint plan, or implementation checklist
- writing release notes or exact delivery dates unless strategically necessary
- using tool choices or current tickets as the main roadmap structure
- redefining system or architecture ownership instead of consuming it

## Required Inputs

Resolve:

- governing specification and version
- upstream purpose, vision, principles, foundations, system, architecture, and methodology documents
- current project posture and major capability gaps
- known strategic initiatives, constraints, and sequencing pressures
- uncertainties, dependencies, and open questions that affect long-horizon planning

## Optional Inputs

Use when available:

- accepted ADRs with roadmap implications
- milestone or release history that reveals capability evolution
- market, user, or organizational evidence that changes strategic priority

Missing evidence must be recorded rather than invented.

## Workflow

1. Read `.agents/specs/architecture/foundation/roadmap.spec.md`.
2. Read the upstream vision and current architectural state before sequencing future work.
3. Identify capability evolution and strategic initiatives rather than individual tasks.
4. Group the future into clear horizons such as now, next, later, and maybe.
5. Keep optionality visible so speculative ideas do not become obligations.
6. Use dates only when strategically necessary; prefer outcomes, capabilities, and sequence.
7. If evidence conflicts or priority is unresolved, preserve the uncertainty and report provisional or blocked completion.
8. Draft or update `ROADMAP.md` using `templates/ROADMAP.template.md`.
9. Validate with `references/validation-checklist.md` and `references/authoring-guide.md`.

## Output Contract

Produce or update:

- `ROADMAP.md`
- governing specification identifier and version
- explicit strategic horizons, initiatives, dependencies, and optional items
- assumptions, contradictions, and open questions
- validation results
- downstream review recommendations for issues, milestones, or planning artifacts

## Boundaries

`ROADMAP.md` owns strategic sequencing, future capability evolution, and major
phases or horizons.

It does not own sprint plans, issue queues, implementation steps, release notes,
or architectural rationale already owned elsewhere.

## Validation

Use [references/validation-checklist.md](references/validation-checklist.md)
for mandatory checks and
[references/authoring-guide.md](references/authoring-guide.md) for horizon,
optionality, and anti-pattern guidance.

## Blocked or Provisional Outcomes

If the vision is missing, if the future is only a tactical backlog, or if major
strategic priorities are contradictory, do not pretend the roadmap is settled.
Report whether the result is blocked, provisional, or limited to a clearly
scoped horizon update.

## Completion Criteria

- [ ] `architecture-roadmap` is identified as the governing specification.
- [ ] Required upstream evidence has been read.
- [ ] The primary question is answered directly.
- [ ] Strategic horizons and initiatives are explicit.
- [ ] Ownership boundaries with backlog, release, and architecture artifacts are respected.
- [ ] Optionality, assumptions, contradictions, and open questions are visible.
- [ ] Acceptance criteria and package-level validation checks pass.
