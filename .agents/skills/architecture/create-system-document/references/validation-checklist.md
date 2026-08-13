# SYSTEM.md Validation Checklist

## Specification

- [ ] `architecture-system` is identified as the governing specification.
- [ ] The applicable specification version is known.
- [ ] Required upstream purpose, foundations, and domain documents have been read.

## Primary Question

- [ ] The document clearly answers:
      What systems make up this project, and what does each one own?

## Boundaries

- [ ] Every major system has a clearly stated purpose.
- [ ] Responsibilities and capability ownership are explicit.
- [ ] System boundaries are conceptual rather than source-tree or framework driven.
- [ ] Architectural layers, detailed APIs, and deployment topology stay out unless briefly referenced.

## Evidence

- [ ] The system inventory is grounded in durable capability evidence.
- [ ] Assumptions are labeled.
- [ ] Missing evidence is acknowledged.
- [ ] Contradictions or overlapping ownership remain visible.

## Durability

- [ ] The decomposition remains useful across implementation changes.
- [ ] The inventory is small enough to understand.
- [ ] High-level interactions and external relationships are understandable to humans and agents.

## Completion

- [ ] Acceptance criteria from `architecture-system` pass.
- [ ] Downstream review implications are recorded.
- [ ] Markdown, links, and metadata pass repository checks.
