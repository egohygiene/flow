---
schema: aether.specification/v1
id: specfile
title: Specification File Standard
kind: specification
version: 2.0.0
status: draft
owners:
  - egohygiene
created: 2026-07-18
updated: 2026-08-02
domain: authoring
tags:
  - specification
  - authoring
  - implementation-contract
  - validation
applies_to:
  - specification-files
  - implementation-contracts
depends_on: []
related:
  - auditor
  - reflector
  - arxiv-publishing
  - create-specification-file
supersedes: []
source_files:
  - specfile.spec.md
---

# Specification File Standard

## Introduction

This specification defines how specification files shall be authored,
organized, maintained, validated, and consumed.

A specification converts an idea, architecture concern, feature, workflow,
integration, or operational requirement into a durable implementation contract.

    idea
        ↓
    architecture
        ↓
    specification
        ↓
    implementation plan
        ↓
    GitHub issues
        ↓
    implementation
        ↓
    validation

Specifications must remain readable by humans, usable by AI agents, and
explicit enough to validate without hidden context.

## 1. Purpose and Scope

This specification answers:

> What makes a specification complete, trustworthy, implementation-ready, and
> reusable?

It covers metadata, scope, requirements, architecture, interfaces,
implementation planning, validation, acceptance criteria, uncertainty, status,
and issue generation.

It does not cover implementation code, decision records, policies, runbooks,
tests, or brainstorming capture.

## 2. Specification Types

Recommended types include:

- feature
- architecture
- process
- workflow
- integration
- data
- interface
- automation
- publishing
- governance
- migration
- quality
- agent
- artifact-standard

The type informs structure but does not remove the need for a clear contract.

## 3. Normative Language

- `MUST` and `SHALL` indicate required behavior.
- `MUST NOT` and `SHALL NOT` indicate prohibited behavior.
- `SHOULD` indicates a recommendation requiring documented justification when
  omitted.
- `MAY` indicates optional behavior.

Requirement strength shall not be exaggerated merely to sound authoritative.

## 4. Naming and Placement

Specification files shall:

- use lowercase kebab-case
- end with `.spec.md`
- have one canonical location
- avoid conflicting duplicate copies

Repository-local specifications commonly live under:

    .agents/specs/

Reusable Aether specifications live under:

    .agents/specs/
    library/community/specs/

## 5. Metadata Contract

Every specification shall define:

- schema
- stable identifier
- title
- kind and domain
- version
- status
- owners
- creation and update dates
- tags
- dependencies
- related artifacts
- supersession lineage

Recommended statuses:

- draft
- review
- approved
- in-progress
- implemented
- deprecated
- superseded

## 6. Required Content Concerns

Unless explicitly documented as not applicable, a specification shall include:

- purpose and scope
- goals
- non-goals
- context and evidence
- definitions
- requirements
- architecture or operating model
- interfaces and dependencies
- implementation plan
- validation plan
- risks and edge cases
- acceptance criteria
- open questions
- related artifacts

## 7. Requirement Contract

Requirements shall:

- use stable identifiers
- express one primary obligation
- use explicit normative strength
- be testable or reviewable
- identify relevant scope
- avoid vague qualities without defined meaning
- avoid unnecessary implementation coupling

Recommended identifier families:

- `FR-NNN`
- `NFR-NNN`
- `SEC-NNN`
- `PRV-NNN`
- `ACC-NNN`
- `OPS-NNN`
- `GOV-NNN`
- `VAL-NNN`

Identifiers shall not be silently reused for unrelated obligations.

## 8. Evidence and Uncertainty

Specifications shall distinguish:

- observed current state
- external requirement
- accepted decision
- assumption
- inference
- proposal
- unresolved question

Authors and AI systems shall not invent current behavior, stakeholder intent,
constraints, or decisions.

Contradictory sources shall remain visible until resolved.

## 9. Architecture and Interface Rules

Architecture sections should define only the detail required to constrain
implementation and validation.

When relevant, include:

- components and responsibilities
- boundaries and dependency direction
- state transitions
- data flow
- APIs, commands, files, events, or protocols
- external dependencies
- extension points
- failure and recovery behavior

## 10. Implementation Planning

Implementation plans shall:

- identify dependencies
- separate foundation, implementation, migration, and validation work
- identify likely file or module changes when known
- preserve unresolved choices
- remain divisible into reviewable issues

A specification is not required to contain every implementation task.

## 11. Validation and Acceptance

Validation shall trace to requirements.

Relevant validation may include:

- static validation
- unit tests
- integration tests
- end-to-end tests
- security review
- privacy review
- accessibility review
- manual inspection
- documentation review
- CI validation
- migration and rollback validation

Acceptance criteria shall be objective enough for a reviewer or agent to
determine conformance.

## 12. GitHub Issue Generation

Generated issues should include:

- context
- outcome
- scope and non-goals
- expected files or modules
- implementation constraints
- source requirement identifiers
- validation steps
- acceptance criteria
- dependencies

Large specifications should produce multiple small issues rather than one
oversized issue.

## 13. Change Management

Changes shall preserve version history, status, identifiers, and supersession
lineage.

Breaking contract changes should increment the major version.

A status change shall not rewrite historical meaning.

## 14. AI Authoring Rules

AI systems shall:

1. inspect available context
2. identify the problem and owner
3. distinguish evidence, decisions, assumptions, and proposals
4. define goals and non-goals
5. describe architecture before implementation
6. create stable and testable requirements
7. preserve open questions
8. avoid speculative overengineering
9. define validation and acceptance criteria
10. report insufficient evidence instead of inventing completeness

## 15. Quality Bar

A conforming specification is:

- clear
- scoped
- internally consistent
- testable
- traceable
- implementation-ready
- explicit about trade-offs
- explicit about uncertainty
- readable by humans
- usable by AI agents

## 16. Acceptance Criteria

- [ ] Metadata is complete.
- [ ] Purpose, goals, and non-goals are explicit.
- [ ] Evidence is distinguished from assumptions.
- [ ] Requirements use stable identifiers and normative language.
- [ ] Architecture is sufficient to constrain implementation.
- [ ] Interfaces and dependencies are explicit where relevant.
- [ ] Implementation work is organized coherently.
- [ ] Validation traces to requirements.
- [ ] Acceptance criteria are testable.
- [ ] Risks, edge cases, and open questions are visible.
- [ ] The specification can generate bounded implementation issues.
- [ ] No hidden context is required.

## 17. Related Artifacts

- `auditor`
- `reflector`
- `arxiv-publishing`
- `create-specification-file`
