---
name: create-pillars-document
description: Creates or updates PILLARS.md from repository evidence and identity context. Use when a project needs to define, repair, or review the foundational pillars that support its purpose and values.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "draft"
  aether-spec-id: "architecture-pillars"
  aether-scope: "organization"
  aether-domain: "architecture"
  aether-owners: "egohygiene"
  aether-created: "2026-08-01"
  aether-updated: "2026-08-01"
---

# Create Pillars Document

## Purpose

Create or update `PILLARS.md` in conformance with `architecture-pillars`.

Primary question:

> What enduring capabilities must remain strong for this work to succeed?

## Use This Skill When

- the canonical document does not exist
- an existing document is incomplete or inconsistent
- upstream identity has changed
- repository architecture is being established or repaired

## Required Inputs

- PURPOSE.md
- VISION.md
- PRINCIPLES.md
- historical and current strategic work
- recurring capabilities
- organizational strengths and gaps
- long-term responsibilities

Missing evidence must be recorded rather than invented.

## Workflow

1. Read upstream identity documents.
2. Identify recurring capabilities needed across multiple initiatives.
3. Remove temporary projects and technologies.
4. Group overlapping capabilities.
5. Define clear boundaries.
6. Validate contribution to purpose and vision.
7. Test whether future initiatives can align to the set.
8. Keep the set intentionally small.

## Output Contract

Produce:

- `PILLARS.md`
- governing specification identifier and version
- assumptions and unresolved questions
- validation results
- downstream review recommendations

## Constraints

- Preserve canonical terminology.
- Separate evidence from inference.
- Do not fabricate intent.
- Do not introduce implementation details outside the specification.
- Do not silently resolve contradictions.
- Do not claim completion when upstream artifacts are missing.

## Validation

Use `references/validation-checklist.md` and the acceptance criteria in
`architecture-pillars`.

## Completion Criteria

- [ ] The governing specification is identified.
- [ ] Required upstream evidence has been read.
- [ ] The document answers its primary identity question.
- [ ] Non-responsibilities are respected.
- [ ] Assumptions and open questions are visible.
- [ ] Structural, semantic, relationship, and evidence checks pass.
- [ ] Downstream review needs are reported.
