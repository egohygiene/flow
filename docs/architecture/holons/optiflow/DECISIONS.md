---
schema: aether.architecture-document/v1
id: optiflow-decisions
title: Optiflow Decisions
kind: architecture-document
version: 0.2.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-21
governed_by:
  - architecture-decisions
depends_on:
  - flow-principles
  - flow-epistemology
  - flow-foundations
  - optiflow-system
  - optiflow-architecture
related:
  - flow-decisions
supersedes: []
---

# Optiflow Decisions

## Purpose

Preserve collection-domain safety and evidence decisions.

## Decision governance

Record changes to relationship proof, traversal policy, state schema,
optimization policy, plan semantics, preconditions, mutation/recovery, or public
Optiflow APIs.

## Storage mode

Inline log until indexed ADRs are warranted.

## Status definitions

Proposed, accepted, deprecated, and superseded follow `flow-decisions`.

## Decision index

| ID | Title | Status | Accepted | Review trigger |
| --- | --- | --- | --- | --- |
| OPTIFLOW-001 | Preserve the v0.1.0 read-only boundary | Accepted | 2026-08-13 | A separately reviewed guarded-apply contract is ready |
| OPTIFLOW-002 | Separate optimization intent from transformation execution | Accepted | 2026-08-13 | A capability cannot be modeled without collapsing the boundary |

## Active decisions

### OPTIFLOW-001 — Preserve the read-only boundary

Flow integration and any provider-owned library extraction do not add mutation.
The first mutation capability requires typed effects, immutable plans, dry runs,
current-state/byte revalidation, containment, interruption semantics, recovery
evidence, and its own review.

### OPTIFLOW-002 — Separate intent from execution

Optiflow may decide that an artifact should be normalized or optimized and may
validate collection effects. It does not absorb temporal or transform-graph
engines. Flow resolves the required public capability to the owning holon.

## Deprecated and superseded decisions

None.

## Historical decisions

The v0.1.0 exact-duplicate rule—equal size and complete BLAKE3 digest, followed
by action-time byte confirmation—remains the baseline evidence policy.

## Evidence gaps and open questions

Quarantine, reversible replacement, hard-linking, transcoding, metadata
normalization, perceptual groups, and keeper policy each require separate threat
models and decisions.
