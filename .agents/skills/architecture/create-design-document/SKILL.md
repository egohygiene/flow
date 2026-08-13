---
name: create-design-document
description: Creates or refines DESIGN.md using the corresponding Aether architecture specification. Use when a project needs to establish, update, or validate its design experience documentation.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "draft"
  aether-spec-id: "architecture-design"
  aether-scope: "organization"
  aether-domain: "architecture"
  aether-owners: "egohygiene"
  aether-created: "2026-08-02"
  aether-updated: "2026-08-02"
---

# Create Design Document

## Purpose

Create or update `DESIGN.md` in conformance with
`architecture-design`.

The document must answer:

> What kind of experience should this create, and why should it feel that way?

## Use This Skill When

- the canonical experience document is missing
- experience intent or design language is inconsistent
- upstream identity or personal-model architecture has changed
- a product is establishing or repairing its experience architecture
- downstream design or implementation needs a stable source of truth

## Required Inputs

Resolve:

- the governing specification
- required upstream architecture documents
- relevant user, accessibility, and design evidence
- existing product experiences
- known inconsistencies and exceptions
- downstream consumers

Missing evidence must be recorded rather than invented.

## Workflow

1. read purpose, principles, and the personal model
2. inventory existing experience claims and assumptions
3. separate experience intent from visual or technical implementation
4. define experiential qualities, interaction philosophy, and communication philosophy
5. define accessibility, agency, cognitive-load, trust, and recovery expectations
6. define aesthetic direction and design anti-goals
7. test the philosophy across multiple products and surfaces
8. record evidence, assumptions, contradictions, and open questions
9. identify downstream design-system implications

## Output Contract

Produce:

- `DESIGN.md`
- governing specification identifier and version
- evidence, assumptions, and unresolved questions
- validation results
- downstream migration or review recommendations

## Constraints

- Preserve human agency and accessibility.
- Separate experience intent from implementation.
- Do not fabricate user needs or research findings.
- Do not silently resolve contradictions.
- Do not treat one product implementation as universally canonical.
- Do not claim completion when required evidence is missing.

## Validation

Use:

    references/validation-checklist.md

and the acceptance criteria in:

    architecture-design

## Completion Criteria

- [ ] The governing specification is identified.
- [ ] Required upstream artifacts have been read.
- [ ] The primary question is answered.
- [ ] Responsibilities and non-responsibilities are respected.
- [ ] Accessibility and agency are addressed.
- [ ] Evidence, assumptions, and open questions are visible.
- [ ] Downstream impacts are reported.
