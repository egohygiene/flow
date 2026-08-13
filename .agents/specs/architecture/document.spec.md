---
schema: aether.specification/v1
id: architecture-document
title: Architecture Document Specification Standard
kind: specification
version: 2.0.0
status: draft
owners:
  - egohygiene
created: 2026-07-18
updated: 2026-08-01
domain: architecture
tags:
  - architecture
  - documentation
  - authoring
  - specification
  - standard
applies_to:
  - architecture-specifications
  - architecture-documents
depends_on: []
related:
  - architecture-purpose
  - architecture-vision
  - architecture-principles
  - architecture-pillars
  - architecture-manifesto
  - architecture-epistemology
  - architecture-ai-constitution
  - architecture-ontology
  - architecture-personal-model
  - architecture-foundations
  - architecture-system
  - architecture-architecture
  - architecture-methodology
  - architecture-roadmap
  - architecture-design
  - architecture-design-system
  - architecture-decisions
supersedes: []
---

# Architecture Document Specification Standard

## Introduction

This specification defines the canonical standard for authoring architecture
document specifications and the architecture documents produced from them
throughout the Ego Hygiene ecosystem.

It establishes the common structure, terminology, metadata, dependency model,
authoring boundaries, validation requirements, and lifecycle conventions shared
by the architecture-document system.

Document-specific specifications extend this standard by defining the purpose,
content, relationships, and acceptance criteria of one architecture document.

Examples include:

    PURPOSE.md
    VISION.md
    PRINCIPLES.md
    SYSTEM.md
    ARCHITECTURE.md
    ONTOLOGY.md
    DESIGN.md
    ROADMAP.md

This specification does not define the complete content of those documents.
It defines the contract that their specifications and generated documents must
follow.

## 1. Purpose and Scope

This specification establishes a consistent architecture language across:

- Ego Hygiene repositories
- generated repository foundations
- architecture bundles
- human contributors
- AI agents
- repository automation
- future Aether and Pace workflows

Its goals are to:

- establish canonical ownership for architectural concerns
- reduce ambiguity and accidental duplication
- improve onboarding and long-term maintainability
- enable deterministic AI-assisted authoring
- support machine-readable architecture relationships
- make architecture validation automatable
- separate stable architectural intent from changing implementation details
- preserve provenance, assumptions, and unresolved questions

This specification applies to canonical organization-owned architecture
specifications beneath:

    .agents/specs/architecture/

It also governs architecture documents produced from those specifications when
they are materialized into consumer repositories.

This specification does not govern every Markdown file or all forms of technical
documentation. General documentation, implementation guides, API references,
runbooks, and generated reports should use their own applicable contracts.

## 2. Conceptual Model

The architecture-document system contains two related artifact types.

### 2.1 Architecture Specification

An architecture specification defines:

- why an architecture document exists
- what concern it owns
- what content it must contain
- what content belongs elsewhere
- which documents it depends upon
- which documents depend upon it
- how humans and agents should author it
- how the result is validated

Example:

    purpose.spec.md

### 2.2 Architecture Document

An architecture document is the repository-specific artifact produced by
applying an architecture specification.

Example:

    PURPOSE.md

The specification is reusable across repositories.

The resulting document is specific to one product, system, organization, or
repository.

    Reusable specification
            ↓ applied to repository context
    Repository-specific architecture document

## 3. Definitions

### Architecture Document

A canonical Markdown document describing one bounded architectural concern for a
repository, product, platform, or organization.

### Architecture Specification

A reusable specification defining how a corresponding architecture document is
authored and validated.

### Architecture-Document System

The complete collection of architecture specifications, relationships,
authoring skills, agents, templates, and validation rules.

### Authoring Contract

The explicit rules governing how humans and AI agents produce or update an
architecture document.

### Canonical Document

The authoritative source for one architectural concern within a defined scope.

### Concern

A bounded area of architectural responsibility owned by one canonical document.

### Upstream Dependency

A document whose concepts or decisions must be understood before another
document can be authored correctly.

### Downstream Dependency

A document that consumes, refines, or operationalizes concepts defined by
another document.

### Architecture Graph

The directed graph connecting architecture documents through dependency,
refinement, and relationship edges.

### Consumer Repository

A repository into which Aether specifications, skills, instructions, or bundles
are installed or materialized.

### Implementation Artifact

Source code, tests, infrastructure, configuration, generated files, or runtime
behavior that realizes part of the architecture.

