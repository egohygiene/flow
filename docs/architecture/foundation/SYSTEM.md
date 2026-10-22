---
schema: aether.architecture-document/v1
id: flow-system
title: Flow System
kind: architecture-document
version: 0.2.1
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-10-22
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
one primary owner. It describes intended ownership; the implementation boundary
below distinguishes delivered library capabilities from the complete suite.

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

- Flow may depend on a released holon's public library API or invoke its
  versioned CLI contract through a Flow-owned adapter.
- Suite interchange schemas are owned and versioned in Flow. Adapters translate
  holon-native contracts at the repository boundary; holons do not need a Flow
  package dependency.
- Holons may not depend on Flow application/orchestration crates.
- Holons may not depend directly on sibling holons.
- Flow may not copy sibling source, use path dependencies into sibling
  checkouts, or depend on mutable branches.
- External tools are systems behind the owning holon's adapter or, for generic
  provenance inspection, a narrowly scoped Flow adapter.

## Major interactions and runtime flows

The intended end-to-end suite flow is:

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

At Flow source `aadf053d48ce8826ee5b9ca4a134985f4e7910e3`, the orchestration
library includes durable prepared execution, verified checkpoints, fresh graph
assessment and explicit recovery. Its native Optiflow adapter consumes the
immutable read-only v0.1.1 CLI for scan, report and review planning. That adapter
retains a separate local receipt; it does not yet participate in the generic
durable graph. Neither surface establishes the complete interaction sequence
above or a supported product CLI.

Released Aniflow/Renderflow adapters, installed-suite workflows and a Flow
release remain separate work. Provider source and package versions do not
establish released compatibility. The [capability matrix](../../integrations/capability-matrix.md)
owns the dated provider evidence; the [current handoff](../../work/11/HANDOFF.md)
owns continuation. Integration must preserve those distinctions.

## Open questions

- Where should generic metadata/provenance inspection live when multiple holons
  need it independently?
- Which orchestration primitives in Aniflow should be generalized versus remain
  temporal-domain behavior?

## Validation

Capability matrices and dependency checks verify ownership and forbidden edges.
