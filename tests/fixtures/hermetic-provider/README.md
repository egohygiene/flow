# Hermetic provider fixture

This directory is the source bundle for Flow's redistribution-safe synthetic
process provider. It is conformance infrastructure, not a production holon
adapter or a public Flow CLI.

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
templates containing an all-zero integrity placeholder. A host must build the
executable, place it with the MIT license in an otherwise empty package
directory, observe that directory and the executable through Flow, and replace
both placeholders in finalized metadata copies with the observed package
SHA-256 before catalog inspection. The finalized manifest, operator lock, and
checksum record remain beside the package in the host's configured catalog;
they are not copied into the hashed package root, which would create a
self-referential package digest. The conformance test performs these steps in
memory and then creates the exact execution-subject lock.

Composition packages for the inspector and renderer add only the documented
`PROVIDER-IDENTITY` marker beside the same executable and license. The marker
bytes are fixed and included in Flow's package observation.

Build from an already populated Cargo cache without network access:

```console
cargo build --bin flow-hermetic-provider --locked --offline
```

Exercise the supported package path through Flow rather than invoking the
fixture as a standalone user command:

```console
cargo test --test hermetic_provider_kit --locked --offline
```

At runtime the host passes only literal long-form arguments:

- `--artifact-root <host path>` selects the explicit workspace root.
- `--artifact-bindings <portable locator>` selects one
  `flow.artifact-bindings/v1` document beneath that root.
- `--lifecycle-control <portable locator>` is optional and is accepted only by
  the `await-interruption` lifecycle mode. The conformance harness uses
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
| `multi-provider-composition` | `inspect` → `transform` | `org.egohygiene.synthetic-inspector` → `org.egohygiene.synthetic-renderer` | accepted `artifact:inspection-report` |

The harness calls each stage explicitly in array order. It creates the second
binding only from the first stage's `AcceptedArtifactSet`, retaining the same
artifact ID, port, media type, kind, locator, and digest in one workspace.
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
fixtures. Final redistribution documentation plus the parent requirement matrix
remain with #59.

The source and generated package are distributed under the repository's MIT
license. Do not redistribute a materialized package without its license or
without replacing and verifying the integrity placeholders.
