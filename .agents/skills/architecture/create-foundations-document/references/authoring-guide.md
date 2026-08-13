# Create Foundations Document — Authoring Guide

## Primary Question

    What enduring truths does the architecture depend on?

## What This Document Owns

- durable assumptions
- architectural axioms
- invariants
- baseline conceptual constraints
- enduring mental models used for reasoning

## Keep These Elsewhere

- `PRINCIPLES.md`: how decisions should be made
- `SYSTEM.md`: what systems exist and what they own
- `ARCHITECTURE.md`: how systems are organized structurally
- `METHODOLOGY.md`: how work is performed and improved
- `ROADMAP.md`: how the project should evolve over time

## Strong Output Characteristics

- uses a small number of durable foundations
- distinguishes assumptions from goals, principles, and decisions
- makes invariants explicit when later work relies on them
- stays implementation-independent
- explains when a foundation is provisional or under revision

## Updating Falsified Assumptions

When evidence disproves a prior foundation:

1. state the old assumption explicitly
2. describe the evidence that broke confidence in it
3. remove or revise the foundation rather than quietly rewriting history
4. mark any replacement as provisional until it is genuinely durable
5. identify downstream documents that need review

## Common Anti-Patterns

- treating a tool choice as a foundational truth
- using `FOUNDATIONS.md` as a second principles document
- filling the document with project plans or workflow rules
- copying system or structural detail that belongs downstream

## Review Questions

- Would these claims still matter after a major implementation rewrite?
- Is each foundation something downstream documents can safely rely on?
- Are contested or falsified assumptions visible?
- Did any principle, decision, or tactic accidentally become a foundation?
- Is the document concise enough to stay stable and reusable?
