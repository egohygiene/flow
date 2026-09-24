# Scenario manifest and fixture profile

## Purpose and scope

`flow.scenario-manifest/v1` is the Flow-owned description of one bounded,
repeatable orchestration conformance scenario. It records topology, immutable
inputs and providers, expected terminal and evidence states, execution tier and
resource budgets, and an honest coverage claim. The manifest is test intent; it
is not a runtime plan, run record, checkpoint, transition log, or execution
engine.

The checked-in scenarios are synthetic and redistribution-safe. They let Flow
stabilize cross-provider semantics before real Aniflow, Optiflow, or Renderflow
adapters exist. Passing them proves only the behaviors listed in each
`coverage.covered_behaviors` array. `coverage.known_gaps` prevents a fixture from
being presented as end-to-end provider or runner evidence.

## Stable identity

Every manifest carries two related identities:

| Field | Meaning |
| --- | --- |
| `scenario_id` | Stable logical scenario name in the `scenario:<id>` namespace |
| `fixture_id` | Stable fixture lineage in the `fixture:<id>` namespace |
| `fixture_version` | Strict `major.minor.patch` revision of the fixture recipe and expectations |
| Canonical digest | Exact identity of the validated manifest after deterministic normalization |

Renaming a scenario or fixture is an identity change. Changing inputs,
providers, topology, expectations, budgets, or coverage requires a fixture
version review and necessarily changes the canonical digest. A locator alone
never establishes identity.

## Immutable inputs and providers

Every input and provider package has an `ImmutableSource`:

- `kind` is `generated`, `vendored`, or `released`;
- `locator` is a retrieval hint, not authority or identity;
- `revision` is either `git:<40-lowercase-hex>` or
  `sha256:<64-lowercase-hex>`;
- `digest` is the SHA-256 identity of the exact resolved bytes; and
- `license` records the redistribution basis.

Generated sources additionally record generator ID, strict semantic version,
seed, and a digest of generator parameters. The small checked-in input bytes are
reproducible from `tools/generate_scenario_sources.py`; CI invokes only its
read-only `--check` mode. Vendored and released sources cannot claim local
generator provenance. Mutable branch names, tags without an immutable revision,
path-only identities, and floating package constraints are invalid.

A provider reference pins its extension manifest by schema and digest, its
package source, interface kind, version, and required capabilities. It does not
embed or reinterpret the provider-owned manifest.

## Topology

`stages` is an ordered, topologically valid graph. Each stage names one declared
provider and one capability required from that provider. Dependencies may refer
only to earlier stages. A stage that consumes an artifact produced by another
stage must depend directly or transitively on that producer.

Stage order is contract-significant: it is the deterministic tie-break between
otherwise independent ready stages. Input, provider, tag, dependency, artifact,
reference, service, and coverage arrays are set-like and are normalized for
identity; the stage array is not reordered.

The manifest describes intended dataflow. It does not authorize arbitrary
commands, grant provider permissions, establish a plan ID, or prove that any
provider ran.

## Typed outcomes and evidence

Terminal state and evidence state are separate, constrained dimensions:

| Terminal state | Permitted evidence state | Meaning |
| --- | --- | --- |
| `complete` | `complete`, `observed-empty` | The scenario completed; empty means an authoritative observation found no items |
| `partial`, `interrupted` | `incomplete` | Some required evidence is absent because work did not fully complete |
| `unavailable` | `unavailable` | A required provider or capability could not be used |
| `unsupported` | `unsupported` | The requested behavior is outside declared support |
| `invalid` | `invalid` | Inputs or contract evidence are invalid |
| `failed` | `failed` | Execution attempted and failed |

`observed-empty` is not `unavailable`, `incomplete`, or `unsupported`, and it
cannot claim expected artifacts. Contradictory terminal/evidence pairs are
invalid. Expected artifacts must be declared inputs or stage outputs.

