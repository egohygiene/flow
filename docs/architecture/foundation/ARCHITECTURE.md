---
schema: aether.architecture-document/v1
id: flow-architecture
title: Flow Architecture
kind: architecture-document
version: 0.1.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-13
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

This document defines the intended structural organization of the future Flow
Rust workspace. It fixes dependency direction and contract seams while leaving
exact crate names and source-import mechanics to migration decisions.

## Structural units

### Suite applications

The `flow` CLI is a thin delivery adapter over an orchestration library. It owns
argument parsing, human presentation, stable exit mapping, and shell completion,
but not planning or execution logic.

### Orchestration core

The core owns pipeline loading, capability resolution, deterministic planning,
run coordination, state transitions, progress, cancellation, recovery, and
diagnostics. It depends only on shared contracts and public holon interfaces.

### Domain holons

Aniflow, Optiflow, and Renderflow each use a core-library plus thin-CLI shape.
Their internal modules remain private. A holon exposes only stable domain types,
capability descriptors, and operations justified by its contract.

### Shared contracts

Small, low-dependency crates own schema-versioned artifact identities, events,
validation results, capability descriptors, and common error categories. They
must not accumulate orchestration policy or domain algorithms.

### External adapters

Adapters isolate process invocation, version/capability probing, structured
result parsing, redaction, signal behavior, and output verification. Commands
are constructed as executable plus argv, never as shell strings.

## Boundary rules

- CLI crates depend inward on their corresponding libraries.
- Flow orchestration may depend on public holon libraries and contracts.
- A holon may depend on shared contracts, never on Flow orchestration.
- No sibling holon dependency is allowed.
- Schema types are separated from local persistence models.
- Human console text is never parsed when a supported structured contract
  exists.
- Pipeline definitions select registered typed capabilities, not arbitrary
  commands.

## Dependency direction

```text
flow-cli -> flow-core -> holon public APIs -> holon internals
                      -> shared contracts <- holon public APIs

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
1.94. One workspace toolchain may be possible but is not assumed. Repository
history and release identity must be preserved during consolidation. The first
import may use subtree-style history, archived mirrors, or another reviewed
method.

## Relationship to system inventory

`SYSTEM.md` owns capability allocation. This document owns the allowed structural
arrangement that realizes it. Holon architecture files refine their private
structure without overriding suite dependency rules.

## Assumptions and evidence gaps

No Flow implementation exists yet. Proposed units are constraints for the
migration and planning phase, not claims about current source layout.

## Open questions

- One Cargo workspace or coordinated nested workspaces?
- Which public APIs are stable enough for in-process composition in the first
  integrated release?
- How will independent holon versioning map onto suite releases?

## Validation

CI will eventually enforce forbidden dependency edges, CLI-thinness, schema
compatibility, and contract tests for both library and subprocess integration.
