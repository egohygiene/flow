---
schema: aether.architecture-decision/v1
id: flow-adr-0016
title: Deliver product commands through existing released adapters
kind: architecture-decision
status: proposed
owners: [egohygiene]
scope: [flow]
governed_by: [architecture-decisions]
supersedes: []
superseded_by: []
related: [flow-architecture, flow-adr-0011, flow-adr-0013, flow-adr-0014]
---

# ADR-0016 — Product command delivery

Proposed for #53/#70; source inclusion is not ratification or release acceptance.

Expose public library command behavior through one thin `flow` source binary.
Start with the independently released Optiflow v0.1.1 read-only adapter. Keep its
native command-result/attempt receipt separate from generic extension JSONL and
durable graph checkpoints. A closed operation plan names exact source scope,
provider byte pins, destination and fixed read-only effects. An explicit supplied
operator declaration permits only that planned local operation; it grants no
Optiflow mutation/review-plan approval or additional process authority. Existing
adapter pins, evidence checks and direct process ownership remain authoritative.

Doctor/inspect/plan do not launch providers. Run freshly rechecks the operation
plan, stores its supplied declaration before launch and delegates the existing
adapter. Status reopens current native receipt evidence. Recovery describes an
interrupted/refused attempt without automatically repeating it or fabricating a
generic graph checkpoint. Cancellation extends the existing native capture,
not a second process runtime. The binary owns signal presentation; libraries
accept caller-owned cancellation and install no global handlers.

No implicit install, download, provider substitution, cloud, source mutation,
signing or posting. Missing immutable Aniflow/Renderflow routes remain unavailable.
One bounded machine result, stable exit categories and content-free diagnostics
remain separate from private local evidence. Full clean-install, provider,
cross-platform, release and dependency criteria in #53/#70 are unchanged. All
new cases are authored/uncompiled/unrun; Flow #88 stays last.
