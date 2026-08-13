# Create System Document — Authoring Guide

## Primary Question

    What systems make up this project, and what does each one own?

## What This Document Owns

- system inventory
- system purpose
- responsibility allocation
- capability ownership
- major conceptual boundaries
- high-level system context and interactions

## Keep These Elsewhere

- `ARCHITECTURE.md`: how systems are structurally organized
- source code and API docs: implementation-level detail
- infrastructure docs: deployment topology and operations
- `ROADMAP.md`: future sequencing and capability rollout

## Strong Output Characteristics

- starts from capabilities before naming systems
- assigns one primary owner for each capability
- keeps system boundaries durable and conceptual
- explains major interactions without drowning in implementation detail
- makes overlaps and uncertainties visible

## Common Anti-Patterns

- mirroring the package tree as the system model
- using framework services or vendors as default system boundaries
- creating too many tiny systems to be useful
- blending architectural-layer rules into system inventory

## Review Questions

- Could a reader name the major systems and what each owns?
- Are overlapping responsibilities resolved or at least visible?
- Is the decomposition independent of the current source layout?
- Did architectural-layer or API detail leak in from adjacent concerns?
- Would this inventory still make sense after a major refactor?
