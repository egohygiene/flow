---
name: create-decisions-document
description: Creates or updates DECISIONS.md with architectural decision records (ADRs) from repository evidence. Use when a project needs to capture, repair, or review architectural decisions and their rationale.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "draft"
  aether-spec-id: "architecture-decisions"
  aether-scope: "organization"
  aether-domain: "architecture"
  aether-owners: "egohygiene"
  aether-created: "2026-08-02"
  aether-updated: "2026-08-02"
---

# Create Decisions Document

## Purpose

Create, reconstruct, update, or validate `DECISIONS.md` and its linked
architecture decision records in conformance with
`architecture-decisions`.

The document must answer:

> Why is the project the way it is, and which significant accepted
> choices created the current state?

## Use This Skill When

- the canonical decision log is missing
- a significant decision has been accepted
- historical decisions need careful reconstruction
- a decision has been superseded or deprecated
- architecture review needs decision lineage
- implementation or repository structure contains unexplained durable
  choices
- decision records need validation or migration

## Do Not Use This Skill For

- brainstorming
- unaccepted proposals
- meeting summaries
- routine implementation choices
- issue or sprint planning
- fabricated historical rationale

## Required Inputs

Resolve:

- governing specification and version
- current architecture documents
- accepted proposal or approval evidence
- contemporaneous issue, pull-request, or discussion history
- implementation and migration evidence
- known alternatives and trade-offs
- decision owner or approving authority
- related and superseding decisions

Missing evidence must remain visible.

## Workflow

1. Determine whether the choice is significant enough to record.
2. identify whether the work is a new record, historical reconstruction,
   correction, outcome update, or supersession.
3. gather contemporaneous evidence.
4. verify that the decision was actually accepted.
5. assign or validate the stable decision identifier.
6. document context, choice, rationale, alternatives, and trade-offs.
7. distinguish evidence, assumptions, uncertainty, and later outcomes.
8. document consequences and review triggers.
9. link affected architecture, proposals, issues, pull requests, and
   implementation artifacts.
10. validate status and supersession lineage.
11. update the canonical index without creating conflicting duplicates.

## Output Contract

Produce:

- `DECISIONS.md`
- linked ADR files when indexed ADR mode is used
- governing specification identifier and version
- validation results
- evidence gaps and unresolved questions
- affected-artifact review recommendations
- complete supersession lineage

## Constraints

- Do not invent rationale, alternatives, or authority.
- Do not rewrite historical reasoning with later knowledge.
- Do not delete superseded decisions.
- Do not promote proposals to accepted decisions without evidence.
- Do not let an index and linked ADR become conflicting canonical copies.
- Do not expose sensitive information unnecessarily.
- Do not claim completion when decision authority or evidence is
  unresolved.

## Validation

Use:

    references/validation-checklist.md

and the acceptance criteria in:

    architecture-decisions

## Completion Criteria

- [ ] The decision is significant.
- [ ] Acceptance authority is verified.
- [ ] Stable identity, status, and date are present.
- [ ] Context and rationale reflect contemporaneous evidence.
- [ ] Trade-offs and consequences are visible.
- [ ] Evidence gaps are explicit.
- [ ] Related artifacts are linked.
- [ ] Supersession lineage is complete.
- [ ] Canonical record ownership is unambiguous.
