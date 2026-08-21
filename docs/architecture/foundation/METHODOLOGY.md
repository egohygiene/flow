---
schema: aether.architecture-document/v1
id: flow-methodology
title: Flow Methodology
kind: architecture-document
version: 0.2.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-21
governed_by:
  - architecture-methodology
depends_on:
  - flow-foundations
  - flow-architecture
related:
  - flow-decisions
  - flow-roadmap
supersedes: []
---

# Flow Methodology

## Purpose and scope

Flow evolves through an evidence-first loop that keeps architecture, contracts,
implementation, and observed behavior synchronized.

## Working method

1. **Observe reality.** Inspect default branches, released interfaces, schemas,
   tests, runtime dependencies, and representative runs at named revisions.
2. **Define boundaries.** Update architecture and decisions before moving or
   generalizing ownership.
3. **Specify contracts.** Write versioned schemas, public API semantics, error
   categories, invariants, and acceptance evidence.
4. **Plan a vertical slice.** Choose the smallest end-to-end outcome that tests
   real boundaries without speculative infrastructure.
5. **Implement inside-out.** Build domain/core behavior, then adapters, then CLIs
   and suite composition.
6. **Validate in layers.** Unit, schema, contract, integration, fixture, and
   opt-in real-tool tests each prove a different claim.
7. **Reflect and propagate.** Compare outcomes with intent, record decisions,
   repair architecture drift, and feed verified gaps into the roadmap.

## Validation loops

- Every holon proves independent library and CLI use.
- Shared contracts are tested by producers and consumers.
- Flow planning is deterministic under shuffled discovery and filesystem order.
- Resume tests change one compatibility input at a time and verify invalidation.
- Real media/document fixtures verify external integrations without making
  heavyweight tests mandatory for every change.
- Documentation examples and help/JSON snapshots are executable where practical.

## Feedback and improvement

Failures become classified evidence: contract gap, compatibility gap, domain
bug, orchestration bug, external dependency, invalid input, or documentation
drift. Repeated local work is generalized only after at least one real use shows
the shared abstraction.

## Human and AI collaboration patterns

AI may accelerate inventory, drafting, implementation, and verification, but it
must cite inspected revisions, label inference, avoid widening scope, and leave
reviewable commits. Humans retain authority over boundary changes, destructive
behavior, remote processing, provenance policy, and release acceptance.

## Boundaries and exclusions

This methodology is not a sprint plan, release checklist, coding style guide, or
permission to rewrite every holon in one change. Large changes are divided
by independently reviewable architectural outcomes.

## Assumptions and evidence gaps

The suite has not yet completed an integrated release. The methodology will be
revised after the first released-adapter vertical slice.

## Open questions

- What test fixture corpus can be redistribution-safe while covering stream,
  provenance, Unicode-path, interruption, and multi-output edge cases?

## Validation

Pull requests name the contract or roadmap outcome they advance and include the
commands/evidence that support completion.
