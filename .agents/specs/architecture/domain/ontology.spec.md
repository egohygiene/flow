---
schema: aether.specification/v1
id: architecture-ontology
title: Ontology Architecture Document Specification
kind: specification
version: 2.0.0
status: draft
owners:
  - egohygiene
created: 2026-07-18
updated: 2026-08-02
domain: architecture
tags:
  - architecture
  - domain
  - ontology
  - ubiquitous-language
  - conceptual-model
applies_to:
  - architecture-documents
  - ontology-documents
depends_on:
  - architecture-document
  - architecture-purpose
  - architecture-vision
  - architecture-principles
  - architecture-epistemology
related:
  - architecture-personal-model
  - architecture-system
  - architecture-architecture
  - architecture-design
  - create-ontology-document
supersedes: []
---

# Ontology Architecture Document Specification

## Introduction

This specification defines how `ONTOLOGY.md` shall be authored, maintained, and
validated.

`ONTOLOGY.md` defines the canonical conceptual language of a repository,
product, platform, or organization. It establishes concepts, entities,
relationships, terms, aliases, invariants, and boundaries used throughout the
domain.

The ontology is authoritative for architecture, documentation, schemas, prompts,
agents, and implementation naming, while remaining independent of software
structure.

## 1. Purpose and Scope

`ONTOLOGY.md` answers:

> What exists in this domain, what does each concept mean, and how do the
> concepts relate?

It covers:

- canonical concepts
- entities and value concepts
- conceptual relationships
- canonical terms
- accepted aliases and deprecated terms
- domain boundaries
- conceptual invariants
- concept lifecycle and ownership

It does not cover:

- source-code modules
- API resources
- database tables
- storage formats
- class hierarchies
- UI components
- infrastructure topology
- workflow implementation

## 2. Conceptual Model

A useful ontology contains four connected layers:

    Terms
        The language used consistently

    Concepts
        The meanings represented by that language

    Relationships
        The semantic connections among concepts

    Boundaries
        What belongs inside and outside the domain

A canonical term names a concept. The word and the concept are not identical.
Historical or external terms may remain as aliases when explicitly mapped.

## 3. Responsibilities

`ONTOLOGY.md` owns:

- authoritative concept definitions
- stable concept identifiers
- canonical terms
- relationship semantics
- aliases and deprecated terminology
- domain boundaries
- conceptual invariants
- high-level domain diagrams
- concept-introduction and deprecation rules

## 4. Non-Responsibilities

`ONTOLOGY.md` does not own:

- software architecture
- implementation types
- persistence schemas
- transport formats
- APIs
- process workflows
- interface structure
- repository layout

Implementation artifacts may realize concepts, but they do not redefine them
implicitly.

## 5. Definitions

### Ontology

The canonical conceptual description of a domain.

### Concept

A meaningful abstraction recognized within the domain.

### Entity

A concept whose identity persists while some attributes change.

### Value Concept

A concept primarily distinguished by meaning or attributes.

### Relationship

A semantically defined association between concepts.

### Canonical Term

The preferred term used for a concept.

### Alias

An accepted alternate term mapped to a canonical term.

### Deprecated Term

A historical or discouraged term retained for migration and compatibility.

### Domain Boundary

The distinction between what belongs inside and outside the domain.

### Conceptual Invariant

A condition that must remain true for the conceptual model to remain coherent.

## 6. Requirements

- **REQ-001**: Every major concept shall have one authoritative definition.
- **REQ-002**: Every canonical concept shall have a stable identifier.
- **REQ-003**: Canonical terms shall be unique within their scope.
- **REQ-004**: Relationships among major concepts shall be explicit.
- **REQ-005**: Domain boundaries shall be documented.
- **REQ-006**: Definitions shall remain implementation-independent.
- **REQ-007**: Aliases and deprecated terms shall map to canonical terms.
- **REQ-008**: Ambiguous or overloaded terms shall be disambiguated.
- **REQ-009**: Conceptual invariants shall be documented when they affect meaning.
- **REQ-010**: Concept changes shall preserve provenance and migration history.
- **REQ-011**: Conflicting definitions shall remain visible until resolved.
- **REQ-012**: Examples shall clarify definitions without becoming definitions.
- **REQ-013**: Downstream artifacts shall use canonical terminology.
- **REQ-014**: The ontology shall be understandable without source code.

## 7. Constraints

