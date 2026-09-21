---
schema: aether.architecture-decision/v1
id: adr-0010
title: Bound direct provider launch and supervision
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
  - adr-0009
---

# ADR-0010 — Bound direct provider launch and supervision

## Context

ADRs 0006, 0008, and 0009 define the process wire protocol, exact locked
package/executable subjects, and exact authority/isolation preflight. Those
boundaries can validate caller-supplied evidence, but they do not launch the
child that produces it. A real runner must preserve all three gates while also
preventing shell interpretation, ambient environment inheritance, unbounded
captured memory, indefinite execution, and unreaped direct children.

Flow issue #42 is the final bounded FLO-3.2 child of parent issue #25. It is not
the durable-run, retry, checkpoint, resume, real-provider-adapter, or
operating-system-sandbox milestone.

## Decision

Flow owns one synchronous `LocalProcessRunner` for an already-resolved,
already-authorized `trusted-unconfined` process invocation.

Before launch the runner freshly observes the exact locked package and
executable, reuses the existing request preflight, canonicalizes the caller-
selected root, and derives the executable only from locked root-relative
locators. It invokes that executable directly with the authorized ordered argv,
sets the package as the working directory, clears the inherited environment,
and reconstructs only authorized variables through caller-owned opaque secret
resolution. It never interpolates a shell string or searches ambient `PATH`.

Standard input, standard output, and standard error are handled by independent
workers. The input worker writes exactly one existing request frame and closes
the pipe. Output workers continue draining both pipes but retain at most the
declared limit plus one byte, which is sufficient for deterministic overflow
classification without retaining unbounded provider data.

`run` enforces the invocation timeout with a never-cancelled signal.
`run_with_cancellation` additionally polls a caller-supplied
`CancellationSignal`. Each supervision cycle observes direct-child completion
first, then caller cancellation, then deadline expiry. Once interruption is
observed, fallback is forbidden.

On Unix, interruption sends `SIGTERM` to the direct child, waits the exact
declared `cancellation_grace_ms`, then sends the standard library's forced kill
if the child remains alive. On hosts without a portable graceful-signal
primitive, interruption force-kills immediately. Every successfully launched
direct child that completes normally, is cancelled, or times out is waited and
reaped before the runner returns. Cleanup failures remain separate typed errors.
Timeout and cancellation remain distinct errors and record whether force was
required.

Only a normally exited child proceeds to the existing
`validate_process_transcript` gate. Process exit, protocol validity, event/result
validity, subject equality, operator trust, authority, and artifact acceptance
remain separate claims. The runner refuses `sandboxed` profiles because it
ships no enforcing sandbox backend.

## Rationale

Direct executable-plus-argv launch preserves the federated adapter boundary
without turning pipeline definitions into arbitrary command execution.
Environment clearing and opaque resolution prevent accidental ambient
authority and portable secret evidence. Concurrent bounded transport avoids
pipe deadlock and unbounded retained memory.

The declared grace window gives cooperative providers an opportunity to clean
up while retaining a deterministic terminal bound. Reaping the direct child on
every launched path prevents zombies. Preserving the existing transcript
validator as the sole promotion path avoids creating a second construction path
to `ValidatedExecution`.

## Evidence and assumptions

- Extension resolution, subject matching, and authority preflight have already
  produced their opaque tokens for the exact invocation.
- The caller-selected root is trusted to identify the intended local package
  store; Flow still rejects escaping or substituted subjects.
- Unix direct-child signal semantics are available on the repository's current
  CI host.
- `trusted-unconfined` means the provider can access authority the host exposes
  outside Flow's explicit environment construction. It is not containment.

## Alternatives considered

- **Shell command strings:** rejected because quoting and interpolation would
  permit command substitution and make executable identity ambiguous.
- **Inherit the parent environment:** rejected because undeclared ambient
  variables would bypass the authority profile.
- **Capture all output before checking limits:** rejected because a provider
  could consume unbounded host memory.
- **Treat timeout and cancellation as equivalent:** rejected because operator
  action and deadline expiry are different lifecycle evidence.
- **Force-kill immediately on Unix:** rejected because the contract already
  declares a cancellation grace and cooperative cleanup is valuable.
- **Kill an inferred process tree:** rejected because the unconfined direct-
  child API does not establish a portable, race-free process-group boundary.
- **Ship sandbox enforcement in the runner:** deferred because authenticated
  host enforcement and platform-specific isolation require a separate backend
  contract and conformance matrix.

## Trade-offs

The synchronous poll loop has millisecond-scale observation granularity rather
than real-time guarantees. Unix gains cooperative `SIGTERM`; non-Unix hosts
currently receive immediate forced termination. The extra Unix dependency is
limited to safe process and signal wrappers.

Direct-child reaping does not prove descendant cleanup. An unconfined provider
can spawn descendants that outlive it or retain inherited pipes, so adapters
must not claim process-tree containment. The fresh digest observation also
remains subject to a time-of-check/time-of-exec race because the runner does not
open and execute one descriptor-bound file object.

## Expected consequences

- Flow can execute one exact approved provider program without a shell or
  ambient executable lookup.
- A provider that never reads stdin, never exits, exceeds output limits, emits
  malformed evidence, or ignores graceful termination cannot produce
  `ValidatedExecution`.
- Caller cancellation and deadline expiry are deterministic, typed, and
  terminal for the selected provider attempt.
- Every normally completed, cancelled, or timed-out direct child is reaped
  before return; cleanup failures remain explicit.
- Sandbox-required providers remain fail-closed until a real backend exists.

## Observed outcomes

Flow issue #42 / PR #43 adds the public runner and cancellation APIs, exact
pre-launch observation, scrubbed process construction, supervised stdin,
independent bounded output capture, deadline and cancellation enforcement,
Unix grace/escalation, direct-child reaping, and real-process conformance tests.
It introduces no portable schema or durable-state migration.

## Security, privacy, and authority

Secret values exist only long enough to populate the child environment and
their debug representation is always redacted. Raw stdout and stderr remain
bounded sensitive evidence and are not placed in portable success values or
error messages.

The runner does not authenticate publisher identity, verify signatures or
transparency logs, enforce filesystem/network/subprocess restrictions, provide
a sandbox, contain descendants, or close the observation-to-exec race. Those
limits must remain visible whenever `trusted-unconfined` execution is exposed.

## Review triggers

Review this decision if:

- Flow adds a real sandbox or authenticated enforcement backend;
- descriptor-bound or platform-native launch closes the subject race;
- process-group/job-object containment becomes a supported guarantee;
- cancellation becomes asynchronous or participates in durable run recovery;
- a released provider cannot preserve the stdio/grace contract; or
- a new transport runtime changes polling, signaling, or capture semantics.

## Related artifacts

ADRs 0005, 0006, 0008, and 0009; `flow-architecture`; `flow-roadmap`;
`docs/integrations/process-runner.md`; `docs/integrations/process-transport.md`;
Flow issues #11, #25, and #42; and Flow PR #43.

## Validation

Conformance must cover exact direct launch, literal argv, explicit working
directory, cleared ambient environment, opaque secret resolution and
redaction, stdin framing and closure, concurrent stdout/stderr draining,
limit-plus-one overflow retention, altered pre-launch bytes, sandbox refusal,
normal and nonzero exit, malformed protocol, caller cancellation, deadline
expiry including blocked stdin, cooperative grace, forced escalation, and
direct-child reaping. No rejected path may produce `ValidatedExecution` or
authorize fallback.
