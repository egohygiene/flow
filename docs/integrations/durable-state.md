# Durable prepared execution

Flow #49 adds a library API for persistent execution of fully prepared process
steps. It builds on the exact subject, authority, transcript, and artifact gates.
Flow #64 adds fresh graph eligibility and safe caller-selected dependent
execution. There is no product CLI or graph scheduler. #31 is qualified by
#65 and #66 through the [lifecycle corpus](lifecycle-scenarios.md), including
authority/effect and residual-state proofs;
#53 owns the supported CLI.

## Public entry points

1. Construct a `ProcessStepContext` with the current resolution, invocation,
   subject lock, opaque authorization, artifact bindings, and explicit execution
   and artifact roots. Roots and tokens remain caller-local.
2. Call `PlannedStep::prepare`, then assemble a `RunPlan` with stable `plan:`,
   `run:`, and `step:` identities and sorted dependencies on preceding steps.
   Plans contain fully pinned expected inputs; unresolved future outputs require
   later planning support.
3. Call `RunStore::create` with a new workspace path. Its parent must exist.
4. Supply a `RunStepContext { step_id, context }` for every step in plan order.
   Call `execute_in_plan` with the expected plan, selected step ID, that complete
   inventory, and explicit secrets, cancellation, and event adapters. It freshly
   assesses the graph and executes only a ready step. The store persists intent
   before launch, then an accepted checkpoint or a bounded typed failure. It
   holds its exclusive workspace lock throughout.
5. Drop the handle and use `RunStore::open` after restart. `state()` exposes the
   recorded status without writing files or launching a process.
6. Use `assess_run` with the complete expected plan and context inventory.
   `Reusable` requires matching intent, validator identity, fresh input/output
   observations, and freshly reusable prerequisites. The returned versioned
   report describes eligibility without reconstructing acceptance or authority.

The original `assess` and `execute` entry points remain supported for root steps
only. They return `DependencyEvidenceRequired` for any dependent step, regardless
of its recorded prerequisites. Use the graph APIs for dependent execution.

`RunState::new` remains an in-memory model. `Orchestrator` and `LocalProcessRunner`
remain useful low-level seams for unit tests and controlled integrations. Callers
requiring restart semantics must enter the durable coordinator instead of
treating a low-level runner result as a persisted run.

## Records and identity

| Contract | Meaning |
| --- | --- |
| `flow.run-plan/v1` | Immutable prepared steps, dependency order, and exact context |
| `flow.run-state/v1` | Complete run state, numbered history, decisions, and checkpoints |
| `flow.run-checkpoint/v1` | Accepted step bound to plan, context, and attempt |
| `flow.run-artifact/v1` | Input/output role and existing host observation record |
| `flow.run-authority/v1` | Granted launch authority or explicit operator denial |
| `flow.run-validation/v1` | Accepted evidence digests and validation implementation identity |
| `flow.run-recovery/v1` | Explicit retry/abandon decision and uncertainty acknowledgement |
| `flow.run-snapshot/v1` | Storage envelope and state integrity digest |
| `flow.run-assessment/v1` | Read-only graph eligibility bound to current state and plan digests |

Every checkpoint binds the complete plan digest and its step context digest.
That context includes provider identity/version/integrity, capability definition,
process interface, exact subject lock, configuration schema and claimed digest,
an independently computed digest of actual configuration values, invocation,
authority/enforcement/grant digests, bindings, and expected input digests.

Validation records hash accepted execution evidence and the retained artifact
records without storing provider messages. The implementation identity includes
the validation sources and locked dependency recipe compiled into the library,
crate version, and operating-system name. It is reproducibility metadata, not
publisher authentication or a digest of the final application binary. Changed
identity requires explicit future compatibility/migration work.

Objects use closed fields. Canonical identity serializes recursively ordered
object keys as compact UTF-8 JSON; arrays retain their declared order. The Rust
semantic validators additionally enforce identity and state relationships that
JSON Schema alone does not express. See `contracts/fixtures/state/` for examples.

## Fresh graph assessment

`assess_run` verifies retained history and the exact expected plan before
inspecting current evidence. The context inventory must identify every step
exactly once in plan order. Missing, extra, duplicate, or reordered entries are
refused. Contexts for blocked descendants are required, but their future input
bytes are not observed until all prerequisites are reusable.

| Eligibility | Current meaning |
| --- | --- |
| `ready` | Pending, matching current evidence, with all prerequisites freshly reusable |
| `reusable` | Historically accepted, with matching current local and prerequisite evidence |
| `invalidated` | Local evidence is stale, or a prerequisite is invalidated |
| `dependency-blocked` | A prerequisite is pending, awaiting recovery, abandoned, or itself blocked |
| `approval-required` | A running, failed, cancelled, or denied step needs explicit recovery |
| `abandoned` | The operator abandoned this step and current evidence still matches |

Dependency classification takes precedence over local checks. Each report entry
retains the recorded status, a typed local stale boundary when checked, and all
non-reusable direct prerequisites in declared order. An invalidated prerequisite
takes precedence over other blockers. Follow `blocked_by` entries to the root
cause. A descendant's absent local reason means its local evidence was not checked.

For a graph with `A → B → C` and independent `D`, stale A inputs invalidate A, B,
and C for reuse while D keeps its own eligibility. A historically succeeded step
stays succeeded in the immutable history. Assessment never rewrites checkpoints,
reruns work, deletes artifacts, or claims migration into a changed plan.

