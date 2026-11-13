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

## Proposed audio-readiness extension — November 12, 2026

Flow can select an exact package payload/destination and a separately approved
read-only Aniflow policy-check plan. The existing native transport and source /
opaque released authority paths remain in charge of process execution. Aniflow
owns level math and current final-QC truth. Flow correlates the exact returned
subject/configuration/policy digest with copied delivery bytes and existing
review declarations. An explicit archival-master preservation choice does not
assert a target check. Every other assigned media/destination pair is covered.

A captured check is supporting evidence; it cannot replace a pilot media stage
or become current QC through retained status. A new explicit check refreshes
the domain owner. Goal matching does not establish meter/platform qualification,
musical approval or upload authority. Failures retain ordinary native attempts;
there is no new runtime, automatic replay/approval or acceptance database. This
extension remains proposed and source-only, with execution deferred to #88.

## Proposed audio-processing translation — November 12, 2026

Selected fixed gain uses Aniflow's structured native plan and current accepted
handoff through the existing native supervisor. Exact source membership/order,
request and descriptor identity, native intent and artifact roles remain bound.
Nested source/QC/tool namespaces are outside Flow's writable run. Explicit
recovery names the existing run and original admitted native plan. Retained
status reobserves bytes and saved framing without launching the owner or making
historical measurements current. A gained WAV remains an unreviewed derivative
requiring fresh exact-file QC; no limiter, automatic mastering or upload is
introduced. This extension remains proposed and unqualified.

The same translation boundary can select either existing resumable final-file
QC owner. Captured reports must describe the exact selected bytes and native
intent, and agree with the actual accepted report file. Reports remain supporting
evidence with separate selected-policy comparison; neither a WAV result nor a
historical Flow status grants current encoded-video QC. This remains proposed.

A separately selected gain-video route binds original video and adjusted WAV
inputs and their returned lineage. Aniflow retains exact clock/picture/audio
and current WAV-QC admission. Flow assigns only a delivery-derivative role,
irrespective of an inherited native source variant. Fresh encoded-byte QC and
human review remain required. This adds no engine or publication authority and
does not ratify this proposal.

Adjusted delivery selection requires an accepted native derivative, matching
final-file QC and explicit selected-policy snapshot under the same provider
cohort. The original source remains bound to the pilot and an accepted clean
master stays in the package. Exact copied history enters a canonical lineage
identity in the payload's review subject. Portable readback checks the captured
relations and current package bytes without observing original host paths or
refreshing native/release/QC authority. Package copying remains the only effect;
human review, current platform proof and publication remain separate. This
extension remains proposed and unqualified.

## Proposed installed-check preparation extension — November 12, 2026

An operator can resolve supplied Aniflow QC/policy requests against exact delivery
payloads and a selected installed policy-check row. Read-only preview exposes
these bindings. Explicit preparation may copy only configuration bytes and ask
the existing native SDK to construct plans in a fresh private bundle. Supplied
approvals remain a later input; setup cannot create them or execute a provider.
Package copies, original final files, installation/tools and native/QC evidence
must remain outside the writable setup namespace. Retained setup status records
local identity rather than current domain QC. This extension is proposed;
source authoring, installation receipts and preview do not qualify a release.

## Proposed publishing report extension — November 12, 2026

A read-only handoff report can correlate a current exact package with a pinned
audio summary and its retained owner records. It must expose remaining review,
metadata, destination, current-QC, qualification, route and publication actions.
Retained numeric results are historical, and a summary's provider qualification
flag is a declaration in that summary, not fresh authenticated admission. The
report grants neither human review nor execution/upload authority. Package/source
identity and snapshot scope are checked; Aniflow retains all audio math and the
existing native engine retains runtime acceptance. This remains proposed source,
with actual execution/qualification deferred and Flow #88 preserved last.

## Proposed static preset extension — November 12, 2026

Explicit supplied locale/source branches can delegate to installed canonical
Renderflow plans without a second execution engine. Required target IDs are
operator requirements, checked against actual retained native acceptance rather
than guessed from profile names. A locale label is not translation evidence.
Preview and preparation grant no approval or runtime authority; existing native
approval/run/resume owners remain unchanged. Creative/accessibility/rights and
release/publication acceptance remain separate. This extension is proposed.
