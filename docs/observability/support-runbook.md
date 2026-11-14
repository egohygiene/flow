# Local observability and support runbook

This runbook applies to #21's metadata-only profile. Read the exact supported
[CLI/SDK contract](support-profile.md) first. It does not install a collector,
enable telemetry, run a provider or change workload/approval/recovery state.

## Disabled and local-only operation

Start with the owning CLI's support doctor. `logging_compiled` describes the
product, while `logging_runtime_observed: false` means the SDK did not inspect
an embedding caller's subscriber. No telemetry configuration is read implicitly.
An inherited `OTEL_*`, endpoint or credential does not enable this profile.

Ordinary Renderflow commands currently initialize human tracing on stderr;
`--verbose` selects DEBUG and `--debug` TRACE. There is no implemented JSONL,
file/rotation or OTLP switch in that current bootstrap. Support commands skip
it. Flow/Aniflow/Optiflow do not currently provide the proposed logging facade.
Their domain result/progress behavior remains independently versioned.
Do not invent a product log-file location or infer one from this proposal.

Use shell redirection only for a deliberately selected private operational
capture, separately from machine stdout. Such captures can contain paths,
provider diagnostics or user content and are **not eligible bundle inputs**.
Do not attach them to a public issue by default. JSONL/sink/filter/retention
adoption is tracked in #15–#20, not silently enabled by diagnostics.

## Preview, create and inspect

Create a small request in an existing private local directory:

```json
{"schema_version":"egohygiene.diagnostics-request/v1","categories":["build","platform","contracts","health"]}
```

Use the product-owned preview command, review categories/exclusions/limits,
then explicitly select a fresh output filename for bundle creation. Inspect
that exact file and privately review its contents before sharing. Support
creation writes no archive into a run workspace by default and performs no
network operation. Preserve any partial file on failure; select a new destination
instead of retrying onto it. Inspection never extracts or executes anything.

The file is private (0600 on applicable Unix hosts); keep its parent directory
private too. Local filesystem placement is operator-owned: a network-mounted
destination is not proof of local data residency. Retention has no timer or
automatic deletion. To remove an obsolete support artifact, remove only the
specific reviewed file under your control, never a media/run/workspace tree.

## Collector unavailable, backpressure and shutdown

Current owned exporters are unsupported, so the doctor reports collector/flush
as not applicable and queue/drop values as unknown/null. These are not outage
measurements or a successful flush. Do not troubleshoot credentials or start a
network service to turn those states green. The missing implementation is #17.

For the future opt-in exporter pilot, keep a local collector bound to loopback,
use synthetic metadata only, pin its version/configuration, and retain actual
outage, queue-byte/record bounds, dropped counts, retry and shutdown observations.
The [local configuration candidate](../../examples/observability/local-collector.yaml)
is illustrative and unexecuted. It cannot prove current product OTLP support.
Never place authentication headers, source content or private paths in debug
exports. Remote collectors require separate explicit endpoint/credential policy.

If an enabled future exporter cannot connect, stop exporting or revert to the
documented local/no-export mode. A bounded warning/drop is operational; it must
not change domain success, validation, authority, output digests or reuse. Queue
pressure must stay within both record and byte budgets. Shutdown must observe
its actual bounded drain result; missing completion remains unknown. Preserve
domain evidence and report unresolved loss without fabricating measurements.

## Correlation, recovery and sharing

Use existing exact run/plan/receipt identities when inspecting a domain failure.
A support bundle has its own digest and no authority to create a checkpoint,
retry a process, approve media or resume an uncertain effect. Telemetry IDs and
logging settings do not enter semantic plan/cache/provider-lock identities.
Keep any private alias-to-run mapping separately controlled; this profile
creates none. Runtime credentials stay at their original boundary and never
enter requests, support payloads, generic provider envelopes or CLI argv.

For a corrupt/changed bundle, preserve the file and inspect a fresh explicitly
generated one. For unsupported host/algorithm, use the matching owning CLI on
an applicable host; an unchecked inspection is not a valid alternative. Do not
upgrade a support observation into signed release or provenance evidence.

Run the shared canary/bounds/no-clobber cases against independently built CLIs
when focused execution is authorized. Record exact executable/source identities,
host, native result/exit/stream and per-case observed state. Not-run, unsupported
and failed cases remain distinct. Final broad suite acceptance stays in #88.
