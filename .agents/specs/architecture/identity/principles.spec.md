---
schema: aether.specification/v1
id: architecture-principles
title: Principles Architecture Document Specification
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
  - principles
  - authoring
applies_to:
  - architecture-documents
  - principles-documents
depends_on:
  - architecture-document
  - architecture-purpose
  - architecture-vision
related:
  - architecture-pillars
  - architecture-manifesto
  - architecture-methodology
  - architecture-foundations
  - create-principles-document
supersedes: []
---

# Principles Architecture Document Specification

## Introduction

This specification defines how `PRINCIPLES.md` shall be authored, maintained, and
validated.

`PRINCIPLES.md` answers:

> How should decisions be evaluated when multiple valid options exist?

The document is part of the architecture identity system and conforms to
`architecture-document`.

## 1. Purpose and Scope

This specification governs the canonical `principles` concern for a
repository, product, platform, or organization.

It should remain durable across implementation, technology, team, and delivery
changes.

## 2. Conceptual Model

The document must make its primary identity concern explicit, preserve its
boundary against adjacent identity documents, and provide enough context for
humans and agents to apply it consistently.

## 3. Responsibilities

`PRINCIPLES.md` owns:

- durable decision heuristics
- cross-disciplinary values
- trade-off guidance
- principle precedence
- exception expectations
- decision consistency

## 4. Non-Responsibilities

`PRINCIPLES.md` does not own:

- manifesto beliefs
- strategic capabilities
- mandatory policies
- coding standards
- operational procedures
- accepted architecture decisions

## 5. Definitions

### Principle

A durable rule or heuristic used to evaluate decisions.

### Trade-off

A cost accepted to gain a preferred outcome.

### Precedence

The relationship used when principles conflict.

### Exception

A documented deviation justified by context.

### Policy

A mandatory governance constraint stronger than a principle.

## 6. Requirements

- **REQ-001**: Every principle shall provide actionable decision guidance.
- **REQ-002**: Principles shall derive from purpose and vision.
- **REQ-003**: Principles shall remain implementation-independent.
- **REQ-004**: Every principle shall explain its rationale.
- **REQ-005**: Every principle shall clarify the trade-off it influences.
- **REQ-006**: The collection shall address conflict or precedence.
- **REQ-007**: Principles shall apply across relevant disciplines.
- **REQ-008**: Principles shall be distinguishable from policies, standards, and preferences.
- **REQ-009**: Exceptions shall be documented rather than silently normalized.
- **REQ-010**: Principles shall be concise enough to remember and specific enough to use.

## 7. Constraints

- **CON-001**: Framework choices shall not become principles.
- **CON-002**: Temporary preferences shall not become principles.
- **CON-003**: Principles shall not merely repeat manifesto beliefs.
- **CON-004**: Principles shall not prescribe implementation unnecessarily.
- **CON-005**: Vague virtues without decision impact shall be excluded.
- **CON-006**: Conflicting principles shall not be left entirely ungoverned.
- **CON-007**: Principles shall not override explicit policy without a formal exception.
- **CON-008**: The document shall not become a collection of slogans.

## 8. Authoring Contract

### Inputs

- PURPOSE.md
- VISION.md
- organizational values
- recurring historical trade-offs
- architectural and engineering experience
- existing policies and standards

### Outputs

- a concise principle set
- rationale for each principle
- trade-off guidance
- application examples
- conflict-resolution guidance
- exception expectations

### Authoring Process

1. Read purpose and vision.
2. Identify recurring decisions and tensions.
3. Convert durable lessons into heuristics.
4. Remove technology-specific language.
5. Test each principle against real trade-offs.
6. Identify conflicts and precedence.
7. Distinguish principles from policies and standards.
8. Validate memorability and applicability.

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

### Downstream

- PILLARS.md
- MANIFESTO.md
- foundations
- system
- architecture
- methodology
- design
- governance

The authoritative graph is represented by artifact identifiers in frontmatter.

## 11. Validation

Confirm that:

- each principle can guide a real decision
- rationale is present
- trade-off guidance is present
- implementation independence is preserved
- policies and standards are not duplicated
- conflicts and exceptions are addressed
- purpose and vision alignment is clear

Validation shall distinguish structural, relationship, semantic, and evidence
results.

## 12. Acceptance Criteria

- [ ] Each principle can guide a real decision.
- [ ] Rationale is present.
- [ ] Trade-off guidance is present.
- [ ] Implementation independence is preserved.
- [ ] Policies and standards are not duplicated.
- [ ] Conflicts and exceptions are addressed.
- [ ] Purpose and vision alignment is clear.

- [ ] The governing specification and version are recorded.
- [ ] Required upstream artifacts are available or explicitly marked missing.
- [ ] Assumptions and open questions are visible.
- [ ] Markdown and metadata pass repository quality gates.

## 13. Examples and Edge Cases

### Strong Direction

    Prefer open standards over proprietary formats because they improve portability, interoperability, and long-term ownership.

### Anti-Pattern

    Use React for every user interface.

The anti-pattern should be rejected or rewritten because it violates the
document boundary.

## 14. Rationale and Context

Shared principles allow independent contributors and agents to reach consistent decisions without centrally prescribing every situation.

## 15. Related Artifacts

- `architecture-pillars`
- `architecture-manifesto`
- `architecture-methodology`
- `architecture-foundations`
- `create-principles-document`
