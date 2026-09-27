---
schema: aether.architecture-document/v1
id: flow-adr-0013
title: Pin Optiflow v0.1.1 behind a native read-only adapter
kind: architecture-document
version: 0.1.0
status: proposed
owners:
  - egohygiene
created: 2026-09-27
updated: 2026-09-27
governed_by:
  - architecture-decisions
depends_on:
  - flow-architecture
related:
  - flow-decisions
  - flow-roadmap
supersedes: []
---

# ADR-0013: Pin Optiflow v0.1.1 behind a native read-only adapter

## Context

Flow's generic process contract uses a Flow JSON Lines invocation, event, and
result protocol. Immutable Optiflow v0.1.1 exposes a different released CLI:
direct argv and one `optiflow.command-result.v1` stdout document with
provider-owned committed artifact sets. An invented JSONL transcript would
confuse provider evidence with Flow's own validation claims. The release
qualifies read-only scan, report, and review planning. Later mutation code is
not part of that immutable release.

## Decision

The initial adapter is a public Flow library API with a strict native-protocol
translator. It pins exact target-specific release archives and executable
digests, probes `--version`, launches the binary with bounded output/time and
literal argv, and validates exit, coverage, native schema, source identity,
artifact-set marker, member digests, and plan safety. It records a separate
versioned local receipt before and after each step; partial coverage remains
partial. Dry-run, quarantine, restore, and finalization are explicit
unsupported capabilities. No review plan grants mutation authority.

This local receipt does not enter `flow.run-state/v1` as an accepted generic
checkpoint. The provider's native CLI cannot satisfy the existing generic
Flow process invocation contract without a separate wrapper and authority
mapping. The suite CLI and cross-provider graph composition will consume the
read-only adapter in later Flow work. A mutation adapter requires a separately
qualified release and a new authority/recovery contract.

## Consequences and review triggers

The adapter writes only local provider and Flow evidence outside the selected
source root. It does not sandbox the trusted pinned binary or prove a
race-free executable open. Root identity is rechecked between commands;
individual source observations are the provider's read-only evidence, not
apply-time authorization. An interrupted receipt requires inspection and no
artifact file alone implies Flow completion. Receipt integrity is local
content verification, not an authenticated signature.

Review this decision when a released mutation protocol is qualified, when the
native CLI contract changes, or when Flow's generic graph accepts a
provider-native adapter through an explicitly versioned authority boundary.

## Validation

The synthetic release fixture calls the public Flow API, verifies source bytes
remain unchanged, checks the three committed artifact sets and receipt reopen,
and exercises release mismatch, path overlap, escape, interruption, and
post-run artifact corruption refusals. macOS binaries are pinned by digest but
need native execution verification.
