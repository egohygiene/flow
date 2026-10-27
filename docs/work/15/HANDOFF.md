# Flow #15 — observability contract checkpoint

Base: `69952725fb4eeb1daeb0e5c8bea7d8d82ba40670`.
Branch: `codex/15-observability-contract`.
Date: October 26, 2026. Issue: [#15](https://github.com/egohygiene/flow/issues/15).

The maintainer authorized continued suite work with frequent draft PR saves,
clean UX/DX and the existing execution/polish deferral. This checkpoint defines
the next contract from the live #11 ledger; it does not add instrumentation,
exporters, a shared package or a new CLI.

## Authored scope

- [Profile](../../observability/README.md): signal/evidence boundaries, library
  ownership, mode/config matrices, privacy controls and bounded sink behavior.
- [Registry](../../observability/registry.md): event/span/metric/field names,
  classifications, finite cardinality/budgets and one JSONL log projection.
- [Inventory](../../observability/inventory.md): exact four-repository source
  revisions, current gaps and migration ownership.
- [OTel mapping](../../observability/otel-mapping.md): immutable core/spec pins
  and separately pinned Development GenAI conventions; payload capture excluded.
- [Conformance plan](../../observability/conformance.md): future reusable cases
  and measurements, all unexecuted.
- Proposed ADR-0014 plus canonical architecture and continuation links. No
  earlier ADR is ratified, superseded or rewritten.

## Important findings to preserve

Aniflow machine errors currently use stderr; its human progress uses stdout.
Optiflow has an explicit CLI-only early machine-mode selector and distinguishes
full effective-config identity from evidence-policy identity. Renderflow's
global `.init()` bootstrap and command-specific output require deliberate pilot
migration. Flow's provider JSONL protocol is not a product command-result mode.
Domain report timestamps/evidence digests and real authority permissions remain
legitimate even though new optional telemetry state must be kept out of semantic
plan/cache identity. None of these current gaps is silently marked conforming.

## Verification and next work

Source/document review and exact Git publication identity are the evidence.
No product tests, compiler/build, lint/format, schema validators, CLI/native/
provider execution, dependency resolution/install, exporter or CI ran. No Actions
were dispatched or polled. Documentation automation and the conformance plan
remain unrun. Review feedback must remain separate from runtime evidence.

Keep this PR draft and #15 open pending the contract review. The proposed ADR
requires its own explicit disposition; merge is not acceptance. After profile
review, #16 owns separate repository-local pilots in Renderflow and Aniflow,
their precise implementation issues and runtime evidence. Do not select a
shared crate or enable export while implementing this contract alone.

The wider suite continues through the live #11 ledger. The first installed
candidate still needs qualified immutable providers, their Flow adapters and
the product CLI; optional AI/creative scope does not silently block that path.
