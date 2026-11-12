---
schema: aether.architecture-document/v1
id: flow-architecture
title: Flow Architecture
kind: architecture-document
version: 0.28.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-11-11
governed_by:
  - architecture-architecture
depends_on:
  - flow-system
related:
  - flow-methodology
  - flow-decisions
supersedes: []
---

# Flow Architecture

## Publisher-authenticated clean installation

A separate opaque publisher verifier authenticates portable owner reports and
actual release metadata before local binaries or runtime witnesses exist. It
reuses the existing fixed signature/client protocol under independently selected
host policy. Its authority is limited to copying exact selected executable,
contract and support bytes. Installation retains a pinned historical admission
inside its existing receipt; status, rollback and removal remain local readback.
Publisher authentication does not imply host/native readiness and cannot be
converted from serialized receipt flags into native execution authority.

## Independent portable publisher declarations

`publisher_report` owns the closed portable archive/member and sidecar declaration
used before a local installation or runtime witness exists. Parsing and row binding
produce no authority. Each repository owns its offline report assembler; Flow's
release owner emits the same protocol without importing siblings. Publisher
signatures and actual release metadata must be independently verified by the host.
Full local/native qualification retains the existing separate evidence contract.

## Authenticated release admission

`release_evidence` consumes explicitly pinned host policy, cohort, installed
files, owner reports and retained positive native evidence. Only an explicit
verification operation may invoke a pinned `gh attestation verify` process and
permit its separate trust-root networking. Ordinary doctor, planning and file
matrix observation remain read-only and never invoke a verifier. Signer workflow,
signer revision and exact source revision/ref are policy, not report authority.
Opaque in-memory evidence is created only after cryptographic verification and
current target, host identity, dependency, clean-install and native/domain
acceptance readback. Source copies, unsigned booleans, synthetic/refusal tests,
missing/failed gates and stale proofs cannot grant release admission.

Released native approval/run/status/resume use a distinct approval contract and
require that opaque evidence again. The existing native execution supervisor and
domain acceptance remain the only media engine. Inner native receipts describe
native completion; an outer release receipt separately binds authenticated
admission. Neither grants creative review, upload or provenance signing authority.
All retained coverage rows, including unavailable targets, remain in the matrix.
The evidence policy/adapter decision is proposed; no ADR is ratified by source.

## Fixed manual-SRT pilot preset

`pilot` composes existing native Aniflow video and Renderflow timed-text/optional
audio plans under one exact cohort and target. A manual SRT remains a pinned
original. Planning is read-only; approval binds each existing native plan and its
authority. Sequential run/continue delegates to the native supervisor, retains a
start marker before each launch, and never replays uncertain stages. Native
status and acceptance validators own actual output acceptance. The local handoff
indexes current accepted payloads and remains pending creative/publishing review;
it grants no upload, provenance signing or release authority.

## Main CLI preparation and diagnostics

The main command delivery delegates explicit local preparation and status to
`ai_preparation::setup::documents`; it does not duplicate observation, install
models or contact a runtime. Doctor can select that private bundle and a pinned
suite installation receipt. Optional native-request preparation pins the existing
`setup.json` marker, rechecks every bound document and selected asset before
execution, and rejects output overlap. Disabled and matched-byte states remain
separate from runtime, offline enforcement, memory and release qualification.
The provisional `flow-ai-setup` source utility remains compatible.

## Independent suite installation

`distribution` owns explicit pinned file acquisition inputs, immutable private
suite installations, offline directory bundles, selected activation/rollback and
bounded removal of unchanged owned copies. Its public SDK validates the selected
cohort/tool/target/contracts and reuses the existing file observation boundary.
Copying never invokes installed providers or downloads dependencies. Installation
receipts bind exact intent, membership and current bytes, separately from publisher
authentication and native qualification. Activation uses a caller-selected pointer
and expected previous digest; rollback reselects a retained installation. Removal
requires an exact receipt pin, complete unchanged membership and an external
create-new removal record; originals and unknown files are never removed.

## Flow artifact preparation

