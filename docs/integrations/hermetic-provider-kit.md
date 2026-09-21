# Hermetic orchestration provider kit

## Purpose and checkpoint boundary

The hermetic provider kit is Flow-owned conformance infrastructure for issue
#29. Checkpoint #44 establishes its immutable package identity and deterministic
success path. It deliberately exercises the released process and artifact
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

Checkpoint #44 executes the inspection capability end to end. The other three
IDs and their declared media/configuration profiles are frozen now so later
failure and graph fixtures do not invent parallel identities. Their end-to-end
coverage and broader behavior matrix remain explicitly deferred.

## Deterministic success profile

The only checkpoint-1 behavior is `success`. Configuration uses
`flow.hermetic-provider-configuration/v1` with exactly two string fields:

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

## Conformance evidence

`tests/hermetic_provider_kit.rs` builds two fresh roots with identical package,
input, binding, invocation, configuration, and authority values. It requires:

- identical package and executable identities;
- identical resolution, subject, authority, event, result, host-observation,
  accepted-artifact, and output-byte evidence;
- unchanged provider-package observations after execution;
- unchanged input and binding bytes;
- an exact filesystem layout in which the sole new entry is the declared
  candidate output; and
- successful promotion through both `ValidatedExecution` and
  `AcceptedArtifactSet`.

The fixture source, manifest, lock, materialization rules, offline commands,
and redistribution terms live in
[`tests/fixtures/hermetic-provider`](../../tests/fixtures/hermetic-provider/README.md).

## Authority and non-claims

The manifest requests only named workspace input/binding reads and named
workspace output writes. It requests no environment, subprocess, network, AI,
GPU, source-mutation, destructive, signing, or publication authority. The
runner clears the inherited environment and passes no secret handles.

The local runner profile is `trusted-unconfined`. This checkpoint proves exact
contract correlation, byte identity, bounded transport, and direct-child use;
it does not claim an operating-system sandbox, filesystem containment,
descriptor-bound execution, publisher authentication, descendant cleanup, or
provider-native semantic validation.

Checkpoint #45 owns warning, partial, unavailable, incompatible, nonzero,
timeout, cancellation, overflow, and invalid protocol behaviors. Checkpoint
#46 owns artifact adversaries, single/multi-provider graph fixtures, and the
final requirement-to-test matrix. Durable run state, retry, checkpoint, and
resume remain outside all three.
