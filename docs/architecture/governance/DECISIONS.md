---
schema: aether.architecture-document/v1
id: flow-decisions
title: Flow Decisions
kind: architecture-document
version: 0.9.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-09-25
governed_by:
  - architecture-decisions
depends_on:
  - flow-principles
  - flow-epistemology
  - flow-foundations
  - flow-system
  - flow-architecture
related:
  - flow-methodology
  - flow-roadmap
supersedes: []
---

# Flow Decisions

## Purpose

This document indexes significant suite decisions and preserves their context,
trade-offs, and review triggers.

## Decision governance

Record decisions that alter capability ownership, dependency direction, public
contracts, safety invariants, provenance policy, workspace/release structure, or
long-lived operational behavior. Routine implementation choices do not require
an ADR.

## Storage mode

Indexed ADR records. Suite decisions live below `governance/decisions/`; holon-
specific decisions live in each holon's `DECISIONS.md` until their volume
justifies separate ADRs.

## Status definitions

- **Proposed:** ready for review but not governing.
- **Accepted:** governs new work.
- **Superseded:** retained for lineage but no longer governs.
- **Deprecated:** discouraged during a transition with a named replacement.

## Decision index

| ID | Title | Status | Accepted | Supersedes | Review trigger |
| --- | --- | --- | --- | --- | --- |
| [ADR-0001](decisions/ADR-0001-holonic-monorepo.md) | Consolidate the suite as nested holons | Superseded | 2026-08-13 | Separate-repository constraint in the original Flow issue | Superseded by ADR-0004 |
| [ADR-0002](decisions/ADR-0002-provenance-preserving-derivatives.md) | Preserve source evidence and issue honest derivative claims | Accepted | 2026-08-13 | None | Standards, legal requirements, or supported signing model materially change |
| [ADR-0003](decisions/ADR-0003-docs-first-initialization.md) | Initialize Flow with architecture before implementation | Accepted | 2026-08-13 | None | Architecture is accepted and adapter planning begins |
| [ADR-0004](decisions/ADR-0004-federated-suite-contracts.md) | Keep holons federated behind versioned contracts | Accepted | 2026-08-21 | ADR-0001 | A required capability cannot be composed through a released interface |
| [ADR-0005](decisions/ADR-0005-federated-extension-authority.md) | Separate extension declarations from operator authority | Accepted | 2026-09-12 | None | A required execution mode cannot preserve the same trust and evidence boundary |
| [ADR-0006](decisions/ADR-0006-bounded-process-transport.md) | Bound process transport with a deterministic JSON Lines transcript | Accepted | 2026-09-16 | None | A released provider or real runner cannot preserve the profile's framing or termination semantics |
| [ADR-0007](decisions/ADR-0007-root-confined-artifact-acceptance.md) | Accept artifacts through root-confined host observations | Accepted | 2026-09-20 | None | A released adapter cannot preserve the root, locator, or deterministic content-identity profile |
| [ADR-0008](decisions/ADR-0008-locked-execution-subjects.md) | Require exact locked package and executable subjects | Accepted | 2026-09-21 | None | Authenticity evidence, launch-time file binding, or transitive runtime identity changes the subject boundary |
| [ADR-0009](decisions/ADR-0009-process-authority-isolation.md) | Bind process authority to explicit isolation evidence | Accepted | 2026-09-21 | None | A real sandbox, authenticated host evidence, or new authority dimension changes the preflight boundary |
| [ADR-0010](decisions/ADR-0010-bounded-direct-process-supervision.md) | Bound direct provider launch and supervision | Accepted | 2026-09-21 | None | Sandbox enforcement, descriptor-bound launch, process-tree containment, or durable interruption changes the runner boundary |
| [ADR-0011](decisions/ADR-0011-durable-run-state.md) | Persist prepared intent and acceptance in immutable local snapshots | Proposed | Pending review | None | Migration, distributed writers, automatic recovery, authenticated state, or stronger durability changes the boundary |

## Active decisions

ADR-0011 is proposed with Flow #49 and remains subject to maintainer review.

The indexed ADRs are authoritative. Summaries in other documents must link back
to them rather than recreate rationale.

## Deprecated and superseded decisions

ADR-0001's monorepo direction is superseded by ADR-0004. Holon independence,
typed contracts, deterministic planning, safe subprocesses, validation, and
recovery remain governing constraints.

## Historical decisions

None beyond the superseded issue constraint.

## Evidence gaps and open questions

Capability-specific library-versus-process adapter selection remains an
implementation decision. Provider compatibility ranges and shared extension
envelope granularity are governed by ADR-0005 and may evolve through additive
contract-set revisions. ADR-0007 closes the portable artifact-binding and
host-observation decision. ADR-0008 closes exact package/executable content
matching while explicitly leaving signatures, publisher authentication,
transparency verification, launch-time file binding, enforced process
isolation, and real process supervision open after ADR-0006's host-neutral
transcript profile. ADR-0009 closes the portable per-invocation authority and
isolation-evidence vocabulary while explicitly leaving operating-system
enforcement and evidence authentication open. ADR-0010 closes the bounded
trusted-unconfined direct-child lifecycle while leaving sandbox enforcement,
authenticated evidence, process-tree containment, descriptor-bound execution,
durable interruption, retry, and resume open.
