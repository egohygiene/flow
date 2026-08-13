---
schema: aether.architecture-document/v1
id: aniflow-ontology
title: Aniflow Ontology
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
  - flow-ontology
  - aniflow-purpose
related:
  - aniflow-system
supersedes: []
---

# Aniflow Ontology

## Domain scope

Ordered media in time and the transformations required to produce a validated
video master.

## Domain boundaries

Collection membership, duplicate relationships, general publication formats,
and cross-holon run state remain outside this ontology.

## Canonical concepts

| Concept | Definition |
| --- | --- |
| Temporal source | Video artifact plus observed streams and timing |
| Timeline | Ordered temporal model relating frames, samples, streams, and duration |
| Decomposition | Lossless or declared extraction of temporal components |
| Processor chain | Ordered, typed operations over frames, audio, subtitles, or whole video |
| Frame set | Ordered immutable images with naming, count, dimensions, and timing relation |
| Continuity invariant | Required relationship among ordering, dimensions, timing, and expected components |
| Reconstruction | Assembly of processed components into an encoded video artifact |
| Master | Selected validated reconstruction suitable for downstream use |
| Temporal checkpoint | Reusable stage evidence bound to timeline, inputs, processor configuration, version, and outputs |

## Relationship model

A temporal source yields a timeline and decomposition. Processor chains consume
decomposed components and produce new immutable components. Reconstruction joins
selected components; validation determines whether it may be called a master.

## Ubiquitous language

Use **extract** for decomposition, **process** for typed transformation,
**assemble** for reconstruction, and **master** only after validation.

## Aliases and deprecated terms

Avoid **frames** as shorthand for an entire video and avoid **completed** when a
stage has not passed continuity and output validation.

## Conceptual invariants

Ordering is explicit; processor outputs do not overwrite inputs; timing intent
survives decomposition; a master's stream behavior is declared and validated.

## Open questions

- Is timeline normalization a planning concept, a stage, or both?

## Migration notes

Existing pipeline v2 and delivery manifests require mapping to suite contracts;
their current identifiers remain valid until an explicit migration exists.

## Validation

Public types and documentation use these terms consistently.
