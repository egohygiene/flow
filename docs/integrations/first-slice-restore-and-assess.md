# First slice: restore and assess

## Outcome

Given an immutable source collection containing one supported video, Flow
produces a validated new temporal master and a read-only relationship assessment
of source plus output. The slice proves real two-provider composition, durable
state, and recovery without claiming optimization mutation or Renderflow video
packaging.

## Stage plan

1. **Discover:** Flow probes named Optiflow and Aniflow releases and records
   compatibility decisions.
2. **Inventory source:** Optiflow scans read-only inputs and emits collection
   evidence through its adapter.
3. **Plan:** Flow resolves immutable inputs, capabilities, configuration,
   expected artifacts, and a stable plan digest before processing.
4. **Create master:** Aniflow processes into an isolated run workspace, validates
   temporal invariants, and emits a new master plus domain evidence.
5. **Assess:** Optiflow scans the original collection and new master, reporting
   identities, relationships, duplicates, and opportunities without mutation.
6. **Finalize:** Flow verifies declared outputs, writes atomic final state, and
   emits provenance, validation, diagnostics, and resume guidance.

## Failure and resume proof

The acceptance fixture induces a failure after the Aniflow stage and before the
second Optiflow scan. Resume must verify the source digest, plan/configuration,
provider and contract versions, checkpoint metadata, and master bytes. If all
remain compatible, Aniflow is not repeated. Changing any one compatibility input
invalidates reuse with an actionable reason.

## Safety and exclusions

- Original bytes are never modified.
- Optiflow remains read-only; this slice does not apply optimization plans.
- Renderflow is not invoked.
- Flow does not run private provider modules or copied source.
- Remote transfer, destructive mutation, signing, and arbitrary shell execution
  are out of scope.

## Acceptance evidence

- one redistribution-safe fixture and pinned provider releases;
- deterministic plan and artifact identities across repeated runs;
- provider outputs translated into the versioned Flow contracts;
- successful induced-failure resume and explicit invalidation cases; and
- agreement among run state, logs, compatibility decisions, provenance,
  validation, and the final assessment.
