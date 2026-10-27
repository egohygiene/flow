# Operational signal registry

Companion to the proposed [profile 0.1.0](README.md). This is the single suite
registry; owning repositories link it and document only their mappings. No
runtime schema or generated registry framework is introduced. **S** below means
stable-in-profile (reserved for candidate v1, not yet an accepted public API);
**P** means provisional pending the two pilots. **E** exportable after explicit
opt-in; **L** local-only; **H** high-cardinality, forbidden in metric labels;
**X** prohibited. Unknown attributes are dropped, not implicitly exportable.

## Resource and attribute vocabulary

| Name | Type/value source | Stability/privacy | Metric use |
| --- | --- | --- | --- |
| `service.name` | Exact `flow`, `aniflow`, `optiflow`, `renderflow` | S/E | Resource grouping |
| `service.version` | Emitting product's actual version, at most 64 bytes | S/E | Resource grouping, never a per-event version label |
| `service.namespace` | Optional literal `egohygiene.flow-suite` | S/E | Resource grouping |
| `flow.command` | Closed command registry name, no user arguments | S/E | Allowed from finite product map |
| `flow.operation` | `inspect`, `plan`, `execute`, `validate`, `reuse`, `recover`, `publish` | S/E | Allowed |
| `flow.run.id` | Local native run ID when available; portable random per-invocation alias | S/L/H, alias E/H | Never |
| `flow.stage.kind` | Closed stage kind; no user stage name | S/E | Optional finite map |
| `flow.stage.id` | Local native ID; portable invocation-scoped alias | S/L/H, alias E/H | Never |
| `flow.provider.kind` | Reviewed provider class, or literal `custom`/`unknown` | S/E | Optional finite map |
| `flow.provider.id` | Exact registered provider ID/version reference | S/L/H | Never; unknown third-party names are not exportable |
| `flow.capability.kind` | Reviewed finite capability class; fallback `custom` | S/E | Optional finite map |
| `flow.capability.id` | Exact native capability ID | S/L/H | Never |
| `flow.artifact.role` | `source`, `intermediate`, `candidate`, `output`, `evidence` | S/E | Optional finite map |
| `flow.artifact.ref` | Existing local evidence reference, never payload or path | S/L/H | Never; omitted from portable projections |
| `flow.validation` | `not_run`, `passed`, `failed`, `unavailable` observation | S/E | Allowed on validation instrument only |
| `flow.outcome` | `succeeded`, `partial`, `failed`, `cancelled`, `refused`, `unavailable`, `skipped`, `unknown` | S/E | Allowed |
| `flow.reuse` | `hit`, `miss`, `ineligible`, `not_checked` | S/E | Cache instrument only |
| `flow.diagnostic.code` | Reviewed native code mapped to finite safe class for export | S/L, reviewed class E | No raw code as metric label |
| `flow.attempt` | Positive ordinal, bounded by native operation policy | S/E | Never |
| `flow.duration` | Finite nonnegative monotonic seconds; absent if unobserved | S/E | Measurement, not label |
| `flow.progress.completed` / `.total` | Nonnegative counts, completed <= known total; total absent when unknown | S/E | Not metric labels |
| `flow.progress.unit` | `frame`, `page`, `artifact`, `byte`, `stage` | S/E | Not a label on general metrics |
| `flow.telemetry.dropped` | Saturating nonnegative count | S/E | Measurement |
| `flow.sink` | `stderr`, `file`, `otlp` | S/E | Dropped-record instrument only |
| `flow.reason` | `queue_full`, `record_budget`, `sink_failed` | S/E | Dropped-record instrument only |
| `flow.measurement` | `process_rss`, `device_used` | S/E | Memory instrument only |
| `flow.overflow` | Literal `true` on the one reserved overflow series | S/E | Metric-only exception; no other point labels on that series |
| `trace_id` / `span_id` | Validated first-class correlation IDs, not resource labels | S/E/H | Never; exemplars disabled in initial pilots |
| `host.name`, username, device serial, IP, process argv/env, URL headers/query | Ambient/private identifiers or data | X | Never |
| Prompt, captions, transcript, source/media text, tool args/results, raw stderr/stdout | User/provider content | X | Never |
| Source/output path, filename, private digest, secret handle/value, arbitrary `Debug`/error/backtrace | Sensitive/re-identifying content | X in this profile | Never |

