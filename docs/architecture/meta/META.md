---
schema: aether.architecture-document/v1
id: flow-meta
title: Flow Meta Architecture
kind: architecture-document
version: 0.1.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-19
governed_by:
  - architecture-meta
depends_on:
  - flow-epistemology
  - flow-ai-constitution
related:
  - flow-decisions
supersedes: []
---

# Flow Meta Architecture

## Architecture system overview

Flow uses a federated document model. Suite documents canonically own shared
intent, principles, foundations, language, system boundaries, experience,
method, governance, and strategy. Holon documents refine only domain-specific
purpose, vocabulary, capability inventory, structure, and decisions.

## Document categories

| Category | Canonical concern |
| --- | --- |
| Identity | why Flow exists, desired future, decision principles, strengths, beliefs |
| Meta | evidence rules, AI authority, and this document system |
| Foundation | enduring assumptions, systems, structure, and working method |
| Domain | suite-level canonical concepts and relationships |
| Experience | human interaction philosophy and quality |
| Governance | significant decision lineage |
| Strategy | capability evolution in the root roadmap |
| Holons | tool-specific refinements that inherit suite policy |

## Document inventory

| ID | Concern | Status |
| --- | --- | --- |
| `flow-purpose` through `flow-manifesto` | shared identity | Draft |
| `flow-epistemology` | evidence and claim handling | Draft |
| `flow-ai-constitution` | AI authority and safety | Draft |
| `flow-foundations` | enduring assumptions and invariants | Draft |
| `flow-ontology` | suite language | Draft |
| `flow-personal-model` | agency, consent, privacy, and human assumptions | Draft |
| `flow-system` | capability ownership | Draft |
| `flow-architecture` | structural and dependency rules | Draft |
| `flow-design` | interaction philosophy | Draft |
| `flow-design-system` | semantic language across CLI, docs, reports, and future UI | Draft |
| `flow-methodology` | evolution method | Draft |
| `flow-decisions` and ADRs | decision lineage | Draft/accepted records |
| `flow-roadmap` | strategic sequencing | Draft |
| `<holon>-purpose/ontology/system/architecture/decisions` | local refinement | Draft |

## Canonical materializations

| File | Artifact ID | Repository path |
| --- | --- | --- |
| `PURPOSE.md` | `flow-purpose` | `docs/architecture/identity/PURPOSE.md` |
| `VISION.md` | `flow-vision` | `docs/architecture/identity/VISION.md` |
| `PRINCIPLES.md` | `flow-principles` | `docs/architecture/identity/PRINCIPLES.md` |
| `PILLARS.md` | `flow-pillars` | `docs/architecture/identity/PILLARS.md` |
| `MANIFESTO.md` | `flow-manifesto` | `docs/architecture/identity/MANIFESTO.md` |
| `EPISTEMOLOGY.md` | `flow-epistemology` | `docs/architecture/meta/EPISTEMOLOGY.md` |
| `AI_CONSTITUTION.md` | `flow-ai-constitution` | `docs/architecture/meta/AI_CONSTITUTION.md` |
| `ONTOLOGY.md` | `flow-ontology` | `docs/architecture/domain/ONTOLOGY.md` |
| `PERSONAL_MODEL.md` | `flow-personal-model` | `docs/architecture/domain/PERSONAL_MODEL.md` |
| `FOUNDATIONS.md` | `flow-foundations` | `docs/architecture/foundation/FOUNDATIONS.md` |
| `SYSTEM.md` | `flow-system` | `docs/architecture/foundation/SYSTEM.md` |
| `ARCHITECTURE.md` | `flow-architecture` | `docs/architecture/foundation/ARCHITECTURE.md` |
| `METHODOLOGY.md` | `flow-methodology` | `docs/architecture/foundation/METHODOLOGY.md` |
| `DESIGN.md` | `flow-design` | `docs/architecture/experience/DESIGN.md` |
| `DESIGN_SYSTEM.md` | `flow-design-system` | `docs/architecture/experience/DESIGN_SYSTEM.md` |
| `DECISIONS.md` | `flow-decisions` | `docs/architecture/governance/DECISIONS.md` |
| `ROADMAP.md` | `flow-roadmap` | `ROADMAP.md` |
| `META.md` | `flow-meta` | `docs/architecture/meta/META.md` |

## Canonical ownership map

Identity, evidence policy, AI rules, shared foundations, suite ontology, cross-
holon ownership, dependency direction, suite experience, methodology, and
roadmap exist exactly once at suite level. A holon owns specialized terms,
private subsystems, internal layering, and local decisions. In conflict, the
suite boundary governs unless a newer suite ADR explicitly delegates it.

## Relationship graph

```text
purpose -> vision -> principles -> pillars -> foundations
                     |                         |
                     -> epistemology -> ontology -> personal model -> system -> architecture
                                      |                    |
                                      -> AI constitution   -> methodology
personal model -> design -> design system
                                                           -> decisions
architecture + methodology + vision ----------------------> roadmap
suite system/architecture -------------------------------> holon contracts
```

## Reading order

Read the suite documents in the order listed in `docs/architecture/README.md`,
then the contract for the holon being changed, then applicable ADRs and roadmap
horizons.

## Authoring order

Update the upstream canonical owner first. Propagate terminology, boundary, and
dependency changes into downstream suite documents and then affected holons.
Roadmap changes follow accepted intent or architecture changes.

## Lifecycle and validation status

All initial documents are draft because implementation and migration evidence
is incomplete. ADR status is independent. A document becomes stable only after
its downstream contracts and representative implementation evidence agree.

## Change propagation

- Purpose/vision changes require review of all documents.
- Principles/foundations changes require system, architecture, design,
  methodology, decision, holon, and roadmap review.
- Ontology changes require schemas, APIs, system, and holon language review.
- System ownership changes require architecture, affected holons, decisions,
  and roadmap review.
- Holon changes require suite review only when a public capability or boundary
  changes.

## Gaps and intentional omissions

No suite-level Aether document is intentionally omitted. Personal-model content
governs human agency without creating a persona or identity schema; design-system
content governs semantic cross-surface language without selecting a framework.

Tool-specific vision, principles, foundations, experience, AI, methodology, and
roadmaps still inherit suite documents to prevent duplicated policy. Migration
and capability matrices are roadmap artifacts, not missing architecture nodes.

## Open questions

- Should future holons use focused contracts or a smaller manifest when their
  internal complexity is low?

## Validation

Validation checks unique IDs, resolvable dependencies, acyclic relationships,
one H1, required frontmatter, canonical ownership, links, and visible omissions.
