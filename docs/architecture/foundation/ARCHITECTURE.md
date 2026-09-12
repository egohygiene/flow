---
schema: aether.architecture-document/v1
id: flow-architecture
title: Flow Architecture
kind: architecture-document
version: 0.3.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-09-12
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

The federated extension boundary separates provider declarations from operator
authority. An extension manifest declares identity, integrity, compatibility,
domain ownership, capabilities, requested permissions, execution modes, hooks,
and checkpoint behavior. A Flow-owned lock records discovery location, the
verified digest, trust mode, granted permissions, capability precedence, and
fallback order. A manifest can never grant itself authority or precedence.

### External adapters

Adapters isolate process invocation, version/capability probing, structured
result parsing, redaction, signal behavior, and output verification. Commands
are constructed as executable plus argv, never as shell strings.

Pinned in-process adapters and bounded process adapters share the same
invocation, event, result, validation, and provenance semantics. Process
adapters additionally enforce declared time, output, cancellation, filesystem,
environment, subprocess, network, AI, GPU, and side-effect limits outside the
provider process.

### Extension lifecycle

Flow coordinates extensions through the ordered lifecycle `discover → inspect →
plan → authorize → execute → validate → commit evidence`. Discovery and
inspection do not execute extension code merely because a binary is available.
Planning resolves compatible capabilities and effects without granting
authority. Execution begins only after the effective lock and requested action
produce an explicit authorization record. Evidence is committed only after
declared outputs and validations have been assessed.

Read-only observers may consume redacted lifecycle events. A content-changing
hook is a declared capability invocation: it consumes immutable artifact
references and produces a new immutable artifact. Hooks cannot rewrite plans,
change authorization, bypass validation, or publish implicitly.

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
- Extension manifests are inert declarations; explicit configuration and a
  digest-pinned lock control discovery and authorization.
- Provider precedence and fallback are operator policy, not self-declared
  authority.
- Cancellation, progress, partial results, diagnostics, validation, checkpoint
  compatibility, and failure classification use versioned envelopes.

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