Local-only attributes are allowed only on an explicitly configured local file
sink and after source classification. Default stderr uses the exportable-safe
field set, because it is commonly captured by terminals or CI. Native result,
diagnostic and evidence APIs retain their own independent contracts. A native
ID that embeds a path is prohibited, not magically safe because its key says ID.
Unrecognized native outcomes map to `unknown` operationally without altering
the domain result. Do not map partial/refused/cancelled to succeeded.

## Events and spans

Fixed names are suite-owned, lower-case dot-separated literals. Event context
supplies product/operation; identifiers never appear in the name. Templates are
static, approved phrases. Human formatting may change; names, typed attributes
and meanings are the machine-facing part of this candidate.

| Event | Default severity | Meaning |
| --- | --- | --- |
| `flow.command.started` | INFO | Parsed/configured invocation entered dispatch |
| `flow.command.finished` | INFO; WARN for partial/refused/unavailable; ERROR for failed | Observed native terminal classification; cancellation is INFO |
| `flow.stage.started` / `flow.stage.finished` | DEBUG / outcome mapping above | Existing domain stage transition was observed |
| `flow.provider.started` / `flow.provider.finished` | DEBUG / outcome mapping above | Actual provider attempt start/end, not mere discovery |
| `flow.validation.finished` | INFO / WARN on failed/unavailable | Actual domain validator result; absent validation never passes |
| `flow.cache.decided` | DEBUG | Actual hit/miss/ineligible/not_checked decision |
| `flow.progress.updated` | INFO | Throttled projection of typed progress, not accepted state |
| `flow.telemetry.dropped` | WARN | Bounded sink loss; never changes domain outcome |

Span names: `flow.command`, `flow.operation`, `flow.stage`, `flow.provider`,
`flow.validation`, `flow.cache`. Use only spans for operations actually entered;
preflight refusals remain explicit observations. Error severity describes an
operational failure, not blame or proof. TRACE is bounded internal decisions,
DEBUG diagnostic detail, INFO ordinary lifecycle, WARN recoverable degradation
or refusal, ERROR failed operation. Native diagnostics retain their original
severity/exit mapping; the log projection does not replace it.

## Metrics and budgets

Metric names have no embedded ID/format/path/model name. No instrument is required
to run when telemetry is disabled. Instruments aggregate actual observations;
missing/unattempted work does not contribute a fabricated zero or success.

| Instrument | Kind / unit | Allowed point attributes |
| --- | --- | --- |
| `flow.operation.duration` | Histogram / `s` | command, operation, outcome |
| `flow.provider.duration` | Histogram / `s` | provider.kind, capability.kind, outcome |
| `flow.operation.count` | Counter / `{operation}` | command, operation, outcome |
| `flow.validation.count` | Counter / `{validation}` | validation, artifact.role |
| `flow.cache.decision.count` | Counter / `{decision}` | reuse, capability.kind |
| `flow.telemetry.dropped.count` | Counter / `{record}` | sink=`stderr`/`file`/`otlp`, reason=`queue_full`/`record_budget`/`sink_failed` |
| `flow.resource.memory` | Optional observed gauge / `By` | measurement=`process_rss`/`device_used` (never invent shared-memory addition) |

Point attribute shorthand expands to the `flow.*` keys above. Resources use only
the three service keys. Each instrument has at most 5 point keys and 128 active
attribute combinations per process. Approved value dictionaries are finite and
versioned with the owning pilot; `custom`/`unknown` collapse unregistered kinds.
If the cap is reached, fold measurements into one pre-reserved unlabeled
overflow series with `flow.overflow=true`; do not retain a growing dropped-key
set. Never use run/stage/trace/artifact IDs, paths, timestamps, error text,
model/provider versions or request IDs as labels.

