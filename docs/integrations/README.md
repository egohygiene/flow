# Flow integration contracts

Flow integrates independently released holons without taking ownership of their
implementations.

- [Suite boundaries](suite-boundaries.md) defines ownership and legal dependency
  directions.
- [Capability matrix](capability-matrix.md) records the evidence baseline and
  gates future adapter claims.
- [First slice](first-slice-restore-and-assess.md) bounds the initial executable
  orchestration outcome.
- [Federated extension contract](extension-contract.md) defines extension
  identity, trust, discovery, execution envelopes, hooks, and resolution.
- [Process transport](process-transport.md) defines deterministic JSON Lines
  request framing and host-neutral transcript acceptance without claiming a
  production process runner.
- [Artifact bindings](artifact-bindings.md) define portable root-relative
  locators, deterministic file/directory observations, and the separate
  artifact-acceptance gate.
- [Scenario fixtures](scenario-fixtures.md) define stable scenario identity,
  immutable topology, typed expectations, resource tiers, canonicalization,
  and honest coverage boundaries.
- [Versioned contracts](../../contracts/README.md) define the provisional suite
  interchange vocabulary.

These documents are governed by
[ADR-0004](../architecture/governance/decisions/ADR-0004-federated-suite-contracts.md),
[ADR-0005](../architecture/governance/decisions/ADR-0005-federated-extension-authority.md),
and
[ADR-0006](../architecture/governance/decisions/ADR-0006-bounded-process-transport.md),
and
[ADR-0007](../architecture/governance/decisions/ADR-0007-root-confined-artifact-acceptance.md).

## Implementation status

Merged PR #24 implements the in-process library seam described by the extension
contract. It validates closed extension-v1 models, resolves one capability from
explicit caller input, invokes a caller-supplied `ExtensionPort`, and validates
correlated events and the terminal result. Merged PR #27 adds deterministic
process request encoding and validation of caller-supplied completion, stdout,
and stderr evidence through the same Flow-owned acceptance gate. Flow #28 /
merged PR #35 adds a closed scenario-manifest model and a synthetic conformance
corpus; it describes test intent and does not execute those scenarios. The
executable reference paths remain hermetic; artifact observation is covered by
isolated temporary-fixture tests rather than a real provider.

Only lock entries with `trusted` trust are resolution-eligible.
`Orchestrator` either invokes a caller-injected in-process port or validates a
caller-supplied process transcript; it does not launch a process. `sandboxed`
entries fail closed because this checkpoint has no enforceable sandbox backend.
A `ValidatedExecution` proves closed-contract and correlation checks, not the
authenticity of a caller-issued configuration digest, authorization ID, or
grants digest. The caller owns configuration canonicalization and authorization
issuance.

Flow #36 adds Flow-owned artifact bindings, root-confined file and directory
observation, immutable input digest checks, and an opaque
`AcceptedArtifactSet`. `ValidatedExecution` remains a weaker provider-evidence
token and never implies that filesystem artifacts were accepted.

Declared limits remain identity-correlated metadata for injected in-process
code. The process-transcript seam checks captured stdout/stderr lengths and a
completion observation, but no Flow runner yet enforces those limits while a
process executes. Neither path provides timeout delivery, cancellation, panic
isolation, filesystem/network containment, or other side-effect enforcement.

The following remain deferred: provider discovery from the filesystem, dynamic
loading, a production child-process runner, sandboxing, real
Aniflow/Optiflow/Renderflow adapters, domain-output validation, a public CLI,
durable plans and run state, interruption,
checkpoints, and resume. `EventSink` is a fallible execution observer, not a
logging or OpenTelemetry adapter. It cannot affect resolution or execution
identity, but its rejection intentionally rejects the execution without
fallback.
