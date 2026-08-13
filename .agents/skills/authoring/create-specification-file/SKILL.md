---
name: create-specification-file
description: Creates a new Aether specification file conforming to the specfile standard. Use when a project needs to define, structure, or validate an implementation contract and its acceptance criteria.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "draft"
  aether-spec-id: "specfile"
  aether-scope: "organization"
  aether-domain: "authoring"
  aether-owners: "egohygiene"
  aether-created: "2026-08-02"
  aether-updated: "2026-08-02"
---

# Create Specification File

## Purpose

Execute the reusable procedure governed by `specfile`.

Primary question:

> What implementation contract is required, and how will conformance be validated?

## Required Inputs

Resolve:

- governing specification and version
- current source or repository state
- scope and constraints
- upstream architecture or evidence
- output location
- validation expectations
- unresolved decisions

Missing evidence must remain visible.

## Workflow

1. gather source context and identify the problem
2. separate facts, decisions, assumptions, and proposals
3. define purpose, goals, and non-goals
4. choose type and canonical ownership
5. define requirements, architecture, interfaces, and dependencies
6. organize implementation work into bounded phases
7. define validation and acceptance criteria
8. record risks, edge cases, and open questions
9. validate against the specification standard

## Output Contract

Primary output:

    <artifact-id>.spec.md

Also report assumptions, evidence gaps, validation status, unresolved
questions, and downstream actions requiring separate authorization.

## Constraints

- Follow the governing specification.
- Preserve provenance and uncertainty.
- Do not invent authority, evidence, or current behavior.
- Do not silently expand scope.
- Do not claim completion when required validation is missing.
- Keep proposed downstream work separate from authorized execution.

## Completion Criteria

- [ ] Governing specification is resolved.
- [ ] Scope and constraints are explicit.
- [ ] Required evidence was inspected.
- [ ] The primary output was created.
- [ ] Validation was executed or its absence documented.
- [ ] Open questions and authorization needs are visible.

## Staged Variant

A staged candidate at `.staging/skills/spec-authoring/` covers similar
ground under the name `spec-authoring`. The canonical skill is named
`create-specification-file` and is governed by the `specfile` specification.

Issue 016 should compare the two and extract any unique workflow guidance,
template detail, or example material from the staged copy before retiring it.
Do not copy the staged file wholesale into canonical source.
