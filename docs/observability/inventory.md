# Pinned observability and stream inventory

Observed October 26, 2026 for [Flow #15](https://github.com/egohygiene/flow/issues/15).
This is a source inventory for the [proposed profile](README.md), not a runtime
audit, adoption claim, or replacement for the providers' existing contracts.
Reads covered named source paths, manifests and relevant contract documentation.
No product tests, compiler/build, lint, schema, native or hosted checks ran.

| Repository | Inspected default-branch revision | Evidence boundary |
| --- | --- | --- |
| flow | `69952725fb4eeb1daeb0e5c8bea7d8d82ba40670` | Kernel and pinned optiflow adapter; no supported product CLI |
| renderflow | `9dac5428df38691b2922e09ea02d51b02a7f43eb` | Current source after merged #454 and audit #460; #455 remains draft/unmerged |
| aniflow | `7a808ff99ca8f557f6a3ba7fd373a393de0feee6` | Landed provider/temporal source; #64 qualification deferral retained |
| optiflow | `d03e53f2e397af03920070d9bb6c076d7827e12b` | Current main, not unmerged macOS draft #122 or the v0.1.1 release |

The revisions above are the observation pins. Later issue/PR status does not
retroactively qualify these snapshots. The profile proposes future behavior;
the differences below remain adoption work under #16 and subsequent owners.

## flow

- [Cargo manifest][flow-cargo] declares the library and a synthetic
  `flow-hermetic-provider` binary, not the supported product `flow` CLI.
  No logging/metrics/OTLP bootstrap or dependency is present in the inspected
  manifest/source. There is no product verbosity, color, sink or export surface
  to describe as implemented.
- [Execution][flow-execution] exposes caller-owned `EventSink`. Its fallible
  delivery is part of execution acceptance: observer rejection rejects evidence.
  It is not a best-effort log sink. A future exporter must use a separate
  observation path and cannot turn an exporter failure into acceptance failure.
- [Process transport][flow-process] expects newline-delimited provider events
  followed by exactly one terminal result. This is a provider protocol, not
  the proposed product CLI's single terminal JSON document or a telemetry stream.
- The [native optiflow adapter][flow-optiflow] separately consumes the pinned
  v0.1.1 command envelope as one JSON document. It validates version, command,
  outcome, coverage, diagnostics and artifact bindings; protocol families are
  not interchangeable merely because both use JSON.
- [Authority][flow-authority] has `TelemetryPropagation` fields/targets and a
  closed vocabulary for baggage, correlation ID, traceparent and tracestate.
  These are requested/granted authority constraints, not an implemented carrier,
  exporter or blanket approval to propagate context to a child process.
- Raw stdout/stderr are caller-owned sensitive transcripts. Structured errors
  may retain rejected provider evidence; unrestricted explanation/provenance
  strings and a provider's `redacted: true` assertion are not content sanitizers.
  Do not export whole events, errors, results or process captures.
- Resolution/acceptance and durable state remain authoritative. Observation
  cannot select providers, grant effects, mutate evidence or establish a reusable
  checkpoint. Preserve current authority identities; do not add telemetry-only
  context to hashed invocation, lock or checkpoint material.

## renderflow

- [CLI bootstrap][renderflow-app] calls `tracing_subscriber::fmt().init()` with
  stderr and a maximum level: `--debug` selects TRACE, otherwise `--verbose`
  selects DEBUG, otherwise INFO. This installs a global subscriber; it is not
  fallible/idempotent composition with a previously installed subscriber.
  This initializer has no `EnvFilter`, JSONL, file or OTLP selection surface.
- Parsing occurs before logging initialization. The [SDK][renderflow-sdk]
  exposes caller-supplied progress reporting and cancellation without calling
  that CLI logging initializer. A shared profile must preserve this library
  ownership boundary rather than initialize logging from SDK methods.
- Output is command-specific. [Build][renderflow-build] prints a plan JSON
  document for dry-run and uses tracing for planned/written output messages.
  [Composition commands][renderflow-composition] emit their own versioned
  plan/run documents. Other commands retain text/raw or selected structured
  formats. No single suite-wide terminal-result envelope is implemented here.
- Human logs/errors and progress are local diagnostic surfaces, including output
  paths and domain diagnostic text. `renderflow.progress/v1` and SDK progress
  callbacks are domain observations, not a portable logging schema. Neither
  tracing formatting nor bounded diagnostic counts establish privacy redaction.
- The [build composition integration][renderflow-build-composition] binds the
  effective specification, native plan and artifact forest to canonical evidence.
  Run IDs and timing belong to run evidence. Existing evidence digests are not
  promised time-free; future telemetry IDs/config must stay outside semantic
  plans, provider locks and cache identity without rewriting those receipts.

## aniflow

- The [CLI][aniflow-cli] has global `--output human|json`, defaulting to human,
  plus legacy unwrapped `inspect --json`. No logging subscriber/exporter,
  filter, telemetry environment, color/sink configuration or custom panic hook
  was found in the inspected source and Cargo manifest.
- Success serializes [MachineEnvelope v1][aniflow-contract] to stdout.
  **Machine failure envelopes currently go to stderr** in `print_error`.
  `Cli::parse()` also handles invocation errors/help/version before this
  dispatcher. The proposed single-result stdout rule is a migration requirement,
  not current behavior or permission to silently break existing consumers.
- Human run/segment/Pipeline v3 progress currently uses stdout. Machine
  presentation suppresses it. Human errors escape terminal controls, while
  progress/outcome fields include local paths and pipeline/stage names.
  Terminal escaping is distinct from privacy filtering.
- [Pipeline v3 callbacks][aniflow-run] explicitly describe non-authoritative
  progress after durable transitions. Running, validating, complete, reused,
  invalidated, failed and cancelled are distinct. Started records identify a
  workspace; they are not evidence that subsequent state initialization succeeded.
- [Provider events/reports][aniflow-provider] retain contiguous sequence,
  timestamps, exact provider-lock identity, termination, validation, output
  observations and bounded captures. Exit success alone does not accept output.
  Existing error categories and exit codes remain aniflow's own contract.
- Capture redaction replaces caller-supplied sensitive values; v3 sorts longest
  first. It is not a classifier for arbitrary prompts, source text or paths.
  [Toolchain probe captures][aniflow-probes] retain original bytes as hex plus
  digest. Hex is reversible and must not be treated as anonymized telemetry.
- General provider launch inherits environment; dedicated probe/audio process
  paths clear it and set narrow defaults. No explicit trace carrier is
  implemented. Do not infer universal child isolation or authorized propagation.
- `stage_invocation_sha256` excludes runtime paths, timestamps and attempt
  evidence from its semantic identity. [Checkpoint/report digests][aniflow-state]
  separately bind historical time, captured diagnostics and execution evidence.
  Preserve that distinction: excluding new telemetry-only state does not remove
  legitimate evidence time or make historical receipt digests reproducible.

## optiflow

- [Main][optiflow-main] installs signals, parses, runs the library, validates
  the command result and renders it. No logging subscriber/exporter, OTLP,
  filter or custom panic hook was found in inspected source/Cargo manifest.
- [Rendering][optiflow-render] buffers a complete JSON document plus LF to
  stdout for success and failure. Human primary output uses stdout and escaped
  diagnostics use stderr. [Typed results][optiflow-outcome] retain outcome,
  actual exit code, coverage, artifacts and diagnostics; they are not run artifacts.
- Early JSON invocation-error selection examines explicit CLI flags, not file
  or environment configuration. Help/version retain Clap presentation, and a
  configuration-resolution failure falls back to explicit CLI output selection.
  These exceptions must stay visible in the adoption/conformance matrix.
- [Configuration][optiflow-config] resolves defaults, user, nearest project,
  recognized environment, then explicit CLI. It exposes effective values and
  provenance, not arbitrary ambient environment. Current v1 does not define
  verbosity, progress, color, logging sinks or export settings.
- The scan CLI has no live progress stream. [Extension events][optiflow-events]
  provide callback/result progress and lifecycle records with sequence, counts,
  phase/state and invocation identity. They do not establish a product JSONL
  stream or replace accepted evidence and separately authorized transactions.
- Native paths, inventory, configuration inspection and local artifacts preserve
  useful private detail, including reversible non-UTF-8 path encodings.
  [Subprocess errors][optiflow-subprocess] can retain bounded raw stderr/program
  text. Escaping control characters is not a secret/content scrubber.
- Generic subprocess requests inherit environment by default; the external
  extension adapter explicitly clears it. There is no explicit trace carrier.
  Embedded extension `catch_unwind` does not itself suppress a default panic
  hook or make arbitrary panic messages safe for telemetry.
- [Policy identities][optiflow-policy] deliberately differ: the full effective
  configuration digest includes presentation and state directory, while the
  evidence-policy fingerprint excludes them and provenance. New telemetry
  state must stay outside evidence/cache/plan semantics; do not claim existing
  full configuration identity already ignores every presentation setting.

## Adoption implications

Use closed outcome/code/resource fields for portable observations. Classify
provider/stage identifiers before export; keep paths, argv/environment values,
source/prompts, captures, raw errors and whole result/manifest objects local.
Content digests and reversible encodings are not privacy anonymization.
No current source observation proves cross-platform behavior, redaction
completeness, exporter isolation or profile conformance. #16 owns the first
two executable pilots; later owners retain migration and qualification.

[flow-cargo]: https://github.com/egohygiene/flow/blob/69952725fb4eeb1daeb0e5c8bea7d8d82ba40670/Cargo.toml
[flow-execution]: https://github.com/egohygiene/flow/blob/69952725fb4eeb1daeb0e5c8bea7d8d82ba40670/src/execution.rs
[flow-process]: https://github.com/egohygiene/flow/blob/69952725fb4eeb1daeb0e5c8bea7d8d82ba40670/src/process.rs
[flow-optiflow]: https://github.com/egohygiene/flow/blob/69952725fb4eeb1daeb0e5c8bea7d8d82ba40670/src/optiflow.rs
[flow-authority]: https://github.com/egohygiene/flow/blob/69952725fb4eeb1daeb0e5c8bea7d8d82ba40670/src/authority.rs
[renderflow-app]: https://github.com/egohygiene/renderflow/blob/9dac5428df38691b2922e09ea02d51b02a7f43eb/crates/renderflow-core/src/app.rs
[renderflow-sdk]: https://github.com/egohygiene/renderflow/blob/9dac5428df38691b2922e09ea02d51b02a7f43eb/crates/renderflow-core/src/sdk.rs
[renderflow-build]: https://github.com/egohygiene/renderflow/blob/9dac5428df38691b2922e09ea02d51b02a7f43eb/crates/renderflow-core/src/commands/build.rs
[renderflow-composition]: https://github.com/egohygiene/renderflow/blob/9dac5428df38691b2922e09ea02d51b02a7f43eb/crates/renderflow-core/src/commands/composition.rs
[renderflow-build-composition]: https://github.com/egohygiene/renderflow/blob/9dac5428df38691b2922e09ea02d51b02a7f43eb/crates/renderflow-core/src/planning/build_composition.rs
[aniflow-cli]: https://github.com/egohygiene/aniflow/blob/7a808ff99ca8f557f6a3ba7fd373a393de0feee6/src/cli.rs
[aniflow-contract]: https://github.com/egohygiene/aniflow/blob/7a808ff99ca8f557f6a3ba7fd373a393de0feee6/src/contract.rs
[aniflow-run]: https://github.com/egohygiene/aniflow/blob/7a808ff99ca8f557f6a3ba7fd373a393de0feee6/src/run_v3.rs
[aniflow-provider]: https://github.com/egohygiene/aniflow/blob/7a808ff99ca8f557f6a3ba7fd373a393de0feee6/src/provider_runtime.rs
[aniflow-probes]: https://github.com/egohygiene/aniflow/blob/7a808ff99ca8f557f6a3ba7fd373a393de0feee6/src/toolchain/probe_types.rs
[aniflow-state]: https://github.com/egohygiene/aniflow/blob/7a808ff99ca8f557f6a3ba7fd373a393de0feee6/src/state_v3.rs
[optiflow-main]: https://github.com/egohygiene/optiflow/blob/d03e53f2e397af03920070d9bb6c076d7827e12b/src/main.rs
[optiflow-render]: https://github.com/egohygiene/optiflow/blob/d03e53f2e397af03920070d9bb6c076d7827e12b/src/render.rs
[optiflow-outcome]: https://github.com/egohygiene/optiflow/blob/d03e53f2e397af03920070d9bb6c076d7827e12b/src/outcome.rs
[optiflow-config]: https://github.com/egohygiene/optiflow/blob/d03e53f2e397af03920070d9bb6c076d7827e12b/src/configuration/resolver.rs
[optiflow-events]: https://github.com/egohygiene/optiflow/blob/d03e53f2e397af03920070d9bb6c076d7827e12b/src/extensions/model.rs
[optiflow-subprocess]: https://github.com/egohygiene/optiflow/blob/d03e53f2e397af03920070d9bb6c076d7827e12b/src/subprocess.rs
[optiflow-policy]: https://github.com/egohygiene/optiflow/blob/d03e53f2e397af03920070d9bb6c076d7827e12b/src/configuration/policy.rs
