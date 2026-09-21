# Flow contract set

This directory contains Flow-owned suite interchange contracts. The initial
contract set is version `0.5.0`, status `provisional`, in the v1 compatibility
family. Provisional means versioned and testable, not stable for production.

| Contract | Purpose |
| --- | --- |
| `flow.artifact/v1` | immutable artifact identity and production lineage |
| `flow.artifact-bindings/v1` | Flow-owned immutable input and candidate-output bindings to logical ports and portable root-relative locators |
| `flow.artifact-observations/v1` | deterministic host-observed file and directory identity for one binding set |
| `flow.capability/v1` | provider capability discovery and side-effect declaration |
| `flow.compatibility/v1` | deterministic compatibility decision and evidence |
| `flow.execution-subject-lock/v1` | operator-controlled exact package, executable, provider-context, and evidence-policy lock |
| `flow.execution-subject-observations/v1` | Flow-observed package/executable identity with separate digest, lock, publisher, trust, cryptographic, and transparency claims |
| `flow.extension-manifest/v1` | provider identity, integrity, compatibility, capabilities, requested permissions, hooks, and checkpoint behavior |
| `flow.extension-lock/v1` | operator-controlled discovery pins, trust, grants, precedence, and fallback |
| `flow.extension-invocation/v1` | immutable inputs, authorization identity, declared execution-limit metadata, and resume references |
| `flow.extension-event/v1` | ordered lifecycle progress, diagnostics, artifacts, checkpoints, and state |
| `flow.extension-result/v1` | partial/final outcomes, failures, validation, provenance, and explanation |
| `flow.extension-resolution/v1` | deterministic selection, rejection, conflict, and fallback evidence |
| `flow.scenario-manifest/v1` | stable scenario identity, immutable fixture topology, typed expectations, execution budgets, and bounded coverage claims |

`contract-set.v1.json` is the machine-readable index. Schemas live in
`schemas/`; deterministic examples live in `examples/`; extension compatibility
fixtures live in `fixtures/extensions/`; execution-subject adversarial fixtures
live in `fixtures/execution-subjects/`; and orchestration fixtures live in
`fixtures/scenarios/`. Invalid fixtures are expected to fail their target
schema or semantic invariants and are checked by the validator.

The extension contracts are described in
[`docs/integrations/extension-contract.md`](../docs/integrations/extension-contract.md).
Provider manifests request behavior; only the operator-controlled lock grants
authority, precedence, or fallback.

Process framing reuses the existing invocation, event, and result documents
rather than introducing a wrapper contract. The provisional JSON Lines wire
rules and host-neutral acceptance boundary are specified in
[`docs/integrations/process-transport.md`](../docs/integrations/process-transport.md).
Wire whitespace and caller read chunking are not contract identity.

Artifact path binding remains separate from the path-independent extension
envelopes. The root-relative locator, SHA-256 file identity, recursive directory
manifest, Flow-observation token, and final correlation gate are specified in
[`docs/integrations/artifact-bindings.md`](../docs/integrations/artifact-bindings.md).
A deserialized observation document is inspectable evidence, not proof that
Flow observed those bytes and not an artifact-acceptance token.

Exact process package and executable identity is specified in
[`docs/integrations/execution-subjects.md`](../docs/integrations/execution-subjects.md).
`flow.execution-subject-lock/v1` binds both subjects to one process provider
context. `flow.execution-subject-observations/v1` records fresh content and lock
equality while explicitly reporting that cryptographic verification and
transparency checks were not performed. A portable document cannot construct
the opaque match token required by process request and transcript validation.

The scenario manifest and its canonical identity profile are described in
[`docs/integrations/scenario-fixtures.md`](../docs/integrations/scenario-fixtures.md).
The Rust and Python implementations independently reproduce the checked-in
SHA-256 catalog. CI validates drift but never regenerates or rewrites canonical
fixtures.

`compatibility.flow_version_requirement` is parsed with Rust's `semver`
`VersionReq` grammar and must use comma-separated comparators. For example,
`>=0.1.0, <0.2.0` is a bounded range; `>=0.1.0 <0.2.0` is invalid. Invalid or
unsatisfied requirements are explicit compatibility failures and are never
silently weakened.

Flow adapters translate provider-native documents into these contracts. Provider
repositories do not depend on this directory. Breaking changes require a new
major schema identifier; additive compatible changes increment the contract-set
semantic version. Consumers reject unknown major identifiers and never silently
downgrade.

The current Rust checkpoint maps the six extension-v1 documents, the scenario
manifest, both artifact-boundary documents, and both execution-subject documents
into closed library models and adds semantic checks that JSON Schema alone does
not express. Its hermetic in-process reference, host-neutral process transcript,
local artifact observer, and exact package/executable observer are not provider
adapters and do not prove process launch/capture enforcement, publisher
authenticity, signatures, transparency, domain-output validity, sandboxing,
checkpoints, or resume.

For this checkpoint, the caller owns configuration canonicalization and digest
generation plus authorization issuance, authorization ID, and grants digest.
Flow shape-checks these values and correlates the identities repeated by the
provider; `ValidatedExecution` does not authenticate them. The process
transcript validator checks already captured stdout/stderr lengths and a caller
completion observation. That is not runtime timeout, cancellation, capture,
panic, sandbox, or side-effect enforcement.

Validate the set with:

```console
python3 tools/validate_contracts.py
python3 tools/generate_scenario_sources.py --check
```
