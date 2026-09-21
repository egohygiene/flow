# Locked package and executable subjects

## Purpose and scope

This profile defines the exact package directory and regular executable file
that may support Flow's process-mode request and transcript seams. It adds a
fresh content-integrity premise before either seam can accept evidence; it does
not launch a process or authenticate a publisher.

The profile is governed by
[ADR-0008](../architecture/governance/decisions/ADR-0008-locked-execution-subjects.md)
and composes the root-confined observation rules from
[ADR-0007](../architecture/governance/decisions/ADR-0007-root-confined-artifact-acceptance.md).

## Contracts

| Contract | Owner | Purpose |
| --- | --- | --- |
| `flow.execution-subject-lock/v1` | Flow operator | Pin one package, one executable, and the process context in which they are eligible |
| `flow.execution-subject-observations/v1` | Flow observer | Record exact observed identities and separate evidence claims |
| `MatchedExecutionSubjects` | Flow runtime | Prove that a fresh observation matched the exact lock and invocation context |

Both JSON contracts are closed. Unknown fields and unsupported claim values are
errors. A portable observation document is inspectable evidence only;
deserialization cannot construct the opaque runtime token.

## Exact subjects

The lock contains exactly two ordered subjects:

1. **Package:** a directory at `package.locator`, relative to one
   caller-selected observation root. Its digest uses
   `flow.directory-manifest/v1`. The digest must also equal the locked and
   resolved extension integrity.
2. **Executable:** a regular file at
   `package.locator/executable.locator`. The lock stores its locator relative
   to the package; observations store the joined root-relative locator. Its
   file bytes receive a separate SHA-256 digest. This locator is the exact file
   a future runner must invoke directly; the correlated manifest entrypoint is
   not authority to search `PATH` or substitute another file.

The executable is therefore covered twice: once as a child in the package
manifest and once as the explicit executable subject. Its kind, locator,
digest, and byte count must agree in both representations.

Subject IDs use `package:<id>` and `executable:<id>`. Locators use the portable
profile from the artifact-binding contract. Absolute paths, traversal,
backslashes, host-ambiguous paths, symlinks, special nodes, non-UTF-8
descendants, missing targets, and wrong node kinds fail closed.

## Bound process context

The subject lock is valid only for one exact combination of:

- extension-lock ID;
- extension ID, version, declared publisher ID, and package integrity;
- capability ID;
- process interface name and `flow.extension-invocation/v1` protocol;
- declared entrypoint; and
- package and executable subjects.

Flow correlates that context with the resolved extension and invocation before
reading content. The opaque match token additionally binds the subject lock's
canonical digest, run ID, invocation ID, and configured trust mode. Reusing a
token with a modified lock or another invocation fails process preflight.

## Deterministic identity

`flow.canonical-json/v1` recursively sorts object member names, preserves array
order and scalar values, emits compact UTF-8 JSON without a trailing newline,
and hashes those bytes with SHA-256. Both the subject lock and observation set
expose deterministic canonical bytes and digests. This profile is intentionally
named rather than presented as RFC 8785.

Package directory identity remains the distinct
`flow.directory-manifest/v1` algorithm documented by
[Artifact bindings](artifact-bindings.md).

## Evidence claims

The observation document never collapses different trust questions into one
boolean:

| Claim | Required v1 value | What it establishes |
| --- | --- | --- |
| `content_digest` | `observed`, `sha256` | Flow read content and calculated identities |
| `lock_equality` | `matched` plus canonical lock digest | Both observed subjects equal the supplied lock |
| `cryptographic_verification` | `not-performed`, empty evidence | No signature, certificate, key, or attestation was checked |
| `publisher_identity` | `declared-and-correlated` | Provider declarations repeat the same publisher ID |
| `operator_trust` | `configured` plus lock trust | The operator lock supplied policy, not objective authenticity |
| `transparency_log` | `not-checked`, empty evidence | No inclusion, checkpoint, or consistency proof was checked |

Digest equality alone must never be described as publisher authentication.
Likewise, configured operator trust must never be described as cryptographic
verification. Future signature or transparency-log evidence requires a new
contract version and an actual verifier. V1 rejects nonempty evidence and
stronger unsupported statuses.

## Observation and process gates

The intended order is:

1. Resolve a trusted process provider through the extension manifest and
   operator lock.
2. Construct and validate the execution-subject lock for that exact resolution
   and invocation.
3. Observe the package directory and executable file beneath the selected
   root, rejecting unsafe nodes and any digest mismatch.
4. Construct `MatchedExecutionSubjects` only after exact correlation.
5. Correlate the separate exact authority/isolation profile and enforcement
   evidence, constructing `AuthorizedProcess` only after both agree with these
   matched subjects.
6. Require the same lock and both tokens when encoding the process request.
7. Require the same lock and both tokens again before validating a supplied
   process transcript.

The current library performs steps 1–7 without implementing the operating-
system actions between request encoding and transcript validation. The caller
still supplies the transcript and completion observation.

## Failure behavior

Subject handling distinguishes invalid locks and invocations, context mismatch,
unsafe or unavailable filesystem content, exact content mismatch,
contradictory observation evidence, and canonicalization failure. All failures
occur before `ValidatedExecution` is constructed. A provider result, exit `0`,
or otherwise valid JSON Lines transcript cannot override a failed subject gate.

Duplicate subjects, repeated IDs, wrong canonical order, a missing executable
inside the package manifest, or disagreement between package-manifest and
standalone executable evidence is contradictory and rejected.

## Guarantees and non-goals

This profile guarantees that the freshly observed package and executable bytes
exactly match the supplied operator-controlled lock and that the match belongs
to the exact resolved process invocation context.

It does not guarantee:

- that the expected digest came from an authentic publisher;
- signature, certificate, attestation, provenance, or transparency-log
  verification;
- that configured operator trust is correct;
- that bytes remain unchanged after observation;
- that a later launcher opens the same file object;
- interpreter, dynamic-library, environment, or transitive dependency
  identity;
- executable permission or platform loadability checks;
- executable discovery, `PATH` search, argv construction, or process launch;
- timeout, cancellation, output capture, signal, or reap enforcement;
- filesystem, environment, subprocess, network, AI, GPU, signing, publication,
  or other authority isolation;
- durable plans, run state, checkpoints, retry, or resume; or
- real Aniflow, Optiflow, or Renderflow adapters.

A production runner must close the observation-to-launch race and apply the
[authority/isolation profile](authority-isolation.md) without weakening this
evidence boundary. The current profile validates caller-attested enforcement
evidence but does not itself launch or sandbox the executable.

## Conformance

Rust and contract fixtures cover positive deterministic matching and
adversarial altered bytes, mismatched provider/version/publisher/capability,
wrong interface/entrypoint, missing or escaping targets, wrong kinds, symlinks,
special nodes, duplicate or contradictory evidence, unsupported verification
claims, and stale lock/invocation tokens. The hermetic process example observes
synthetic package bytes and exercises both process preflight gates without
launching an external executable.
