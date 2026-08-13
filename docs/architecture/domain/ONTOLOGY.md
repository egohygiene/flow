---
schema: aether.architecture-document/v1
id: flow-ontology
title: Flow Ontology
kind: architecture-document
version: 0.1.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-13
governed_by:
  - architecture-ontology
depends_on:
  - flow-purpose
  - flow-vision
  - flow-principles
  - flow-epistemology
related:
  - flow-system
  - aniflow-ontology
  - optiflow-ontology
  - renderflow-ontology
supersedes: []
---

# Flow Ontology

## Domain scope

Flow's domain is the planning, coordination, execution, validation, recovery,
and explanation of content-processing work across compatible holons.

## Domain boundaries

Flow models artifacts and operations at contract level. Specialized frame,
collection, document, codec, model, or renderer concepts belong to the owning
holon's ontology unless they cross the suite boundary.

## Canonical concepts

| Concept | Definition | Key relationships |
| --- | --- | --- |
| Holon | An independently useful system that also participates in a larger system | exposes capabilities and contracts |
| Capability | A versioned operation a holon can truthfully provide | required by a stage; resolved by an adapter |
| Artifact | Immutable content plus typed identity and observed metadata | consumed/produced by stages |
| Source | An artifact selected as an origin for a run | preserved and fingerprinted |
| Derivative | An artifact produced through declared transformations | references source lineage |
| Pipeline definition | Versioned declarative intent and constraints | resolves into a plan |
| Plan | Deterministic resolved graph of stages, tools, inputs, outputs, and validation | executed as a run |
| Stage | Typed unit of planned work with one capability owner | produces events, results, artifacts |
| Run | One execution attempt of one plan in an isolated workspace | owns manifest and checkpoints |
| Checkpoint | Evidence that a stage result remains reusable under a compatibility predicate | may satisfy a stage on resume |
| Manifest | Atomic machine-readable account of run state, tools, artifacts, evidence, and errors | updated through the run |
| Provenance evidence | Preserved information about origin, history, signatures, watermarks, or transformations | attached without overclaiming |
| Validation result | Outcome of an explicit check, including method and evidence | gates completion or reuse |
| Adapter | Boundary translating a public holon contract into suite events and results | never owns domain processing |

## Relationship model

A pipeline definition requires capabilities. Planning resolves those
capabilities to holons and yields stages. A run executes the plan. Stages consume
and produce artifacts, validation results, events, and checkpoints. The manifest
records every relationship and final status.

## Ubiquitous language

Use **inspect** for evidence collection, **plan** for non-expensive resolution,
**process** or a specific capability for execution, **validate** for contract
checks, and **resume** for compatible checkpoint reuse.

## Aliases and deprecated terms

- Prefer **artifact** to ambiguous **file** when identity matters.
- Prefer **derivative** to **clean copy**; the latter obscures transformation.
- Prefer **provenance claim** to **C2PA thing** in contracts.
- Deprecate **watermark-free** unless the claim identifies detector coverage and
  uncertainty.

## Conceptual invariants

Paths do not establish artifact identity. A completed process is not necessarily
a valid stage. A pipeline definition is not executable until capability and
compatibility resolution succeeds.

## Open questions

- Should a provenance observation be a first-class artifact or a typed evidence
  record attached to an artifact?

## Migration notes

Existing Aniflow run manifests, Optiflow reports/plans, and Renderflow build
results require explicit adapters; they are not silently relabeled as Flow
contracts.

## Validation

Schemas and public APIs use canonical terms or document a compatibility alias.
