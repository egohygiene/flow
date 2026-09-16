---
schema: aether.architecture-decision/v1
id: adr-0006
title: Bound process transport with a deterministic JSON Lines transcript
kind: architecture-decision
status: accepted
accepted: 2026-09-16
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
  - adr-0004
  - adr-0005
---

# ADR-0006 — Bound process transport with a deterministic JSON Lines transcript

## Context

ADR-0004 keeps Aniflow, Optiflow, and Renderflow independently released behind
versioned library or process contracts. ADR-0005 separates provider declarations
from operator authority and requires process evidence to remain subject to the
same Flow-owned semantic validation as in-process evidence.

Flow issue #23 proved only a caller-injected in-process port. Before Flow can add
real provider adapters, it needs one provider-neutral process wire boundary that
does not parse human console output or let a process exit impersonate validated
completion.

A complete process runtime also requires executable discovery, direct argv
launch, independent pipe capture, executable integrity verification, artifact
binding, timeout and cancellation enforcement, and possibly operating-system
isolation. Combining those concerns before the wire contract is stable would
make platform behavior part of the protocol decision and turn the next
checkpoint into an unreviewable runtime implementation.

Flow issue #26 therefore accepts a smaller dependency-unlocking boundary:
deterministic standard-input encoding and host-neutral validation of supplied
standard-output, standard-error, and termination evidence.

## Decision

Flow process providers use a JSON Lines stdio profile built from the existing
closed v1 extension contracts:

1. Standard input contains exactly one compact UTF-8
   `flow.extension-invocation/v1` object, one terminating line feed, and EOF.
2. Standard output contains zero or more complete
   `flow.extension-event/v1` records followed by exactly one
   `flow.extension-result/v1` record and EOF.
3. Each standard-output record is terminated by LF or CRLF. The result must be
   terminated and final; blank lines, partial lines, unknown schemas, human
   presentation, duplicate results, and trailing records are invalid.
4. Standard output is protocol-only. Standard error is a separate opaque byte
   stream and is never parsed as protocol or merged with standard output.
5. Raw standard-output and standard-error bytes are checked against the
   invocation's declared limits before accepted execution can be constructed.
6. Only a supplied normal exit observation with code `0` is eligible for
   acceptance. A nonzero exit, an exit without a portable code, timeout, or
   host cancellation rejects the transcript regardless of provider-authored
   success evidence.
7. Exit `0` remains necessary but insufficient. Flow applies its existing event
   and result schema, identity, ordering, diagnostic, validation, artifact-ID,
   and terminal-consistency checks before constructing `ValidatedExecution`.
8. Events retain the existing fallible `EventSink` behavior. The sink is an
   authoritative acceptance observer, not a best-effort telemetry exporter;
   its rejection prevents `ValidatedExecution`.
9. No failure after invocation begins authorizes implicit retry or fallback.

The initial implementation accepts already-captured bytes and an explicit,
host-neutral termination observation. It neither starts nor controls a process.
The observation is correlated evidence supplied by a caller, not proof of an
actual operating-system event or executable identity.

## Rationale

JSON Lines preserves ordered event streaming while reusing the versioned JSON
contracts already validated by Flow. One record per line makes truncation,
trailing data, result multiplicity, and human-output contamination explicit.
Keeping standard error opaque prevents provider-native diagnostics from being
mistaken for authoritative events or results.

Separating host-neutral transcript validation from a launcher keeps the first
process checkpoint deterministic and redistribution-safe. The same fixtures can
run on every host without requiring signal semantics, executable permissions,
filesystem topology, timers, or a provider installation. Later runners can map
platform-specific behavior into this stable evidence boundary.

Requiring both a valid transcript and exit `0` preserves layered validation. A
provider-reported failure can complete the wire protocol normally, while a
provider-reported success cannot hide an abnormal process termination.

## Evidence and assumptions

- The repository already has closed Rust models and semantic validation for
  `flow.extension-invocation/v1`, `flow.extension-event/v1`, and
  `flow.extension-result/v1`.
- The current execution seam constructs `ValidatedExecution` only after
  Flow-owned preflight, event, observer, and terminal-result checks.
- `EventSink::emit` is intentionally fallible and its rejection already fails
  execution. This decision preserves that authoritative behavior and does not
  recast it as telemetry.
- Synthetic host-neutral transcripts can prove framing and correlation without
  claiming that a real provider process, timeout, cancellation, or output
  capture has been observed.
