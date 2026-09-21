---
schema: aether.architecture-decision/v1
id: adr-0007
title: Accept artifacts through root-confined host observations
kind: architecture-decision
status: accepted
accepted: 2026-09-20
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
---

# ADR-0007 — Accept artifacts through root-confined host observations

## Context

ADR-0005 makes provider events and results untrusted contributions. ADR-0006
applies that rule to a process transcript: a coherent result and exit `0` are
necessary evidence but do not prove that declared files exist, remain inside a
workspace, or contain the bytes named by an input digest.

The extension-v1 envelopes deliberately carry artifact identifiers rather than
host paths. Before a real provider runner or adapter can use them safely, Flow
needs to bind those identifiers to caller-selected workspace locations and
independently observe their bytes. That boundary must work for both in-process
and future process adapters without putting provider-specific formats into the
orchestration core.

Executable integrity, operating-system authority enforcement, real process
supervision, and domain-specific output validation are separate trust
boundaries. Combining them with artifact identity would prevent focused review
and would make a portable content contract depend on a particular launcher or
sandbox.

Flow issue #36 therefore implements the artifact-binding and host-observation
slice of parent issue #25.

## Decision

Flow adds two provisional closed contracts:

1. `flow.artifact-bindings/v1` is a Flow-owned binding set. Each immutable input
   and candidate output has a unique artifact ID, logical port, media type,
   kind, and portable root-relative locator. An input additionally carries its
   expected SHA-256 content digest.
2. `flow.artifact-observations/v1` records Flow-observed identity, digest, byte
   count, and, for directories, a deterministic flattened manifest. It names
   the binding set it observes and the digest/manifest profiles it used.

The v1 locator profile is a nonempty UTF-8 string of slash-separated normal
segments. Absolute paths, drive or URI colons, backslashes, empty segments,
`.` segments, `..` segments, control characters, host-ambiguous punctuation,
trailing spaces or periods, and Windows device names are invalid. Spaces and
Unicode scalar values are otherwise preserved exactly; Flow performs no case
folding or Unicode normalization.

Observation starts from one caller-selected root. The root must exist as a
real directory rather than a symlink. Flow inspects every bound path segment
and every directory descendant with link-aware metadata, rejects symlinks and
special filesystem nodes, canonicalizes each bound target as a containment
defense, and requires the target to remain beneath the canonical root.
Non-UTF-8 descendant names are rejected because the portable manifest cannot
represent them losslessly.

The portable observation document is serializable evidence, not an authority
token. Flow wraps freshly observed evidence in an `ObservedArtifactSet` whose
fields are private. Deserializing or constructing
`HostArtifactObservationSet` cannot recreate that token or enter artifact
acceptance without a fresh host observation.

Files use the lowercase hexadecimal SHA-256 digest of their bytes and their
exact byte length. Directories use `flow.directory-manifest/v1`:

- direct children are ordered by their exact UTF-8 name;
- each child contributes `name`, `kind`, `digest`, and `size_bytes` to a compact
  JSON array in that field order;
- the directory digest is the SHA-256 digest of those JSON bytes;
- a directory byte count is the checked sum of all descendant file byte
  lengths; and
- the portable observation flattens every descendant in strictly increasing
  slash-separated locator order while retaining directory entries and their
  recursively computed identities.

Permissions, timestamps, ownership, extended attributes, allocation size, and
directory enumeration order do not enter identity. An empty directory hashes
the UTF-8 bytes of `[]`.

`ValidatedExecution` remains provider-evidence acceptance, not artifact
acceptance. Flow constructs the separate opaque `AcceptedArtifactSet` only when
all of these checks pass together:

1. the invocation still matches the resolved extension, capability, interface,
   lock, configuration schema, expected output types, and limits;
2. the validated result still matches that invocation and represents a
   complete `produced` or `reused` outcome;
3. invocation input IDs and digests exactly match the input bindings;
4. input and output media types are allowed by the resolved capability, every
   invocation-expected output type has a candidate binding, and no binding
   introduces an undeclared type;
5. result-consumed IDs, result-produced IDs, and `artifact-produced` event IDs
   each match their declared sets exactly, with no duplicates;
6. host observations cover every input and output exactly once and match every
   bound ID, port, media type, kind, and locator; and
7. every observed input digest equals its immutable expected digest.

An output digest is learned from host observation rather than accepted from the
provider. A provider result, event, exit code, or file existence alone cannot
construct `AcceptedArtifactSet`.

## Rationale

Separate bindings preserve the extension envelopes' path independence while
letting each run select an isolated workspace. Separate observations make the
authority boundary explicit: provider-authored identifiers are compared with
evidence derived from host-visible bytes rather than being promoted directly.

A recursive, name-sensitive directory identity detects additions, removals,
renames, kind changes, byte changes, and empty-directory changes. The compact
JSON profile is simple to reproduce in other languages and makes the hashed
representation inspectable without treating platform metadata as portable.

Opaque observed and accepted types prevent callers from manufacturing either
the host-observation premise or the completion token with a struct literal.
Keeping `ValidatedExecution` separate also prevents existing event/result
validation from silently acquiring stronger filesystem claims than it can
support.

## Evidence and assumptions

