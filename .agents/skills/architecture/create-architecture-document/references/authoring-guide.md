# Create Architecture Document — Authoring Guide

## Primary Question

    How is this project structurally organized to accomplish its purpose?

## What This Document Owns

- structural units or layers
- responsibility allocation across those units
- boundary rules
- dependency direction
- major communication or coordination patterns
- significant structural constraints

## Keep These Elsewhere

- `SYSTEM.md`: which major systems exist and what each one owns
- `DECISIONS.md`: why a durable choice was accepted
- source code or API docs: implementation detail
- `ROADMAP.md`: future sequencing and commitments

## Strong Output Characteristics

- starts from the system model rather than the source tree
- distinguishes system inventory from structural organization
- explains dependency direction in plain language
- keeps durable constraints explicit
- stays implementation-light and reviewable

## Common Anti-Patterns

- equating the architecture with a framework, package manager, or deployment stack
- restating `SYSTEM.md` without adding structural reasoning
- turning temporary repository layout into permanent truth
- hiding contradictions by forcing a neat but unsupported model

## Review Questions

- Could a contributor explain the structure without opening the code?
- Is the difference between a system and an architectural layer clear?
- Do boundaries and dependency rules reduce ambiguity?
- Are unsupported or contradictory claims labeled instead of normalized?
- Would the document still hold after a large refactor?
