# Artifact binding and host-observation profile

## Purpose and scope

This profile binds path-independent extension artifact identifiers to one
caller-selected filesystem root, observes the bound bytes with Flow-owned code,
and correlates those observations with resolved capability, invocation, event,
and result evidence.

It is governed by
[ADR-0007](../architecture/governance/decisions/ADR-0007-root-confined-artifact-acceptance.md).
The implementation is a library boundary for Flow issue #36. It does not launch
a provider, grant filesystem authority, enforce a sandbox, validate provider-
native document semantics, or commit durable run state. Exact package and
executable content is a separate pre-execution boundary documented in
[Execution subjects](execution-subjects.md).

## Evidence layers

| Layer | Type or contract | Claim |
| --- | --- | --- |
| Declaration | `flow.artifact-bindings/v1` | Flow intends these immutable inputs and candidate outputs to occupy these logical ports and root-relative locators |
| Portable observation document | `flow.artifact-observations/v1` | A serializable description of content identity and directory structure; deserialization alone does not prove who observed it |
| Flow observation token | `ObservedArtifactSet` | Flow's observer produced the contained document from one binding set and root during this process |
| Provider evidence | `ValidatedExecution` | Provider events and result passed the extension contract and correlation gate; this is not filesystem acceptance |
| Accepted artifacts | `AcceptedArtifactSet` | Flow correlated a complete provider outcome with the resolved context, immutable bindings, and Flow-observed host evidence |

Both `ObservedArtifactSet` and `AcceptedArtifactSet` have private fields. A
caller can serialize or deserialize the portable observation contract for
inspection, but cannot turn deserialized or provider-authored JSON directly
into either token. Acceptance requires a fresh result from `observe_artifacts`.

## Binding contract

`flow.artifact-bindings/v1` contains one `binding_set_id`, the fixed
`sha256` algorithm, and separate `inputs` and `outputs`. The combined set must
not be empty.

Every binding has:

- a globally unique `artifact_id` within the set;
- a globally unique logical `port` within the set;
- a concrete `type/subtype` media type rather than a wildcard;
- `file` or `directory` kind;
- a globally unique portable locator beneath the selected root; and
- for inputs, a 64-character lowercase SHA-256 `expected_digest`.

Input digests are immutable preconditions. Output digests are deliberately not
declared by the provider or caller before execution; they are learned from
Flow's host observation.

## Portable locator profile

A locator is a UTF-8, slash-separated relative path. Each segment must be
nonempty and cannot be `.` or `..`. The whole locator cannot:

- start with `/`;
- contain `\`;
- contain `:`; or
- resolve to anything other than normal relative path components.

Segments also reject ASCII controls, Windows device stems (`CON`, `PRN`,
`AUX`, `NUL`, `COM1`–`COM9`, and `LPT1`–`LPT9`), trailing spaces or periods,
and the host-ambiguous characters `"`, `*`, `<`, `>`, `?`, and `|`.

This rejects Unix absolute paths, traversal, repeated separators, Windows drive
prefixes, UNC/backslash ambiguity, reserved device paths, and URI-like prefixes
before any path is joined. Spaces and Unicode are valid and preserved
byte-for-byte in UTF-8. No Unicode normalization or case folding occurs.

## Root-confined observation

`observe_artifacts(root, bindings)` applies this sequence:

1. Validate the closed binding contract and its uniqueness invariants.
2. Require `root` to exist as a directory and reject it if it is a symlink.
3. Canonicalize the root once.
4. Join each already-validated locator one segment at a time. Inspect every
   segment with symlink-aware metadata and reject any symlink or missing node.
5. Canonicalize the complete bound target and require it to remain beneath the
   canonical root.
6. Recursively observe the target without following links. Reject symlinks,
   sockets, devices, FIFOs, other special nodes, and non-UTF-8 descendant
   names.
7. Validate the generated observation document before returning the opaque
   `ObservedArtifactSet`.

Observation is read-only, but it reads every bound file. Callers must provide a
quiescent, isolated root. This portable implementation is not an atomic
filesystem snapshot and cannot prevent a hostile concurrent path swap.

## Content identity

### Files

A file digest is lowercase hexadecimal SHA-256 over its exact bytes. Its
`size_bytes` is the exact file length. File observations have an empty
`manifest`.

