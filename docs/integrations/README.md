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
  request framing and host-neutral transcript acceptance.
- [Bounded local process runner](process-runner.md) defines exact direct launch,
  supervised transport, timeout/cancellation grace and escalation, and
  direct-child reaping for `trusted-unconfined` providers.
- [Execution subjects](execution-subjects.md) define exact locked package and
  executable identity, fresh Flow-owned observation, and separated digest,
  authenticity, publisher, trust, and transparency claims.
- [Process authority and isolation](authority-isolation.md) define exact
  per-invocation authority, trust/isolation selection, caller-attested host
  enforcement evidence, and the opaque process-authorization gate.
- [Artifact bindings](artifact-bindings.md) define portable root-relative
  locators, deterministic file/directory observations, and the separate
  artifact-acceptance gate.
- [Scenario fixtures](scenario-fixtures.md) define stable scenario identity,
  immutable topology, typed expectations, resource tiers, canonicalization,
  and honest coverage boundaries.
- [Hermetic provider kit](hermetic-provider-kit.md) defines the immutable
  synthetic package, stable capability IDs, deterministic success profile,
  bounded lifecycle/protocol outcome matrix, and process-to-artifact
  acceptance proof used by orchestration conformance.
- [Versioned contracts](../../contracts/README.md) define the provisional suite
  interchange vocabulary.

These documents are governed by
[ADR-0004](../architecture/governance/decisions/ADR-0004-federated-suite-contracts.md),
[ADR-0005](../architecture/governance/decisions/ADR-0005-federated-extension-authority.md),
[ADR-0006](../architecture/governance/decisions/ADR-0006-bounded-process-transport.md),
[ADR-0007](../architecture/governance/decisions/ADR-0007-root-confined-artifact-acceptance.md),
and
[ADR-0008](../architecture/governance/decisions/ADR-0008-locked-execution-subjects.md),
and
[ADR-0009](../architecture/governance/decisions/ADR-0009-process-authority-isolation.md).
The bounded runner is governed by
[ADR-0010](../architecture/governance/decisions/ADR-0010-bounded-direct-process-supervision.md).

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

Operator-`trusted` entries remain eligible for either execution mode.
Operator-`sandboxed` entries may resolve only for process mode and remain
subject to the downstream authority/isolation preflight; they cannot use the
unconfined in-process seam. `Orchestrator` either invokes a caller-injected
in-process port or validates a caller-supplied process transcript; it does not
launch a process. A `ValidatedExecution` proves closed-contract and correlation
checks, not the authenticity of a caller-issued configuration digest,
authorization ID, or grants digest. The caller owns configuration
canonicalization and authorization issuance.

Flow #36 adds Flow-owned artifact bindings, root-confined file and directory
observation, immutable input digest checks, and an opaque
`AcceptedArtifactSet`. `ValidatedExecution` remains a weaker provider-evidence
token and never implies that filesystem artifacts were accepted.

Flow #38 adds closed execution-subject lock and observation contracts. A fresh
opaque `MatchedExecutionSubjects` token is required before process request
encoding and transcript validation. The token proves exact package and
executable digest equality for the correlated invocation; it does not prove a
signature, publisher authenticity, transparency-log inclusion, or trustworthy
operator policy.

Flow #40 adds closed process-authority profile and enforcement-evidence
contracts. Both process seams additionally require an opaque
`AuthorizedProcess` token bound to the exact resolution, invocation, matched
subjects, request, operator grant, trust, isolation, and evidence identities.
For `sandboxed`, the evidence must report exact enforcement of every requested
dimension. That statement is caller-attested and is not proof that an operating-
system sandbox actually ran.

Flow #42 adds the bounded local runner. It freshly re-observes the locked
subjects, directly launches the exact executable with literal argv, clears the
environment, resolves only authorized opaque handles, supervises all three
stdio streams, enforces timeout and caller cancellation with Unix grace and
escalation, reaps the direct child, and sends normal evidence through the
existing transcript gate. The in-process path provides no isolation. The local
runner accepts only `trusted-unconfined` and does not provide filesystem,
network, subprocess, or descendant containment.

Flow #44 adds the first real hermetic provider-kit checkpoint. A separately
compiled synthetic provider declares four stable Flow-owned capabilities and
drives the inspection success path through catalog resolution, exact subject
matching, authority preflight, `LocalProcessRunner`, host artifact observation,
and `AcceptedArtifactSet`. Two fresh roots prove identical semantic and content
evidence while package, input, and binding bytes remain unchanged. Flow #45
adds prelaunch rejection, warning/partial evidence, bounded process lifecycle,
stream overflow, invalid protocol evidence, and authoritative host-rejection
cases without weakening the immutable-package or success baseline. Physical
artifact adversaries, graph fixtures, and the final parent matrix remain with
#46.

The following remain deferred: provider discovery from the filesystem, dynamic
loading, cryptographic authenticity and transparency verification,
authenticated operating-system sandboxing, descriptor-bound launch,
process-tree containment, real Aniflow/Optiflow/Renderflow adapters,
domain-output validation, a public CLI, durable plans and run state,
checkpoints, and resume. `EventSink` is a fallible execution observer, not a
logging or OpenTelemetry adapter. It cannot affect resolution or execution
identity, but its rejection intentionally rejects the execution without
fallback.
