---
schema: aether.architecture-document/v1
id: flow-architecture
title: Flow Architecture
kind: architecture-document
version: 0.6.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-09-20
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
capability descriptors, compatibility decisions, events, results, and
orchestration conformance scenarios. Holons may keep native models; Flow
adapters perform explicit translation rather than forcing a Flow dependency
into a provider repository.

The federated extension boundary separates provider declarations from operator
authority. An extension manifest declares identity, integrity, compatibility,
domain ownership, capabilities, requested permissions, execution modes, hooks,
and checkpoint behavior. A Flow-owned lock records discovery location, the
verified digest, trust mode, granted permissions, capability precedence, and
fallback order. A manifest can never grant itself authority or precedence.

The scenario manifest pins redistribution-safe inputs and providers, ordered
topology, typed expectations, resource budgets, provenance, and bounded
coverage claims. It references other versioned documents by schema and digest.
It does not duplicate provider algorithms or define runtime plans, durable runs,
checkpoints, or transition storage.

Artifact bindings remain separate from path-independent extension envelopes.
A Flow-owned binding set maps immutable input and candidate-output identities to
logical ports, media types, kinds, and portable root-relative locators for one
run. A Flow observer computes file or recursive directory identity beneath one
caller-selected root. Only the separate artifact-acceptance gate can correlate
those observations with resolved capability, invocation, event, and terminal
result evidence.

### External adapters

Adapters isolate process invocation, version/capability probing, structured
result parsing, redaction, signal behavior, and output verification. Commands
are constructed as executable plus argv, never as shell strings.

Pinned in-process adapters and bounded process adapters share the same
invocation, event, result, validation, and provenance semantics. Process
adapters must additionally enforce every time, output, cancellation,
filesystem, environment, subprocess, network, AI, GPU, and side-effect
guarantee they claim outside the provider process. A declaration, grant,
transcript check, or adapter policy is not by itself sandbox enforcement.

The first process boundary is host-neutral: Flow deterministically encodes one
versioned invocation and validates a caller-supplied JSON Lines stdout
transcript, bounded stderr length, and completion observation. That seam reuses
the same Flow-owned event and result validation as in-process execution. It does
not spawn, signal, time out, cancel, reap, inspect files from, or isolate a child
process.

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

## Current implementation boundary

Flow issue #23 / merged PR #24 supplies closed extension-v1 models,
deterministic single-capability resolution, one caller-injected in-process
extension port, Flow-owned event/result validation, and a hermetic no-effects
reference implementation. Issue #26 / merged PR #27 adds deterministic process
request framing and host-neutral transcript validation through that same
acceptance gate. Issue #28 adds a closed scenario manifest, synthetic fixtures,
and canonical digest drift checks; it does not run those scenarios. Issue #36
adds explicit artifact bindings, root-confined host observation, deterministic
file/directory identity, and an opaque accepted artifact set. Flow does not yet
supply the public CLI, a production child-process runner, real holon adapters,
executable verification, provider-native artifact validation, enforceable
isolation, durable run state, checkpoints, interruption, or resume. Structural
units beyond these library seams remain constraints for later adapter and
orchestration work, not claims about current source layout.

## Open questions

- Which public APIs are stable enough for in-process composition in the first
  integrated release?
- How will independent holon versioning map onto suite releases?

## Validation

Default-branch CI run 35510751421 validates Rust 1.85 and stable library builds,
closed contract models, deterministic resolution, the hermetic in-process and
process-transcript seams, scenario-manifest identity, and repository
architecture metadata at `aac62ab80c18c936e3a7ebd5f4e70b7845f39faa`.
Issue #36 extends that matrix with artifact schema, semantic, filesystem, and
adversarial correlation checks. Forbidden dependency-edge checks,
CLI-thinness, real launcher enforcement, provider-native artifact validation,
scenario execution, and sandbox conformance remain later gates for their
corresponding runtime surfaces.