### Provenance

Information identifying the origin, ownership, version, and derivation of an
artifact or architectural claim.

## 4. Architecture Philosophy

Every architecture specification and resulting document shall follow these
principles.

### 4.1 Single Concern Ownership

Each architecture document shall own one bounded architectural concern.

A document may discuss related concerns when necessary, but it shall not become
their authoritative source.

### 4.2 Canonical Ownership

Every significant architectural concept should have one authoritative home.

Other documents should reference that source rather than copy or independently
redefine it.

### 4.3 Stable Knowledge

Architecture documents should change more deliberately than implementation
artifacts.

Implementation may evolve rapidly.

Architectural intent should evolve through explicit review and versioned change.

### 4.4 Progressive Refinement

Higher-level documents establish purpose, worldview, principles, and system
intent.

Lower-level documents refine those concepts into structure, behavior,
governance, design, and implementation guidance.

### 4.5 Explicit Relationships

Dependencies and related documents shall be declared rather than inferred only
from prose or directory placement.

### 4.6 Evidence Integrity

Observed facts, inferred conclusions, assumptions, decisions, and proposals
should remain distinguishable.

Documents shall not present uncertain claims as established architectural fact.

### 4.7 Human and Agent Readability

Architecture documents shall remain understandable to engineers, architects,
product contributors, technical writers, AI coding assistants, automated
validators, and future maintainers.

### 4.8 Implementation Independence

Architecture specifications should remain implementation-independent unless
implementation technology is part of the concern they explicitly own.

### 4.9 Honest Incompleteness

Missing evidence, unresolved questions, blocked decisions, and uncertain
assumptions shall be recorded honestly rather than silently invented.

## 5. Architecture Document Categories

| Category | Responsibility |
| --- | --- |
| Identity | Purpose, vision, principles, pillars, and manifesto |
| Meta | Epistemology, AI constitution, and architecture-system governance |
| Domain | Ontology, terminology, entities, and conceptual models |
| Foundation | Foundations, system model, architecture, methodology, and roadmap |
| Experience | Design intent and design-system architecture |
| Governance | Decisions, architectural change, and decision history |

Categories improve navigation but do not replace explicit dependency metadata.

## 6. Metadata Contract

Every architecture specification shall contain valid YAML frontmatter with:

- `schema`
- `id`
- `title`
- `kind`
- `version`
- `status`
- `owners`
- `created`
- `updated`
- `domain`
- `tags`
- `applies_to`
- `depends_on`
- `related`
- `supersedes`

Metadata constraints:

- Artifact identifiers shall remain stable after publication.
- Filenames and identifiers should align predictably.
- Dates shall use `YYYY-MM-DD`.
- Unknown creation dates shall not be fabricated.
- Dependencies shall reference artifact identifiers rather than file paths.
- Directory placement shall not substitute for metadata.
- Metadata changes affecting interpretation shall participate in versioning.

## 7. Standard Specification Structure

Every architecture specification shall contain the following sections unless a
document-specific exception is explicitly justified.

| Section | Required | Purpose |
| --- | --- | --- |
| Frontmatter | Yes | Machine-readable identity and relationships |
| H1 title | Yes | Canonical human-readable title |
| Introduction | Yes | Summary of the concern and specification |
| Purpose and Scope | Yes | Ownership and applicability |
| Conceptual Model or Definitions | Yes | Shared terminology and mental model |
| Responsibilities | Yes | What the resulting document owns |
| Non-Responsibilities | Yes | What belongs elsewhere |
| Requirements | Yes | Normative requirements |
| Constraints | Yes | Prohibited or bounded behavior |
| Authoring Contract | Yes | Inputs, process, and output expectations |
| AI Authoring Strategy | Yes | Agent-specific generation rules |
| Dependency Model | Yes | Upstream, downstream, and related artifacts |
| Validation | Yes | Objective validation procedure |
| Acceptance Criteria | Yes | Completion conditions |
| Examples and Edge Cases | When useful | Clarifies ambiguous situations |
| Rationale and Context | When useful | Explains significant design choices |
| Related Artifacts | Yes | Referenced specs, skills, agents, and prompts |

Empty boilerplate sections shall not be added solely to satisfy structure.

## 8. Responsibilities

Each architecture specification shall define what its corresponding architecture
document owns.

Responsibilities should be bounded, explicit, nonoverlapping, testable where
practical, and expressed independently of transient implementation details.

## 9. Non-Responsibilities

