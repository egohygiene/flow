# Durable lifecycle scenarios

Flow #65 and #66 qualify durable recovery through the public APIs introduced by #49
and #64. The test-owned corpus lives in `tests/lifecycle_matrix/`, with 49 fixed
recipes and expected outcomes in `tests/fixtures/lifecycle-scenarios.v1.json`.
It supplements the [acceptance matrix](acceptance-scenarios.md). It does not add
a scheduler, a runtime state model, or a new public contract.

## What the evidence proves

Every recipe runs twice in independent workspaces, with the exact compiled
synthetic provider package. Both executions must produce equal normalized
receipts. Expectations are checked in as reviewed assertions; tests never
regenerate expected outcomes from observed results.

| Family | Scenarios | Required evidence |
| --- | --- | --- |
| Cancellation | Before launch, after intent, during provider execution, between steps | Ordered snapshots, attempt numbers, absent checkpoints, current graph eligibility |
| Provider failures | Timeout, nonzero exit, actual Unix signal, stdout/stderr overflow, partial output | Exact typed process or acceptance error; failed evidence retained without promotion |
| Fresh-process restart | Completed run, partial graph, host exit after intent, host exit after provider reaping | A new host process reopens the same workspace, reconstructs current public contexts, and reuses or continues only eligible work |
| Recovery | Retry cancelled or interrupted work, abandon invalid partial output, refuse unacknowledged or completed retries | Explicit recovery decisions, ordered attempt transitions, preserved uncertain output before retry |
| Staleness | Input, exact plan, configuration values, provider identity, executable bytes, validator implementation, output bytes | Local stale boundary, downstream invalidation, independent branch reuse, dependent launch refusal without state writes |
| Store refusal | Changed validation profile, corrupt checkpoint context, checksum mismatch, partial write, future schema, malformed JSON | Exact reopen refusal; no fallback to an older accepted snapshot |
| Authority | Source mutation, upload, publication, signing, network, paid AI service, destructive action | Declaration denied by unchanged operator lock; exact-profile escalation denied; recorded denial and blocked descendant have zero launches/effects; independent positive control runs once |
| Effect accounting | Completed reuse after fresh-process restart, stale authority, failed/interrupted retry, replayed recovery decision, explicit abandonment | Persisted authority before launch, observed counters, unchanged completed work, exact authority/attempt correlation, and no launch from approval alone |
| Cleanup residuals | Cancellation after partial cleanup; actual nonempty-directory removal failure | Reaped direct provider, typed cancellation/failure, absent checkpoint, preserved source/candidate/unfinished work, blocked descendant, and refusal to relaunch on reopen |

The graph fixture is A → B plus independent C, with separate source bindings.
It proves ordering dependencies and stale-ancestor propagation. The existing
provider-kit compositions separately prove artifact handoff; #64's graph tests
cover transitive chains, fan-in, and complete context inventories.

## Trace and receipt contract

The test-owned `flow.lifecycle-scenario-catalog/v1` names each recipe's provider
mode, graph shape, recovery classification, expected ordered state frames, and
expected ordered assessments. Frames project the existing `RunState` snapshots:
sequence, status, attempt, checkpoint presence, typed failure, and recorded
recovery actions. Assessments reuse `RunStepAssessment` fields directly.

`flow.lifecycle-scenario-receipt/v1` adds the exact recipe digest, observed package,
executable, manifest, binding and source identities, immutable plan digest,
retained history digest, and candidate artifact digests. On corrupt-history
recipes, frames describe the valid history before fault injection; the final
history digest identifies the rejected snapshot bytes. No successful reopen or
new assessment is implied by those frames.

Fixture version `1.1.0` adds an optional closed `outcome.safety` projection for the
15 #66 recipes. It records ordered launch/effect/cleanup counter observations,
authority decisions projected to step/attempt/granted, typed refusal codes, and a
closed residual disposition. Older recipe outcomes remain unchanged. Rust checks
every authority/recovery identity against the saved plan and compares both fresh
executions; Python rejects changed or missing safety evidence. This is test-owned
evidence, not an extension to the durable run schemas.

