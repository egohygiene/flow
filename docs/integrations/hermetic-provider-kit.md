# Hermetic orchestration provider kit

## Purpose and checkpoint boundary

The hermetic provider kit is Flow-owned conformance infrastructure for issue
#29. Checkpoint #44 establishes its immutable package identity and deterministic
success path. Checkpoint #45 keeps that baseline intact while adding a closed
matrix for provider selection, lifecycle supervision, and protocol outcomes.
Checkpoint #56 adds the first physical artifact-outcome matrix for missing,
extra, and partial outputs. All three checkpoints exercise released public
boundaries without importing a sibling implementation or pretending to be an
Aniflow, Optiflow, or Renderflow algorithm.

The checkpoint composes only public contracts and APIs:

1. materialize the synthetic executable plus MIT license as an immutable
   package;
2. observe the package and install its SHA-256 in the provider manifest,
   operator lock, and execution-subject lock;
3. resolve one stable capability from explicit catalog input;
4. match the exact package and executable subjects;
5. authorize the exact `trusted-unconfined` invocation profile;
6. launch the direct child through `LocalProcessRunner`;
7. validate the JSON Lines events and terminal result as
   `ValidatedExecution`;
8. observe every explicit input and candidate output beneath the selected
   artifact root; and
9. correlate bindings, host observations, events, and the result before
   constructing `AcceptedArtifactSet`.

The manifest and operator lock are finalized external catalog metadata. They
remain outside the package directory whose digest they name, avoiding a
self-referential integrity value.

Exit code zero and output-file existence are evidence at intermediate gates;
neither is accepted as completion.

## Stable synthetic surface

The provider identity is
`org.egohygiene.synthetic-scenario-provider@0.1.0`. Its
`flow/inspect-fixture` capability reuses the synthetic scenario vocabulary
introduced by issue #28. Its one process entrypoint exposes four stable
Flow-domain capability IDs:

| Capability | Role | Content-changing | Output media type |
| --- | --- | --- | --- |
| `flow/inspect-fixture` | Inspect one immutable text fixture | No | `application/vnd.flow.fixture-inspection+json` |
| `flow/transform-fixture` | Produce a synthetic derivative description | Yes | `application/vnd.flow.fixture-transformation+json` |
| `flow/validate-fixture` | Produce synthetic validation evidence | No | `application/vnd.flow.fixture-validation+json` |
| `flow/observe-fixture` | Observe a source without mutating it | No | `application/vnd.flow.fixture-observation+json` |

The observation capability still writes a new evidence artifact. “Read-only”
means that its bound source is not mutated; it is not a claim that the shared
multi-capability provider receives no output-write authority.

Checkpoint #44 executes the inspection capability end to end. Checkpoint #45
uses that same capability for the lifecycle and protocol matrix, and checkpoint
#56 uses it for physical artifact outcomes after protocol-valid execution. The
other three IDs and their declared media/configuration profiles remain frozen
so the later graph fixtures do not invent parallel identities. Their
end-to-end coverage remains explicitly deferred.

## Configuration and deterministic success profile

Configuration uses `flow.hermetic-provider-configuration/v1` with exactly two
string fields. Checkpoint #44 defines `success`; checkpoint #45 expands the
closed `mode` vocabulary, and checkpoint #56 adds three artifact-outcome modes
without changing the schema or the success evidence:

| Field | Checkpoint-1 value | Identity rule |
| --- | --- | --- |
| `mode` | `success` | Included in compact JSON over the sorted configuration map |
| `seed` | `hermetic-success-v1` | Included in the same SHA-256 configuration identity |

The provider requires exactly one bound `text/plain` input and one bound file
output of the capability's declared media type. It recomputes the input digest,
creates the output with create-new semantics, and emits deterministic started,
artifact-produced, and completed events with sequences `0`, `1`, and `2`.
The terminal result names the exact consumed and produced artifact IDs and
records passed binding/input validation plus extension, configuration, and
output-digest provenance.

The output is compact JSON followed by one line feed. It contains the artifact
schema, capability, synthetic operation, configuration digest, seed, and
sorted input identity evidence. It never includes ambient or host-specific
values.

## Checkpoint-2 behavior matrix

The case ID is part of the test harness. A provider `mode` exists only when the
child is actually invoked. `unavailable` and `incompatible` are therefore
prelaunch harness cases, not runtime modes: no provider can consume a
configuration after resolution rejects it.

