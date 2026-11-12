---
schema: aether.architecture-decision/v1
id: flow-adr-0019
title: Separate publisher trust, runtime readiness and delivery review
kind: architecture-decision
status: proposed
owners: [egohygiene]
scope: [flow]
governed_by: [architecture-decisions]
supersedes: []
superseded_by: []
related: [flow-architecture, flow-adr-0017, flow-adr-0018]
---

# ADR-0019: Separate publisher trust, runtime readiness and delivery review

Status: **Proposed**. Date: 2026-11-11. Relates to #54, #70, #71, #72, #75, #78.

A clean installation cannot require a previously installed native witness. Add a
portable publisher report and a separate opaque publisher verifier that shares
the fixed signer/source/publication protocol while granting only exact file-copy
authority. Each owner assembles and attests its own reports. Historical install
admission is retained in the existing receipt and never becomes native execution
authority through deserialized data.

Native approval binds the exact plan and installed/runtime scope. Current positive
proofs must still authenticate and meet freshness policy, but renewed evidence for
identical cohort/policy/verifier, host, executable/contracts and requirement-file
identities may refresh readiness without changing approved effects. Audits pin
each exact proof request and capture set. Scope changes invalidate authority;
old audits remain retained and stale proof is never accepted as a passive cache.
This refines the source-only admission implementation, not an accepted public
release protocol or previously executed qualification.

One publishing project resolves installed interfaces and composes the existing
native/pilot SDK. A separate local package copies selected accepted outputs,
preserves the native-owned clean-master identity, and binds supplied operator
review declarations to exact payload/metadata identity. Byte/evidence retention,
provider qualification, human content review, current platform preflight and
publishing authority remain distinct. Exact-plan read-only receipt observation
can support either retained authority mode without granting run/resume authority.
No new process engine, sibling dependency, media algorithm or automatic upload
is introduced.

Limits: host trust and owner attestations remain trusted configuration; this is
not protection against a compromised host. Current platform/native/interoperability
execution, human reviews and all actual signatures/releases remain unexecuted.
Existing release/qualification owners and final #88 remain open. Source or merge
does not ratify this proposed decision.
