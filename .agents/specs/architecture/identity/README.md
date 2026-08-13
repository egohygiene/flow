# Architecture Identity Specifications

The identity family establishes the enduring foundation for a repository,
product, platform, or organization.

## Documents

| Document | Primary question | Responsibility |
| --- | --- | --- |
| `PURPOSE.md` | Why do we exist? | Reason, beneficiaries, and enduring value |
| `VISION.md` | What future do we seek? | Desired long-term future state |
| `PRINCIPLES.md` | How should decisions be made? | Durable decision heuristics |
| `PILLARS.md` | What must remain strong? | Enduring strategic capabilities |
| `MANIFESTO.md` | What do we stand for? | Public beliefs and commitments |

## Default Authoring Order

    purpose
        ↓
    vision
        ↓
    principles
        ↓
    pillars
        ↓
    manifesto

## Boundary Rules

- Purpose is not a feature list.
- Vision is not a roadmap.
- Principles are not coding standards or policies.
- Pillars are not projects or system components.
- Manifesto is not advertising copy or governance policy.

## Corresponding Skills

    purpose.spec.md
        ↔ create-purpose-document

    vision.spec.md
        ↔ create-vision-document

    principles.spec.md
        ↔ create-principles-document

    pillars.spec.md
        ↔ create-pillars-document

    manifesto.spec.md
        ↔ create-manifesto-document

The `architecture-authoring` skill coordinates the complete sequence.
