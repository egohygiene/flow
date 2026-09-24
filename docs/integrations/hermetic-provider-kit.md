# Hermetic orchestration provider kit

## Purpose and checkpoint boundary

The hermetic provider kit is Flow-owned conformance infrastructure for issue
#29. Checkpoint #44 establishes its immutable package identity and deterministic
success path. Checkpoint #45 keeps that baseline intact while adding a closed
matrix for provider selection, lifecycle supervision, and protocol outcomes.
Checkpoint #56 adds the first physical artifact-outcome matrix for missing,
extra, and partial outputs. Checkpoint #57 closes corrupt, contradictory,
changed, and stale artifact-evidence cases and hardens final artifact freshness.
Checkpoint #58 adds deterministic single-provider and two-provider composition
fixtures. Checkpoint #59 closes the package, behavior, requirement-evidence, and
roadmap documentation. The five executable checkpoints exercise released public
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
   artifact root;
9. correlate bindings, host observations, events, and the result; then
10. re-observe the retained root and require unchanged evidence immediately
    before constructing `AcceptedArtifactSet`.

The manifest and operator lock are finalized external catalog metadata. They
remain outside the package directory whose digest they name, avoiding a
self-referential integrity value.

Exit code zero and output-file existence are evidence at intermediate gates;
neither is accepted as completion.

## Immutable package, finalization, and verification

The primary hashed package root has exactly this build-specific layout:

```text
packages/hermetic-provider/
├── LICENSE
└── flow-hermetic-provider[.exe]
```

The two multi-provider package roots replace `hermetic-provider` with
`hermetic-provider-inspector` or `hermetic-provider-renderer` and add one
`PROVIDER-IDENTITY` regular file. Its exact LF-terminated bytes are respectively
`org.egohygiene.synthetic-inspector\n` and
`org.egohygiene.synthetic-renderer\n`. The fixed marker gives the packages
different directory identities while both retain the same generic executable
bytes and executable digest.

The manifest, operator lock, execution-subject lock and observation evidence,
artifact bindings, and workspace remain outside every hashed package root.
Adding any of them to a package after observation would change the package
identity; placing a manifest inside the directory whose digest it declares
would also create a self-reference.

An offline finalization uses this exact order:

1. At the intended repository revision, populate Cargo's dependency cache while
   network access is available. The repository does not vendor dependencies.
2. Disconnect from the network and build the provider with
   `cargo build --bin flow-hermetic-provider --locked --offline`.
3. Create a new, otherwise empty package root. Copy the built executable and
   repository `LICENSE`; add the exact identity marker only for an inspector or
   renderer composition package.
4. Call Flow's `observe_artifacts` for that directory and retain the returned
   SHA-256 identity over the canonical `flow.directory-manifest/v1`. Separately
   hash the executable's raw bytes with SHA-256. A shell hash of the directory
   is not the Flow package digest.
5. Parse the checked-in manifest and lock templates. Replace their all-zero
   sentinel integrity values with the observed package digest. For a composition
   package, also finalize the provider ID, configured location, and capability
   resolution entries. Validate both documents before catalog inspection.
6. Construct `flow.execution-subject-lock/v1` with the same package digest and
   the raw executable digest. Call `observe_execution_subjects`; accept the
   package only when the fresh `flow.execution-subject-observations/v1` evidence
   exactly matches both locked digests and the correlated invocation context.
7. Re-observe the subjects before launch and after execution. Any net change to
   the executable, license, marker, file kind, or directory membership that is
   present at re-observation rejects the locked package.
8. Run `cargo test --test hermetic_provider_kit --locked --offline` to exercise
   materialization, finalization, tamper rejection, public execution, and
   artifact acceptance without an external service.

`finalized_packages_have_exact_layout_and_correlated_digests` covers the exact
layouts, primary manifest/lock finalization, digest correlation, and changed-
license rejection. Together with the success/composition tests and
`tests/execution_subjects.rs::altered_package_or_executable_bytes_never_match_the_lock`,
it covers steps 3–7. No universal package digest is checked in: executable bytes
can differ by target, toolchain, and build profile. The immutable unit is one
finalized materialization and its correlated Flow lock and observation
evidence, not a cross-platform reproducible-build claim.

