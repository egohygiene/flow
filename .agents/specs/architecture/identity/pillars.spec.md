---
schema: aether.specification/v1
id: architecture-pillars
title: Pillars Architecture Document Specification
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
  - identity
  - pillars
  - authoring
applies_to:
  - architecture-documents
  - pillars-documents
depends_on:
  - architecture-document
  - architecture-purpose
  - architecture-vision
  - architecture-principles
related:
  - architecture-manifesto
  - architecture-foundations
  - architecture-system
  - architecture-roadmap
  - create-pillars-document
supersedes: []
---

# Pillars Architecture Document Specification

## Introduction

This specification defines how `PILLARS.md` shall be authored, maintained, and
validated.

`PILLARS.md` answers:

> What enduring capabilities must remain strong for this work to succeed?

The document is part of the architecture identity system and conforms to
`architecture-document`.

## 1. Purpose and Scope

This specification governs the canonical `pillars` concern for a
repository, product, platform, or organization.

It should remain durable across implementation, technology, team, and delivery
changes.

## 2. Conceptual Model

The document must make its primary identity concern explicit, preserve its
boundary against adjacent identity documents, and provide enough context for
humans and agents to apply it consistently.

## 3. Responsibilities

`PILLARS.md` owns:

- enduring strategic capabilities
- sustained investment themes
- capability boundaries
- relationships between pillars
- initiative-to-pillar alignment
- high-level capability health signals

## 4. Non-Responsibilities

`PILLARS.md` does not own:

- decision heuristics
- temporary initiatives
- product features
- system decomposition
- architecture components
- roadmap sequencing
- staffing plans
- implementation technology

## 5. Definitions

### Pillar

An enduring strategic capability requiring sustained attention and investment.

### Capability

An ability the organization or system must maintain over time.

### Investment Area

A domain in which continued effort is intentionally committed.

### Initiative

A time-bounded effort that may reinforce one or more pillars.

### Health Signal

Evidence that a pillar remains effective without becoming a roadmap target.

## 6. Requirements

- **REQ-001**: Every pillar shall support purpose and vision.
- **REQ-002**: Pillars shall represent enduring capabilities.
- **REQ-003**: Pillars shall be mutually distinguishable.
- **REQ-004**: Every pillar shall define scope and non-scope.
- **REQ-005**: Every pillar shall explain its strategic contribution.
- **REQ-006**: Pillars shall remain implementation-independent.
- **REQ-007**: Initiatives shall be mappable to one or more pillars.
- **REQ-008**: The document shall explain pillar relationships.
- **REQ-009**: Health signals may be defined without becoming milestones.
- **REQ-010**: The set shall remain small enough to preserve strategic focus.

## 7. Constraints

- **CON-001**: Temporary projects shall not become pillars.
- **CON-002**: Technology choices shall not define pillars.
- **CON-003**: Pillars shall not duplicate principles.
- **CON-004**: Pillars shall not become system components.
- **CON-005**: Pillars shall not become roadmap epics.
- **CON-006**: Broad categories with no strategic meaning shall be excluded.
- **CON-007**: Every initiative need not create a new pillar.
- **CON-008**: Pillar health signals shall not imply false precision.

## 8. Authoring Contract

### Inputs

- PURPOSE.md
- VISION.md
- PRINCIPLES.md
- historical and current strategic work
- recurring capabilities
- organizational strengths and gaps
- long-term responsibilities

### Outputs

- a focused pillar set
- intent and contribution for each pillar
- scope and non-scope
- pillar relationships
- optional health signals
- initiative alignment guidance

### Authoring Process

1. Read upstream identity documents.
2. Identify recurring capabilities needed across multiple initiatives.
3. Remove temporary projects and technologies.
4. Group overlapping capabilities.
5. Define clear boundaries.
6. Validate contribution to purpose and vision.
7. Test whether future initiatives can align to the set.
8. Keep the set intentionally small.

### Update Conditions

Update this document only when its underlying identity concern changes
materially. Implementation changes alone are insufficient.

## 9. AI Authoring Strategy

AI systems shall:

1. read all required upstream identity documents
2. inspect available repository and organizational evidence
3. preserve canonical terminology
4. distinguish observed evidence from inference and recommendation
5. avoid implementation and roadmap leakage
6. surface contradictions rather than smoothing them over
7. label assumptions and missing evidence
8. validate the result against this specification

AI systems shall not invent organizational intent or claim completion when
required evidence is unavailable.

## 10. Dependency Model

### Upstream

- PURPOSE.md
- VISION.md
- PRINCIPLES.md

### Downstream

- MANIFESTO.md
- foundations
- system
- architecture
- roadmap
- strategic initiatives
- investment planning

The authoritative graph is represented by artifact identifiers in frontmatter.

## 11. Validation

Confirm that:

- each pillar is enduring
- each pillar contributes to purpose and vision
- pillars differ from principles and systems
- temporary initiatives are absent
- scope boundaries are clear
- relationships are understandable
- initiatives can align without redefining pillars

Validation shall distinguish structural, relationship, semantic, and evidence
results.

## 12. Acceptance Criteria

- [ ] Each pillar is enduring.
- [ ] Each pillar contributes to purpose and vision.
- [ ] Pillars differ from principles and systems.
- [ ] Temporary initiatives are absent.
- [ ] Scope boundaries are clear.
- [ ] Relationships are understandable.
- [ ] Initiatives can align without redefining pillars.

- [ ] The governing specification and version are recorded.
- [ ] Required upstream artifacts are available or explicitly marked missing.
- [ ] Assumptions and open questions are visible.
- [ ] Markdown and metadata pass repository quality gates.

## 13. Examples and Edge Cases

### Strong Direction

    Open knowledge: keep knowledge portable, transparent, and accessible in support of user ownership and interoperability.

### Anti-Pattern

    Migrate to Kubernetes.

The anti-pattern should be rejected or rewritten because it violates the
document boundary.

## 14. Rationale and Context

Pillars preserve strategic continuity while allowing specific initiatives and technologies to change.

## 15. Related Artifacts

- `architecture-manifesto`
- `architecture-foundations`
- `architecture-system`
- `architecture-roadmap`
- `create-pillars-document`
