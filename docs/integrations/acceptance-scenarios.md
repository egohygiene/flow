# Acceptance scenario matrix

## Design and scope

Flow #30 proves one provider execution boundary using the closed scenario
contract and the finalized hermetic provider kit. Test recipes own fault
injection; production code continues to own resolution, process preflight,
transcript validation, filesystem observation, and final artifact acceptance.
No new scheduler, provider algorithm, or recovery state is introduced.

The versioned, test-owned catalog in
`tests/fixtures/acceptance-scenarios.v1.json` assigns stable scenario IDs,
deterministic recipes, expected typed outcomes, PR-tier budgets, and explicit
coverage gaps. Its executor lives under `tests/scenario_matrix/` and reuses the
provider kit's exact package finalization and execution helpers. Catalog rows
are conformance recipes, not public runtime plans or authorization.

Each row must produce exactly one normalized receipt from an actual public API
outcome. An unexpected error, missing row, duplicate row, or expected-outcome
mismatch fails the suite. Raw error strings and provider-authored free text
are excluded from receipts. Recovery instructions name the failed boundary.
The harness compares receipts across fresh roots and checks canaries before
printing portable JSON evidence.

## Boundaries under test

- Resolution: compatibility, missing or unavailable observations, stale and
  future versions, exact pins, deterministic conflict, and pre-invocation
  fallback policy.
- Contract and preflight: schema rejection, identity, configuration,
  authorization, executable integrity, and malformed provider evidence.
- Artifact observation and acceptance: complete, partial, missing, extra,
  changed, contradictory, corrupt, escaped, linked, and incorrectly typed
  outputs; exit zero cannot authorize promotion.
- Evidence: distinct empty, unavailable, incomplete, unsupported, invalid,
  and failed states; bounded operational evidence and portable privacy.

## Running and interpreting the matrix

Run after populating Cargo's locked dependency cache:

```console
python3 tools/run_acceptance_scenarios.py
python3 tools/run_acceptance_scenarios.py --all-targets
```

The driver uses `cargo test --locked --offline` and writes
`target/acceptance-scenarios.v1.report.json`. It removes an older report before
starting, fails on any failed Rust test or missing/duplicate/unexpected receipt,
and records the catalog digest, tested source-file digests, Cargo version,
coverage counts, residual gaps, and actual normalized receipts. Failure logs
stay local; CI uploads only the successful portable report. The source digest
is checked before and after execution so an edited source tree cannot be
mistaken for the tested one.

The PR catalog budgets 30 seconds per execution, two executions per case,
16 KiB per receipt, four immediate output entries, and 1 MiB of generated
regular-file output. The fixed recipes create only files and empty directories;
this output-count budget is not a general recursive filesystem quota. The
driver has a 600-second Cargo wait timeout and a 4 MiB post-capture test-log
acceptance limit; it is not a general descendant-process supervisor. Provider
stdout/stderr and process deadlines remain enforced by the existing locked kit
limits. Budget measurements exclude compilation from the per-case time, but
include compilation in the driver timeout. Memory, whole-filesystem limits,
and kernel network isolation remain explicit gaps.

The test profile omits debug symbols because the exact provider executable is
copied and hashed repeatedly. This keeps generated package size and PR time
bounded without bypassing any package or executable digest check. Each receipt
still names the actual build-specific bytes. The `argv-environment` recipe also
uses a tiny first-party shell probe; its exact observed subjects are retained
in that receipt's `probe_subjects` field.

`observed-empty` means Flow actually observed an empty directory; its receipt
does not claim an accepted provider execution. Resolution success similarly
does not claim execution or artifact acceptance. Only receipts with
`accepted: true` crossed `accept_artifacts` successfully. Transcript mutations
start from actual kit execution and re-enter the public host-neutral transcript
validator; the receipt identifies the validator outcome, not a second launch.

## Claim limits

The kit is synthetic. Real released providers, native format validators,
cryptographic publisher authentication, OS sandboxing, atomic filesystem
snapshots, descriptor-bound launch, and descendant containment are not proven.
Flow #49 owns durable state; #31 owns retry and resume. The scenario catalog
does not expand those checkpoints or claim that two synthetic fixtures are
two real provider adapters.

Operational error objects and rejected provider evidence remain host-local
diagnostics and may contain private values. Only the explicitly allowlisted
normalized receipts are portable. Privacy canaries do not establish a generic
redactor for arbitrary provider-authored messages.