`release` exposes the compiled Flow CLI/library/contract inventory without I/O.
The repository-owned packaging adapter prepares only the real `flow` executable,
its exact compiled descriptor, a tracked portable source archive, supplied provider
cohort lock, dependency inventory, notices and checksum manifest. Native builds
require the matching installed toolchain/host and explicit fresh external work
roots. Registry publication remains unavailable. Fixture binaries are not shipped.
Preparation and its printed manual publication handoff establish byte/source
relationships, not supply-chain authentication, native compatibility, installation
or qualification. Release signing/publication stays with the existing release
authority; this source adds no signer, auto-dispatch or sibling dependency.

## Native command and release selection delivery

The public command delivery SDK routes versioned native request/plan documents
through `native` rather than the older released-only Optiflow v1 scope plan.
`local_input::plan_native` composes exact file/ordered-collection inspection with
an explicit cohort/provider/configuration selection and checks identical ordered
bytes; directory widening is refused. Candidate approvals carry two explicit
acknowledgements. Status stays local/read-only, and native resume requires an
explicit retained native-run locator. `release-matrix` and command doctor v2 use
the cohort projection and installed file observer without launching or installing
providers. Source routes, released support and unobserved qualification stay
distinct; old command/local-input wire contracts retain their original meaning.

## Native publication route

`native::renderflow` adds canonical single-file and ordered-collection translation
against `renderflow.release-contract/v2`. Flow checks exact selected scope/order,
offline/local-only configuration and reserved output root before native launch.
The existing Renderflow planner/executor, forest identity, source inventory and
native publication verifier remain the domain owners. Complete Flow acceptance
requires a complete matching native forest, every selected target's accepted
publication and current confined payload hashes. Partial targets retain recovery
state. Explicit graph resume requires the original forest and output root.
Native composition/presentation/briefing/AI/extension lanes retain separate
coverage and are not silently enabled by this canonical file/collection route.

## Native temporal route

`native` translates an operator-pinned Aniflow source-entry music-video candidate
through its independent CLI. Read-only preparation binds the exact cohort row,
descriptor, executable, source, configuration, target and limits without launch.
Explicit plan-bound operator approval acknowledges unqualified candidate and
trusted-unconfined execution. The existing runner's direct-child supervisor is
shared with the native adapters, including the older Optiflow protocol; no second
process engine or sibling dependency is added. Attempts are immutable and locked.
Aniflow owns recipe planning, accepted temporal prefixes, status and completion.
Flow correlates their identities, reobserves current confined accepted payloads,
and retains native evidence separately from graph state and release qualification.
Status performs local readback only. Explicit resume names one existing native run
and requires its original intent; interruption never causes guessed replacement.
See [native adapters](../../integrations/native-adapters.md) and proposed ADR-0017.

## Release cohort projection

`release_cohort` owns the versioned suite inventory and release matrix, separately
from the existing `flow.compatibility/v1` extension decision. Each row binds a
tool/interface/profile, input and output direction, target, effects, requirements,
independently versioned immutable artifact and contract bytes. Retained rows must
remain present even when artifacts or implementations are missing. Declarations
cannot deserialize into host evidence. An explicit trusted evidence port verifies
each of five facts against the complete row subject; the built-in local observer
only establishes installed executable and contract bytes. Publication, native
availability and qualification require their own evidence. No downloads, provider
launch, source checkout discovery or release promotion occurs during projection.
See [ADR-0017](../governance/decisions/ADR-0017-native-release-cohorts.md), proposed.

## Purpose and scope

This document defines the intended structural organization of the Flow
orchestrator repository and its released integration seams. It fixes dependency
direction and contract ownership while leaving exact Flow crate names to
implementation decisions.

## Structural units

### Suite applications

