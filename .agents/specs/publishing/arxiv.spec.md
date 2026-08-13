---
schema: aether.specification/v1
id: arxiv-publishing
title: arXiv Publishing Infrastructure Specification
kind: specification
version: 2.0.0
status: draft
owners:
  - egohygiene
created: 2026-07-18
updated: 2026-08-02
domain: publishing
tags:
  - arxiv
  - scholarly-publishing
  - deterministic-builds
  - accessibility
  - release-engineering
applies_to:
  - scholarly-publishing
  - arxiv-release-pipelines
depends_on:
  - specfile
related:
  - prepare-arxiv-release
supersedes: []
source_files:
  - arxiv.spec.md
---

# arXiv Publishing Infrastructure Specification

## Introduction

This specification defines deterministic, publisher-agnostic scholarly
publishing infrastructure with first-class arXiv support.

Scholarly publishing is treated as a reproducible release-engineering process.

Canonical scholarly content remains publisher-independent. Publisher adapters
transform canonical source into validated, traceable release artifacts.

arXiv rules are mutable external dependencies and shall be revalidated against
official documentation before release.

## 1. Purpose and Scope

This specification answers:

> How shall canonical scholarly source be transformed into a deterministic,
> accessible, publisher-compatible arXiv release?

It covers canonical source, publisher adapters, staging, TeX compatibility,
bibliography, figures, ancillary files, source hygiene, validation, packaging,
manifests, checksums, and accessibility-aware authoring.

It does not cover browser automation, moderation bypass, endorsement bypass,
fabricated metadata, or editorial decisions.

## 2. Architectural Principles

### Canonical Source

The repository shall maintain one publisher-independent scholarly source.

It shall preserve semantic structure, metadata, citations, accessibility intent,
and transformation capability.

### Derived Publisher Artifacts

arXiv bundles, PDFs, manifests, logs, and archives are derived artifacts.

Adapters shall not mutate canonical source in place.

### Deterministic Build

Builds shall pin or record:

- TeX distribution and version
- processor
- bibliography backend
- build environment
- source revision
- adapter version
- transformation version
- artifact hashes

### External Requirement Verification

Mutable arXiv rules shall be recorded with source URL, verification date,
requirement value, and compatibility notes.

A release shall not rely on stale unverified assumptions.

## 3. Recommended Repository Model

    papers/
    ├── paper/
    │   ├── paper.tex
    │   ├── references.bib
    │   ├── sections/
    │   ├── figures/
    │   └── metadata/
    ├── publishers/
    │   └── arxiv/
    │       ├── adapter/
    │       ├── target.yml
    │       ├── transforms/
    │       ├── validators/
    │       └── templates/
    ├── scripts/
    ├── tests/
    └── dist/
        └── arxiv/

## 4. Target Capability Record

The arXiv adapter shall maintain a machine-readable target record containing:

    target: arxiv
    verified_at: YYYY-MM-DD
    official_sources: []
    texlive:
      supported: []
      default: null
    processors: []
    filename_policy: null
    hidden_file_policy: null
    bibliography:
      supported_backends: []
    figures:
      processor_compatibility: {}
    ancillary:
      path: anc/
    verification_status: current

A configurable freshness threshold shall determine when re-verification is
required.

## 5. Canonical Source Requirements

Canonical source shall:

- use semantic sectioning and emphasis
- preserve citations and attribution
- avoid unnecessary manual positioning
- avoid hidden local dependencies
- preserve Unicode safely
- preserve figure descriptions or alt-text intent
- keep publisher-specific transformations outside canonical content

## 6. Release Pipeline

Required stages:

    resolve
        determine target capability and source revision

    stage
        create an isolated workspace

    transform
        apply deterministic publisher rules

    compile
        build in the recorded environment

    validate
        run compatibility, accessibility, and hygiene checks

    inspect
        require human verification of the generated PDF and bundle

    package
        create deterministic artifacts

    attest
        generate manifest, checksums, and logs

Canonical source shall remain unchanged.

## 7. Staging and Transformation

Staging shall:

- copy only required source and assets
- normalize or validate filenames
- exclude version-control and cache material
- remove unused outputs and assets
- preserve required generated bibliography or index files
- record every transformation