Each architecture specification shall define what its corresponding document
does not own.

When ownership belongs elsewhere, the specification should identify the
responsible artifact.

## 10. Requirements

- **REQ-001**: Every architecture specification shall conform to this standard.
- **REQ-002**: Every architecture specification shall define one primary
  architectural concern.
- **REQ-003**: Every architecture specification shall define responsibilities
  and non-responsibilities.
- **REQ-004**: Every architecture specification shall declare upstream,
  downstream, and related artifacts when applicable.
- **REQ-005**: Every architecture specification shall define measurable
  acceptance criteria.
- **REQ-006**: Every architecture specification shall define objective
  validation criteria.
- **REQ-007**: Every resulting architecture document shall identify assumptions,
  unresolved questions, or unavailable evidence when relevant.
- **REQ-008**: Every architecture document shall preserve canonical terminology
  from its upstream dependencies.
- **REQ-009**: Every architecture specification shall be independently
  addressable through a stable artifact identifier.
- **REQ-010**: Every architecture document shall be reviewable without requiring
  access to private reasoning or undocumented context.
- **REQ-011**: Dependency relationships shall be acyclic.
- **REQ-012**: Required upstream documents shall exist or be explicitly marked
  unavailable before downstream authoring is considered complete.
- **REQ-013**: Related documents shall reference canonical concepts rather than
  duplicate their definitions.
- **REQ-014**: Conflicting architectural statements shall be surfaced rather than
  reconciled silently.
- **REQ-015**: Superseded documents shall identify their successors.

## 11. Constraints

- **CON-001**: Architecture specifications shall not duplicate another
  specification's primary concern.
- **CON-002**: Architecture documents shall not silently redefine canonical
  terms owned upstream.
- **CON-003**: Architecture specifications shall not require implementation
  technology unless that technology is intrinsic to the concern.
- **CON-004**: Architecture documents shall not fabricate missing organizational,
  technical, product, or user context.
- **CON-005**: Architecture documents shall not present proposals as accepted
  decisions.
- **CON-006**: Architecture documents shall not hide contradictions merely to
  produce a coherent narrative.
- **CON-007**: Generated documents shall not contain inaccessible references to
  private chain-of-thought or undocumented agent reasoning.
- **CON-008**: Directory hierarchy shall not be the sole representation of
  dependency order.
- **CON-009**: Validation shall not rely exclusively on subjective prose review
  when structural checks are possible.
- **CON-010**: Architecture documents shall not become implementation changelogs.

## 12. Authoring Contract

Every architecture specification shall define:

- purpose
- responsibilities
- non-responsibilities
- required inputs
- downstream outputs
- authoring process
- update process
- validation process

The authoring process should cover:

1. discovering available evidence
2. reading upstream documents
3. identifying missing context
4. resolving or recording ambiguity
5. drafting the document
6. checking cross-document consistency
7. validating acceptance criteria
8. recording unresolved questions

## 13. AI Authoring Strategy

AI systems authoring architecture documents shall:

1. resolve the applicable specification and version
2. read required upstream documents before drafting
3. inspect available repository evidence
4. preserve canonical terminology
5. distinguish observed facts from inference and recommendation
6. avoid duplicating ownership from related documents
7. identify missing evidence and unresolved questions
8. use deterministic structure and stable headings
9. retain source-relative references where evidence is cited
10. validate the resulting document against the specification
11. report blocked or partial completion honestly
12. avoid changing unrelated architecture documents without explicit scope

AI systems shall not invent organizational intent, conceal architectural
conflicts, fabricate measurements, silently merge distinct concepts, expose
private reasoning as architecture evidence, or treat stylistic coherence as
proof of correctness.

## 14. Dependency Model

Architecture specifications form a directed graph.

A common sequence is:

    PURPOSE
        ↓
    VISION
        ↓
    PRINCIPLES
        ↓
    PILLARS
        ↓
    FOUNDATIONS
        ↓
    ONTOLOGY
        ↓
    SYSTEM
        ↓
    ARCHITECTURE
        ↓
    METHODOLOGY
        ↓
    ROADMAP

The authoritative dependency graph shall be derived from artifact metadata.

Validation shall detect missing artifacts, unresolved identifiers, dependency
cycles, invalid supersession chains, duplicated concern ownership, inconsistent
relationship declarations, and downstream documents based on superseded
sources.

## 15. Evidence and Uncertainty

