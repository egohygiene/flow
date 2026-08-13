# Specification Metadata Schema — `aether.specification/v1`

This document describes the canonical metadata schema for all specification
files in the Aether library. Every `.spec.md` file in
`.agents/specs/` must open with a YAML frontmatter block that
conforms to this schema.

---

## Schema identifier

```
schema: aether.specification/v1
```

---

## Required fields

| Field     | Type               | Description                                                        |
|-----------|--------------------|--------------------------------------------------------------------|
| `schema`  | string             | Must be `aether.specification/v1`.                                 |
| `id`      | string             | Stable, unique kebab-case identifier for this specification.       |
| `title`   | string             | Human-readable name.                                               |
| `kind`    | string             | Artifact kind. Typically `specification`.                          |
| `version` | string (semver)    | Semantic version (e.g. `1.0.0`, `2.1.0`).                        |
| `status`  | string             | Lifecycle status: `draft`, `stable`, `deprecated`, or `retired`.  |
| `owners`  | list of strings    | One or more owner handles (e.g. GitHub org or username).           |
| `created` | date (`YYYY-MM-DD`)| Date the specification was first created.                          |
| `updated` | date (`YYYY-MM-DD`)| Date of the most recent significant change.                        |
| `domain`  | string             | Top-level domain this specification belongs to (e.g. `architecture`). |
| `tags`    | list of strings    | Searchable tags.                                                   |

---

## Optional relationship fields

| Field        | Type             | Semantic                                                                 |
|--------------|------------------|--------------------------------------------------------------------------|
| `applies_to` | list of strings  | Descriptive artifact categories this specification governs. Not validated against the ID catalog. |
| `depends_on` | list of spec IDs | Normative prerequisites. Must resolve to known spec IDs. Must remain acyclic. |
| `related`    | list of IDs      | Non-normative cross-references to spec IDs or skill IDs. May be cyclic.  |
| `supersedes` | list of spec IDs | Specs that this spec replaces. Must resolve to known or intentionally retired IDs. |

---

## Relationship rules

- **`depends_on`** is a required prerequisite. All targets must resolve to
  known spec IDs. The dependency graph must remain acyclic.
- **`related`** is non-normative and informational. Targets may reference
  spec IDs or skill IDs in the combined catalog. Cycles are permitted.
  Do not add a `related` target that does not exist in the catalog.
- **`supersedes`** must point to an existing or intentionally retired
  identity. Do not use it to reference a spec that has never existed.
- **`applies_to`** is a plain-language description of what artifact categories
  the spec governs (e.g. `architecture-documents`). It is not validated
  against the ID catalog.

---

## Valid example

```yaml
---
schema: aether.specification/v1
id: architecture-system
title: System Document Specification
kind: specification
version: 1.1.0
status: draft
owners:
  - egohygiene
created: 2026-07-18
updated: 2026-08-02
domain: architecture
tags:
  - architecture
  - specification
  - systems
  - decomposition
  - boundaries
applies_to:
  - architecture-documents
depends_on:
  - architecture-document
  - architecture-ontology
related:
  - architecture-architecture
  - architecture-foundations
supersedes: []
---
```

---

## Invalid examples

### Missing required field

```yaml
---
schema: aether.specification/v1
id: my-spec
title: My Specification
# ERROR: missing kind, version, status, owners, created, updated, domain, tags
---
```

### Wrong schema value

```yaml
---
schema: aether.specification/v0   # ERROR: not the accepted schema URI
id: my-spec
---
```

### Unresolvable depends_on target

```yaml
---
schema: aether.specification/v1
id: my-spec
# ...
depends_on:
  - nonexistent-spec-id   # ERROR: no spec with this ID exists
---
```

### Path reference in skill

```yaml
# In SKILL.md — ERROR: use stable spec ID, not a file path
specs:
  - .agents/specs/architecture/foundation/architecture-document
```

The correct form:

```yaml
# In SKILL.md — correct
specs:
  - architecture-architecture
```

---

## Migration notes (legacy → v1)

The five foundation specifications in
`.agents/specs/architecture/foundation/` previously used a legacy
frontmatter shape with only `title`, `version`, `date_created`, `last_updated`,
`owner`, and `tags`.

**Migration steps applied (PR: refactor/contracts):**

1. Replace `title: Document Specification — <NAME>.md` with:
   - `schema: aether.specification/v1`
   - `id: architecture-<name>` (stable ID from the architecture graph)
   - `title: <Name> Document Specification`
   - `kind: specification`
2. Convert `version: 1.1` to semver `version: 1.1.0`.
3. Convert `date_created` / `last_updated` to `created` / `updated`.
4. Convert `owner: Ego Hygiene` to `owners: [egohygiene]`.
5. Add `domain`, `applies_to`, `depends_on`, `related`, `supersedes`.

**Stable IDs assigned:**

| File                         | Stable ID                   |
|------------------------------|-----------------------------|
| `architecture.spec.md`       | `architecture-architecture` |
| `foundations.spec.md`        | `architecture-foundations`  |
| `methodology.spec.md`        | `architecture-methodology`  |
| `roadmap.spec.md`            | `architecture-roadmap`      |
| `system.spec.md`             | `architecture-system`       |

---

## Validation

Run the deterministic validation script from the repository root:

```sh
python3 .agents/specs/validate-specs.py
```

This will be superseded by `aether validate --format text` once issue 007
delivers the canonical validator.
