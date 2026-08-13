---
schema: aether.architecture-document/v1
id: optiflow-system
title: Optiflow System
kind: architecture-document
version: 0.1.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-13
governed_by:
  - architecture-system
depends_on:
  - flow-foundations
  - optiflow-ontology
related:
  - optiflow-architecture
supersedes: []
---

# Optiflow System

## Purpose and scope

Optiflow decomposes collection knowledge and change safety into independently
testable subsystems.

## System inventory

| Subsystem | Owns |
| --- | --- |
| Discovery | root containment, traversal policy, path observations, content classification |
| Inspection | filesystem and optional media metadata observations |
| Evidence store | local durable observations, analysis cache, schema migration |
| Identity and relationships | staged hashing, byte identity, qualified similarity evidence |
| Opportunity analysis | reclaimable storage and normalization/optimization opportunities |
| Planner | deterministic immutable proposals, selected policy, effects, preconditions |
| Guarded application | future revalidation, typed actions, atomicity/recovery evidence |
| Reporting | human and versioned structured run/report/plan/result output |

## Responsibilities and capability ownership

Optiflow owns collection-scale policy and safety. Codecs or transformations may
be delegated through Flow to a suitable holon; Optiflow determines the collection
intent and verifies collection-level effects, not the transform algorithm.

## System boundaries

The evidence database is an internal model, not the public contract. Read-only
commands remain separable from future mutation. External filesystems and probes
are untrusted systems. Flow consumes public operations and structured results.

## Major interactions and runtime flows

Discover → inspect → persist observations → analyze relationships/opportunities
→ report or plan. A future apply path loads a plan, resolves exact targets,
revalidates every precondition, previews effects, performs typed actions, and
records results/recovery evidence.

## External system relationships

Local/removable filesystems, SQLite, and optional ffprobe are current systems.
Future codec/normalization execution must cross a typed capability boundary.

## Assumptions and evidence gaps

Current v0.1.0 supports scan, report, exact-duplicate planning, and cache status
without apply/delete/replace/move/quarantine/optimization commands.

## Open questions

- Should mutation live in the same crate behind strict capability gates or a
  separately released application crate?

## Validation

Tests cover path containment, symlinks, filesystem boundaries, Unicode, stale
state, deterministic plans, hashing, atomic writes, and non-mutation guarantees.
