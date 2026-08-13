---
schema: aether.architecture-document/v1
id: flow-principles
title: Flow Principles
kind: architecture-document
version: 0.1.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-13
governed_by:
  - architecture-principles
depends_on:
  - flow-purpose
  - flow-vision
related:
  - flow-foundations
  - flow-methodology
supersedes: []
---

# Flow Principles

## Preserve before transforming

Never modify the original in place. Capture source fingerprints, observable
metadata, embedded claims, and inspection evidence before creating derivatives.

## Plan before executing

Resolve tools, versions, inputs, outputs, dependencies, retention, validation,
and expected effects before expensive or mutating work begins.

## Evidence over implication

Separate observed facts, inferred conclusions, declared intent, and unresolved
uncertainty. Never label a heuristic as proof.

## Holons remain whole

Every tool must remain independently useful as a library and CLI. Flow may
depend on public holon contracts; sibling holons must not depend on each other.

## One capability, one primary owner

Shared orchestration belongs to Flow. Specialized engines belong to their
holons. Convenience is not sufficient reason to duplicate capability.

## Safety and reversibility precede throughput

Refuse ambiguous destructive behavior. Prefer immutable artifacts, atomic
state, validated checkpoints, explicit retention, and recoverable failure.

## Structured contracts over console folklore

Automation consumes versioned data and typed APIs. Human output may be rich,
but it is not the integration contract.

## Local-first and privacy-preserving by default

Processing, state, and diagnostics remain local unless the user explicitly
selects and understands a remote capability. Secrets and unnecessary absolute
paths do not enter artifacts.

## Truthful provenance

Preserve discovered provenance evidence. A transformed derivative receives a
new, honest claim that references its source fingerprint and declared actions;
it must never impersonate the untouched source.

## Validate outcomes, not file existence

Completion requires contract-specific validation. A present or nonempty file is
not, by itself, a valid artifact or checkpoint.

## Principle conflicts and precedence

Human safety, authorization, source preservation, and evidence integrity outrank
speed and convenience. Holon independence outranks short-term reuse when reuse
would create sibling coupling. When principles still conflict, record a
decision and its review trigger.

## Exceptions

Exceptions must be explicit, scoped, reversible where possible, and recorded in
the decision log. Silent fallback is not an exception mechanism.

## Open questions

None currently. Proposed exceptions belong in `DECISIONS.md`.

## Validation

Architectural and implementation reviews must identify which principles govern
a trade-off and document any exception.
