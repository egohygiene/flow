# Architecture Domain Specifications

This document family defines the conceptual world in which a repository,
product, platform, or organization operates.

| Document | Primary question | Canonical responsibility |
| --- | --- | --- |
| `ONTOLOGY.md` | What exists in this domain? | Concepts, terms, relationships, and boundaries |
| `PERSONAL_MODEL.md` | How does the project understand the people it serves? | Human assumptions, agency, identity, context, consent, and inference boundaries |

## Authoring Sequence

    ONTOLOGY
        ↓
    PERSONAL_MODEL

## Boundaries

- Ontology is not a class diagram, API schema, or database model.
- Personal model is not a persona library, diagnosis, or prediction engine.
- Both documents distinguish evidence, assumptions, and inference.
- A person is never reduced to a profile, record, or optimization target.

## Corresponding Skills

    ontology.spec.md
        ↔ create-ontology-document

    personal-model.spec.md
        ↔ create-personal-model-document
