---
name: create-architecture-document
description: Creates or updates ARCHITECTURE.md from repository evidence. Use when a project needs to define, repair, or review its structural organization, boundaries, and dependency rules.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "draft"
  aether-spec-id: "architecture-architecture"
  aether-scope: "organization"
  aether-domain: "architecture"
  aether-owners: "egohygiene"
  aether-created: "2026-08-01"
  aether-updated: "2026-08-08"
---

# Create Architecture Document

## Purpose

Create, update, or validate `ARCHITECTURE.md` in conformance with
`architecture-architecture`.

Primary question:

> How is this project structurally organized to accomplish its purpose?

## Use This Skill When

- the canonical document is missing
- structural boundaries are unclear or conflicting
- major subsystems or dependency directions changed
- `SYSTEM.md` exists but the structural organization is still implicit
- architecture review needs a canonical explanation of layers, boundaries, or communication patterns

## Do Not Use This Skill For

- redefining which major systems exist when `SYSTEM.md` is the real gap
- documenting deployment topology, provisioning, or infrastructure operations
- recording decision rationale owned by `DECISIONS.md`
- writing detailed APIs, algorithms, or source-file walkthroughs
- producing an implementation plan or roadmap

## Required Inputs

Resolve:

- governing specification and version
- `SYSTEM.md`
- relevant upstream identity and foundation documents
- current repository or system boundary evidence
- known architectural constraints, contradictions, and open questions

## Optional Inputs

Use when available:

- `DESIGN.md`
- `ONTOLOGY.md`
- existing diagrams or decomposition notes
- accepted ADRs that constrain structure

Missing evidence must be recorded rather than invented.

## Workflow

1. Read `.agents/specs/architecture/foundation/architecture.spec.md`.
2. Read the upstream documents that define purpose, foundations, and the current system model.
3. Identify the stable structural units, layers, or subsystems that actually organize the project.
4. Separate system inventory from structural organization so `SYSTEM.md` and `ARCHITECTURE.md` do not become duplicates.
5. Make boundaries, dependency direction, coordination patterns, and important constraints explicit.
6. Keep tactical implementation, deployment, and ADR rationale out of the document unless the specification requires a concise reference.
7. If evidence conflicts, preserve the conflict, label assumptions, and report provisional or blocked completion.
8. Draft or update `ARCHITECTURE.md` using `templates/ARCHITECTURE.template.md`.
9. Validate with `references/validation-checklist.md` and `references/authoring-guide.md`.

## Output Contract

Produce or update:

- `ARCHITECTURE.md`
- governing specification identifier and version
- explicit assumptions, contradictions, and open questions
- validation results
- downstream review recommendations for affected documents such as `SYSTEM.md`, `DESIGN.md`, or ADRs

## Boundaries

`ARCHITECTURE.md` owns structural organization, boundaries, dependency direction,
and communication patterns.

It does not own system decomposition best handled by `SYSTEM.md`, durable
rationale owned by `DECISIONS.md`, roadmap sequencing, or implementation detail.

## Validation

Use [references/validation-checklist.md](references/validation-checklist.md)
for mandatory checks and
[references/authoring-guide.md](references/authoring-guide.md) for
boundary, evidence, and anti-pattern guidance.

## Blocked or Provisional Outcomes

If the system model is missing, structure is contradicted by evidence, or major
boundaries cannot be justified, do not fabricate a final architecture. Report
whether the outcome is blocked, provisional, or limited to a partial update.

## Completion Criteria

- [ ] `architecture-architecture` is identified as the governing specification.
- [ ] Required upstream evidence has been read.
- [ ] The primary question is answered directly.
- [ ] Structural boundaries and dependency direction are explicit.
- [ ] Ownership boundaries with `SYSTEM.md` and `DECISIONS.md` are respected.
- [ ] Assumptions, contradictions, and open questions are visible.
- [ ] Acceptance criteria and package-level validation checks pass.
