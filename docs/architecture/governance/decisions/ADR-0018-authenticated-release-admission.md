---
schema: aether.architecture-decision/v1
id: flow-adr-0018
title: Admit independently authenticated released native rows
kind: architecture-decision
status: proposed
owners: [egohygiene]
scope: [flow]
governed_by: [architecture-decisions]
supersedes: []
superseded_by: []
related: [flow-architecture, flow-adr-0010, flow-adr-0015, flow-adr-0017]
---

# ADR-0018: Admit independently authenticated released native rows

Status: **Proposed**. Date: 2026-11-11. Relates to #51, #52, #54, #71, #72.

Release declarations and copied files do not prove publisher identity or positive
native qualification. Preserve the existing evidence port and source-candidate
authority while adding one concrete authenticated consumer.

Host configuration selects a pinned verifier and signer/source policy separately
from untrusted evidence/pipeline requests. An explicit operation may invoke the
pinned GitHub attestation verifier and read actual release asset metadata. Positive
clean-install and native/domain evidence must bind the exact row, archive/member,
contracts, dependencies, current host identity and bounded freshness. Serialized
verification summaries cannot construct the opaque host evidence implementation.

Aniflow/Renderflow Flow routes reuse current native status and acceptance checks;
standalone/library interfaces consume the independent owner's authenticated native
gate. No sibling dependency, domain algorithm fork or second supervisor is added.
Released approval/run/status/resume require opaque evidence again. Candidate
approval semantics remain intact; no false candidate acknowledgement is generated.
Release admission wraps the native completion receipt rather than changing the
meaning of historical receipt fields. Creative and publication authority remain
separate. Missing targets and qualification remain in the coverage inventory.

Costs and limits: the adapter trusts the explicitly selected verifier, signer and
host configuration; it is not a defense against a compromised host. Verification
may retrieve trust roots and release metadata, so it is never implicit in doctor
or plan. Recent native evidence is not continuous GPU/service health. Report
producer conformance, real attestations, native host gates and interoperability
testing remain unexecuted in this source checkpoint. Existing qualification issues
and final #88 remain open. Source implementation or merge does not ratify this ADR.
