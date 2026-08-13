---
name: bug-fixing
description: Reproduces, diagnoses, fixes, and validates a concrete software defect with the smallest safe change and regression protection. Use when a bug report, failing test, regression, crash, or incorrect behavior requires corrective engineering work.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "experimental"
  aether-scope: "organization"
  aether-domain: "quality"
  aether-owners: "egohygiene"
  aether-created: "2026-08-08"
  aether-updated: "2026-08-08"
---

# Bug Fixing

## Purpose

Execute a bounded defect-correction workflow from reproduction through focused
validation while preserving behavior outside the broken invariant.

## Required Inputs

Resolve as much of the following as possible:

- observed behavior, expected behavior, and failure evidence
- reproduction steps, environment, and scope
- affected code paths, tests, and recent relevant changes
- validation commands and repository constraints
- authorization boundaries, especially when diagnosis-only work was requested

Label missing reproduction evidence instead of masking it.

## Workflow

1. establish the failure and reproduce it when feasible
2. diagnose the root cause without confusing symptoms for the underlying defect
3. choose the smallest safe correction that restores the intended invariant
4. add or update regression protection using:

    - `./references/regression-checklist.md`
    - `./templates/BUG_FIX_REPORT.template.md`

5. validate the focused fix with targeted tests and required broader checks
6. report reproduction evidence, root cause, the exact fix, residual risk, and any checks that could not run

## Constraints

- Do not implement changes during a diagnosis-only request.
- Do not suppress errors, remove safeguards, or weaken tests merely to pass validation.
- Do not broaden the change set into unrelated refactors.
- Do not claim a bug is fixed without evidence from focused validation.
- Do not hide missing reproduction steps or ambiguous evidence.

## Completion Criteria

- [ ] Reproduction or the strongest available proxy signal is explicit.
- [ ] The root cause is distinguished from symptoms.
- [ ] The code change is minimal and behavior-preserving outside the defect.
- [ ] Regression protection covers the broken behavior.
- [ ] Validation results and residual risk are reported truthfully.

## Provenance

This canonical skill is first-party Ego Hygiene content curated from the staged
candidate at `.staging/skills/bug-fixing/SKILL.md`.

## Source Delta

- Adopted: the staged reproduce -> diagnose -> minimal fix -> regression test
  -> validate workflow and the explicit warning against unrelated refactors.
- Rewritten: canonical metadata, regression-focused resources, diagnosis-only
  authorization handling, and deterministic eval coverage.
- Rejected: the narrower `diagnose` overlap as a separate canonical identity for
  this issue; diagnosis remains part of this workflow but must stay separate when
  the request does not authorize implementation.
