---
name: create-design-system-document
description: Creates or refines DESIGN_SYSTEM.md using the corresponding Aether architecture specification. Use when a project needs to establish or update its canonical design system documentation.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "draft"
  aether-spec-id: "architecture-design-system"
  aether-scope: "organization"
  aether-domain: "architecture"
  aether-owners: "egohygiene"
  aether-created: "2026-08-02"
  aether-updated: "2026-08-02"
---

# Create Design System Document

## Purpose

Create or update `DESIGN_SYSTEM.md` in conformance with
`architecture-design-system`.

The document must answer:

> How should the intended experience be expressed consistently across products, platforms, and implementations?

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

1. read the design philosophy and personal model
2. inventory existing visual, interaction, content, and motion patterns
3. separate semantic roles from implementation values
4. identify reusable patterns, variants, and local exceptions
5. define accessibility guarantees and input-state coverage
6. define hierarchy, typography, color, spacing, surfaces, imagery, and motion
7. define feedback, errors, recovery, responsive behavior, and content patterns
8. define product-identity variation and theming boundaries
9. define contribution, governance, and deprecation behavior
10. validate downstream token and component implementability

## Output Contract

Produce:

- `DESIGN_SYSTEM.md`
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

    architecture-design-system

## Completion Criteria

- [ ] The governing specification is identified.
- [ ] Required upstream artifacts have been read.
- [ ] The primary question is answered.
- [ ] Responsibilities and non-responsibilities are respected.
- [ ] Accessibility and agency are addressed.
- [ ] Evidence, assumptions, and open questions are visible.
- [ ] Downstream impacts are reported.
