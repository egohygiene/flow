---
name: audit-repository
description: Audits a repository for quality, alignment, risk, and strengths using observable evidence. Use when a project needs an impartial assessment of its current state and actionable findings.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "draft"
  aether-spec-id: "auditor"
  aether-scope: "organization"
  aether-domain: "quality"
  aether-owners: "egohygiene"
  aether-created: "2026-08-02"
  aether-updated: "2026-08-02"
---

# Audit Repository

## Purpose

Execute the reusable procedure governed by `auditor`.

Primary question:

> What does observable repository evidence show about quality, alignment, risk, and strengths?

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

1. resolve request, defaults, scope, and read-only constraints
2. inspect repository context in the required order
3. inspect historical audits
4. gather evidence within scope
5. classify findings and positive observations
6. assign severity, confidence, status, effort, and impact
7. record uncertainty and uninspected areas
8. propose bounded validation and candidate issues
9. write and validate the immutable report

## Output Contract

Primary output:

    audits/{audit-name}-{utc-timestamp}.md

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

A staged candidate at `.staging/skills/repository-audit/` covers similar
ground under the name `repository-audit`. The canonical skill is named
`audit-repository` and is governed by the `auditor` specification.

Issue 016 should compare the two and extract any unique findings workflow,
output format, or severity taxonomy from the staged copy before retiring it.
Do not copy the staged file wholesale into canonical source.
