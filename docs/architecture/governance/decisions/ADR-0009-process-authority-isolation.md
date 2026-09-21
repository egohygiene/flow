---
schema: aether.architecture-decision/v1
id: adr-0009
title: Bind process authority to explicit isolation evidence
kind: architecture-decision
status: accepted
accepted: 2026-09-21
owners:
  - egohygiene
scope:
  - flow
  - aniflow
  - optiflow
  - renderflow
governed_by:
  - architecture-decisions
supersedes: []
superseded_by: []
related:
  - flow-architecture
  - flow-roadmap
  - adr-0005
  - adr-0006
  - adr-0008
---

# ADR-0009 — Bind process authority to explicit isolation evidence

## Context

ADR-0005 separates provider permission requests from operator grants. ADR-0006
defines a bounded host-neutral process transcript, and ADR-0008 requires exact
package and executable bytes before that seam can be used. Those decisions do
not yet define the concrete argv, secret handles, resource allowlists, or
isolation posture selected for one invocation.

A broad manifest permission or operator grant is not a launch profile. A trust
label is not an operating-system sandbox. Likewise, a digest match identifies
content but cannot prove publisher identity, signature verification,
transparency inclusion, or runtime containment. Flow needs a fail-closed
preflight that preserves those distinctions before later runner work begins.

Flow issue #40 is the bounded FLO-3.2d child of parent issue #25. Process launch
and supervision, durable state, and real provider adapters remain later roadmap
work.

## Decision

Flow adds two provisional closed contracts and one opaque runtime proof:

1. `flow.process-authority-profile/v1` records the exact requested and granted
   authority plus operator trust and selected isolation for one process
   invocation.
2. `flow.process-enforcement-evidence/v1` records a caller-supplied statement
   about the host backend and exact authority it reports enforcing.
3. `AuthorizedProcess` is constructed only after the resolution, invocation,
   matched execution subjects, profile, and evidence agree exactly.

The authority vocabulary is closed over ordered argv; environment names with
opaque `secret:` handles; filesystem reads and writes; network endpoints;
subprocesses; AI providers; GPU devices and capabilities; source-mutation
targets; destructive operations; publication destinations; signing-key
handles; and telemetry fields and targets. Set-like collections are canonical,
strictly sorted, and duplicate-free. Empty collections deny the corresponding
dimension, wildcard-only authority is invalid, and v1 ambient authority is
always `deny-unlisted`.

Manifest list permissions must equal the profile request. Broad manifest
booleans must agree with whether the request names a concrete authority. The
operator grant must cover every request and cannot widen ordered argv. The
profile binds the extension-lock ID, caller-issued authorization ID and grants
digest, run/invocation, provider identity, capability, process interface,
subject-lock identity, operator trust, isolation, request, and grant.

V1 defines two isolation profiles:

- `trusted-unconfined` is eligible only for operator-`trusted` providers. Its
  evidence must explicitly state source `none`, backend `none` version `0.0.0`,
  empty enforced authority, and `not-enforced` for every dimension.
- `sandboxed` is eligible for operator-`trusted` or operator-`sandboxed`
  providers. Its evidence must state `caller-attested-host`, name a versioned
  non-`none` backend, reproduce the exact requested authority as enforced, and
  report every dimension `enforced`.

A sandbox-required process candidate may resolve so that the downstream
preflight can assess its profile. A sandbox-required in-process candidate
remains blocked because the injected seam is unconfined. Disabled candidates
remain ineligible.

Both contracts use `flow.canonical-json/v1` and lowercase SHA-256 identities.
The evidence binds the profile's canonical digest and the fresh execution-
subject observation digest. The opaque token retains both canonical identities
and all correlated context. Process request encoding and transcript validation
require both `MatchedExecutionSubjects` and `AuthorizedProcess`; stale or
mismatched tokens fail before request or transcript acceptance.

The enforcement evidence is explicitly caller-attested. Flow validates its
closed shape, completeness, and correlation but does not authenticate the
caller or prove that an OS sandbox applied the reported policy. The empty
`unsupported_claims` array reserves a visible versioning point; v1 rejects
signature, transparency-log, attestation, authenticated-host, or other stronger
claims it cannot verify.

## Rationale

An exact per-invocation profile makes ambient assumptions inspectable and gives
a future runner a deterministic input. Keeping requested authority separate
from operator grants preserves the ADR-0005 authority split. Keeping both
separate from enforcement evidence prevents intended policy from being reported
as observed behavior.

Requiring all authority dimensions in canonical order makes omissions visible.
Requiring sandbox evidence to equal the request exactly rejects both incomplete
containment and unexplained widening. Explicit `not-enforced` evidence for the
unconfined profile avoids implying security properties merely because an empty
field was omitted.

