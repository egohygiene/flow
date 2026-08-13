---
name: test-engineering
description: Designs, implements, repairs, or reviews deterministic automated tests that protect meaningful behavior without chasing superficial coverage. Use when confidence is insufficient and stronger regression or validation evidence is needed.
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

# Test Engineering

## Purpose

Increase confidence with deterministic tests that cover meaningful behavior,
risk, and boundaries without weakening assertions or overfitting to
implementation trivia.

## Required Inputs

Resolve as much of the following as possible:

- behavior or contract at risk
- failure impact and the boundary under test
- existing coverage and why it is insufficient
- repository test conventions, infrastructure, and validation commands
- sources of nondeterminism such as time, randomness, network, or environment

## Workflow

1. model the risk and choose the lowest sufficient test layer
2. design deterministic fixtures, seams, and assertions using:

    - `./references/determinism-checklist.md`
    - `./templates/TEST_PLAN.template.md`

3. assert observable behavior and meaningful state transitions
4. add success, failure, boundary, and recovery coverage proportional to risk
5. write a regression test when addressing a defect
6. validate with the smallest focused command first, then broader required checks
7. report covered behavior, remaining risk, and any required follow-up testability work

## Constraints

- Do not chase coverage percentages by weakening assertions.
- Do not paper over flakiness with sleeps, retries, or disabled tests.
- Do not test implementation trivia when observable behavior suffices.
- Do not duplicate the same assertion across many layers without added value.
- Do not update expected output blindly after a failing test.

## Completion Criteria

- [ ] The chosen test layer is justified by risk and cost.
- [ ] Tests are deterministic and assert meaningful behavior.
- [ ] Important success, failure, or boundary paths are covered proportionally.
- [ ] Validation commands and outcomes are explicit.
- [ ] Remaining risk and follow-up seams are visible.

## Provenance

This canonical skill is first-party Ego Hygiene content curated from the staged
candidate at `.staging/skills/test-engineering/SKILL.md`.

## Source Delta

- Adopted: the staged focus on risk modeling, layer selection, deterministic
  design, and anti-flakiness constraints.
- Rewritten: canonical metadata, deterministic-testing resources, and explicit
  update/regression eval coverage.
- Rejected: the narrower `breakdown-test` and `pytest-coverage` overlaps as
  canonical identities because the core workflow must stay general-purpose and
  behavior-focused rather than framework- or coverage-metric-centric.
