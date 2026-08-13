---
schema: aether.architecture-document/v1
id: optiflow-ontology
title: Optiflow Ontology
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
  - optiflow-purpose
related:
  - optiflow-system
supersedes: []
---

# Optiflow Ontology

## Domain scope

Local artifact collections, their observed properties and relationships, and
safe plans for normalization or optimization.

## Domain boundaries

Optiflow does not interpret timeline continuity or general transform-graph
semantics. It may request a holon capability through Flow but never owns it.

## Canonical concepts

| Concept | Definition |
| --- | --- |
| Collection root | Authorized filesystem boundary for discovery or action |
| Observation | Timestamped fact about a path or content at inspection time |
| Content identity | Cryptographic identity derived from complete bytes |
| Relationship | Evidence-backed connection such as exact identity or qualified similarity |
| Exact duplicate group | Artifacts with equal length and complete content digest, subject to action-time byte confirmation |
| Opportunity | Quantified possible improvement without an instruction to mutate |
| Plan | Immutable ordered proposal with effects, rationale, and preconditions |
| Precondition | Fact that must be revalidated immediately before an action |
| Action | Typed, bounded mutation with explicit source and destination semantics |
| Recovery record | Evidence needed to understand or reverse an applied action where possible |

## Relationship model

Discovery creates observations. Analysis derives relationships and opportunities
with named methods. Planning chooses proposed actions and preconditions. A
separate application step revalidates current state before any action.

## Ubiquitous language

Use **exact duplicate** only for complete content equality. Use **similar** with
a method and threshold. Use **reclaimable** for potential bytes, not guaranteed
savings. Use **plan** for non-mutating intent and **apply** only for execution.

## Aliases and deprecated terms

Avoid **best copy** without explicit policy; use **selected keeper**. Avoid
**clean up** in machine contracts because its effects are ambiguous.

## Conceptual invariants

Paths can change; plans do not prove current state; no destructive equivalence
claim rests only on names, metadata, thumbnails, or perceptual similarity.

## Open questions

- How should semantic and perceptual relationships coexist with exact identity
  without encouraging unsafe mutation?

## Migration notes

Existing `optiflow.run.v1`, `report.v1`, and `plan.v1` remain explicit contracts
and require versioned adapters to suite types.

## Validation

Schemas distinguish observations, inferences, opportunities, plans, and results.
