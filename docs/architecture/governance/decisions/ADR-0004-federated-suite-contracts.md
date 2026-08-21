---
schema: aether.architecture-decision/v1
id: adr-0004
title: Keep holons federated behind versioned contracts
kind: architecture-decision
status: accepted
accepted: 2026-08-21
owners:
  - egohygiene
scope:
  - flow
  - aniflow
  - optiflow
  - renderflow
governed_by:
  - architecture-decisions
supersedes:
  - adr-0001
superseded_by: []
related:
  - flow-architecture
  - flow-roadmap
---

# ADR-0004 — Keep holons federated behind versioned contracts

## Context

ADR-0001 proposed importing Aniflow, Optiflow, and Renderflow into this
repository. The organization architecture now defines each as an independently
coherent repository and assigns Flow only cross-holon orchestration. Copying
source would duplicate ownership, history, release identity, and truth.

Flow #2 also needs a concrete dependency policy, versioned initial contracts,
and one first slice before implementation begins.

## Decision

Aniflow, Optiflow, and Renderflow remain independent repositories and releases.
Flow composes a named release through either its stable public library or a
versioned CLI contract. Flow owns adapters and suite interchange contracts.

The allowed dependency directions are:

- a holon CLI may depend on its own library;
- a Flow library adapter may depend on a released holon library;
- a Flow process adapter may invoke a versioned holon CLI;
- Flow core may depend on Flow adapter ports and Flow-owned contracts; and
- no holon may depend on Flow or a sibling holon.

Copied sibling source, path dependencies into sibling checkouts, Git submodules,
and mutable branch dependencies are forbidden. Holon-native contracts remain
producer-owned; Flow adapters translate them into Flow contracts.

The first vertical slice is **restore-and-assess**: Optiflow performs read-only
collection evidence before and after Aniflow creates a new temporal master.
Flow owns planning, compatibility, run state, provenance coordination,
validation coordination, failure handling, and resume. Renderflow is not part of
the first slice.

## Rationale

Federation preserves one source of truth per capability and allows providers to
release on their actual toolchains. Versioned adapters make integration cost
visible and testable. The chosen slice proves two-provider composition without
misrepresenting Optiflow v0.1's read-only boundary or inventing a missing
Renderflow capability.

## Alternatives considered

- One monorepo: rejected because it duplicates canonical repositories and
  couples history, release, and toolchain decisions.
- Submodules or path dependencies: rejected because they introduce mutable
  checkout state and fragile local topology.
- CLI-only composition: safe but unnecessarily excludes stable public libraries.
- A three-provider first slice: rejected because it adds an unrelated seam
  before the orchestration core proves recovery and compatibility.

## Trade-offs

Cross-repository compatibility testing and coordinated contract changes require
more explicit release discipline. In return, ownership, rollback, provenance,
and independently useful provider releases remain clear.

## Expected consequences

Flow adds contracts, fixtures, and adapters rather than provider implementation.
Provider changes land in their owning repositories. Unsupported versions fail
actionably, with no silent fallback to private or copied code.

## Observed outcomes

The v1 artifact, capability, and compatibility contracts and first-slice plan
are recorded with this decision. Implementation evidence is still pending.

## Review triggers

Review if a required capability cannot be composed safely through any released
library or CLI interface, or if organization architecture changes repository
ownership.

## Related artifacts

`flow-system`, `flow-architecture`, `flow-roadmap`, and
`docs/integrations/suite-boundaries.md`.

## Validation

Architecture checks, contract validation, compatibility fixtures, and dependency
audits must enforce this decision.
