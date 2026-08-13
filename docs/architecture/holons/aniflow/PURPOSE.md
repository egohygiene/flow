---
schema: aether.architecture-document/v1
id: aniflow-purpose
title: Aniflow Purpose
kind: architecture-document
version: 0.1.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-13
governed_by:
  - architecture-purpose
depends_on:
  - flow-purpose
related:
  - aniflow-system
supersedes: []
---

# Aniflow Purpose

## Purpose statement

Aniflow exists to make time-based video processing reproducible, resumable, and
verifiable while allowing specialized processors to retain their own algorithms.

## Need

Frame-oriented work requires precise stream inspection, deterministic extraction
and ordering, processor isolation, temporal continuity, audio/subtitle handling,
reconstruction, and recovery. Generic transform graphs do not by themselves own
these timeline invariants.

## Beneficiaries

Creators, restorers, animation/video engineers, and automation systems that need
repeatable temporal pipelines and inspectable masters.

## Enduring value

A stable temporal engine lets new processors participate without each rebuilding
video decomposition, checkpoints, validation, and assembly.

## Scope boundaries

Aniflow owns one video's temporal processing lifecycle. It does not inventory or
deduplicate collections, choose collection optimization policy, render general
documents, orchestrate sibling holons, or make claims beyond its observed run.

## Assumptions

FFmpeg-compatible tooling remains a primary media boundary, while processors may
be libraries or isolated external commands. Current v0.2.0 CFR/first-stream
limits are implementation constraints, not enduring domain truths.

## Open questions

- Which timeline model will support VFR, multiple streams, and segment-level
  invalidation without destabilizing the simple path?

## Validation

Aniflow remains independently usable through a Rust library and CLI, and its
outputs carry enough timing and stage evidence for external validation.
