---
schema: aether.architecture-decision/v1
id: adr-0008
title: Require exact locked package and executable subjects
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
  - adr-0007
  - adr-0009
---

# ADR-0008 — Require exact locked package and executable subjects

## Context

ADR-0005 separates provider declarations from operator authority. ADR-0006
defines a host-neutral process transcript but intentionally cannot prove which
program produced it. ADR-0007 gives Flow a deterministic, root-confined content
observer, but its artifact-acceptance token describes run inputs and outputs,
not the provider code eligible for execution.

A provider identity, version, publisher string, declared entrypoint, or process
exit cannot establish that the installed package and selected executable still
contain the bytes approved by the operator. Conversely, digest equality proves
content equality under the selected digest profile; it does not authenticate a
publisher, verify a signature, establish transparency-log inclusion, or make an
operator's trust policy objectively correct.

Flow issue #38 is the bounded executable/package-integrity child of parent
issue #25. Authority enforcement, isolation, process launch, durable state, and
real provider adapters remain separate roadmap work.

## Decision

Flow adds two provisional, closed contracts:

1. `flow.execution-subject-lock/v1` names the exact package directory and the
   exact regular executable file eligible for one process-mode provider
   context.
2. `flow.execution-subject-observations/v1` records a fresh Flow-owned
   observation of those subjects and keeps each evidence category explicit.

The package subject is the directory at the lock's portable root-relative
`package.locator`. Its SHA-256 identity uses `flow.directory-manifest/v1` from
ADR-0007, including names, node kinds, child digests, sizes, and empty
directories. The package digest must equal both the subject lock's extension
integrity and the resolved extension manifest/lock integrity.

The executable subject is the regular file at
`package.locator/executable.locator`. The executable locator is package-relative
in the lock and root-relative in observations. Its SHA-256 digest is checked
separately even though the executable is also represented inside the package
directory manifest. This deliberate redundancy makes both the package release
and the selected launch subject explicit and independently diagnosable.

The subject lock also binds:

- the operator extension-lock ID;
- extension ID, strict version, declared publisher ID, and package integrity;
- capability ID;
- process interface kind, name, and invocation protocol; and
- the provider-declared entrypoint.

All of those fields must agree with the resolved extension and invocation.
Wrong provider, version, publisher, capability, interface, protocol,
entrypoint, package kind, executable kind, locator, or digest fails closed.

The lock's deterministic identity uses `flow.canonical-json/v1`: recursively
sort object member names by their exact strings, preserve array order and
scalar values, serialize compact UTF-8 JSON, append no newline, and compute
lowercase hexadecimal SHA-256 over those bytes. This is a Flow profile, not an
RFC 8785 claim.

Observation reuses ADR-0007's caller-selected root, portable locator, recursive
directory identity, containment, link, special-node, and non-UTF-8 rejection
rules. Flow constructs an opaque `MatchedExecutionSubjects` only after fresh
observation and exact comparison of both subjects with the lock. Portable JSON
can be inspected or persisted as evidence, but deserializing it cannot recreate
the opaque token.

Process request encoding and process-transcript validation require the subject
lock and its matching opaque token. The token is bound to the lock's canonical
digest plus the extension lock, run, invocation, provider identity, capability,
process interface, entrypoint, and configured operator trust. A token from a
different lock or invocation is rejected before request encoding or transcript
acceptance.

The observation contract reports these claims separately:

| Evidence category | v1 status | Meaning |
| --- | --- | --- |
| Content digest | `observed` | Flow read the package and executable content using SHA-256 profiles |
| Lock equality | `matched` | Both observations exactly equal the supplied lock, identified by its canonical digest |
| Cryptographic verification | `not-performed` | No signature, certificate, attestation, or key verification occurred |
| Publisher identity | `declared-and-correlated` | The same publisher identifier appears in the correlated declarations; no identity proof occurred |
| Operator trust | `configured` | The resolved operator lock supplied the trust mode; this is policy, not authenticity |
| Transparency log | `not-checked` | No log entry, inclusion proof, checkpoint, or consistency proof was checked |

V1 supports only these statuses. Cryptographic-verification and
transparency-log evidence arrays must be empty. Unknown fields, extra subjects,
duplicate or contradictory subjects, alternative status strings, and supplied
signature or log evidence are rejected. Future signature, attestation, or
transparency-log support requires a new contract version and verifier; it must
not reinterpret digest equality as authenticity.

## Rationale

Locking the whole package detects additions, removals, renames, and changes to
supporting content. Locking the executable separately makes the exact intended
process subject visible rather than relying on an implicit path inside a
package digest. Binding both to resolution and invocation prevents a correct
digest from being replayed for the wrong provider operation.

An opaque token distinguishes evidence that Flow freshly observed from a
portable document that any caller could deserialize. Requiring it at both
process seams ensures that protocol-valid bytes alone cannot become execution
evidence without the package/executable integrity premise.

