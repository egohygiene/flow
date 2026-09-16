# Federated extension contract

## Scope

This contract defines how Flow discovers, resolves, authorizes, invokes, and
records independently released extensions. It is the reusable substrate for
Flow issue #7. Flow issue #23 implements a bounded subset as a Rust library:
deterministic resolution plus one injected, trusted in-process execution seam
proven with a hermetic reference port. Product orchestration, real provider
adapters, a two-holon vertical slice, and runtime CLI commands remain work for
Flow issue #3 and its later children.

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

## Execution modes

- **In-process:** a version-pinned public library entry point. Flow still binds
  it to the invocation, event, result, authorization, and provenance contracts.
- **Process (future target):** a version-pinned executable invoked directly
  with argv. A future process adapter must independently capture stdout/stderr,
  enforce timeout, output bounds, cancellation grace, permissions, and declared
  outputs, and never parse human console text as a contract.

Both modes must produce the same structured semantic result. Flow may choose
process isolation when toolchains, licensing, trust, or failure containment make
an in-process edge unsuitable.

The current executable checkpoint resolves only `trusted` candidates and proves
only a caller-injected in-process library port. It performs no filesystem
discovery, dynamic loading, external process invocation, executable integrity
verification, or sandbox enforcement. `sandboxed` candidates fail closed.
Declared limits are validated and correlated with the resolved mode as policy
metadata only; Flow does not enforce a timeout, cancellation, stdout/stderr
bound, panic isolation, filesystem/network containment, or any other side
effect on the injected code. The process behavior above remains a future
contract target, not a claim about the implemented runtime.

## Observation and telemetry

The orchestrator wraps the caller's `EventSink` with Flow-owned validation.
Only provider events whose schema, identities, unique event ID, and strictly
increasing sequence pass validation reach the caller sink. The sink is the
intended attachment point for future human logging, structured logging,
tracing, metrics, or OpenTelemetry adapters.

Diagnostics marked `redacted: false` in events or terminal results are
rejected. A `redacted: true` value remains a provider assertion: this
checkpoint does not content-scan diagnostics or sanitize unrestricted contract
strings such as result explanations, validation evidence, provenance values,
or artifact, checkpoint, and event references. Rejected raw evidence remains
available through `ExecutionError`, but it is never forwarded to the caller
sink or returned in `ValidatedExecution`.

Observations are deliberately outside resolution and execution identity. A
sink cannot select an extension, directly mutate provider evidence, or grant
authority. `EventSink::emit` is fallible and its error is visible to the
provider; rejection makes `Orchestrator` return `ExecutionError`. This
checkpoint includes neither a logging backend nor an OpenTelemetry exporter.

## Permissions and trust

The manifest requests filesystem reads/writes, environment names, subprocesses,
network hosts, AI providers, GPU access, source mutation, destructive behavior,
signing, and publication. The lock grants a subset and selects `trusted`,
`sandboxed`, or `disabled` operation. No grant is implied by capability
selection.

Issue #23 makes only `trusted` candidates resolution-eligible, while
`Orchestrator` accepts only a caller-injected in-process port. It marks
`sandboxed` candidates unauthorized and unavailable because it has no
enforceable sandbox backend; `disabled` candidates also fail closed.

Secrets are referenced through runtime handles. Providers and callers are
responsible for keeping credentials, tokens, private prompt content, and other
sensitive values out of portable evidence. `discovery.location` is arbitrary
and may be absolute, so manifests, locks, and resolution inputs must be handled
according to their actual sensitivity. Issue #23 checks the diagnostic
`redacted` flag only; it does not verify the provider's `true` assertion,
inspect diagnostic content, or sanitize other contract strings. Raw
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

Synthetic examples and compatibility fixtures cover all three holon domains and
explicit compatible, incompatible, over-permissioned, duplicate, and malformed
outcomes. They contain no Ego Hygiene publication content.

## Current proof and deferred runtime

Issue #23 adds a library-only proof: closed extension-v1 models, semantic
validation, deterministic single-capability resolution, one injected
`ExtensionPort`, correlated event/result validation, and a no-effects,
no-artifacts hermetic reference port. The proof does not promote an extension's
terminal success report to a validated execution until Flow-owned checks pass.

Those checks prove contract validity and correlation only. They do not prove
the authenticity of the caller-issued configuration digest, authorization ID,
or grants digest, nor do they enforce the declared execution limits or contain
side effects from the trusted injected code.

For terminal evidence, `produced` and `reused` require every reported
validation to be `passed`; `skipped` permits `passed` or `not-run` but rejects
`failed`. Outcome, failure classification, and failure payload must also be
coherent.

Artifact handling is identifier self-consistency only. Invocation input IDs
must be unique, result `consumed_artifacts` must be a subset of those IDs, and
references on `artifact-produced` events must appear in terminal
`produced_artifacts`; generic event references remain unrestricted. This does
not bind identifiers to locators, recompute or verify artifact digests or
bytes, establish artifact existence, or perform domain-output validation.

`extensions list`, `extensions inspect`, `doctor`, process transport, sandbox
enforcement, real provider adapters, interruption/cancellation, durable run
state, checkpoints/resume, and the cross-holon vertical slice remain deferred
work for Flow #3 after the required holon contracts are available as releases
or versioned process envelopes.
