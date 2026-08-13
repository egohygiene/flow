# meta architecture specifications

The meta architecture family governs how knowledge is justified, how AI systems
participate, and how the architecture-document system itself is understood and
maintained.

## Documents

| Document | Primary question | Canonical responsibility |
| --- | --- | --- |
| `EPISTEMOLOGY.md` | How do we know what we claim to know? | Evidence, provenance, confidence, uncertainty, and conflict handling |
| `AI_CONSTITUTION.md` | How may AI systems participate? | AI authority, behavior, safety, oversight, and escalation |
| `META.md` | How is the architecture system organized? | Architecture inventory, graph, ownership, navigation, and evolution |

## Authoring sequence

    EPISTEMOLOGY
        ↓
    AI_CONSTITUTION
        ↓
    META

`EPISTEMOLOGY.md` defines the knowledge rules used by people and AI systems.
`AI_CONSTITUTION.md` applies those rules to AI participation. `META.md` then
maps the architecture system and its governance without duplicating the content
of individual documents.

## Boundary rules

- Epistemology governs how claims are evaluated, not which conclusions must be
  accepted.
- The AI constitution governs behavior and authority, not provider-specific
  prompts or model configuration.
- Meta architecture maps the document system, not the product or runtime
  architecture itself.
- None of these documents should duplicate policies, decisions, or
  implementation details owned elsewhere.

## Corresponding skills

    epistemology.spec.md
        ↔ create-epistemology-document

    ai-constitution.spec.md
        ↔ create-ai-constitution-document

    meta.spec.md
        ↔ create-meta-architecture-document

## Change propagation

A substantive epistemology change requires reviewing AI governance, research,
knowledge extraction, documentation, and decision records.

A substantive AI-constitution change requires reviewing agents, prompts,
instructions, orchestration, automation, permissions, and approval boundaries.

A substantive meta-architecture change requires reviewing the architecture
catalog, navigation, relationship graph, bundles, and validation tooling.
