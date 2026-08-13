---
name: create-foundations-document
description: Creates or updates FOUNDATIONS.md from repository evidence. Use when a project needs to define, repair, or review enduring assumptions, invariants, and baseline architectural constraints.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "draft"
  aether-spec-id: "architecture-foundations"
  aether-scope: "organization"
  aether-domain: "architecture"
  aether-owners: "egohygiene"
  aether-created: "2026-08-01"
  aether-updated: "2026-08-08"
---

# Create Foundations Document

## Purpose

Create, update, or validate `FOUNDATIONS.md` in conformance with
`architecture-foundations`.

Primary question:

> What enduring truths does the architecture depend on?

## Use This Skill When

- the canonical document is missing
- assumptions or invariants are implicit, contradictory, or drifting
- downstream documents need a stable architectural baseline
- the project must separate durable truths from principles, plans, or implementation choices
- falsified assumptions need an explicit canonical update

## Do Not Use This Skill For

- listing temporary engineering preferences or tool choices as foundations
- decomposing systems or layers owned by `SYSTEM.md` or `ARCHITECTURE.md`
- writing methodology, roadmap, or sprint guidance
- recording rationale better owned by `DECISIONS.md`
- promoting an unverified belief into canonical truth

## Required Inputs

Resolve:

- governing specification and version
- upstream identity documents such as `PURPOSE.md`, `VISION.md`, and `PRINCIPLES.md`
- durable assumptions already relied on by the project
- known invariants, baseline constraints, and conceptual models
- evidence of contradictions, failures, or recently falsified assumptions

## Optional Inputs

Use when available:

- `MANIFESTO.md` and `PILLARS.md`
- accepted ADRs that made a baseline assumption explicit
- research or domain evidence that supports a durable architectural truth

Missing evidence must be recorded rather than invented.

## Workflow

1. Read `.agents/specs/architecture/foundation/foundations.spec.md`.
2. Read the upstream identity and philosophy documents before touching downstream architecture.
3. Separate durable assumptions and invariants from principles, decisions, tactics, and current implementation.
4. Keep the set small, stable, and useful for downstream reasoning.
5. State baseline constraints explicitly when later documents must respect them.
6. If an assumption has been falsified, update or remove it explicitly and record the uncertainty or replacement condition rather than smoothing over the change.
7. Draft or update `FOUNDATIONS.md` using `templates/FOUNDATIONS.template.md`.
8. Validate with `references/validation-checklist.md` and `references/authoring-guide.md`.

## Output Contract

Produce or update:

- `FOUNDATIONS.md`
- governing specification identifier and version
- explicit assumptions, invariants, and evidence gaps
- notes about falsified, revised, or provisional foundations
- validation results
- downstream review recommendations for dependent documents

## Boundaries

`FOUNDATIONS.md` owns enduring assumptions, invariants, conceptual starting
points, and baseline reasoning constraints.

It does not own system decomposition, structural layout, task workflow,
roadmap sequencing, or temporary implementation choices.

## Validation

Use [references/validation-checklist.md](references/validation-checklist.md)
for mandatory checks and
[references/authoring-guide.md](references/authoring-guide.md) for
examples, update rules, and anti-patterns.

## Blocked or Provisional Outcomes

If the project lacks evidence for a claimed foundation, if assumptions are
currently under dispute, or if a recently falsified belief has no validated
replacement, keep the result provisional. Do not present fragile or contested
claims as durable truths.

## Completion Criteria

- [ ] `architecture-foundations` is identified as the governing specification.
- [ ] Required upstream evidence has been read.
- [ ] The primary question is answered directly.
- [ ] Durable assumptions and invariants are explicit.
- [ ] Ownership boundaries with principles, systems, and methodology are respected.
- [ ] Falsified or contested assumptions are handled explicitly.
- [ ] Acceptance criteria and package-level validation checks pass.