The supported redistributable artifact is this MIT-licensed synthetic source
bundle plus its locked build instructions. The repository license is copied
into every local materialization. A built executable may incorporate a target-
specific subset of third-party dependency versions resolved by `Cargo.lock`.
This repository neither assembles nor audits a distributable third-party notice
bundle, so a local materialization is conformance input rather than a turnkey
binary distribution. Anyone redistributing that binary must first determine
and satisfy the licenses and notices of the dependencies actually included in
the build. The all-zero templates are never final metadata and must not be
redistributed as if they verified a package.

## Stable synthetic surface

The provider identity is
`org.egohygiene.synthetic-scenario-provider@0.1.0`. Its
`flow/inspect-fixture` capability reuses the synthetic scenario vocabulary
introduced by issue #28. Its one process entrypoint exposes four stable
Flow-domain capability IDs:

| Capability | Accepted input | Content-changing | Output media type |
| --- | --- | --- | --- |
| `flow/inspect-fixture` | `text/plain` | No | `application/vnd.flow.fixture-inspection+json` |
| `flow/transform-fixture` | `text/plain` or inspection JSON | Yes | `application/vnd.flow.fixture-transformation+json` |
| `flow/validate-fixture` | `text/plain` | No | `application/vnd.flow.fixture-validation+json` |
| `flow/observe-fixture` | `text/plain` | No | `application/vnd.flow.fixture-observation+json` |

The observation capability still writes a new evidence artifact. “Read-only”
means that its bound source is not mutated; it is not a claim that the shared
multi-capability provider receives no output-write authority.

Checkpoint #44 executes the inspection capability end to end. Checkpoint #45
uses that same capability for the lifecycle and protocol matrix, and checkpoint
#56 uses it for physical artifact outcomes after protocol-valid execution.
Checkpoint #57 adds two provider evidence modes and host-harness freshness
cases without changing the capability surface. Checkpoint #58 executes
`flow/transform-fixture` for the first time and narrowly adds inspection JSON
to that capability's accepted inputs so a typed handoff is possible. It does
not widen any other capability profile or invent a real holon capability.
Validation and observation remain stable declared synthetic IDs but are not
claimed as executed graph stages by this minimal checkpoint.

## Configuration and deterministic success profile

Configuration uses `flow.hermetic-provider-configuration/v1` with exactly two
string fields. Checkpoint #44 defines `success`; checkpoint #45 expands the
closed `mode` vocabulary, checkpoint #56 adds three artifact-outcome modes, and
checkpoint #57 adds two artifact-evidence modes without changing the schema or
the success evidence:

| Field | Checkpoint-1 value | Identity rule |
| --- | --- | --- |
| `mode` | `success` | Included in compact JSON over the sorted configuration map |
| `seed` | `hermetic-success-v1` | Included in the same SHA-256 configuration identity |

The provider requires exactly one bound input accepted by the selected
capability and one bound file output of that capability's declared media type.
Inspection, validation, and observation accept `text/plain`; transformation
additionally accepts the exact inspection JSON type used by checkpoint #58. It
recomputes the input digest,
creates the output with create-new semantics, and emits deterministic started,
artifact-produced, and completed events with sequences `0`, `1`, and `2`.
The terminal result names the exact consumed and produced artifact IDs and
records passed binding/input validation plus extension, configuration, and
output-digest provenance.

The artifact provenance entry remains provider-contributed free-form
`flow.extension-result/v1` evidence. Flow validates that its value is nonempty,
but does not parse it as an authoritative expected output digest or use it to
promote host identity. Checkpoint #57 instead protects freshness by retaining
the Flow-owned observation context and requiring an equal final host
re-observation immediately before artifact acceptance.

The output is compact JSON followed by one line feed. It contains the artifact
schema, capability, synthetic operation, configuration digest, seed, and
sorted input identity evidence. It never includes ambient or host-specific
values.

## Closed behavior catalog

The catalog begins with the complete baseline and then closes every configured
or harness-owned refusal case. Each case names one trigger, owning Flow boundary,
exact outcome, and strongest promotion reached.

