---
name: create-ai-constitution-document
description: Creates or updates AI_CONSTITUTION.md from repository evidence and policy. Use when a project needs to define, repair, or review the rules and oversight under which AI systems may participate in its work.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "draft"
  aether-spec-id: "architecture-ai-constitution"
  aether-scope: "organization"
  aether-domain: "architecture"
  aether-owners: "egohygiene"
  aether-created: "2026-08-02"
  aether-updated: "2026-08-02"
---

# Create AI Constitution Document

## Purpose

Create or update `AI_CONSTITUTION.md` in conformance with `architecture-ai-constitution`.

The document must answer:

> Under what rules, authority, and oversight may AI systems participate in this work?

## Use this skill when

- the canonical document does not exist
- the existing document is incomplete or inconsistent
- upstream architecture changed materially
- the repository architecture system is being established or repaired
- a repeatable, validated authoring workflow is needed

## Do not use this skill for

- system prompt disguised as constitution
- granting authority based on capability
- provider-specific rules
- requiring private chain-of-thought

## Required inputs

- governing specification and version
- required upstream architecture documents
- applicable policies and decisions
- existing repository evidence
- known conflicts, assumptions, and open questions

Missing evidence must be recorded rather than invented.

## Workflow

1. inventory current and expected AI use cases
2. read identity, epistemology, and applicable policies
3. classify actions by impact and reversibility
4. define human authority and approval boundaries
5. define honesty, evidence, privacy, security, and least-privilege rules
6. define instruction precedence and escalation
7. test against ambiguous, destructive, and high-impact scenarios
8. remove provider-specific prompt and model details

## Output contract

Produce:

- `AI_CONSTITUTION.md`
- governing specification identifier and version
- assumptions and unresolved questions
- validation results
- downstream review recommendations

## Constraints

- Preserve canonical terminology.
- Separate evidence, inference, proposal, and decision.
- Do not invent organizational intent or authority.
- Do not silently resolve contradictions.
- Do not claim completion when required inputs are missing.
- Keep implementation detail outside the document unless the specification
  explicitly requires it.

## Validation

Use:

    references/validation-checklist.md

and the acceptance criteria in:

    architecture-ai-constitution

## Completion criteria

- [ ] The governing specification is identified.
- [ ] Required upstream evidence has been read.
- [ ] The document answers its primary question.
- [ ] Responsibilities and non-responsibilities are respected.
- [ ] Assumptions and unresolved questions are visible.
- [ ] Structural, semantic, relationship, and evidence checks pass.
- [ ] Downstream review needs are reported.
