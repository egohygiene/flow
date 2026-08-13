---
schema: aether.specification/v1
id: architecture-foundations
title: Foundations Document Specification
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
  - foundations
  - assumptions
  - invariants
applies_to:
  - architecture-documents
depends_on:
  - architecture-document
related:
  - architecture-principles
  - architecture-methodology
  - architecture-system
supersedes: []
---

## Introduction

This specification defines how the `FOUNDATIONS.md` architecture document shall
be authored, maintained, and validated.

`FOUNDATIONS.md` captures the enduring assumptions, invariants, and conceptual
truths that the rest of the architecture is allowed to rely on. It defines what
must remain stably true for the architecture to remain coherent.

Where `PRINCIPLES.md` describes how decisions should be made,
`FOUNDATIONS.md` describes what the architecture assumes to already be true.

---

### Conformance

This specification conforms to:

    .agents/specs/architecture/document.spec.md

Any justified deviations shall be explicitly documented.

---

### 1. Purpose & Scope

The purpose of `FOUNDATIONS.md` is to define the architectural baseline that
other documents inherit.

This document answers the question:

    What enduring truths does the architecture depend on?

This specification covers:

- foundational assumptions
- architectural axioms
- invariants
- baseline conceptual constraints
- enduring mental models
- durable qualities that should survive implementation change

This specification does not cover:

- implementation details
- subsystem structure
- architectural layering
- workflow procedures
- APIs
- deployment details
- project plans
- release sequencing

---

### 2. Definitions

- **Foundation**: A durable assumption or truth that the architecture depends
  upon.
- **Architectural Axiom**: A foundational statement treated as a starting point
  for reasoning.
- **Invariant**: A property that should remain true as the system evolves.
- **Baseline Constraint**: A persistent condition that later architectural work
  must respect.
- **Mental Model**: A conceptual lens used to reason about the system.

---

### 3. Requirements, Constraints & Guidelines

#### Requirements

- **REQ-001**: `FOUNDATIONS.md` shall define the core architectural assumptions
  of the project.
- **REQ-002**: Each foundation shall be stated clearly and unambiguously.
- **REQ-003**: Invariants shall be identified explicitly when applicable.
- **REQ-004**: Foundations shall remain implementation-independent.
- **REQ-005**: Foundations shall support downstream system and architecture
  reasoning.
- **REQ-006**: Foundations shall be stable enough to outlive ordinary
  implementation churn.

#### Constraints

- **CON-001**: Temporary engineering decisions shall not be elevated into
  foundations.
- **CON-002**: Framework, library, or vendor choices shall not be presented as
  foundational truths unless truly essential.
- **CON-003**: Project management concerns shall not appear.
- **CON-004**: Repeated details that belong in `SYSTEM.md` or `ARCHITECTURE.md`
  shall not be duplicated.

#### Guidelines

- **GUD-001**: Prefer a small number of strong, durable foundations.
- **GUD-002**: Prefer conceptual truths over tactical constraints.
- **GUD-003**: Explain why a foundation matters when useful.
- **GUD-004**: Distinguish clearly between a principle and an assumption.
- **GUD-005**: Prefer stability over comprehensiveness.

---

### 4. Authoring Contract

#### Purpose

Define the enduring assumptions and invariants that all downstream
architectural documents may rely upon.

#### Responsibilities

`FOUNDATIONS.md` owns:

- architectural assumptions
- invariants
- enduring truths
- conceptual starting points
- baseline reasoning constraints

#### Non-Responsibilities

`FOUNDATIONS.md` does not own:

- system decomposition
- architectural structure
- implementation planning
- delivery methodology
- roadmap planning
- design-system guidance

#### Inputs

Authoring should consider:

- `PURPOSE.md`
- `VISION.md`
- `PRINCIPLES.md`
- `MANIFESTO.md`
- `PILLARS.md`

#### Outputs

`FOUNDATIONS.md` informs:

- `SYSTEM.md`
- `ARCHITECTURE.md`
- `METHODOLOGY.md`
- `ROADMAP.md`
- downstream engineering decisions
- AI reasoning workflows

#### AI Generation Rules

When generating `FOUNDATIONS.md`, AI systems should:

- identify durable assumptions rather than tactical decisions
- preserve compatibility with upstream identity documents
- avoid implementation details
- express invariants explicitly when they matter
- produce stable, reusable conceptual statements

#### Validation

The resulting document should provide a clear architectural baseline that later
specifications can reference without redefining it.

---

### 5. Acceptance Criteria

- **AC-001**: Core foundations are explicitly stated.
- **AC-002**: Invariants are clearly identified where relevant.
- **AC-003**: The document is implementation-independent.
- **AC-004**: The document does not duplicate system or architecture structure.
- **AC-005**: Foundations are stable, conceptually meaningful, and reusable.

---

### 6. AI Authoring Strategy

AI systems should author `FOUNDATIONS.md` by:

1. Reading the identity and philosophy documents first.
2. Identifying durable assumptions that the project depends on.
3. Distinguishing assumptions from principles and goals.
4. Avoiding tactical or transient implementation details.
5. Producing concise but stable statements suitable for reuse.

---

### 7. Rationale & Context

Without explicit foundations, architecture documents often drift into
inconsistent assumptions, repeated justifications, and contradictory models.

A strong `FOUNDATIONS.md` reduces ambiguity by establishing the conceptual
baseline once and allowing the rest of the architecture to build upon it.

---

### 8. Dependencies

#### Upstream Dependencies

- `PURPOSE.md`
- `VISION.md`
- `PRINCIPLES.md`
- `MANIFESTO.md`
- `PILLARS.md`

#### Downstream Dependencies

- `SYSTEM.md`
- `ARCHITECTURE.md`
- `METHODOLOGY.md`
- `ROADMAP.md`

---

### 9. Examples & Edge Cases

#### Example

A valid foundation might state that the system is local-first, human-centered,
AI-assisted, or modular by design when those assumptions are expected to remain
true over time.

#### Edge Cases

- If an assumption is likely to change frequently, it probably does not belong
  in `FOUNDATIONS.md`.
- If a statement prescribes behavior rather than defines a truth, it may belong
  in `PRINCIPLES.md` instead.
- If a statement describes concrete structure, it may belong in `SYSTEM.md` or
  `ARCHITECTURE.md`.

---

### 10. Validation Criteria

Validation should confirm that:

- the document states enduring assumptions rather than tactics
- invariants are identifiable
- responsibilities are not duplicated with neighboring documents
- the content is conceptually stable
- downstream architecture work can safely rely on it

---

### 11. Related Specifications

Related specifications include:

- `document.spec.md`
- `principles.spec.md`
- `system.spec.md`
- `architecture.spec.md`
- `methodology.spec.md`
- `roadmap.spec.md`
