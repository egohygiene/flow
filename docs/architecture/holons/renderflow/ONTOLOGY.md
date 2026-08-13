---
schema: aether.architecture-document/v1
id: renderflow-ontology
title: Renderflow Ontology
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
  - renderflow-purpose
related:
  - renderflow-system
supersedes: []
---

# Renderflow Ontology

## Domain scope

Declarative multi-target builds expressed as supported transformations among
artifact formats and content representations.

## Domain boundaries

Renderflow models transform paths and build semantics, not collection-wide
optimization intent or temporal continuity across decomposed video streams.

## Canonical concepts

| Concept | Definition |
| --- | --- |
| Build specification | Validated declaration of source, targets, transforms, variables, templates, and policy |
| Format | Typed representation with declared detection and validation semantics |
| Transform | Operation from one representation to another with requirements, cost, quality, and execution semantics |
| Transform graph | Directed graph of registered formats and supported transformations |
| Path | Ordered transform sequence from one source format to one target |
| Build plan | Merged acyclic execution graph for all requested targets |
| Execution wave | Independent plan nodes safe to run concurrently |
| Intermediate | Immutable reusable artifact produced within a build path |
| Renderer | Adapter that materializes a target through an external or native engine |
| Plugin | Extension implementing a versioned public execution contract |
| Build result | Structured record of plan, execution, outputs, cache use, warnings, and validation |

## Relationship model

A build specification requests targets. Registered transforms create a graph;
planning selects paths and merges shared intermediates into a build plan.
Execution waves produce validated intermediates and outputs recorded in a result.

## Ubiquitous language

Use **build** for executing a specification, **render** for materializing a
publication representation, **convert** for format change, and **transform** for
the general typed edge. A pipeline is a selected path, not the whole suite run.

## Aliases and deprecated terms

Avoid **optimal** without naming the optimization objective. Prefer **selected
path under speed/quality/balanced policy**. Avoid **supported format** unless at
least one validated path exists for the requested direction.

## Conceptual invariants

Plans are acyclic, format direction matters, every edge has one executor,
intermediate reuse preserves content identity semantics, and cached nodes are
compatible with all relevant inputs and implementation versions.

## Open questions

- How should non-deterministic AI transforms declare reproducibility and cache
  identity?

## Migration notes

Existing configuration and public Rust APIs require compatibility mapping; they
are not silently replaced by suite pipeline definitions.

## Validation

Format registry, planner, plugins, schemas, and user documentation share this
canonical language.