- Closed extension invocation, event, result, and resolution models already
  exist and are validated before `ValidatedExecution` is constructed.
- Both the injected in-process seam and host-neutral process seam produce the
  same `ValidatedExecution` type, so artifact acceptance can follow either.
- The caller controls the observation root and is responsible for keeping it
  quiescent while Flow observes it. Portable Rust filesystem APIs do not provide
  an atomic, race-free directory capability across all supported hosts.
- SHA-256 is already the suite's provisional content-digest algorithm.
- Real adapters are expected to stage outputs beneath an isolated run root and
  to expose provider-native validation separately from byte identity.

## Alternatives considered

- **Put host paths in extension invocation/result v1:** rejected because host
  topology is not portable provider identity and would require changing an
  existing contract family.
- **Trust provider-reported digests:** rejected because the provider would be
  approving the same evidence that Flow must independently assess.
- **Accept file existence plus exit `0`:** rejected because neither establishes
  immutable input identity, output bytes, exact declared coverage, or event and
  result agreement.
- **Hash directory file bytes only:** rejected because renames, empty
  directories, and file-versus-directory changes would be ambiguous.
- **Include timestamps, modes, or ownership:** rejected because those fields are
  host-specific and unstable under safe copying.
- **Follow symlinks that remain beneath the root:** rejected because link
  retargeting and host-dependent resolution would weaken the declared
  workspace boundary.
- **Implement executable verification, sandboxing, and the process runner in
  this decision:** deferred to the remaining children of Flow #25 because each
  has distinct platform and authority evidence.

## Trade-offs

Rejecting all symlinks is stricter than many build tools and prevents legitimate
link-preserving directory artifacts in v1. It makes the initial containment and
identity claims understandable and portable.

Exact UTF-8 names allow spaces and international text but exclude byte-oriented
Unix names. No normalization means visually similar Unicode names remain
different artifacts. Case behavior follows exact string identity even on a
case-insensitive filesystem.

Directory observation reads every descendant and can be expensive. The
checkpoint has no streaming manifest sink, incremental cache, hard-link
identity, or resource quota. A future runner must apply authority and resource
controls before observing untrusted large trees.

The check is not a race-free sandbox boundary. A concurrently mutating or
hostile workspace can change between metadata inspection and byte reads. The
production runner must create an isolated, quiescent workspace or replace this
portable observer with a stronger platform capability before making adversarial
concurrency claims.

## Expected consequences

- Adapters can bind stable artifact IDs to per-run paths without leaking paths
  into provider identity.
- Inputs are re-read and digest-checked after execution, detecting provider or
  caller mutation before acceptance.
- Candidate outputs acquire host-observed content identities that downstream
  planning and provenance can reference.
- File and directory artifacts have one deterministic cross-platform profile.
- `ValidatedExecution` and `AcceptedArtifactSet` communicate different evidence
  strengths in the type system.
- Executable provenance, enforced authority, domain semantics, and actual child
  supervision remain visible follow-up work.

## Observed outcomes

Flow issue #36 adds the two contracts, Rust binding/observation/acceptance APIs,
portable path and manifest validation, and adversarial conformance tests. It
does not add a real provider adapter or process runner. Default-branch CI after
merge remains the acceptance evidence for this implementation.

## Security, privacy, and authority

All provider artifact claims remain untrusted until correlated with bindings
and host observations. Locators are validated before joining, and links,
special nodes, path traversal, absolute paths, Windows-style separators, drive
prefixes, and canonical target escape are rejected.

Artifact contents and names may be sensitive. The observation contract contains
locators, sizes, and digests and must be handled according to the underlying
workspace's sensitivity. This decision does not export contents, scan for
secrets, redact names, or make a digest anonymous.

This decision grants no filesystem access to a provider and does not enforce a
manifest's requested permissions. It does not verify executable/package bytes,
publisher identity, signatures, or transparency logs. It is not an
operating-system sandbox, a process launcher, a domain validator, or an atomic
filesystem snapshot.

## Review triggers

Review this decision if:

- a supported provider must exchange symlink or non-UTF-8 artifacts;
- directory identity must interoperate with an established content-addressed
  storage format;
- SHA-256 no longer meets suite requirements;
- case or Unicode normalization must be portable across providers;
- hard links, sparse files, permissions, or extended attributes become
  semantically meaningful;
- hostile concurrent mutation must be resisted without an isolated workspace;
  or
- a released adapter cannot stage every declared artifact beneath one root.

## Related artifacts

ADR-0005, ADR-0006, `flow-architecture`, `flow-roadmap`,
`docs/integrations/artifact-bindings.md`,
`docs/integrations/extension-contract.md`, Flow issue #25, Flow issue #36, and
Flow issue #11.

## Validation

Conformance must cover stable repeated observations, files, empty and nested
directories, spaces, Unicode, byte and name changes, immutable input mismatch,
missing artifacts, kind mismatch, duplicate IDs/ports/locators, absolute and
traversing locators, backslash and drive ambiguity, symlinks, special nodes,
non-UTF-8 names where supported, invalid/tampered manifests, incomplete host
coverage, invocation/resolution mismatch, result/event mismatch, duplicates,
and partial or unsuccessful terminal evidence. Every rejected case must return
a typed error and never an `AcceptedArtifactSet`.
