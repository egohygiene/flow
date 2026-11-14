# Mac native trial kit observation — 2026-11-14

Relates to Flow #71/#72 and Renderflow #419. The
[operator walkthrough](../../MACOS-NATIVE-TRIAL.md) and one
`task macos:trial:preflight REQUEST=… OUTPUT=…` entry point are concrete source
preparation for a later Mac trial. **No actual Mac execution occurred here.**

The executed observer source is
`8c7c1fccd0a6e97eebc8dad66728d61a5b8e26ce` / tree
`e58cdd4313b530aadb220a2990fa2340803c012d`, clean at capture. The following
evidence-only commit retains portable reports without changing that observer.
Its [observation seal](macos-native-trial-20261114/observations.json) binds the
driver/test SHA-256, exact test command, six captured command outcomes and a
closed 17-file private inventory. The inventory excludes its own seal to avoid a
circular digest. Raw input locks and captures remain private; portable copies
contain only allowlisted pins, states, fixed reasons and byte counts.

Executed checks:

- **17 focused fixture tests passed** for read-only planning, Linux refusal,
  unsupported/unknown host metadata, exact source HEAD/tree/lock/cleanliness,
  missing reviewed candidate/unsupported declared target, native Mach-O CPU and
  executable container, exact cohort/tool/contract membership, changed pins,
  pre-read byte limits, single-link/symlink rejection and private redacted output.
- Python AST and Task YAML/explicit-variable checks passed. The Task executable
  is unavailable on this host, so Task runtime execution is not claimed. The
  equivalent direct Python commands were actually executed.
- `git diff --check` passed at the recorded clean observer source.
- The real [Apple-first source plan](macos-native-trial-20261114/source-plan.json)
  exited 0 without invoking any program. The real Linux
  [Apple preflight](macos-native-trial-20261114/linux-apple-preflight.json) and
  [Intel preflight](macos-native-trial-20261114/linux-intel-preflight.json) both
  correctly exited 2: native matching Mac unavailable and Aniflow reviewed
  successor absent. Exact starting Flow/Renderflow source checks passed. The
  selected Aniflow source passed HEAD/tree/clean/lock and declared-target checks
  before its missing-candidate refusal; its full source prerequisite stays blocked.

The three selected starting revisions/trees/lock hashes are in each report's
`source_locks`. They identify the inspected initial sprint source, not a future
integrated candidate. All three exact sources already declare both Mac packaging
targets; Apple Silicon is first, and Intel retains its own native evidence gate.
Operator-selected successor source must replace its full lock together. These
local Mac candidates are separate from the existing immutable Optiflow 0.1.1
read-only release; no Linux artifact is relabeled for a Mac trial.

The driver only observes prerequisites and explicitly selected local bytes.
Builds, product invocations, installation, acquisition, services, credentials,
models, signing, publication, CI dispatch and removable-volume operations are
all **unexecuted**. Real native owner builds, Mach-O/contract readback, installer
management, domain/workload acceptance and human review still require explicit
operator execution on each matching Mac. Fixture/header checks cannot establish
those facts. Existing release/tag/candidate gates remain intact, and Flow #88's
original final-pass ledger remains unchanged and last.
