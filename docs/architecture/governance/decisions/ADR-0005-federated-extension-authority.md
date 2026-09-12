---
schema: aether.architecture-decision/v1
id: adr-0005
title: Separate extension declarations from operator authority
kind: architecture-decision
status: accepted
accepted: 2026-09-12
owners:
  - egohygiene
scope:
  - flow
  - aniflow
  - optiflow
  - renderflow
governed_by:
  - architecture-decisions
supersedes: []
superseded_by: []
related:
  - flow-architecture
  - adr-0004
---

# ADR-0005 — Separate extension declarations from operator authority

## Context

ADR-0004 keeps holons independently released behind versioned library or
process contracts. A federated extension contract must now describe identity,
capabilities, compatibility, effects, hooks, and recovery without allowing a
provider to authorize its own network, AI, GPU, filesystem, subprocess,
destructive, signing, or publication behavior.

The same contract family must support pinned in-process libraries and bounded
processes while preserving deterministic discovery, explicit conflict handling,
structured failure, and independently useful holons.

## Decision

Flow separates the extension boundary into four authorities:

1. A provider-owned manifest declares identity, publisher, integrity,
   compatibility, domain ownership, capabilities, execution modes, requested
   permissions, hooks, replacement intent, and checkpoint behavior.
2. A Flow-owned lock pins the discovered provider bytes and records trust mode,
   granted permissions, precedence, and fallback policy.
3. A Flow-owned invocation records the immutable inputs, effective
   authorization, bounded execution policy, configuration identity, and resume
   inputs for one lifecycle action.
4. Provider events and results contribute progress, diagnostics, artifacts,
   checkpoints, validation, failure classification, and provenance. Flow
   validates and commits suite evidence; providers cannot mark suite work
   complete by declaration.

Discovery is explicit and configured. Presence on `PATH` is not authorization
to execute. In-process and process providers use the same semantic envelopes;
process providers additionally run behind enforced resource and side-effect
bounds.

Provider replacement declarations are advisory. The operator lock owns final
precedence, conflicts, fallback, and grants. Unknown contract majors,
incompatible providers, duplicate unresolved capabilities, malformed manifests,
and requested-but-ungranted permissions fail explicitly.

Observer hooks are read-only event consumers. Any content-changing hook is a
normal declared transform capability that consumes immutable inputs and emits a
new immutable artifact. Hooks cannot mutate a plan, approval state, validation
result, or publication state in place.

## Rationale

Separating declaration from authority prevents self-escalation while preserving
third-party extensibility. One semantic envelope keeps library and process
adapters behaviorally comparable, and immutable artifact/result records make
partial work inspectable and resumable without importing provider internals.

## Alternatives considered

- Trust every discovered binary: rejected because discovery would become code
  execution and bypass reproducible configuration.
- Let manifests grant permissions and precedence: rejected because providers
  would control their own authority and conflict resolution.
- Support only in-process plugins: rejected because release toolchains,
  dependency graphs, and failure isolation differ across holons.
- Support only process plugins: rejected because stable released libraries can
  provide useful typed composition without console parsing.
- Permit arbitrary pre/post mutation hooks: rejected because they can bypass
  immutable artifacts, planning, authorization, and validation.

## Trade-offs

The split adds explicit lock and envelope artifacts, plus compatibility work for
each provider release. In return, authority, reproducibility, rollback,
diagnostics, and provenance remain inspectable across execution modes.

## Expected consequences

Flow publishes versioned manifest, lock, invocation, event, result, and
resolution schemas. Holons retain native domain contracts and may expose
adapters without depending on Flow source. Flow #3 supplies the executable
cross-holon interruption/resume proof after released provider contracts exist.

## Security, privacy, and AI authority

Secrets are resolved from approved handles and are never serialized in
manifests, locks, events, results, diagnostics, or provenance. Requested remote,
AI, GPU, mutation, signing, destructive, and publication effects require
explicit grants and remain visible in the effective plan and evidence.

## Review triggers

Review if a required execution mode cannot preserve the same trust and evidence
boundary, if sandbox guarantees materially change, or if a released contract
requires provider-controlled authorization.

## Related artifacts

`flow-architecture`, ADR-0004, `docs/integrations/extension-contract.md`, Flow
issue #7, and Flow issue #3.

## Validation

Contract validation and compatibility fixtures must cover compatible,
incompatible, over-permissioned, duplicate, and malformed providers. Runtime
work must later prove bounded cancellation, partial results, and resume without
weakening this authority split.
