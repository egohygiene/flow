---
name: skill-authoring
description: Creates, curates, upgrades, or validates portable Aether skill packages with normalized metadata, progressive disclosure, deterministic evals, and explicit provenance. Use when adding or maintaining reusable first-party skills rather than solving a one-off task.
license: MIT
metadata:
  aether-version: "1.0.0"
  aether-status: "experimental"
  aether-scope: "organization"
  aether-domain: "authoring"
  aether-owners: "egohygiene"
  aether-created: "2026-08-08"
  aether-updated: "2026-08-08"
---

# Skill Authoring

## Purpose

Create or maintain a portable skill package whose instructions, resources,
metadata, provenance, and evaluations are reusable across repositories and
hosts.

## Required Inputs

Resolve as much of the following as possible:

- requested capability and trigger conditions
- canonical output path and stable skill identity
- repository or organization conventions that the skill must respect
- required references, templates, or scripts
- provenance, authorship, and license status
- validation and evaluation expectations
- open questions or unresolved ownership boundaries

Record missing evidence instead of inventing conventions.

## Workflow

1. define the reusable capability and its routing boundary
2. choose one stable kebab-case identity and canonical directory
3. normalize frontmatter to the Agent Skills and Aether contracts
4. keep the core workflow concise and move detail into focused resources such as:

    - `./references/validation-checklist.md`
    - `./templates/SKILL.template.md`

5. add only the resources that reduce repeated work or make execution deterministic
6. make authorship, provenance, license, and adoption status explicit
7. create deterministic eval coverage for trigger, boundary, insufficient-evidence, negative, and update scenarios
8. validate links, package structure, metadata, and any included executable resources

## Constraints

- Do not turn a skill into a generic agent or plugin installer.
- Do not duplicate repository-wide instructions that belong elsewhere.
- Do not keep synonym identities alive without a clear behavior split.
- Do not add scripts, assets, or examples that do not reduce repeated work.
- Do not claim first-party ownership without explicit evidence.

## Completion Criteria

- [ ] Stable identity and canonical path are explicit.
- [ ] Frontmatter satisfies the current skill contract.
- [ ] Progressive-disclosure resources are focused and necessary.
- [ ] Provenance, license, and source-delta decisions are visible.
- [ ] Deterministic eval coverage exists for the supported workflow.
- [ ] Validation steps and remaining questions are reported truthfully.

## Provenance

This canonical skill is first-party Ego Hygiene content curated from the staged
candidate at `.staging/skills/skill-authoring/SKILL.md`.

## Source Delta

- Adopted: the staged workflow for defining capability, planning resources,
  keeping `SKILL.md` concise, and validating package structure.
- Rewritten: canonical frontmatter, Aether metadata, reusable template,
  evaluation coverage, and explicit provenance handling.
- Rejected: the staged `.agents/skills/` placement guidance from the historical
  copy in favor of the canonical `.agents/skills/<domain>/<name>/`
  source layout; the broader `create-skill` synonym remains only as a reviewed
  legacy reference.
