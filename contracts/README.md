# Flow contract set

This directory contains Flow-owned suite interchange contracts. The initial
contract set is version `0.1.0`, status `provisional`, in the v1 compatibility
family. Provisional means versioned and testable, not stable for production.

| Contract | Purpose |
| --- | --- |
| `flow.artifact/v1` | immutable artifact identity and production lineage |
| `flow.capability/v1` | provider capability discovery and side-effect declaration |
| `flow.compatibility/v1` | deterministic compatibility decision and evidence |

`contract-set.v1.json` is the machine-readable index. Schemas live in
`schemas/`, and deterministic examples live in `examples/`.

Flow adapters translate provider-native documents into these contracts. Provider
repositories do not depend on this directory. Breaking changes require a new
major schema identifier; additive compatible changes increment the contract-set
semantic version. Consumers reject unknown major identifiers and never silently
downgrade.

Validate the set with:

```console
python3 tools/validate_contracts.py
```
