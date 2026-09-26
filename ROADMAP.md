---
schema: aether.architecture-document/v1
id: flow-roadmap
title: Flow Roadmap
kind: architecture-document
version: 1.3.2
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-09-25
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

## 2026-09-25 live suite handoff

> [!IMPORTANT]
> This is the current near-term execution handoff for agents. It supersedes
> older active-checkpoint and queue text below where they conflict. Re-query
> live issue, release, and CI state before starting a branch.

#30 is complete: [PR #61](https://github.com/egohygiene/flow/pull/61) merged as
`dfb16b347975e3292dc2928c8c48463f51dcf6d1`, with green
[default-branch CI](https://github.com/egohygiene/flow/actions/runs/36209682152).
Its [acceptance matrix](docs/integrations/acceptance-scenarios.md) proves 81
scenarios twice on Rust 1.85 and stable.

The #49 review candidate adds [durable prepared execution](docs/integrations/durable-state.md):
versioned plans and state, atomic immutable snapshots, workspace locking,
accepted checkpoints, fresh resume eligibility, and explicit recovery decisions.
After this candidate merges and the default-branch gate passes, #31 is next.
FLO-Q03 remains active until real released-provider adapters satisfy its
remaining exit criteria. [#62](https://github.com/egohygiene/flow/issues/62) is
later documentation visualization work and does not block this sequence.

### Central Flow chain

1. [#30](https://github.com/egohygiene/flow/issues/30) — prove compatibility,
   artifact, provider-failure, diagnostic, and privacy scenarios.
2. [#49](https://github.com/egohygiene/flow/issues/49) — persist versioned
   plans, runs, checkpoints, validation evidence, authority decisions, and
   recovery state.
3. [#31](https://github.com/egohygiene/flow/issues/31) — prove interruption,
   retry, resume, invalidation, and authority transitions against durable state.
4. After #49, integrate immutable provider releases as they become available:
   [#50](https://github.com/egohygiene/flow/issues/50) for Optiflow,
   [#52](https://github.com/egohygiene/flow/issues/52) for Renderflow, and
   [#51](https://github.com/egohygiene/flow/issues/51) for Aniflow. These adapter
   tasks may proceed in parallel with #31 when their own release dependencies
   are satisfied.
5. [#53](https://github.com/egohygiene/flow/issues/53) — ship the supported
   Flow CLI and reconcile the vertical slice in [#3](https://github.com/egohygiene/flow/issues/3).
6. [#32](https://github.com/egohygiene/flow/issues/32) and
   [#33](https://github.com/egohygiene/flow/issues/33) — prove clean-room static
   publication and temporal workflows with released providers.
7. [#34](https://github.com/egohygiene/flow/issues/34) — turn those scenarios
   into the clean-room compatibility and regression release gate.
8. [#54](https://github.com/egohygiene/flow/issues/54) — publish Flow's first
   immutable integration-candidate release after #3 and #34, with an explicit
   disposition for the observability work under
   [#14](https://github.com/egohygiene/flow/issues/14).

[#10](https://github.com/egohygiene/flow/issues/10), the exhaustive comic and
multilingual orchestration workload, consumes this maturing foundation and the
Renderflow localization lane. It is a concrete application/proving lane, not a
hidden prerequisite for #54 unless a later reviewed dependency decision makes
it one.

### Provider release lanes feeding Flow

```text
Optiflow:   #88 → #89 → #90 → #91 → #92 → #96 → #93
                         └─ read-only #89 can feed Flow #50
             #65 fixtures → #94
                    #93 + #94 → #95

Renderflow: #415 → (#416 + #417) → #418 → #419 → Flow #52
             #406 remains the parallel/later localization lane

Aniflow:    #8 → #13 → #32 → #33 → #34 → bounded #24 closeout → #10 → Flow #51

Creative artifact forest:
Renderflow #421 → #422–#430 → Flow #10 exhaustive comic workflow
```

The suite therefore has four useful parallel ready fronts:
Flow #30, Renderflow #415, Optiflow #88, and Aniflow #8. Keep provider domain
logic in the provider repositories and consume only immutable public contracts
from Flow.

### Finish-line sequence

After the provider lanes and Flow integration/release work, complete the
provider post-roadmap audits
([Optiflow #61](https://github.com/egohygiene/optiflow/issues/61),
[Renderflow #409](https://github.com/egohygiene/renderflow/issues/409), and
[Aniflow #17](https://github.com/egohygiene/aniflow/issues/17)) before the final
suite-level Flow audit in [#12](https://github.com/egohygiene/flow/issues/12).
Audit findings should become bounded remediation issues rather than silently
expanding the active implementation issue.

<!-- BEGIN ROADMAP EXECUTION SNAPSHOT -->
<!-- roadmap-manifest
schema: hygiene.roadmap/v1alpha1
repository: egohygiene/flow
visibility: public
publication: central
route: /roadmap/flow/
updated: 2026-09-25
-->
## 2026-09-24 execution snapshot

> [!IMPORTANT]
> **2026-09-24 hermetic-kit closeout checkpoint**
>
> Flow #42, #44, and #45 are merged. Draft PR #60 consolidates #46's four
> sequential checkpoints (#56–#59), including the final hermetic-kit package
> layout/finalization, behavior, and requirement evidence. Merge #60, require
> green default-branch CI, then close #46 and #29 and hand the executable
> conformance matrix to #30.
> [The 2026-09-22 live-sweep checkpoint](docs/roadmaps/flow-suite-live-sweep-2026-09-22.md)
> remains the historical suite-wide queue; this checkpoint supersedes only its
> older Flow PR state.


> This evidence-reconciled snapshot is the issue-generation and visual-roadmap handoff. The longer-horizon strategy below remains canonical context; generated HTML, JSON, progress, issue plans, and commit lists are projections.

**Lifecycle:** executable contract prototype

**Current gate:** Review and merge Flow PR #60, then require green
default-branch CI before closing #46 and #29. Flow #30 is the exact next
checkpoint for the broader executable compatibility, artifact, provider,
diagnostic, and privacy scenario matrix.

**North-star outcome:** Federated orchestration across holons with stable provider seams, resumable work, and explicit evidence.

### Visual roadmap publication

**Mode:** `central`  
**Route:** `/roadmap/flow/`  
**Current publication evidence:** Architecture, contract source, merged Flow
#23 / PR #24, #26 / PR #27, #28 / PR #35, #36 / PR #37, #38 / PR #39, #40 /
PR #41, #42 / PR #43, #44 / PR #47, and #45 / PR #48, plus candidate #46 /
PR #60; no executable release or Pages publication observed.

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
issues: [13, 25, 28, 29, 30, 36, 38, 40, 42]
-->
#### FLO-Q03 — Freeze conformance fixtures and implement provider adapters

**State:** `active`
**Depends on:** `FLO-Q02`

**Outcome:** Flow-owned scenarios describe bounded cross-provider behavior
without embedding provider algorithms, and adapters reflect current provider
behavior through stable error and evidence contracts.

**Exit criteria:**

- [x] Stable scenario and fixture identities, immutable provenance, typed
  expectations, execution budgets, and deterministic drift checks are defined.
- [x] Bound artifact IDs to root-confined host observations and accept them only
  after exact invocation, provider, and content correlation.
- [x] Require exact locked package and executable observations before process
  request or transcript acceptance while keeping authenticity claims separate.
- [x] Require exact requested/granted process authority and explicit isolation
  evidence before request or transcript acceptance.
- [x] Launch and supervise one exact `trusted-unconfined` direct child with
  bounded stdio, timeout/cancellation control, escalation, and reaping.
- [ ] At least two adapters pass contract tests.
- [ ] Provider-specific behavior does not leak into the core model.

**Current evidence:**

- Flow #13 decomposes the orchestration and failure corpus into bounded child
  checkpoints. Flow #28 / merged PR #35 freezes the scenario manifest before
  the hermetic provider kit or real-provider workflows.
- Flow #36 / merged PR #37 supplies the artifact-binding and host-observation
  boundary with green default-branch CI at
  `55341605968d3343e74d8bdd2b188c99a855aca0`.
- Flow #38 / merged PR #39 supplies exact locked package/executable observation
  with green default-branch CI at
  `9cbe58eff5174faa9511a64211e8119e5c7bda6d`.
- Flow #40 / merged PR #41 supplies exact authority/isolation preflight with
  green default-branch CI at
  `5f411da6e7040e3494cbd275729de9a2ed8c67a3`.
- Flow #25 is complete through #42 / merged PR #43, which adds bounded direct
  launch and supervision without claiming sandbox, durable-state, or
  real-adapter behavior.
- Flow #44 / merged PR #47 establishes the immutable hermetic package and
  accepted success path. Flow #45 / merged PR #48 adds its closed lifecycle and
  protocol matrix.
- Flow #46 / candidate PR #60 carries the evidence intended to complete the
  remaining artifact adversaries, fixed single-provider and two-provider
  compositions, exact package and executable-digest correlation and tamper
  rejection, the closed behavior catalog, and every parent #29 requirement
  mapping. Its evidence is indexed in
  [the hermetic provider kit](docs/integrations/hermetic-provider-kit.md).
- The synthetic providers are conformance infrastructure, not real adapters.
  FLO-Q03 remains active for #30 and the later released-provider adapter work.

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
in-process evidence, host-neutral process transcript validation, and one bounded
`trusted-unconfined` local runner. It is not yet a product orchestrator or real
provider integration. Flow will integrate named releases through public
libraries or versioned CLI contracts. It will not consolidate repositories,
copy sibling source, or hide missing provider capabilities inside the facade.

This roadmap advances on capability evidence rather than dates.

## Approved baseline

The suite ownership table, forbidden dependency edges, federated repository
model, and first contract set are approved by [ADR-0004](docs/architecture/governance/decisions/ADR-0004-federated-suite-contracts.md).
The extension declaration, trust, and operator-authority split is approved by
[ADR-0005](docs/architecture/governance/decisions/ADR-0005-federated-extension-authority.md).
The process transcript and artifact-observation boundaries are approved by
[ADR-0006](docs/architecture/governance/decisions/ADR-0006-bounded-process-transport.md)
and
[ADR-0007](docs/architecture/governance/decisions/ADR-0007-root-confined-artifact-acceptance.md),
and
[ADR-0008](docs/architecture/governance/decisions/ADR-0008-locked-execution-subjects.md).
Exact process authority and isolation evidence is approved by
[ADR-0009](docs/architecture/governance/decisions/ADR-0009-process-authority-isolation.md).
Bounded direct-child launch and supervision is approved by
[ADR-0010](docs/architecture/governance/decisions/ADR-0010-bounded-direct-process-supervision.md).
Artifact, capability, compatibility, binding, host-observation, execution-
subject, and authority/isolation schemas begin as provisional v1 contracts so
real adapter work can refine them without claiming stability.

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

**Delivered evidence:** Flow #28 / merged PR #35 adds the closed
`flow.scenario-manifest/v1` model, redistribution-safe positive and adversarial
fixtures, independent Rust/Python canonical digest checks, and explicit
coverage gaps. It describes conformance intent and does not implement a
scenario runner.

### Bind and observe artifacts

Bind immutable input and candidate-output IDs to logical ports, media types,
kinds, and portable locators beneath one caller-selected root. Recompute file
and deterministic recursive-directory identity with Flow-owned code. Keep
provider execution validation separate from artifact acceptance, and require a
complete result plus exact invocation, result, event, binding, and host
observation correlation. Bind each opaque observation token to its exact
binding snapshot and host-local canonical root, then require equal final
root-confined evidence immediately before promotion. This freshness check does
not make either observation atomic.

**Delivered evidence:** Flow #36 / merged PR #37 adds
`flow.artifact-bindings/v1`, `flow.artifact-observations/v1`, opaque observed and
accepted artifact tokens, deterministic file/directory identity, and
adversarial filesystem/correlation tests. It does not verify executables,
enforce provider authority, launch a process, or validate provider-native
artifact semantics.

Flow #57 hardens that acceptance boundary against changed evidence and stale
binding context without changing the portable artifact schemas or assigning
authoritative digest semantics to free-form provider provenance. Flow #58 adds
fixed two-stage single-provider and two-provider compositions that hand the
exact accepted inspection artifact into a transformation stage through public
Flow boundaries and produce equal normalized evidence and bytes across fresh
roots. It remains test-owned sequencing rather than a graph scheduler. Flow #59
adds candidate closeout evidence for the exact materialized package layouts,
correlated package and executable digests across the manifest, locks, and
observations, tamper rejection, redistribution terms, behavior-to-test catalog,
parent #29 requirement matrix, residual gaps, and #30 handoff. Parent closure
waits for PR #60 to merge and default-branch CI to pass.

### Verify locked package and executable subjects

Pin exactly one package directory and one regular executable file to the
resolved extension lock, provider identity, capability, process interface, and
declared entrypoint. Reuse root-confined observation, compare both SHA-256
identities exactly, and require the resulting opaque match token for process
request encoding and transcript validation. Keep content equality, publisher
declaration, configured operator trust, cryptographic verification, and
transparency-log status as distinct claims.

**Delivered evidence:** Flow #38 / merged PR #39 adds
`flow.execution-subject-lock/v1`,
`flow.execution-subject-observations/v1`, deterministic canonical lock and
observation identity, fresh package/executable matching, closed unsupported-
claim rejection, and adversarial context/content tests. It does not authenticate
a publisher, verify signatures or transparency proofs, bind observation to a
later launched file object, enforce authority/isolation, launch a process,
persist state, or implement a real provider adapter.

### Enforce authority and isolation profiles

Translate broad provider permission declarations and operator grants into one
closed, exact process-authority profile. Bind ordered argv, opaque secret
handles, every resource and side-effect allowlist, denied ambient authority,
operator trust, and selected isolation to the exact invocation and already-
matched execution subjects. Require separate host evidence to identify the
canonical profile, subject observation, backend, and enforcement status for
every authority dimension.

**Delivered evidence:** Flow #40 / merged PR #41 adds
`flow.process-authority-profile/v1`,
`flow.process-enforcement-evidence/v1`, deterministic canonical identities,
opaque `AuthorizedProcess` preflight, sandbox-required process routing, and
positive/adversarial authority and evidence tests. It validates caller-attested
host evidence; it does not launch a process, implement an operating-system
sandbox, authenticate the evidence source, persist run state, or implement a
real provider adapter.

### Launch and supervise bounded provider processes

Compose the subject, authority, and transport gates around one real local
`trusted-unconfined` direct child. Invoke the exact locked executable with
literal argv and an explicit cwd; clear and reconstruct the environment from
authorized opaque handles; supervise stdin and independently bounded output;
enforce deadline and caller cancellation with grace and escalation; reap the
direct child; and retain transcript validation as the sole promotion path.

**Delivered evidence:** Flow #42 / merged PR #43 adds `LocalProcessRunner`,
`CancellationSignal`, bounded stdio workers, Unix `SIGTERM` grace and forced
escalation, typed interruption, direct-child reap proof, and real-process
positive/adversarial tests. It does not provide a sandbox, descendant
containment, descriptor-bound execution, durable state, retry/resume, or a real
provider adapter.

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
