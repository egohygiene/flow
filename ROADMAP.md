---
schema: aether.architecture-document/v1
id: flow-roadmap
title: Flow Roadmap
kind: architecture-document
version: 0.6.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-09-20
governed_by:
  - architecture-roadmap
depends_on:
  - flow-vision
  - flow-architecture
related:
  - flow-pillars
  - flow-system
  - flow-methodology
supersedes: []
---

# Flow Roadmap

<!-- BEGIN ROADMAP EXECUTION SNAPSHOT -->
<!-- roadmap-manifest
schema: hygiene.roadmap/v1alpha1
repository: egohygiene/flow
visibility: public
publication: central
route: /roadmap/flow/
updated: 2026-09-20
-->
## 2026-09-20 execution snapshot

> This evidence-reconciled snapshot is the issue-generation and visual-roadmap handoff. The longer-horizon strategy below remains canonical context; generated HTML, JSON, progress, issue plans, and commit lists are projections.

**Lifecycle:** executable contract prototype

**Current gate:** Freeze Flow #28's deterministic orchestration scenario and
fixture contract. Coordinate its reference-only artifact and evidence fields
with active #25, but do not wait for #25's later runtime enforcement to define
the conformance vocabulary.

**North-star outcome:** Federated orchestration across holons with stable provider seams, resumable work, and explicit evidence.

### Visual roadmap publication

**Mode:** `central`  
**Route:** `/roadmap/flow/`  
**Current publication evidence:** Architecture, contract source, merged Flow
#23 / PR #24, #26 / PR #27, and successful default-branch CI at
`4599c575a1d0eab46352f900bba47cc97201c426`; no executable release or Pages
publication observed.

Publish the public-safe projection through egohygiene.io at /roadmap/flow/. This repository owns intent and acceptance evidence; it does not add a second site deployment.

### Quest line

<!-- roadmap-step
id: FLO-Q01
status: complete
depends_on: []
issues: []
-->
#### FLO-Q01 — Define orchestration contracts

**State:** `complete`
**Depends on:** None

**Outcome:** Initial architecture and provider contracts describe the intended orchestration boundary.

**Exit criteria:**

- [x] Core contracts and responsibilities are documented.
- [x] Renderflow and other provider boundaries are explicitly deferred or scoped.

**Current evidence:**

- The audit observed a contract and architecture prototype.

<!-- roadmap-step
id: FLO-Q02
status: complete
depends_on: [FLO-Q01]
issues: [7, 23, 26]
-->
#### FLO-Q02 — Freeze extension seams and create a tested executable core

**State:** `complete`
**Depends on:** `FLO-Q01`

**Outcome:** Versioned extension/trust envelopes constrain provider SDK mappings,
a minimal orchestrator executes a fixture through one provider seam, and the
same acceptance gate validates a deterministic external-process transcript.

**Exit criteria:**

- [x] Provider declarations and operator authority are separated by versioned
  manifest and lock contracts.
- [x] Invocation, event, result, and resolution schemas cover both execution
  modes and compatibility failures.
- [x] A runnable library path and hermetic example exist in merged Flow #23 /
  PR #24.
- [x] Default-branch CI proves the in-process success and failure behavior.
- [x] A deterministic process request/transcript seam reuses Flow-owned event
  and result validation without claiming a production runner or sandbox.

**Current evidence:**

- Flow #7 defines the extension/trust schemas, synthetic fixtures, and
  compatibility outcomes.
- Flow #23 / PR #24 supplies deterministic resolution, injected in-process
  execution, Flow-owned event/result validation, and a no-effects hermetic
  example.
- Flow #26 / merged PR #27 supplies deterministic invocation framing,
  host-neutral transcript validation, and a hermetic process example through
  the existing acceptance gate.
- Default-branch CI run 35138134331 passed on Rust 1.85, stable Rust, and every
  repository validator at `4599c575a1d0eab46352f900bba47cc97201c426`.
- This evidence does not claim a public CLI, a production
  child-process runner, artifact acceptance, executable verification, sandbox
  enforcement, real provider adapters, durable state, or resume.

<!-- roadmap-step
id: FLO-Q03
status: active
depends_on: [FLO-Q02]
issues: [13, 25, 28]
-->
#### FLO-Q03 — Freeze conformance fixtures and implement provider adapters

**State:** `active`
**Depends on:** `FLO-Q02`

**Outcome:** Flow-owned scenarios describe bounded cross-provider behavior
without embedding provider algorithms, and adapters reflect current provider
behavior through stable error and evidence contracts.

**Exit criteria:**

- [ ] Stable scenario and fixture identities, immutable provenance, typed
  expectations, execution budgets, and deterministic drift checks are defined.
- [ ] At least two adapters pass contract tests.
- [ ] Provider-specific behavior does not leak into the core model.

**Current evidence:**

- Flow #13 decomposes the orchestration and failure corpus into bounded child
  checkpoints. Flow #28 is executable now and freezes the scenario manifest
  before the hermetic provider kit or real-provider workflows.