- Future provider adapters are assumed to be able to emit the versioned
  invocation/event/result semantics directly or through a Flow-owned adapter.
  That assumption remains unproven for real holon releases until adapter work
  reinspects them.

## Alternatives considered

- **One terminal JSON document:** rejected because it removes ordered event
  streaming and requires buffering all provider evidence before any observation.
- **Length-prefixed or binary frames:** deferred because they add codec and
  interoperability complexity without evidence that JSON Lines is insufficient
  for the first local provider adapters.
- **Parse human console output:** rejected because presentation text is not a
  stable, versioned machine contract and can interleave unpredictably.
- **Allow mixed logs and protocol records on stdout:** rejected because a
  heuristic separator would make framing ambiguous and diagnostics capable of
  corrupting authoritative evidence.
- **Treat a result or exit `0` alone as success:** rejected because either can
  disagree with events, validation evidence, identity, or the other termination
  signal.
- **Implement the launcher, artifact binding, integrity, and sandbox in the
  same checkpoint:** rejected for issue #26 because those are independent trust
  and platform boundaries that need their own evidence.

## Trade-offs

The strict final line ending and protocol-only stdout require providers or
adapters to route all human presentation to stderr or another explicit surface.
This is less permissive than many CLI conventions but makes truncation and EOF
semantics deterministic.

JSON Lines is human-inspectable and broadly interoperable, but it does not
provide binary framing or make the JSON bytes canonical. Compact request bytes
are reproducible for the same typed value but are not used as an authenticity or
artifact-identity proof.

The host-neutral seam can reject captured evidence that exceeds declared bounds
but cannot prevent a real child from producing unbounded output. It can classify
a timeout or cancellation observation but cannot cause either event. Those
enforcement guarantees remain unavailable until a runner implements and tests
them.

## Expected consequences

- Every future process adapter has one exact structured stdout grammar and does
  not invent provider-specific console parsing.
- The process and in-process seams reuse the same authoritative event/result
  validation and `ValidatedExecution` boundary.
- Synthetic fixtures can cover success, malformed framing, ordering,
  correlation, output bounds, observer rejection, and termination contradictions
  without a child process.
- A future launcher must capture stdout and stderr independently and translate
  host termination into this profile without weakening it.
- Real provider, artifact, package-integrity, sandbox, cancellation, and resume
  work remains visible rather than being implied by transcript validation.

## Observed outcomes

The issue #26 checkpoint adds a host-neutral encoder and transcript-validation
candidate plus hermetic conformance coverage. It does not supply a supported
process launcher or real holon adapter. Default-branch CI after merge remains the
acceptance evidence for the implementation.

## Security, privacy, and authority

All provider output is untrusted. Standard-output records must pass closed
contract and correlation checks. Opaque stderr and rejected raw records may
contain secrets or private paths; they are bounded sensitive evidence and must
not be exported or rendered as portable diagnostics without explicit
caller-controlled inspection and redaction.

This decision grants no filesystem, environment, subprocess, network, AI, GPU,
source-mutation, destructive, signing, or publication authority. It does not
verify package bytes, publisher identity, signatures, or executable provenance.
It does not provide an operating-system sandbox. In particular, validating a
transcript cannot make a `sandboxed` provider executable when no enforcing
backend exists.

The fallible `EventSink` remains part of authoritative acceptance. No logging,
metrics, tracing, OpenTelemetry, or hosted exporter is introduced, and
telemetry-only state does not enter invocation, result, or artifact identity.

## Review triggers

Review this decision if:

- a released provider cannot emit or be adapted to this JSON Lines profile;
- required payload volume or binary data makes line-oriented JSON unsuitable;
- streaming launch evidence requires a framing guarantee not expressible here;
- a new contract major changes invocation, event, or result discrimination;
- a real runner exposes incompatible cross-platform EOF or termination
  behavior; or
- an accepted telemetry contract changes the role of authoritative versus
  non-authoritative observers.

## Related artifacts

ADR-0004, ADR-0005, `flow-architecture`, `flow-roadmap`,
`docs/integrations/extension-contract.md`,
`docs/integrations/process-transport.md`, Flow issue #26, Flow issue #3, and
Flow issue #11.

## Validation

Conformance tests must prove the single-request encoding, LF and CRLF record
handling, event-then-result order, required terminal line ending and EOF,
protocol-only stdout, opaque stderr bounds, shared semantic validation,
fallible observer behavior, exit `0` requirement, and rejection of timeout,
cancellation, nonzero exit, malformed, duplicate, partial, contradictory, and
trailing evidence. Every invalid fixture must return an error and never a
`ValidatedExecution`.