Architecture documents should distinguish:

    Observed
    Inferred
    Decided
    Proposed
    Assumed
    Unverified
    Open question

At minimum:

- decisions shall not be presented as observations
- proposals shall not be presented as accepted architecture
- assumptions shall be explicit
- unavailable evidence shall be recorded
- contradictions shall remain visible until resolved

## 16. Validation Model

### 16.1 Structural Validation

Confirm valid frontmatter, required metadata, one H1, valid heading progression,
required sections, valid Markdown, UTF-8, and stable filename conventions.

### 16.2 Relationship Validation

Confirm dependency identifiers resolve, the graph is acyclic, supersession is
valid, related artifacts exist, and declared ownership does not conflict.

### 16.3 Semantic Validation

Confirm the document addresses its concern, respects responsibilities and
non-responsibilities, preserves terminology, remains coherent, represents its
requirements, and exposes unresolved questions.

### 16.4 Evidence Validation

Confirm factual claims are supported where required, assumptions are labeled,
unavailable evidence is acknowledged, contradictions remain visible, and
recommendations are distinguishable from decisions.

## 17. Acceptance Criteria

An architecture specification conforms when:

- [ ] It contains valid canonical frontmatter.
- [ ] It has a stable artifact identifier.
- [ ] It declares one primary architectural concern.
- [ ] It defines responsibilities and non-responsibilities.
- [ ] It identifies required inputs and downstream consumers.
- [ ] It declares dependencies through artifact identifiers.
- [ ] It defines normative requirements and constraints.
- [ ] It defines an authoring contract and AI authoring behavior.
- [ ] It defines objective validation and measurable completion criteria.
- [ ] It distinguishes evidence, assumptions, proposals, and decisions.
- [ ] It introduces no unresolved identifiers or dependency cycles.
- [ ] It does not duplicate another document's primary responsibility.
- [ ] Its Markdown passes applicable repository quality gates.

A generated architecture document conforms when:

- [ ] It follows its document-specific specification.
- [ ] It respects required upstream documents.
- [ ] It addresses repository-specific context.
- [ ] It records assumptions and unresolved questions.
- [ ] It preserves canonical terminology.
- [ ] It contains no fabricated architectural claims.
- [ ] It passes structural, relationship, semantic, and evidence validation.
- [ ] It is understandable without undocumented author context.

## 18. Markdown Standards

Architecture specifications and generated architecture documents shall:

- use valid UTF-8
- contain exactly one H1 heading
- use incremental heading levels
- use descriptive headings
- use fenced code blocks with language identifiers when needed
- prefer four-space indentation for plain text examples intended for copying
- use kebab-case specification filenames
- use stable section ordering
- avoid raw HTML unless necessary
- use relative repository links where practical
- remain compatible with configured Markdown tooling

## 19. Versioning and Lifecycle

Architecture specifications use semantic versioning.

Patch revisions cover corrections without contract changes. Minor revisions
cover backward-compatible additions. Major revisions cover incompatible
metadata, ownership, dependency, structure, or validation changes.

Recommended lifecycle states are:

    draft
    active
    deprecated
    superseded
    archived

## 20. Examples and Edge Cases

### Missing Upstream Document

Do not invent its contents. Record the missing dependency, produce a blocked or
provisional result, and identify the next required document.

### Conflicting Terminology

Preserve evidence of the conflict, identify the canonical owner, and reconcile
only when explicitly in scope.

### Multiple Concerns in One Document

Identify the concerns, determine canonical ownership, propose a safe split, and
preserve historical context.

### Significant Implementation Detail

Implementation technology may appear when it represents an intentional,
long-lived architectural constraint and the document explains why.

### Repository Does Not Need Every Document

Use only the subset appropriate to the repository. Do not create empty
placeholders.

## 21. Rationale and Context

A consistent architecture-document system enables reusable standards,
predictable navigation, reliable AI-assisted authoring, automated dependency
analysis, cross-repository consistency, deliberate evolution, Aether bundle
installation, and future Pace synchronization and drift detection.

## 22. Related Artifacts

This specification is the parent contract for canonical architecture
specifications beneath:

    .agents/specs/architecture/

Related artifact families include:

    .agents/skills/architecture/
    .agents/agents/architecture/
    library/organization/prompts/architecture/
    library/organization/instructions/
    library/organization/policies/

Expected related artifacts include the architecture-authoring skill, architect
agent, relationship maps, document templates, validation tooling, focused
document-authoring skills, and architecture bundles.