The fixture preserves originals before deliberate byte corruption and retains
failed candidates. Interrupted output is moved to a separate retained file
before an explicitly approved retry. An ignored Rust helper is a subprocess
entry point, explicitly executed by the matrix; it is not a skipped scenario.
Host exit code 73 bypasses destructors and proves lock release and fresh reopening.
The two exit points are before provider launch and after the runner reaps its
direct child but before durable acceptance. They do not simulate host death
while an unconfined provider or its descendants remain alive.

Recovery classifications describe each **test recipe's operator decision**.
Two provider-failure recipes revalidate the exact retained process transcript
through Flow: a provider-classified retryable failure and a terminal validation
failure. Both remain unaccepted; only an explicit operator decision moves them
to pending or abandoned. Provider retryability hints do not trigger execution.
Cancellation or interruption may be retried after acknowledgement and fresh
context checks; invalid partial output is abandoned in its terminal recipe.
Flow still exposes typed `RunFailure` and `ResumeEligibility`, not an automatic
retryability policy. Reopening, assessing, or deserializing a receipt grants no
authority and launches nothing. The [durable-state guide](durable-state.md) owns
production storage and recovery semantics.

## Authority, effects, and cleanup boundaries

The effect provider writes only a bounded, allowlisted local witness journal and
synthetic candidate files. Before entering the runner, a cancellation callback
inspects the actual persisted running snapshot and granted authority while the
counters are still unchanged. The counters distinguish launches, simulated effects,
cleanup starts, and disposable items removed. A repeated launch remains visible
even if a provider later refuses to overwrite an existing candidate.

Seven denial recipes test both declaration-versus-lock resolution and attempted
per-invocation grant escalation. Each denial is then recorded with `deny_pending`;
execution of that step and its dependent is refused. Independently authorized C
executes the same harmless counter mode as a positive control. Upload is represented
by a network endpoint and paid-service use by an AI-provider grant: v1 has no separate
upload or billing authorization dimension. These cases perform no real transfers,
publication, signing, payments, source mutation, or destructive collection actions.

Completed work stays at one launch/effect through reopen, reuse assessment, rejected
execution, and rejected retry. An uncertain attempt stays at one effect until a
fresh matching recovery decision explicitly acknowledges uncertainty. Missing
acknowledgement, changed authorization identity, changed grants, and reuse of an
earlier recovery decision ID are rejected without history or counter changes.
Approval alone leaves counters unchanged. A subsequent explicit retry reaches two
effects while retaining the first candidate. **That second effect is intentional
evidence of the guarantee's limit:** accepted work is not silently repeated, but
Flow cannot undo or prove exactly-once external effects from an unconfined provider.

Cleanup happens inside the synthetic provider. It creates a candidate, removes one
disposable file, and leaves another item unfinished. One recipe cancels after an
atomic readiness signal; the other encounters a real nonempty-directory removal
error. Neither gets a checkpoint. Reopening retains the original source, candidate,
unfinished evidence, and history and requires a recovery decision. Flow provides
no cleanup scheduler, automatic rollback, or compensation here. Moving a candidate
aside before retry is an explicit test-operator action, not automatic Flow behavior.

## Parent #31 acceptance reconciliation

| Original criterion | Executable evidence |
| --- | --- |
| Compatible resume preserves accepted completed work | `completed-restart`, `partial-restart`, `effect-completed-reuse`; #64 graph reuse tests |
| Changed/corrupt identity invalidates affected and dependent work | `changed-*`, `corrupt-*`, `effect-stale-authority`; #64 transitive/fan-in/context inventory tests |
| Retry never silently duplicates accepted or consequential work | `retry-completed`, `effect-completed-reuse`, `effect-retry-failed`, `effect-retry-interrupted`, `effect-recovery-decision-replay`, `effect-abandon-interrupted`; uncertain repeats require explicit acknowledgement |
| Authority denial precedes prohibited effects | Seven `deny-*` recipes with zero denied/descendant counters and a working independent positive control |
| Interrupted runs retain bounded inspection/recovery evidence | Host-exit recipes, preserved retry candidates, `cleanup-cancelled`, `cleanup-failed`, and enforced history/artifact budgets |
| Traces/explanations match #3 durable schemas | Public `RunStore` APIs, validated `RunState` history and `RunAssessment`, typed failures/refusals, allowlisted receipts and privacy canaries |
| Suite is deterministic, hermetic, budgeted, drift-checked | Every recipe twice in fresh roots, strict Python receipt verification, source identities, kernel limits, and negative report-gate tests |

