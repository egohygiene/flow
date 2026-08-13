---
schema: aether.architecture-document/v1
id: renderflow-architecture
title: Renderflow Architecture
kind: architecture-document
version: 0.1.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-13
governed_by:
  - architecture-architecture
depends_on:
  - renderflow-system
related:
  - flow-architecture
  - renderflow-decisions
supersedes: []
---

# Renderflow Architecture

## Purpose and scope

Refine the existing workspace shape into durable boundaries for the reusable
engine, delivery CLI, and extension SDK.

## Structural units

- **Core domain:** formats, transforms, graph, optimization policy, plan, build
  state, and domain errors.
- **Core application:** spec validation, registry assembly, planning, cache
  decisions, execution waves, output collection, and build results.
- **Ports/adapters:** filesystem, cache, command runner, renderers, AI providers,
  templates, assets, clock, and events.
- **Plugin SDK:** minimal versioned extension types and executor interface.
- **CLI:** command parsing, configuration locations, presentation, exit mapping,
  watch coordination, and completion.

## Boundary rules

The CLI delegates builds to core. Planner logic is pure with respect to I/O.
Optimization objectives are explicit inputs. Executors do not mutate the graph.
Plugins cannot access private core types. External commands use argv or typed
stdin/stdout modes with declared outputs. AI secrets are resolved from approved
secret sources and never serialized into effective specifications or caches.

## Dependency direction

`renderflow-cli -> renderflow-core -> domain`; adapters implement inward-facing
ports. `renderflow-plugin-sdk` remains a narrow leaf dependency usable without
the CLI. No Aniflow, Optiflow, or Flow orchestration dependency is allowed.

## Communication patterns

Validated specs enter the core; immutable plans, typed events, and structured
results leave it. Content-addressed cache keys include relevant source,
configuration, transform implementation, template, and external tool identity.

## Significant constraints

Parallel waves require deterministic scheduling semantics and collision-free
output paths. Transform cost/quality values are policy inputs, not universal
truth. Remote/non-deterministic transforms require explicit cache and provenance
rules. Plugin compatibility follows semantic versioning and bounded capability.

## Relationship to system inventory

The existing three-crate workspace is aligned with the target shape. Refactoring
should strengthen it rather than introduce a suite-specific facade inside core.

## Assumptions and evidence gaps

Current Rust 1.94 minimum and edition 2021 differ from sibling projects; suite
workspace mechanics remain undecided.

## Open questions

- Should renderer adapters live in core feature modules or separate crates to
  control dependency and toolchain surface?

## Validation

Dependency checks, library examples, plugin compatibility fixtures, planner
property tests, deterministic build tests, and CLI delegation tests enforce the
architecture.