### Package finalization outcomes

| Case | Explicit trigger | Owning boundary | Exact outcome | Promotion |
| --- | --- | --- | --- | --- |
| `finalized-package` | Materialize the exact binary, license, and optional fixed provider marker; install the observed package digest in the manifest and extension/subject locks and the raw executable digest in the subject lock. | Artifact observation, catalog inspection, and execution-subject matching | The primary and composition layouts are exact; manifest, lock, subject lock, and fresh observations correlate the same package/executable content identities. | Produces a catalog candidate and `MatchedExecutionSubjects`; it does not by itself launch a child or accept an output artifact. |
| `changed-package-after-finalization` | Change package bytes after its digests are locked; the focused fixture changes `LICENSE`, while execution-subject tests also alter package or executable bytes. | Fresh execution-subject observation | `ExecutionSubjectError::ContentMismatch` names the package or executable subject. | No fresh `MatchedExecutionSubjects`, child launch, `ValidatedExecution`, or `AcceptedArtifactSet`. |

### Deterministic success baseline

| Case | Explicit trigger | Owning boundary | Exact outcome | Promotion |
| --- | --- | --- | --- | --- |
| `success` | Runtime configuration selects `mode=success` with an explicit seed and one valid input/output binding. | Full public chain: resolution, subject observation, authority, process validation, artifact observation, and acceptance | `Outcome::Produced`, `partial_result: false`, exact started/artifact/completed events, exact consumed/produced IDs, and equal provider/host output identity. | Produces `ValidatedExecution`, an opaque observed-artifact token, and `AcceptedArtifactSet`; two fresh roots retain equal normalized evidence and output bytes. |

### Selection, lifecycle, and protocol outcomes

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

### Physical artifact outcomes

These modes deliberately pass process transport and semantic result validation.
They fail only when Flow observes or accepts physical artifact evidence. The
extra file is a closed fixture behavior, not an automatic discovery feature.

| Case | Explicit trigger | Owning boundary | Exact outcome | Promotion |
| --- | --- | --- | --- | --- |
| `missing-output` | Runtime configuration suppresses creation of the one bound candidate while retaining a complete success-shaped transcript. | Host artifact observation | `observe_artifacts` returns `ArtifactObservationError::Missing` for the exact bound ID and locator. | Produces `ValidatedExecution`; no `ObservedArtifactSet` or `AcceptedArtifactSet`. |
| `extra-output` | The provider creates the bound candidate plus `outputs/undeclared-extra-output.json` and names both IDs in its event and result. | Artifact acceptance | Observation covers only the explicit binding set; `accept_artifacts` returns `ArtifactAcceptanceError::Mismatch` because provider-produced IDs do not exactly match declared outputs. | Produces `ValidatedExecution` and `ObservedArtifactSet`; no `AcceptedArtifactSet`. The undeclared sibling is not discovered or promoted. |
| `partial-output` | The provider writes a deterministic truncated synthetic candidate, reports its exact digest, and sets `partial_result: true`. | Artifact acceptance | Observation succeeds for the bytes that exist; `accept_artifacts` returns `ArtifactAcceptanceError::Mismatch` because acceptance requires a complete produced or reused result. | Produces `ValidatedExecution` and `ObservedArtifactSet`; no `AcceptedArtifactSet`. Flow does not claim generic JSON or provider-native semantic validation. |

### Artifact evidence and freshness outcomes

The first two cases are real provider modes. The remaining cases are
host-harness operations because only Flow can create an opaque observation
token and only its caller can attempt to reuse that token with changed host or
correlation context.