| Case | Explicit trigger | Owning boundary | Exact outcome | Promotion |
| --- | --- | --- | --- | --- |
| `unavailable` | The exact provider observation has `available: false`. | Resolution / prelaunch | `ResolutionResult::Blocked`; the candidate remains compatible and authorized but unavailable, and `resolved()` is `None`. | No child, `ValidatedExecution`, or `AcceptedArtifactSet`. |
| `incompatible` | A case-local external manifest requires Flow `>=9.0.0`. | Resolution / prelaunch | `ResolutionResult::NoCompatibleProvider`; the candidate is incompatible but remains authorized and available, and `resolved()` is `None`. | No child, `ValidatedExecution`, or `AcceptedArtifactSet`. |
| `warning` | Runtime configuration selects `mode=warning`. | Semantic protocol validation | A complete `Outcome::Produced` result with `partial_result: false` and a redacted `Severity::Warning` diagnostic. A warning is not a separate outcome or event state. | Produces `ValidatedExecution` and, after normal host observation, `AcceptedArtifactSet`; the warning remains evidence distinct from silent success. |
| `partial-result` | Runtime configuration selects `mode=partial-result`. | Semantic result plus artifact-acceptance gate | A protocol-valid `Outcome::Produced` result with `partial_result: true`. | Produces `ValidatedExecution`; `accept_artifacts` returns `ArtifactAcceptanceError::Mismatch` because acceptance requires a complete produced or reused result. |
| `nonzero-after-success` | The provider writes and flushes success-shaped evidence, then exits with code `7`. | Process supervision / termination validation | `ProcessRunnerError::Validation` containing `ExecutionError::ProcessExit { code: Some(7) }`. Termination is rejected before stdout is promoted. | No `ValidatedExecution` or `AcceptedArtifactSet`; the sink observes no event. |
| `await-interruption` / timeout | The provider writes the lifecycle control record, then remains alive until the invocation deadline. | Process supervision | On Unix, `ProcessRunnerError::TimedOut { timeout_ms, forced: false }` after the direct child is reaped. | No `ValidatedExecution` or `AcceptedArtifactSet`. |
| `await-interruption` / cancellation | The caller requests cancellation only after observing the lifecycle control record. | Caller cancellation plus process supervision | On Unix, `ProcessRunnerError::Cancelled { forced: false }` after the direct child is reaped. | No `ValidatedExecution` or `AcceptedArtifactSet`. |
| `stdout-overflow` | The provider writes more than `max_stdout_bytes` to stdout and exits. | Bounded process capture | `ProcessRunnerError::Validation` containing `ExecutionError::ProcessOutputLimit` for `ProcessStream::Stdout`, with `observed = limit + 1`. | No parsed provider evidence, `ValidatedExecution`, or `AcceptedArtifactSet`. |
| `stderr-overflow` | The provider writes more than `max_stderr_bytes` to stderr and exits. | Bounded process capture | `ProcessRunnerError::Validation` containing `ExecutionError::ProcessOutputLimit` for `ProcessStream::Stderr`, with `observed = limit + 1`. | No parsed provider evidence, `ValidatedExecution`, or `AcceptedArtifactSet`. |
| `invalid-event` | Runtime configuration emits a duplicate/non-increasing event sequence. | Semantic event validation | `ProcessRunnerError::Validation` containing `ExecutionError::InvalidEvent`; raw decoded evidence is retained for inspection. | No `ValidatedExecution` or `AcceptedArtifactSet`. |
| `invalid-result` | Runtime configuration emits a terminal result whose authorization identity does not match the invocation. | Semantic result validation | `ProcessRunnerError::Validation` containing `ExecutionError::InvalidResult`; the raw result is retained for inspection. | No `ValidatedExecution` or `AcceptedArtifactSet`. |
| `success-with-host-rejection` | The provider emits valid success-shaped evidence and the caller's authoritative `EventSink` rejects an event. | Host semantic-observation boundary | `ProcessRunnerError::Validation` containing `ExecutionError::EventSink`; decoded events and the result remain inspectable. | No `ValidatedExecution`, fallback, or `AcceptedArtifactSet`. |

## Checkpoint-3a physical artifact outcome matrix

These modes deliberately pass process transport and semantic result validation.
They fail only when Flow observes or accepts physical artifact evidence. The
extra file is a closed fixture behavior, not an automatic discovery feature.

