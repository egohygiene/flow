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

The provider accepts one LF-terminated `flow.extension-invocation/v1` document
on standard input. It validates the invocation, binding set, configuration
digest, and actual input digest; creates exactly one new declared output; then
emits three events and one terminal result as compact JSON Lines. Artifact
bytes depend only on the capability, configuration, and immutable input
identity. They contain no time, hostname, process ID, temporary path, random
value, environment value, or external-service result.

The provider reads no ambient environment, opens no network connection, starts
no subprocess, mutates no source, and performs no destructive, signing, or
publication action. `trusted-unconfined` remains an explicit test profile, not
a sandbox or containment claim.

The source and generated package are distributed under the repository's MIT
license. Do not redistribute a materialized package without its license or
without replacing and verifying the integrity placeholders.
