# Federated extension contract

## Scope

This contract defines how Flow discovers, resolves, authorizes, invokes, and
records independently released extensions. It is the reusable substrate for
Flow issue #7. Product orchestration, a real two-holon vertical slice, and
runtime CLI commands belong to Flow issue #3.

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

The normative lifecycle is:

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
- Unknown major contracts and invalid version ranges are rejected without
  downgrade.
- Capability IDs are globally qualified and name one primary domain owner.
- The lock must match the inspected identity, version, publisher, and digest.

## Capability and artifact declarations

Each capability declares its domain, accepted artifact/media types, produced
types, configuration schema, machine-readable preconditions and postconditions,
determinism, cacheability, loss behavior, content-changing behavior, and
requested effects. The invocation records the effective configuration digest.
Provider-native domain types remain authoritative inside the holon; Flow
adapters translate only the released boundary.

Inputs and outputs cross the suite boundary as immutable artifact references.
Any content-changing operation produces a new artifact identity and lineage.

## Execution modes

- **In-process:** a version-pinned public library entry point. Flow still binds
  it to the invocation, event, result, authorization, and provenance contracts.
- **Process:** a version-pinned executable invoked directly with argv. Flow
  independently captures stdout/stderr, enforces timeout, output bounds,
  cancellation grace, permissions, and declared outputs, and never parses human
  console text as a contract.

Both modes must produce the same structured semantic result. Flow may choose
process isolation when toolchains, licensing, trust, or failure containment make
an in-process edge unsuitable.

## Permissions and trust

The manifest requests filesystem reads/writes, environment names, subprocesses,
network hosts, AI providers, GPU access, source mutation, destructive behavior,
signing, and publication. The lock grants a subset and selects `trusted`,
`sandboxed`, or `disabled` operation. No grant is implied by capability
selection.

Secrets are referenced through runtime handles and never serialized. Absolute
paths, credentials, tokens, prompts containing private content, and raw
unredacted diagnostics do not enter portable evidence.

## Replacement, conflict, and fallback

An extension may declare replacement intent or fallback eligibility, but the
lock owns effective precedence. Resolution is deterministic:

1. Reject malformed, incompatible, disabled, integrity-mismatched, and
   over-permissioned candidates.
2. Apply domain ownership and capability compatibility.
3. Apply operator precedence.
4. Fail unresolved equal-precedence conflicts.
5. Record every rejected candidate and reason.
6. Use fallback only when the selected provider is unavailable and the plan
   explicitly permits fallback; never silently after partial side effects.

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

## Deferred runtime proof

This issue freezes the reusable data and policy boundary. `extensions list`,
`extensions inspect`, `doctor`, provider adapters, interruption/resume execution,
and the cross-holon vertical slice are implementation work for Flow #3 after the
required holon contracts are available as releases or versioned process
envelopes.
