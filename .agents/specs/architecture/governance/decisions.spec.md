---
schema: aether.specification/v1
id: architecture-decisions
title: Decisions Architecture Document Specification
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
  - governance
  - decisions
  - adr
  - institutional-memory
  - traceability
applies_to:
  - architecture-documents
  - decision-log-documents
depends_on:
  - architecture-document
  - architecture-principles
  - architecture-epistemology
  - architecture-foundations
  - architecture-system
  - architecture-architecture
related:
  - architecture-ai-constitution
  - architecture-methodology
  - architecture-roadmap
  - create-decisions-document
supersedes: []
---

# Decisions Architecture Document Specification

## Introduction

This specification defines how `DECISIONS.md` shall be authored,
maintained, and validated.

`DECISIONS.md` preserves the significant architectural, engineering,
product, and governance choices that explain why a repository, product,
platform, or organization is the way it is.

It records accepted decisions together with the context, evidence,
rationale, alternatives, trade-offs, consequences, and historical
lineage needed to understand them later.

Where `PRINCIPLES.md` defines how decisions should be evaluated,
`EPISTEMOLOGY.md` defines how supporting claims should be justified, and
proposal artifacts describe choices under consideration,
`DECISIONS.md` records which significant choices were actually accepted.

The document is architectural memory, not a backlog or meeting archive.

## 1. Purpose and Scope

`DECISIONS.md` answers:

> Why is the project the way it is, and which significant accepted
> choices created the current state?

This specification covers:

- accepted architectural decisions
- significant engineering decisions
- significant product and governance decisions
- decision context
- rationale
- evidence and assumptions
- alternatives considered
- trade-offs
- consequences
- status and lifecycle
- supersession
- review triggers
- related architecture and implementation impact

It does not cover:

- brainstorming
- unaccepted proposals
- meeting minutes
- GitHub issues
- implementation task lists
- sprint planning
- routine low-impact choices
- temporary experiments
- release notes
- undocumented reconstruction of historical reasoning

## 2. Conceptual Model

A complete decision record connects:

    Context
        What situation required a decision?

    Decision
        What was accepted?

    Rationale
        Why was it accepted based on information available then?

    Alternatives
        What meaningful options were considered?

    Trade-offs
        What benefits and costs were consciously accepted?

    Consequences
        What becomes easier, harder, required, or prohibited?

    Lineage
        What does this decision supersede, and what later supersedes it?

A decision record describes the reasoning available at the time of
acceptance. Later knowledge may be appended as an outcome or review note,
but must not rewrite history.

## 3. Decision Significance

A choice should normally be recorded when one or more of the following is
true:

- it changes a major architectural boundary
- it establishes or changes a durable dependency direction
- it selects a significant platform, protocol, format, or operating model
- it introduces a consequential trade-off
- it affects multiple systems, products, or repositories
- it creates a migration or compatibility obligation
- it constrains future implementation choices
- it changes governance, ownership, security, privacy, or AI authority
- it is likely to be questioned again without preserved rationale
- reversing it would be costly or disruptive

Routine, local, and easily reversible choices should generally remain in
implementation artifacts rather than the architectural decision log.

## 4. Responsibilities

`DECISIONS.md` owns:

- canonical decision index
- significant accepted decisions
- decision identifiers
- decision status
- decision context
- accepted choice
- rationale
- alternatives
- trade-offs
- expected consequences
- known outcomes
- supersession lineage
- review triggers
- links to affected architecture and implementation work

## 5. Non-Responsibilities

`DECISIONS.md` does not own:

- decision-making principles
- epistemology
- unaccepted proposals
- implementation tasks
- project management
- meeting history
- source-code commentary
- complete architecture descriptions
- product roadmaps
- policy text
- operational runbooks

Decision records may reference these artifacts but shall not replace
them.

## 6. Definitions

### Decision

A deliberate and accepted choice with meaningful architectural,
engineering, product, or governance impact.

### Decision Record

The durable artifact preserving a decision and its reasoning.

### Context

The situation, constraints, evidence, and forces that made a choice
necessary.

### Rationale

The reasoning connecting the available context to the accepted choice.

### Alternative

A meaningful option considered but not selected.

### Trade-off

A cost, limitation, or risk consciously accepted in exchange for a
benefit.

### Consequence

An expected or observed effect of the decision.

### Review Trigger

A condition that should cause the decision to be reconsidered.

### Superseded Decision

A decision intentionally replaced by a later decision.

### Decision Lineage

The traceable relationship among original, amended, and superseding
decisions.

## 7. Decision Status Model

