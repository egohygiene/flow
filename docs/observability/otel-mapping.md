# OpenTelemetry mapping and propagation proposal

This mapping belongs to proposed profile **0.1.0** in the [profile](README.md).
It is a source-reviewed design for later pilots, not installed instrumentation,
an exporter, an enabled network service or evidence of runtime conformance.
The [registry](registry.md) owns suite fields and privacy/cardinality budgets.
Native command results, process transcripts and durable evidence keep their
existing schemas; OpenTelemetry is an optional operational projection.

## Immutable upstream references

Observed October 26, 2026:

| Reference | Exact revision and scope |
| --- | --- |
| [Semantic conventions v1.44.0](https://github.com/open-telemetry/semantic-conventions/tree/e10a930844c6951757a43b849d364f7d056ac32b) | `e10a930844c6951757a43b849d364f7d056ac32b`; stable service attributes and general naming/units |
| [Specification v1.62.0](https://github.com/open-telemetry/opentelemetry-specification/tree/18c5c1706e346fec3bdb5e135a6e3a78f461e586) | `18c5c1706e346fec3bdb5e135a6e3a78f461e586`; log data model, trace API and propagation requirements |
| [GenAI Development snapshot](https://github.com/open-telemetry/semantic-conventions-genai/tree/e19717d65f890955777b044bca2a3cd71ae8b498) | `e19717d65f890955777b044bca2a3cd71ae8b498`; provisional AI appendix only; no latest release was available |

GenAI [moved to its own repository](https://github.com/open-telemetry/semantic-conventions/blob/e10a930844c6951757a43b849d364f7d056ac32b/docs/gen-ai/README.md).
Do not follow either upstream main automatically. Updating these pins requires
a reviewed mapping revision, privacy review and the affected conformance cases.
These document pins do not select Rust crates or authorize dependency changes.

## Adopted, extended and avoided surfaces

| Surface | Disposition | Mapping or boundary |
| --- | --- | --- |
| Resource identity | Adopt | `service.name` is the emitting product; `service.version` is its actual version. Optional `service.namespace` groups the suite. |
| Instrumentation scope | Adopt | Scope name/version identify the instrumented library/module, separately from the product resource. |
| Semantic schema URL | Bounded adoption | Use `https://opentelemetry.io/schemas/1.44.0` only for that semantic mapping; record the separate GenAI source pin rather than claiming its Development fields belong to the stable schema. |
| Log severity | Adopt | TRACE 1, DEBUG 5, INFO 9, WARN 13, ERROR 17; FATAL 21 only for an actual fatal condition. No severity establishes domain success or failure. |
| Log occurrence/observation | Adopt | Map JSONL wall time to `Timestamp`, and separately available observation time to `ObservedTimestamp`, using epoch nanoseconds in OTel. |
| Trace correlation | Adopt | Native TraceId/SpanId fields; valid nonzero identifiers, never metric labels or artifact identity. |
| Duration and size units | Adopt | Monotonic elapsed durations in `s`, byte sizes in `By`, ratios in `1`, discrete counts in annotated units such as `{operation}`. |
| Run/stage/capability/validation | Extend | Use the suite registry namespace and vocabulary; no invented `otel.*` fields or changes to native evidence. |
| Resource instance identity | Deferred | Initial resources use only the registry's three service keys. Use existing run/trace aliases for correlation; no service.instance.id or host/user identity. |
| HTTP/RPC conventions | Avoid here | The CLI is not an HTTP server. A later actual network adapter may adopt a separately reviewed mapping. |
| Automatic host/process/resource detection | Avoid | No argv, executable paths, user IDs, environment dumps, endpoint URLs or private host attributes by default. |
| Content and raw exceptions | Avoid | No prompts, captions, source text, provider streams, arbitrary error strings or backtraces. Use bounded typed codes. |
| Baggage and vendor tracestate | Avoid | Neither crosses a child boundary under this profile. An upstream SDK default does not override suite authority. |

[Service conventions](https://github.com/open-telemetry/semantic-conventions/blob/e10a930844c6951757a43b849d364f7d056ac32b/docs/resource/service.md),
[log fields and severity](https://github.com/open-telemetry/opentelemetry-specification/blob/18c5c1706e346fec3bdb5e135a6e3a78f461e586/specification/logs/data-model.md),
[scope and trace identifiers](https://github.com/open-telemetry/opentelemetry-specification/blob/18c5c1706e346fec3bdb5e135a6e3a78f461e586/specification/trace/api.md),
[metric units](https://github.com/open-telemetry/semantic-conventions/blob/e10a930844c6951757a43b849d364f7d056ac32b/docs/general/metrics.md)
and [custom namespace guidance](https://github.com/open-telemetry/semantic-conventions/blob/e10a930844c6951757a43b849d364f7d056ac32b/docs/general/naming.md)
are the exact external owners. Suite budgets may be stricter than their defaults.
Do not subtract wall clocks across processes to infer duration or causal order.

## Cross-process propagation

Default: no imported or exported context and no telemetry network access.
An inherited `TRACEPARENT`, `TRACESTATE`, `OTEL_*` or baggage value is not consent.
The future adapter may carry only a validated W3C `traceparent`, and only when
all of the following hold:

1. The caller explicitly selects propagation for this invocation.
2. Effective process authority grants both the exact `traceparent` field and
   the exact selected child/provider target; a provider cannot grant itself either.
3. That adapter declares its carrier and validates the W3C syntax, version,
   trace/span identifiers and flags before launch, without echoing rejected data.
4. The child environment/carrier is constructed from allowed values. Baggage,
   tracestate and arbitrary inherited tracing variables are omitted.

Construct a restricted context with empty tracestate; do not claim this subset
is a general-purpose W3C propagator. The [upstream propagation specification](https://github.com/open-telemetry/opentelemetry-specification/blob/18c5c1706e346fec3bdb5e135a6e3a78f461e586/specification/context/api-propagators.md)
defines full `traceparent`/`tracestate` behavior. Invalid or denied context starts
an independent trace or no trace, with a safe operational notice when applicable;
it does not expand provider authority or change domain work.

Flow already models telemetry field/target authority. That does **not** prove
its current runner transports trace context; the pilot must implement and prove
the selected carrier. Security grants stay in authority identity. Ephemeral
trace IDs, sampling, sinks and exporter state stay outside deterministic domain
identity. A valid trace is neither provider authentication nor accepted evidence.
The Flow extension event/result JSONL protocol remains distinct from a product
CLI's single command-result document; neither becomes an OTLP transport.

## Provisional GenAI example

An optional inference stage may correlate the existing command/run alias,
stage, capability, attempt, cache outcome, validation/refusal and measured
duration. Tool-use loops additionally report explicit bounded call/step counts.
No stage, model service, collector, inference request or paid endpoint is enabled
by this mapping. A local report remains usable offline.

| Observation | Provisional mapping and limitation |
| --- | --- |
| Operation | `gen_ai.operation.name` reflects the operation actually invoked. |
| Selected model | `gen_ai.request.model` is the requested model name, only after privacy admission; private local model names/paths remain local. |
| Reported model | `gen_ai.response.model` is reported by the provider; absence remains unknown. It is not independent proof of weights. |
| Provider | `gen_ai.provider.name` reflects instrumentation knowledge/API flavor, possibly an intermediary; do not infer an unobserved upstream provider. |
| Token totals | `gen_ai.usage.input_tokens` and `.output_tokens` represent known provider-reported totals. Missing is unknown, never zero. |
| Token counters | `gen_ai.client.inference.usage.input_tokens` / `.output_tokens`, unit `{token}`; record only known strictly positive contributions. |
| Cache/reasoning tokens | Upstream detailed counters are subsets of totals. Do not add them again; provider cache usage differs from a local artifact-cache hit. |
| Inference duration | `gen_ai.client.inference.duration`, histogram in `s`; the older `gen_ai.client.operation.duration` excludes inference/embeddings at this pin. |
| Cost | Suite-namespaced `provider_reported`, `estimated` or `unknown` classification, with currency and pricing-revision identity for estimates; no invented `gen_ai.cost` convention. |
| Local resources | Measured CPU/memory/device observations remain separately identified; they are not token counts, cost, quality or validation. |

See the pinned [inference contract](https://github.com/open-telemetry/semantic-conventions-genai/blob/e19717d65f890955777b044bca2a3cd71ae8b498/docs/gen-ai/client-inference.md),
[token metrics](https://github.com/open-telemetry/semantic-conventions-genai/blob/e19717d65f890955777b044bca2a3cd71ae8b498/docs/gen-ai/gen-ai-token-metrics.md)
and [other operation metrics](https://github.com/open-telemetry/semantic-conventions-genai/blob/e19717d65f890955777b044bca2a3cd71ae8b498/docs/gen-ai/gen-ai-metrics.md).
When both billable and consumed counts are reported, the pinned token convention
uses billable counts; retain that meaning rather than silently mixing measures.
Per-operation token histograms describe distributions, not an additional total
or cost source. Do not emit guessed usage samples or model identifiers as
unbounded metric labels. Prompts, messages, tool arguments/results, instructions
and content-capture hooks are excluded even though upstream has opt-in surfaces.

All GenAI fields remain provisional. The pilot must retain unknown, skipped,
unavailable and observed states separately and prove the registry/privacy limits.
Source review of this document supplies no exporter or native execution proof.
