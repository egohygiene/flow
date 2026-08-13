---
schema: aether.architecture-document/v1
id: aniflow-architecture
title: Aniflow Architecture
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
  - aniflow-system
related:
  - flow-architecture
  - aniflow-decisions
supersedes: []
---

# Aniflow Architecture

## Purpose and scope

Define the internal dependency direction required for Aniflow to serve both
library callers and its CLI without leaking temporal logic into delivery code.

## Structural units

- **Domain:** timeline, streams, components, processor/stage contracts,
  continuity rules, and domain errors.
- **Application:** inspection, decomposition, processing, reconstruction,
  validation, checkpoint decisions, and use-case coordination.
- **Ports:** media probe/decode/encode, processor execution, clock, filesystem,
  hashing, and event sinks.
- **Adapters:** FFmpeg/ffprobe and optional processor integrations.
- **Delivery:** public library facade and thin standalone CLI.

## Boundary rules

Domain types do not depend on CLI, process, or filesystem implementations.
Application code invokes typed ports. External commands use direct argv and
declare their inputs/outputs. Pipeline configuration maps into validated domain
intent before execution. The CLI delegates all meaningful behavior to the
library.

## Dependency direction

`aniflow-cli -> aniflow public API -> application -> domain`, with adapters
implementing inward-facing ports. Shared suite contracts may be used at the
public boundary but cannot replace domain types where temporal semantics matter.

## Communication patterns

Use typed requests/results, a domain event stream, immutable stage directories,
atomic checkpoints, and structured delivery metadata. Raw external logs remain
available separately from interpreted events.

## Significant constraints

Large frame sets require bounded concurrency and storage-aware execution.
Cancellation must terminate child process trees safely. Checkpoints include
source/timeline identity, processor version/configuration, input digests, and
validated output observations.

## Relationship to system inventory

The application layer coordinates the subsystems named in `SYSTEM.md`; adapters
do not become alternate owners of temporal behavior.

## Assumptions and evidence gaps

Exact crate decomposition and whether pipeline/run types remain Aniflow-specific
are deferred to the extraction plan.

## Open questions

- Separate core, FFmpeg adapter, processor SDK, and CLI crates, or a smaller
  library/CLI split initially?

## Validation

Architecture tests enforce forbidden dependencies; library examples execute a
synthetic temporal pipeline without invoking the CLI parser.
