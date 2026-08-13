---
name: create-system-document
description: Creates or updates SYSTEM.md from repository evidence. Use when a project needs to define, repair, or review its logical system decomposition, capability ownership, and high-level interactions.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "draft"
  aether-spec-id: "architecture-system"
  aether-scope: "organization"
  aether-domain: "architecture"
  aether-owners: "egohygiene"
  aether-created: "2026-08-01"
  aether-updated: "2026-08-08"
---

# Create System Document

## Purpose

Create, update, or validate `SYSTEM.md` in conformance with
`architecture-system`.

Primary question:

> What systems make up this project, and what does each one own?

## Use This Skill When

- the canonical document is missing
- major capabilities or ownership boundaries are unclear
- repository structure is misleading the conceptual system model
- new or changed capabilities require the system decomposition to be updated
- architecture work needs a stable upstream inventory of major systems and interactions

## Do Not Use This Skill For

- explaining architectural layers or dependency direction better owned by `ARCHITECTURE.md`
- documenting APIs, modules, packages, or source files in detail
- writing deployment topology or infrastructure configuration
- sequencing future work or release plans
- using framework or vendor boundaries as the system model by default

## Required Inputs

Resolve:

- governing specification and version
- upstream purpose, vision, foundations, ontology, and other domain-defining documents
- the major capabilities the project must provide
- known system boundaries, external relationships, and overlapping ownership conflicts
- contradictions, assumptions, and open questions in the current system model

## Optional Inputs

Use when available:

- `PERSONAL_MODEL.md` or other domain-specific context documents
- existing diagrams or component inventories
- accepted ADRs that constrain capability ownership

Missing evidence must be recorded rather than invented.

## Workflow

1. Read `.agents/specs/architecture/foundation/system.spec.md`.
2. Read the upstream identity, foundation, and domain documents before naming systems.
3. Identify major capabilities first, then group them into cohesive systems.
4. Assign one primary owner for each capability and make boundaries explicit.
5. Describe major high-level interactions and external relationships without collapsing into implementation or source layout.
6. Keep the decomposition conceptual so temporary repository organization does not dictate the model.
7. If evidence conflicts, preserve the disagreement, label assumptions, and report provisional or blocked completion.
8. Draft or update `SYSTEM.md` using `templates/SYSTEM.template.md`.
9. Validate with `references/validation-checklist.md` and `references/authoring-guide.md`.

## Output Contract

Produce or update:

- `SYSTEM.md`
- governing specification identifier and version
- explicit system inventory, capability ownership, boundaries, and high-level interactions
- assumptions, contradictions, and open questions
- validation results
- downstream review recommendations for `ARCHITECTURE.md`, designs, or implementation planning

## Boundaries

`SYSTEM.md` owns the logical decomposition into major systems, their purposes,
responsibilities, capability ownership, and high-level context.

It does not own module-level structure, architectural layer rules, detailed
APIs, deployment topology, or roadmap sequencing.

## Validation

Use [references/validation-checklist.md](references/validation-checklist.md)
for mandatory checks and
[references/authoring-guide.md](references/authoring-guide.md) for
capability, ownership, and anti-pattern guidance.

## Blocked or Provisional Outcomes

If major capabilities are not yet understood, if ownership boundaries are
contested, or if the evidence only supports an implementation artifact list, do
not fabricate a stable system model. Report whether the outcome is blocked,
provisional, or limited to a partial update.

## Completion Criteria

- [ ] `architecture-system` is identified as the governing specification.
- [ ] Required upstream evidence has been read.
- [ ] The primary question is answered directly.
- [ ] Major systems, capabilities, and ownership boundaries are explicit.
- [ ] Ownership boundaries with `ARCHITECTURE.md` and implementation artifacts are respected.
- [ ] Assumptions, contradictions, and open questions are visible.
- [ ] Acceptance criteria and package-level validation checks pass.
