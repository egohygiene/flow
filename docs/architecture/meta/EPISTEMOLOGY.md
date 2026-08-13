---
schema: aether.architecture-document/v1
id: flow-epistemology
title: Flow Epistemology
kind: architecture-document
version: 0.1.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-13
governed_by:
  - architecture-epistemology
depends_on:
  - flow-purpose
  - flow-principles
related:
  - flow-ai-constitution
  - flow-decisions
supersedes: []
---

# Flow Epistemology

## Scope

This document governs how Flow represents knowledge about repositories, tools,
media, transformations, provenance, compatibility, and validation.

## Claim states

| State | Meaning | Example |
| --- | --- | --- |
| Observed | Directly produced by an inspected source or tool | A file hash or probed stream list |
| Declared | Asserted by a person, configuration, manifest, or tool | A pipeline's intended output |
| Inferred | Derived from evidence through a documented rule | Two files are likely visual variants |
| Verified | Tested against an explicit contract | An output satisfies required streams and tolerances |
| Disputed | Credible evidence conflicts | Container metadata and decoded duration disagree |
| Unknown | Required evidence is missing | Invisible watermark status when no reliable detector exists |

## Evidence and source evaluation

Prefer primary evidence: bytes, cryptographic digests, tool versions, decoded
media probes, embedded manifests, signed claims, structured tool results, and
repository revisions. Secondary documentation may explain behavior but must not
replace runtime verification where correctness depends on the environment.

## Provenance

Provenance has two layers. Evidence provenance records where a claim came from.
Content provenance records how an artifact relates to sources and transforms.
Flow preserves discovered source evidence verbatim when practical and stores
normalized interpretations separately. New claims identify the derivative,
source fingerprint, declared actions, software versions, and validation result.

## Confidence and uncertainty

Cryptographic identity and valid signatures can support high confidence in
narrow claims. Heuristic watermark detection, perceptual similarity, and model
attribution remain probabilistic and must expose method, version, threshold, and
limitations. Absence of detection is not proof of absence.

## Conflict resolution

Preserve conflicting observations, prefer evidence closest to the bytes, test
when possible, and record the rule used to choose an operational value. Never
discard signed or embedded source evidence merely because a later tool reports
a different interpretation.

## Canonical working knowledge

Architecture documents govern intended boundaries. Capability matrices govern
verified integration facts at named revisions. Run manifests govern one resolved
execution. Implementation cannot silently redefine any of these.

## Revision and deprecation

Claims are appendable and supersedable, not silently rewritten. Schema versions
and decision lineage make interpretation changes visible.

## Examples

- `ffprobe` reporting one audio stream is observed evidence.
- A detector score above its threshold is an inference, not proof of origin.
- A valid C2PA claim verifies claim integrity and signer data; trust in the
  signer remains a policy decision.

## Open questions

- Which trust policies should ship for C2PA validation?
- Which evidence formats can be retained without leaking private metadata?

## Validation

Reports and manifests distinguish claim state, evidence source, method version,
confidence, and unresolved conflict.