`evidence_refs` and `state_trace_refs` point to other Flow documents by their
versioned schema identifier and digest. State-trace references identify the
expected golden transition trace; they do not embed or define its future
schema. Future plan, run, checkpoint, evidence, and transition contracts remain
independently versioned, so the scenario manifest must reference them rather
than duplicate their fields.

## Execution tiers and budgets

Each scenario declares one tier: `pull-request`, `scheduled`, or `release`.
Every tier records positive limits for wall-clock time, memory, stdout, stderr,
artifact bytes, and artifact count. These are conformance budgets for a future
runner, not proof that the current library enforces operating-system resources.

Pull-request scenarios must be clean-room, deny network access, and declare no
external service. Scheduled and release scenarios may use an explicit
allowlist, but `allowlisted` requires named services and `denied` forbids them.
Clean-room always means denied network and no external service.

## Canonicalization and drift detection

`flow.canonical-json/v1` defines scenario identity for this contract:

1. Deserialize the closed `flow.scenario-manifest/v1` model and validate every
   semantic invariant.
2. Sort object keys lexicographically.
3. Sort set-like arrays: tags; inputs by artifact ID; providers by provider ID;
   provider capabilities; each stage's dependencies, inputs, and outputs;
   expected artifacts and diagnostics; evidence and trace references by schema
   then digest; external services; covered behaviors; and known gaps.
4. Preserve `stages` array order.
5. Serialize compact UTF-8 JSON with no insignificant whitespace.
6. Compute the lowercase SHA-256 digest of those bytes.

The independent Rust and Python implementations must match the digests in
`contracts/fixtures/scenarios/canonical-digests.v1.json`. CI validates that
catalog without rewriting it. Any intentional fixture change therefore
requires a reviewed manifest, version decision, and explicit digest update;
drift cannot silently bless itself.

## Executed composition conformance

Flow issue #58 aligns the declarative multi-provider fixture with the frozen
`flow/inspect-fixture` and `flow/transform-fixture` capability names and keeps
its explicit `inspect`-then-`transform` order and artifact dependency. The
manifest still is not a runtime plan. In particular, scenario artifact names
such as `inspection-report` and runtime binding IDs such as
`artifact:inspection-report` belong to different closed namespaces.

Execution evidence lives in the hermetic provider-kit tests. A fixed test-owned
sequencer materializes these two cases:

| Runtime fixture | Ordered providers | Accepted handoff |
| --- | --- | --- |
| `single-provider-composition` | scenario provider → same provider | `artifact:inspection-report` |
| `multi-provider-composition` | synthetic inspector → synthetic renderer | `artifact:inspection-report` |

Every stage independently crosses public resolution, subject observation,
authority, local-process execution, artifact observation, and acceptance. The
downstream binding is derived only from the upstream `AcceptedArtifactSet`, and
two fresh runs must retain equal normalized evidence and byte-identical
outputs. This focused evidence does not add a general scenario executor,
production scheduler, durable plan or run, retry, checkpoint, resume, or real
provider-algorithm claim.

Flow issue #59 indexes that executable evidence against every parent #29
criterion and records the exact handoff to #30. The later checkpoint still owns
machine-readable execution of the broader compatibility, artifact, diagnostic,
privacy, and budget matrix; this document remains the declarative fixture
contract rather than an executor.

## Checked-in corpus

The corpus includes a single-provider success example plus multi-provider,
interrupted, observed-empty, and expected-unavailable fixtures. The unavailable
fixture is a valid negative scenario; it is distinct from malformed manifest
tests. Adversarial fixtures cover an unknown schema version, missing identity,
contradictory expectations, mutable source revision, and invalid zero budget.

Validate the schema, semantics, adversarial corpus, and canonical digests with:

```console
python3 tools/validate_contracts.py
python3 tools/generate_scenario_sources.py --check
cargo test --test scenario --locked
cargo run --example scenario_manifest --locked
```

These checks do not run real provider releases, launch a process, enforce a
sandbox, materialize artifacts, persist a plan or run, resume a checkpoint, or
claim provider compatibility.
