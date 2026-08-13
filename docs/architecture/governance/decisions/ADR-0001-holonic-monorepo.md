---
schema: aether.architecture-decision/v1
id: adr-0001
title: Consolidate the suite as nested holons
kind: architecture-decision
status: accepted
accepted: 2026-08-13
owners:
  - egohygiene
scope:
  - flow
  - aniflow
  - optiflow
  - renderflow
governed_by:
  - architecture-decisions
supersedes: []
superseded_by: []
related:
  - flow-architecture
---

# ADR-0001 — Consolidate the suite as nested holons

## Context

The initial Flow proposal required separate repositories and CLI-only
orchestration. The project owner subsequently chose to consolidate Flow,
Aniflow, Optiflow, and Renderflow so shared contracts and coordinated evolution
can be developed together.

## Decision

The target is one Flow repository containing independently usable Aniflow,
Optiflow, and Renderflow holons. Each holon exposes a Rust library and CLI, keeps
its domain ownership, and has no direct dependency on a sibling. Flow composes
public holon contracts. Exact import/workspace mechanics require a later ADR.

## Rationale

Colocation reduces cross-repository synchronization cost and makes dependency,
schema, integration, and release validation visible in one change while the
holon rule prevents a monolith.

## Evidence and assumptions

All three projects are Rust-based and already expose meaningful standalone CLI
behavior. Renderflow already demonstrates a core-library/thin-CLI workspace
shape. Rust-version differences and history migration remain untested.

## Alternatives considered

- Separate repositories with subprocess-only adapters: strongest release
  isolation, highest contract coordination cost.
- Git submodules: preserves repositories but adds operational friction and does
  not solve atomic shared-contract changes.
- Absorb all engines into Flow: simplest facade, unacceptable ownership loss.

## Trade-offs

The monorepo simplifies coordination but raises toolchain, CI, release, history,
and ownership complexity. Those costs must be addressed explicitly rather than
by coupling holons.

## Expected consequences

Source import becomes a reviewed migration phase. Old repositories remain
available and should be archived only after replacement releases and links are
verified.

## Observed outcomes

None yet; this PR defines architecture only.

## Review triggers

Review if history-preserving import, independent publishing, CI cost, or minimum
Rust-version alignment proves materially worse than coordinated repositories.

## Related artifacts

`flow-system`, `flow-architecture`, and `flow-roadmap`.

## Validation

Future dependency and packaging tests must prove independent library and CLI use
for every holon.