Default duration bucket bounds in seconds: 0.005, 0.01, 0.025, 0.05, 0.1, 0.25,
0.5, 1, 2.5, 5, 10, 30, 60, 300, 1800, plus overflow. These are operational
guidance, not timeouts. A pilot may justify a different finite view before
adoption; it cannot change domain deadlines through a histogram option.

Per log/span: at most 24 allowlisted attributes, 64-byte keys, 128-byte strings,
256-byte fixed message, and 8 KiB total encoded JSONL record. No nested maps,
arrays, nonfinite floats, recursive errors or content blobs. IDs retain their
exact bounded syntax or are omitted with a dropped-field observation; never
truncate an identifier into a plausible different ID. Excess ordinary fields
are omitted deterministically in registry order. Drop an oversized record as a
whole, count it and avoid recursive diagnostics. Span events are capped at 32;
ongoing high-frequency progress uses the independently throttled event path.

## JSON Lines projection

Each line is one UTF-8 JSON object plus LF, without ANSI. Candidate schema tag:
`flow.operational-log/v1`; it is distinct from all domain command-result and
Flow process-protocol schemas. Reserved top-level keys are `schema_version`,
`timestamp`, optional `observed_timestamp`, `severity`, `event`, `message`,
`resource`, `attributes`, and optional validated `trace_id`/`span_id`. Unknown
top-level/attribute keys are excluded by producers. A formal runtime schema is
owned by the pilots once this profile is adopted; this example is not a receipt.

```json
{"schema_version":"flow.operational-log/v1","timestamp":"2026-10-26T20:00:00Z","severity":"info","event":"flow.command.finished","message":"Command finished","resource":{"service.name":"aniflow","service.version":"0.1.0"},"attributes":{"flow.command":"inspect","flow.operation":"inspect","flow.outcome":"succeeded","flow.duration":0.125}}
```

The version is illustrative, not a release assertion. Logs do not nest a result
or serialized provider capture. An error projection uses an approved diagnostic
code/class and fixed message. No caller-supplied `message` bypass exists.

## Optional inference example

A future inference span records capability class, selected versus reported
provider/model as classified local-only fields, attempt count, cache decision,
validation/refusal and observed latency. A planned budget is not observed usage.
Tool loops have explicit domain call/step budgets; telemetry cannot grant more.

`flow.ai.usage.source` is `provider_reported`, `measured`, `estimated`, or
`unknown` (P/E). `flow.ai.cost.source` is `provider_reported`, `estimated`, or
`unknown` (P/E); known cost requires a nonnegative amount and ISO currency,
estimates additionally require a reviewed pricing-revision reference (P/L/H).
Unknown amount/tokens are absent, never zero. Provider-reported counters are
claims, not independently measured truth. Model names are local-only by default;
only a reviewed public alias can be exported and none becomes a metric label.
Use the [pinned GenAI mapping](otel-mapping.md), keeping payload capture disabled.

The provisional inference allowlist adds `flow.ai.cost.amount` (finite
nonnegative number), `flow.ai.cost.currency` (three uppercase ASCII letters),
and `flow.ai.cost.pricing_revision` (bounded reviewed reference, L/H). None is a
metric label. For known provider-reported usage, `gen_ai.usage.input_tokens`
and `gen_ai.usage.output_tokens` are nonnegative integers. `gen_ai.operation.name`
is a reviewed finite operation name. `gen_ai.request.model`,
`gen_ai.response.model` and `gen_ai.provider.name` are L/H unless mapped to an
explicitly reviewed public alias; none is a metric label. The standard
inference metrics in the mapping are provisional external projections using
this profile's resource/series caps, not permission to export other upstream
attributes. Unknown usage/cost omits corresponding amounts. No other GenAI
attribute or payload surface is admitted by implication.
