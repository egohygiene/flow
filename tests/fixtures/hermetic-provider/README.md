# Hermetic provider fixture

This directory is the MIT-licensed, redistribution-safe source bundle for
Flow's synthetic process provider. A locally built executable is conformance
input, not a turnkey binary distribution: a binary distributor must separately
satisfy the licenses and notices of the dependencies in its target-specific
build. This is not a production holon adapter or a public Flow CLI.

## Stable identity

- Extension: `org.egohygiene.synthetic-scenario-provider@0.1.0`
- Publisher declaration: `org.egohygiene`
- Process mode: `hermetic-process`
- Entrypoint: `flow-hermetic-provider`

The executable composition fixtures also finalize the same generic manifest
under the already declared synthetic identities
`org.egohygiene.synthetic-inspector@0.1.0` and
`org.egohygiene.synthetic-renderer@0.1.0`. Their package directories include a
provider-identity marker, so the two package digests are distinct while the
executable digest remains exactly equal. This proves separate provider
boundaries, not separate algorithms.

The manifest freezes four Flow-owned synthetic capabilities:

| Capability | Accepted input type | Candidate artifact type |
| --- | --- | --- |
| `flow/inspect-fixture` | `text/plain` | `application/vnd.flow.fixture-inspection+json` |
| `flow/transform-fixture` | `text/plain` or `application/vnd.flow.fixture-inspection+json` | `application/vnd.flow.fixture-transformation+json` |
| `flow/validate-fixture` | `text/plain` | `application/vnd.flow.fixture-validation+json` |
| `flow/observe-fixture` | `text/plain` | `application/vnd.flow.fixture-observation+json` |

“Read-only” describes the source boundary: the provider never mutates an input.
Every capability writes a new, explicitly bound evidence artifact, so the
shared manifest requests a narrowly named output-write grant.

## Package materialization

`extension-manifest.v1.json` and `extension-lock.v1.json` are external metadata
templates containing an all-zero integrity sentinel. It is never a verified
package checksum. The primary hashed package root contains exactly:

```text
packages/hermetic-provider/
├── LICENSE
└── flow-hermetic-provider[.exe]
```

The inspector and renderer roots use
`packages/hermetic-provider-inspector/` and
`packages/hermetic-provider-renderer/`. Each adds exactly one regular
`PROVIDER-IDENTITY` file beside the same executable and license. Its fixed
LF-terminated bytes are `org.egohygiene.synthetic-inspector\n` or
`org.egohygiene.synthetic-renderer\n`.

The finalized manifest, operator lock, execution-subject lock and observation
evidence, artifact bindings, and workspace remain outside the hashed package
root. Otherwise the declared directory identity would be self-referential or
would change when run evidence is added.

First populate Cargo's dependency cache while network access is available. The
repository locks but does not vendor dependencies. After disconnecting, build
and exercise the supported package path with:

```console
cargo build --bin flow-hermetic-provider --locked --offline
cargo test --test hermetic_provider_kit --locked --offline
```

Finalize one materialization in this order:

1. Create a new, otherwise empty package root and copy the built executable and
   repository `LICENSE`; add the fixed marker only for a composition identity.
2. Use Flow's `observe_artifacts` to compute the package SHA-256 over its
   canonical `flow.directory-manifest/v1`. Separately compute SHA-256 over the
   executable's raw bytes. A shell hash of the directory is not the Flow
   package digest.
3. Clone the manifest and operator-lock templates, replace both all-zero values
   with the observed package digest, and validate both documents. A composition
   clone also finalizes the provider ID, configured location, and capability
   resolution entries.
4. Put that package digest and the raw executable digest in the
   `flow.execution-subject-lock/v1` record. Freshly call
   `observe_execution_subjects` and require the resulting
   `flow.execution-subject-observations/v1` digests to match exactly before
   launch and again after execution.
5. Reject any net change to executable, license, marker, kind, or package
   membership that is present at a fresh observation.

`finalized_packages_have_exact_layout_and_correlated_digests` covers exact
layouts, primary manifest/lock finalization, digest correlation, and changed-
license rejection. Together with the deterministic success/composition tests
and `tests/execution_subjects.rs`, it covers prelaunch and post-run
re-observation plus altered package/executable rejection. Build outputs are
target/toolchain/profile-specific, so no universal binary or package digest is
checked in and no cross-platform reproducible-build claim is made.

At runtime the host passes only literal long-form arguments:

