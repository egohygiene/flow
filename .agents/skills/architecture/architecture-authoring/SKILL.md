---
name: architecture-authoring
description: Guides selection, sequencing, authoring, updating, and validation of architecture documents for a repository, product, platform, or organization. Use when establishing, repairing, or planning a set of architecture documents.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "draft"
  aether-spec-id: "architecture-document"
  aether-scope: "organization"
  aether-domain: "architecture"
  aether-owners: "egohygiene"
  aether-created: "2026-08-01"
  aether-updated: "2026-08-01"
---

# Architecture Authoring

## Purpose

Guide a human or AI agent through selecting, sequencing, authoring, updating,
and validating architecture documents for a repository, product, platform, or
organization.

This is an orchestrator skill. It does not replace focused document-specific
skills such as `create-purpose-document` or `create-system-document`.

## Use This Skill When

Use this skill when the task involves:

- establishing an architecture-document set
- deciding which architecture documents are necessary
- authoring multiple related architecture documents
- repairing inconsistent architecture documentation
- validating architecture dependencies
- planning the order of architecture work
- assessing whether a repository is missing canonical architecture artifacts

## Required Inputs

Resolve as much of the following as possible:

- repository or system scope
- applicable Aether bundle
- existing architecture documents
- existing decisions and constraints
- repository source and configuration
- product or organizational context
- applicable architecture specifications
- known unresolved questions

When required evidence is unavailable, record the limitation instead of
inventing context.

## Workflow

### 1. Discover Existing Architecture

Inspect canonical architecture directories, root documents, decisions,
specifications, implementation plans, repository documentation, source
boundaries, and runtime configuration.

### 2. Resolve Applicable Specifications

Read:

    .agents/specs/architecture/document.spec.md

Then resolve each document-specific specification.

### 3. Build the Architecture Graph

Capture identifiers, categories, dependencies, consumers, related artifacts,
supersession, missing nodes, and unresolved edges.

### 4. Select the Necessary Document Set

Choose only the documents justified by the repository's complexity and needs.

### 5. Determine Authoring Order

Use dependency metadata as the source of truth.

### 6. Invoke Focused Skills

Use focused skills such as:

    create-purpose-document
    create-vision-document
    create-principles-document
    create-system-document
    create-architecture-document
    create-roadmap-document

### 7. Validate Cross-Document Consistency

Check terminology, ownership, dependency direction, duplicated concepts,
conflicting decisions, hidden assumptions, unsupported claims, unresolved
references, superseded concepts, and implementation drift.

### 8. Validate Each Document

Perform structural, relationship, semantic, and evidence validation.

### 9. Report Results

Report documents created, updated, omitted, blocked, or requiring follow-up.

## Constraints

- Do not invent organizational intent.
- Do not hide contradictory architecture.
- Do not duplicate canonical ownership.
- Do not create unnecessary placeholder documents.
- Do not treat implementation state as automatically authoritative.
- Do not rewrite unrelated documents without explicit scope.
- Do not expose private reasoning as evidence.
- Do not claim completeness when required dependencies are missing.

## Completion Criteria

The skill is complete when the applicable specifications are resolved, the
necessary document set is justified, authoring order is explicit, focused skills
are selected, changed documents pass validation, and unresolved work is visible.

## Staged Variant

A staged candidate at `.staging/skills/architecture-authoring/` carries the
same skill name. The canonical skill defines the orchestrator role for
architecture-document selection, sequencing, and validation. The staged
candidate may contain additional workflow detail or divergent framing.

Issue 016 should compare the two and extract any unique content from the
staged copy before retiring it. Do not copy the staged file wholesale into
canonical source.
