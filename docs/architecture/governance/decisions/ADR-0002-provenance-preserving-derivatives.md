---
schema: aether.architecture-decision/v1
id: adr-0002
title: Preserve source evidence and issue honest derivative claims
kind: architecture-decision
status: accepted
accepted: 2026-08-13
owners:
  - egohygiene
scope:
  - flow
  - provenance
governed_by:
  - architecture-decisions
supersedes: []
superseded_by: []
related:
  - flow-epistemology
---

# ADR-0002 — Preserve source evidence and issue honest derivative claims

## Context

Sources may contain EXIF/container metadata, C2PA claims, signatures, visible or
invisible watermark signals, and tool-specific provenance. Processing often
normalizes or removes metadata as a side effect, while outputs need an honest
account of their new history.

## Decision

Before transformation, Flow records available source metadata and provenance
evidence and fingerprints the original bytes. Processing uses a distinct working
derivative. When supported and explicitly configured, the final derivative may
receive a new C2PA claim referencing the source fingerprint and declared
transformations. Original claims are preserved as evidence and are not copied in
a way that implies the derivative is unchanged.

Invisible-watermark integrations begin as detection and reporting capabilities.
Absence of detection is represented as unknown within detector coverage, not as
proof of absence. Flow does not make provenance bypass or concealment a suite
capability.

## Rationale

This keeps inspection evidence useful, prevents false continuity claims, and
separates authorized content editing from provenance deception.

## Evidence and assumptions

Candidate tools include ExifTool, MediaInfo, GPAC, `c2patool`, and `c2pa-rs`.
Their exact capability, output stability, licensing, and trust policy require a
versioned capability matrix before implementation.

## Alternatives considered

- Strip all metadata and start fresh: simple, but destroys evidence.
- Copy source claims unchanged: may falsely describe transformed bytes.
- Block any processing of claimed content: overbroad and not required for honest
  derivative workflows.

## Trade-offs

Evidence storage increases artifact size and privacy review surface. Signing
introduces identity, key custody, trust-store, and reproducibility concerns.

## Expected consequences

Provenance inspection, normalization, claim construction, validation, and
redaction become separate typed stages with distinct evidence.

## Observed outcomes

None yet.

## Review triggers

Review when C2PA standards, supported trust models, legal requirements, or
detector reliability materially change.

## Related artifacts

`flow-epistemology`, `flow-foundations`, and `flow-roadmap`.

## Validation

Fixtures must prove original preservation, evidence retention, derivative
fingerprinting, claim validation, and truthful handling of unknown watermark
status.
