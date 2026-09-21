# Hermetic provider fixture

This directory is the source bundle for Flow's redistribution-safe synthetic
process provider. It is conformance infrastructure, not a production holon
adapter or a public Flow CLI.

## Stable identity

- Extension: `org.egohygiene.synthetic-scenario-provider@0.1.0`
- Publisher declaration: `org.egohygiene`
- Process mode: `hermetic-process`
- Entrypoint: `flow-hermetic-provider`

The manifest freezes four Flow-owned synthetic capabilities:

| Capability | Synthetic role | Candidate artifact type |
| --- | --- | --- |
| `flow/inspect-fixture` | Inspection | `application/vnd.flow.fixture-inspection+json` |
| `flow/transform-fixture` | Transformation | `application/vnd.flow.fixture-transformation+json` |
| `flow/validate-fixture` | Validation | `application/vnd.flow.fixture-validation+json` |
| `flow/observe-fixture` | Read-only source observation | `application/vnd.flow.fixture-observation+json` |

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

The provider reads no ambient environment, opens no network connection, starts
no subprocess, mutates no source, and performs no destructive, signing, or
publication action. `trusted-unconfined` remains an explicit test profile, not
a sandbox or containment claim.

Artifact missing/extra/corrupt/stale/contradictory cases, host physical-
artifact observation and acceptance failures after complete protocol success,
graph fixtures, and the final parent requirement matrix remain outside this
checkpoint and are owned by issue #46.

The source and generated package are distributed under the repository's MIT
license. Do not redistribute a materialized package without its license or
without replacing and verifying the integrity placeholders.
