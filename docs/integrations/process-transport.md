# Bounded process transcript profile

## Purpose and scope

This document defines the provider-neutral standard-input, standard-output,
standard-error, and termination transcript that a future Flow process adapter
must use. It composes the existing `flow.extension-invocation/v1`,
`flow.extension-event/v1`, and `flow.extension-result/v1` contracts without
changing their semantic ownership.

The implemented checkpoint is deliberately host-neutral. It can encode one
invocation request and validate already-captured transcript bytes plus an
explicit termination observation. Both operations require a fresh opaque match
for the exact locked package and executable subjects. It does not locate,
start, supervise, signal, or reap an operating-system process. A transcript
proves only that supplied evidence is internally valid under this profile and
that the earlier subject observation matched its lock; it does not prove how
the evidence was captured or that the observed executable produced it.

This profile is governed by
[ADR-0006](../architecture/governance/decisions/ADR-0006-bounded-process-transport.md)
and refines the process mode described by the
[federated extension contract](extension-contract.md). Package and executable
preflight is governed separately by
[ADR-0008](../architecture/governance/decisions/ADR-0008-locked-execution-subjects.md).

## Boundary ownership

| Channel or observation | Owner | Meaning |
| --- | --- | --- |
| Standard input | Flow | Exactly one validated invocation request |
| Standard output | Provider contribution, Flow validation | Protocol records only: zero or more events followed by exactly one result |
| Standard error | Provider contribution, caller-controlled handling | Opaque, bounded diagnostic bytes; never protocol |
| Termination | Future process runner contribution, Flow validation | Host-neutral exit, timeout, or cancellation observation |
| Event observation | Flow and caller | A fallible, authoritative acceptance boundary through `EventSink` |
| Execution-subject lock | Flow operator | Exact package/executable content and correlated process context |
| Matched execution subjects | Flow observer | Opaque proof that a fresh observation exactly matched that lock and invocation |

The provider cannot promote its result to accepted completion. Flow returns a
`ValidatedExecution` only after framing, contract, correlation, ordering,
observer, terminal-consistency, and termination checks all pass.

`ValidatedExecution` is not artifact acceptance. A future runner must next use
the separate [artifact-binding profile](artifact-bindings.md) to observe bound
workspace bytes and construct `AcceptedArtifactSet`.

For process mode, `ValidatedExecution` does require the separate
[execution-subject profile](execution-subjects.md). That gate establishes
content equality to the supplied lock, not signature validity, publisher
authenticity, transparency-log inclusion, sandbox enforcement, or proof that a
later child opened the observed file object.

## Standard-input request

The encoded request frame has this grammar:

```text
stdin = compact-invocation-json LF EOF
```

The request requirements are:

- the value is one valid `flow.extension-invocation/v1` object;
- the invocation passes Flow's semantic validation before encoding;
- the JSON is UTF-8 and compact rather than pretty-printed;
- exactly one line-feed byte terminates the object;
- no byte-order mark, blank prefix, second request, or trailing data is
  permitted; and
- a future runner must write only that frame and close standard input after the
  terminating line feed.

Compact encoding makes requests reproducible for the same typed value, but it
is not an RFC 8785 canonicalization claim. Request bytes are not an extension,
configuration, authorization, artifact, or result identity.

## Standard-output records

The provider writes JSON Lines using this grammar:

```text
stdout     = *event-record result-record EOF
event-record = extension-event-json record-end
result-record = extension-result-json record-end
record-end = LF / CRLF
```

Each record is one complete UTF-8 JSON object on one physical line. Flow uses
the closed `schema_version` field to distinguish
`flow.extension-event/v1` from `flow.extension-result/v1`; there is no
unversioned wrapper or console-text heuristic.

The record rules are:

- zero or more event records may precede the result record;
- exactly one result record is required;
- the result must be the final record;
- both line-feed and carriage-return-plus-line-feed endings are accepted;
- every record, including the result, requires a line ending;
- EOF must follow the result's line ending immediately;
- blank lines, byte-order marks, invalid UTF-8, malformed JSON, unknown schema
  versions, duplicate results, records after the result, and unterminated final
  records are rejected; and
- human logs, progress bars, banners, prompts, and any other non-protocol text
  are forbidden on standard output.

