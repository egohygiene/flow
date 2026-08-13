# Flow Architecture

The architecture set defines one suite-level policy system and three nested,
independently useful holons. Suite documents own shared intent and rules. Holon
documents refine those rules into tool-specific capability and dependency
boundaries.

## Reading order

1. [Purpose](identity/PURPOSE.md)
2. [Vision](identity/VISION.md)
3. [Principles](identity/PRINCIPLES.md)
4. [Pillars](identity/PILLARS.md)
5. [Manifesto](identity/MANIFESTO.md)
6. [Epistemology](meta/EPISTEMOLOGY.md)
7. [AI constitution](meta/AI_CONSTITUTION.md)
8. [Foundations](foundation/FOUNDATIONS.md)
9. [Ontology](domain/ONTOLOGY.md)
10. [System](foundation/SYSTEM.md)
11. [Architecture](foundation/ARCHITECTURE.md)
12. [Design](experience/DESIGN.md)
13. [Methodology](foundation/METHODOLOGY.md)
14. [Decisions](governance/DECISIONS.md)
15. [Meta architecture](meta/META.md)
16. [Roadmap](../../ROADMAP.md)

## Holon contracts

- [Aniflow](holons/aniflow/README.md)
- [Optiflow](holons/optiflow/README.md)
- [Renderflow](holons/renderflow/README.md)

Each contract contains `PURPOSE.md`, `ONTOLOGY.md`, `SYSTEM.md`,
`ARCHITECTURE.md`, and `DECISIONS.md`. Suite-level principles, foundations,
epistemology, design, methodology, AI governance, and roadmap apply to every
holon and are not duplicated.

## Intentional omissions

- `PERSONAL_MODEL.md`: Flow processes user-selected local artifacts but does
  not currently model people, identities, relationships, or inferred intent.
- `DESIGN_SYSTEM.md`: interaction philosophy is defined, but a reusable visual
  component/token system would be speculative before an interface exists.

Both omissions must be revisited if the product begins modeling people or gains
a substantial graphical interface.
