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
only. The executable checkpoints provide a small Rust library that validates
the federated extension contracts, resolves one capability deterministically,
executes one caller-injected in-process extension, and validates a provider-
neutral process transcript. Flow can also launch one exact authorized
`trusted-unconfined` local provider with bounded stdio, deadline/cancellation
control, and direct-child reaping; observe exact locked package/executable
subjects; require exact correlated process authority/isolation evidence; accept
explicitly bound artifacts; and validate a closed scenario manifest for
synthetic orchestration fixtures. It does not copy holon source or claim a
product orchestrator, CLI, real provider adapter, operating-system sandbox,
general scenario executor, automatic recovery scheduler, or public resume CLI.
Prepared process execution now has [durable run state](docs/integrations/durable-state.md)
with immutable plans, atomic snapshots, verified checkpoints, status inspection,
explicit recovery decisions, and fresh dependency-graph assessment before
caller-selected downstream execution. Stale prerequisites invalidate their
descendants for reuse while unrelated valid branches retain their eligibility.

## Executable checkpoint

The executable contract surface remains library-led. The only repository
binary is a synthetic conformance provider; it is not a public Flow CLI:

- closed Rust models and semantic validation cover the six extension-v1
  documents;
- `ExtensionCatalog` inspection and resolution retain deterministic selection
  and rejection evidence;
- `Orchestrator` invokes a matching `ExtensionPort` and returns an execution
  only after Flow validates event and terminal-result correlation;
- the process seam deterministically encodes one JSON Lines invocation and
  requires a fresh opaque match for the exact locked package and executable,
  checks bounded caller-supplied stdout/stderr plus process completion evidence,
  then routes decoded stdout through the same event/result gate;
- `flow.execution-subject-lock/v1` and
  `flow.execution-subject-observations/v1` keep content equality, lock match,
  publisher declaration, operator trust, cryptographic verification, and
  transparency-log status as separate claims;
- `flow.process-authority-profile/v1` and
  `flow.process-enforcement-evidence/v1` bind exact requested/granted authority,
  trust, isolation, and caller-attested enforcement to the matched subjects;
- `LocalProcessRunner` freshly re-observes and directly launches the exact
  `trusted-unconfined` executable, clears and reconstructs its environment,
  manages bounded stdio workers, enforces timeout/cancellation grace and
  escalation, reaps the direct child, and reuses the transcript validator;
- artifact bindings map immutable input and candidate-output IDs to portable
  root-relative locators; Flow observes file/directory bytes beneath one root
  and returns an accepted set only after exact host/provider correlation;
- `RunStore` persists prepared intent and authority before launch, records accepted
  checkpoints after validation, and reopens with typed stale/corrupt-state refusal;
- `flow.scenario-manifest/v1` pins synthetic inputs, providers, topology,
  expectations, resource budgets, coverage gaps, and cross-language canonical
  digests without defining plans or runs; and
- a no-effects, no-artifacts hermetic port and example prove the seam without a
  provider binary, filesystem output, network access, or external service;
- a separately compiled hermetic provider fixture freezes four synthetic
  capability IDs and proves a real deterministic process-to-artifact success
  path plus bounded resolution, lifecycle, and protocol outcomes without
  sibling source, ambient environment, network access, or an external service.

Run the reference example with:

```console
cargo run --example hermetic_extension --locked
cargo run --example hermetic_process_transport --locked
cargo run --example scenario_manifest --locked
cargo test --test hermetic_provider_kit --locked
cargo test --test durable_state --locked
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

Process-mode `ValidatedExecution` additionally requires
`MatchedExecutionSubjects`, an opaque token created only after Flow observes
the locked package directory and executable file and both SHA-256 identities
match exactly. This proves content equality to the supplied operator lock for
that invocation. It does not prove a publisher signature, authentic publisher
identity, transparency-log inclusion, trustworthy operator policy, or that a
later launcher opened the same unchanged file object.

Process mode also requires `AuthorizedProcess`, an opaque token constructed only
after an exact authority profile and host-enforcement statement agree with the
resolution, invocation, operator grants, trust, and freshly matched subjects.
The profile makes argv, secret handles, resource allowlists, denied ambient
authority, and isolation explicit. For `sandboxed`, every dimension must be
reported enforced exactly as requested. This evidence is caller-attested; it
does not prove that a sandbox ran.

Only `trusted` candidates may use the unconfined in-process seam. A `sandboxed`
process candidate may resolve only to the downstream authority/isolation gate.
`Orchestrator` either executes a caller-injected in-process port or validates a
caller-supplied process transcript. In-process limits remain correlated metadata
only. The pure process-transcript seam checks caller-supplied stdout/stderr byte
counts and completion evidence. `LocalProcessRunner` supplies the real bounded
direct-child lifecycle for `trusted-unconfined` only. It does not provide
filesystem/network containment, descendant cleanup, descriptor-bound launch,
or a sandbox.

## Architecture

- [Architecture map](docs/architecture/README.md)
- [Suite system model](docs/architecture/foundation/SYSTEM.md)
- [Suite structural architecture](docs/architecture/foundation/ARCHITECTURE.md)
- [Decision index](docs/architecture/governance/DECISIONS.md)
- [Suite boundaries](docs/integrations/suite-boundaries.md)
- [Federated extension contract](docs/integrations/extension-contract.md)
- [Process transport contract](docs/integrations/process-transport.md)
- [Bounded local process runner](docs/integrations/process-runner.md)
- [Execution-subject integrity](docs/integrations/execution-subjects.md)
- [Process authority and isolation](docs/integrations/authority-isolation.md)
- [Artifact binding contract](docs/integrations/artifact-bindings.md)
- [Scenario fixture contract](docs/integrations/scenario-fixtures.md)
- [Executable acceptance matrix](docs/integrations/acceptance-scenarios.md)
- [Hermetic provider kit](docs/integrations/hermetic-provider-kit.md)
- [Versioned contracts](contracts/README.md)
- [Roadmap](ROADMAP.md)

The `.agents/` directory contains the repository-local Aether specifications,
skills, agents, templates, and validators used to maintain these documents.

## Status

Flow is in the **executable contract seam** phase. Issues #23, #26, #28, #36,
#38, #40, #42, #44, and #45 are merged through PRs #24, #27, #35, #37, #39,
#41, #43, #47, and #48. PR #60 consolidates #46's four sequential checkpoints:
physical and evidence adversaries, deterministic single- and two-provider
compositions, exact package/executable content-identity correlation, a closed
behavior catalog, and the parent #29 evidence matrix. After that PR merges and
default-branch CI is green, #46 and #29 can close and #30 becomes the next
conformance-matrix checkpoint.
The synthetic providers are not the two real adapters required to finish
FLO-Q03. The process seam still does not implement authenticity verification,
operating-system sandbox enforcement, authenticated host evidence,
descriptor-bound launch, or process-tree containment, and the scenario contract
is not an executor.
Current descriptions of Aniflow, Optiflow, and Renderflow are grounded in their default
branches as inspected on 2026-08-13. The holons remain independently released
repositories; real provider adapters and the restore-and-assess workflow remain
follow-up work.

## License

MIT
