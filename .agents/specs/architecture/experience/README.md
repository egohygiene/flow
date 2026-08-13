# Architecture Experience Specifications

The experience document family defines how a product, platform, repository,
or organization should respond to people through design.

It separates enduring experiential intent from the reusable language used to
express that intent consistently.

## Documents

| Document | Primary question | Canonical responsibility |
| --- | --- | --- |
| `DESIGN.md` | What kind of experience should this create? | Experience philosophy, qualities, values, and human impact |
| `DESIGN_SYSTEM.md` | How should that experience be expressed consistently? | Reusable visual, interaction, content, motion, and accessibility language |

## Authoring Sequence

    PERSONAL_MODEL
        ↓
    DESIGN
        ↓
    DESIGN_SYSTEM

## Boundary Rules

- `DESIGN.md` defines desired experience, not components or tokens.
- `DESIGN_SYSTEM.md` defines reusable design language, not implementation
  libraries or product-specific screens.
- Visual identity may vary by product while accessibility and experience
  commitments remain traceable to the shared philosophy.
- Accessibility is a foundational constraint, not an optional style layer.
- Motion, imagery, tone, and interaction patterns must preserve human agency
  and respect reduced-motion, cognitive-load, and sensory needs.
- A design system may enable variation without flattening product identity.

## Corresponding Skills

    design.spec.md
        ↔ create-design-document

    design-system.spec.md
        ↔ create-design-system-document

## Change Propagation

A substantive `DESIGN.md` change requires review of:

- `DESIGN_SYSTEM.md`
- product visual-identity documents
- interaction patterns
- accessibility guidance
- content and voice guidance
- AI-generated experience rules

A substantive `DESIGN_SYSTEM.md` change requires review of:

- design tokens
- component libraries
- product sites
- documentation themes
- visual-regression baselines
- accessibility tests
- downstream implementation adapters
