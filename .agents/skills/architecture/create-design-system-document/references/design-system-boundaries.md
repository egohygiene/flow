# Design System Boundaries

## Semantic System

    primary action
    critical status
    focus state
    supporting text
    elevated surface

## Implementation System

    CSS custom properties
    React components
    Flutter themes
    Figma variables
    native platform resources

`DESIGN_SYSTEM.md` owns the semantic system and behavioral guarantees.
Implementation repositories own the concrete realizations.

## Product Variation

Products may vary:

- typography families
- color mappings
- imagery
- illustration
- motion character
- shape language
- density within documented bounds

Products may not bypass:

- contrast and non-color communication
- focus visibility
- input accessibility
- reduced-motion behavior
- feedback and recovery expectations
- semantic state meaning

## Common Failure Modes

- component inventory without design semantics
- one framework library treated as canonical
- color names used instead of semantic roles
- inaccessible product themes
- local variants with no governance
- identical styling mistaken for coherent experience
