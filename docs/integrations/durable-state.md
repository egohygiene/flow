# Durable prepared execution

Flow #49 adds a library API for persistent execution of fully prepared process
steps. It builds on the exact subject, authority, transcript, and artifact gates.
There is no product CLI or graph scheduler in this checkpoint. #31 owns the
broader lifecycle scenario matrix; #53 owns the supported CLI.

## Public entry points

1. Construct a `ProcessStepContext` with the current resolution, invocation,
   subject lock, opaque authorization, artifact bindings, and explicit execution
   and artifact roots. Roots and tokens remain caller-local.
2. Call `PlannedStep::prepare`, then assemble a `RunPlan` with stable `plan:`,
   `run:`, and `step:` identities and sorted dependencies on preceding steps.
   Plans contain fully pinned expected inputs; unresolved future outputs require
   later planning support.
3. Call `RunStore::create` with a new workspace path. Its parent must exist.
4. Call `execute` for a ready step with explicit secrets, cancellation, and event
   adapters. The store persists intent before launch, then an accepted checkpoint
   or a bounded typed failure. It holds its exclusive workspace lock throughout.
5. Drop the handle and use `RunStore::open` after restart. `state()` exposes the
   recorded status without writing files or launching a process.
6. Use `assess` with the complete expected plan and fresh runtime context.
   `Reusable` requires matching intent, validator identity, and fresh input/output
   observations. It is eligibility evidence, not a reconstructed acceptance token.

`RunState::new` remains an in-memory model. `Orchestrator` and `LocalProcessRunner`
remain useful low-level seams for unit tests and controlled integrations. Callers
requiring restart semantics must use `RunStore::execute` rather than treating a
low-level runner result as a persisted run.

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
through this API. Automatic reuse scheduling, targeted downstream invalidation,
provider-native checkpoint restoration, and exactly-once external effects are
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
python3 tools/validate_contracts.py
```

The portable store suite covers restart, abrupt process exit, concurrent opens,
schema/corruption refusal, partial writes, and immutable history. Unit fault
injection covers failures after pending-file creation, after file sync, and after
rename. Real provider tests exercise the durable success/failure path, interrupted
acceptance, stale evidence, explicit recovery, and privacy. macOS/Windows CI runs
the portable store suite; Linux also runs the full provider matrix.
