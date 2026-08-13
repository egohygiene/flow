---
schema: aether.architecture-document/v1
id: renderflow-system
title: Renderflow System
kind: architecture-document
version: 0.1.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-13
governed_by:
  - architecture-system
depends_on:
  - flow-foundations
  - renderflow-ontology
related:
  - renderflow-architecture
supersedes: []
---

# Renderflow System

## Purpose and scope

Renderflow decomposes a derivative build into configuration, registry, planning,
execution, rendering, extension, caching, and result systems.

## System inventory

| Subsystem | Owns |
| --- | --- |
| Specification | parsing, defaults, variables, targets, transforms, policy, validation |
| Format/transform registry | supported representations, directed edges, requirements, executor identity |
| Graph planner | path selection, multi-target merge, shared intermediates, waves, plan diagnostics |
| Transform runtime | in-memory/native/command/AI transform execution and failure policy |
| Renderers | target-specific publication and conversion adapters |
| Templates/assets | resolution, validation, rendering context, output packaging |
| Plugin boundary | stable extension types, registration, compatibility, isolation policy |
| Build cache | content/config/implementation identity and reusable node results |
| Build result | outputs, plan, timing, cache decisions, warnings, failures, validation |

## Responsibilities and capability ownership

Renderflow owns the transform graph and derivative build. Pandoc, Tectonic,
FFmpeg, AI providers, and plugins own specialized execution behind adapters.
Flow may compose a Renderflow build but does not choose internal transform paths.

## System boundaries

The core library contains planning and execution semantics. The CLI is delivery.
The plugin SDK is narrower and more stable than core internals. External tools
are probed and invoked safely. Watch mode is a repeated build coordinator, not a
second engine.

## Major interactions and runtime flows

Load/validate specification → resolve formats/transforms → plan merged DAG →
evaluate cache → execute waves → validate/collect outputs → emit build result.

## External system relationships

Current adapters include Pandoc, Tectonic, FFmpeg, Ollama/OpenAI-compatible
services, external commands, and runtime plugins. Remote AI is optional and must
respect suite privacy/authority policy.

## Assumptions and evidence gaps

Current v0.2.1 supports documents plus broad image/audio conversions and already
has `renderflow` core, CLI, and plugin-SDK crates. Structured build-result and
suite capability coverage need verification.

## Open questions

- Which existing public API surfaces are stable enough to retain unchanged
  during consolidation?

## Validation

Graph, path, wave, cache, renderer, plugin, failure-mode, and multi-target fixture
tests validate subsystem contracts.
