# Observability conformance plan

This is an **unexecuted plan** for the proposed [profile](README.md), not a
passing report. #15 defines the contract; [#16](https://github.com/egohygiene/flow/issues/16)
owns separate Renderflow and Aniflow pilot issues/PRs, with later exporter,
shared-package and remaining-CLI work under #14. Do not instrument all four
products merely to close this document task.

## Reusable harness shape

Use each repository's existing test harness and synthetic inputs. A fixture
adapter invokes a public library or built CLI with literal argv, captures stdout
and stderr separately, injects a fake monotonic/wall clock and bounded sink where
the existing seams permit, and retains exact product/contract/source revisions.
No sibling source dependency, new framework, live collector, credentials or real
user content is needed for deterministic cases. Native/exporter proof is a
separate explicitly enabled layer; skipped work never becomes passed work.

Use recognizable synthetic sentinels for secrets, paths with spaces/Unicode,
argv, headers, caption/prompt text, provider captures and hostile terminal
control sequences. Assert absence across every operational sink and fail on
unknown keys; merely finding `[redacted]` does not prove no other leak occurred.
Inspect native command results separately under their own existing schemas.

## Required matrix

| Case | Required observation |
| --- | --- |
| Human TTY / non-TTY / redirected | Useful results and explicit states; progress/logs on stderr; redirected output has no cursor controls; no color-only meaning |
| Machine success / failed / partial / refused / unavailable / cancelled | Exactly one native result on stdout; no operational log, banner, progress or ANSI; correct native outcome/exit |
| Help/version / raw export | Explicit exceptions match the mode matrix; no accidental extra envelope |
| Parse/config failure | Safe recognized machine mode yields one typed stdout failure; ambiguous/malformed selection yields sanitized usage with no argv echo; test `--`, duplicates and non-UTF-8 argv |
| Broken stdout / serialization failure / panic | No fabricated success or second result; documented incomplete-result boundary; panic payload/secret/backtrace absent from default operational output |
| Human vs JSONL vs file sink | Domain stdout bytes/semantics unchanged; each JSONL line has only registered keys, valid encoding and budgets; no mixed progress text |
| CLI/env/config/default precedence | Independently vary each setting; explicit aliases/conflicts behave as specified; ambient `RUST_LOG`/`OTEL_*` cannot enable export |
| Color / `NO_COLOR` / accessibility | CLI override applies only to human TTY; machine/redirected output remains plain; semantic words identify states |
| Library embedding / repeated bootstrap | Caller-owned subscriber remains active; no global initialization/panic hook/environment side effect; CLI setup failure is fallible |
| Provider failure/captures | Only approved code/class/count leaves local evidence; raw stderr/stdout, args, content and reversed encodings never become telemetry |
| Allowlist / unknown field / injection | Source and sink filtering agree; unknown fields are omitted; control characters cannot forge new records |
| Default offline | No telemetry socket, DNS/discovery, telemetry file or background exporter; explicitly selected domain network use remains a separate policy |
| Collector absent / sink full / disk failure | Bounded memory, deadlines and warnings; telemetry drops are counted; domain result and evidence unaffected |
| Buffer/backpressure | Queue records AND bytes capped; oversized events dropped; growth remains bounded with many distinct bad fields/series |
| Flush/shutdown/cancellation | Guard lives through result emission; drain deadline enforced; no worker survives shutdown; cancellation does not rewrite domain result |
| Identity invariance | Change logging/export/filter/color/sampling/correlation settings; compare the same deterministic plan/cache/compatibility/provider-lock/artifact identities |
| Evidence nuance | Existing native execution timestamps/report digests remain valid; new telemetry fields do not enter semantic identity; real authority-grant changes still change authority identity |
| Child propagation denied/allowed | No inherited carrier by default; exact field AND target permission required; only validated traceparent; no baggage/tracestate, authority escalation or domain-envelope injection |
| Trace sampling | Ratios 0/1 and deterministic intermediate sampling behavior; no effect on operation count, result, acceptance or evidence |
| Clocks/retries/concurrency | Monotonic nonnegative duration; wall jumps don't alter duration; actual parents/attempts retained; no fabricated global order |
| Metric cardinality | Adversarial IDs/model names/paths cannot create series; finite map and 128-series cap with one reserved overflow series |
| Inference synthetic response | Requested/reported identity distinct; unknown tokens/cost not zero; cached/reasoning subsets not double-counted; no payload capture |

Identity comparisons must select deterministic fields, not demand byte-identical
entire execution reports containing legitimate time or attempt observations.
Conversely, do not exclude a newly added telemetry field from the assertion just
to make the test pass. Document the exact semantic projection before running it.

## Runtime and release measurements

For each pilot, retain built-binary size, cold-start distribution, representative
no-op/disabled versus enabled runtime and memory overhead, queue stress results,
dependency/MSRV change and platform limits. Record sample size, host, toolchain,
exact revision and methodology; no overhead figure is invented in this contract.
Native tools are used only where required for the representative operation, with
their existing qualification rules and budgeted fixtures.

Exporter/propagation qualification additionally records explicit opt-in,
collector protocol/configuration, endpoint authorization and bounded loss on
outage. Local fake collector tests establish only that fixture behavior;
they do not establish hosted-service retention, privacy or reliability.

## Review and continuation record

The later report lists command/mode/platform, expected and observed behavior,
source/contract pins, pass/fail/skipped/not-run, and links to retained bounded
evidence. Do not reduce this to an overall green checkbox. Conformance requires
both products to fit the profile honestly; revise the proposed contract before
extracting any shared API when a real pilot contradicts it.

This checkpoint performed source and document review only. All executable
checks, including this matrix, remain deferred by the maintainer. No product
test, build, lint, schema validator, native process, exporter, package install,
Actions dispatch or Actions polling is part of #15's authored evidence.