Separating claim categories prevents a common trust escalation: a matching
SHA-256 digest is not a publisher signature, and a publisher string is not a
verified identity. Closed v1 vocabularies make unsupported stronger claims fail
rather than silently degrade.

## Evidence and assumptions

- The extension manifest and operator lock already agree on extension
  identity, version, declared publisher, package digest, capability, and
  process interface before resolution succeeds.
- The caller selects the observation root and keeps it quiescent while Flow
  reads it.
- `flow.directory-manifest/v1` is the current package-directory identity
  profile and SHA-256 is the current content-digest algorithm.
- The caller or a future runner retains the opaque token between observation,
  request encoding, and transcript validation.
- Trusted candidates are the only resolution-eligible candidates in the
  current executable checkpoint.

## Alternatives considered

- **Treat the extension digest as sufficient without an executable subject:**
  rejected because the exact entrypoint bytes and location would remain
  implicit.
- **Hash only the executable:** rejected because supporting package files and
  package composition would not be pinned.
- **Trust a provider-reported digest:** rejected because the execution subject
  cannot authorize its own identity evidence.
- **Infer publisher authenticity from digest equality:** rejected because a
  digest identifies content only relative to a trusted expected value.
- **Accept arbitrary signature or transparency evidence in v1:** rejected
  because storing unverifiable material would invite stronger claims than Flow
  can assess.
- **Combine launch, sandbox, and integrity enforcement:** deferred because
  those require platform-specific authority and lifecycle evidence distinct
  from deterministic content observation.

## Trade-offs

The package directory is read recursively and the executable is read again as
its own subject. That costs extra I/O in exchange for explicit subject
evidence. Large packages have no streaming evidence sink or resource quota in
this checkpoint.

Portable observation cannot provide a race-free guarantee that a later child
process opens the same file object. A package can change after observation, and
the current host-neutral seam neither opens nor launches it. A production
runner must close this time-of-check/time-of-use gap with an isolated,
quiescent workspace or stronger platform file capabilities.

The declared entrypoint remains manifest metadata correlated alongside the
locked file locator. The executable locator, not a `PATH` lookup, names the
exact file a future runner must invoke. This checkpoint does not construct
argv, inspect executable permission bits, resolve dynamic libraries, inspect
interpreters, or validate transitive runtime code.

## Expected consequences

- Process request and transcript evidence cannot pass without a fresh token
  for the exact locked package and executable bytes.
- Altered package content and altered executable content are rejected before
  the process protocol can be accepted.
- Provider identity, capability, interface, entrypoint, lock, run, and
  invocation replay mismatches fail deterministically.
- Portable evidence communicates content equality without claiming publisher
  authenticity, signature validity, transparency, or trustworthy policy.
- Future authenticity evidence has a versioned extension path rather than an
  untyped field in v1.

## Observed outcomes

Flow issue #38 adds the lock and observation contracts, deterministic lock and
observation identities, fresh root-confined observation, the opaque matched
token, process-preflight integration, and positive/adversarial conformance
tests. It does not add a launcher, sandbox, authority backend, durable state, or
real provider adapter. Pull-request and default-branch CI remain the acceptance
evidence for the implementation.

## Security, privacy, and authority

Digests, file names, package layout, provider identity, and publisher strings
may reveal sensitive operational information. Evidence must be handled at the
sensitivity of the installed package. This contract exports no file contents
and makes no digest-anonymity claim.

Observation grants no provider permission and enforces no filesystem, network,
environment, subprocess, secret, GPU, AI, signing, or publication policy. The
configured `trusted` status records operator policy; it does not make code safe
or authentic. Cryptographic and transparency fields explicitly state that no
such verification occurred.

## Review triggers

Review this decision if:

- Flow adds signature, certificate, attestation, or transparency-log
  verification;
- a runner can bind observation to the exact subsequently executed file
  object;
- package identity moves from a directory manifest to a released archive or
  content-addressed packaging standard;
- executables require interpreter, dynamic-library, or transitive-code
  identity;
- SHA-256 or canonical JSON no longer meets the suite's requirements; or
- a released provider cannot represent its exact process subject with this
  lock profile.

## Related artifacts

ADR-0005, ADR-0006, ADR-0007, ADR-0009, `flow-architecture`, `flow-roadmap`,
`docs/integrations/execution-subjects.md`,
`docs/integrations/process-transport.md`, Flow issue #25, Flow issue #38, and
Flow issue #11.

## Validation

Conformance must cover deterministic positive observations; spaces and Unicode
names; altered package and executable bytes; wrong provider, version,
publisher, capability, interface, protocol, entrypoint, lock, run, and
invocation; missing, escaping, symlinked, special, or wrong-kind subjects;
duplicate and contradictory evidence; unsupported verification claims; closed
unknown fields; canonical identity; and replay of a matched token against a
different context. Every rejected case must return a typed error and never a
`MatchedExecutionSubjects` or `ValidatedExecution`.
