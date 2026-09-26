---
schema: aether.architecture-decision/v1
id: adr-0012
title: Require fresh prerequisite evidence for durable graph execution
kind: architecture-decision
status: proposed
accepted: null
owners:
  - egohygiene
scope:
  - flow
governed_by:
  - architecture-decisions
supersedes: []
superseded_by: []
related:
  - flow-architecture
  - flow-roadmap
  - adr-0011
---

# ADR-0012 — Require fresh prerequisite evidence

## Context and authority

Flow #49 persists successful steps, but a historical success does not establish
that its current inputs, provider, authority, validator, or outputs still match.
Checking only a dependent step can miss stale upstream evidence. The maintainer
authorized the next dependency-ready checkpoint on 2026-09-26. #64 is the first
checkpoint under #31; this decision is proposed for review with its implementation.

## Decision

Add a closed, versioned `flow.run-assessment/v1` report bound to the exact saved
plan digest, current state digest, and snapshot sequence. Require one explicitly
identified current context for every planned step in plan order; reject missing,
extra, duplicated, or reordered entries. Reject a different expected plan and
corrupt retained history before producing a report.

Visit steps in topological plan order. Freshly check each eligible step through
the existing context, input, validator, and artifact gates. A local stale boundary
invalidates that step. An invalidated prerequisite invalidates every descendant;
other non-reusable prerequisites block descendants. Record direct blockers in
declared dependency order and local stale boundaries as typed data. A blocked
step's future inputs need not exist yet and are not observed. Independent branches
remain eligible according to their own evidence.

Assessment is read-only. Historical success and accepted checkpoints remain
intact even when currently invalidated for reuse. Serialized reports are ordinary
descriptive data, never capabilities or accepted execution evidence. Dependent
execution accepts the complete expected plan and context inventory and computes
a new assessment immediately before entering the existing durable launch path.
The single-step assessment and execution APIs refuse any step with dependencies.

## Rationale and alternatives

Checking only recorded dependency status misses changed upstream evidence. Taking
a caller-supplied report as permission would allow stale or forged readiness.
Rewriting succeeded records to pending would erase historical acceptance and
silently permit repeat effects. The bounded alternative keeps immutable history,
fresh eligibility, and explicit execution separate. Full inventory checking also
avoids implying that omitted branches were validated.

## Limits, recovery, and compatibility

The expected plan must match exactly. Selective reuse within that plan does not
authorize migration to a changed plan or rerunning an invalidated success. Retry
and abandonment continue to use explicit recorded recovery decisions. There is
no scheduler, automatic retry, artifact deletion, or exactly-once external-effect
claim. #65 owns the broad lifecycle corpus and #66 owns effect/cleanup proofs.

The store retains ADR-0011's trusted local filesystem assumptions. Observations
are sequential, not an atomic snapshot of external files. Callers must keep inputs,
artifacts, and provider packages quiescent during assessment and execution; the
workspace lock does not contain external writers. A future concurrency guarantee
requires a stronger artifact/process boundary.

The report contract is additive; saved plan and state schemas remain unchanged.
The changed validation implementation digest intentionally refuses reuse of
checkpoints accepted by a different implementation until a future explicit
compatibility path exists. Unknown report versions/fields are refused. Reports
retain identifiers and digests but no roots, configuration values, tokens, raw
provider messages, or source bytes. They require the same privacy handling as
the workspace metadata.

## Validation and review triggers

Exercise real hermetic providers, fresh reopen, branching and transitive stale
propagation, unchanged independent work, blocked dependencies, complete inventory
refusals, forged/stale report non-authority, and the single-step bypass refusal.
Assert no launch events or history changes on refusal and no repeated successful
launch. Repeat deterministic report comparisons in fresh roots. Revisit for
automatic scheduling, cross-plan reuse, concurrent mutation, or report authority.

## Related artifacts

Flow #11, #31, and #64; `flow-architecture`;
[`durable-state.md`](../../../integrations/durable-state.md); and
`flow.run-assessment/v1`.
