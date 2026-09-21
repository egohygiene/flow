# Process authority and isolation profiles

## Purpose and scope

This profile defines the exact authority selected for one process invocation and
the host-enforcement evidence that must accompany it before Flow encodes a
request or accepts a process transcript. It composes extension resolution,
operator authorization, and locked execution-subject observation without
collapsing those concerns into one trust claim.

The profile is governed by
[ADR-0009](../architecture/governance/decisions/ADR-0009-process-authority-isolation.md)
and depends on the exact package/executable boundary in
[ADR-0008](../architecture/governance/decisions/ADR-0008-locked-execution-subjects.md).

This checkpoint validates contracts and correlation. It does not start a
process, configure an operating-system sandbox, authenticate the evidence
source, or prove that the reported host enforcement occurred.

## Contracts and runtime proof

| Artifact | Owner | Meaning |
| --- | --- | --- |
| Extension manifest permissions | Provider | Broad authority requested by the provider package |
| Extension-lock grants and trust | Flow operator | Maximum allowed authority and whether unconfined execution is permitted |
| `flow.process-authority-profile/v1` | Flow caller/operator boundary | Exact authority and isolation selected for one invocation |
| `flow.process-enforcement-evidence/v1` | Future runner/host | Closed statement about which profile a named backend reports enforcing |
| `AuthorizedProcess` | Flow runtime | Opaque proof that the resolution, invocation, execution subjects, profile, and evidence correlate |

Portable JSON remains inspectable evidence. Deserializing either document
cannot construct `AuthorizedProcess`; only `authorize_process` can return that
token after every validation and correlation gate succeeds.

## Exact authority dimensions

`ProcessAuthority` represents every v1 process authority dimension explicitly:

| Dimension | Representation | Rule |
| --- | --- | --- |
| Arguments | Ordered `argv` tail | Grant must equal the request exactly; no added arguments |
| Environment | Variable name plus opaque `secret:` source handle | Invocation secret handles must equal the requested handles; secret values never enter portable evidence |
| Filesystem | Separate read and write allowlists | Empty denies the dimension |
| Network | Endpoint allowlist | Empty denies network authority |
| Subprocess | Direct subprocess allowlist | Empty denies child-process authority |
| AI | Provider allowlist | Empty denies external or local AI-provider authority |
| GPU | Device IDs plus closed capability sets | Empty denies GPU access |
| Source mutation | Target allowlist | Empty denies mutation of source artifacts |
| Destructive operations | Operation allowlist | Empty denies destructive authority |
| Publication | Destination allowlist | Empty denies publication |
| Signing | Opaque key-handle allowlist | Key material never enters the profile |
| Telemetry | Closed field set plus target allowlist | Empty denies telemetry propagation/export |

Every set-like collection is strictly sorted and duplicate-free. Empty,
control-containing, and wildcard-only values are invalid. The ambient policy is
the single closed v1 value `deny-unlisted`; omission never means ambient access.

The manifest's list-valued permission requests must match the profile request
exactly. A manifest boolean such as `gpu`, `publish`, or `sign` must correspond
to a nonempty concrete request when true and an empty request when false. The
profile supplies the exact device, destination, operation, or handle because
the broad manifest boolean does not. Ordered argv and telemetry selections are
also invocation-specific profile data. The operator grant must cover every
requested item and cannot widen argv.

Flow correlates the profile with the invocation's authorization ID and grants
digest, but it does not issue or authenticate that authorization in this
checkpoint. The caller remains responsible for producing the authorization
record and exact operator-approved profile.

## Isolation profiles

V1 has two explicit isolation values:

| Isolation | Eligibility | Required evidence |
| --- | --- | --- |
| `trusted-unconfined` | Only an operator-`trusted` resolution | Source `none`, backend `none` version `0.0.0`, empty enforced authority, and all dimensions `not-enforced` |
| `sandboxed` | Operator-`trusted` or operator-`sandboxed` resolution | Source `caller-attested-host`, a named versioned backend, exact enforced authority equal to the request, and all dimensions `enforced` |

Operator trust and isolation are distinct. `trusted` permits the operator to
select either profile. `sandboxed` forbids `trusted-unconfined`. `disabled`
never resolves. A sandbox-required process candidate may pass deterministic
catalog resolution so that the downstream authority gate can assess it;
sandbox-required in-process execution remains blocked because that seam is
unconfined.

The evidence source name is intentionally `caller-attested-host`. Flow checks
that the statement is complete, closed, and correlated; it does not verify the
caller, inspect host policy, or independently interrogate the named backend.
Consequently, an `AuthorizedProcess` is a Flow preflight token, not an operating-
system capability or proof of sandbox execution.