- `--artifact-root <host path>` selects the explicit workspace root.
- `--artifact-bindings <portable locator>` selects one
  `flow.artifact-bindings/v1` document beneath that root.
- `--lifecycle-control <portable locator>` is optional and is accepted only by
  the `await-interruption` and `cleanup-cancelled` lifecycle modes. The conformance harness uses
  `outputs/lifecycle-control.json`.

The provider accepts one LF-terminated `flow.extension-invocation/v1` document
on standard input. It validates the invocation and configuration identity
before selecting one closed behavior. Artifact-producing modes additionally
validate the binding set and actual input digest. Successful candidate artifact
bytes depend only on the capability, configuration, and immutable input
identity. They contain no time, hostname, process ID, temporary path, random
value, environment value, or external-service result.

## Closed behavior vocabulary

Configuration always contains exactly the `mode` and `seed` string fields. The
runtime `mode` values are:

| Mode | Provider contribution |
| --- | --- |
| `success` | Writes one deterministic candidate artifact and a complete valid transcript. |
| `warning` | Writes the same complete evidence with a redacted warning diagnostic; warning is not a distinct terminal outcome. |
| `partial-result` | Writes a complete candidate and a protocol-valid produced result whose `partial_result` flag is true. |
| `missing-output` | Emits a complete valid transcript for the bound candidate without creating its file, so host observation returns the exact typed missing-artifact error. |
| `extra-output` | Writes the bound candidate plus one closed, bounded undeclared sibling and names both in the artifact event and result; observation remains binding-driven and acceptance rejects the extra provider declaration. |
| `partial-output` | Writes a deterministic truncated synthetic candidate, reports its exact byte digest, and sets `partial_result` so acceptance rejects incomplete evidence without claiming generic format validation. |
| `corrupt-artifact-evidence` | Writes the normal candidate but clears the artifact-provenance value so semantic result validation returns an exact invalid-result error before `ValidatedExecution`. |
| `contradictory-artifact-evidence` | Writes the normal candidate and names it in the result while omitting it from the artifact-produced event; protocol validation succeeds, but artifact acceptance rejects the contradiction. |
| `nonzero-after-success` | Flushes success-shaped stdout, then exits with code `7`. |
| `await-interruption` | Creates the explicit lifecycle control record, then waits for host timeout or cancellation. |
| `stdout-overflow` | Emits deterministic stdout beyond the invocation limit. |
| `stderr-overflow` | Emits deterministic stderr beyond the invocation limit. |
| `invalid-event` | Emits a duplicate/non-increasing event sequence. |
| `invalid-result` | Emits a result with an authorization identity that conflicts with the invocation. |
| `success-with-host-rejection` | Emits valid success-shaped evidence so a rejecting caller-owned `EventSink` can exercise the host boundary. |
| `signal-termination` | Raises a real Unix termination signal before producing artifacts. |
| `retryable-failure` / `terminal-failure` | Retains a local transcript with typed provider/validation failure and a private canary; no acceptance. |
| `effect-success` | Records bounded local launch/effect witnesses and writes the normal candidate. |
| `effect-failure` / `effect-fail-once` | Records the same witnesses and candidate, then exits `7` on every attempt or only the first effect respectively. |
| `cleanup-cancelled` | After candidate creation, removes one disposable fixture file, preserves unfinished evidence, signals readiness, and waits for cancellation. |
| `cleanup-failed` | Performs the same partial cleanup, then fails to remove a nonempty fixture directory; candidate and unfinished evidence remain. |

Effect/cleanup modes append only fixed witness records to
`outputs/effect-journal.txt`, bounded to 512 bytes by the fixture. They never perform
real external effects or mutate original inputs. Cleanup residual evidence is
private fixture data; only allowlisted counters/dispositions and candidate digests
enter the [lifecycle receipts](../../../docs/integrations/lifecycle-scenarios.md).

`unavailable` and `incompatible` are harness-only resolution cases. They are
not runtime modes because both reject selection before an invocation exists.
The unavailable case marks the exact observation unavailable. The incompatible
case applies a case-local external manifest requirement of Flow `>=9.0.0`.
Neither change mutates the hashed provider package.

## Deterministic composition fixtures

Checkpoint #58 owns two explicit, test-only two-stage fixtures:

