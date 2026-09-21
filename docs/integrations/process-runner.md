# Bounded local process runner

## Purpose and scope

`LocalProcessRunner` turns Flow's validated process contracts into one real
local direct-child lifecycle. It is governed by
[ADR-0010](../architecture/governance/decisions/ADR-0010-bounded-direct-process-supervision.md)
and composes the transport, execution-subject, and authority boundaries without
redefining them.

The runner is deliberately narrow. It executes only the exact locked executable
for an already-resolved and already-authorized `trusted-unconfined` invocation.
It is not a general command runner, shell, provider discovery system, sandbox,
durable run engine, or real holon adapter.

## Public API

| API | Purpose |
| --- | --- |
| `LocalProcessRunner::run` | Execute with the invocation deadline and `NeverCancelled` |
| `LocalProcessRunner::run_with_cancellation` | Execute with the deadline plus a caller-owned `CancellationSignal` |
| `SecretResolver` | Resolve only authorized opaque `secret:` handles into ephemeral child environment values |
| `SecretValue` | Hold one environment value with permanently redacted debug output |
| `ProcessRunnerError` | Distinguish preflight, launch, transport, supervision, interruption, capture, and validation failure |

The cancellation signal is observational, monotonic, and expected to return
promptly for one call. Once Flow observes cancellation, it terminates the
selected attempt and never falls back to another provider.

## Lifecycle

The runner applies these stages in order:

1. Re-observe the package directory and executable file beneath the selected
   root and require exact equality to `ExecutionSubjectLock`.
2. Encode the request through the existing subject and authority preflight.
3. Refuse any isolation profile other than `trusted-unconfined`.
4. Resolve every authorized environment binding from its opaque handle before
   launch; a missing value prevents launch.
5. Canonicalize the root, derive the locked package and executable paths, and
   launch that executable directly with the authorized argv.
6. Set the package directory as cwd, clear the inherited environment, and add
   only the resolved authorized variables.
7. Start independent stdin, stdout, and stderr workers. Stdin receives exactly
   the existing request frame and EOF. Both output streams are drained
   concurrently.
8. Supervise the direct child until normal completion, caller cancellation, or
   deadline expiry.
9. Wait and reap every normally completed, cancelled, or timed-out direct
   child, then join all three transport workers. Report cleanup failures
   separately.
10. Only normal completion proceeds through
    `Orchestrator::validate_process_transcript` and the existing authoritative
    `EventSink`.

## Deadlines and cancellation

The deadline begins once the pipes and transport workers are established. This
includes a provider that refuses to read stdin: request delivery cannot block
deadline enforcement.

Each poll observes states in this order:

1. already-completed direct child;
2. caller cancellation; and
3. elapsed invocation timeout.

Normal completion therefore wins if it is already observable. Otherwise
caller cancellation wins when cancellation and timeout become observable in
the same cycle.

On Unix, Flow sends `SIGTERM`, waits `cancellation_grace_ms`, then force-kills
the direct child if it remains alive. The typed timeout/cancellation error
reports whether forced termination was required. On hosts without a portable
graceful signal, the runner force-kills immediately. Cleanup or reap failure is
reported separately because Flow cannot honestly claim a completed lifecycle.

## Bounded transport

Stdout and stderr remain distinct. Each worker drains until EOF but retains at
most its configured byte limit plus one byte. The extra byte proves overflow;
Flow never needs to retain the rest. After normal exit the existing transcript
validator reports the matching `ProcessOutputLimit` and does not parse or emit
provider records.

Raw streams may contain secrets or private paths. They remain local sensitive
evidence and do not enter `ValidatedExecution` or runner error display.

## Claims kept separate

The runner preserves these independent facts:

- package/executable content equality;
- publisher declaration and any future authenticity evidence;
- configured operator trust;
- selected authority and isolation;
- direct-child launch and completion;
- process transport validity;
- provider event/result semantic validity; and
- downstream artifact observation and acceptance.

An exit code of zero or a valid JSON result is never sufficient by itself.
Timeout, cancellation, nonzero exit, overflow, malformed output, observer
rejection, or any preflight mismatch returns no `ValidatedExecution`.

## Explicit limitations

- `trusted-unconfined` is not filesystem, network, subprocess, GPU, signing,
  publication, or destructive-operation containment.
- Flow signals and reaps only the direct child. It does not claim process-tree
  or descendant cleanup.
- Descendants may retain inherited pipes and delay stream-worker completion.
- Fresh subject observation is not descriptor-bound execution; bytes may change
  in the observation-to-exec window.
- `sandboxed` profiles remain unsupported until a real enforcing backend exists.
- The runner does not persist state, retry, resume, discover providers, or
  integrate Aniflow, Optiflow, or Renderflow.

## Conformance

Unix real-process tests cover direct launch, literal argv, explicit cwd, empty
ambient environment, opaque secrets, exact pre-launch bytes, sandbox refusal,
stdin framing/closure, independent bounded streams, limit-plus-one overflow,
normal and nonzero exit, malformed output, deterministic cancellation,
deadline expiry including blocked stdin, cooperative grace, forced escalation,
redaction, and direct-child reap ownership.
