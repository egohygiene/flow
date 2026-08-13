---
name: prepare-arxiv-release
description: Prepares a verified, reproducible arXiv release package from repository source. Use when a project needs to compile, validate, and package a submission for the arXiv preprint server.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "draft"
  aether-spec-id: "arxiv-publishing"
  aether-scope: "organization"
  aether-domain: "publishing"
  aether-owners: "egohygiene"
  aether-created: "2026-08-02"
  aether-updated: "2026-08-02"
---

# Prepare arXiv Release

## Purpose

Execute the reusable procedure governed by `arxiv-publishing`.

Primary question:

> Can this source be transformed into a verified, reproducible, and inspectable arXiv release?

## Required Inputs

Resolve:

- governing specification and version
- current source or repository state
- scope and constraints
- upstream architecture or evidence
- output location
- validation expectations
- unresolved decisions

Missing evidence must remain visible.

## Workflow

1. verify current official arXiv requirements
2. resolve source revision and canonical metadata
3. create isolated staging
4. copy only required source and assets
5. apply deterministic publisher transformations
6. compile in the recorded environment
7. validate bibliography, figures, paths, hidden files, and ancillary content
8. scan source for extraneous or sensitive material
9. require human inspection of the PDF
10. package source, PDF, manifest, checksums, and logs

## Output Contract

Primary output:

    dist/arxiv/

Also report assumptions, evidence gaps, validation status, unresolved
questions, and downstream actions requiring separate authorization.

## Constraints

- Follow the governing specification.
- Preserve provenance and uncertainty.
- Do not invent authority, evidence, or current behavior.
- Do not silently expand scope.
- Do not claim completion when required validation is missing.
- Keep proposed downstream work separate from authorized execution.

## Completion Criteria

- [ ] Governing specification is resolved.
- [ ] Scope and constraints are explicit.
- [ ] Required evidence was inspected.
- [ ] The primary output was created.
- [ ] Validation was executed or its absence documented.
- [ ] Open questions and authorization needs are visible.

## Staged Variant

A staged candidate at `.staging/skills/arxiv-publishing/` covers similar
ground under the name `arxiv-publishing`. The canonical skill is named
`prepare-arxiv-release` and is governed by the `arxiv-publishing` specification.

Issue 016 should compare the two and extract any unique submission-format
guidance, checklist detail, or arXiv-requirement references from the staged
copy before retiring it. Do not copy the staged file wholesale into canonical
source.