The framing grammar permits an empty event prefix, but accepted execution still
uses the shared event/result validator. That validator requires the existing
identity, ordering, diagnostic, artifact-reference, and terminal-event/result
invariants. A result without the matching terminal event therefore cannot
produce `ValidatedExecution`.

The raw standard-output byte count includes JSON, carriage returns, line feeds,
and any invalid or trailing bytes. It must not exceed the invocation's
`limits.max_stdout_bytes`. The host-neutral validator rejects an oversized
captured transcript; preventing an operating-system child from producing or
buffering excess output is deferred to a future runner.

## Standard error

Standard error is an independent opaque byte stream. Flow does not require it
to be UTF-8, parse it as JSON, merge it with standard output, convert it into an
extension diagnostic, or use it to decide provider success.

The raw byte count must not exceed `limits.max_stderr_bytes`. An oversized
captured stream rejects the transcript. Raw standard error may contain secrets,
private paths, or provider-native text, so it is sensitive operational evidence
and is not portable result, provenance, event, or telemetry data. Callers must
apply an explicit inspection and redaction policy before exporting it.

This checkpoint validates the supplied byte bound. Independent concurrent
capture, backpressure, truncation, and forced child termination remain future
runner responsibilities.

## Termination observations

Termination is represented without assuming Unix signals, Windows status
objects, or a particular async runtime. Only a normal exit with code `0` is
eligible for accepted execution.

| Observation | Validation meaning |
| --- | --- |
| Normal exit `0` | Necessary but not sufficient; the complete transcript must also validate |
| Normal nonzero exit | Transport failure, even if stdout claims success |
| Exit without a portable code | Transport failure, including signal termination reported without a code |
| Timeout | Transport failure; no validated execution |
| Host cancellation | Transport failure; no validated execution |

A coherent provider result whose semantic outcome is `failed`, `blocked`,
`unavailable`, or `cancelled` may still be valid evidence when the provider
completed the protocol and exited `0`. The result outcome describes the domain
operation; the process exit describes whether the transport completed normally.

Conversely, exit `0` never converts missing, malformed, contradictory, or
observer-rejected evidence into completion. A timeout, host cancellation, or
nonzero exit cannot be repaired by a provider-authored result.

The current library validates a caller-supplied termination observation. It
does not measure a timeout, issue cancellation, deliver a signal, wait through
`cancellation_grace_ms`, or prove that a process was reaped.

## Validation and acceptance order

The boundary applies these gates before constructing accepted execution:

1. Validate the invocation and require that it matches the resolved process
   interface.
2. Require the exact execution-subject lock and a fresh opaque match token
   bound to that lock, resolution, run, invocation, provider, capability,
   process interface, entrypoint, and configured operator trust. Request
   encoding is a separate operation with the same invocation and subject
   preflight gates for use by a future runner.
3. Enforce the captured standard-output and standard-error byte limits.
4. Require the supplied termination observation to be normal exit `0`.
5. Decode the JSON Lines framing and require the event-then-result grammar.
6. Apply the shared Flow-owned event validation, correlation, uniqueness,
   strictly increasing sequence, and diagnostic checks. Forward each
   individually valid event to the caller's `EventSink` in transcript order.
7. Validate the result contract, correlation, diagnostics, artifact references,
   outcome, and consistency with the terminal event.
8. Construct `ValidatedExecution` only after every preceding gate succeeds.

This ordering gives byte-limit and termination failures precedence over parsing
or observing provider-authored records. In particular, events from an
abnormally terminated process never reach the authoritative sink. A valid
event can reach the sink before a later event or terminal result rejects the
execution; observation is authoritative but not transactional rollback.

Invalid provider bytes remain untrusted evidence. The process boundary must not
deserialize bytes directly into `ValidatedExecution` or offer a public
unchecked constructor from raw events and a result.

## Event observation is authoritative

`EventSink` remains fallible and authoritative. It receives an immutable event
only after that event passes its individual contract and invocation-correlation
checks. If the sink rejects an event, Flow rejects the execution and returns no
`ValidatedExecution`.

