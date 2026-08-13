# Canonical Agent Frontmatter Contract

This document defines the required and optional frontmatter fields for every
`AGENT.md` in `.agents/agents/`. It is the authoritative
reference for contributors, validators, and projection builders.

---

## Specification source

Aether agents conform to the GitHub custom-agent profile format. The GitHub
documentation for repository agents is
<https://docs.github.com/en/copilot/customizing-copilot/using-github-copilot-extensions/about-github-copilot-agents>
and for organization agents is defined in the same source. Hosts that load
agent profiles include GitHub Copilot (repository and organization scopes)
and VS Code with the Copilot extension.

---

## Fields

### `aether-id` (required)

- Type: string
- Constraints:
  - lowercase kebab-case (`[a-z][a-z0-9-]*[a-z0-9]` or single letter)
  - at most 64 characters
  - **must match the parent directory name exactly**
- This field is stripped from generated projections.

### `name` (required)

- Type: string
- Human-readable display name (may include spaces and mixed case).
- Used verbatim in generated GitHub projections.

### `description` (required)

- Type: string
- Concise routing description: what the agent does and when to invoke it.
- Non-empty, at most 1,024 characters.

### `tools` (required)

- Type: list of strings
- Explicit least-privilege tool list. Must not be omitted or left as `[]`
  unless the intended policy is no tools. Omitting the field on some hosts
  implies broad access; this contract requires the field to be explicit.
- Accepted values (GitHub Copilot): `read`, `search`, `edit`, `execute`,
  `web`. Preview-only and host-specific values must be isolated in a
  compatibility profile.

### `metadata` (required)

- Type: YAML mapping
- All Aether-specific lifecycle and relationship data. Uses the `aether-`
  prefix. This entire block is stripped from generated projections.

#### Supported `metadata` keys

| Key | Type | Description |
|---|---|---|
| `aether-version` | string (semver) | Agent version in Aether catalog |
| `aether-status` | string | Lifecycle state: `draft`, `stable`, `deprecated`, `retired` |
| `aether-scope` | string | `organization` or `repository` |
| `aether-domain` | string | Primary domain: `architecture`, `authoring`, `quality`, `publishing`, `methodology` |
| `aether-owners` | string | Owning GitHub org or username |
| `aether-created` | string (`YYYY-MM-DD`) | Date first created |
| `aether-updated` | string (`YYYY-MM-DD`) | Date of most recent significant change |
| `aether-skills` | list of strings | Canonical skill IDs this agent depends on |
| `aether-specs` | list of strings | Canonical spec IDs this agent references |

---

## Body structure (required sections)

Each `AGENT.md` body must contain the following sections in order:

1. `## Mission` — single-sentence role statement
2. `## Operating contract` — linked canonical skills and specs with paths relative to
   `library/organization/` (e.g. `../skills/<domain>/<skill>/SKILL.md`), and
   governing specifications.
3. `## Workflow` — numbered ordered procedure
4. `## Boundaries` — explicit non-responsibilities and prohibited behaviors
5. `## Completion` — definition of done

---

## Projection rules

Projection is performed by `build-projections.py`. For each source `AGENT.md`:

1. The `aether-id` field is removed.
2. The entire `metadata` block is removed.
3. Internal skill links (``../skills/<domain>/<skill>/SKILL.md``) are rewritten
   to the consumer-local path ``.agents/skills/<skill>/SKILL.md`` for both
   repository and organization projections.
4. Internal spec links (``../specs/<path>/<file>.spec.md``) are rewritten to
   ``.agents/specs/<path>/<file>.spec.md`` for repository projections and
   ``specs/<path>/<file>.spec.md`` for organization projections.

---

## Lifecycle rules

- Only `stable` agents may be published to the organization release package.
- `draft` agents may appear in repository projections for internal validation.
- Promotion from `draft` to `stable` requires human review.

---

## Tool and permission policy

- Architecture and planning roles (`architect`, `specfile-creator`,
  `implementation-planner`, `github-issue-creator`) must not include
  `edit` (except for documentation artifacts) or `execute`.
- Audit roles (`auditor`) must not include `edit` or `execute`; audit must remain non-destructive.
- Implementation-capable roles (`bug-fix-teammate`, `cleanup-specialist`,
  `test-specialist`) may include `edit` and `execute` for their bounded work.
- Publishing roles (`arxiv-publisher`) may include `edit` and `execute` when
  the role requires building and validating release artifacts.
- `web` is permitted when the role requires live documentation lookup.
- MCP server credentials must not be embedded in agent profiles.
- Omitting the `tools` field implies broad access on some hosts; this contract requires the field to be explicit.

---

## Handoff model

The canonical bounded handoff flow is:

```
Architect
  -> Specfile Creator
  -> Implementation Planner
  -> GitHub Issue Creator
  -> implementation by Copilot / default implementer
  -> Test Specialist
  -> Auditor
```

This is guidance, not a mandatory autonomous loop. Human review remains the
authority between stages.