The opaque token prevents portable documents alone from bypassing the Flow
preflight API and binds the authority decision to the exact already-observed
package and executable bytes.

## Evidence and assumptions

- Extension resolution has already validated the manifest and operator lock.
- `MatchedExecutionSubjects` already proves exact equality to the supplied
  package/executable lock for this invocation.
- The caller issues the authorization ID and grants digest and supplies the
  exact profile approved by operator policy.
- A future runner will consume the profile and produce host evidence without
  weakening its dimensions.
- This checkpoint trusts neither a provider result nor a digest match as proof
  of runtime containment or publisher authenticity.

## Alternatives considered

- **Treat manifest permissions as the launch policy:** rejected because boolean
  capabilities and broad resource classes do not identify concrete per-run
  argv, handles, devices, targets, or destinations.
- **Treat operator trust as proof of isolation:** rejected because policy
  selection and host enforcement are different claims.
- **Permit sandboxed providers to use the injected in-process seam:** rejected
  because the seam has no enforceable containment boundary.
- **Let process transport accept profiles without an opaque token:** rejected
  because arbitrary deserialized documents could bypass exact correlation.
- **Accept partial sandbox guarantees:** rejected because an omitted dimension
  could silently become ambient authority.
- **Call digest equality authenticity:** rejected because content identity does
  not verify a publisher, signature, or transparency log.
- **Implement a platform sandbox in the same change:** deferred because host-
  specific launch, lifecycle, and authenticated evidence require a separate
  adapter boundary and conformance matrix.

## Trade-offs

The contracts are verbose because every dimension and status is explicit.
Adding a new authority dimension or stronger evidence source requires a new
contract version rather than an unreviewed optional field.

The current token can prove only that the caller supplied a self-consistent
host statement. It cannot prove the statement true. That limitation is made
visible in the source vocabulary and public documentation rather than hidden
behind a generic `verified` boolean.

Manifest boolean permissions still require the per-invocation profile to name
concrete targets. The caller-issued grants digest is correlated but not
recomputed or authenticated by this library checkpoint.

## Expected consequences

- Process evidence cannot cross either Flow seam without exact requested and
  granted authority plus an explicit isolation posture.
- Sandbox-required process candidates can advance to a fail-closed downstream
  gate instead of being permanently excluded at catalog resolution.
- Unconfined in-process execution remains restricted to trusted providers.
- Duplicate, wildcard, contradictory, stale, unsupported, under-enforced, and
  overbroad evidence fails deterministically.
- A later runner gains a closed input/evidence vocabulary without this change
  claiming that such a runner already exists.

## Observed outcomes

Flow issue #40 adds the contracts, canonical identities, opaque preflight token,
catalog trust routing, process-seam requirements, synthetic examples, portable
fixtures, and positive/adversarial Rust tests. It does not add a child-process
runner, operating-system sandbox, authenticated backend, durable state, or real
provider adapter. Pull-request and default-branch CI are the acceptance evidence
for the implementation.

## Security, privacy, and authority

Environment values and signing keys never enter the contracts; only opaque
handles do. Resource names, endpoints, package identity, and policy may still
be sensitive and must be handled accordingly.

`AuthorizedProcess` is not an OS capability. Caller-attested evidence must not
be exposed as independently verified containment. Content-digest observation,
cryptographic verification, publisher identity, operator trust, and host
enforcement remain separate evidence and claims.

## Review triggers

Review this decision if:

- Flow ships a real runner or platform sandbox;
- host evidence becomes authenticated or remotely attested;
- manifest permissions gain exact resource selectors;
- a new authority dimension is required;
- in-process isolation becomes enforceable;
- authorization issuance or grants-digest verification moves into Flow;
- launch-time binding closes the observed-file race; or
- signature or transparency-log verification is implemented.

## Related artifacts

ADR-0005, ADR-0006, ADR-0008, `flow-architecture`, `flow-roadmap`,
`docs/integrations/authority-isolation.md`,
`docs/integrations/process-transport.md`, Flow issue #25, Flow issue #40, and
Flow issue #11.

## Validation

Conformance must cover both isolation profiles, every authority dimension,
canonical identity, wrong provider/version/publisher/capability/interface,
wrong authorization/run/invocation/subject/profile/evidence identity, stale
tokens, authority outside operator grants, ungranted requests, wildcard and
duplicate values, mismatched secret handles, incomplete or extra sandbox
authority, missing/duplicate/reordered/contradictory guarantees, unsupported
claims, and closed unknown fields. Every rejected case must return a typed error
and never an `AuthorizedProcess`, encoded request, or `ValidatedExecution`.
