# Federated extension contract

## Scope

This contract defines how Flow discovers, resolves, authorizes, invokes, and
records independently released extensions. It is the reusable substrate for
Flow issue #7. Flow issue #23 implements a bounded subset as a Rust library:
deterministic resolution plus one injected, trusted in-process execution seam
proven with a hermetic reference port. Flow issue #26 adds deterministic
process request encoding and host-neutral validation of caller-supplied
completion, stdout, and stderr evidence. Flow issue #36 adds a separate
Flow-owned artifact-binding, host-observation, and acceptance boundary. Flow
issue #38 adds exact package/executable subject observation before process
request or transcript acceptance. Flow issue #40 adds exact per-invocation
authority and isolation evidence before either process seam. Flow issue #42
adds one bounded local `trusted-unconfined` direct-child runner through those
gates. Product orchestration, real provider adapters, a two-holon vertical
slice, and runtime CLI commands remain work for Flow issue #3 and its later
children.

An extension is an independently versioned provider package. It can expose one
or more Aniflow, Optiflow, Renderflow, Flow, or third-party domain capabilities
without importing sibling source or moving domain logic into Flow.

## Authority split

| Artifact | Owner | Authority |
| --- | --- | --- |
| Extension manifest | Provider | Declares identity, integrity, compatibility, capabilities, requested permissions, hooks, and checkpoint behavior |
| Extension lock | Flow operator | Pins discovered bytes and grants trust, permissions, precedence, and fallback |
| Invocation | Flow | Binds one capability call to immutable inputs, configuration, authorization, limits, and resume evidence |
| Events and result | Provider contribution, Flow validation | Report observations; cannot self-approve suite completion |
| Resolution evidence | Flow | Explains selection, rejection, conflict, replacement, and fallback |
| Artifact bindings | Flow | Bind immutable input and candidate-output IDs to logical ports, media types, kinds, and root-relative locators |
| Host artifact observations | Flow observer | Recompute file/directory content identity beneath one selected root; cannot be supplied by the provider as an acceptance token |
| Execution-subject lock | Flow operator | Pins one exact package directory and executable file to a process provider context |
| Execution-subject observations | Flow observer | Recompute package/executable identities and report digest, lock, publisher-declaration, operator-trust, cryptographic, and transparency claims separately |
| Process-authority profile | Flow caller/operator boundary | Select exact requested/granted authority, trust, and isolation for one invocation |
| Process-enforcement evidence | Future runner/host | Report the exact profile and authority a named backend claims to enforce; not self-authenticating proof |

Manifests are inert data. Flow reads manifests only from explicitly configured
locations and never executes a binary merely because it is on `PATH`.

## Lifecycle

The target lifecycle is:

`discover → inspect → plan → authorize → execute → validate → commit evidence`

| Phase | Required behavior |
| --- | --- |
| Discover | Read configured manifests and locks without loading or executing providers |
| Inspect | Verify schema, identity, publisher, digest, compatibility, and declared entry points |
| Plan | Resolve domain ownership, capability types, effects, conflicts, precedence, and fallback deterministically |
| Authorize | Compare requested permissions and effects with explicit lock grants; emit an authorization identity |
| Execute | Invoke the pinned in-process entry point or direct executable-plus-argv process under declared bounds |
| Validate | Verify structured output, declared artifacts, domain evidence, checkpoints, and permission compliance |
| Commit evidence | Atomically persist the result, resolution, validation, provenance, and reusable partial state |

Skipping, blocking, unavailability, failure, reuse, cancellation, and successful
production remain distinct states. A process exit code or an extension's own
success field is evidence, not suite completion.

## Identity, compatibility, and integrity

- `extension_id`, semantic version, and publisher identify the package.
- SHA-256 identifies the exact installed package bytes.
- Required Flow versions and contract families are explicit.
- `flow_version_requirement` uses the Rust `semver` crate's comma-separated
  comparator syntax. For example, `>=0.1.0, <0.2.0` is valid; the
  whitespace-separated form `>=0.1.0 <0.2.0` is not.
- Unknown major contracts and invalid version ranges are rejected without
  downgrade.
- The executable checkpoint requires the
  `flow.extension-invocation/v1`, `flow.extension-event/v1`, and
  `flow.extension-result/v1` runtime families.
