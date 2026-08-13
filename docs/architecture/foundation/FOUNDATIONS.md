---
schema: aether.architecture-document/v1
id: flow-foundations
title: Flow Foundations
kind: architecture-document
version: 0.1.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-13
governed_by:
  - architecture-foundations
depends_on:
  - flow-purpose
  - flow-vision
  - flow-principles
  - flow-pillars
related:
  - flow-system
  - flow-architecture
supersedes: []
---

# Flow Foundations

## Foundational assumptions

### Content is evidence before it is input

A source artifact may contain meaningful bytes, metadata, streams, manifests,
signatures, and watermarks. Inspection precedes normalization or transformation.

### A holon is whole and composable

Aniflow, Optiflow, and Renderflow each own a coherent domain and remain useful
through a public Rust library and standalone CLI. Flow coordinates them without
making siblings aware of one another.

### Plans and results are different artifacts

A plan describes intended work against resolved capabilities. A result records
observed work and validation. Neither may impersonate the other.

### Artifacts are immutable identities

Content-addressed identity is derived from bytes. Paths are locations, not
identity. A transformation produces a new artifact even when it replaces a
preferred pathname later.

### Validation is layered

Schema validity, process success, artifact existence, decodability, semantic
invariants, and provenance integrity are separate checks.

### External tools are fallible systems

Version, capability, exit status, stdout, stderr, signals, declared artifacts,
and observed artifacts can disagree. Adapters preserve these distinctions.

## Invariants

- Original source bytes are never modified by a processing run.
- Sibling holons have no direct runtime or crate dependency.
- Every capability has one primary architectural owner.
- Run state is written atomically and never reports unvalidated completion.
- Checkpoint reuse requires compatible inputs, configuration, implementation,
  and verified outputs.
- Remote transfer, destructive mutation, and signing are explicit actions.
- A derivative provenance claim never presents itself as the original claim.

## Baseline constraints

The suite is local-first, cross-platform where its external toolchain permits,
safe for paths containing spaces and Unicode, and automation-friendly through
versioned structured contracts. Rust is the primary implementation language,
but external processors remain valid behind narrow adapters.

## Mental models

- **Holarchy:** Flow is a holon composed of smaller holons, not a feature pile.
- **Evidence ledger:** manifests accumulate observations and decisions.
- **Typed graph:** pipelines are validated dependency graphs, not shell scripts.
- **Immutable workshop:** stages consume artifacts and produce new artifacts.

## Falsified or revised foundations

None yet. Revisions require an ADR and explicit downstream review.

## Assumptions and evidence gaps

The final monorepo/workspace mechanics and shared-crate boundaries have not been
validated through migration. Current holons use different Rust editions and
minimum toolchains.

## Open questions

- Can all holons share one workspace toolchain without delaying independent
  releases or forcing unnecessary upgrades?

## Validation

Architecture reviews and contract tests continuously check the invariants.
