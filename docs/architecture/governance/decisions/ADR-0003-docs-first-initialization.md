---
schema: aether.architecture-decision/v1
id: adr-0003
title: Initialize Flow with architecture before source import
kind: architecture-decision
status: accepted
accepted: 2026-08-13
owners:
  - egohygiene
scope:
  - flow
governed_by:
  - architecture-decisions
supersedes: []
superseded_by: []
related:
  - flow-roadmap
---

# ADR-0003 — Initialize Flow with architecture before source import

## Context

The three existing tools overlap in orchestration vocabulary and have different
levels of maturity, workspace structure, Rust versions, schemas, and release
automation. Moving source before defining ownership would make accidental layout
look like intentional architecture.

## Decision

The first Flow pull request contains the repository-local Aether framework,
suite architecture, focused holon contracts, decision records, and root roadmap.
It imports no tool source and creates no Cargo workspace.

## Rationale

A documentation-only change makes boundaries reviewable before history,
packaging, contract, and code decisions constrain them.

## Evidence and assumptions

Aniflow is a single crate at v0.2.0, Optiflow is a single package at v0.1.0,
and Renderflow is a multi-crate workspace at v0.2.1. Current behavior was
inspected at named default-branch revisions on 2026-08-13.

## Alternatives considered

- Import everything immediately: faster initial motion, poor boundary review.
- Define architecture separately in each old repository: duplicates suite policy
  and increases drift during consolidation.

## Trade-offs

No executable progress lands in the first change. Some structural decisions are
deliberately deferred.

## Expected consequences

The next phase creates a migration plan and capability matrix before source is
moved.

## Observed outcomes

The architecture set exposes several genuine open questions instead of hiding
them in a premature workspace layout.

## Review triggers

This decision has served its purpose once architecture is accepted and migration
planning begins; it does not prohibit later implementation PRs.

## Related artifacts

`flow-meta` and `flow-roadmap`.

## Validation

The initializing PR contains no Cargo manifest or imported application source.
