# Metadata-only support diagnostics, v1

Owner: Flow #21. This is a support interchange profile, separate from the
proposed telemetry adoption/exporter track (#15–#20). Product-owned SDKs and
thin owning-CLI adapters implement it without a sibling dependency or shared
observability crate. Source-local protocol implementations are deliberately
duplicated during this pilot; #18 still owns any later shared-package decision.

The profile provides explicit preview, no-clobber bundle creation, inspection
without extraction, and capability/health description. Native command envelopes
and exit/stream policies remain product-owned. A support operation's success
does not establish media acceptance, a resumable checkpoint, release support,
publisher authenticity or telemetry conformance.

## Inputs and records

`egohygiene.diagnostics-request/v1` is a closed object with `schema_version` and
one to four unique `categories`: `build`, `platform`, `contracts`, `health`.
The request is at most 4 KiB. It accepts no paths/attachments, raw logs,
environment values, headers, argv, prompts/lyrics/transcripts or run artifacts.
The path selecting the request belongs only to the CLI/I/O adapter. File I/O
requires canonical absolute input paths and an output in an existing canonical
parent directory; select a fresh filename. Symlinked parents are refused.

`egohygiene.diagnostics-bundle/v1` is one JSON container, at most 64 KiB. It
records the creation tool/version, declared creation time, policy revision,
selected static member names, typed payload sizes and digests. Payloads are at
most 4 KiB; static members are `build.json`, `platform.json`, `contracts.json`,
`health.json`. No files are extracted and no container compression is accepted.
Unknown fields, duplicate categories, unsafe/member names, changed digests,
aliases, oversized records and evidence-authority claims are refused.

The product uses its existing digest owner: SHA-256 for Flow/Aniflow/Renderflow,
BLAKE3-256 for Optiflow. Algorithm names are explicit. An inspector that lacks
the selected digest algorithm returns unavailable rather than accepting an
unchecked hash. These are integrity observations, never signed provenance.
All tools inspect their own bundles; SHA-256 implementations can also inspect
other tools' SHA-256 profile bundles. This is not universal hash negotiation.

Preview reports exact categories/names/counts, exclusions, redaction policy,
privacy warnings, fixed limits and an
estimated encoded size. Creation time can differ at later creation. Generation
uses only compiled metadata. Current source-contract IDs describe this support
profile, not released media capabilities. Compiled OS/architecture do not prove
actual hardware/backend qualification.

## Health meaning

`egohygiene.observability-health/v1` reports product-owned compiled capabilities.
Current sources have no owned telemetry exporter; collector and flush states
are therefore `not_applicable_no_exporter`, with queue/drop measurements `null`.
Unknown is not zero. Renderflow has compiled local tracing; the other current
products have no integrated operational logging facade. Runtime subscribers or
sinks installed by an embedding caller are unobserved. Diagnostics never creates
or replaces a global subscriber, reads telemetry environment configuration,
probes a collector, grants propagation or initializes provider discovery.

Support commands skip Renderflow's ordinary logging bootstrap so their output
remains clean. This does not change ordinary Renderflow logging or resolve the
separate #16/#20 migration issues. Actual OTLP exporter/queue/outage/flush
instrumentation belongs to #17; this profile cannot mark it implemented.

## I/O and privacy

On applicable Unix hosts, non-following/nonblocking opens bind singly linked
regular files and recheck descriptor/path identity, lengths, mtime, ctime,
ownership and permissions around bounded reads. Outputs are explicitly selected,
fresh 0600 files; existing files and links are never replaced. Creation reads
back the exact generated file. Failure/cancellation preserves partial output,
which must not be adopted as a completed bundle. Operations have a cooperative
five-second budget; OS/kernel calls and hostile concurrent filesystems are not
an atomic snapshot or hard real-time guarantee.

Hosts lacking the required file-identity adapter report file I/O unsupported
before creation. Preview and compiled capability descriptions remain available.
Windows support is a visible #71/#72 platform gap; no Unix/WSL execution is
relabeled native Windows qualification.

Archives can contain untrusted producer/host claims supplied by someone else.
Inspection verifies structure and supported digests and reports only fixed
summaries, not arbitrary payload text. `claims_authenticated`, `domain_evidence`
and `network_contacted` remain false. Review the exact file privately before
deciding to share it. No transmission, attachment, upload or cleanup is automatic.

## Native entry points

| Product | Preview | Create | Inspect | Health |
| --- | --- | --- | --- | --- |
| Flow | `flow diagnostics-preview --request FILE --format json` | `flow diagnostics-bundle --request FILE --output NEW --format json` | `flow diagnostics-inspect --bundle FILE --format json` | `flow doctor-observability --format json` |
| Aniflow | `aniflow --output json diagnostics preview --request FILE` | `aniflow --output json diagnostics bundle --request FILE --output-file NEW` | `aniflow --output json diagnostics inspect --bundle FILE` | `aniflow --output json diagnostics doctor` |
| Optiflow | `optiflow --json diagnostics preview --request FILE` | `optiflow --json diagnostics bundle --request FILE --output NEW` | `optiflow --json diagnostics inspect --bundle FILE` | `optiflow --json diagnostics doctor` |
| Renderflow | `renderflow diagnostics preview --request FILE` | `renderflow diagnostics bundle --request FILE --output NEW` | `renderflow diagnostics inspect --bundle FILE` | `renderflow diagnostics doctor` |

Renderflow's support family emits structured JSON directly; the other products
use their native machine-result envelopes. Aniflow's existing machine-error
stderr placement remains unchanged. Full stream migration is separately owned.
See the [operator runbook](support-runbook.md) and shared synthetic conformance
driver. Source inclusion is not passing independently built CLI evidence.


## Shared acceptance fixtures

The [fixture inventory](fixtures/support-conformance.v1.json) and the
[conformance driver](../../tools/check_support_clis.py) describe one synthetic
profile exercised through each independently built native CLI. This driver
never builds, installs, starts a collector or runs a media provider. After local
execution is authorized, select exact canonical binary paths:

```sh
python3 tools/check_support_clis.py \
  --flow /absolute/flow --aniflow /absolute/aniflow \
  --optiflow /absolute/optiflow --renderflow /absolute/renderflow \
  --output /absolute/new-support-conformance.json
```

The equivalent Task entry is `diagnostics:conformance`, with explicit `FLOW`,
`ANIFLOW`, `OPTIFLOW`, `RENDERFLOW` and `OUTPUT` variables. It does not build or
download products. `diagnostics:conformance:test` checks the driver's negative
observers without launching a product. The current unreleased Aniflow candidate
uses `--output-file` for the bundle destination; `--output` remains the global
human/JSON presentation option. The older colliding destination spelling
panicked and is not retained as an alias.

Omitting a binary records that tool as unobserved; a partial run cannot satisfy
all-CLI acceptance. The driver retains executable SHA-256/length, actual case
results and observation time only after execution. It searches generated bundle
bytes and command output for a synthetic private canary carried by environment,
headers, paths and a refused request/argv. It does not inspect real credentials
or content and does not prove absence of every possible side effect. Each child
has a 15-second subprocess timeout; captured-output checks are post-capture and
are not an operating-system memory sandbox. Use only the explicitly selected
trusted build under review.

Every negative case must match its native command and exact support-domain
refusal. A panic, argument error or unrelated failure is not a passing refusal.
The driver retains completed cases and a fixed failure reason for a failed
product, continues the other selected products and exits nonzero if any fail.
`all_selected_passed` covers only selected products; `complete` additionally
requires all four. Existing report paths are refused before any command runs.
Actual selected Linux observations and their remaining limits are recorded in
[the dated checkpoint](../work/21/LOCAL-SUPPORT-20261114.md).

The bundle/request/health schemas are in `schemas/support-*-v1.schema.json`;
digest equality, canonical typed serialization, cross-field agreement and total
encoded byte limits are additionally enforced by the owning SDK. Native copies
under each product's `docs/support` describe the same pilot revision. Source
inclusion and fixture names are not executed conformance evidence.
