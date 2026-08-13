---
name: create-ontology-document
description: Creates or refines ONTOLOGY.md using the corresponding Aether architecture specification. Use when a project needs to establish or validate its canonical vocabulary and domain model.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "draft"
  aether-spec-id: "architecture-ontology"
  aether-scope: "organization"
  aether-domain: "architecture"
  aether-owners: "egohygiene"
  aether-created: "2026-08-02"
  aether-updated: "2026-08-02"
---

# Create Ontology Document

## Purpose

Create or update `ONTOLOGY.md` in conformance with `architecture-ontology`.

The document must answer:

> What exists in this domain, what does each concept mean, and how do the concepts relate?

## Use This Skill When

- the canonical document is missing
- terminology or assumptions are inconsistent
- upstream architecture has changed
- a repository is establishing or repairing its domain model
- downstream architecture needs a stable canonical reference

## Required Inputs

Resolve the governing specification, upstream architecture, relevant
evidence, existing terminology or assumptions, and downstream consumers.

Missing evidence must be recorded rather than invented.

## Workflow

1. define the domain boundary
2. collect terminology from architecture, documentation, code, and users
3. separate concepts from implementation artifacts
4. group synonyms and distinguish overloaded terms
5. assign stable identifiers and canonical terms
6. write definitions, relationships, and invariants
7. record aliases and deprecated terms
8. surface conflicts and unknowns
9. report downstream migration impact

## Output Contract

Produce:

- `ONTOLOGY.md`
- governing specification identifier and version
- assumptions and unresolved questions
- validation results
- downstream migration or review recommendations

## Constraints

- Preserve canonical terminology.
- Separate evidence, assumptions, and inference.
- Do not fabricate domain or human knowledge.
- Do not silently resolve contradictions.
- Do not leak implementation structure into canonical concepts.
- Do not claim completion when required evidence is missing.

## Validation

Use:

    references/validation-checklist.md

and the acceptance criteria in:

    architecture-ontology

## Completion Criteria

- [ ] The governing specification is identified.
- [ ] Required upstream artifacts have been read.
- [ ] The primary question is answered.
- [ ] Boundaries are respected.
- [ ] Assumptions and open questions are visible.
- [ ] Structural, semantic, relationship, and evidence checks pass.
- [ ] Downstream impacts are reported.
