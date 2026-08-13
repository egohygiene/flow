---
schema: aether.architecture-document/v1
id: flow-design
title: Flow Design
kind: architecture-document
version: 0.1.0
status: draft
owners:
  - egohygiene
created: 2026-08-13
updated: 2026-08-13
governed_by:
  - architecture-design
depends_on:
  - flow-purpose
  - flow-vision
  - flow-principles
related:
  - flow-ontology
  - flow-architecture
supersedes: []
---

# Flow Design

## Design philosophy

Flow should feel like a careful studio assistant: calm before work, explicit
about consequences, quiet during success, precise during failure, and always
able to show its evidence.

## Intended experience

Users move through a consistent loop: inspect, plan, approve meaningful effects,
process, validate, and review. Standalone holon CLIs use the same vocabulary and
result concepts even when they expose deeper domain controls.

## Experience qualities

- **Legible:** stage ownership, tool selection, effects, and artifacts are clear.
- **Predictable:** planning and execution agree; fallback is never silent.
- **Calm:** routine progress is concise and stable.
- **Recoverable:** interruption and failure lead to an exact next safe action.
- **Scriptable:** human presentation and structured output are equally first-class.

## Interaction philosophy

Default commands perform the least surprising safe action. Potentially
destructive, remote, expensive, or provenance-signing actions are explicit.
Planning does not perform expensive processing. Dry runs never mutate user data.

## Communication philosophy

Use canonical nouns and stage states. Errors say what failed, what remains
valid, what evidence was retained, and how to continue. Warnings are actionable
and deduplicated. JSON has no ANSI output or interleaved prose.

## Accessibility philosophy

Do not encode state through color alone. Support non-interactive terminals,
screen-readable messages, stable text snapshots, bounded line length where
practical, and progress that degrades to discrete events.

## Agency and meaningful control

Users control sources, output roots, intermediate retention, remote services,
provenance signing, and mutation. Configuration precedence is explainable and
the effective result can be inspected before execution.

## Cognitive load

Offer a coherent suite surface for common workflows and preserve expert holon
surfaces for specialized work. Do not mirror every underlying flag in Flow;
expose typed capability configuration instead.

## Trust, feedback, and recovery

Status distinguishes planned, preparing, running, validating, completed,
skipped, failed, and cancelled. A failed run retains partial artifacts, reusable
checkpoints, interpreted causes, raw logs, and an exact resume command.

## Aesthetic direction

Text output favors restrained hierarchy, aligned summaries, meaningful icons
only where they remain accessible, and no decorative animation that obscures
state. The suite may give each holon a distinct identity without changing shared
semantics.

## Design anti-goals

No conversational ambiguity in automation mode, magical optimization,
progress bars that hide stages, success messages before validation, or fear-based
warnings for ordinary reversible work.

## Product identity and shared commitments

Flow is the unifying suite voice. Holons may use domain-specific terms but inherit
the same safety, evidence, and recovery commitments.

## Evidence and assumptions

Current CLIs already expose parts of this model, but consistency has not yet
been measured across command help, JSON, exit codes, and error presentation.

## Open questions

- Should suite JSON events share one envelope across all standalone CLIs?

## Downstream implications

CLI specifications, schemas, help snapshots, diagnostic reports, and future UI
work must preserve these experience semantics.

## Validation

Golden output, JSON schema, terminal-mode, accessibility, failure, and resume
tests validate the experience contract.
