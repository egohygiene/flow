---
schema: aether.specification/v1
id: architecture-methodology
title: Methodology Document Specification
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
  - methodology
  - workflow
  - engineering-practice
applies_to:
  - architecture-documents
depends_on:
  - architecture-document
related:
  - architecture-principles
  - architecture-foundations
  - architecture-architecture
supersedes: []
---

## Introduction

This specification defines how the `METHODOLOGY.md` architecture document shall
be authored, maintained, and validated.

`METHODOLOGY.md` defines the intentional method by which the project or
organization designs, builds, validates, and evolves work over time.

Where `ARCHITECTURE.md` defines structure, `METHODOLOGY.md` defines the
repeatable way that work should move through that structure.

It captures how the project thinks, not just what it contains.

---

### Conformance

This specification conforms to:

    .agents/specs/architecture/document.spec.md

Any justified deviations shall be explicitly documented.

---

### 1. Purpose & Scope

The purpose of `METHODOLOGY.md` is to document the project's intentional method
for building and evolving work.

This document answers the question:

    How does this project intentionally move from ideas to validated outcomes?

This specification covers:

- working method
- stages of work
- validation loops
- review and refinement patterns
- design-to-implementation flow
- learning and feedback mechanisms
- AI-assisted or human-assisted process design

This specification does not cover:

- detailed issue lists
- sprint plans
- release checklists
- one-off implementation tactics
- low-level coding standards
- deployment configuration

---

### 2. Definitions

- **Methodology**: The repeatable method used to design, build, validate, and
  evolve work.
- **Workflow Stage**: A meaningful phase in the progression of work.
- **Validation Loop**: A recurring mechanism for checking correctness, quality,
  or alignment.
- **Feedback Cycle**: The pattern by which learning informs future work.
- **Operating Model**: The higher-level way a project intentionally functions.

---

### 3. Requirements, Constraints & Guidelines

#### Requirements

- **REQ-001**: `METHODOLOGY.md` shall describe the intentional working method of
  the project.
- **REQ-002**: Major stages or loops in the method shall be made explicit.
- **REQ-003**: Validation and feedback should be represented clearly.
- **REQ-004**: The methodology shall align with the project's principles and
  architectural assumptions.
- **REQ-005**: The methodology shall remain reusable and implementation-light.

#### Constraints

- **CON-001**: Detailed sprint management shall not replace methodology.
- **CON-002**: Temporary tooling habits shall not be mistaken for durable
  method.
- **CON-003**: The document shall not devolve into a task list.
- **CON-004**: Excessive duplication with contribution guides or runbooks shall
  be avoided.

#### Guidelines

- **GUD-001**: Prefer durable patterns over momentary process details.
- **GUD-002**: Keep stages comprehensible and finite.
- **GUD-003**: Make validation explicit.
- **GUD-004**: Document how learning feeds back into the system.
- **GUD-005**: Include AI-assisted workflow behavior when it is architecturally
  important.

---

### 4. Authoring Contract

#### Purpose

Define the repeatable method by which the project creates, validates, and
improves work.

#### Responsibilities

`METHODOLOGY.md` owns:

- working method
- workflow stages
- validation loops
- refinement cycles
- human and AI collaboration patterns
- feedback-oriented evolution

#### Non-Responsibilities

`METHODOLOGY.md` does not own:

- detailed issue queues
- release sequencing
- repository structure
- infrastructure layout
- implementation specifics
- tactical daily planning

#### Inputs

Authoring should consider:

- `PURPOSE.md`
- `VISION.md`
- `PRINCIPLES.md`
- `FOUNDATIONS.md`
- `ARCHITECTURE.md`
- `DECISIONS.md`

#### Outputs

`METHODOLOGY.md` informs:

- implementation planning
- workflow automation
- AI orchestration
- quality and review processes
- roadmap execution patterns

#### AI Generation Rules

When generating `METHODOLOGY.md`, AI systems should:

- identify the durable operating pattern behind the project
- represent stages or loops clearly
- include review, validation, and refinement behavior
- avoid reducing the document to a task plan
- preserve alignment with architectural principles

#### Validation

The resulting document should make the project's working method understandable,
repeatable, and suitable for both human and AI participants.

---

### 5. Acceptance Criteria

- **AC-001**: The working method is clearly defined.
- **AC-002**: Major stages or loops are explicit.
- **AC-003**: Validation and refinement behavior is present.
- **AC-004**: The methodology is durable rather than tactical.
- **AC-005**: The document remains distinct from planning artifacts.

---

### 6. AI Authoring Strategy

AI systems should author `METHODOLOGY.md` by:

1. Reading upstream philosophy and architecture documents.
2. Identifying the real repeatable workflow pattern used by the project.
3. Organizing that pattern into comprehensible stages or loops.
4. Making validation and feedback mechanisms explicit.
5. Producing a reusable process model rather than a temporary plan.

---

### 7. Rationale & Context

Projects often possess an implicit methodology even when none is documented.
When left implicit, process quality depends too heavily on memory and intuition.

`METHODOLOGY.md` externalizes the durable working method so that it can be
shared, reviewed, improved, and reused across people, repositories, and AI
systems.

---

### 8. Dependencies

#### Upstream Dependencies

- `PURPOSE.md`
- `VISION.md`
- `PRINCIPLES.md`
- `FOUNDATIONS.md`
- `ARCHITECTURE.md`
- `DECISIONS.md`

#### Downstream Dependencies

- implementation planning
- workflow automation
- AI operating procedures
- quality processes
- roadmap execution

---

### 9. Examples & Edge Cases

#### Example

A methodology document may describe a loop such as:

- observe or audit reality
- define or refine the specification
- plan implementation
- implement
- validate
- reflect and refactor
- propagate learning

#### Edge Cases

- If the document is mostly a schedule, it belongs closer to `ROADMAP.md` or
  a planning artifact.
- If the content is primarily a contribution guide, it may belong in
  contributor documentation instead.
- If the content describes one repository's local command usage, it is probably
  too tactical for this document.

---

### 10. Validation Criteria

Validation should confirm that:

- the methodology is understandable and repeatable
- validation and feedback are explicit
- the content is durable rather than tactical
- the document is distinct from planning artifacts
- the method supports human and AI collaboration where relevant

---

### 11. Related Specifications

Related specifications include:

- `document.spec.md`
- `architecture.spec.md`
- `decisions.spec.md`
- `roadmap.spec.md`
- `github-issue-authoring.spec.md`