This completes the synthetic lifecycle proof when #66 is merged. Parent #13 remains
open for real released-provider integration and the #32/#33/#34 qualification gates.

The [#66 local validation record](../validation/flow-66-local.json) pins the tested
commit, source identities, report/receipt digests, and executed checks. Rust 1.85
ran all targets and both corpora; stable Rust ran the lifecycle corpus. Hosted CI
and macOS/Windows results were not checked for this handoff. Full reports remain
reproducible with the commands below; the committed record is their compact summary.

## Running and checking coverage

Populate Cargo's locked cache, then run on Linux:

```console
python3 tools/run_acceptance_scenarios.py --lifecycle-only
python3 tools/run_acceptance_scenarios.py --all-targets
python3 tools/test_lifecycle_report.py
```

The first command writes `target/lifecycle-scenarios.v1.report.json`. The second
runs all Rust targets and verifies both this corpus and all 81 acceptance rows.
CI runs it on Rust 1.85 and stable and uploads both normalized JSON reports.
The driver removes older reports before running, captures output with a bound,
and refuses any failed test, missing/duplicate/unknown receipt, changed expected
trace, unsupported shape/version, recipe digest mismatch, inconsistent provider
identity, or fixture source mismatch. It hashes the tested sources before and
after execution; changes during validation prevent a successful report.

Reports carry source and catalog digests, the actual toolchain, checked budgets,
coverage gaps, and receipts. Rust compares both fresh-workspace executions;
Python independently checks completeness and each exact expected outcome.
Failure logs and raw process output remain local, outside CI uploads. The
portable allowlist excludes absolute roots, PIDs, timing samples, configuration
values, source bytes, provider messages, and raw streams. Canary assertions check
fixture paths and values. This is not a general redactor for arbitrary content.

## Resource bounds and remaining work

| Resource | Qualification bound |
| --- | --- |
| Rust test workers | One; explicit child-process contention tests still overlap their opens |
| Per recipe | Two executions; each completed execution must take at most 60 seconds |
| Entire Cargo command | 600 seconds, including compilation; timeout kills and reaps the test command's process group |
| Captured command output | 4 MiB enforced while reading, before any unbounded capture |
| Test/provider process address space | Linux `RLIMIT_AS`: 4 GiB per process, inherited by host children and providers |
| Single file written by a test/provider | Linux `RLIMIT_FSIZE`: 16 MiB; core dumps disabled |
| Receipt | 32 KiB |
| Durable history | At most 16 snapshots and 2 MiB per recipe workspace |
| Fixture artifacts | At most 16 recursive files and 1 MiB per node artifact root |
| All fixture files | 128 MiB across the recipe's roots, including copied provider binaries and retained evidence |
| Provider execution | Existing pinned kit deadline (5 seconds), 64 KiB stdout and stderr limits, cancellation grace |

The driver serializes Rust test workers because concurrent fork/exec can briefly
inherit another worker's Unix `flock` descriptors and make an unrelated immediate
reopen report `Busy`. This is test isolation, matching the portable store suite;
no lock assertions are weakened and the deliberate contention tests still run.
Concurrent unrelated host spawning is not qualified by this serial tier.

The Cargo target runner applies kernel limits to test executables, not compiler
or linker processes. Direct `cargo test` is useful for debugging but does not
establish the memory/file qualification. Per-recipe elapsed time and aggregate
filesystem totals are checked after execution; an otherwise hung helper is
bounded by the driver's command deadline. The address-space limit is not an
aggregate process-tree RSS quota. Killing the test process group on failure is
test supervision, not a production descendant-containment claim.

This tier qualifies Linux synthetic trusted-unconfined providers. Existing
macOS/Windows durable-store jobs provide narrower portability evidence. Real
provider releases, sandbox enforcement, hardware power loss, every crash window,
provider-native checkpoints, migration, automatic scheduling/retry, and
exactly-once external effects remain outside this proof. **Flow #66** supplies the
bounded authority/effect/cleanup qualification above. Parent **#31 stays open** until
its review PR is merged and the acceptance reconciliation is recorded. Future
scenario visualization in #62 can consume these checked receipts while retaining
these coverage limits.