Decision records shall use an explicit status.

Recommended statuses:

- `accepted`: currently authoritative
- `deprecated`: still present but discouraged pending replacement or
  removal
- `superseded`: replaced by another decision
- `rejected`: preserved only when the rejected choice itself has durable
  architectural importance
- `withdrawn`: removed from consideration before acceptance
- `historical`: no longer operational but retained for institutional
  memory

Proposals should normally use a separate proposal artifact rather than
appearing as accepted decision records.

## 8. Requirements

- **REQ-001**: Every significant accepted decision shall have a stable
  identifier.
- **REQ-002**: Every decision shall record its status and acceptance date.
- **REQ-003**: Every decision shall state the accepted choice clearly.
- **REQ-004**: Every decision shall preserve the context available at the
  time.
- **REQ-005**: Every decision shall include rationale.
- **REQ-006**: Meaningful alternatives shall be recorded when known.
- **REQ-007**: Trade-offs shall be documented honestly.
- **REQ-008**: Expected consequences shall be documented.
- **REQ-009**: Evidence, assumptions, and uncertainty shall remain
  distinguishable.
- **REQ-010**: Related principles, foundations, systems, architecture,
  proposals, issues, and implementation artifacts shall be linked when
  relevant.
- **REQ-011**: Superseded decisions shall remain discoverable.
- **REQ-012**: A superseded decision shall identify its replacement.
- **REQ-013**: A superseding decision shall identify what it replaces.
- **REQ-014**: Historical reasoning shall not be silently rewritten.
- **REQ-015**: Corrections shall be distinguishable from later outcome
  notes.
- **REQ-016**: Decision ownership or responsible maintainers shall be
  identifiable.
- **REQ-017**: Review triggers shall be documented when the decision
  depends on changeable assumptions.
- **REQ-018**: Consequential security, privacy, safety, accessibility, or
  AI-authority impact shall be made explicit.
- **REQ-019**: The canonical record location shall be unambiguous.
- **REQ-020**: The decision log shall remain understandable without
  inspecting source code.

## 9. Constraints

- **CON-001**: GitHub issues shall not replace decision records.
- **CON-002**: Meeting notes shall not be copied into the canonical log.
- **CON-003**: Speculation about undocumented motives shall not be
  presented as historical fact.
- **CON-004**: A current preference shall not be backdated as an original
  rationale.
- **CON-005**: Superseded records shall not be deleted merely because
  they are no longer authoritative.
- **CON-006**: Decision records shall not duplicate complete architecture
  documents.
- **CON-007**: Routine local choices shall not overwhelm the log.
- **CON-008**: Alternatives shall not be fabricated for completeness.
- **CON-009**: Outcome bias shall not rewrite the quality of the original
  reasoning.
- **CON-010**: A decision shall not be marked accepted when authority or
  approval is unresolved.
- **CON-011**: Sensitive information shall not be exposed merely to make
  the record exhaustive.
- **CON-012**: A linked ADR and its index entry shall not become
  conflicting canonical copies.

## 10. Authoring Contract

### Inputs

Use:

- `PRINCIPLES.md`
- `EPISTEMOLOGY.md`
- `FOUNDATIONS.md`
- `SYSTEM.md`
- `ARCHITECTURE.md`
- `METHODOLOGY.md`
- accepted proposals
- issue and pull-request history
- implementation evidence
- relevant discussions
- migration records
- security, privacy, accessibility, or AI-governance reviews

Discussions and implementation history are evidence sources, not
automatically canonical decision records.

### Outputs

Produce:

- stable decision identifier
- concise title
- status
- acceptance date
- owner or responsible maintainers
- scope
- context
- accepted decision
- rationale
- evidence and assumptions
- alternatives considered
- trade-offs
- expected consequences
- observed outcomes when available
- review triggers
- related artifacts
- supersession lineage
- validation result

### Authoring Process

1. Determine whether the choice is significant enough to record.
2. gather contemporaneous evidence and source material.
3. identify the accepted decision and authoritative approval.
4. write context without importing later knowledge.
5. record rationale, alternatives, and trade-offs.
6. distinguish facts, assumptions, and uncertainty.
7. document consequences and review triggers.
8. link affected architecture and implementation artifacts.
9. assign a stable identifier and status.
10. validate lineage and canonical ownership.
11. append later outcomes without rewriting the original record.

### Update Conditions

Update a decision record when:

- its status changes
- a correction is required
- an observed outcome becomes known
- a review trigger occurs
- a superseding decision is accepted
- affected-artifact links change

