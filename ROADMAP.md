---
schema: aether.architecture-document/v1
id: flow-roadmap
title: Flow Roadmap
kind: architecture-document
version: 0.2.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-21
governed_by:
  - architecture-roadmap
depends_on:
  - flow-vision
  - flow-architecture
related:
  - flow-pillars
  - flow-system
  - flow-methodology
supersedes: []
---

# Flow Roadmap

## Strategic context

Flow begins with three independently released Rust tools and no suite
orchestrator implementation. Flow will integrate named releases through public
libraries or versioned CLI contracts. It will not consolidate repositories,
copy sibling source, or hide missing provider capabilities inside the facade.

This roadmap advances on capability evidence rather than dates.

## Approved baseline

The suite ownership table, forbidden dependency edges, federated repository
model, and first contract set are approved by [ADR-0004](docs/architecture/governance/decisions/ADR-0004-federated-suite-contracts.md).
Artifact, capability, and compatibility schemas begin as provisional v1
contracts so real adapter work can refine them without claiming stability.

**Exit evidence:** architecture validation passes; every suite capability has
one primary owner; contract documents are machine-valid; no mutable or copied
sibling dependency is permitted.

## Now — Prove released integration seams

### Pin the capability matrix

Reinspect each provider at named released revisions. Record available public
libraries, CLI commands, structured formats, side effects, runtime tools, and
known gaps in `docs/integrations/capability-matrix.md`.

**Exit evidence:** every adapter target names a release, interface kind, contract
version, observed behavior, and compatibility result.

### Build adapter contract tests

Implement Flow-owned fixtures and adapters without changing provider ownership.
Choose an in-process adapter only for a stable released library; otherwise use
direct argv invocation with independent stdout/stderr capture, version probing,
signal forwarding, redacted diagnostics, and declared-output verification.

**Exit evidence:** supported versions pass; unknown versions and malformed
provider output fail actionably; no human console text is parsed.

### Implement the restore-and-assess slice

The first slice composes Optiflow and Aniflow:

1. Optiflow inventories a read-only source collection and emits evidence.
2. Flow resolves and records one deterministic plan.
3. Aniflow creates and validates a new temporal master without modifying the
   source.
4. Optiflow rescans source plus output and reports relationships and
   opportunities without applying mutations.
5. Flow records artifacts, compatibility decisions, stage state, provenance,
   validation, and exact resume guidance.

Renderflow is deliberately outside this slice. It joins through its own adapter
after the first two-provider composition proves the orchestration seam.

**Exit evidence:** a redistribution-safe fixture completes with real released
providers; an induced post-Aniflow failure resumes without repeating compatible
work; plan, state, report, logs, provenance, and validation agree.

## Next — Strengthen provider seams

### Aniflow

Formalize structured capability/result output, retain temporal-domain
checkpoint semantics, and make unsupported stream/timing behavior explicit.

### Optiflow

Preserve the read-only v0.1 safety boundary. Expose inventory and relationship
evidence without implying that observation is mutation or optimization.

### Renderflow

Stabilize structured build results and the public core/CLI seam. Prove a
separate Flow adapter for document, image, or audio derivatives; do not describe
general video packaging as present capability.

**Exit evidence:** each provider remains independently buildable and releasable;
Flow compatibility fixtures exercise only public released interfaces.

## Next — Build orchestration primitives

- `flow doctor`: discover provider versions and explain compatibility.
- `flow inspect`: preserve source observations and artifact identity.
- `flow plan`: resolve typed capabilities before expensive work and emit a
  stable plan digest.
- `flow run`: use isolated workspaces, atomic state, bounded cancellation, and
  verified outputs.
- `flow resume`: reuse only checkpoints whose inputs, configuration,
  implementation, and outputs remain compatible.

## Later

- Versioned events, results, provenance evidence, plans, and run manifests.
- Opt-in provenance tooling and honest derivative claims under ADR-0002.
- Reusable signed pipeline packs and third-party adapter contracts.
- Cross-run content-addressed caching with explicit privacy and invalidation.
- Richer interfaces after CLI and structured semantics stabilize.

## Maybe

- Distributed execution with explicit data-residency policy.
- Hardware-aware scheduling across local accelerators.
- A trust-policy registry for provenance validators.

## Dependencies and risks

| Risk | Strategic response |
| --- | --- |
| Provider release or toolchain divergence | Pin named releases and prefer process isolation when library compatibility is costly |
| Schema churn | Version explicitly and reject unsupported versions without silent fallback |
| Overlapping orchestration logic | Keep domain execution in providers and cross-holon decisions in Flow |
| External tool fragility | Probe capabilities, isolate adapters, and retain raw diagnostics |
| Misleading provenance claims | Enforce ADR-0002 and explicit epistemic claim states |
| Source or history duplication | Keep implementations and history in their owning repositories; consume releases only |

## Open questions

- Which provider APIs are stable enough for library adapters?
- How should a Flow release publish its tested provider-version matrix?
- Which provisional JSON contracts should later gain Rust bindings?

## Validation

Roadmap horizons advance on named evidence, not elapsed time. New work must align
with the ownership table, [suite boundaries](docs/integrations/suite-boundaries.md),
versioned contracts, and accepted decisions.
