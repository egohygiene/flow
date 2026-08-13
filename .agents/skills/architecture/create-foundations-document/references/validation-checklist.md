# FOUNDATIONS.md Validation Checklist

## Specification

- [ ] `architecture-foundations` is identified as the governing specification.
- [ ] The applicable specification version is known.
- [ ] Required upstream identity documents have been read.

## Primary Question

- [ ] The document clearly answers:
      What enduring truths does the architecture depend on?

## Boundaries

- [ ] Foundations are durable assumptions, invariants, or baseline constraints.
- [ ] Principles, methodology, roadmap, and system decomposition are not used as substitutes.
- [ ] Framework, library, or vendor choices are excluded unless truly foundational.
- [ ] Repeated detail that belongs in `SYSTEM.md` or `ARCHITECTURE.md` is absent.

## Evidence

- [ ] Each foundation is supported by upstream intent, durable evidence, or clearly labeled assumption.
- [ ] Invariants are explicit where relevant.
- [ ] Missing evidence is acknowledged.
- [ ] Contradictions or recently falsified assumptions remain visible.

## Durability

- [ ] The document survives ordinary implementation churn.
- [ ] Temporary engineering choices are not elevated into canonical truths.
- [ ] The document is useful to downstream system, architecture, methodology, and roadmap work.

## Completion

- [ ] Acceptance criteria from `architecture-foundations` pass.
- [ ] Downstream review implications are recorded.
- [ ] Markdown, links, and metadata pass repository checks.
