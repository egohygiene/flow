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
only. The current FLO-Q02 candidate adds a small Rust library that validates the
federated extension contracts, resolves one capability deterministically, and
executes one caller-injected in-process extension through a seam proven by a
hermetic reference port. It does not copy holon source or claim a product
orchestrator, CLI, external process adapter, durable run state, or resume
support.

## Executable checkpoint

The first executable checkpoint is deliberately library-only:

- closed Rust models and semantic validation cover the six extension-v1
  documents;
- `ExtensionCatalog` inspection and resolution retain deterministic selection
  and rejection evidence;
- `Orchestrator` invokes a matching `ExtensionPort` and returns an execution
  only after Flow validates event and terminal-result correlation; and
- a no-effects, no-artifacts hermetic port and example prove the seam without a
  provider binary, filesystem output, network access, or external service.

Run the reference example with:

```console
cargo run --example hermetic_extension --locked
```

`EventSink` is the observation boundary for this checkpoint. A future logging
or OpenTelemetry adapter can attach there, but observations do not influence
provider selection or execution identity. `emit` is fallible and its error is
visible to the provider; rejection makes `Orchestrator` return
`ExecutionError`. A sink cannot directly mutate provider evidence or grant
authority. No logging backend or telemetry exporter ships in this slice.

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

Only `trusted` candidates are resolution-eligible in this checkpoint;
`Orchestrator` executes only a caller-injected in-process port. `sandboxed`
candidates fail closed because no sandbox backend exists. Declared execution
limits are validated and correlated as metadata, not enforced. The injected
code has no Flow-owned timeout, cancellation, stdout/stderr bound, panic
isolation, filesystem or network containment, or other side-effect enforcement.

## Architecture

- [Architecture map](docs/architecture/README.md)
- [Suite system model](docs/architecture/foundation/SYSTEM.md)
- [Suite structural architecture](docs/architecture/foundation/ARCHITECTURE.md)
- [Decision index](docs/architecture/governance/DECISIONS.md)
- [Suite boundaries](docs/integrations/suite-boundaries.md)
- [Federated extension contract](docs/integrations/extension-contract.md)
- [Versioned contracts](contracts/README.md)
- [Roadmap](ROADMAP.md)

The `.agents/` directory contains the repository-local Aether specifications,
skills, agents, templates, and validators used to maintain these documents.

## Status

Flow is in the **executable contract seam** phase. Issue #23 supplies the
candidate library implementation and CI definition for the remaining FLO-Q02
evidence; FLO-Q02 stays active until that change is merged and exercised by
default-branch CI. Current descriptions of Aniflow, Optiflow, and Renderflow are
grounded in their default branches as inspected on 2026-08-13. The holons
remain independently released repositories; real provider adapters and the
restore-and-assess workflow remain follow-up work.

## License

MIT
