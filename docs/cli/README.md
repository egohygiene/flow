# Initial Flow command source (#53)

The `flow` source target delivers the public `command` SDK. It is an unqualified
implementation checkpoint, separate from the synthetic provider and provisional
AI setup utility. The first concrete route is the existing independently released
Optiflow v0.1.1 read-only scan/report/exact-duplicate-review-plan adapter. Current
Optiflow development builds are not that release. No binary is installed or
published by these commands or by authoring this checkpoint. Missing immutable
Aniflow/Renderflow routes remain unavailable, without source-worktree imports.
All original #53 dependencies and release/clean-install acceptance remain open.

## Selected operation

Author an explicit [request](command-request.example.json) with normalized
absolute source, provider and new run paths. Native paths retain spaces, Unicode
and Unix byte paths through the existing encoded native-path contract. The source
must be a canonical directory; the run output is disjoint from source/provider
and has an existing canonical parent. All documents are private local evidence.
There is no ambient PATH/config discovery, download, installer or fallback.

The following are command source examples, not executed qualification receipts:

```console
flow doctor --provider-executable /selected/optiflow-0.1.1 --format json
flow inspect --request /private/request.json --format json
flow plan --request /private/request.json --output /private/plan.json --format json
flow approve --plan /private/plan.json --approved-by 'operator declaration' --acknowledge-trusted-unconfined --output /private/approval.json --format json
flow run --plan /private/plan.json --approval /private/approval.json --format json
flow status --run-directory /private/new-run --format json
flow resume --run-directory /private/new-run --format json
```

Doctor observes selected release bytes/host compatibility; it does not execute
`--version`. Inspect/plan observe source directory identity, exact selected
release bytes, effects and destination without launching a provider or writing a
run. Plan's optional explicit output is a no-clobber sidecar separate from source.
The source binding is **scope**, not a frozen content snapshot: native scan owns
member discovery and actual content evidence at execution. The plan states this
limit. A later fresh inventory requires a new plan/attempt.

Approve retains an explicitly supplied operator label and acknowledgment for
that exact local read-only operation. It is not authenticated identity, creative
approval, an Optiflow review-plan apply approval or a generic process grant. It
permits the pinned direct process and local evidence writes, with no source
mutation/network/signing/publication grant. Trusted-unconfined is explicit; no OS
sandbox or descendant-containment proof is claimed. Exact hashes do not prevent
an independently compromised host or authenticate the release publisher.

Run rechecks scope/provider identity and the exact declaration, reserves a new
private root, saves operation plan/approval before its first native version
probe, then delegates the existing Optiflow adapter. It retains native SQLite,
reports, plans and attempt receipts through that owner. Existing run roots never
become retries. The last outer completion marker binds the native receipt bytes;
it is not a replacement for native validation or generic durable graph evidence.
The selected route cannot apply/delete/quarantine/restore/finalize source files.

## Status, interruption and presentation

Status reopens the exact native attempt and its artifact bytes without launching
it. Multiple or partial receipt members refuse; no newest-file heuristic exists.
An interrupted/refused native scan remains recovery-required. Resume does not
replay it. If the native owner already completed and only the outer marker is
missing, explicit resume can bind that marker after current declaration,
provider and source-scope rechecks. Completed status is historical attempt/output
evidence, not a new scan or proof that all current source contents are unchanged.
`source_content_reobserved:false` makes this distinction machine-readable.

The existing native capture now accepts caller cancellation during file hashing,
before launch and while the direct child runs. The binary owns SIGINT/SIGTERM
handling on Unix; library callers provide their own signal. The direct child is
killed/reaped on cancellation, with partial native/operation state retained and
no fallback. Process-tree containment and platform/native signal/pipe behavior
remain unqualified. The known Aniflow alignment cancellation failure is unrelated
and remains unresolved, cause unproven.

`--format json` emits one `flow.command-result/v1` object with command, outcome,
exit code, result and fixed bounded diagnostics; no log/banner/progress is mixed
into machine stdout. Full local receipts may contain private paths/content and
are not portable telemetry. Human output uses plain text and escaped JSON, no
color dependency. No tracing subscriber/exporter is installed by the library.

| Exit | Meaning |
| --- | --- |
| 0 | Command complete; not a qualification/release claim |
| 2 | Argument contract |
| 3 | Invalid input or stale plan |
| 4 | Missing/incompatible/unavailable selected provider/route |
| 5 | Exact operation declaration required |
| 6 | Provider/evidence refusal |
| 7 | Recovery/reconciliation required; no automatic replay |
| 8 | Local I/O or presentation failure |
| 9 | Explicit partial native coverage |
| 130 | Caller/CLI cancellation |

Six SDK and three parser/envelope cases are authored/uncompiled/unrun. No product
CLI, provider, binary probe, test/compiler/build/dependency/format/schema/docs
validator, model, media or hosted CI operation ran during this source task.
Flow #88 remains last; ADR-0016 remains proposed. The supported product CLI,
released adapter and clean-install criteria remain visible in #53 rather than
being inferred from source inclusion or a help/version response.
