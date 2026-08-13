---
schema: aether.specification/v1
id: architecture-personal-model
title: Personal Model Architecture Document Specification
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
  - personal-model
  - human-centered
  - agency
  - privacy
applies_to:
  - architecture-documents
  - personal-model-documents
depends_on:
  - architecture-document
  - architecture-purpose
  - architecture-vision
  - architecture-principles
  - architecture-epistemology
  - architecture-ontology
related:
  - architecture-ai-constitution
  - architecture-system
  - architecture-architecture
  - architecture-design
  - architecture-design-system
  - create-personal-model-document
supersedes: []
---

# Personal Model Architecture Document Specification

## Introduction

This specification defines how `PERSONAL_MODEL.md` shall be authored,
maintained, and validated.

`PERSONAL_MODEL.md` defines the project's conceptual understanding of the people
it serves or affects.

It documents assumptions about agency, autonomy, identity, needs, context,
relationships, growth, consent, privacy, change, and participation.

The personal model is an architecture constraint. It must never reduce a person
to a static profile, diagnosis, persona, behavioral prediction, or optimization
target.

## 1. Purpose and Scope

`PERSONAL_MODEL.md` answers:

> How does this project understand the people it serves while preserving their
> agency, dignity, privacy, complexity, and capacity to change?

It covers:

- human assumptions
- agency and autonomy
- identity and self-description
- needs and intentions
- context and relationships
- growth and change
- consent
- privacy
- inference boundaries
- correction and contestability
- limits of the project's understanding

It does not cover:

- individual user records
- behavioral personas
- diagnoses
- psychological profiling
- medical guidance
- UI implementation
- product requirements
- data schemas
- prediction models
- universal claims about human nature

## 2. Conceptual Model

The personal model distinguishes:

    Person
        The human being, never reducible to a system representation

    Representation
        Information the system holds or derives

    Context
        Circumstances that shape meaning

    Agency
        The person's ability to make meaningful choices

    Consent
        Permission that is specific, informed, revocable, and contextual

    Inference
        A conclusion derived rather than directly provided

    Change
        The expectation that people, identity, goals, and context evolve

The system's representation is always partial.

    person
        ≠
    profile
        ≠
    persona
        ≠
    dataset row

## 3. Responsibilities

`PERSONAL_MODEL.md` owns:

- canonical assumptions about people
- agency and autonomy expectations
- identity and self-description principles
- context sensitivity
- relationship considerations
- growth and change assumptions
- consent and privacy expectations
- inference boundaries
- correction and contestability expectations
- model limitations
- human participation in decisions

## 4. Non-Responsibilities

It does not own:

- individual profiles
- personas used for research
- diagnoses
- medical doctrine
- implementation architecture
- data schemas
- interface behavior
- feature requirements
- algorithmic prediction logic

## 5. Definitions

### Personal Model

The project's explicit conceptual understanding of a person in its domain.

### Agency

A person's ability to make meaningful choices and influence outcomes.

### Autonomy

A person's ability to direct their own participation, goals, and decisions.

### Identity

The evolving ways a person understands and expresses themselves.

### Context

Circumstances that affect the meaning of information or behavior.

### Consent

Specific, informed, voluntary, and revocable permission.

### Inference

A conclusion derived from evidence rather than directly stated.

### Contestability

A person's ability to challenge, correct, or reject a consequential
representation or conclusion.

### Representation

The limited information a system stores, displays, or derives about a person.

## 6. Requirements

- **REQ-001**: Human assumptions shall be explicit.
- **REQ-002**: Agency and autonomy shall be preserved.
- **REQ-003**: The model shall align with purpose, principles, epistemology, and
  ontology.
- **REQ-004**: A person shall remain distinct from the system's representation.
- **REQ-005**: User-provided, observed, and inferred information shall remain
  distinguishable.
- **REQ-006**: Inference shall preserve confidence, provenance, and uncertainty.
- **REQ-007**: The model shall assume people and context can change.
- **REQ-008**: Identity shall not be treated as fixed without justification.
- **REQ-009**: Consent shall be contextual and revocable.
- **REQ-010**: Consequential representations should be correctable or
  contestable where practical.
- **REQ-011**: Diversity shall be recognized without assuming one universal
  person.
- **REQ-012**: Privacy and sensitive-inference boundaries shall be explicit.
- **REQ-013**: Unsupported hidden profiling shall not drive consequential
  decisions.
- **REQ-014**: The model shall identify what the project does not know.
- **REQ-015**: Scientific, medical, or psychological claims shall preserve
  evidence and scope.
- **REQ-016**: Engagement, efficiency, or compliance shall not be treated as
  universal human goals.
- **REQ-017**: Meaningful human control shall be addressed for AI-assisted
  decisions.

## 7. Constraints