| Case | Explicit trigger | Owning boundary | Exact outcome | Promotion |
| --- | --- | --- | --- | --- |
| `missing-output` | Runtime configuration suppresses creation of the one bound candidate while retaining a complete success-shaped transcript. | Host artifact observation | `observe_artifacts` returns `ArtifactObservationError::Missing` for the exact bound ID and locator. | Produces `ValidatedExecution`; no `ObservedArtifactSet` or `AcceptedArtifactSet`. |
| `extra-output` | The provider creates the bound candidate plus `outputs/undeclared-extra-output.json` and names both IDs in its event and result. | Artifact acceptance | Observation covers only the explicit binding set; `accept_artifacts` returns `ArtifactAcceptanceError::Mismatch` because provider-produced IDs do not exactly match declared outputs. | Produces `ValidatedExecution` and `ObservedArtifactSet`; no `AcceptedArtifactSet`. The undeclared sibling is not discovered or promoted. |
| `partial-output` | The provider writes a deterministic truncated synthetic candidate, reports its exact digest, and sets `partial_result: true`. | Artifact acceptance | Observation succeeds for the bytes that exist; `accept_artifacts` returns `ArtifactAcceptanceError::Mismatch` because acceptance requires a complete produced or reused result. | Produces `ValidatedExecution` and `ObservedArtifactSet`; no `AcceptedArtifactSet`. Flow does not claim generic JSON or provider-native semantic validation. |

The stream-capture `observed` value is a bounded overflow sentinel, not the
provider's total emitted byte count. Each worker drains its stream to EOF but
retains at most the configured limit plus one byte.

Event observation is authoritative but not transactional. An invalid later
event or terminal result can reject the execution after earlier individually
valid events reached the caller's sink; Flow does not claim rollback. Likewise,
the runner receives exactly one already-selected provider and has no catalog or
alternate provider with which to retry after invocation starts.

### Lifecycle synchronization and reaping evidence

`await-interruption` accepts the explicit
`--lifecycle-control outputs/lifecycle-control.json` argument. The provider
writes and syncs this compact, LF-terminated test-control record to a sibling
temporary file, then atomically publishes it before it waits:

```json
{"state":"ready","pid":1234}
```

The PID value is illustrative and host-specific. The control record is outside
the artifact binding set, is never passed to `observe_artifacts`, and never
becomes portable result, provenance, normalized evidence, or a candidate for
`AcceptedArtifactSet`. Its sole purpose is to remove timing guesses from caller
cancellation and to let the Unix test verify that the recorded direct child is
no longer waitable after the runner returns. The timeout case uses a generous
deadline and requires the same ready record, but the current runner starts its
deadline when transport workers are established; it does not expose a separate
"arm timeout after ready" API.

The control record is written beneath the authorized workspace output tree,
never inside the immutable provider package. Reaping claims cover only the
direct child. The kit does not claim descendant discovery, signalling, or
containment. Unix expects graceful default `SIGTERM` handling and therefore
asserts `forced: false`; other hosts may require immediate forced termination.

## Conformance evidence

`tests/hermetic_provider_kit.rs` builds fresh roots with identical package,
input, binding, invocation, configuration, and authority values. The success
baseline requires:

- identical package and executable identities;
- identical resolution, subject, authority, event, result, host-observation,
  accepted-artifact, and output-byte evidence;
- unchanged provider-package observations after execution;
- unchanged input and binding bytes;
- an exact filesystem layout in which the sole new entry is the declared
  candidate output; and
- successful promotion through both `ValidatedExecution` and
  `AcceptedArtifactSet`.

The checkpoint #56 cases reuse the same real process path and additionally
prove that missing output stops at typed host observation, that an explicitly
undeclared sibling never expands the binding set, and that extra or partial
provider evidence stops at typed artifact acceptance. Every case re-observes
the immutable package and verifies that input and binding bytes remain
unchanged.

The fixture source, manifest, lock, materialization rules, offline commands,
and redistribution terms live in
[`tests/fixtures/hermetic-provider`](../../tests/fixtures/hermetic-provider/README.md).

## Authority and non-claims

The manifest requests only named workspace input/binding reads and named
workspace output writes. It requests no environment, subprocess, network, AI,
GPU, source-mutation, destructive, signing, or publication authority. The
runner clears the inherited environment and passes no secret handles.

The local runner profile is `trusted-unconfined`. This checkpoint proves exact
contract correlation, byte identity, bounded transport, and direct-child use
and reaping; it does not claim an operating-system sandbox, filesystem containment,
descriptor-bound execution, publisher authentication, descendant cleanup, or
provider-native semantic validation.

Checkpoint #57 owns corrupt, stale, changed, and contradictory artifact
evidence. Checkpoint #58 owns the single- and multi-provider composition
fixtures. Checkpoint #59 owns completed redistribution documentation and the
final parent #29 requirement-to-test matrix. Durable run state, retry,
checkpoint, and resume remain outside the hermetic provider kit.
