---
name: create-principles-document
description: Creates or updates PRINCIPLES.md from repository evidence and values. Use when a project needs to define, repair, or review the guiding principles that govern its decisions and behavior.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "draft"
  aether-spec-id: "architecture-principles"
  aether-scope: "organization"
  aether-domain: "architecture"
  aether-owners: "egohygiene"
  aether-created: "2026-08-01"
  aether-updated: "2026-08-01"
---

# Create Principles Document

## Purpose

Create or update `PRINCIPLES.md` in conformance with `architecture-principles`.

Primary question:

> How should decisions be evaluated when multiple valid options exist?

## Use This Skill When

- the canonical document does not exist
- an existing document is incomplete or inconsistent
- upstream identity has changed
- repository architecture is being established or repaired

## Required Inputs

- PURPOSE.md
- VISION.md
- organizational values
- recurring historical trade-offs
- architectural and engineering experience
- existing policies and standards

Missing evidence must be recorded rather than invented.

## Workflow

1. Read purpose and vision.
2. Identify recurring decisions and tensions.
3. Convert durable lessons into heuristics.
4. Remove technology-specific language.
5. Test each principle against real trade-offs.
6. Identify conflicts and precedence.
7. Distinguish principles from policies and standards.
8. Validate memorability and applicability.

## Output Contract

Produce:

- `PRINCIPLES.md`
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
`architecture-principles`.

## Completion Criteria

- [ ] The governing specification is identified.
- [ ] Required upstream evidence has been read.
- [ ] The document answers its primary identity question.
- [ ] Non-responsibilities are respected.
- [ ] Assumptions and open questions are visible.
- [ ] Structural, semantic, relationship, and evidence checks pass.
- [ ] Downstream review needs are reported.
