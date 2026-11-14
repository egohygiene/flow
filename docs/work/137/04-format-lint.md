# Flow #137 checkpoint 4 — flow, November 13, 2026

The 39 `assert_is_empty` findings in the retained Rust/Clippy 1.99 run
are rewritten using its typed `assert_eq!` / `assert_ne!` suggestions, preserving
Rust 1.85 compatibility and displaying the offending value. This covers the
recorded vector assertions across ten files without adding a lint allowance.
A pre-existing semicolon ended the generated-member delivery match arm as a
statement. Rustfmt refused to parse that module; the arm now ends with a comma.
The formatter subsequently parses the delivery module successfully. That is not
Rust type-checking or a completed publishing workload.

## Reproducible formatting and lint policy

`task format` applies Cargo/Rustfmt 1.85.0; `task format:check` uses that exact
formatter. The root manifest's MSRV is unchanged. Flow's MSRV matrix row owns
the canonical formatting gate; its moving-stable row retains strict Clippy and
product checks. Aniflow/Optiflow install the pinned formatter without changing
their selected compiler lane. Renderflow uses the existing 1.94 MSRV at the
explicit 1.94.0 patch in its owning workflows. No product lane is made advisory
or given `continue-on-error`, and no new lint allowance is introduced.

The maintainer selected the generated-source and formatting repair checkpoints.
Canonical source formatting was applied and the corresponding explicit-version
`cargo fmt --all -- --check` commands exited 0. The adjacent JSON records exact
formatter identities and scope; Optiflow includes its separate fuzz workspace.
The formatter parses source but does not perform Rust name/type/borrow checking,
run build scripts, execute tests or qualify providers. Source-formatting edits
are extensive because many prior source checkpoints had deferred formatting;
review the logical repairs separately from line wrapping, indentation and field
shorthand. No unrelated repository or Identity workspace was formatted.

Clippy, product compilation/builds/tests, schema/corpus/native checks and hosted
CI remain deferred. The existing `-D warnings` policy stays enforced for the
later authorized lint pass. Correct additional findings at their owner; do not
hide failures, raise the MSRV to silence a lint, regenerate fixtures blindly or
report a skipped/unavailable lane as qualified. #88 remains unchanged and last.

This branch is a source candidate stacked after checkpoint 3. Formatting success
and historical-diagnostic source fixes do not establish a green Actions suite.