- Capability IDs are globally qualified and name one primary domain owner.
- The lock must match the inspected identity, version, publisher, and digest.
- Process execution additionally requires the package-directory digest to
  equal extension integrity and a separately locked executable-file digest to
  match the selected entrypoint subject. Digest equality is content evidence,
  not publisher authentication.

## Capability and artifact declarations

Each capability declares its domain, accepted artifact/media types, produced
types, configuration schema, machine-readable preconditions and postconditions,
determinism, cacheability, loss behavior, content-changing behavior, and
requested effects. The invocation records the effective configuration digest.
Provider-native domain types remain authoritative inside the holon; Flow
adapters translate only the released boundary.

In issue #23, the caller constructs the invocation and owns configuration
canonicalization, its digest, authorization issuance, the authorization ID, and
the grants digest. Flow validates their shape and correlates fields repeated in
provider evidence; it does not recompute or authenticate those caller-issued
identities.

Inputs and outputs cross the suite boundary as immutable artifact references.
Any content-changing operation produces a new artifact identity and lineage.

The extension envelopes remain path-independent. A separate
`flow.artifact-bindings/v1` document maps their IDs to portable root-relative
locators for one run, and `flow.artifact-observations/v1` records Flow-observed
file or directory identity. Inputs carry an expected SHA-256 digest; outputs
acquire their digest from host observation. The complete profile is specified
in [Artifact bindings](artifact-bindings.md).

Process package and executable identity uses the same root-confined observer
but a separate operator-owned lock and opaque match token. Its exact subject
definitions and non-authenticity claims are specified in
[Execution subjects](execution-subjects.md).

Exact per-invocation authority and isolation evidence remain separate from
subject identity. Their request/grant split, canonical identity, closed
dimensions, and caller-attested evidence boundary are specified in
[Process authority and isolation](authority-isolation.md).
The bounded direct-child lifecycle is specified separately in
[Bounded local process runner](process-runner.md).

## Execution modes

- **In-process:** a version-pinned public library entry point. Flow still binds
  it to the invocation, event, result, authorization, and provenance contracts.
- **Process:** a version-pinned executable invoked directly with argv. The v1
  host-neutral transport encodes one invocation and validates a caller-supplied
  JSON Lines event/result transcript, captured-output lengths, and completion
  observation. Before request encoding or transcript validation, Flow requires
  a fresh opaque token proving exact observation of the locked package and
  executable subjects for that invocation, plus a second opaque token proving
  exact correlation of its authority profile and isolation evidence. A
  local runner now re-observes those bytes, invokes the pinned executable,
  captures stdout/stderr concurrently, and enforces timeout, output bounds, and
  cancellation grace. A real provider adapter must still close the
  observation-to-exec race where supported, validate provider-native outputs,
  and never parse human console text as a contract.

Both modes must produce the same structured semantic result. Flow may choose
process isolation when toolchains, licensing, trust, or failure containment make
an in-process edge unsuitable.

The current executable checkpoint restricts the unconfined in-process seam to
`trusted` candidates. A `sandboxed` process candidate may resolve only to the
downstream authority/isolation preflight, where incomplete or contradictory
evidence fails closed. The checkpoint proves a caller-injected in-process
library port, host-neutral process request/transcript validation, and one local
`trusted-unconfined` direct-child runner. Process mode also observes the exact
locked package and executable content and correlates exact authority/isolation
evidence before either process seam. A post-execution library seam can observe explicitly
bound files and directories beneath a caller-selected root and construct an
opaque accepted artifact set only after binding, host, invocation, event, and
result correlation. It performs no filesystem discovery, dynamic loading,
signature or transparency verification, or operating-system sandbox
enforcement. Caller-attested sandbox evidence is not authenticated proof.
In-process limits remain policy metadata only. The local runner enforces direct-
child timeout, caller cancellation, bounded retained output, and reaping, but it
does not enforce panic isolation, filesystem/network authority, descendant
containment, or other provider side effects.

## Observation and telemetry

