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
- [Versioned contracts](../../contracts/README.md) define the provisional suite
  interchange vocabulary.

These documents are governed by
[ADR-0004](../architecture/governance/decisions/ADR-0004-federated-suite-contracts.md),
[ADR-0005](../architecture/governance/decisions/ADR-0005-federated-extension-authority.md),
and
[ADR-0006](../architecture/governance/decisions/ADR-0006-bounded-process-transport.md).

## Implementation status

Merged PR #24 implements the in-process library seam described by the extension
contract. It validates closed extension-v1 models, resolves one capability from
explicit caller input, invokes a caller-supplied `ExtensionPort`, and validates
correlated events and the terminal result. Issue #26 adds deterministic process
request encoding and validation of caller-supplied completion, stdout, and
stderr evidence through the same Flow-owned acceptance gate. Both reference
paths are hermetic and produce no effects or artifacts.

Only lock entries with `trusted` trust are resolution-eligible.
`Orchestrator` either invokes a caller-injected in-process port or validates a
caller-supplied process transcript; it does not launch a process. `sandboxed`
entries fail closed because this checkpoint has no enforceable sandbox backend.
A `ValidatedExecution` proves closed-contract and correlation checks, not the
authenticity of a caller-issued configuration digest, authorization ID, or
grants digest. The caller owns configuration canonicalization and authorization
issuance.

Declared limits remain identity-correlated metadata for injected in-process
code. The process-transcript seam checks captured stdout/stderr lengths and a
completion observation, but no Flow runner yet enforces those limits while a
process executes. Neither path provides timeout delivery, cancellation, panic
isolation, filesystem/network containment, or other side-effect enforcement.

The following remain deferred: provider discovery from the filesystem, dynamic
loading, a production child-process runner, sandboxing, real
Aniflow/Optiflow/Renderflow adapters, artifact locator and domain-output
validation, a public CLI, durable plans and run state, interruption,
checkpoints, and resume. `EventSink` is a fallible execution observer, not a
logging or OpenTelemetry adapter. It cannot affect resolution or execution
identity, but its rejection intentionally rejects the execution without
fallback.
