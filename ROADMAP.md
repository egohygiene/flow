---
schema: aether.architecture-document/v1
id: flow-roadmap
title: Flow Roadmap
kind: architecture-document
version: 0.1.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-13
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

Flow begins with three working but differently shaped Rust tools and no suite
orchestrator implementation. The path forward must preserve their useful
behavior and histories while making capability ownership, public contracts,
validation, provenance, and composition coherent.

This roadmap is organized by capability outcomes rather than dates or issue
queues. A horizon advances only when its exit evidence exists.

## Now — Define and prepare

### Establish the architectural baseline

Accept the suite identity, foundations, ontology, system ownership, structural
rules, holon contracts, provenance policy, decisions, and roadmap. Install the
repository-local Aether framework so changes can be validated consistently.

**Exit evidence:** architecture graph validates; boundaries have no unresolved
ownership conflicts; accepted ADRs record the monorepo and provenance direction.

### Produce a migration and capability record

Inventory each current repository at named revisions, including source history,
crate/package structure, public APIs, commands, schemas, runtime tools, release
automation, tests, documentation, and known defects. Produce the first
`docs/integrations/capability-matrix.md` and a history-preserving import plan.

**Exit evidence:** every claimed suite capability has one verified owner; gaps
and version/toolchain differences are explicit; import rollback is documented.

### Consolidate without coupling

Import the three codebases through the accepted history strategy, retain old
repository references, and establish CI that tests each holon independently.
Choose coordinated workspace and versioning mechanics only after the inventory.

**Exit evidence:** each original test suite passes in its new location; each CLI
builds independently; history and license attribution are discoverable; forbidden
sibling dependency checks pass.

## Next — Strengthen the holons

### Aniflow: temporal engine maturity

Separate reusable temporal-domain logic from its CLI, formalize structured
results/events, harden checkpoint compatibility and cancellation, expand stream
and timing correctness beyond the current first-stream/CFR constraints, and
retain its focus on video decomposition, ordered processing, reconstruction, and
temporal validation.

**Exit evidence:** public library examples and CLI contract tests pass; resume
invalidation is deterministic; representative variable-frame-rate and multi-
stream fixtures have explicit supported or rejected behavior.

### Optiflow: evidence-led optimization

Preserve the read-only v0.1.0 safety boundary while extracting its reusable
inventory/evidence library. Expand from exact-duplicate observation toward
reviewable normalization and optimization capabilities only behind immutable
plans, current-state verification, dry runs, and reversible policies.

**Exit evidence:** scanning/planning are independently reusable; mutation is
absent or separately gated; adversarial path/filesystem and stale-plan tests
prove refusal behavior.

### Renderflow: transform graph clarity

Preserve the existing core/CLI/plugin split, stabilize graph and plugin APIs,
clarify document versus image/audio capability boundaries, strengthen structured
build results, and validate incremental/parallel execution deterministically.
Do not describe general video derivative packaging as current capability until
it is intentionally designed and implemented.

**Exit evidence:** public API compatibility policy is tested; graph plans and
build results are serializable; plugin isolation and cache correctness have
contract evidence.

## Next — Establish suite contracts

### Shared vocabulary and schemas

Define minimal versioned contracts for artifact identity, capability discovery,
tool results, events, validation, provenance evidence, plans, and run manifests.
Adapt existing holon schemas explicitly rather than relabeling them.

### Library and process adapters

Choose in-process or subprocess composition per holon capability. Process
adapters use direct argv invocation, independent stdout/stderr capture, signal
forwarding, redacted diagnostics, version probing, and declared-output
verification. Both adapter styles emit the same logical result contract.

### Suite compatibility policy

Publish tested version ranges and capability negotiation behavior. Unsupported
schema or tool versions fail actionably; there is no silent fallback.

**Exit evidence:** producer/consumer contract suites pass across all holons;
unknown-version rejection and migration behavior are tested.

## Later — Build Flow orchestration

### Deterministic planning

Implement `flow doctor`, `flow inspect`, and `flow plan` over typed pipeline
definitions and registered capabilities. Resolve complete plans before expensive
work and produce stable plan digests.

### Recoverable execution

Implement isolated run workspaces, atomic manifests, stage state, unified
events, bounded cancellation, compatible checkpoints, invalidation, retention,
diagnostics, and exact resume guidance.

### First honest vertical slice

Select the smallest pipeline justified by the capability matrix. Preserve the
source, inspect provenance, execute real holon-owned stages, validate final
artifacts, and demonstrate resume after an induced failure. Do not emulate a
missing holon capability inside Flow.

**Exit evidence:** one redistribution-safe fixture completes through compatible
real tools; plan, run, report, logs, provenance, validation, and resume evidence
agree.

## Later — Provenance and authenticity

### Evidence capture

Integrate verified metadata and provenance probes such as ExifTool, MediaInfo,
GPAC, and C2PA tooling behind versioned adapters. Preserve raw and normalized
evidence with privacy-aware retention.

### Detection reporting

Evaluate visible/invisible watermark and synthetic-media detectors for
reliability, licensing, model availability, and structured output. Report method
coverage and uncertainty. Do not infer absence or origin beyond evidence.

### Derivative claims

Add opt-in C2PA claim construction and validation for final derivatives,
including source fingerprints, declared transformations, software identity, and
signing policy. Key custody and signer trust require explicit design.

**Exit evidence:** fixtures prove source-evidence preservation, truthful
derivative claims, validation, redaction, and failure behavior.

## Later — Product and ecosystem maturity

- Pipeline authoring with schema-aware tooling and reusable, signed packs.
- Stable extension contract for third-party holons and adapters.
- Cross-run content-addressed caching with privacy and invalidation guarantees.
- Consistent packages, release notes, compatibility matrices, and independent
  holon versioning within suite releases.
- Richer interfaces only after the CLI/JSON semantics are stable; introduce a
  `DESIGN_SYSTEM.md` when reusable visual language becomes real.

## Maybe

- Distributed or remote execution with explicit data-residency policy.
- Hardware-aware scheduling across multiple local accelerators.
- A trust-policy registry for provenance validators.
- Visual pipeline exploration generated from the same typed plan contract.

These remain possibilities, not commitments, until local correctness and suite
contracts are mature.

## Dependencies and risks

| Risk | Strategic response |
| --- | --- |
| Rust toolchain/version divergence | Decide after inventory; allow coordinated nested workspaces or process adapters |
| History loss during consolidation | Use a reviewed import strategy and archive old repositories only after verification |
| Overlapping orchestration logic | Assign domain versus suite ownership before extraction |
| Schema churn | Version explicitly; test producers and consumers together |
| External tool fragility | Probe capabilities; isolate adapters; keep raw diagnostics |
| Misleading provenance claims | Enforce ADR-0002 and epistemic claim states |
| Monorepo becomes monolith | Enforce independent builds, public APIs, and forbidden sibling dependencies |

## Assumptions and evidence gaps

The current repository scan is sufficient to define intent, not to select the
final workspace or adapter layout. Capability matrices and import rehearsals are
required before implementation commitments.

## Open questions

- What source-import method best preserves history and reviewability?
- Which shared contracts deserve Rust crates versus JSON Schema only?
- How should suite releases express independently versioned holons?

## Validation

Roadmap horizons advance on named evidence, not elapsed time. New initiatives
must align with the purpose, vision, pillars, system ownership, and accepted
decisions.
