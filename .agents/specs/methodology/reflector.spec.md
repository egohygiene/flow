---
schema: aether.specification/v1
id: reflector
title: Reflective Development System Specification
kind: specification
version: 2.0.0
status: draft
owners:
  - egohygiene
created: 2026-05-14
updated: 2026-08-02
domain: methodology
tags:
  - reflective-development
  - recursive-ai
  - human-governance
  - synchronization
  - drift-containment
applies_to:
  - ai-assisted-development
  - recursive-workflows
depends_on:
  - specfile
  - auditor
related:
  - orchestrate-reflective-development
supersedes: []
source_files:
  - reflector.spec.md
---

# Reflective Development System Specification

## Introduction

This specification defines a human-governed recursive development method for
AI-assisted software engineering.

Reflective development allows implementation, auditing, planning, and
refinement to repeat while keeping recursion bounded by explicit scope, budgets,
synchronization checkpoints, and human authority.

## 1. Purpose and Scope

This specification answers:

> How can AI-assisted development recurse productively while remaining bounded,
> reviewable, synchronized, and human-governed?

It covers recursive execution, milestone synchronization, human alignment,
auditing, drift detection, recursion budgets, execution modes, and resumable
state.

It does not authorize unlimited issue generation, hidden autonomous execution,
or bypassing human approval.

## 2. Core Invariants

- Human authority remains final for consequential scope changes.
- Recursive expansion stops at synchronization boundaries.
- Every cycle has explicit scope and completion criteria.
- Audit findings do not automatically authorize implementation.
- New work is bounded by recursion budgets.
- State is externalized in reviewable artifacts.
- Ambiguity is escalated.
- Speedrun mode reduces ceremony, not governance.
- Historical cycle artifacts remain traceable.
- Paused cycles preserve a safe resume point.

## 3. Core Entities

### Reflective Development Cycle

A bounded sequence of observation, planning, execution, validation, reflection,
and synchronization.

### Human Alignment Anchor

The person or group responsible for scope, architecture, escalation, and
continuation approval.

### Milestone Synchronization Boundary

A checkpoint that pauses recursion before additional execution.

### Scoped Autonomous Agent

An AI participant operating within explicit permissions, scope, and budgets.

### Recursive Audit

An evidence-based evaluation of repository state, alignment, quality, and drift.

### Recursive Drift

Loss of coherence, scope control, or intentionality across repeated cycles.

### Recursion Budget

Limits on depth, artifacts, issues, changed systems, unresolved decisions, time,
or review capacity.

## 4. State Machine

Recommended states:

    captured
        ↓
    scoped
        ↓
    planned
        ↓
    executing
        ↓
    validating
        ↓
    reflecting
        ↓
    synchronization-required
        ↓
    approved-to-continue | complete | paused | blocked | abandoned

Transitions shall be recorded.

Execution shall not continue from `synchronization-required` without approval.

## 5. Execution Modes

### Standard

Full specification, issue, implementation, validation, and synchronization.

### Speedrun

Grouped execution units with reduced ceremony while preserving scope, budget,
validation, and human synchronization.

### Audit-Only

Observations and recommendations without implementation.

### Recovery

State reconstruction, drift assessment, scope reduction, unresolved-decision
capture, and safe continuation planning.

## 6. Recursion Budget Contract

Each recursive workflow shall define one or more limits:

- maximum cycle depth
- maximum generated issues
- maximum generated specifications
- maximum audit expansion
- maximum changed systems
- maximum unresolved decisions
- milestone boundary
- time or review-capacity budget

When a limit is reached:

- execution pauses
- state is recorded
- additional work becomes proposed scope
- human synchronization is required

## 7. Cycle Artifact Contract

Each cycle should record:

- cycle identifier
- parent cycle
- repository revision
- mode
- scope and exclusions
- human alignment anchor
- agents and tools
- recursion budget
- source specification or issue
- produced artifacts
- validation results
- audit findings
- unresolved decisions
- drift indicators
- status
- safe resume point
- synchronization requirement

## 8. Reflective Audit Contract

Reflective audits shall conform to `auditor`.

Audits may recommend remediation, specification correction, architecture review,
issue creation, pause, recovery, or completion.

Audit output shall not automatically trigger implementation.

## 9. Drift Indicators

Indicators may include:

- unapproved scope expansion
- repeated issue multiplication
- architecture and implementation divergence
- duplicated responsibility
- contradictory decisions
- unresolved questions hidden by implementation
- repeated low-confidence assumptions
- validation failures carried across cycles
- bypassed checkpoints
- work exceeding human review capacity

## 10. Human Synchronization

Synchronization shall result in one explicit decision:

- continue
- expand scope
- reduce scope
- revise architecture
- create a new milestone
- enter recovery
- complete
- abandon

Silence shall not imply approval.

## 11. Partial and Failure State

Allowed states:

- complete
- partial
- paused
- blocked
- failed
- abandoned

Incomplete states shall preserve completed work, remaining work, validation,
unresolved decisions, safe resume point, and required human input.

## 12. Portability and Overlays

Repositories may define overlays for milestone semantics, roles, budgets,
templates, audit focus, validation gates, and speedrun policy.

Overlays shall not remove core governance invariants.

## 13. AI Execution Rules

AI participants shall:

1. resolve current cycle state
2. read scope and budgets
3. operate only within authorization
4. externalize artifacts and decisions
5. validate before continuation
6. detect and report drift
7. stop at synchronization boundaries
8. escalate ambiguity
9. avoid speculative issue expansion
10. preserve resumable state

## 14. Acceptance Criteria

- [ ] Human alignment authority is explicit.
- [ ] Recursive scope is bounded.
- [ ] Recursion budgets are recorded.
- [ ] Synchronization is enforced.
- [ ] Audit and implementation authority remain separate.
- [ ] Cycle state is externalized.
- [ ] Speedrun mode preserves governance.
- [ ] Drift indicators are defined.
- [ ] Incomplete states are resumable.
- [ ] Ambiguity is escalated.
- [ ] Overlays cannot remove core invariants.

## 15. Related Artifacts

- `specfile`
- `auditor`
- `orchestrate-reflective-development`