## Correlation and deterministic identity

Both portable contracts use `flow.canonical-json/v1`: object member names are
sorted recursively, arrays retain their contract-significant order, compact
UTF-8 JSON is emitted without a trailing newline, and lowercase SHA-256 names
the resulting bytes. This is a named Flow profile, not an RFC 8785 claim.

The authority profile binds:

- extension-lock ID, authorization ID, and grants digest;
- run and invocation IDs;
- extension ID, version, declared publisher ID, and package integrity;
- capability and process interface;
- execution-subject lock ID and canonical digest;
- configured operator trust and selected isolation; and
- exact requested and granted authority.

Enforcement evidence binds its own ID to the profile ID and canonical digest,
the same run/provider/capability/interface and subject lock, the fresh subject-
observation digest, isolation, ambient policy, backend identity/version, exact
enforced authority, and one status for every authority dimension.

`AuthorizedProcess` additionally retains canonical profile/evidence digests and
the correlated resolution, invocation, and matched-subject context. Reusing the
token with another run, invocation, lock, provider, capability, interface, or
subject observation fails process preflight.

## Process gate order

The process evidence seams apply these prerequisites:

1. Resolve an eligible process provider from a validated manifest and operator
   lock.
2. Observe the exact locked package and executable bytes and construct
   `MatchedExecutionSubjects`.
3. Validate the exact authority profile against the resolved provider request,
   operator grants, trust, invocation authorization, and secret handles.
4. Validate and correlate the host-enforcement evidence with the authority
   profile and fresh subject observation.
5. Construct `AuthorizedProcess` only after all prior gates succeed.
6. Require both opaque tokens when encoding the request and again when
   validating the supplied transcript.

The current library performs these correlation gates around a host-neutral
transport seam. The operating-system work that would occur between encoding
and validation remains outside this checkpoint.

## Evidence claims and extension points

Content-digest observation, lock equality, cryptographic verification,
publisher identity, operator trust, selected isolation, and host enforcement
are separate evidence categories. None substitutes for another:

- matching bytes establish equality to a supplied digest, not who published
  them;
- a correlated publisher string is not authenticated identity;
- operator trust is policy, not cryptographic proof;
- a profile states intended authority, not actual containment; and
- caller-attested enforcement evidence is not authenticated host telemetry.

`unsupported_claims` is a closed empty extension point in v1. Signature,
attestation, transparency-log, authenticated sandbox, or launch-time file-
binding claims require a new contract version plus an implementation that can
verify them. V1 rejects attempts to insert such claims rather than treating
them as opaque proof.

## Failure behavior

Preflight rejects invalid or noncanonical profiles, duplicate authority,
wildcards, requested authority absent from the manifest, authority outside the
operator grant, ungranted requests, mismatched secret handles, invalid trust/
isolation combinations, incomplete guarantees, contradictory enforcement
status, over- or under-enforced sandbox authority, unsupported claims, wrong
profile/subject digests, and stale context.

All failures occur before process request bytes are returned or transcript
evidence can produce `ValidatedExecution`. Protocol-valid stdout, exit `0`, or
a provider success result cannot override a failed authority gate.

## Guarantees and non-goals

This profile guarantees deterministic representation and exact correlation of
the authority selected for one process evidence boundary. For sandboxed
profiles, it also requires a complete caller statement that every dimension
was enforced exactly as requested.

It does not provide:

- process discovery, argv execution, launch, supervision, signal, timeout,
  cancellation, output capture, or reap behavior;
- an operating-system sandbox, namespace, container, jail, security profile,
  seccomp filter, filesystem mount policy, or network policy;
- authenticated runner identity or verification that a backend applied its
  claimed policy;
- a race-free binding from observed executable bytes to the launched file;
- signature, certificate, attestation, publisher, or transparency-log
  verification;
- durable plans, run state, checkpoints, retry, or resume;
- real Aniflow, Optiflow, or Renderflow adapters; or
- authority enforcement for the injected in-process seam.

## Conformance

Rust and portable-contract fixtures cover trusted-unconfined and sandboxed
positive cases; every authority dimension; stable canonical identities; wrong
provider, version, publisher, capability, interface, run, invocation, lock,
profile, and observation identities; stale tokens; missing, extra, wildcard,
duplicate, or unsorted authority; incomplete sandbox coverage; duplicate,
reordered, or contradictory guarantees; unsupported claims; and the existing
altered package/executable subject corpus.
