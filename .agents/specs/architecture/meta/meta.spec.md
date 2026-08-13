---
schema: aether.specification/v1
id: architecture-meta
title: Meta Architecture Document Specification
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
  - meta
  - framework
  - catalog
  - governance
  - navigation
applies_to:
  - architecture-documents
  - meta-documents
depends_on:
  - architecture-document
  - architecture-epistemology
  - architecture-ai-constitution
related:
  - architecture-decisions
  - architecture-authoring
  - create-meta-architecture-document
supersedes: []
---

# Meta Architecture Document Specification

## Introduction

This specification defines how `META.md` shall be authored, maintained, and
validated.

`META.md` describes the architecture-document system itself. It maps the
applicable documents, categories, ownership boundaries, relationships, reading
order, authoring order, lifecycle, and change-propagation rules for a specific
repository, product, platform, or organization.

It is a navigation and governance map, not a duplicate summary of every
architecture document.

## 1. Purpose and scope

`META.md` answers:

> How is this architecture-document system organized, navigated, validated, and
> evolved?

This specification covers:

- architecture-document inventory
- category and layer model
- canonical ownership
- dependency and relationship graph
- reading and authoring order
- lifecycle and status
- change propagation
- validation state
- missing or intentionally omitted documents
- architecture evolution

It does not cover:

- product or system architecture itself
- implementation details
- methodology
- roadmap
- individual document content
- accepted architecture decisions
- general repository documentation

## 2. Conceptual model

`META.md` is a repository-specific view over the architecture catalog.

    reusable aether specifications
        ↓ materialized into
    repository architecture documents
        ↓ indexed by
    META.md
        ↓ consumed by
    humans, agents, validation, pace, and observatory

The authoritative relationship graph should be derived from artifact metadata
where possible. `META.md` explains the graph in human-readable form.

## 3. Responsibilities

`META.md` owns:

- the inventory of applicable architecture documents
- categories and architectural layers
- each document's primary concern
- dependency and relationship overview
- recommended reading order
- recommended authoring order
- lifecycle and status overview
- change-propagation guidance
- documented omissions and gaps
- onboarding into the architecture system

## 4. Non-responsibilities

`META.md` does not own:

- purpose, vision, principles, pillars, or manifesto content
- domain ontology
- system or implementation architecture
- design-system details
- individual decisions
- detailed validation rules owned by specifications
- generated catalog data as a competing source of truth

## 5. Definitions

### Architecture system

The set of architecture documents, specifications, relationships, and
governance rules applicable to a scope.

### Category

A navigational grouping such as identity, meta, domain, foundation, experience,
or governance.

### Canonical owner

The document that authoritatively owns a concern.

### Reading order

The sequence most useful for understanding the architecture.

### Authoring order

The dependency-based sequence for creating or materially revising documents.

### Change propagation

The review and update process triggered by a substantive upstream change.

### Omission

A document intentionally not used because it is unnecessary for the scope.

## 6. Required content model

The document should contain:

### Architecture overview

A concise explanation of the document system and its scope.

### Document inventory

For each applicable document:

- artifact identifier
- path
- category
- status
- primary concern
- governing specification
- upstream dependencies
- downstream consumers

### Relationship graph

A human-readable representation of dependency and related-artifact edges.

### Ownership map

A concise mapping of concerns to canonical documents.

### Reading order

A recommended sequence for contributors and agents.

### Authoring order

A dependency-driven sequence for creation and substantial updates.

### Evolution rules

How new documents are introduced, split, deprecated, superseded, or removed.

### Gaps and omissions

Missing, blocked, provisional, or intentionally omitted documents.

## 7. Requirements

- **REQ-001**: The document shall inventory all applicable architecture
  documents.
- **REQ-002**: It shall identify canonical ownership for each concern.
- **REQ-003**: It shall document categories or layers.
- **REQ-004**: It shall present the dependency graph or a usable representation.
- **REQ-005**: It shall define reading and authoring order.
- **REQ-006**: It shall document lifecycle and status.
- **REQ-007**: It shall document change-propagation expectations.
- **REQ-008**: It shall identify missing, provisional, and intentionally omitted
  documents.
- **REQ-009**: It shall avoid duplicating substantive document content.
- **REQ-010**: It shall remain consistent with machine-readable metadata.
- **REQ-011**: It shall identify unresolved cycles, ownership conflicts, and
  broken relationships.
- **REQ-012**: It shall be understandable before a contributor reads every
  individual architecture document.
- **REQ-013**: It shall distinguish the architecture-document system from the
  product or runtime architecture.
- **REQ-014**: It shall provide stable links to canonical documents.

## 8. Constraints

- **CON-001**: `META.md` shall not become a duplicate architecture summary.
- **CON-002**: It shall not redefine concerns owned by individual documents.
- **CON-003**: It shall not rely solely on directory order.
- **CON-004**: It shall not conceal missing or invalid architecture documents.
- **CON-005**: It shall not present generated catalog data as manually maintained
  prose without synchronization.
