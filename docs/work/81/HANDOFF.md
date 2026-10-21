# Flow #81 — Optiflow receipt contract correctness

Date: 2026-10-21. Issue: [#81](https://github.com/egohygiene/flow/issues/81).
Base: `c673654fb7ad86569d3bb32f1412f5eb2fbf2bf3`.
The containing PR identifies the candidate commit; this file does not predict it.

## Failure and root cause

The loader could accept a completed receipt with empty command artifact lists
and a null plan digest. An absent digest compared equal to absent artifact proof,
and the empty verification loop succeeded. Typed decoding also accepted omitted
required nullable keys, unknown native-path fields and other schema violations.
The old loader did not correlate recorded identities with retained native JSON.

## Fix

`src/optiflow.rs` owns one pure receipt validation gate, used before persistence
and after decoding. Required nullable keys must be present, nested paths remain
closed, and constants, capability membership, diagnostics, digests and state
transitions are checked. Complete/partial records require the expected unique
artifact roles and a present matching plan digest and identity.

Reopening retains bounded regular-file, canonical confinement, size, SHA-256 and
BLAKE3 checks. The same observed bytes bind native schema markers, source roots,
run/source-set/plan IDs and committed artifact-set membership to the receipt.
Source existence is not required to reopen historical local evidence. Paths used
as argv tokens retain their original behavior. Refusal after any valid command
prefix, including all three completed commands, remains possible.

No provider protocol, receipt schema version, runtime dependency or mutation
authority changes. These editable local records remain evidence, not signatures.
The architecture owner and integration guide describe the stronger boundary.
This repairs an existing contract; no new architecture decision is introduced.

## Regression protection

The shared corpus has **84 cases**: 12 accepted controls, 41 schema-invalid
cases, and 31 further runtime-invalid cases. It covers all six statuses, refusal
prefixes of length zero through three, the five audit counterexamples, all nine
required nullable keys, native identity/member correlation, tampering and paths.
The persistence unit test checks rejection before creating or replacing a file.

The Rust harness materializes consistent synthetic native artifacts in disposable
directories. The independent Python Draft 2020-12 validator checks schema outcomes
against the same mutations. The repository-validator CI job owns the Python
check explicitly; this native receipt is outside the existing 25-contract set.
`jsonschema` is a pinned CI/development dependency, not a Rust runtime dependency.

Baseline replay at the base above, with only the new harness/fixture files added,
incorrectly accepted **45 cases**, including all five audit counterexamples and
all nine omitted nullable keys; it rejected no positive control. Current focused
checks pass. The [local validation receipt](../../validation/flow-81-local.json)
records commands, source/log hashes and the expected failing baseline replay.

## Validation and residual risk

Local evidence is Linux x86-64, Rust 1.85.0. The three existing adapter tests ran
with the independently pinned Optiflow v0.1.1 executable explicitly installed;
two invoked that executable and one exercised missing/incompatible-provider
refusal. Inputs were disposable synthetic text, including Unicode and spaces.
This is actual native compatibility evidence, separate from synthetic parity.
The corpus is Unix-gated to the native adapter's supported platform family;
macOS execution and Windows native support are not qualified here.

Do not infer a green full suite or release from these focused checks. The prior
600-second all-target wrapper timeout and moving-stable Clippy failures in
unchanged code remain owned by [#34](https://github.com/egohygiene/flow/issues/34).
No timeout was raised, lint suppressed, hosted CI polled or release published.
Aniflow/Renderflow validation deferrals remain in force.

Next checkpoint: [#82](https://github.com/egohygiene/flow/issues/82), reconcile
current implementation and continuation documentation. Re-query live state
before starting; do not fold that broader reconciliation into this repair.
