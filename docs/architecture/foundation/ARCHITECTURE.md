---
schema: aether.architecture-document/v1
id: flow-architecture
title: Flow Architecture
kind: architecture-document
version: 0.2.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-21
governed_by:
  - architecture-architecture
depends_on:
  - flow-system
related:
  - flow-methodology
  - flow-decisions
supersedes: []
---

# Flow Architecture

## Purpose and scope

This document defines the intended structural organization of the Flow
orchestrator repository and its released integration seams. It fixes dependency
direction and contract ownership while leaving exact Flow crate names to
implementation decisions.

## Structural units

### Suite applications

The `flow` CLI is a thin delivery adapter over an orchestration library. It owns
argument parsing, human presentation, stable exit mapping, and shell completion,
but not planning or execution logic.

### Orchestration core

The core owns pipeline loading, capability resolution, deterministic planning,
run coordination, state transitions, progress, cancellation, recovery, and
diagnostics. It depends only on Flow-owned contracts and adapter ports.

### Domain holons

Aniflow, Optiflow, and Renderflow remain independent repositories and releases.
Their internal modules remain private. A holon exposes only stable domain types,
capability descriptors, and operations justified by its contract.

### Shared contracts

Flow-owned, schema-versioned documents define suite artifact identities,
capability descriptors, compatibility decisions, events, and results. Holons
may keep native models; Flow adapters perform explicit translation rather than
forcing a Flow dependency into a provider repository.

### External adapters

Adapters isolate process invocation, version/capability probing, structured
result parsing, redaction, signal behavior, and output verification. Commands
are constructed as executable plus argv, never as shell strings.

## Boundary rules

- CLI crates depend inward on their corresponding libraries.
- Flow orchestration depends on Flow-owned adapter ports.
- A library adapter may depend on a released public holon library.
- A process adapter may invoke a versioned holon CLI and consume its structured
  output.
- A holon never depends on Flow orchestration or Flow contracts.
- No sibling holon dependency is allowed.
- No copied sibling source, path dependency, Git submodule, or mutable branch
  dependency is allowed.
- Schema types are separated from local persistence models.
- Human console text is never parsed when a supported structured contract
  exists.
- Pipeline definitions select registered typed capabilities, not arbitrary
  commands.

## Dependency direction

```text
flow-cli -> flow-core -> Flow adapter ports -> Flow-owned contracts
                             |                    ^
                             +-> library adapter -+-> released holon library
                             +-> process adapter ----> versioned holon CLI

holon-cli -> holon library -> holon internals

aniflow -X-> optiflow
aniflow -X-> renderflow
optiflow -X-> renderflow
```

Subprocess adapters may replace a library edge where process isolation,
toolchain incompatibility, or release independence warrants it; the logical
direction is unchanged.

## Communication patterns

- Request/response for discovery, planning, and bounded operations.
- Typed event streams for stage progress and diagnostics.
- Atomic manifest snapshots for durable run state.
- Immutable artifacts and content digests for stage handoff.
- Versioned schemas with explicit rejection or migration of unsupported versions.

## Significant constraints

The current repositories use Rust 2021/2024 and minimum versions from 1.85 to
1.94. Release and toolchain independence are architectural constraints. Flow
pins supported provider releases and records compatibility evidence; it does
not require a shared workspace toolchain.

## Relationship to system inventory

`SYSTEM.md` owns capability allocation. This document owns the allowed structural
arrangement that realizes it. Holon architecture files refine their private
structure without overriding suite dependency rules.

## Assumptions and evidence gaps

No Flow implementation exists yet. Proposed units are constraints for adapter
and orchestration work, not claims about current source layout.

## Open questions

- Which public APIs are stable enough for in-process composition in the first
  integrated release?
- How will independent holon versioning map onto suite releases?

## Validation

CI will eventually enforce forbidden dependency edges, CLI-thinness, schema
compatibility, and contract tests for both library and subprocess integration.