Transformations may include path normalization, flattening, bibliography
freezing, metadata adaptation, package substitution, ancillary relocation, and
release README generation.

## 8. TeX and Bibliography Compatibility

The adapter shall:

- verify the current arXiv TeX environment
- select a supported processor
- verify package availability
- verify bibliography backend compatibility
- ensure included `.bbl` files match the selected toolchain
- preserve required `.ind`, `.gls`, or `.nls` files when applicable
- prohibit undeclared external runtime dependencies

Current compatibility shall be configuration-driven rather than permanently
hardcoded into this core specification.

## 9. Figures and Embedded Content

The adapter shall:

- validate figure formats against the selected processor
- prohibit runtime figure-conversion assumptions
- verify case-sensitive paths
- reject embedded JavaScript
- preserve scientific meaning after conversion
- identify unused assets
- preserve accessibility descriptions where supported

## 10. Source-Package Hygiene

Preflight shall detect:

- hidden files and directories
- version-control material
- caches
- credentials
- private notes
- unused figures
- backup files
- unintended logs
- sensitive comments or metadata
- unrelated source files

The staged source shall be treated as a public archival artifact.

## 11. Ancillary Files

Ancillary content may include datasets, code, images, diagrams, spreadsheets,
and reproducibility tooling.

Ancillary files shall:

- use the target-required path
- avoid TeX runtime dependencies
- avoid embedded executable content
- include explanatory metadata
- be intentionally selected

## 12. Accessibility

The system should preserve:

- semantic headings
- machine-readable text
- Unicode
- embedded fonts
- meaningful links
- figure descriptions
- structured metadata
- HTML-conversion compatibility where practical

Limitations shall be reported.

## 13. Scholarly Integrity

The pipeline shall preserve authorship, references, license, accurate metadata,
category relevance, and required disclosures.

It shall not fabricate affiliations, authorship, evidence, categories, or
research claims.

## 14. Release Artifact Contract

Recommended output:

    dist/arxiv/
    ├── source/
    ├── paper.pdf
    ├── submission.tar.gz
    ├── manifest.json
    ├── checksums.txt
    └── logs/

The manifest should record target, adapter version, source revision, build
environment, TeX version, processor, bibliography backend, generation time,
license, hashes, verification date, transformations, validation, and warnings.

## 15. Validation

Validate:

- isolated compilation
- target-record freshness
- processor and package compatibility
- bibliography compatibility
- filename and path safety
- case-sensitive references
- hidden and extraneous files
- sensitive-data hygiene
- figure compatibility
- ancillary structure
- embedded JavaScript
- fonts and machine readability
- accessibility metadata
- deterministic manifest and checksums
- human PDF inspection

## 16. Failure and Overrides

A release shall fail when compilation fails, requirements are stale beyond
policy, hidden dependencies remain, prohibited content is found, hashes cannot
be generated, or canonical source was mutated unexpectedly.

Overrides shall require explicit authorization and shall appear in the manifest.

## 17. Publisher-Agnostic Extension

Shared abstractions should support additional publisher targets through common:

- metadata
- adapter interfaces
- staging interfaces
- validators
- manifest schema
- hashing
- transformation provenance

## 18. Non-Goals

The infrastructure shall not automate submission clicks, bypass moderation or
endorsement, fabricate metadata, conceal disclosures, rely on unsupported TeX
behavior, or prioritize visual hacks over semantic structure.

## 19. Acceptance Criteria

- [ ] Canonical source remains publisher-independent.
- [ ] arXiv requirements use a verified target capability record.
- [ ] Release staging is isolated.
- [ ] Transformations are deterministic and recorded.
- [ ] Compilation uses a recorded compatible environment.
- [ ] Bibliography and generated files are validated.
- [ ] Hidden, extraneous, and sensitive files are detected.
- [ ] Figures and ancillary content conform.
- [ ] Accessibility intent is preserved where practical.
- [ ] Human PDF inspection is required.
- [ ] Manifest, checksums, and logs are generated.
- [ ] Canonical source is not mutated.
- [ ] Overrides are explicit and auditable.

## 20. Related Artifacts

- `specfile`
- `prepare-arxiv-release`
