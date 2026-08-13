---
schema: aether.architecture-document/v1
id: renderflow-decisions
title: Renderflow Decisions
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
  - renderflow-system
  - renderflow-architecture
related:
  - flow-decisions
supersedes: []
---

# Renderflow Decisions

## Purpose

Preserve transform-graph, renderer, plugin, cache, and public-API decisions.

## Decision governance

Record changes to graph semantics, optimization policy, executor/renderer
ownership, caching identity, plugin compatibility, AI behavior, supported-format
meaning, or public Renderflow APIs.

## Storage mode

Inline log until migration can reconcile this index with existing repository
architecture decisions.

## Status definitions

Proposed, accepted, deprecated, and superseded follow `flow-decisions`.

## Decision index

| ID | Title | Status | Accepted | Review trigger |
| --- | --- | --- | --- | --- |
| RENDERFLOW-001 | Preserve the core/CLI/plugin-SDK split | Accepted | 2026-08-13 | Import reveals an API or dependency cycle that cannot be resolved within the split |
| RENDERFLOW-002 | Describe current capability truthfully | Accepted | 2026-08-13 | A tested general video-derivative capability is intentionally added |

## Active decisions

### RENDERFLOW-001 — Preserve the workspace boundary

The existing reusable core, thin CLI, and narrow plugin SDK are the target
starting point. Consolidation may rename paths but does not fold the engine into
Flow or move graph logic into the CLI.

### RENDERFLOW-002 — Describe current capability truthfully

Renderflow is currently a spec-driven document rendering engine with image and
audio conversions and a generic transform DAG. Architecture may preserve
extension seams but must not assign thumbnails, video previews, transcripts, or
media packages as existing capabilities until verified implementations land.

## Deprecated and superseded decisions

None at suite initialization. Existing Renderflow ADRs must be imported and
reconciled rather than overwritten.

## Historical decisions

Graph planning, shared intermediates, parallel waves, plugin extensibility, and
core/CLI separation are existing implementation evidence.

## Evidence gaps and open questions

AI transform determinism, plugin isolation, cache compatibility, structured
result coverage, and renderer crate granularity require focused review.
