---
schema: aether.architecture-document/v1
id: aniflow-system
title: Aniflow System
kind: architecture-document
version: 0.2.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-21
governed_by:
  - architecture-system
depends_on:
  - flow-foundations
  - aniflow-ontology
related:
  - aniflow-architecture
supersedes: []
---

# Aniflow System

## Purpose and scope

Aniflow is one domain holon with subsystems for temporal modeling, processing,
reconstruction, and run evidence.

## System inventory

| Subsystem | Owns |
| --- | --- |
| Source inspection | media probing, stream/timing observations, support checks |
| Temporal model | frame/sample order, rate/time-base representation, continuity invariants |
| Decomposition | frame, audio, subtitle, and metadata extraction plans/results |
| Processor runtime | ordered typed frame/audio/video processors and safe external adapters |
| Reconstruction | stream selection, assembly, encoding intent, master production |
| Temporal validation | counts, sequence, integrity, dimensions, timing, streams, decodability |
| Run evidence | domain-stage state, logs, checkpoints, pipeline snapshot, delivery metadata |

## Responsibilities and capability ownership

Aniflow owns temporal video stages and domain validation. External algorithms
such as upscalers or authorized restorers own their specialized transformation;
Aniflow owns their ordering, media handoff, and result verification.

## System boundaries

Flow may invoke Aniflow through its public library or CLI. Aniflow may emit
native artifact/events/results but cannot select Optiflow or Renderflow. A former
optional Renderflow handoff is treated as a compatibility seam to remove or move
into a Flow-owned adapter without copying Aniflow source.

## Major interactions and runtime flows

Inspect → derive timeline → decompose → process ordered components → validate
components → reconstruct → validate master → emit evidence.

## External system relationships

FFmpeg/ffprobe are foundational adapters. Upscayl, restoration models, subtitle
tools, and other processors are optional capabilities subject to discovery,
version, authorization, and output validation.

## Assumptions and evidence gaps

Current behavior extracts PNG frames and PCM audio, processes the first video and
audio streams, and targets CFR reconstruction. The target model must be designed
from tests rather than assumed from current paths.

## Open questions

- Should domain run evidence be directly suite-compatible or adapted at the Flow
  boundary?

## Validation

Subsystem tests prove ownership, and integration tests cover timing/stream edge
cases plus interruption and resume.
