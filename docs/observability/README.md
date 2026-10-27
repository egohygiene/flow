# Suite observability profile

Profile: `flow.observability-profile`, revision **0.1.0, proposed**.
Owner: [Flow #15](https://github.com/egohygiene/flow/issues/15), under #14.
Observation date: October 26, 2026. This is a contract candidate for the two
pilots in #16, not a claim that existing CLIs implement it. `MUST` and `SHOULD`
below describe conformance to this proposed profile. Adoption does not require
a shared crate, Flow dependency, collector, account, or new domain schema.

Start with the [pinned inventory](inventory.md). The [registry](registry.md)
owns names and privacy/cardinality limits; the [OpenTelemetry mapping](otel-mapping.md)
owns external mappings; the [conformance plan](conformance.md) owns future
proof. [ADR-0014](../architecture/governance/decisions/ADR-0014-operational-observability.md)
is proposed, not accepted merely because this draft is published or merged.

## One purpose per signal

| Signal | Purpose | Authority and owner |
| --- | --- | --- |
| Operational log | Bounded explanation of a code path or operational problem | Optional observation; never proof of an accepted result |
| Metric | Aggregate resource, rate, failure and latency measurement | Optional measurement; missing samples mean unknown |
| Trace/span | Causal and latency relationships between operations | Optional correlation; no authorization or provenance authority |
| Progress event | Current user-visible lifecycle and completed/total work | Existing typed domain event owner; may be coalesced only in presentation |
| Result/diagnostic | Machine-readable outcome, typed refusal, partial result and next action | Existing versioned product/provider contract; no human-log parsing |
| Durable manifest/evidence | Recorded identity, observations, validation and accepted state | Existing domain gates and stores; never reconstructed from telemetry |

A successful process exit, span, progress bar, log message or exporter flush
MUST NOT create a completed stage, validated artifact or reusable checkpoint.
Cancellation, refusal, unavailable, partial, failed and succeeded remain
distinct. A provider-reported result remains a claim until its normal domain
acceptance gate checks it. Telemetry may observe these transitions afterward.

## Runtime and library boundaries

- Library entry points MUST NOT install a global subscriber, logger, exporter,
  panic hook or environment-derived telemetry configuration. They may emit
  bounded events through the existing facade and caller-owned sinks.
- CLI bootstrap MUST be fallible and idempotent for its own invocation. A
  subscriber already installed by an embedding caller is preserved; no panic,
  replacement or second global initialization is allowed.
- No-op telemetry MUST remain a supported path. Disabled instrumentation may
  not change provider selection, validation, outputs, result status or exit code.
- No content-changing telemetry hook is allowed. Domains own cancellation and
  progress; exporters may neither cancel nor resume domain work.
- New telemetry configuration/state MUST stay outside deterministic plans,
  cache keys, provider locks, compatibility fingerprints and artifact identity.
  Do not insert correlation IDs into hashed domain envelopes to make joining
  easier. Existing evidence timestamps and evidence digests are not removed.
  The [inventory](inventory.md) records those existing distinctions.

## Correlation, time and sampling

Use the registry's bounded product, command, operation, stage, provider,
capability, role, validation and outcome vocabulary. Run/trace/span correlation
is optional and high-cardinality: it is forbidden as a metric label. Domain
identifiers may be used locally only after classification; portable telemetry
uses invocation-scoped random aliases, not file names or hashes of private data.
Keep alias-to-evidence mappings local and separately opt-in; no new alias store
is required by this contract.

Wall timestamps are UTC RFC 3339 in JSONL. Duration comes from a monotonic clock
and uses nonnegative seconds. Do not subtract wall clocks across processes or
interpret timestamp order as causal proof. Record observation time separately
only if different/available. Host sleep and clock discontinuities are not a
reason to rewrite recorded domain evidence.

Emit one root command span, optional operation/stage children, and provider,
validation and cache children when those operations occur. Fixed span names
never contain paths, IDs, prompts or errors. Concurrent children use their
actual parent; retries have separate attempts and do not overwrite earlier
outcomes. A trace may be incomplete: filtering, sampling and cancellation are
not evidence that omitted operations never happened.

Local operational logs obey a severity filter, without sampling away typed
results or durable evidence. Optional trace export uses parent-based sampling
with a default root ratio of 0.1; an explicit ratio in [0,1] overrides it.
No exporter exists in this checkpoint. These defaults are pilot proposals;
sampling never changes domain execution. No automatic capture-on-error of
source content or previously prohibited fields is permitted.

## Proposed CLI and configuration surface

These names are **reserved pilot proposals**, not commands users can run today.
Keep product result switches (for example existing `--json`) separate from
operational `--log-format`. Each product maps its existing configuration loader
to a delivery-only telemetry object; no config-file discovery is added here.

| Setting | Proposed CLI | Product environment suffix | Default |
| --- | --- | --- | --- |
| Severity | `--log-level off\|error\|warn\|info\|debug\|trace` | `LOG_LEVEL` | `warn` |
| Reviewed target filter | `--log-filter FILTER` | `LOG_FILTER` | No target override |
| Operational encoding | `--log-format human\|jsonl` | `LOG_FORMAT` | Human |
| Color | `--color auto\|always\|never` | `COLOR` | Auto |
| Operational sink | `--log-sink stderr\|file\|off` | `LOG_SINK` | stderr |
| Local file | `--log-file PATH` | `LOG_FILE` | Absent |
| Progress | `--progress auto\|always\|never` | `PROGRESS` | Auto |
| Telemetry export | `--telemetry-export off\|otlp` | `TELEMETRY_EXPORT` | Off |

Environment variables are product-prefixed, e.g. `RENDERFLOW_LOG_LEVEL` and
`ANIFLOW_LOG_LEVEL`; no suite-global environment variable changes every tool.
Precedence per setting: explicit CLI > explicit product-prefixed environment >
explicit product config > built-in default. CLI aliases `--verbose`/`--debug`
map to debug/trace only when no explicit CLI level is supplied; conflicting
aliases or malformed values are typed configuration errors before domain work.
Severity filters accept only known target prefixes plus levels, not arbitrary
field-value/regex capture. They cannot relax redaction or budgets.

An `OTEL_*` variable, `RUST_LOG`, collector address, file path or inherited trace
context alone MUST NOT enable telemetry or override this precedence. Future
exporter settings are read only after explicit export selection. Endpoint,
headers and credentials are delivery-local secrets; they never appear in
effective domain configs or diagnostic echoes. Existing environment behavior
is inventoried and requires explicit migration, not silent reinterpretation.

`NO_COLOR` disables auto color; explicit CLI `--color always` may override it
only for human TTY output. Non-TTY output and machine modes always suppress ANSI,
cursor controls and spinners. Auto progress is human-TTY stderr only. Explicit
`always` outside a TTY uses bounded plain lines in human encoding, or registered
JSONL progress records when `--log-format jsonl` is selected. JSONL always wins
over human line formatting. `--progress` independently controls presentation;
log severity filtering does not suppress explicitly requested progress. A file
or off log sink leaves independently enabled progress on stderr in the selected
encoding. `--log-sink off` does not disable typed error results. `quiet` behavior
is composed from `off` and `never`, not a second precedence system.

## stdout/stderr ownership

| Interface/mode | stdout | stderr |
| --- | --- | --- |
| Human product command | Requested human output or explicit artifact stream | Human diagnostics and bounded progress/logs |
| Machine product command | Exactly one native versioned command-result document, including typed failure/partial/cancelled when the command reaches its result boundary | Selected operational encoding only; no second result envelope |
| Explicit raw artifact export | Declared raw artifact bytes; mutually exclusive with result JSON on stdout | Operational detail only; receipts go to an explicit file |
| `--help` / `--version` | Parser-owned human help/version; explicit discovery commands retain their own versioned contracts | Empty unless a real write failure occurs |
| Argument/configuration failure | Versioned failure result only when the product's minimal parser safely recognized machine-result mode; otherwise empty | Bounded sanitized usage error in human mode; operational JSONL if safely selected |
| Panic/unrecoverable abort | Never fabricate a successful or complete result; may be empty/incomplete | Sanitized generic fatal notice when safe; no payload/argv/backtrace by default |
| Flow extension process protocol | Existing invocation/event/result JSONL protocol per ADR-0006 | Existing bounded provider stderr; not an operational profile escape hatch |

The exactly-one-result guarantee applies to handled product outcomes, not help,
raw artifact mode, an OS kill, abort or broken output pipe. Serialize one complete
result before beginning its write where bounded by the existing result contract;
on write failure do not append another envelope. The adapter must reject a
missing/incomplete result. Minimal machine-mode detection must respect `--`,
recognized global switches, repeated/conflicting switches and non-UTF-8 argv;
it must not echo unknown argv or source paths. Each pilot publishes its parser
limitations and tests them before claiming profile conformance.

Logs MUST NOT be redirected onto product stdout by a sink choice. Machine stdout
keeps its existing schema and native ownership; this profile defines no replacement
command-result schema. Existing diagnostic/result prose may contain local paths
under its own native contract: do not copy it wholesale into portable logs.

## Redaction and handling threat model

| Threat surface | Required control | Residual boundary |
| --- | --- | --- |
| Debug/Display of requests, errors or argv | Emit allowlisted codes/counts; skip input structs and arbitrary error strings at instrumentation sites | Existing raw diagnostic/evidence APIs need their own access controls |
| Provider stdout/stderr, prompt, transcript, caption or media content | Never log wholesale; classify a bounded typed outcome instead | Raw local evidence stays sensitive under its domain retention policy |
| Credentials, headers, environment values, URLs with query/userinfo | Prohibited at source; sink filter drops unknown fields and unsafe templates | String scrubbing alone is not sufficient assurance |
| File names, paths, artifact hashes, model names, user stage IDs | Local-only or alias/classify per registry; never metric labels | Hashing private strings does not anonymize them |
| Log injection/control sequences | Fixed event names/templates; JSON encoding; strip terminal controls in human projections | Do not concatenate provider text into log templates |
| Inherited child environment/trace context | Explicit field-and-target allowlist and existing process authority | A cleared environment alone is not sandboxing |
| Full collector or stalled sink | Bounded queue, drop telemetry, preserve result/evidence progress | A telemetry file is not durable run evidence |

Apply allowlisting at instrumentation sites, then again before human, JSONL,
file or export serialization. Unknown attributes are dropped with a bounded
counter, not echoed in an error. Debug/trace never relax these rules. A deliberate
local diagnostic report is a separately requested product feature, not permission
to export its contents. No source text, prompts, captions, secrets, raw argv,
environment dumps, arbitrary provider output or backtrace enters this profile.

## Sinks, retention, export and failure

Default operation performs **no telemetry network access**, creates no telemetry
file and does no collector discovery. Human stderr remains useful offline.
File sinks are opt-in, must not overwrite an existing file or follow a final
symlink, and must reside outside source/accepted-output/cache trees. They retain
only profile-allowed fields, with owner-only permissions where supported. If
private file creation is unsupported, refuse that sink before domain execution.
No automatic deletion or rotation of user files: cap the current invocation's
log file at 16 MiB, then stop writing, report one safe sink warning and preserve
domain work. The user owns cleanup; future retention policy requires explicit
configuration. Export retention/access control belongs to the chosen collector
and must be disclosed before opt-in; this profile promises no remote deletion.

Every operational sink/export worker has a bounded nonblocking queue: at most
256 records and 1 MiB serialized bytes, whichever limit is reached first.
Drop incoming records on overflow, increment a saturating local dropped count,
and emit at most one safe warning per sink failure class per invocation when
a functioning stderr sink exists. Do not recursively log an exporter error.
Typed results and authoritative evidence do not use this lossy queue.
JSONL record and field budgets are in the registry. Coalesce live progress to
at most four updates/second/stage; terminal typed outcomes are never discarded
from their own protocol by this presentation limit.

Invalid explicit telemetry configuration is refused before work; an unavailable
collector or sink failure after work begins is nonfatal to domain outcome.
Optional OTLP uses a bounded asynchronous worker, no persistent retry spool,
no implicit endpoint and no automatic fallback. Network requests have a 1 s
deadline, no background work after shutdown, and no unbounded retry loop.
Retain the shutdown guard through result/evidence emission; allow at most 2 s
for final drain/flush, also on cancellation, then abandon telemetry safely.
Telemetry failure never changes an already determined domain result or accepted
evidence. Actual sink/worker limits and overhead must be measured by the pilots.

## Change and adoption

Names marked stable-in-profile are reserved for the proposed v1 vocabulary;
their current document is still a candidate. Provisional OTel/AI fields may be
changed only with a mapping revision, not by silently following upstream main.
An implementation claims conformance only for its exact command/mode/revision
matrix after the [conformance plan](conformance.md) runs. #16 pilots precede
shared-package selection and exporter work. No promise of complete suite
instrumentation, hosted backend, release readiness or new creative capability
follows from this contract checkpoint.
