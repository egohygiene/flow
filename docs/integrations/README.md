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
- [Versioned contracts](../../contracts/README.md) define the provisional suite
  interchange vocabulary.

These documents are governed by
[ADR-0004](../architecture/governance/decisions/ADR-0004-federated-suite-contracts.md).

## Implementation status

The FLO-Q02 candidate implements only the in-process library seam described by
the extension contract. It validates closed extension-v1 models, resolves one
capability from explicit caller input, invokes a caller-supplied
`ExtensionPort`, and validates correlated events and the terminal result. The
included reference port is hermetic and produces no effects or artifacts.

Only lock entries with `trusted` trust are resolution-eligible, and
`Orchestrator` executes only a caller-injected in-process port. `sandboxed`
entries fail closed because this checkpoint has no enforceable sandbox backend.
A `ValidatedExecution` proves closed-contract and correlation checks, not the
authenticity of a caller-issued configuration digest, authorization ID, or
grants digest. The caller owns configuration canonicalization and authorization
issuance.

Declared limits are validated and identity-correlated metadata only: the
injected in-process code has no Flow-owned timeout, cancellation, output-bound,
panic-isolation, filesystem/network, or other side-effect enforcement.

The following remain deferred: provider discovery from the filesystem, dynamic
loading, process transport and sandboxing, real Aniflow/Optiflow/Renderflow
adapters, artifact locator and domain-output validation, a public CLI, durable
plans and run state, cancellation, checkpoints, and resume. `EventSink` is an
observation attachment point for future logging and OpenTelemetry adapters; it
is not a shipped telemetry backend and cannot affect resolution or execution
identity. Its fallible `emit` can reject the checkpoint execution.