| Case | Explicit trigger | Owning boundary | Exact outcome | Promotion |
| --- | --- | --- | --- | --- |
| `corrupt-artifact-evidence` | The provider clears the otherwise valid artifact-provenance value while still creating the candidate output. | Semantic result validation | `ProcessRunnerError::Validation` contains `ExecutionError::InvalidResult` with `result.provenance[2].value: must not be empty`. | Valid events and the raw invalid result remain inspectable, and the output may exist; no `ValidatedExecution` or `AcceptedArtifactSet`. “Corrupt” names explicit evidence corruption, not provider-native format validation. |
| `contradictory-artifact-evidence` | The result names the bound output while the `artifact-produced` event omits it. | Artifact acceptance | The transcript remains protocol-valid; `accept_artifacts` returns `ArtifactAcceptanceError::Mismatch` with `artifact-produced events do not exactly match declared outputs`. | Produces `ValidatedExecution` and `ObservedArtifactSet`; no `AcceptedArtifactSet`. |
| `changed-output-after-observation` | After normal provider success and initial host observation, the harness overwrites the bound output with deterministic replacement bytes. | Final artifact-freshness gate | `accept_artifacts` returns `ArtifactAcceptanceError::ObservationChanged` (`bound artifact evidence changed after host observation`). | Produces `ValidatedExecution` and the initial `ObservedArtifactSet`; no `AcceptedArtifactSet`. |
| `changed-input-after-observation` | After normal success and initial host observation, the artifact-library harness overwrites a bound input. | Final artifact-freshness gate | `accept_artifacts` returns `ArtifactAcceptanceError::ObservationChanged`. | Produces `ValidatedExecution` and the initial `ObservedArtifactSet`; no `AcceptedArtifactSet`. |
| `removed-output-after-observation` | After normal provider success and initial host observation, the harness removes the bound output. | Final artifact re-observation | `accept_artifacts` returns `ArtifactAcceptanceError::Reobservation` containing `ArtifactObservationError::Missing` for the exact output ID and locator. | Produces `ValidatedExecution` and the initial `ObservedArtifactSet`; no `AcceptedArtifactSet`. |
| `stale-invocation-and-binding-contexts` | The harness first supplies a changed run identity with valid execution evidence, then separately supplies a changed binding set with the valid observation token. | Artifact acceptance correlation | The invocation case returns `ArtifactAcceptanceError::Mismatch` with `validated execution context does not match the supplied invocation`; the binding case returns the same variant with `artifact bindings changed after host observation`. | Neither stale context can construct `AcceptedArtifactSet`. |
| `invalid-portable-observation` | Corrupt a portable digest or reorder a directory manifest before correlation. | `HostArtifactObservationSet::validate` | Validation rejects the exact digest path as non-hexadecimal or the manifest path as not unique and sorted. | Portable evidence cannot recreate the opaque observed token or reach artifact acceptance. |

The artifact-binding library cases additionally prove that changed input bytes
after initial observation return `ObservationChanged`, malformed portable
digests fail `HostArtifactObservationSet::validate`, contradictory directory
manifests fail before correlation, and deserialized portable evidence cannot
recreate the opaque observation token. The final re-observation detects a net
evidence change; it is not an atomic snapshot and does not interpret provider
artifact provenance as authoritative host identity.

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

## Deterministic compositions

Checkpoint #58 adds two fixed, test-owned compositions. Both declare the exact
stage array `inspect` then `transform`; the latter depends on the former and
consumes its one accepted output.

| Fixture | Explicit trigger and providers | Owning boundary | Exact outcome | Promotion |
| --- | --- | --- | --- | --- |
| `single-provider-composition` | Fixed `inspect` → `transform`; `org.egohygiene.synthetic-scenario-provider@0.1.0` owns both stages. | Each stage independently crosses resolution, subject observation, authority, runner, artifact observation, and acceptance. | Stage two consumes the exact accepted `artifact:inspection-report` from stage one with equal ID, port, media type, kind, locator, and digest. Two fresh roots retain equal normalized evidence and bytes. | Both stages produce `ValidatedExecution`, an opaque observed-artifact token, and `AcceptedArtifactSet`. |
| `multi-provider-composition` | Fixed `inspect` → `transform`; `org.egohygiene.synthetic-inspector@0.1.0` then `org.egohygiene.synthetic-renderer@0.1.0`. | The same complete public boundary per stage, with distinct provider/package identity. | The exact accepted handoff is preserved; package digests differ because of fixed markers while executable digests remain equal. Two fresh roots retain equal normalized evidence and bytes. | Both stages produce `ValidatedExecution`, an opaque observed-artifact token, and `AcceptedArtifactSet`. |

