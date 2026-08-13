# ARCHITECTURE.md Validation Checklist

## Specification

- [ ] `architecture-architecture` is identified as the governing specification.
- [ ] The applicable specification version is known.
- [ ] Required upstream documents, especially `SYSTEM.md` and `FOUNDATIONS.md`, have been read.

## Primary Question

- [ ] The document clearly answers:
      How is this project structurally organized to accomplish its purpose?

## Boundaries

- [ ] Structural units or layers are explicit.
- [ ] Boundaries and dependency direction are clear.
- [ ] `SYSTEM.md` inventory is not duplicated as the whole architecture.
- [ ] `DECISIONS.md` rationale, deployment detail, and implementation minutiae stay out unless briefly referenced.

## Evidence

- [ ] Structural claims are grounded in upstream documents or observable repository evidence.
- [ ] Assumptions are labeled.
- [ ] Missing evidence is acknowledged.
- [ ] Contradictions remain visible.

## Durability

- [ ] The architecture remains meaningful across framework or implementation changes.
- [ ] Temporary repository layout does not define the architecture by itself.
- [ ] Communication patterns and constraints are understandable to humans and agents.

## Completion

- [ ] Acceptance criteria from `architecture-architecture` pass.
- [ ] Downstream review implications are recorded.
- [ ] Markdown, links, and metadata pass repository checks.