- **CON-001**: The document shall not diagnose individuals.
- **CON-002**: It shall not prescribe one universal model of well-being.
- **CON-003**: It shall not treat personas as real people.
- **CON-004**: It shall not essentialize identity, ability, culture, gender, or
  behavior.
- **CON-005**: It shall not infer sensitive traits without explicit
  justification.
- **CON-006**: It shall not equate engagement with benefit.
- **CON-007**: It shall not assume observed behavior reveals intent.
- **CON-008**: It shall not hide uncertainty in generated human models.
- **CON-009**: It shall not optimize people toward organization-defined outcomes
  without meaningful choice.
- **CON-010**: Historical data shall not be treated as permanently
  representative.

## 8. Authoring Contract

### Inputs

Use:

- `PURPOSE.md`
- `VISION.md`
- `PRINCIPLES.md`
- `EPISTEMOLOGY.md`
- `ONTOLOGY.md`
- `AI_CONSTITUTION.md` when available
- relevant research and expertise
- user and community evidence
- privacy and consent requirements
- documented harms and failure modes

### Outputs

Produce:

- explicit human assumptions
- agency and autonomy model
- identity and context model
- relationship model
- growth and change expectations
- consent expectations
- privacy and inference boundaries
- correction and contestability expectations
- limitations and unknowns
- downstream architecture implications

### Process

1. Read upstream identity, epistemology, and ontology.
2. inventory explicit and implicit assumptions.
3. separate evidence from values and proposals.
4. define agency, autonomy, identity, context, and change.
5. identify sensitive inference and privacy boundaries.
6. define correction and contestability.
7. remove implementation and product-specific behavior.
8. challenge universal or normative assumptions.
9. document limitations and unknowns.
10. seek expert review for high-stakes claims.

### Update Conditions

Update the personal model when foundational assumptions change, new evidence
invalidates the model, repeated harms expose a gap, or the project begins
serving substantially different populations.

Do not update it merely because a feature changes.

## 9. AI Authoring Strategy

AI systems shall:

1. read upstream architecture
2. distinguish evidence from design preference
3. avoid diagnosing, stereotyping, or essentializing people
4. preserve provenance and uncertainty
5. identify hidden assumptions
6. avoid creating a supposedly complete person model
7. define sensitive-inference boundaries
8. preserve correction and contestability
9. flag high-stakes claims for review
10. use human-centered rather than optimization language

## 10. Dependency Model

Upstream:

- `PURPOSE.md`
- `VISION.md`
- `PRINCIPLES.md`
- `EPISTEMOLOGY.md`
- `ONTOLOGY.md`

Related governance:

- `AI_CONSTITUTION.md`
- privacy and security policies

Downstream:

- `SYSTEM.md`
- `ARCHITECTURE.md`
- `DESIGN.md`
- `DESIGN_SYSTEM.md`
- AI behavior
- interaction design
- data and inference boundaries
- user-facing documentation

## 11. Validation

Validate:

- alignment with canonical ontology
- explicit assumptions
- distinction between person and representation
- evidence and uncertainty
- agency, consent, correction, and contestability
- privacy and inference boundaries
- absence of diagnosis and stereotyping
- implementation independence
- downstream design implications

## 12. Acceptance Criteria

- [ ] Human assumptions are explicit.
- [ ] Agency and autonomy are addressed.
- [ ] Identity and context can evolve.
- [ ] People are distinct from their representations.
- [ ] User-provided, observed, and inferred information are distinguished.
- [ ] Consent is contextual and revocable.
- [ ] Privacy and inference boundaries are explicit.
- [ ] Correction and contestability are addressed.
- [ ] Diversity is recognized without essentializing people.
- [ ] Unsupported scientific or medical claims are absent.
- [ ] Limitations and unknowns are visible.
- [ ] The model aligns with ontology and epistemology.
- [ ] Implementation details are absent.
- [ ] Downstream architecture and design can reference it directly.

## 13. Examples and Edge Cases

### Valid Assumption

    People are active participants in their own growth rather than passive
    recipients of recommendations.

Architectural impact:

    Systems preserve reflection, choice, correction, and meaningful control.

### Behavior Mistaken for Intent

Repeatedly dismissing a prompt does not prove disinterest, resistance, or
pathology.

### Identity Changes

A newer self-description should not be rejected merely to preserve consistency
with historical data.

### High-Stakes Inference

A model predicts a mental-health condition.

This exceeds the personal model's authority and requires appropriate clinical,
legal, ethical, and product review.

## 14. Rationale and Context

Every system contains assumptions about people. Making them explicit allows
architecture and design to be reviewed for agency, dignity, privacy, fairness,
evidence quality, and human control.

## 15. Related Artifacts

- `architecture-document`
- `architecture-epistemology`
- `architecture-ontology`
- `architecture-ai-constitution`
- `architecture-system`
- `architecture-architecture`
- `architecture-design`
- `architecture-design-system`
- `architecture-authoring`
- `create-personal-model-document`
