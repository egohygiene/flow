---
schema: aether.architecture-decision/v1
id: adr-0011
title: Persist prepared execution intent and acceptance in immutable local snapshots
kind: architecture-decision
status: proposed
accepted: null
owners:
  - egohygiene
scope:
  - flow
governed_by:
  - architecture-decisions
supersedes: []
superseded_by: []
related:
  - flow-architecture
  - flow-roadmap
  - adr-0007
  - adr-0009
  - adr-0010
---

# ADR-0011 — Persist prepared execution intent and acceptance

## Context and authority

Flow #49 requires durable plans, execution intent, checkpoints, authority,
validation, and recovery decisions before #31 can prove lifecycle behavior.
The existing library gates produce opaque tokens but lose their history when
the host exits. A file left by a provider cannot resolve that uncertainty.
The maintainer authorized implementation on 2026-09-25; this decision is proposed
for review with the implementation PR and has no claimed acceptance date yet.

## Decision

Use closed v1 records and an explicit local workspace. Persist immutable prepared
plans containing ordered dependencies and exact context digests. Record launch
authority and a running attempt before entering the existing process runner.
Record success only after transcript validation and fresh artifact acceptance.

Commit numbered immutable JSON snapshots by writing and syncing a pending file,
then renaming it within the same directory. Link each state to its predecessor's
digest. Hold an OS-backed exclusive lock for the store's lifetime, including
provider execution. Never delete the lock file or guess that another writer died
from a PID or elapsed time. A process exit releases the OS lock.

Reopen checks the complete retained history and every state transition. A pending
file, missing middle snapshot, unsupported version, malformed record, or identity
mismatch is a typed refusal. No automatic truncation, repair, or fallback occurs.
An error after rename can mean the new snapshot committed; the old handle becomes
unusable and the caller must reopen to learn the durable result.

An interrupted running record remains uncertain. Matching current identities and
fresh observations support eligibility assessment; they do not recreate opaque
authorization or acceptance tokens. Retry requires an explicit recorded decision,
fresh matching authority, and acknowledgement of uncertain external effects. It
returns the step to pending without launching it. Successful work cannot be
re-executed through the same step. Whole-graph scheduling, downstream invalidation,
provider-native checkpoint restoration, and automatic retry policy remain later work.

## Rationale and alternatives

Immutable snapshots keep each reviewed state and make interrupted writes visible.
They avoid overwriting an open destination, which simplifies portable rename
behavior. A single overwritten JSON file was considered but would retain less
diagnostic history. SQLite transactions would also provide a sound persistence
boundary, but would add a database engine and migration surface to this bounded
local document store. The initial implementation uses the small `fs2` dependency
for advisory locks on Unix and Windows.

## Assumptions and trade-offs

- The caller controls a local filesystem with working advisory locks and atomic
  same-directory rename. Network filesystems and hostile concurrent writers are
  outside the v1 guarantee.
- Unix syncs both files and directories; Windows flushes files and claims process-
  crash consistency, not power-loss durability. Hardware durability is not proven.
- Full-history validation and repeated snapshots favor inspectability over scale.
  Step, record, snapshot-count, and total-history budgets bound the initial store.
- Digests detect accidental corruption and mismatched context. They do not
  authenticate the operator, attest a running binary, detect an intentionally
  removed tail without an external anchor, or contain an unconfined provider.
- A validator-source and dependency-recipe digest makes source changes visible.
  It is not a claim about every downstream build flag or dynamically linked byte.
- Configuration values, secrets, source bytes, raw streams, and provider messages
  are omitted. Identifiers, relative locators, and content digests remain sensitive
  local metadata; hashing is not encryption.

## Migration, rollback, and retention

Breaking schema or interpretation changes require a new major identifier. Never
reinterpret unknown fields or downgrade records in place. A future explicit
migrator must preserve the original quiescent workspace, write a separate target,
record the source identity and migration decision, and revalidate compatibility
and authority. No migrator is shipped in v1.

Rollback restores a preserved compatible workspace only after an operator resolves
work performed after that snapshot. Restoring older state never grants permission
to repeat effects. Incompatible binaries refuse the newer state. Retain all
snapshots until the operator archives or deletes the entire quiescent workspace;
there is no automatic pruning, repair, artifact deletion, or secret retention.

## Validation and review triggers

Validate real process success/failure, fresh reopen, stale identities and bytes,
cancellation, interrupted acceptance, explicit retry, and preserved sources.
Exercise partial writes and failures before/after rename, abrupt process exit,
cross-process lock contention, corruption, schema refusal, and privacy canaries.
Run portable store tests on Linux, macOS, and Windows. Revisit the decision for
distributed writers, automatic recovery, authenticated state, a new durable
backend, schema migration, or stronger filesystem/process containment.

## Related artifacts

Flow #11, #30, #49, and #31; `flow-architecture`;
[`durable-state.md`](../../../integrations/durable-state.md); and the
`flow.run-*/v1` contract family.