This behavior is intentionally not a best-effort telemetry contract. The sink
may implement a caller-required audit, policy, or durable observation boundary,
so its failure is allowed to affect acceptance. It cannot select a provider,
rewrite an event, alter extension integrity, grant authority, or change result
identity. A separate non-authoritative logging, metrics, tracing, or
OpenTelemetry contract remains work for Flow's observability roadmap; this
checkpoint ships no telemetry backend or exporter.

## Failure classes

The transcript boundary distinguishes at least these failure concerns:

| Concern | Representative evidence |
| --- | --- |
| Invalid invocation | Wrong schema, malformed identity, or non-process interface |
| Execution-subject preflight | Invalid lock, wrong context, altered package/executable bytes, or stale match token |
| Output limit | Raw stdout or stderr exceeds its declared byte limit |
| Encoding or framing | Invalid UTF-8, malformed JSON line, blank line, unknown record, or missing terminator |
| Protocol order | Missing result, duplicate result, or record after result |
| Invalid event | Invalid schema, identity, event ID, sequence, phase, diagnostic, or post-terminal event |
| Observer rejection | The authoritative `EventSink` rejects a valid event |
| Invalid result | Invalid schema, identity, outcome, validation, artifact reference, or terminal consistency |
| Abnormal termination | Nonzero exit, timeout, or host cancellation |

The caller retains its borrowed raw transcript for explicit inspection; Flow's
process errors do not copy stdout or stderr or reproduce opaque provider bytes
in display text. Structured event or result evidence may be retained by later
semantic errors. No failure after invocation begins authorizes implicit retry
or fallback.

## Determinism and conformance

Conformance fixtures are synthetic, redistribution-safe, and independent of
filesystem enumeration, locale, wall-clock time, network access, and a real
provider executable. The required proof covers:

- a compact invocation request with exactly one terminating line feed;
- successful LF and CRLF event/result transcripts;
- stable validation regardless of host read-chunk boundaries;
- zero-event framing followed by semantic rejection for the missing terminal
  event;
- invalid UTF-8, malformed JSON, blank lines, unknown schemas, partial final
  records, missing and duplicate results, and trailing records;
- duplicate event identifiers, non-increasing sequences, mismatched identities,
  unredacted diagnostics, contradictory outcomes, and failed success-like
  validation evidence;
- stdout and stderr at and above their declared byte boundaries;
- exit `0`, nonzero exit, timeout, and host-cancellation observations;
- authoritative sink rejection; and
- proof that every invalid case returns an error rather than
  `ValidatedExecution`.

After the subject token is constructed, transcript validation depends only on
the resolved extension, invocation, exact subject lock and token, transcript
bytes, termination observation, and authoritative sink outcome. It does not
reinspect the host filesystem, environment, process table, clock, or network.

## Implemented guarantees and deferred work

Issue #26 implements the host-neutral request encoder and transcript validation
seam described above. Issue #38 adds its mandatory locked package/executable
preflight. It reuses the closed invocation, event, result, execution-subject,
and Flow-owned semantic validation models.

It does not implement or prove:

- executable discovery, direct argv construction, or process launch;
- standard-input writing or concurrent stdout/stderr capture against a child;
- runtime output backpressure, timeout measurement, cancellation delivery,
  grace periods, kill, or reap behavior;
- publisher authentication, signature/attestation verification, or
  transparency-log verification; exact observed package and executable digest
  matching is implemented separately and must not be described as any of
  those authenticity claims;
- automatic artifact discovery or provider-native output validation; the
  separate issue #36 library seam requires explicit bindings and a fresh
  post-transcript host observation beneath a caller-selected root;
- filesystem, environment, subprocess, network, AI, GPU, signing, publication,
  or other side-effect isolation;
- an operating-system sandbox or an enforceable `sandboxed` trust profile;
- real Aniflow, Optiflow, or Renderflow process adapters;
- durable plans, run state, checkpoints, retry, or resume;
- a public Flow CLI; or
- logging, metrics, tracing, OpenTelemetry, or a hosted observability backend.

Later work may place a platform-specific runner in front of this validator. The
runner must supply evidence without weakening the transcript grammar or
promoting launch, exit, or file existence alone to accepted completion. It must
also bind observation to the exact launched file object, quiesce and isolate
the workspace, and preserve both execution-subject and artifact-observation
claims without overstating authenticity.
