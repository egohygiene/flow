---
name: repository-cleanup
description: Performs behavior-preserving repository hygiene, consistency, and low-risk cleanup with explicit classification of safe, unsafe, and uncertain changes. Use when reducing clutter or inconsistency without redesigning the system.
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

# Repository Cleanup

## Purpose

Perform bounded cleanup that improves consistency and hygiene while preserving
intended behavior, ownership boundaries, and recoverability.

## Required Inputs

Resolve as much of the following as possible:

- authorized cleanup scope and behavior-preservation invariants
- repository status, ignore rules, formatters, linters, and generators
- ownership expectations for files, configuration, and generated artifacts
- validation commands needed to confirm no behavioral drift
- uncertain files or changes that require separate authorization

## Workflow

1. state the exact invariant that must remain unchanged
2. classify candidate changes before editing using:

    - `./references/classification-checklist.md`
    - `./templates/CLEANUP_PLAN.template.md`

3. edit only safe mechanical or safe documentation/configuration corrections
4. keep dependency upgrades, architectural redesign, and uncertain deletions out of routine cleanup scope
5. validate formatting, links, tests, or configuration behavior proportional to the edits
6. review the final diff for accidental semantic change, secrets, generated files, and unrelated churn

## Constraints

- Do not use cleanup as cover for redesign or feature work.
- Do not delete files when ownership, references, or recovery are uncertain.
- Do not change public behavior silently.
- Do not mix broad dependency updates into hygiene-only work.
- Do not hide uncertain candidates that were intentionally left unchanged.

## Completion Criteria

- [ ] Cleanup scope and preserved behavior are explicit.
- [ ] Each candidate change was classified before editing.
- [ ] Only safe categories were changed.
- [ ] Validation evidence supports behavior preservation.
- [ ] Uncertain or blocked cleanup candidates remain visible.

## Provenance

This canonical skill is first-party Ego Hygiene content curated from the staged
candidate at `.staging/skills/repository-cleanup/SKILL.md`.

## Source Delta

- Adopted: the staged invariant-first workflow, pre-edit classification, and
  emphasis on behavior-preserving hygiene.
- Rewritten: canonical metadata, explicit cleanup planning resources, and
  deterministic eval coverage.
- Rejected: `repository-audit` as a synonym identity because auditing and
  cleanup own different authorization boundaries; audit findings may feed
  cleanup work, but the workflows remain distinct.
