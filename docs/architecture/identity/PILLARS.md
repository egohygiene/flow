---
schema: aether.architecture-document/v1
id: flow-pillars
title: Flow Pillars
kind: architecture-document
version: 0.1.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-13
governed_by:
  - architecture-pillars
depends_on:
  - flow-purpose
  - flow-vision
  - flow-principles
related:
  - flow-system
  - flow-roadmap
supersedes: []
---

# Flow Pillars

## Composable holons

Each domain tool is complete alone and predictable in composition. Health is
measured by independent CLI and library tests, stable public contracts, and the
absence of sibling dependencies.

## Deterministic orchestration

The same resolved inputs, definitions, configuration, and toolchain produce the
same plan. Health is measured by reproducible plan digests, explicit capability
resolution, and deterministic ordering.

## Evidence and provenance

Every important claim can be traced to a source, observation, decision, or
transformation. Health is measured by source fingerprints, versioned manifests,
validation reports, and honest derivative claims.

## Recoverable execution

Failures preserve useful state and explain the next safe action. Health is
measured by atomic manifests, compatible checkpoints, bounded cancellation,
partial-run reports, and tested resume behavior.

## Human agency

Users can inspect plans, understand effects, choose retention and remote-service
policy, and refuse unsafe actions. Health is measured by actionable errors,
meaningful previews, no silent fallback, and privacy-safe diagnostics.

## Relationships between pillars

Composition without evidence is opaque; evidence without recovery is brittle;
recovery without deterministic plans is unreliable; and none are valuable if
the user cannot understand or control the outcome.

## Initiative alignment

Every roadmap initiative must strengthen at least one pillar without materially
weakening another. Trade-offs require a recorded decision.

## Open questions

- What compatibility window balances rapid holon evolution with suite stability?

## Validation

Milestones define observable health signals for the pillars they claim to
advance.
