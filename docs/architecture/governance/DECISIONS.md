---
schema: aether.architecture-document/v1
id: flow-decisions
title: Flow Decisions
kind: architecture-document
version: 0.1.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-13
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
| [ADR-0001](decisions/ADR-0001-holonic-monorepo.md) | Consolidate the suite as nested holons | Accepted | 2026-08-13 | Separate-repository constraint in the original Flow issue | Source-import plan reveals unacceptable history, release, or toolchain cost |
| [ADR-0002](decisions/ADR-0002-provenance-preserving-derivatives.md) | Preserve source evidence and issue honest derivative claims | Accepted | 2026-08-13 | None | Standards, legal requirements, or supported signing model materially change |
| [ADR-0003](decisions/ADR-0003-docs-first-initialization.md) | Initialize Flow with architecture before source import | Accepted | 2026-08-13 | None | Architecture is accepted and migration planning begins |

## Active decisions

The indexed ADRs are authoritative. Summaries in other documents must link back
to them rather than recreate rationale.

## Deprecated and superseded decisions

The original orchestration issue's prohibition on a monorepo is superseded by
ADR-0001. Its useful constraints—holon independence, typed contracts,
deterministic planning, safe subprocesses, validation, and recovery—remain.

## Historical decisions

None beyond the superseded issue constraint.

## Evidence gaps and open questions

Crate layout, import mechanics, common toolchain, release trains, shared contract
granularity, and library-versus-process adapters remain undecided.