| Fixture | Ordered stages | Providers | Exact handoff |
| --- | --- | --- | --- |
| `single-provider-composition` | `inspect` → `transform` | `org.egohygiene.synthetic-scenario-provider` for both stages | accepted `artifact:inspection-report` |
| `multi-provider-composition` | `inspect` → `transform` | `org.egohygiene.synthetic-inspector@0.1.0` → `org.egohygiene.synthetic-renderer@0.1.0` | accepted `artifact:inspection-report` |

The harness calls each stage explicitly in array order. It creates the second
binding only from the first stage's `AcceptedArtifactSet`, retaining the same
artifact ID, port, media type, kind, locator, and digest in one workspace.
Those exact fields are `artifact:inspection-report`, `port:inspection-report`,
`application/vnd.flow.fixture-inspection+json`, `file`,
`outputs/inspection-report.json`, and the digest learned from the first stage's
accepted output.
Every stage independently resolves its provider, matches package and executable
subjects, authorizes the invocation, runs the direct child, observes the bound
artifacts, and accepts the output. Two fresh roots must produce equal portable
resolution, invocation, subject, authority, execution, observation, and
accepted-artifact evidence plus byte-identical outputs.

The output read grant is required only so a later composition stage may consume
an earlier accepted output; it grants no mutation of that input. These fixtures
do not implement a ready queue, DAG scheduler, retry, resume, checkpoint,
durable state, scenario-manifest executor, or real holon algorithm.

## Host-harness artifact cases

Two checkpoint #57 cases deliberately keep `mode=success` because only the host
can create or attempt to reuse Flow's opaque observation token:

| Case | Harness operation | Exact rejection boundary |
| --- | --- | --- |
| `changed-output-after-observation` | Run the real provider successfully, observe the complete binding set, then overwrite the bound output with deterministic replacement bytes. | The final acceptance read returns `ArtifactAcceptanceError::ObservationChanged`; no accepted token is created. |
| `stale-invocation-and-binding-contexts` | Reuse valid execution or observation evidence first with a changed run identity and then with a changed binding set. | Artifact acceptance returns the exact invocation-context or retained-binding mismatch; no accepted token is created. |

The supporting artifact-library cases also change an input after observation,
remove an output before the final read, and validate deliberately malformed or
contradictory portable observation documents. They do not add provider modes:
portable JSON cannot recreate `ObservedArtifactSet`, and the retained absolute
canonical root is sensitive host context omitted from portable evidence and
from the token's manual `Debug` output.

For `await-interruption`, the provider writes and syncs this compact record to
a sibling temporary file, then atomically publishes it with one line feed at
the explicit control locator:

```json
{"state":"ready","pid":1234}
```

The PID is host-specific test control data. The record is not named by the
artifact binding set, is not observed or accepted as an artifact, and is
excluded from normalized portable evidence. Cancellation begins only after the
record is readable. On Unix the test also uses the PID to prove the runner
reaped that direct child before returning. This proves neither descendant
cleanup nor process-tree containment, and it does not turn an undeclared file
into an automatically discovered artifact.

The runner drains overflow streams but retains only the configured limit plus
one byte. That retained length proves overflow; it is not a count of every byte
the provider emitted. Invalid-event, invalid-result, and host-rejection cases
may retain decoded provider evidence for inspection. Event delivery is not
transactional, so an earlier valid event can reach the authoritative sink
before later evidence rejects execution.

The provider reads only the named binding, input, and prior-output workspace
trees. It reads no ambient environment, opens no network connection, starts
no subprocess, mutates no source, and performs no destructive, signing, or
publication action. `trusted-unconfined` remains an explicit test profile, not
a sandbox or containment claim.

Checkpoint #57 delivers corrupt, changed, stale, and contradictory artifact
evidence cases while leaving provider provenance v1 free-form and
non-authoritative. Checkpoint #58 delivers the deterministic composition
fixtures. Checkpoint #59 closes the exact package workflow, complete behavior
catalog, parent #29 requirement matrix, residual gaps, and #30 handoff in the
[integration guide](../../../docs/integrations/hermetic-provider-kit.md).

The fixture source is MIT-licensed, and every local materialization retains the
repository `LICENSE`. A compiled executable may incorporate a target-specific
subset of dependency versions resolved by `Cargo.lock`; downstream binary
redistributors must determine and retain any licenses and notices those
dependencies require. This fixture neither assembles nor audits a distributable
third-party notice bundle. Never redistribute an all-zero template as finalized
metadata, and never redistribute a materialized package without freshly
verifying its package and executable evidence.