The orchestrator wraps the caller's `EventSink` with Flow-owned validation.
Only provider events whose schema, identities, unique event ID, and strictly
increasing sequence pass validation reach the caller sink. The sink is the
fallible authoritative observer for this execution seam, not a best-effort
logging, tracing, metrics, or OpenTelemetry exporter.

Diagnostics marked `redacted: false` in events or terminal results are
rejected. A `redacted: true` value remains a provider assertion: this
checkpoint does not content-scan diagnostics or sanitize unrestricted contract
strings such as result explanations, validation evidence, provenance values,
or artifact, checkpoint, and event references. When `ExecutionError` retains
structured event or result evidence, it remains caller-inspectable but is never
promoted to `ValidatedExecution`. Raw process stdout and stderr remain
caller-owned transcript evidence and are not copied into the error.

Observations are deliberately outside resolution and execution identity. A
sink cannot select an extension, directly mutate provider evidence, or grant
authority. `EventSink::emit` is fallible and its error is visible to an
in-process provider; rejection makes either execution seam return
`ExecutionError` without fallback. A separate best-effort observability seam
belongs to issue #15. This checkpoint includes neither a logging backend nor an
OpenTelemetry exporter.

## Permissions and trust

The manifest requests filesystem reads/writes, environment names, subprocesses,
network hosts, AI providers, GPU access, source mutation, destructive behavior,
signing, and publication. The lock grants a subset and selects `trusted`,
`sandboxed`, or `disabled` operation. No grant is implied by capability
selection.

The executable in-process seam makes only `trusted` candidates eligible.
Process resolution permits `trusted` candidates and `sandboxed` candidates
that must pass the downstream `sandboxed` authority profile. `Orchestrator`
either invokes a caller-injected in-process port or validates a caller-supplied
process transcript. `LocalProcessRunner` launches only an already-authorized
`trusted-unconfined` process and refuses `sandboxed`; it does not operate a
sandbox. `disabled` candidates fail closed.

The process authority profile binds ordered argv, environment names and opaque
secret handles, filesystem reads/writes, network endpoints, subprocesses, AI
providers, GPU access, source mutation, destructive operations, publication,
signing, and telemetry to the exact invocation. Empty collections deny a
dimension and the only v1 ambient policy is `deny-unlisted`. The operator grant
must cover the provider request without widening argv. Both process seams
require `AuthorizedProcess`, which also correlates the host's explicit
enforcement statement. This is contract preflight, not runtime containment.

Secrets are referenced through runtime handles. Providers and callers are
responsible for keeping credentials, tokens, private prompt content, and other
sensitive values out of portable evidence. `discovery.location` is arbitrary
and may be absolute, so manifests, locks, and resolution inputs must be handled
according to their actual sensitivity. The executable seams check the
diagnostic `redacted` flag only; they do not verify the provider's `true`
assertion, inspect diagnostic content, or sanitize other contract strings. Raw
`ExecutionError` events and results are sensitive, nonportable evidence and
must not be exported without caller-controlled inspection and redaction.

## Replacement, conflict, and fallback

An extension may declare replacement intent or fallback eligibility, but the
lock owns effective precedence. Resolution is deterministic:

1. Fail resolution atomically when a manifest is semantically malformed, with
   top-level reasons because resolution-v1 cannot represent that candidate
   without fabricating required evidence.
2. For each well-formed candidate, retain compatibility, authorization,
   availability, precedence, and failed-check reasons.
3. Apply domain ownership and capability compatibility.
4. Apply operator precedence.
5. Fail unresolved equal-precedence conflicts.
6. Use fallback only when a higher-priority or preferred provider is
   unavailable before invocation and the plan explicitly permits fallback;
   never after `ExtensionPort::invoke` begins.

When fallback policy forbids an otherwise technically compatible lower
candidate, resolution retains `compatible: true` and records
`authorized: false`; operator policy does not rewrite technical compatibility.

Selection uses a unique highest numeric precedence. `fallback_order` preserves
the lock's `ordered_extensions` policy order as resolution evidence; it does
not enable fallback after `ExtensionPort::invoke` begins.

Rust semantic validation requires each lock `ordered_extensions` list to follow
non-increasing locked precedence. JSON Schema alone does not express this
ordering invariant.