- Flow #25 remains the active owner of artifact acceptance, executable/package
  integrity, authority profiles, and process enforcement. Scenario references
  do not pre-empt those contracts.

<!-- roadmap-step
id: FLO-Q04
status: planned
depends_on: [FLO-Q03]
issues: [3]
-->
#### FLO-Q04 — Restore, assess, and resume work

**State:** `planned`  
**Depends on:** `FLO-Q03`

**Outcome:** Issue #3 proves interruption-safe orchestration and explicit recovery decisions.

**Exit criteria:**

- [ ] A saved run can be assessed and resumed deterministically.
- [ ] Recovery evidence distinguishes retry, skip, rollback, and terminal failure.

**Current evidence:**

- Issue #3 tracks restore, assess, and resume behavior.

<!-- roadmap-step
id: FLO-Q05
status: planned
depends_on: [FLO-Q04]
issues: []
-->
#### FLO-Q05 — Stabilize primitives and add Renderflow

**State:** `planned`  
**Depends on:** `FLO-Q04`

**Outcome:** Stable orchestration primitives support Renderflow without coupling the core to rendering.

**Exit criteria:**

- [ ] Primitive contracts are versioned and backward-compatible.
- [ ] A Renderflow integration passes an end-to-end resumability fixture.

**Current evidence:**

- Renderflow integration is intentionally later than the core seams and primitives.

### Roadmap-to-issue handoff

- A step is complete only when its exit criteria and required evidence are satisfied; commit count never determines progress.
- Ready steps without an issue are candidates for the private, duplicate-aware roadmap.issue-plan.json dry run. Planned steps remain preview-only unless a reviewer explicitly opts them in with issue_policy: propose.
- Issue creation or reconciliation requires human approval or an explicitly authorized Pace operation and returns issue references through a reviewable roadmap pull request.
- Pull requests and commits should include Roadmap-Step: <ID>; historical evidence may be linked through existing issue and pull-request relationships.
- Public rendering uses only allowlisted build-time evidence and never places a GitHub token or private issue plan in the browser artifact.

<!-- END ROADMAP EXECUTION SNAPSHOT -->

## Strategic context

Flow begins with three independently released Rust tools. Its first executable
checkpoint is a bounded extension resolution and execution seam: merged
in-process evidence plus host-neutral process transcript validation. It is not
yet a product orchestrator, process runner, or provider integration. Flow will
integrate named releases through public libraries or versioned CLI contracts.
It will not consolidate repositories, copy sibling source, or hide missing
provider capabilities inside the facade.

This roadmap advances on capability evidence rather than dates.

## Approved baseline

The suite ownership table, forbidden dependency edges, federated repository
model, and first contract set are approved by [ADR-0004](docs/architecture/governance/decisions/ADR-0004-federated-suite-contracts.md).
The extension declaration, trust, and operator-authority split is approved by
[ADR-0005](docs/architecture/governance/decisions/ADR-0005-federated-extension-authority.md).
Artifact, capability, and compatibility schemas begin as provisional v1
contracts so real adapter work can refine them without claiming stability.

**Exit evidence:** architecture validation passes; every suite capability has
one primary owner; contract documents are machine-valid; no mutable or copied
sibling dependency is permitted.

## Now — Prove released integration seams

### Freeze the federated extension contract

Use provider-owned manifests plus Flow-owned locks, invocation/event/result
envelopes, and deterministic resolution evidence. In-process libraries and
future bounded process adapters share semantic contracts; only the operator
lock grants permissions, precedence, or fallback.

**Exit evidence:** the contract set validates compatible, incompatible,
over-permissioned, duplicate, malformed, and fallback fixtures; content-changing
hooks produce immutable artifacts; observer hooks remain read-only.

### Bootstrap the executable library seam

Implement deterministic inspection and resolution from explicit manifests,
operator locks, and caller observations. Invoke only a matching,
caller-injected in-process port and return success only after Flow validates the
ordered events and terminal result. Keep observation through `EventSink`
outside selection and result identity so future logging and OpenTelemetry
adapters cannot rewrite provider evidence. A sink failure rejects the
checkpoint execution.

**Delivered evidence:** Flow #23 / PR #24 includes a no-effects, no-artifacts
hermetic port and executable library example, success and adversarial tests,
and CI for Rust 1.85, stable Rust, and the existing Python validators.
Default-branch CI passed at
`979e033409c823b38591b59eca820522efabfa12`. Only trusted candidates are
resolution-eligible. The in-process seam treats declared limits as correlated
metadata, and caller-issued configuration and authorization identities are
correlated but not authenticated.

### Freeze external-process framing

Encode one compact, LF-terminated invocation document and validate a bounded,
caller-supplied JSON Lines provider transcript containing ordered events and one
terminal result. Reuse the existing Flow-owned correlation, redaction-flag,
terminal-consistency, and observer checks. Treat stdout as protocol-only,
stderr as opaque operational evidence, and exit zero as transport evidence
rather than semantic success.