### Directories

`flow.directory-manifest/v1` computes each directory recursively. Direct
children are sorted by exact UTF-8 name. Each child is encoded as this compact
JSON object with the shown field order:

```json
{"name":"child","kind":"file","digest":"<64 lowercase hex>","size_bytes":12}
```

The directory representation is the compact JSON array of those child objects,
with no trailing newline. Its SHA-256 digest is the directory digest. The empty
directory representation is `[]`. A directory's `size_bytes` is the checked sum
of all descendant file lengths.

The portable observation contains a flattened manifest of all descendants,
including directory entries, sorted by their slash-separated locator. Every
directory entry repeats its recursively computed digest and byte count. The
validator reconstructs each directory identity from the flattened manifest, so
reordering, omission, insertion, kind changes, and digest or size tampering are
rejected.

Timestamps, permissions, ownership, extended attributes, allocation size,
inodes, and enumeration order do not enter identity.

## Acceptance gate

`accept_artifacts(resolved, invocation, execution, bindings, observed)` returns
an `AcceptedArtifactSet` only after all inputs agree:

1. The invocation is valid and still matches the resolved extension identity,
   capability, execution interface, lock, configuration schema, expected output
   types, and limits.
2. The opaque `ValidatedExecution` result still matches that invocation.
3. The result is complete rather than partial and has outcome `produced` or
   `reused`.
4. Invocation input IDs and digests exactly match the input bindings.
5. Bound input media types match the resolved capability's accepted patterns.
6. Bound output media types match both the capability and invocation expected
   output types, and every expected output type has at least one candidate
   binding.
7. Provider-consumed IDs exactly match inputs; provider-produced IDs exactly
   match outputs; and `artifact-produced` event references name every output
   exactly once.
8. The Flow observation covers every binding exactly once and matches its ID,
   port, media type, kind, and locator.
9. Every host-observed input digest equals its expected immutable digest.

Set comparison rejects omissions, additions, and duplicates. No individual
success report, event, exit code, or existing path bypasses the combined gate.

## Typical library sequence

```rust,ignore
let execution = Orchestrator::execute(resolved, &invocation, port, event_sink)?;
let observed = observe_artifacts(&workspace_root, &bindings)?;
let accepted = accept_artifacts(
    resolved,
    &invocation,
    &execution,
    &bindings,
    &observed,
)?;
```

A future process runner uses the same final two calls after
`validate_process_transcript` returns `ValidatedExecution`. The runner still
owns safe workspace creation, child authority, concurrent output capture,
timeout/cancellation enforcement, executable integrity, and quiescence before
observation.

## Failure surfaces

`ArtifactObservationError` separates invalid bindings, invalid generated
observations, unsafe roots, missing nodes, symlinks, kind mismatch, unsupported
nodes, non-UTF-8 names, canonical escape, I/O failure, manifest encoding, and
size overflow.

`ArtifactAcceptanceError` separates invalid invocation, bindings, or
observation contracts from cross-evidence mismatch. Error display text names
the failed invariant but does not include artifact contents.

Errors return no accepted token. Raw filesystem contents and rejected portable
observation documents remain caller-controlled evidence.

## Conformance evidence

The Rust suite covers repeated deterministic observation, files, nested
directories, spaces, Unicode, immutable input changes, traversal and absolute
locators, backslash and drive ambiguity, symlinks, tampered manifests,
incomplete and mismatched observations, invocation mismatch, duplicate provider
artifact events, and partial results. JSON Schema plus the independent Python
validator cover the checked-in examples and semantic-invalid fixtures.

Run the full local gates with:

```console
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --locked
cargo test --doc --locked
python3 tools/validate_contracts.py
python3 tools/generate_scenario_sources.py --check
```

## Deferred boundaries

This profile does not itself provide executable/package integrity. The separate
execution-subject profile provides exact digest matching but not publisher
authentication, signature or transparency verification, launch-time file
binding, operating-system permission enforcement, sandboxing, process
launch/supervision, resource quotas during directory traversal, domain-specific
artifact validation, durable provenance/state commit, checkpoint compatibility,
retry, or resume. Those claims require their own Flow #25 and orchestration
checkpoints.
