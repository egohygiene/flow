---
schema: aether.architecture-document/v1
id: flow-system
title: Flow System
kind: architecture-document
version: 0.1.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-13
governed_by:
  - architecture-system
depends_on:
  - flow-foundations
  - flow-ontology
related:
  - flow-architecture
  - aniflow-system
  - optiflow-system
  - renderflow-system
supersedes: []
---

# Flow System

## Purpose and scope

The Flow suite consists of one orchestration system, three domain holons, and a
small set of shared contracts. This inventory assigns each major capability to
one primary owner.

## System inventory

| System | Primary capabilities | Explicit boundary |
| --- | --- | --- |
| Flow orchestrator | discovery, compatibility, planning, run workspaces, state, progress, cancellation, provenance coordination, validation coordination, recovery, diagnostics | does not implement specialized transformations |
| Aniflow | temporal video inspection, decomposition, ordered frame/audio/video processing, reconstruction, temporal validation, master creation | does not inventory collections or render general documents |
| Optiflow | local collection discovery, evidence-backed identity/relationships, reporting, safe optimization planning, normalization policy | does not reconstruct video timelines or execute general transform DAGs |
| Renderflow | spec-driven transform graph planning/execution, document publication, image/audio conversion, templates, plugins, build caching | does not own collection mutation policy or time-based video pipelines |
| Shared contracts | artifact identity, capability description, tool result, event, provenance evidence, validation result, schema compatibility | contain no domain engine and no orchestration policy |

## Responsibilities and capability ownership

Flow selects and sequences capabilities; holons declare and execute them. Shared
contracts define interchange without choosing which holon runs. Cross-cutting
source inspection is coordinated by Flow but performed through specialized
probes/adapters; it must not become a duplicate media engine.

## System boundaries

- Flow may depend on each holon's public library API and CLI contract.
- Holons may depend on shared contract crates that contain no sibling logic.
- Holons may not depend on Flow application/orchestration crates.
- Holons may not depend directly on sibling holons.
- External tools are systems behind the owning holon's adapter or, for generic
  provenance inspection, a narrowly scoped Flow adapter.

## Major interactions and runtime flows

1. Inspect the source and preserve provenance observations.
2. Resolve a pipeline definition against installed capabilities and policy.
3. Produce and validate a deterministic plan.
4. Execute ready stages while recording events and atomic state.
5. Validate stage artifacts and create compatible checkpoints.
6. Validate selected final derivatives and emit provenance/reporting artifacts.

## External system relationships

Likely external systems include FFmpeg/ffprobe, ExifTool, MediaInfo, GPAC,
`c2patool`/`c2pa-rs`, renderer toolchains, local models, and explicitly selected
remote AI services. Presence in the architecture is not a commitment to ship an
integration; the capability matrix must verify actual contracts first.

## Assumptions and evidence gaps

Aniflow currently has the broadest run orchestration behavior, Optiflow is
deliberately read-only at v0.1.0, and Renderflow currently centers documents
plus image/audio conversion rather than general video derivative packaging.
Migration must preserve reality rather than rename aspiration as capability.

## Open questions

- Where should generic metadata/provenance inspection live when multiple holons
  need it independently?
- Which orchestration primitives in Aniflow should be generalized versus remain
  temporal-domain behavior?

## Validation

Capability matrices and dependency checks verify ownership and forbidden edges.