**Delivered evidence:** Flow #26 / merged PR #27 adds the pure framing decoder, hermetic
transcript example, typed framing/output/completion failures, and adversarial
tests. It does not launch or isolate a process, bind filesystem artifacts,
verify executable bytes, deliver cancellation, or authenticate a publisher.

### Freeze the orchestration scenario contract

Define stable scenario and fixture IDs, immutable input and provider references,
ordered topology, typed outcomes and evidence, PR/scheduled/release resource
budgets, bounded coverage claims, and deterministic canonical identity. Keep
real provider execution, durable plan/run/checkpoint semantics, and provider
algorithm corpora outside this contract.

**Candidate evidence:** Flow #28 adds the closed
`flow.scenario-manifest/v1` model, redistribution-safe positive and adversarial
fixtures, independent Rust/Python canonical digest checks, and explicit
coverage gaps. It describes conformance intent and does not implement a
scenario runner.

### Pin the capability matrix

Reinspect each provider at named released revisions. Record available public
libraries, CLI commands, structured formats, side effects, runtime tools, and
known gaps in `docs/integrations/capability-matrix.md`.

**Exit evidence:** every adapter target names a release, interface kind, contract
version, observed behavior, and compatibility result.

### Build adapter contract tests

Implement Flow-owned fixtures and adapters without changing provider ownership.
Choose an in-process adapter only for a stable released library; otherwise use
direct argv invocation with independent stdout/stderr capture, version probing,
signal forwarding, redacted diagnostics, and declared-output verification.

**Exit evidence:** supported versions pass; unknown versions and malformed
provider output fail actionably; no human console text is parsed.

### Implement the restore-and-assess slice

The first slice composes Optiflow and Aniflow:

1. Optiflow inventories a read-only source collection and emits evidence.
2. Flow resolves and records one deterministic plan.
3. Aniflow creates and validates a new temporal master without modifying the
   source.
4. Optiflow rescans source plus output and reports relationships and
   opportunities without applying mutations.
5. Flow records artifacts, compatibility decisions, stage state, provenance,
   validation, and exact resume guidance.

Renderflow is deliberately outside this slice. It joins through its own adapter
after the first two-provider composition proves the orchestration seam.

**Exit evidence:** a redistribution-safe fixture completes with real released
providers; an induced post-Aniflow failure resumes without repeating compatible
work; plan, state, report, logs, provenance, and validation agree.

## Next — Strengthen provider seams

### Aniflow

Formalize structured capability/result output, retain temporal-domain
checkpoint semantics, and make unsupported stream/timing behavior explicit.

### Optiflow

Preserve the read-only v0.1 safety boundary. Expose inventory and relationship
evidence without implying that observation is mutation or optimization.

### Renderflow

Stabilize structured build results and the public core/CLI seam. Prove a
separate Flow adapter for document, image, or audio derivatives; do not describe
general video packaging as present capability.

**Exit evidence:** each provider remains independently buildable and releasable;
Flow compatibility fixtures exercise only public released interfaces.

## Next — Build orchestration primitives

- `flow doctor`: discover provider versions and explain compatibility.
- `flow inspect`: preserve source observations and artifact identity.
- `flow plan`: resolve typed capabilities before expensive work and emit a
  stable plan digest.
- `flow run`: use isolated workspaces, atomic state, bounded cancellation, and
  verified outputs.
- `flow resume`: reuse only checkpoints whose inputs, configuration,
  implementation, and outputs remain compatible.

## Later

- Versioned events, results, provenance evidence, plans, and run manifests.
- Opt-in provenance tooling and honest derivative claims under ADR-0002.
- Reusable signed pipeline packs and third-party adapter contracts.
- Cross-run content-addressed caching with explicit privacy and invalidation.
- Richer interfaces after CLI and structured semantics stabilize.

## Maybe

- Distributed execution with explicit data-residency policy.
- Hardware-aware scheduling across local accelerators.
- A trust-policy registry for provenance validators.

## Dependencies and risks

| Risk | Strategic response |
| --- | --- |
| Provider release or toolchain divergence | Pin named releases and prefer process isolation when library compatibility is costly |
| Schema churn | Version explicitly and reject unsupported versions without silent fallback |
| Overlapping orchestration logic | Keep domain execution in providers and cross-holon decisions in Flow |
| External tool fragility | Probe capabilities, isolate adapters, and retain raw diagnostics |
| Misleading provenance claims | Enforce ADR-0002 and explicit epistemic claim states |
| Source or history duplication | Keep implementations and history in their owning repositories; consume releases only |

## Open questions

- Which provider APIs are stable enough for library adapters?
- How should a Flow release publish its tested provider-version matrix?
- Which provisional JSON contracts should later gain Rust bindings?

## Validation

Roadmap horizons advance on named evidence, not elapsed time. New work must align
with the ownership table, [suite boundaries](docs/integrations/suite-boundaries.md),
versioned contracts, and accepted decisions.
