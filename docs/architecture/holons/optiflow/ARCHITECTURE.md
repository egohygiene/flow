---
schema: aether.architecture-document/v1
id: optiflow-architecture
title: Optiflow Architecture
kind: architecture-document
version: 0.1.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-13
governed_by:
  - architecture-architecture
depends_on:
  - optiflow-system
related:
  - flow-architecture
  - optiflow-decisions
supersedes: []
---

# Optiflow Architecture

## Purpose and scope

Define a reusable evidence and planning engine with mutation isolated behind an
explicit, verifiable boundary.

## Structural units

- **Domain:** observations, identity, relationships, opportunities, plans,
  preconditions, effects, and result semantics.
- **Application:** scan, analyze, report, plan, and future guarded-apply use cases.
- **Ports:** filesystem, metadata probe, hashing, persistence, clock, event sink,
  and typed transformation capability.
- **Adapters:** local filesystem, SQLite, ffprobe, suite contracts, and future
  action executors.
- **Delivery:** public library facade and thin CLI.

## Boundary rules

Discovery cannot mutate. Analysis cannot emit actions. Planning cannot execute.
Application cannot proceed on stale or uncontained targets. Persistence models
do not leak as public types. A transformation adapter returns new artifacts; a
separate policy decides what collection action may follow.

## Dependency direction

`optiflow-cli -> optiflow public API -> application -> domain`; adapters
implement domain/application ports. Shared contracts are allowed at the public
edge; no Aniflow or Renderflow dependency is allowed.

## Communication patterns

Use immutable run/report/plan/result documents, transactional local state,
streaming hashes, deterministic ordering, direct structured probes, and atomic
artifact writes.

## Significant constraints

Traversal must handle symlinks, mount boundaries, hidden trees, special files,
concurrent changes, permissions, spaces, and Unicode. Content hashing must be
streaming and staged. Removable/network filesystem behavior cannot assume WAL
semantics or stable device identity.

## Relationship to system inventory

Each subsystem has one directional role from evidence acquisition toward
proposal and, eventually, guarded action. Reverse shortcuts are prohibited.

## Assumptions and evidence gaps

The initial library extraction must preserve v0.1.0 JSON and state compatibility
or provide explicit migrations.

## Open questions

- How should planner policies be versioned and serialized independently from
  executable action implementations?

## Validation

Architecture tests enforce non-mutation dependencies and the thin CLI; contract
tests prove stable plans and refusal on changed targets.
