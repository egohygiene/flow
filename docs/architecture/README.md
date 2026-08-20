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
10. [Personal model](domain/PERSONAL_MODEL.md)
11. [System](foundation/SYSTEM.md)
12. [Architecture](foundation/ARCHITECTURE.md)
13. [Design](experience/DESIGN.md)
14. [Design system](experience/DESIGN_SYSTEM.md)
15. [Methodology](foundation/METHODOLOGY.md)
16. [Decisions](governance/DECISIONS.md)
17. [Meta architecture](meta/META.md)
18. [Roadmap](../../ROADMAP.md)

## Holon contracts

- [Aniflow](holons/aniflow/README.md)
- [Optiflow](holons/optiflow/README.md)
- [Renderflow](holons/renderflow/README.md)

Each contract contains `PURPOSE.md`, `ONTOLOGY.md`, `SYSTEM.md`,
`ARCHITECTURE.md`, and `DECISIONS.md`. Suite-level principles, foundations,
epistemology, design, methodology, AI governance, and roadmap apply to every
holon and are not duplicated.

## Complete-reference applicability

`PERSONAL_MODEL.md` is applicable even though Flow does not model identity:
the suite processes user-selected artifacts and must make assumptions about
agency, consent, attention, privacy, and recovery explicit.

`DESIGN_SYSTEM.md` defines semantic states and cross-surface language for the
CLI, documentation, plans, reports, diagrams, and future graphical interfaces.
It does not prematurely select a visual framework or component library.