- **CON-006**: It shall not introduce dependency cycles.
- **CON-007**: It shall not require every repository to use every architecture
  document.
- **CON-008**: It shall not treat an omitted document as missing when omission is
  intentional and justified.
- **CON-009**: It shall not replace decision records or governance policy.
- **CON-010**: It shall not describe implementation topology unless necessary to
  explain document ownership.

## 9. Architecture categories

A common category model is:

| Category | Typical concerns |
| --- | --- |
| Identity | Purpose, vision, principles, pillars, manifesto |
| Meta | Epistemology, AI constitution, architecture-system map |
| Domain | Ontology, concepts, personal or domain models |
| Foundation | Foundations, system, architecture, methodology, roadmap |
| Experience | Design and design system |
| Governance | Decisions and architectural evolution |

Categories improve navigation. They do not replace explicit metadata.

## 10. Evolution model

When adding a document:

1. define its unique concern
2. create or identify its specification
3. assign a stable artifact identifier
4. declare dependencies and relationships
5. verify no concern duplication
6. update the catalog and `META.md`
7. update relevant skills, bundles, and validation

When changing an upstream document:

1. identify downstream consumers
2. classify the change as patch, minor, or major
3. review affected documents
4. record unresolved impact
5. update relationships and status
6. preserve supersession history when necessary

When removing a document:

1. confirm its concern is no longer needed or has a successor
2. preserve historical discoverability
3. update references and bundles
4. validate that no orphaned dependency remains

## 11. Authoring contract

### Inputs

Use:

- every applicable architecture document
- architecture specifications
- artifact metadata and generated catalog
- relationship graph
- lifecycle and validation results
- architecture decisions
- known omissions and open questions

### Outputs

Produce:

- architecture-system overview
- document inventory
- ownership map
- relationship graph
- reading and authoring order
- lifecycle and validation overview
- change-propagation guidance
- gaps and omissions

### Authoring process

1. discover all architecture artifacts
2. resolve stable identifiers and specifications
3. classify documents by concern and category
4. build the relationship graph
5. detect cycles, missing edges, and ownership conflicts
6. derive reading and authoring order
7. record status, gaps, and omissions
8. write concise human-readable navigation
9. validate against machine-readable metadata

### Update conditions

Update whenever the architecture inventory, relationship graph, ownership map,
lifecycle, or category model changes materially.

## 12. AI authoring strategy

AI systems shall:

1. discover documents rather than rely on a hardcoded list
2. use metadata as the primary relationship source
3. summarize ownership without duplicating content
4. preserve missing and invalid states
5. detect cycles and conflicts
6. distinguish reading order from authoring order
7. avoid inventing relationships
8. report unresolved identifiers and incomplete metadata

## 13. Dependency model

Upstream:

- architecture document standard
- `EPISTEMOLOGY.md`
- `AI_CONSTITUTION.md`
- the applicable architecture inventory and metadata

Downstream:

- architecture onboarding
- architecture-authoring skill
- architect agent
- aether bundles and catalogs
- pace validation and synchronization
- observatory architecture views
- repository documentation navigation

`META.md` should be generated late enough to observe the applicable document
set. It should not become an upstream semantic dependency for every document.

## 14. Validation

Confirm:

- every applicable document is inventoried
- each concern has a canonical owner
- dependencies resolve
- the graph is acyclic
- reading and authoring order are understandable
- omitted documents are distinguished from missing ones
- lifecycle status is accurate
- prose matches generated metadata
- substantive content is not duplicated
- links resolve

## 15. Acceptance criteria

- [ ] Applicable architecture documents are inventoried.
- [ ] Categories and layers are explained.
- [ ] Canonical ownership is explicit.
- [ ] Dependency and relationship flow is understandable.
- [ ] Reading order is provided.
- [ ] Authoring order is provided.
- [ ] Lifecycle and validation status are represented.
- [ ] Missing, provisional, and intentionally omitted documents are distinct.
- [ ] Change-propagation rules are documented.
- [ ] No individual document's substantive content is duplicated.
- [ ] The document agrees with machine-readable metadata.
- [ ] New contributors and agents can navigate the architecture system.

## 16. Examples and edge cases

### New document introduced

Add its specification, stable identifier, concern ownership, dependencies,
catalog entry, and `META.md` relationship.

### Repository does not need `PERSONAL_MODEL.md`

Record it as intentionally omitted, not missing.

### Metadata and prose disagree

Treat the machine-readable relationship data as requiring reconciliation. Do not
silently choose one representation.

### Cycle discovered

Report the cycle and block full validation until the relationship model is
corrected.

## 17. Rationale and context

Large architecture-document systems become difficult to navigate unless their
own organization, ownership, and evolution are explicit. `META.md` supplies the
human-readable map while catalogs and validators provide machine-readable
support.

## 18. Related artifacts

- `architecture-document`
- `architecture-epistemology`
- `architecture-ai-constitution`
- `architecture-decisions`
- `architecture-authoring`
- `create-meta-architecture-document`
