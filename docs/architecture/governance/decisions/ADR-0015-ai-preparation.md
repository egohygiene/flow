---
schema: aether.architecture-document/v1
id: flow-adr-0015
title: Separate local AI preparation bytes from invocation readiness
kind: architecture-document
version: 0.1.0
status: proposed
owners: [egohygiene]
created: 2026-11-10
updated: 2026-11-10
governed_by: [architecture-decisions]
depends_on: [flow-architecture]
related: [flow-decisions, flow-adr-0005, flow-adr-0008, flow-adr-0014]
supersedes: []
---

# ADR-0015: Local AI preparation evidence

Proposed for #80, acceptance pending. Source inclusion or merge does not adopt it.

The current native Renderflow profile declares preparation but does not observe
weight bytes or prove local execution. Define a Flow-owned proposed nonsecret
profile and a separate private deployment/preparation declaration. Read-only
inspection may match explicitly pinned runtime/model/config files, but must
report native model identity, hardware, license review and offline enforcement
as unverified. Matched bytes grant no extension, launch or publication authority.

Reuse existing extension locks, exact subjects, per-invocation effects and secret
handles. No new router, gateway, installer, model format, configuration resolver
or shared dependency is justified. Domain adapters retain their own contracts
and adopt only through reviewed immutable releases. Local inspection refuses
remote execution; missing local subjects never imply fallback or acquisition.

The alternative of treating a loopback URL, catalog alias or supplied pin as
ready would conflate declarations with runtime observations. The alternative of
forcing holons to depend on Flow would violate federation. Bounded explicit
file inspection addresses a narrower gap and preserves those boundaries.

Two bounded reads are ordinary observations, not atomic snapshot/launch binding.
Actual runtime-native identity, selected hardware/resource enforcement, egress
isolation, useful inference and credential/configuration handoff remain open.
Seven source-authored cases and closed schemas are uncompiled/unvalidated;
qualification stays deferred to #88 last. No prior ADR status changes.
