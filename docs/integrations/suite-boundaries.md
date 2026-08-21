# Flow suite boundaries

## Ownership

| Holon | Owns | Does not own |
| --- | --- | --- |
| Flow | cross-holon discovery, compatibility, deterministic planning, run state, provenance and validation coordination, failure handling, recovery, suite diagnostics | temporal processing, collection policy, or transform-graph execution |
| Aniflow | temporal media inspection, decomposition, ordered processors, reconstruction, temporal validation, domain checkpoints | collection inventory/policy, generic derivative DAGs, or cross-holon selection |
| Optiflow | collection discovery, identity and relationship evidence, reports, safe optimization plans, normalization policy | temporal reconstruction, generic derivative DAGs, or cross-holon execution |
| Renderflow | spec-driven transform graph planning/execution, document publication, image/audio conversion, templates, plugins, build cache/results | collection mutation policy, temporal reconstruction, or cross-holon selection |

If a capability has no clear owner, Flow reports it as unavailable until an
architecture change names one. Flow does not implement a convenient duplicate.

## Allowed dependency directions

| Consumer | Provider | Allowed seam |
| --- | --- | --- |
| Flow CLI | Flow core | internal library API |
| Flow core | Flow adapter port | internal library API |
| Flow library adapter | released holon library | published, version-constrained API |
| Flow process adapter | released holon CLI | direct argv plus versioned structured I/O |
| Holon CLI | its own library | provider-owned API |

All other suite edges are denied unless an ADR changes this table. In
particular:

- holons do not depend on Flow or sibling holons;
- Flow does not copy provider source or use sibling path dependencies;
- Git submodules and mutable branch/revision dependencies are forbidden; and
- provider-native contracts remain provider-owned and are translated by
  Flow-owned adapters.

## Contract ownership

Flow owns the suite-level artifact, capability, and compatibility schemas.
Their v1 major identifier names a compatibility family; the manifest's semantic
version records the provisional contract-set revision. A provider does not claim
native Flow compatibility merely because an adapter can translate it.

Breaking schema changes require a new major identifier or an explicit migration.
Unknown versions are rejected; there is no silent downgrade.

## Enforcement evidence

- Architecture documents and ADRs declare the same dependency graph.
- Contract validation checks manifest/schema/example consistency.
- Adapter tests pin provider releases and exercise structured outputs.
- Dependency audits reject copied source, path dependencies, submodules, and
  holon-to-holon or holon-to-Flow edges.
