---
schema: aether.architecture-document/v1
id: flow-adr-0014
title: Keep operational observability separate from domain evidence
kind: architecture-document
version: 0.1.0
status: proposed
owners:
  - egohygiene
created: 2026-10-26
updated: 2026-10-26
governed_by:
  - architecture-decisions
depends_on:
  - flow-architecture
related:
  - flow-decisions
  - flow-roadmap
supersedes: []
---

# ADR-0014: Keep operational observability separate from domain evidence

## Status and authority

**Proposed; acceptance date pending maintainer review.** Flow #15 requests a
normative profile before the two CLI pilots. This record introduces a proposed
portable telemetry allowlist, configuration/stream behavior and bounded sink
lifecycle; it does not restate existing evidence authority as a new decision.
Publishing or merging the draft alone does not accept this ADR. ADR-0002,
0004, 0006 and 0009 keep their existing scope and status.

## Context

The [pinned inventory](../../../observability/inventory.md) shows useful native
results/evidence across the suite but incompatible presentation and logging
surfaces. Aniflow sends machine errors to stderr; Optiflow sends its native
result there only as human diagnostics and keeps JSON on stdout. Renderflow has
a global infallible CLI subscriber bootstrap and command-specific output modes.
Flow owns both its existing extension JSONL process protocol and a distinct
Optiflow native single-document adapter, but no supported product CLI yet.

Blindly forwarding native errors, provider captures or manifests to a collector
would expose local paths/content and confuse observations with accepted state.
Putting telemetry knobs in domain configs would also risk changing deterministic
identity. A shared package chosen before two pilots could freeze those mistakes.

## Proposed decision

Use one suite-owned [observability profile](../../../observability/README.md)
and [registry](../../../observability/registry.md), implemented locally by each
holon through its existing public seams. Libraries remain subscriber-agnostic;
delivery adapters own optional sinks and bootstrap. Default operation exports
no telemetry and creates no telemetry file. Export and propagation need separate
explicit selection and authority; no inherited carrier creates permission.

Operational logs/metrics/traces are bounded, lossy, privacy-filtered observations.
Typed progress, results/diagnostics, acceptance and durable evidence remain
separate native contracts. Product machine stdout contains its one versioned
result for handled outcomes, with documented parser/help/abort exceptions;
the accepted provider JSONL transport remains unchanged. Telemetry-only state
is excluded from deterministic semantic identity. Real propagation permissions
remain part of process authority identity.

The registry adopts pinned stable OTel mappings and labels the evolving GenAI
mapping provisional. It captures no prompts, captions, arbitrary provider output,
argv, environment values or private paths. Sinks have finite memory/time/size
limits and cannot turn an accepted domain outcome into a failure on collector
outage. The two pilots must verify feasibility before shared-package selection.

## Alternatives and trade-offs

- Reuse each current CLI unchanged: least migration work, but leaves contradictory
  streams and unsafe automatic capture surfaces for a future exporter.
- Standardize all native result/evidence schemas now: would break released
  protocols and duplicate domain ownership; this proposal keeps adapters explicit.
- Extract a suite crate first: reduces apparent duplication early, but the
  required API and MSRV/dependency costs have not yet been measured in two pilots.

The proposal favors small allowlists and safe default stderr over maximum debug
detail. Local diagnostic evidence stays available through deliberate domain
interfaces. Bounded lossy telemetry cannot provide a complete audit trail; the
existing evidence stores retain that responsibility.

## Evidence and review triggers

Source reads establish current behavior; no runtime conformance is claimed.
The [matrix](../../../observability/conformance.md) must demonstrate stream
purity, privacy, identity invariance, embedding, backpressure and cancellation.
Review this record if either pilot cannot preserve native contracts, proposed
budgets prove impractical, upstream mapping stability changes, exported fields
expand, a shared package is selected or authenticated/distributed context is added.