The exact handoff in both fixtures is:

| Field | Value |
| --- | --- |
| Artifact ID | `artifact:inspection-report` |
| Port | `port:inspection-report` |
| Media type | `application/vnd.flow.fixture-inspection+json` |
| Kind | `file` |
| Locator | `outputs/inspection-report.json` |
| Digest | The SHA-256 digest carried only from stage one's `AcceptedArtifactSet` |

The multi-provider packages contain different fixed identity markers and
therefore have different package digests. They intentionally reuse the same
generic executable bytes, so the executable digest is equal; this is a
provider-boundary test, not an algorithm-diversity claim.

For each stage, the harness calls public catalog resolution, execution-subject
observation, process authorization, `LocalProcessRunner`, artifact observation,
and artifact acceptance. Only the first stage's `AcceptedArtifactSet` may
supply the second stage's input binding. Artifact ID, port, media type, kind,
locator, and digest remain equal in the shared workspace. Provider, package,
executable, invocation, configuration, authority, execution, observation, and
accepted-artifact identities are retained as normalized stage evidence.

Each fixture runs twice in fresh roots. Equal normalized evidence and
byte-identical accepted outputs are required. The code explicitly calls stage
zero and stage one; it has no ready queue, topological scheduler, retry,
checkpoint, resume, durable plan/run state, or scenario-manifest execution.

### Exact behavior-to-test index

All unqualified test names below live in `tests/hermetic_provider_kit.rs`.
Supporting artifact-library tests are explicitly prefixed with
`tests/artifacts.rs::`.

| Cases | Exact test |
| --- | --- |
| Template identity and permissions | `templates_freeze_the_provider_identity_and_four_capabilities` |
| Package layout, digest correlation, and representative tamper rejection | `finalized_packages_have_exact_layout_and_correlated_digests`; `tests/execution_subjects.rs::altered_package_or_executable_bytes_never_match_the_lock` |
| `success` | `inspection_is_deterministic_across_fresh_process_and_artifact_boundaries` |
| `unavailable` | `unavailable_provider_is_blocked_before_invocation_with_retained_evidence` |
| `incompatible` | `incompatible_provider_is_rejected_before_invocation_with_retained_evidence` |
| `warning`, `partial-result` | `warning_and_partial_results_remain_semantically_distinct` |
| `missing-output` | `missing_bound_output_fails_observation_after_valid_provider_success` |
| `extra-output` | `extra_provider_output_fails_exact_acceptance_without_auto_discovery` |
| `partial-output` | `partial_output_is_observable_but_cannot_be_accepted` |
| `corrupt-artifact-evidence` | `corrupt_artifact_evidence_never_promotes_execution` |
| `contradictory-artifact-evidence` | `contradictory_artifact_evidence_fails_exact_acceptance` |
| `changed-output-after-observation` | `changed_output_after_host_observation_fails_freshness_gate` |
| Stale invocation and binding contexts | `stale_invocation_and_binding_contexts_cannot_reuse_valid_evidence` |
| `removed-output-after-observation` | `tests/artifacts.rs::removed_output_after_observation_returns_typed_reobservation_error` |
| Changed input after observation | `tests/artifacts.rs::changed_input_after_observation_cannot_be_accepted` |
| Invalid portable observations | `tests/artifacts.rs::corrupt_or_contradictory_portable_observations_are_invalid_before_correlation` |
| `nonzero-after-success` | `nonzero_after_success_never_promotes_provider_evidence` |
| `invalid-event`, `invalid-result` | `invalid_event_and_result_return_typed_semantic_errors` |
| `success-with-host-rejection` | `authoritative_host_rejection_blocks_valid_provider_success` |
| `stdout-overflow`, `stderr-overflow` | `stdout_and_stderr_overflow_retain_only_the_limit_sentinel` |
| `await-interruption` timeout and cancellation | `timeout_and_readiness_gated_cancellation_reap_the_direct_child` (Unix) |
| `single-provider-composition` | `single_provider_composition_preserves_exact_handoff_and_determinism` |
| `multi-provider-composition` | `multi_provider_composition_preserves_exact_handoff_and_determinism` |

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

