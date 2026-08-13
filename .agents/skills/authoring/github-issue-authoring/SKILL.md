---
name: github-issue-authoring
description: Converts evidence, specifications, audits, bug reports, and rough notes into scoped, copy-ready GitHub issues or dependency-aware issue batches. Use when defining implementation work clearly without performing the implementation itself.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "experimental"
  aether-scope: "organization"
  aether-domain: "authoring"
  aether-owners: "egohygiene"
  aether-created: "2026-08-08"
  aether-updated: "2026-08-08"
---

# GitHub Issue Authoring

## Purpose

Transform repository evidence into a clear GitHub issue that another engineer or
agent can execute without guessing scope, constraints, or validation.

## Required Inputs

Resolve as much of the following as possible:

- problem statement, desired outcome, and user motivation
- relevant specifications, audits, notes, or defect evidence
- affected systems, constraints, exclusions, and dependencies
- repository conventions, templates, labels, and validation commands
- unresolved questions or decisions that must stay visible

When repository evidence is incomplete, label assumptions explicitly.

## Workflow

1. extract the durable problem, goal, boundaries, and observable outcome
2. inspect repository evidence before prescribing paths, commands, or ownership
3. choose whether the result should be one issue or an ordered batch of issues
4. write copy-ready issue content using the focused resources:

    - `./references/copy-ready-checklist.md`
    - `./templates/GITHUB_ISSUE.template.md`

5. make acceptance criteria observable and keep non-goals explicit
6. preserve the user's copy-and-paste formatting preference; when fenced blocks would break copying or rendering, use four-space-indented inner code examples instead
7. validate title clarity, internal consistency, dependency order, and execution readiness

## Constraints

- Do not implement the requested work inside the issue-authoring step.
- Do not invent repository paths, commands, labels, or ownership.
- Do not mix multiple independently shippable outcomes into one issue without stating why.
- Do not hide unresolved decisions or missing evidence.
- Do not embed provider-specific tooling instructions into the core workflow.

## Completion Criteria

- [ ] The issue can be executed without reopening accepted decisions.
- [ ] Scope, non-goals, and dependencies are explicit.
- [ ] Validation steps are observable and repository-aware.
- [ ] Copy-ready formatting is preserved.
- [ ] Missing evidence and open questions remain visible.

## Provenance

This canonical skill is first-party Ego Hygiene content curated from the staged
candidate at `.staging/skills/github-issue-authoring/SKILL.md`.

## Source Delta

- Adopted: the staged workflow for extracting durable intent, inspecting the
  repository before prescribing work, and validating issue completeness.
- Rewritten: canonical metadata, copy-ready template, deterministic evals, and
  the explicit formatting rule for four-space-indented inner code examples when
  fenced blocks would copy poorly.
- Rejected: the placeholder `github-issue` staging copy as a synonym identity;
  broader or specialized legacy variants such as `github-issues` and
  `create-github-issue-*` were reviewed as overlap but not adopted into the core
  provider-neutral workflow.