Reports include plan/state digests and snapshot sequence, with no timestamp or
absolute roots. `RunAssessment::validate_against` checks shape and correlation
only; it cannot prove external evidence is still fresh. Saving, editing, or
deserializing a report grants no authority. `execute_in_plan` accepts no report:
it recomputes eligibility and enters the ordinary durable launch path only for
the caller-selected ready step. It returns typed `Ineligible` otherwise.

Keep provider packages, sources, and artifacts quiescent during assessment and
execution. The workspace lock protects state coordination; these sequential
observations do not lock external writers or form an atomic filesystem snapshot.
Validation implementation changes invalidate old checkpoints conservatively;
no compatibility override or cross-plan migration is shipped here. See
[ADR-0012](../architecture/governance/decisions/ADR-0012-fresh-graph-assessment.md).

## Commit and restart behavior

A workspace contains `workspace.lock`, numbered immutable `*.json` snapshots,
and, only while a commit is unfinished, `snapshot.pending`. Each snapshot records
the previous state's digest. Flow syncs the pending file before renaming it and
never replaces an existing numbered snapshot. Unix also syncs parent directories.
Windows v1 guarantees cover process crashes; directory/power-loss durability is
not claimed. Use a local filesystem with working locks and atomic rename.

The lock is advisory and OS-backed. A concurrent open returns `Busy`; a killed
writer releases the lock. Do not delete or replace `workspace.lock` to bypass
contention. A failed commit poisons that handle where commit progress is uncertain;
reopen to inspect whether the new snapshot actually committed.

Reopen checks the full retained sequence. Pending bytes, corruption, unsupported
versions, identity conflicts, unsafe entries, and sequence gaps produce typed
refusals. Flow preserves the directory for inspection instead of guessing that an
older snapshot is safe. A checksum is not authentication: a hostile writer can
rewrite history, and removing a complete tail is not detectable without an
external anchor.

| Recorded condition | Meaning after reopen |
| --- | --- |
| Pending | Ready only when dependencies, current context, and input observations match |
| Running | Host completion is uncertain; explicit recovery is required |
| Succeeded | Eligible for reuse only after current evidence is checked |
| Failed, cancelled, denied | Inspect the typed outcome; explicit recovery is required |
| Abandoned | Operator stopped this step; no automatic execution |

`cancel_pending` and `deny_pending` record decisions before provider code runs.
Normal timeout/cancellation retains the runner's direct-child cleanup semantics.
A host crash cannot promise that an unconfined provider or its descendants died.

`decide_recovery` requires matching fresh context, authority, a stable decision ID,
and acknowledgement of uncertain effects. Retry records a pending step without
launching it. Operators must inspect and preserve/quarantine residual outputs and
resolve surviving processes before an explicit new attempt. Flow never deletes
or overwrites those artifacts automatically. A succeeded step cannot be retried
through this API. Invalidation reports describe current reuse eligibility without
authorizing reruns. Automatic reuse scheduling, cross-plan reuse/migration,
provider-native checkpoint restoration, and exactly-once external effects remain
outside v1.

## Privacy, budgets, and retention

No configuration values, secret values/handles, source contents, raw process
streams, absolute workspace roots, or provider-authored free text are stored.
The allowlisted identities and relative locators can still be private metadata.
Digests are not encryption, and caller-supplied identifiers are not generically
redacted. Unix creates workspace directories with mode 0700 and files with 0600;
Windows inherits the selected parent's ACL.

V1 limits a plan to 256 steps, each step to 4096 artifact records, a snapshot to
8 MiB, a run to 4096 snapshots, and retained snapshot bytes to 64 MiB. Exceeding
a budget is a refusal, never silent pruning. Large artifact directory manifests
may exceed the state budget.

Keep the entire history until explicitly archived or removed. Stop writers and
resolve uncertain provider effects first. Archive the whole workspace, not a
single middle snapshot. Cleanup of state never implies deletion of source or
output artifacts. No automatic retention task or garbage collector is installed.

## Migration and rollback

Unknown versions are refused. Breaking interpretation changes use new major
identifiers; v1 has no in-place migration or downgrade. A future explicit migrator
must preserve the original quiescent workspace, record its exact identity, produce
a separate destination, and revalidate compatibility and authority. Rollback uses
a preserved compatible workspace only after an operator resolves effects that
occurred since it was saved. Restoring a backup never authorizes repeated work.
The rationale and review triggers live in
[ADR-0011](../architecture/governance/decisions/ADR-0011-durable-run-state.md).

## Verification

```console
cargo test --lib state::store::tests --locked
cargo test --test durable_state --locked
cargo test --test hermetic_provider_kit durable_execution --locked
cargo test --test hermetic_provider_kit graph_recovery --locked
python3 tools/validate_contracts.py
```

The portable store suite covers restart, abrupt process exit, concurrent opens,
schema/corruption refusal, partial writes, and immutable history. Unit fault
injection covers failures after pending-file creation, after file sync, and after
rename. Real provider tests exercise the durable success/failure path, interrupted
acceptance, stale evidence, explicit recovery, and privacy. Eight graph tests
cover deterministic fresh-root/reopen reports, transitive and branch invalidation,
all local identity boundaries, complete inventories, stale-report non-authority,
the single-step bypass, blocked future inputs, and preserved history. The
acceptance driver includes these test sources in its evidence identity and runs
them with `--all-targets`, along with the 49-row [lifecycle receipt corpus](lifecycle-scenarios.md)
from #65/#66. The latter runs every recipe twice, includes fresh-process recovery,
and checks bounded portable reports.
macOS/Windows CI runs the portable store suite; Linux runs the full provider matrix.