The `flow` CLI source is a thin delivery adapter over the orchestration
library. Its responsibility is argument parsing, human presentation, stable
exit mapping and shell completion. The supported product CLI remains
[#53](https://github.com/egohygiene/flow/issues/53); the existing synthetic
conformance-provider binary is not that delivery surface.

The first product-command source checkpoint exposes the existing pinned
Optiflow read-only adapter behind separate inspect/plan, explicit local-operation
approval, run and receipt/status/recovery surfaces. Planning observes selected
local identities but launches no provider; execution rechecks the plan and pins
before using the existing native adapter. Its attempt receipt remains distinct
from generic durable-graph state. Interrupted native attempts require inspection,
never automatic replay. Missing independently released Aniflow/Renderflow routes
remain unavailable. Machine presentation is one versioned result; telemetry and
private provider diagnostics cannot become authority. Source inclusion does not
establish a qualified CLI/release or waive #53's retained dependencies. See
proposed ADR-0016.

The local-input boundary observes explicitly selected file, bounded-directory or
ordered-collection identities before profile planning. It preserves order and
private native locators, bounds traversal/hash work, refuses links/aliases/mount
escapes, and derives no media-format acceptance from names/extensions. These
observations are neither execution input freezing nor provider-native validation.
The pinned native adapter currently accepts only whole recursive directory
scope; narrower selections remain unavailable rather than being broadened.
Inspection limits do not become unimplemented provider runtime quotas. Scope-only
planning remains explicitly partial; independently released profile/adapter and
clean-install acceptance in #70 remain open.

### Orchestration core

The core owns pipeline loading, capability resolution, deterministic planning,
run coordination, state transitions, progress, cancellation, recovery, and
diagnostics. It depends only on Flow-owned contracts and adapter ports.

### Domain holons

Aniflow, Optiflow, and Renderflow remain independent repositories and releases.
Their internal modules remain private. A holon exposes only stable domain types,
capability descriptors, and operations justified by its contract.

### Shared contracts

Flow-owned, schema-versioned documents define suite artifact identities,
capability descriptors, compatibility decisions, events, results, and
orchestration conformance scenarios. Holons may keep native models; Flow
adapters perform explicit translation rather than forcing a Flow dependency
into a provider repository.

The federated extension boundary separates provider declarations from operator
authority. An extension manifest declares identity, integrity, compatibility,
domain ownership, capabilities, requested permissions, execution modes, hooks,
and checkpoint behavior. A Flow-owned lock records discovery location, the
verified digest, trust mode, granted permissions, capability precedence, and
fallback order. A manifest can never grant itself authority or precedence.

The scenario manifest pins redistribution-safe inputs and providers, ordered
topology, typed expectations, resource budgets, provenance, and bounded
coverage claims. It references other versioned documents by schema and digest.
It does not duplicate provider algorithms or define runtime plans, durable runs,
checkpoints, or transition storage.

Artifact bindings remain separate from path-independent extension envelopes.
A Flow-owned binding set maps immutable input and candidate-output identities to
logical ports, media types, kinds, and portable root-relative locators for one
run. A Flow observer computes file or recursive directory identity beneath one
caller-selected root. Its opaque token privately retains the exact binding
snapshot and absolute canonical root while omitting that sensitive host context
from portable evidence and manual `Debug` output. Only the separate artifact-
acceptance gate can correlate those observations with resolved capability,
invocation, event, and terminal result evidence. Immediately before promotion,
the gate re-observes the retained root and requires the portable evidence to
remain equal.

Process execution subjects are a third, distinct boundary. A Flow-owned lock
pins one package directory and one regular executable file to an exact
extension lock, provider identity, capability, process interface, and declared
entrypoint. Flow observes both beneath a selected root and issues an opaque
match token only when both digests equal the lock. Process request encoding and
transcript validation require that token. Content equality, publisher
declaration, configured operator trust, cryptographic verification, and
transparency-log verification remain separate evidence claims; digest equality
does not imply authenticity.

Process authority and isolation are a fourth distinct boundary. A closed
profile translates broad manifest permissions and operator grants into exact
per-invocation argv, opaque secret handles, resource allowlists, telemetry
propagation, operator trust, and selected isolation. Separate host evidence
states which exact profile a named backend reports enforcing. Flow issues an
opaque authorization token only after the resolution, invocation, matched
execution subjects, profile, and evidence agree. The evidence is caller-
attested: correlation is implemented, while operating-system enforcement and
evidence authentication are not.

### Durable execution state

Flow owns an explicit run workspace and a versioned immutable plan. The durable
coordinator records authorization and execution intent before entering the local
runner, then records a checkpoint only after the existing transcript and artifact
acceptance gates succeed. In-memory seams remain available for conformance and
unit tests; supported persistent execution enters through the durable coordinator.

Snapshots are immutable, ordered, digest-linked documents committed by a
same-directory rename under an exclusive workspace lock. Reopening validates the
whole retained sequence and never infers completion from output files. Interrupted
attempts require an explicit recovery decision. Reuse assessment compares the
plan, inputs, provider and capability identities, configuration, authority,
implementation profile, and freshly observed artifacts. Deserialized evidence
does not reconstruct opaque execution or authorization tokens.

State contains typed decisions, identities, relative artifact bindings, and
allowlisted validation evidence. Secret values, configuration values, raw process
streams, and provider-authored messages remain outside the store. The workspace
is trusted local state, not a cryptographically authenticated or sandboxed store.
ADR-0011 owns the persistence, migration, and recovery rationale. ADR-0012 adds
read-only graph assessment: a complete current context inventory and the exact
saved plan are required before classifying every step in dependency order.
Completed prerequisites must be freshly reusable, not merely historically
succeeded. Stale evidence invalidates that step and every descendant for reuse;
unrelated branches retain their independently assessed eligibility. Pending or
unresolved prerequisites block their descendants. These assessments never rewrite
accepted history or grant execution authority. Dependent execution recomputes
the assessment, and single-step entry points refuse dependent steps. Automatic
scheduling, retry, cross-plan migration, and provider-native checkpoints remain
later orchestration work.

The test-owned lifecycle corpus exercises these same public durable APIs with
synthetic providers, immutable transition projections, fresh-process reopening,
and explicit recovery decisions. Its versioned receipts are conformance evidence,
not executable plans, authorization tokens, or a second runtime state model.
Portable reports contain allowlisted identities and typed outcomes; raw operational
evidence stays local. The lifecycle guide owns recipe coverage and resource limits.

Counter-backed conformance additionally checks that recorded authority and intent
precede provider launch, accepted work is not relaunched, and unresolved attempts
require matching recovery authority. Synthetic provider cleanup may fail or be
cancelled after removing disposable work. Flow preserves the unaccepted candidate,
unfinished work, and failure history; it does not infer complete cleanup or perform
automatic compensation. Explicitly acknowledged retries may repeat an uncertain
effect. These proofs do not establish exactly-once behavior for unconfined providers.

### External adapters

Adapters isolate process invocation, version/capability probing, structured
result parsing, redaction, signal behavior, and output verification. Commands
are constructed as executable plus argv, never as shell strings.

Pinned in-process adapters and bounded process adapters share the same
invocation, event, result, validation, and provenance semantics. Process
adapters must additionally enforce every time, output, cancellation,
filesystem, environment, subprocess, network, AI, GPU, and side-effect
guarantee they claim outside the provider process. A declaration, grant,
transcript check, or adapter policy is not by itself sandbox enforcement.

A sandbox-required process candidate may resolve to the downstream authority
gate, where only a complete `sandboxed` profile and exact caller-attested
enforcement statement can construct the process authorization token. The
unconfined in-process seam remains restricted to operator-`trusted` providers.

The first process boundary is host-neutral: Flow deterministically encodes one
versioned invocation and validates a caller-supplied JSON Lines stdout
transcript, bounded stderr length, and completion observation. That seam reuses
the same Flow-owned event and result validation as in-process execution. It does
not spawn, signal, time out, cancel, reap, or isolate a child process. Before
encoding or accepting process evidence, it does require a fresh opaque match
for the exact locked package and executable subjects. The portable observer
cannot prove that a later launcher opens the same unchanged file object.

The bounded local runner composes those gates for one real
`trusted-unconfined` direct child. It freshly observes the locked subjects,
invokes the exact executable with literal argv and an explicit cwd, clears and
reconstructs the environment from opaque authorized handles, supervises stdin
plus independently bounded stdout/stderr workers, enforces the declared
deadline and caller cancellation, applies the declared Unix termination grace,
and reaps the direct child before returning. It preserves the host-neutral
transcript validator as the sole promotion path to `ValidatedExecution`.

This runner is not a sandbox. It does not authenticate host evidence, constrain
filesystem/network/subprocess authority, contain descendants, or bind the
fresh digest observation to the host's later executable file object.

The first released-provider adapter pins the immutable Optiflow v0.1.1
executable and consumes its native `optiflow.command-result.v1` single JSON
document. It does not translate native stdout into a synthetic Flow JSON Lines
transcript. Its public library API probes the exact release, exposes only
read-only scan, report, and exact-duplicate review-plan capabilities, and
rejects mutation capabilities individually. It clears the child environment,
disables configuration discovery and media probing, constrains traversal, and
checks source root and filesystem identity between steps. Flow independently
verifies each committed provider artifact set, its member bytes and schema
bindings, and the review-only plan safety fields.

The adapter records a separate `flow.optiflow-read-only-receipt/v1` local
attempt before launch and after scan, report, and plan. A reopened intermediate
attempt requires review. Saving and reopening share one pure receipt-contract
gate: closed shapes and explicit nullable fields, pinned environment and
capabilities, ordered command evidence and exact artifact roles, digest forms,
and status consistency. Completion requires a present plan digest correlated
with the retained plan artifact; absent values never count as matching proof.
Reopening additionally confines and re-hashes the recorded artifact files, then
binds source/run/plan/set identities and marker members to those same observed
native bytes. It never authenticates an editable local record.
Independent schema/runtime fixtures cover this native receipt outside the
generic suite contract-set manifest. This receipt is local audit evidence, not an execution grant,
checkpoint in the generic durable graph, provider signature, sandbox claim, or
approval. The existing Flow JSONL runner and durable graph contracts remain
unchanged. ADR-0013 owns this native-protocol exception and the staged handoff
to later composition.

### Extension lifecycle

Flow coordinates extensions through the ordered lifecycle `discover → inspect →
plan → authorize → execute → validate → commit evidence`. Discovery and
inspection do not execute extension code merely because a binary is available.
Planning resolves compatible capabilities and effects without granting
authority. Execution begins only after the effective lock and requested action
produce an explicit authorization record. Evidence is committed only after
declared outputs and validations have been assessed.

Read-only observers may consume redacted lifecycle events. A content-changing
hook is a declared capability invocation: it consumes immutable artifact
references and produces a new immutable artifact. Hooks cannot rewrite plans,
change authorization, bypass validation, or publish implicitly.

## Boundary rules

### Proposed operational observability boundary

The [suite observability profile](../../observability/README.md) is the proposed
contract for #15. It keeps operational logs, aggregate metrics and causal traces
outside deterministic planning, acceptance and checkpoint identity. Typed
progress, diagnostics, native command results and durable evidence keep their
existing owners. A product CLI owns its subscriber and presentation; embedded
library callers own theirs. Telemetry export and child context propagation are
disabled by default and require separate explicit configuration and authority.

This proposal does not change the accepted Flow JSON Lines process protocol or
the native Optiflow single-document adapter. Product machine stdout and provider
transport stdout are separate interfaces. No exporter, common package,
instrumentation or new runtime guarantee is delivered by this document.
[ADR-0014](../governance/decisions/ADR-0014-operational-observability.md) records
the new proposal; #16 owns two product pilots and their runtime evidence.

### Governing structural boundaries

- CLI crates depend inward on their corresponding libraries.
- Flow orchestration depends on Flow-owned adapter ports.
- A library adapter may depend on a released public holon library.
- A process adapter may invoke a versioned holon CLI and consume its structured
  output.
- A holon never depends on Flow orchestration or Flow contracts.
- No sibling holon dependency is allowed.
- No copied sibling source, path dependency, Git submodule, or mutable branch
  dependency is allowed.
- Schema types are separated from local persistence models.
- Human console text is never parsed when a supported structured contract
  exists.
- Pipeline definitions select registered typed capabilities, not arbitrary
  commands.
- Extension manifests are inert declarations; explicit configuration and a
  digest-pinned lock control discovery and authorization.
- Provider precedence and fallback are operator policy, not self-declared
  authority.
- Cancellation, progress, partial results, diagnostics, validation, checkpoint
  compatibility, and failure classification use versioned envelopes.

## Dependency direction

```text
flow-cli -> flow-core -> Flow adapter ports -> Flow-owned contracts
                             |                    ^
                             +-> library adapter -+-> released holon library
                             +-> process adapter ----> versioned holon CLI

holon-cli -> holon library -> holon internals

aniflow -X-> optiflow
aniflow -X-> renderflow
optiflow -X-> renderflow
```

Subprocess adapters may replace a library edge where process isolation,
toolchain incompatibility, or release independence warrants it; the logical
direction is unchanged.

## Communication patterns

- Request/response for discovery, planning, and bounded operations.
- Typed event streams for stage progress and diagnostics.
- Atomic manifest snapshots for durable run state.
- Immutable artifacts and content digests for stage handoff.
- Versioned schemas with explicit rejection or migration of unsupported versions.

## Significant constraints

The current repositories use Rust 2021/2024 and minimum versions from 1.85 to
1.94. Release and toolchain independence are architectural constraints. Flow
pins supported provider releases and records compatibility evidence; it does
not require a shared workspace toolchain.

## Relationship to system inventory

`SYSTEM.md` owns capability allocation. This document owns the allowed structural
arrangement that realizes it. Holon architecture files refine their private
structure without overriding suite dependency rules.

## Current implementation boundary

Flow issue #23 / merged PR #24 supplies closed extension-v1 models,
deterministic single-capability resolution, one caller-injected in-process
extension port, Flow-owned event/result validation, and a hermetic no-effects
reference implementation. Issue #26 / merged PR #27 adds deterministic process
request framing and host-neutral transcript validation through that same
acceptance gate. Issue #28 adds a closed scenario manifest, synthetic fixtures,
and canonical digest drift checks; it does not run those scenarios. Issue #36 /
merged PR #37 adds explicit artifact bindings, root-confined host observation,
deterministic file/directory identity, and an opaque accepted artifact set.
Issue #38 adds exact locked package/executable observation and an opaque token
required by both process seams. Issue #40 adds exact process authority and
isolation profiles, caller-attested enforcement evidence, and a second opaque
token required by both process seams. Issue #42 adds the bounded local
trusted-unconfined runner, direct transport workers, timeout/cancellation grace
and escalation, and direct-child reaping. Issue #57 binds an opaque artifact
observation to its exact binding snapshot and canonical root and requires an
equal final observation before acceptance, without changing portable artifact
schemas or free-form extension-result v1 provenance. Issue #58 adds a test-owned
fixed `inspect`-then-`transform` sequencer that runs both single-provider and
two-provider compositions through public resolution, subject observation,
authority, local-process execution, artifact observation, and acceptance. The
second stage consumes the exact accepted first-stage artifact in the same
workspace, and two fresh roots must produce equal portable evidence and output
bytes. This is conformance infrastructure, not a scenario-manifest executor or
production graph scheduler. Issue #49 adds the versioned durable coordinator,
immutable prepared plans, atomic snapshots, accepted checkpoints, fresh reuse
assessment, and explicit recovery decisions described above. Issue #64 requires
fresh prerequisite evidence for graph assessment and dependent execution without
adding a scheduler or rewriting accepted history. Issue #65 adds the bounded
[durable lifecycle corpus](../../integrations/lifecycle-scenarios.md), including
fresh-process restart and deterministic recovery receipts. Issue #66 extends it
with authority denials, observable effect counters, and retained cleanup residuals
using the existing public APIs, without changing runtime schemas. Completed
issue #50 supplies the native read-only Optiflow v0.1.1 adapter described
above; #81 / merged PR #84 strengthens its saved receipt contract and native
artifact correlation. It is a released-provider integration through a library
API and a separate native protocol, not a generic graph checkpoint.

Flow does not yet supply the supported public CLI, released Aniflow or
Renderflow adapters, complete installed-suite workflows, signature or
transparency verification, an atomic filesystem snapshot, an
operating-system sandbox or authenticated enforcement evidence,
descriptor-bound execution, process-tree containment, automatic retry/resume,
or provider-native checkpoint restoration. Structural units beyond these library seams
remain constraints for later adapter and orchestration work, not claims about
current source layout.

## Open questions

- Which public APIs are stable enough for in-process composition in the first
  integrated release?
- How will independent holon versioning map onto suite releases?

## Validation

The [scenario showcase](../../integrations/scenario-showcase.md) is a read-only
documentation projection of existing catalogs and optionally selected normalized
reports. It cannot issue runtime authority, validate native receipt acceptance
or promote a report into current qualification. Raw catalog/report and recorded
source digests keep inspected bytes separate from expected outcomes and claims.
Catalogs remain the owners of IDs, counts, traces and coverage gaps; no second
scenario registry, execution engine or sibling dependency is introduced.

Historical default-branch CI run 35558851556 validated Rust 1.85 and stable library builds,
closed contract models, deterministic resolution, the hermetic in-process and
process-transcript seams, scenario-manifest identity, and repository
architecture metadata plus artifact acceptance and exact execution subjects at
`9cbe58eff5174faa9511a64211e8119e5c7bda6d`. Issue #40 extends that matrix with
authority/isolation schema and semantic validation, canonical identity,
request/grant coverage, every explicit authority dimension, trust routing,
exact sandbox enforcement correlation, context replay, contradictory evidence,
and unsupported-claim checks. Issue #38 previously added
execution-subject schema, canonical identity, filesystem, content-mismatch,
context-replay, contradictory-evidence, and unsupported-claim checks. Forbidden
dependency-edge checks, CLI-thinness, signature/transparency verification,
authenticated sandbox conformance, descriptor-bound launch, process-tree
containment, broader provider-native/domain qualification, and general scenario execution remain
later gates for their corresponding runtime surfaces. Issue #42 adds real Unix
direct-child conformance for literal launch state, supervised transport,
overflow, interruption, grace, escalation, and reaping. Issue #58 additionally
validates deterministic fixed-order composition without adding plan/run state,
retry, checkpoint, resume, or real holon algorithms.

Later merged work adds the durable coordinator and bounded native Optiflow
checks described above. [#81's local receipt](../../validation/flow-81-local.json)
records focused Linux/Rust 1.85.0 regression checks and actual pinned v0.1.1
execution. This is separate from the earlier CI observation and from full-suite
qualification. [#34](https://github.com/egohygiene/flow/issues/34) retains the
observed all-target timeout, native unavailable/skip semantics and moving-stable
Clippy baseline. No current all-target green or release qualification follows
from merging these implementations. ADR-0011, ADR-0012 and ADR-0013 remain
proposed in the [decision index](../governance/DECISIONS.md).

## Optional AI integration inventory (#80)

The proposed [preparation boundary](../../ai/preparation.md) adds an inert
Flow-owned SDK inspection of explicitly selected nonsecret profiles and local
file declarations. It separates matched file bytes from runtime-native model
identity, hardware compatibility, offline enforcement and inference readiness.
It creates no runtime, installer, router, provider authority or shared package.
Remote execution is refused by this first local inspection surface; normal
disabled workflows do not inspect files. Domain adapters remain independent,
and adoption requires a separately reviewed immutable released contract.
See [proposed ADR-0015](../governance/decisions/ADR-0015-ai-preparation.md).

The [dated client inventory](../../ai/README.md) proposes reusing Renderflow's
existing native catalog/profile/skill/summary boundary for the first bounded
text consumer. This is documentation mapping, not a Flow runtime implementation,
ratified suite profile or new shared dependency. Existing extension locks,
per-invocation authority, prelaunch fallback and domain acceptance remain the
owners. Preparation declarations, model/hardware proof and immutable provider
release remain separate. No domain holon imports Flow or sibling source.

## Explicit local preparation delivery

The AI preparation owner derives byte-bound private records only from selected
existing local files and reuses the existing observer for set confirmation.
Portable doctor actions refer to logical assets and fixed remediation categories;
private locations and deployment metadata stay in local preparation records.
A thin provisional setup utility uses this library boundary without becoming
the supported Flow orchestration CLI. Preparation never owns installation,
service startup, runtime-native model verification or invocation authority.