An entirely empty manifest catalog produces `no-compatible-provider`. If at
least one enabled, `trusted` lock entry listed by the requested policy is
represented by a manifest with exactly matching extension ID, version,
publisher, and integrity, every such entry must have an exact lock-matching
manifest. A partial or substituted set produces an atomic `malformed` outcome
with no candidate evidence. This prevents caller omission or pin substitution
from bypassing preferred-provider or fallback policy.

## Hooks

Observer hooks receive redacted lifecycle events and are read-only. They cannot
modify artifacts, plans, grants, validation, or publication state.

Transform hooks are ordinary declared capabilities. They accept immutable
artifacts, produce new immutable artifacts, and participate in planning,
authorization, validation, checkpointing, and provenance. There are no
unstructured in-place pre/post hooks.

## Checkpoint and resume

An extension declares whether it supports no resume, provider checkpoints, or
Flow checkpoints, plus the checkpoint contract family and compatibility keys.
Reuse requires compatible input digests, extension identity and digest,
capability version, effective configuration, granted permissions, and validated
checkpoint outputs. Incompatible state remains inspectable and is not reused.

## Contract files

The machine-readable schemas live in `contracts/schemas/`:

- `flow.extension-manifest/v1`
- `flow.extension-lock/v1`
- `flow.extension-invocation/v1`
- `flow.extension-event/v1`
- `flow.extension-result/v1`
- `flow.extension-resolution/v1`
- `flow.artifact-bindings/v1`
- `flow.artifact-observations/v1`
- `flow.execution-subject-lock/v1`
- `flow.execution-subject-observations/v1`
- `flow.process-authority-profile/v1`
- `flow.process-enforcement-evidence/v1`

Synthetic examples and compatibility fixtures cover all three holon domains and
explicit compatible, incompatible, over-permissioned, duplicate, and malformed
outcomes. They contain no Ego Hygiene publication content.

## Current proof and deferred runtime

Issue #23 / merged PR #24 adds closed extension-v1 models, semantic validation,
deterministic single-capability resolution, one injected `ExtensionPort`,
correlated event/result validation, and a no-effects, no-artifacts hermetic
reference port. Issue #26 reuses that validation for deterministic request
framing and caller-supplied process transcripts. Issue #36 adds portable
artifact bindings, deterministic host observation, immutable input verification,
and a separate artifact-acceptance token. Issue #38 adds a closed execution-
subject lock, fresh package/executable observation, deterministic canonical
identity, separated evidence claims, and an opaque match token required by both
process seams. Issue #40 adds exact request/grant profiles, explicit trust and
isolation, canonical enforcement evidence, and an opaque authorization token
also required by both process seams. Issue #42 adds exact direct launch,
supervised stdio, deadline/cancellation grace and escalation, and direct-child
reaping. No proof promotes an extension's terminal success report until its
applicable Flow-owned checks pass.

Those checks prove contract validity and correlation only. They do not prove
the authenticity of the caller-issued configuration digest, authorization ID,
or grants digest. The in-process seam does not enforce declared execution
limits; the process seam checks only supplied captured byte counts and
completion evidence. The local runner enforces those direct-child limits but
does not contain filesystem, network, subprocess, or descendant side effects.

For terminal evidence, `produced` and `reused` require every reported
validation to be `passed`; `skipped` permits `passed` or `not-run` but rejects
`failed`. Outcome, failure classification, and failure payload must also be
coherent.

`ValidatedExecution` still proves provider-envelope consistency only. Artifact
acceptance is a subsequent explicit gate: bindings and Flow-created host
observations must cover every declared input and output, match capability media
types, and correlate exactly with invocation, result, and `artifact-produced`
event IDs. Inputs are rehashed against their expected digest; output file and
directory identities come from host-observed bytes. This establishes portable
byte identity beneath the selected root, not provider-native domain validity,
publisher authenticity, or sandbox enforcement.

`extensions list`, `extensions inspect`, `doctor`, real process launching and
capture, launch-time binding to the observed file object, signature and
transparency verification, operating-system sandbox enforcement and
authenticated host evidence, real provider adapters,
domain-output validation, interruption delivery, durable run state,
checkpoints/resume, and the cross-holon vertical slice remain deferred work for
Flow #3 after the required holon contracts are available as releases or
versioned process envelopes.