- **CON-001**: Source-code classes shall not define ontology structure.
- **CON-002**: Database tables shall not automatically become domain entities.
- **CON-003**: API resources shall not redefine canonical concepts.
- **CON-004**: Technology names shall not become domain concepts unless the
  technology is genuinely part of the domain.
- **CON-005**: Synonyms shall not remain undocumented.
- **CON-006**: Similar words shall not cause distinct concepts to be merged.
- **CON-007**: Different implementations shall not automatically create distinct
  concepts.
- **CON-008**: The ontology shall not become a glossary without relationships.
- **CON-009**: Unsupported universal claims shall not be encoded as facts.

## 8. Authoring Contract

### Inputs

Use:

- `PURPOSE.md`
- `VISION.md`
- `PRINCIPLES.md`
- `EPISTEMOLOGY.md`
- domain research
- subject-matter expertise
- existing architecture and documentation
- implementation terminology
- user-facing language
- historical vocabulary
- relevant external standards

### Outputs

Produce:

- domain scope and boundaries
- concept inventory
- canonical term table
- authoritative definitions
- relationship model
- aliases and deprecated terms
- conceptual invariants
- unresolved terminology questions
- optional domain diagram
- migration notes

### Process

1. Define the domain boundary.
2. collect terminology from available evidence.
3. identify concepts independently of implementation.
4. group synonyms and separate homonyms.
5. assign stable identifiers and canonical terms.
6. write concise definitions.
7. define relationships and invariants.
8. preserve aliases and deprecated terms.
9. surface conflicts and unknowns.
10. review downstream naming impact.

### Update Conditions

Update the ontology when a genuine domain concept changes, a boundary changes,
or persistent terminology ambiguity appears.

Do not revise it merely to mirror an implementation refactor.

## 9. AI Authoring Strategy

AI systems shall:

1. read upstream architecture and epistemology
2. inspect terminology across available evidence
3. distinguish concepts from implementation artifacts
4. preserve provenance for externally derived concepts
5. surface conflicting definitions
6. avoid inventing concepts to make the model look complete
7. label uncertain classifications
8. preserve historical vocabulary during renaming
9. test relationship coherence
10. report downstream migration impact

## 10. Dependency Model

Upstream:

- `PURPOSE.md`
- `VISION.md`
- `PRINCIPLES.md`
- `EPISTEMOLOGY.md`

Downstream:

- `PERSONAL_MODEL.md`
- `SYSTEM.md`
- `ARCHITECTURE.md`
- `DESIGN.md`
- schemas
- source-code naming
- documentation
- prompts and agents

A substantive ontology change requires a downstream terminology review.

## 11. Validation

Validate:

- frontmatter and stable identifiers
- unique canonical terms
- complete definitions
- valid relationship endpoints
- explicit domain boundaries
- documented aliases and deprecated terms
- evidence and provenance where required
- implementation independence
- downstream terminology consistency

## 12. Acceptance Criteria

- [ ] Domain scope and boundaries are explicit.
- [ ] Every major concept has a stable identifier.
- [ ] Every major concept has one authoritative definition.
- [ ] Canonical terms are consistent.
- [ ] Aliases and deprecated terms are mapped.
- [ ] Relationships are explicit and semantically defined.
- [ ] Conceptual invariants are present where needed.
- [ ] Conflicts and uncertainty remain visible.
- [ ] Implementation details are absent from canonical definitions.
- [ ] Downstream artifacts can reference concepts without redefining them.
- [ ] Concept changes include migration impact.

## 13. Examples and Edge Cases

### Valid Concept

    Concept ID:
    knowledge-garden

    Canonical term:
    Garden

    Definition:
    A curated collection of interconnected knowledge artifacts.

    Relationship:
    A Garden contains Knowledge Artifacts.

### Table Mistaken for a Concept

    user_events

A storage table is not a domain concept without independent domain evidence.

### Overlapping Concepts

Merge concepts only when their meanings are genuinely identical. Otherwise,
define the distinction and preserve migration history.

## 14. Rationale and Context

A canonical ontology prevents architecture, documentation, implementation, and
AI systems from developing incompatible meanings for the same terms.

## 15. Related Artifacts

- `architecture-document`
- `architecture-epistemology`
- `architecture-personal-model`
- `architecture-system`
- `architecture-architecture`
- `architecture-design`
- `architecture-authoring`
- `create-ontology-document`
