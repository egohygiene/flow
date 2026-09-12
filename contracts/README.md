# Flow contract set

This directory contains Flow-owned suite interchange contracts. The initial
contract set is version `0.2.0`, status `provisional`, in the v1 compatibility
family. Provisional means versioned and testable, not stable for production.

| Contract | Purpose |
| --- | --- |
| `flow.artifact/v1` | immutable artifact identity and production lineage |
| `flow.capability/v1` | provider capability discovery and side-effect declaration |
| `flow.compatibility/v1` | deterministic compatibility decision and evidence |
| `flow.extension-manifest/v1` | provider identity, integrity, compatibility, capabilities, requested permissions, hooks, and checkpoint behavior |
| `flow.extension-lock/v1` | operator-controlled discovery pins, trust, grants, precedence, and fallback |
| `flow.extension-invocation/v1` | immutable inputs, effective authorization, execution bounds, and resume references |
| `flow.extension-event/v1` | ordered lifecycle progress, diagnostics, artifacts, checkpoints, and state |
| `flow.extension-result/v1` | partial/final outcomes, failures, validation, provenance, and explanation |
| `flow.extension-resolution/v1` | deterministic selection, rejection, conflict, and fallback evidence |

`contract-set.v1.json` is the machine-readable index. Schemas live in
`schemas/`, deterministic examples live in `examples/`, and extension
compatibility fixtures live in `fixtures/extensions/`. Invalid fixtures are
expected to fail their target schema and are checked by the validator.

The extension contracts are described in
[`docs/integrations/extension-contract.md`](../docs/integrations/extension-contract.md).
Provider manifests request behavior; only the operator-controlled lock grants
authority, precedence, or fallback.

Flow adapters translate provider-native documents into these contracts. Provider
repositories do not depend on this directory. Breaking changes require a new
major schema identifier; additive compatible changes increment the contract-set
semantic version. Consumers reject unknown major identifiers and never silently
downgrade.

Validate the set with:

```console
python3 tools/validate_contracts.py
```
