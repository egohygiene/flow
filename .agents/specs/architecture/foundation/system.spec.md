---
schema: aether.specification/v1
id: architecture-system
title: System Document Specification
kind: specification
version: 1.1.0
status: draft
owners:
  - egohygiene
created: 2026-07-18
updated: 2026-08-02
domain: architecture
tags:
  - architecture
  - specification
  - systems
  - decomposition
  - boundaries
applies_to:
  - architecture-documents
depends_on:
  - architecture-document
  - architecture-ontology
related:
  - architecture-architecture
  - architecture-foundations
supersedes: []
---

## Introduction

This specification defines how the `SYSTEM.md` architecture document shall be
authored, maintained, and validated.

`SYSTEM.md` defines the major systems that make up the project, along with the
responsibilities, boundaries, and primary capabilities owned by each one.

Where `ONTOLOGY.md` describes what exists in the domain,
`SYSTEM.md` describes what major systems exist to fulfill the project's
purpose.

Where `ARCHITECTURE.md` later explains how those systems are organized and
connected, `SYSTEM.md` identifies the systems themselves.

---

### Conformance

This specification conforms to:

    .agents/specs/architecture/document.spec.md

Any justified deviations shall be explicitly documented.

---

### 1. Purpose & Scope

The purpose of `SYSTEM.md` is to provide a stable conceptual decomposition of
the project into major systems.

This document answers the question:

    What systems make up this project, and what does each one own?

This specification covers:

- system inventory
- system purpose
- capability ownership
- conceptual boundaries
- primary responsibilities
- major interactions at a high level
- external system relationships when relevant

This specification does not cover:

- internal module breakdown
- implementation details
- source layout
- APIs
- deployment topology
- infrastructure configuration
- sprint or project planning

---

### 2. Definitions

- **System**: A cohesive set of capabilities with a distinct responsibility.
- **Capability**: A high-level function provided by a system.
- **System Boundary**: The conceptual line separating what a system owns from
  what it does not own.
- **Capability Ownership**: The assignment of a capability to one primary
  system.
- **System Context**: The relationship between a system and adjacent internal
  or external systems.

---

### 3. Requirements, Constraints & Guidelines

#### Requirements

- **REQ-001**: `SYSTEM.md` shall identify the project's major systems.
- **REQ-002**: Each system shall have a clearly stated purpose.
- **REQ-003**: Each system shall have explicitly defined responsibilities.
- **REQ-004**: System boundaries shall be clear enough to reduce ambiguity.
- **REQ-005**: Major capabilities shall be assigned to a primary owning system.
- **REQ-006**: The system model shall remain implementation-independent.
- **REQ-007**: The system decomposition shall align with the project's purpose,
  vision, and foundations.

#### Constraints

- **CON-001**: Source code or package layout shall not be used as a substitute
  for conceptual system definition.
- **CON-002**: Framework-specific or vendor-specific abstractions shall not
  define system boundaries.
- **CON-003**: Temporary repository organization shall not dictate the system
  model.
- **CON-004**: Detailed architectural layering shall be deferred to
  `ARCHITECTURE.md`.

#### Guidelines

- **GUD-001**: Prefer conceptual decomposition over technical decomposition.
- **GUD-002**: Prefer clear ownership over perfect granularity.
- **GUD-003**: Minimize overlapping responsibilities.
- **GUD-004**: Keep the system inventory small enough to be understood.
- **GUD-005**: Treat systems as durable architectural units, not ephemeral
  features.

---

### 4. Authoring Contract

#### Purpose

Define the major systems of the project and clarify what responsibilities and
capabilities each one owns.

#### Responsibilities

`SYSTEM.md` owns:

- system identification
- system purpose
- responsibility allocation
- capability ownership
- major conceptual boundaries
- high-level system context

#### Non-Responsibilities

`SYSTEM.md` does not own:

- module-level structure
- dependency graphs between internal modules
- implementation details
- source-tree design
- infrastructure layout
- roadmap sequencing

#### Inputs

Authoring should consider:

- `PURPOSE.md`
- `VISION.md`
- `FOUNDATIONS.md`
- `ONTOLOGY.md`
- `PERSONAL_MODEL.md`

#### Outputs

`SYSTEM.md` informs:

- `ARCHITECTURE.md`
- repository and component decomposition
- implementation planning
- technical documentation
- AI reasoning and orchestration

#### AI Generation Rules

When generating `SYSTEM.md`, AI systems should:

- identify major capabilities before naming systems
- group related capabilities into cohesive systems
- assign one primary owner for each capability
- define system boundaries before describing interactions
- avoid implementation detail and source-structure bias

#### Validation

The resulting document should allow a reader to understand the project's major
systems and their ownership boundaries without reading code.

---

### 5. Acceptance Criteria

- **AC-001**: Every major system is identified.
- **AC-002**: Every system has a clearly defined purpose.
- **AC-003**: Responsibilities and capabilities are assigned explicitly.
- **AC-004**: Boundaries between systems are understandable.
- **AC-005**: The document remains implementation-independent.

---

### 6. AI Authoring Strategy

AI systems should author `SYSTEM.md` by:

1. Reading upstream identity, foundation, and domain documents.
2. Identifying the major capability areas implied by those documents.
3. Grouping capabilities into cohesive systems.
4. Assigning responsibilities and boundaries to each system.
5. Describing the resulting decomposition in a stable, conceptual form.

---

### 7. Rationale & Context

Without a system model, projects often collapse directly from high-level vision
into implementation detail. This leads to ambiguous ownership, duplicated
capabilities, and unclear boundaries.

`SYSTEM.md` provides the bridge between domain understanding and structural
architecture by naming the major systems first.

---

### 8. Dependencies

#### Upstream Dependencies

- `PURPOSE.md`
- `VISION.md`
- `FOUNDATIONS.md`
- `ONTOLOGY.md`
- `PERSONAL_MODEL.md`

#### Downstream Dependencies

- `ARCHITECTURE.md`
- component decomposition
- implementation planning
- technical design work

---

### 9. Examples & Edge Cases

#### Example

A project might define separate systems for product experience, artifact
processing, orchestration, observability, or publication when those represent
meaningful responsibility boundaries.

#### Edge Cases

- A repository is not automatically a system.
- A feature is not automatically a system.
- If a unit is too small and tactical, it likely belongs inside
  `ARCHITECTURE.md` instead.
- If a unit is primarily an external dependency, it should be described as an
  external system rather than a core internal system.

---

### 10. Validation Criteria

Validation should confirm that:

- major systems are explicit and finite
- responsibilities do not significantly overlap
- capability ownership is understandable
- the decomposition is conceptual rather than implementation-driven
- the document supports downstream architecture work

---

### 11. Related Specifications

Related specifications include:

- `document.spec.md`
- `foundations.spec.md`
- `architecture.spec.md`
- `ontology.spec.md`
- `personal-model.spec.md`
