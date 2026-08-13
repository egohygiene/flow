---
name: create-methodology-document
description: Creates or updates METHODOLOGY.md from repository evidence. Use when a project needs to define, repair, or review its repeatable way of moving from ideas to validated outcomes.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "draft"
  aether-spec-id: "architecture-methodology"
  aether-scope: "organization"
  aether-domain: "architecture"
  aether-owners: "egohygiene"
  aether-created: "2026-08-01"
  aether-updated: "2026-08-08"
---

# Create Methodology Document

## Purpose

Create, update, or validate `METHODOLOGY.md` in conformance with
`architecture-methodology`.

Primary question:

> How does this project intentionally move from ideas to validated outcomes?

## Use This Skill When

- the canonical document is missing
- the working method is implicit, inconsistent, or drifting
- human and AI collaboration patterns need a stable shared process
- validation loops or feedback cycles must be clarified
- a project needs durable methodology rather than ad hoc habits

## Do Not Use This Skill For

- writing a backlog, sprint plan, or issue queue
- encoding CI implementation detail as the methodology itself
- documenting repository structure owned by `ARCHITECTURE.md`
- producing a roadmap or release schedule
- turning temporary habits into permanent method without evidence

## Required Inputs

Resolve:

- governing specification and version
- upstream purpose, principles, foundations, and architecture documents
- current working patterns used by humans and AI systems
- explicit validation and feedback loops
- known process pain points, contradictions, and open questions

## Optional Inputs

Use when available:

- `DECISIONS.md` entries that changed the operating model
- automation or review workflows that reflect durable method
- contributor guidance or runbooks that reveal repeatable patterns

Missing evidence must be recorded rather than invented.

## Workflow

1. Read `.agents/specs/architecture/foundation/methodology.spec.md`.
2. Read the upstream identity, foundation, and architecture documents before modeling the workflow.
3. Separate the durable operating method from one-off tasks, current tooling quirks, and scheduling artifacts.
4. Identify the real stages, loops, validation points, and feedback mechanisms.
5. Capture human and AI collaboration only when it is a durable part of how work is done.
6. Keep the methodology implementation-light so it survives tooling or provider changes.
7. If evidence conflicts, preserve the disagreement, label assumptions, and report provisional or blocked completion.
8. Draft or update `METHODOLOGY.md` using `templates/METHODOLOGY.template.md`.
9. Validate with `references/validation-checklist.md` and `references/authoring-guide.md`.

## Output Contract

Produce or update:

- `METHODOLOGY.md`
- governing specification identifier and version
- explicit workflow stages, validation loops, and feedback mechanisms
- assumptions, contradictions, and open questions
- validation results
- downstream review recommendations for automation or contribution guidance

## Boundaries

`METHODOLOGY.md` owns the repeatable way work is performed, validated, and
improved.

It does not own issue lists, roadmap sequencing, repository topology,
infrastructure layout, or the low-level implementation of CI/CD tools.

## Validation

Use [references/validation-checklist.md](references/validation-checklist.md)
for mandatory checks and
[references/authoring-guide.md](references/authoring-guide.md) for stage,
feedback, and anti-pattern guidance.

## Blocked or Provisional Outcomes

If the project has no consistent operating model yet, if the evidence only shows
ad hoc tasks, or if human and AI workflows materially disagree, do not invent a
stable methodology. Report whether the result is blocked, provisional, or only
suitable as an observed current-state draft.

## Completion Criteria

- [ ] `architecture-methodology` is identified as the governing specification.
- [ ] Required upstream evidence has been read.
- [ ] The primary question is answered directly.
- [ ] Stages, validation loops, and feedback cycles are explicit.
- [ ] Ownership boundaries with roadmap, architecture, and CI implementation are respected.
- [ ] Assumptions, contradictions, and open questions are visible.
- [ ] Acceptance criteria and package-level validation checks pass.
