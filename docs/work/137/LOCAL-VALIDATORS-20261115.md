# Flow local repository-validation checkpoint — November 15, 2026

This checkpoint follows the unchanged required repository-validator job in
`.github/workflows/ci.yml` and the canonical Rust 1.85 formatter. It adds their
exact local selections and Task aliases. It changes no Rust source, dependency,
lock, MSRV, hosted condition, lint severity, validator criterion or expected
scenario byte. The eight original manifest rows remain unchanged. Flow #137
remains open; Flow #88's final acceptance stays unchanged and last.

## Measured starting point

At clean merged Flow `80127ca4f9ad14ebb8e8c5a8f7321f242aa36607`, the canonical
`cargo +1.85.0 fmt --all -- --check` passes. All nine existing repository-validator
commands pass with the exact pinned PyYAML 6.0.3 and jsonschema 4.26.0 dependencies.
These checks read current schemas/fixtures and exercise their existing negative
report/resource-boundary cases. The lifecycle suite launches bounded synthetic
Python subprocesses; it does not execute Rust scenarios, real media or providers.

Actual strict Clippy was also attempted at that source:

```sh
cargo +1.85.0 clippy --all-targets --all-features --locked --offline --jobs 2 -- -D warnings
```

It **fails with exit 101**, recording 305 library findings and 276 library-test
findings, largely overlapping. The initial command completed in 18.990081 seconds;
its raw log is pinned by SHA-256
`769261083e553f0d90695a8d97406acc667ecdbefe06b03cd37970a2548ad5d4`.
Missing error documentation, similar match arms, wildcard imports, long functions
and public report boolean shapes dominate the recorded debt. Examples include
`src/ai_preparation/mod.rs:114`, `src/static_project.rs:455`, and
`src/static_project.rs:869`. These are freshly observed current-source findings,
not a fresh reproduction of every historical moving-stable failure.

The Clippy 1.85 component was explicitly provisioned in task-private Rustup/Cargo
homes. Actual tools are Rust 1.85.0 / Cargo 1.85.0 / Clippy 0.1.85. Two build jobs,
disabled incremental compilation and omitted development/test debug information
bound compiler artifacts. No public API refactor, lint allowance, policy waiver,
MSRV change or `continue-on-error` was used to hide the result. `task ci:clippy`
preserves the exact failing strict command for later repair. The passing local
lane below does not qualify strict Clippy or the Rust runtime/moving-stable matrix.

## Selected lane and existing owner

The full `config/local-ci.json` now has fourteen required checks: the original
four plus canonical formatting and the nine repository validators. The new
`ci:validators` Task runs exactly those nine scripts; `format:check` remains its
existing owner. `ci:validators:test` checks that the local manifest and Task agree
with the unchanged hosted script inventory, required status, 120-second bounds,
canonical formatter and strict hosted Clippy severity. Its negative controls
reject removed/optional/skipped/duplicate/unbounded checks, command substitution,
generated-source rewriting, Task drift and relaxed hosted lint severity.

The reviewed Realm owner remains
`75c1b2e1475b972a0ce1c8594e8daba2e1cf8686`. It has an explicit `--manifest`
argument, but no check-ID filter. The selected run therefore uses a private
ten-row manifest under the ignored `.cache/realm/local-ci/` directory. Its rows
are exactly canonical formatting plus the nine required validator rows from the
committed full manifest, with no missing, duplicate, extra, optional or skipped
row. Full-manifest and derived-selection SHA-256 values, IDs and the owner's
actual report are bound in the portable observation added after execution.

For a similarly bounded run, derive that exact selection from the reviewed
manifest and invoke the existing owner explicitly:

```sh
python3 -B /absolute/reviewed/realm/tools/realm_local_ci.py run \
  --repository /absolute/flow \
  --manifest .cache/realm/local-ci/validator-selection-20261115.json \
  --output /absolute/private/fresh-validator-report
```

This runs no act job or Docker container. That avoids another measured ~3.38 GB
Docker VFS rootfs copy on the low-headroom workspace. The original act row and
all other required full-manifest checks remain present and unchanged. A derived
report's `complete: true` applies only to its named ten checks; this checkpoint
does not claim the full fourteen-check selection, hosted Actions, product test
execution, providers, other native platforms, releases or publication are green.

The initial unchanged-main observations are separate from the later
manifest/Task/test/documentation source and its post-freeze Realm observation.
No earlier receipt is relabeled as having tested these later changes. Actual
exact-source results and immutable portable receipts follow below.

## Post-freeze selected results

The full manifest plan and the selected Realm run both pass at clean source
`9c312081404345b2bcd82aa1bc0762fd3e4877bb`, tree
`f25368d896b6f8ec3dfcf0da3f7fd3907847b2ea`. Source stayed clean with the same
revision before and after the run. All ten selected required checks report
`passed`, exit 0, no timeout, no truncation and no unavailable prerequisite.
The selected Realm invocation took 1.920492 seconds. It invoked no act/Docker job
and its preflight is null. The original full selection was planned, not executed.

The unchanged validators report two Optiflow-receipt cases, five acceptance-report
cases, eleven lifecycle/report/resource cases and six durable-contract cases.
The remaining commands validate contracts, exact generated bytes, 23 architecture
specifications, 29 skills and nine agent definitions; these inventory counts are
observations of this source, not permanently fixed limits. Four separate
configuration-observer methods pass, including 73 negative mutations, at the same
clean source. The compatible Go Task client is absent, so native Task execution
is not claimed; its parsed commands exactly match the selected existing owners.

The exact portable
[observation](evidence/20261115/local-repository-validators.json) has SHA-256
`66e11cc010bb05cf9a6ed42907f6872dafc87bd2f1901fe50ddcea6213d5b61e`.
It binds full/derived manifest hashes and selected row encodings, all ten actual
log hashes, pinned Realm source/script/raw-report identities, declared tools,
Python requirements, configuration observers, unchanged lock and the separate
failed Clippy baseline. The raw report and execution ledger remain private
because they contain machine paths. This subsequent amendment adds only the
portable observation and this account; it does not relabel earlier results as
having tested later documentation or claim a full CI/release qualification.