Do not rewrite the original context and rationale to match later
understanding.

## 11. Storage and Organization

Repositories may use one of two modes.

### Inline Log Mode

All complete records are stored directly in `DECISIONS.md`.

This mode is suitable for smaller repositories with a limited number of
significant decisions.

### Indexed ADR Mode

`DECISIONS.md` contains:

- purpose and governance
- decision index
- status summary
- navigation
- lineage overview

Detailed records are stored separately, for example:

    docs/decisions/
        0001-use-specification-first-development.md
        0002-adopt-local-first-storage.md

In this mode, each decision shall have one canonical detailed record.
The index shall summarize and link rather than duplicate complete
rationale.

## 12. AI Authoring Strategy

AI systems shall:

1. read current architecture and governance artifacts
2. identify candidate significant decisions
3. verify that a choice was actually accepted
4. gather contemporaneous evidence
5. avoid inventing rationale or alternatives
6. distinguish historical context from later interpretation
7. preserve trade-offs and uncertainty
8. detect supersession and lineage
9. identify missing links and review triggers
10. report insufficient evidence rather than fabricating a record

AI systems may summarize evidence, but shall not claim undocumented
reasoning as fact.

## 13. Dependency Model

Upstream:

- `PRINCIPLES.md`
- `EPISTEMOLOGY.md`
- `FOUNDATIONS.md`
- `SYSTEM.md`
- `ARCHITECTURE.md`

Related governance:

- `AI_CONSTITUTION.md`
- policies
- contracts
- approval mechanisms

Downstream:

- architecture reviews
- implementation planning
- roadmap changes
- migrations
- onboarding
- AI engineering agents
- repository synchronization and validation
- organizational observability

## 14. Validation

Validate:

- significance
- stable identity
- accepted authority
- status and date
- contemporaneous context
- explicit rationale
- evidence and assumptions
- alternatives and trade-offs
- consequences
- related-artifact traceability
- lineage integrity
- historical accuracy
- absence of invented reasoning
- canonical record ownership

## 15. Acceptance Criteria

- [ ] The decision is significant enough to preserve.
- [ ] A stable identifier is present.
- [ ] Status and acceptance date are present.
- [ ] The accepted choice is explicit.
- [ ] Context and rationale are understandable.
- [ ] Evidence, assumptions, and uncertainty are distinguishable.
- [ ] Alternatives are documented when known.
- [ ] Trade-offs and consequences are explicit.
- [ ] Relevant architecture and implementation artifacts are linked.
- [ ] Security, privacy, accessibility, safety, and AI-authority impacts
  are addressed when relevant.
- [ ] Review triggers are present when useful.
- [ ] Supersession lineage is complete.
- [ ] Historical reasoning has not been rewritten.
- [ ] No undocumented rationale has been invented.
- [ ] The canonical record location is unambiguous.
- [ ] Future contributors can understand why the decision was made.

## 16. Examples and Edge Cases

### Valid Decision

    Decision ID:
    adr-0001

    Title:
    Adopt specification-first development

    Status:
    accepted

    Context:
    AI-assisted implementation required more stable and explicit
    contracts than issue descriptions alone provided.

    Decision:
    Significant features begin with an approved specification before
    implementation.

    Rationale:
    This improves traceability, enables deterministic validation, and
    gives humans and AI a shared implementation contract.

    Trade-off:
    Additional up-front design effort is accepted in exchange for
    reduced implementation ambiguity.

### Inferred Rationale

A repository uses a particular database, but no decision evidence exists.

Do not create a confident decision record claiming why it was selected.
Record the evidence gap or reconstruct the record explicitly as
provisional historical research.

### Superseded Decision

Retain the original record, mark it `superseded`, and link both
directions:

    superseded_by:
      - adr-0018

The replacement record should contain:

    supersedes:
      - adr-0004

### Routine Choice

Renaming a private helper function does not normally require an
architectural decision record.

## 17. Rationale and Context

Architecture is shaped as much by accepted trade-offs as by current
structure.

Without durable decision records, contributors repeatedly revisit old
questions, lose historical constraints, and accidentally reverse choices
whose rationale is no longer visible.

`DECISIONS.md` preserves institutional memory while allowing decisions
to evolve through explicit, traceable supersession.

## 18. Related Artifacts

- `architecture-document`
- `architecture-principles`
- `architecture-epistemology`
- `architecture-foundations`
- `architecture-system`
- `architecture-architecture`
- `architecture-ai-constitution`
- `architecture-methodology`
- `architecture-roadmap`
- `architecture-authoring`
- `create-decisions-document`
