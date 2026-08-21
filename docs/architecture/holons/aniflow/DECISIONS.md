---
schema: aether.architecture-document/v1
id: aniflow-decisions
title: Aniflow Decisions
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
  - aniflow-system
  - aniflow-architecture
related:
  - flow-decisions
supersedes: []
---

# Aniflow Decisions

## Purpose

Preserve temporal-domain decisions that do not govern the whole suite.

## Decision governance

Record changes to the timeline model, stream policy, interchange formats,
processor contract, reconstruction invariants, checkpoint compatibility, or
public Aniflow API.

## Storage mode

Inline log until multiple active decisions require indexed ADRs.

## Status definitions

Proposed, accepted, deprecated, and superseded follow `flow-decisions`.

## Decision index

| ID | Title | Status | Accepted | Review trigger |
| --- | --- | --- | --- | --- |
| ANIFLOW-001 | Own temporal processing; remove cross-holon handoff ownership | Accepted | 2026-08-13 | Flow cannot compose a required downstream capability without domain leakage |
| ANIFLOW-002 | Preserve current limitations as constraints, not foundations | Accepted | 2026-08-13 | A constraint is intentionally promoted into a public compatibility promise |

## Active decisions

### ANIFLOW-001 — Own temporal processing

Aniflow ends at a validated temporal master and domain evidence. Cross-holon
selection and handoff belong to Flow. This supersedes the architectural meaning
of the current optional Renderflow CLI handoff while preserving compatibility
inside Aniflow until its owning repository changes the public contract.

### ANIFLOW-002 — Constraints are not foundations

PNG interchange, first-stream processing, CFR targeting, AAC output, and
run-local caching describe v0.2.0. They must remain documented and tested during
provider evolution but do not define the target domain model.

## Deprecated and superseded decisions

None removed yet; the optional Renderflow handoff remains provider-owned and is
excluded from Flow's cross-holon architecture rather than silently deleted.

## Historical decisions

Current repository behavior remains evidence in the capability matrix.

## Evidence gaps and open questions

The target timeline representation, multi-stream policy, and crate split require
focused ADRs after fixture-based discovery.