The checkpoint #57 provider modes distinguish invalid result evidence from a
protocol-valid event/result contradiction. Its host-harness cases prove that a
changed output, a removed output, a stale invocation, or a changed binding
snapshot cannot reuse otherwise valid evidence to construct
`AcceptedArtifactSet`. The focused artifact tests also cover changed input
evidence and malformed or contradictory portable observations.

The checkpoint #58 tests additionally require exact accepted-output-to-input
equality across both fixed compositions, stable ordered dependency evidence,
the expected provider selection at each stage, unchanged locked subjects, and
fresh-root repeat equivalence. The checked-in scenario manifests remain
declarative conformance intent rather than runtime instructions.

The fixture source, manifest, lock, materialization rules, offline commands,
and redistribution terms live in
[`tests/fixtures/hermetic-provider`](../../tests/fixtures/hermetic-provider/README.md).

## Parent #29 requirement-to-test matrix

The matrix maps every parent acceptance criterion to exact executable evidence
and its documentation owner. Test names without a path are in
`tests/hermetic_provider_kit.rs`.

| #29 acceptance criterion | Exact tests | Documentation and residual |
| --- | --- | --- |
| The kit implements only the accepted public Flow process/artifact contract. | `templates_freeze_the_provider_identity_and_four_capabilities`; `inspection_is_deterministic_across_fresh_process_and_artifact_boundaries`; `single_provider_composition_preserves_exact_handoff_and_determinism`; `multi_provider_composition_preserves_exact_handoff_and_determinism` | This document, [process runner](process-runner.md), [execution subjects](execution-subjects.md), [authority](authority-isolation.md), and [artifact bindings](artifact-bindings.md). The fixture is an external integration test over public `flow` exports; it is not a real holon adapter, algorithm, or scheduler. |
| Identical configuration and inputs produce identical normalized evidence and artifact digests. | `inspection_is_deterministic_across_fresh_process_and_artifact_boundaries`; `single_provider_composition_preserves_exact_handoff_and_determinism`; `multi_provider_composition_preserves_exact_handoff_and_determinism` | The deterministic profile and composition sections above. The claim is bounded to these fixtures, not a general scheduler or cross-toolchain reproducible build. |
| Every configured failure is intentional, named, bounded, and distinguishable. | Every exact function named in the [behavior-to-test index](#exact-behavior-to-test-index), including the Unix lifecycle function and prefixed artifact-library tests | The closed behavior catalog above names each trigger, owner, exact outcome, and promotion. Direct-child reap proof is Unix-specific; other hosts retain typed interruption without that Unix assertion. |
| Provider exit zero or file existence cannot establish accepted execution. | `missing_bound_output_fails_observation_after_valid_provider_success`; `extra_provider_output_fails_exact_acceptance_without_auto_discovery`; `partial_output_is_observable_but_cannot_be_accepted`; `corrupt_artifact_evidence_never_promotes_execution`; `contradictory_artifact_evidence_fails_exact_acceptance`; `invalid_event_and_result_return_typed_semantic_errors`; `authoritative_host_rejection_blocks_valid_provider_success`; converse case `nonzero_after_success_never_promotes_provider_evidence` | The catalog and the separate process/artifact gates above. Only Flow-owned semantic validation plus exact host correlation can construct the opaque accepted token. |
| Source inputs remain byte-for-byte unchanged. | `inspection_is_deterministic_across_fresh_process_and_artifact_boundaries`; `single_provider_composition_preserves_exact_handoff_and_determinism`; `multi_provider_composition_preserves_exact_handoff_and_determinism`; every named lifecycle/artifact adversary function in the [behavior-to-test index](#exact-behavior-to-test-index) through `assert_immutable_workspace_bytes`; `tests/artifacts.rs::changed_input_after_observation_cannot_be_accepted` | The conformance-evidence section. Inputs and bindings are snapshotted; composition also asserts the original source bytes directly. |
| No network, upload, publication, signing, arbitrary subprocess, or inherited-secret access occurs. | `templates_freeze_the_provider_identity_and_four_capabilities`; `inspection_is_deterministic_across_fresh_process_and_artifact_boundaries`; `single_provider_composition_preserves_exact_handoff_and_determinism`; `multi_provider_composition_preserves_exact_handoff_and_determinism`; `tests/process_runner.rs::runner_launches_exact_executable_with_scrubbed_process_state` (Unix); `tests/process_runner.rs::runner_refuses_sandbox_claims_without_an_enforcing_backend` (Unix) | The authority section below. The manifest requests none of those powers, invocations use empty secret handles and `NoSecrets`, the runner clears the environment, and source inspection finds no such fixture path. Because execution is `trusted-unconfined`, this is contract, source, and runner evidence—not operating-system syscall containment. |
| Package, documentation, examples, and repository validation pass. | `finalized_packages_have_exact_layout_and_correlated_digests`; the full Rust and repository-validator jobs in `.github/workflows/ci.yml` | This document, the fixture README, integration index, and `ROADMAP.md`. CI checks Rust 1.85 and stable formatting, clippy, all targets, Rustdoc, examples, crate packaging, contracts, deterministic sources, and repository specifications/skills/agents. The focused offline command is documented for an already populated cache. |

## Authority and non-claims

The manifest requests only named workspace input, binding, and prior-output
reads plus named workspace output writes. Prior-output read authority permits
the second fixed stage to consume an accepted artifact; it does not permit
input mutation. The provider requests no environment, subprocess, network, AI,
GPU, source-mutation, destructive, signing, or publication authority. The
runner clears the inherited environment and passes no secret handles.

The local runner profile is `trusted-unconfined`. This checkpoint proves exact
contract correlation, byte identity, bounded transport, and direct-child use
and reaping; it does not claim an operating-system sandbox, filesystem
containment, atomic artifact observation, descriptor-bound execution,
publisher authentication, descendant cleanup, or provider-native semantic
validation.

## Residual gaps

- Offline build and test require a previously populated Cargo cache; dependencies
  are locked but not vendored.
- Package and executable digests identify one target/toolchain/profile build.
  The kit does not claim bit-for-bit reproducible compilation across platforms.
- Flow's directory identity covers canonical names, kinds, bytes, and recursive
  structure, not ownership, timestamps, extended attributes, or an executable
  permission guarantee.
- The package includes the repository MIT license but no generated consolidated
  third-party notice bundle. The supported redistributable artifact is the
  source bundle; binary redistributors retain responsibility for the licenses
  and notices of the dependencies actually included in their build.
- `trusted-unconfined` correlation does not enforce filesystem, network, or
  subprocess containment; deny environment access at the operating-system
  boundary; observe upload/publication/signing syscalls; authenticate a
  publisher; verify signatures or a transparency log; bind a later open
  executable object; or contain descendants.
- Artifact freshness is a final equal re-observation, not an atomic filesystem
  snapshot or provider-native semantic validation.
- The compositions are fixed test-owned calls. There is no scenario-manifest
  executor, ready queue, production DAG scheduler, retry, durable plan/run
  state, checkpoint, resume, public Flow CLI, or real holon algorithm.
- `main.rs` contains a source-level unit test for the closed behavior spellings,
  but Cargo declares the fixture binary with `test = false`. Active integration
  tests exercise every declared mode; unknown-mode rejection does not yet have
  its own active integration case.

## #30 handoff and parent closure

After PR #60 is merged and default-branch CI is green, issues #46 and #29 may be
closed and #30 becomes unblocked. #30 inherits the stable package source and
finalization procedure, four synthetic capability IDs, closed behavior modes,
typed expected outcomes, and the public
resolve → observe subjects → authorize → run → observe artifacts → accept
harness documented here.

That handoff does not complete #30's broader work. #30 still owns executable,
machine-readable scenario coverage for version/manifest/lock/report/provenance
and schema disagreements; conflict and fallback; the full path, symlink, kind,
duplicate, digest, and undeclared-output families; diagnostic/privacy canaries;
and explicit PR-tier budgets. The checked-in scenario manifests remain
declarative, and recovery, retry, checkpoint, and resume remain later work.
