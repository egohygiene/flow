# Local support metadata

Relates to #21. The public `diagnostics` owner previews, creates and inspects a
bounded JSON support container. It is support material, not provenance, a run
receipt, release qualification or a success signal. No data is uploaded.

The #53 delivery checkpoint adds these explicit commands and matching task
aliases. They use the same SDK and single versioned command-result envelope:

```bash
flow diagnostics-preview --request /private/support-request.json --format json
flow diagnostics-bundle --request /private/support-request.json --output /private/new-support.json --format json
flow diagnostics-inspect --bundle /private/new-support.json --format json
flow doctor-observability --format json
task diagnostics:preview REQUEST=/private/support-request.json
task diagnostics:bundle REQUEST=/private/support-request.json OUTPUT=/private/new-support.json
task diagnostics:inspect BUNDLE=/private/new-support.json
task observability:doctor
```

`doctor-observability` reports unimplemented Flow-owned instrumentation/exporters
and unknown collector/queue/flush health as partial. It does not read environment
configuration or connect to a collector. No telemetry is enabled by these commands.
Two additional parser/health regressions are authored, uncompiled/unrun.

The closed `flow.diagnostics-request/v1` request explicitly selects one to three
unique categories: `build`, `platform`, `contracts`. Build uses the Cargo product
version; platform records compiled OS/architecture, not observed physical
hardware. Contracts are named source interfaces, not released compatibility.
No source content, environment, credentials, hostname, path, complete argv,
raw operational logs or run artifacts can be selected. Broader #21 telemetry
categories/health/retention and cross-tool conformance remain unfinished.

Preview returns categories, exact generated size/count and exclusions without
writing. Explicit creation writes a fresh file with no-clobber/private-file
semantics and independently inspects it. A failed write preserves the partial
file; choose a new destination rather than adopting it. The container is at most
64 KiB, with at most three static relative member names and 4 KiB per typed
payload. Member checksums describe canonical payload bytes. Inspection parses
without extraction, execution or network and reports categories/counts/digest
without echoing untrusted payload text. Supplied metadata claims are unauthenticated.

Keep bundles private. Select an existing private directory, inspect the exact
bundle, review its contents locally and decide independently whether to share
it. No email, upload or issue attachment is automatic. Retain or remove only
the specific support file under operator control; there is no automated cleanup.
This owner reads no logging directory and enables no telemetry/exporter. The
proposed suite profile and ADR-0014 remain unratified by source inclusion.

Three meaningful source cases are authored, uncompiled/unrun. No tests/builds,
dependencies, validators, product CLI/SDK, native/provider/media or CI ran.
Issue #21 retains its broader operational and independently built CLI acceptance;
Flow #88 remains last and untouched.
