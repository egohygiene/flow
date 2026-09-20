# Flow

> One coherent way to inspect, plan, transform, validate, and explain content.

Flow is the suite and future orchestration layer for Ego Hygiene's independently
useful Rust tools:

| Holon | Owns | Does not own |
| --- | --- | --- |
| [Aniflow](docs/architecture/holons/aniflow/README.md) | Time-based video decomposition, ordered processing, reconstruction, and temporal validation | Collection optimization or general document rendering |
| [Optiflow](docs/architecture/holons/optiflow/README.md) | Local inventory, evidence-backed relationships, safe optimization plans, and collection normalization | Video-frame processing or transform-graph execution |
| [Renderflow](docs/architecture/holons/renderflow/README.md) | Spec-driven transform graphs and publication-ready document, image, and audio derivatives | Collection policy or temporal video reconstruction |
| Flow | Cross-holon planning, compatibility, execution state, provenance, validation, recovery, and suite experience | Reimplementing a holon's specialized engine |

Each holon remains usable as both a Rust library and a standalone CLI. Flow may
compose them through stable public library interfaces or versioned CLI
contracts, without introducing direct dependencies between sibling holons.

The repository remains architecture-led, but it is no longer documentation
only. FLO-Q02 provides a small Rust library that validates the federated
extension contracts, resolves one capability deterministically, executes one
caller-injected in-process extension, and validates a provider-neutral process
transcript without launching a child process. It also validates a closed,
deterministic scenario manifest for synthetic orchestration fixtures. It does
not copy holon source or claim a product orchestrator, CLI, production process
adapter, scenario runner, durable run state, or resume support.

## Executable checkpoint

The first executable checkpoint is deliberately library-only:

- closed Rust models and semantic validation cover the six extension-v1
  documents;
- `ExtensionCatalog` inspection and resolution retain deterministic selection
  and rejection evidence;
- `Orchestrator` invokes a matching `ExtensionPort` and returns an execution
  only after Flow validates event and terminal-result correlation;
- the process seam deterministically encodes one JSON Lines invocation and
  checks bounded, caller-supplied stdout/stderr plus process completion
  evidence, then routes decoded stdout through the same event/result gate;
- artifact bindings map immutable input and candidate-output IDs to portable
  root-relative locators; Flow observes file/directory bytes beneath one root
  and returns an accepted set only after exact host/provider correlation;
- `flow.scenario-manifest/v1` pins synthetic inputs, providers, topology,
  expectations, resource budgets, coverage gaps, and cross-language canonical
  digests without defining plans or runs; and
- a no-effects, no-artifacts hermetic port and example prove the seam without a
  provider binary, filesystem output, network access, or external service.

Run the reference example with:

```console
cargo run --example hermetic_extension --locked
cargo run --example hermetic_process_transport --locked
cargo run --example scenario_manifest --locked
```

`EventSink` is a fallible, authoritative execution observer, not a best-effort
telemetry exporter. Observations do not influence provider selection or
execution identity, but `emit` rejection makes `Orchestrator` return
`ExecutionError` and never triggers fallback. A sink cannot directly mutate
provider evidence or grant authority. The separate observability roadmap owns
any future non-authoritative logging or OpenTelemetry seam; no such backend or
exporter ships here.

`ValidatedExecution` means the provider evidence passed Flow's contract,
identity-correlation, ordering, diagnostic redaction-flag, and
terminal-consistency checks. It does not authenticate the caller-issued
configuration digest, authorization ID, or grants digest; the caller still owns
configuration canonicalization and authorization issuance. Diagnostics marked
`redacted: false` and other invalid provider evidence are retained only on
`ExecutionError` and never reach the caller's `EventSink` or a
`ValidatedExecution`. A `redacted: true` value remains a provider assertion;
this checkpoint does not content-scan diagnostics or sanitize unrestricted
contract strings.

`ValidatedExecution` does not imply artifact acceptance. `observe_artifacts`
returns an opaque Flow-observation token after root, path, link, node, digest,
and directory-manifest checks. `accept_artifacts` then requires a complete
produced/reused result and exact agreement among the resolution, invocation,
bindings, host observations, result IDs, and artifact-produced events before it
returns `AcceptedArtifactSet`.

Only `trusted` candidates are resolution-eligible in this checkpoint;
`Orchestrator` either executes a caller-injected in-process port or validates a
caller-supplied process transcript. `sandboxed` candidates fail closed because
no sandbox backend exists. In-process limits remain correlated metadata only.
The process-transcript seam checks already captured stdout/stderr byte counts
and completion evidence, but it does not launch, time out, cancel, signal, reap,
or isolate a process. Neither seam provides filesystem or network containment.

## Architecture

- [Architecture map](docs/architecture/README.md)
- [Suite system model](docs/architecture/foundation/SYSTEM.md)
- [Suite structural architecture](docs/architecture/foundation/ARCHITECTURE.md)
- [Decision index](docs/architecture/governance/DECISIONS.md)
- [Suite boundaries](docs/integrations/suite-boundaries.md)
- [Federated extension contract](docs/integrations/extension-contract.md)
- [Process transport contract](docs/integrations/process-transport.md)
- [Artifact binding contract](docs/integrations/artifact-bindings.md)
- [Scenario fixture contract](docs/integrations/scenario-fixtures.md)
- [Versioned contracts](contracts/README.md)
- [Roadmap](ROADMAP.md)

The `.agents/` directory contains the repository-local Aether specifications,
skills, agents, templates, and validators used to maintain these documents.

## Status

Flow is in the **executable contract seam** phase. Issues #23, #26, and #28 are
merged through PRs #24, #27, and #35, and default-branch CI passed at
`aac62ab80c18c936e3a7ebd5f4e70b7845f39faa`. Issue #36 implements the next
bounded parent-#25 slice: artifact bindings and host observation. The process
seam still does not implement a runner, executable verification, authority
enforcement, or host isolation, and the scenario contract is not an executor.
Current descriptions of Aniflow, Optiflow, and Renderflow are grounded in their default
branches as inspected on 2026-08-13. The holons remain independently released
repositories; real provider adapters and the restore-and-assess workflow remain
follow-up work.

## License

MIT
