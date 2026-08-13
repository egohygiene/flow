---
name: implementation-planning
description: Converts accepted requirements, architecture, and constraints into an ordered, dependency-aware implementation plan with validation and rollback thinking. Use when execution needs a plan before code changes or when complex work must be decomposed into safe phases.
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

# Implementation Planning

## Purpose

Turn approved requirements and architecture into a plan that can be executed in
small, dependency-aware, verifiable steps.

## Required Inputs

Resolve as much of the following as possible:

- accepted requirements, acceptance criteria, and architecture decisions
- affected components, interfaces, tests, and delivery constraints
- current implementation patterns and repository instructions
- migration, rollout, rollback, and validation expectations
- unresolved questions that block safe sequencing

Stop at material architecture uncertainty instead of planning across it.

## Workflow

1. separate accepted decisions from assumptions and open questions
2. inspect the affected code paths, interfaces, tests, and operational constraints
3. decompose the work into independently verifiable phases using:

    - `./references/phase-design-checklist.md`
    - `./templates/IMPLEMENTATION_PLAN.template.md`

4. trace each requirement and acceptance criterion to one or more plan steps
5. surface migration, compatibility, security, privacy, and documentation work only when the system requires them
6. recommend the smallest safe first executable unit
7. report risks, rollback considerations, and any blocked decisions explicitly

## Constraints

- Do not silently implement production changes during a planning-only request.
- Do not reopen accepted architecture decisions without new authority.
- Do not assign dates, estimates, or ownership without evidence or an explicit estimation method.
- Do not hide unresolved dependencies inside a confident-looking plan.
- Do not force all work into one phase when safer slices exist.

## Completion Criteria

- [ ] The plan traces back to accepted inputs.
- [ ] Dependency order and independently verifiable outcomes are explicit.
- [ ] Risks, rollback concerns, and validation are visible.
- [ ] The first executable unit is safe and concrete.
- [ ] Blocked decisions remain separate from approved work.

## Provenance

This canonical skill is first-party Ego Hygiene content curated from the staged
candidate at `.staging/skills/implementation-planning/SKILL.md`.

## Source Delta

- Adopted: the staged focus on dependency-aware decomposition, requirement
  tracing, and planning without silently changing production code.
- Rewritten: canonical metadata, validation-oriented resources, explicit first
  executable unit guidance, and deterministic eval coverage.
- Rejected: the overlapping `create-implementation-plan` synonym as a canonical
  identity; the core workflow stays provider-neutral and narrower than issue
  generation or execution.
