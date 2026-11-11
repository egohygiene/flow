# Native candidates and release matrix

Flow now has source adapters for Aniflow `music-video-source-v1` and Renderflow
`canonical-build-v1`. Both remain unqualified candidates until independent build,
installation, publication and native acceptance gates are completed. No provider
is implicitly built, installed, downloaded or selected from a sibling checkout.
The released Optiflow v0.1.1 path keeps its original read-only scope.

## Read-only visibility

```text
flow doctor --format json
flow release-matrix --cohort /private/cohort.json --format human
flow doctor --cohort /private/cohort.json --installations /private/installed.json --format json
```

Doctor v2 exposes implemented candidate profiles separately from the legacy
released-provider check. Matrix and doctor never launch providers or install
dependencies. `flow.installed-cohort/v1` supplies `installations`: each row contains
`row_id`, encoded native `executable` and `contract_files` entries with `identifier`
and encoded native `path`. Only explicitly selected files are read. Missing or
mismatched selected files are failed cells, not hidden rows or a successful setup.
Current installed hashes can pass installation/contract cells only. Publication,
runtime preparation and positive native qualification need independently verified
host evidence through the cohort API; this read-only CLI never manufactures those
facts. A non-green matrix uses partial exit 9 and retains every declared row.

## Explicit candidate operation

Create a private `flow.native-request/v1` selecting an exact cohort/hash/row,
independently built executable, compiled descriptor, inputs/configuration, fresh
run output, explicit tool search directories and limits. All native paths use
`{"encoding":"utf8","value":"/absolute/path"}` or the existing Unix byte codec.
See the [inert request template](native-request.example.json) and
[native adapter requirements](../integrations/native-adapters.md). Replace template
paths and digests with observed values; templates are not execution evidence.

```text
flow plan --request /private/native-request.json --output /private/native-plan.json --format json
flow approve --plan /private/native-plan.json --approved-by 'operator declaration' --acknowledge-trusted-unconfined --acknowledge-unqualified-candidate --output /private/native-approval.json --format json
flow run --plan /private/native-plan.json --approval /private/native-approval.json --format json
flow status --run-directory /private/new-run --format json
flow resume --run-directory /private/new-run --native-run-directory /private/new-run/native/EXACT-RETAINED-RUN --format json
```

Preparation and approval do not launch the provider. Run and explicit native resume
are the launch operations. Both check the exact plan/selection/configuration and
retain attempts. Acknowledgement permits the planned unconfined candidate process;
it does not grant sandbox/network isolation, native qualification, creative review,
publishing, provenance, original replacement or disposal. The provider retains
domain algorithms/acceptance and checks its exact original intent or forest on
resume. Status only reobserves retained native evidence and current file hashes.
No native run locator is guessed. Renderflow graph resume names the exact
`RUN/native/output` root; frozen native package recovery is a different contract.

For ordinary file selection, the additive path is:

```text
flow plan --input /selected/video.mkv --profile aniflow-temporal --native-selection /private/native-request.json --output /private/native-plan.json --format json
```

The selected file must match the native request exactly. An ordered collection
uses `flow.local-input-native-request/v1` with `request` and `selection` fields;
both must declare the same ordered paths and observed bytes. Directory widening
and mixed native/provider/output overrides are refused. Default local preview
bounds stay smaller than direct native execution-input bounds.

These are source examples, not executed receipts. No tests, builds, validators,
native/media/model/provider/CLI/SDK/CI or publication ran during authoring. Native
release/installation/qualification work stays open under provider owners and
Flow #51/#52/#53/#70/#71/#72. Flow #88 stays last and unchanged, with the known
alignment cancellation/start-marker failure unresolved and its cause unproven.
