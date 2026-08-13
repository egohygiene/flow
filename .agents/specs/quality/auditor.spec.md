---
schema: aether.specification/v1
id: auditor
title: Repository Auditor Specification
kind: specification
version: 2.0.0
status: approved
owners:
  - egohygiene
created: 2026-07-12
updated: 2026-08-02
domain: quality
tags:
  - audit
  - repository
  - quality
  - evidence
  - read-only
applies_to:
  - repositories
  - repository-audits
depends_on:
  - specfile
related:
  - reflector
  - audit-repository
supersedes: []
source_files:
  - auditor.spec.md
---

# Repository Auditor Specification

## Introduction

This specification defines a repository-agnostic contract for evidence-based,
read-only auditing.

An audit observes repository reality, compares it with explicit architecture,
policy, specification, and engineering expectations, and produces an immutable
report.

## 1. Purpose and Scope

This specification answers:

> How shall a repository be audited reproducibly, transparently, and without
> destructive behavior?

It covers request scope, discovery order, strategies, evidence, findings,
severity, confidence, report structure, prior-audit awareness, positive
observations, and partial or blocked behavior.

It does not authorize remediation, issue creation, dependency upgrades, or
claims unsupported by evidence.

## 2. Core Principles

Every audit shall be:

- evidence-based
- repository-aware
- non-destructive
- reproducible
- transparent about uncertainty
- explicitly scoped
- prioritized
- actionable
- balanced
- historically additive

## 3. Audit Request Contract

Recommended request:

    audit_name: repository-health
    strategy: holistic
    scope:
      include:
        - "."
      exclude:
        - "generated/**"
    focus:
      - architecture
      - testing
      - developer-experience
    depth: standard
    outputs:
      - markdown
    constraints:
      - read-only

Only `audit_name` is required.

Inferred defaults shall be recorded.

## 4. Repository Discovery Order

Inspect when present:

1. entry documentation
2. architecture and system documents
3. governance and decisions
4. specifications
5. agents and skills
6. task and build automation
7. dependency manifests
8. CI/CD workflows
9. source
10. tests
11. documentation
12. existing audits

Missing context shall be reported, not assumed.

## 5. Audit Strategies

Supported strategies:

- holistic
- architecture
- code-quality
- testing
- security-and-privacy
- performance
- accessibility-and-ux
- dependencies
- ci-and-automation
- documentation
- developer-experience
- scoped

## 6. Observation Types

Use one primary classification:

- confirmed defect
- probable issue
- architectural concern
- maintainability risk
- optimization opportunity
- documentation gap
- future enhancement
- intentional trade-off
- needs human clarification
- positive observation

## 7. Evidence Contract

Evidence may include files, paths, symbols, configuration, workflows, tests,
diagnostics, runtime output actually observed, and repository structure.

Evidence labels:

- observed
- inferred
- recommended
- unverified

The auditor shall not fabricate line numbers, command results, runtime behavior,
coverage, vulnerability exploitability, or stakeholder intent.

## 8. Finding Contract

Every nontrivial finding shall include:

- `AUDIT-NNN` identifier
- title
- classification
- severity
- confidence
- status
- area
- effort
- impact
- observation
- evidence
- why it matters
- recommendation
- suggested validation
- dependencies or risks

## 9. Normalized Models

Severity:

- critical
- high
- medium
- low
- informational

Confidence:

- high
- medium
- low

Status:

- confirmed
- probable
- needs-validation
- intentional-trade-off
- needs-clarification
- not-applicable

Effort:

- small
- medium
- large
- unknown

Impact:

- low
- medium
- high
- critical

Low-confidence findings shall state what evidence is missing.

## 10. Output Contract

Required output:

    audits/{audit-name}-{utc-timestamp}.md

Timestamp:

    YYYYMMDDTHHMMSSZ

Existing audit reports are immutable.

Optional future adapters may produce JSON, SARIF, annotations, or GitHub job
summaries from the same finding model.

## 11. Report Structure

Reports should include:

- executive summary
- scope
- repository context
- methodology
- overall assessment
- findings summary
- severity-grouped findings
- positive observations
- opportunities by area
- suggested issue backlog
- deferred or out-of-scope observations
- uncertainties and clarifications
- evidence index
- validation notes

Omitted sections shall be marked not applicable.

## 12. Report Status

Allowed statuses:

- complete
- partial
- blocked
- failed

Incomplete reports shall record inspected scope, missing scope, reason, and
continuation requirements.

## 13. Positive Observations

Completed and partial audits shall capture strengths when evidence supports
them.

A defect-only report is incomplete unless the absence of positive evidence is
explicitly stated.

## 14. Existing Audit Awareness

Before writing a new report, inspect `audits/`.

Classify recurring history as:

- new
- recurring
- changed
- apparently resolved
- unable to verify

Historical reports shall not be modified.

## 15. Non-Destructive Behavior

Without explicit authorization, the auditor shall not:

- modify files
- run destructive commands
- update dependencies
- reformat source
- commit
- open issues
- apply fixes
- alter historical audits

## 16. Suggested Issue Backlog

Candidate issues may be proposed with:

- priority
- source findings
- dependencies
- outcome
- acceptance criteria

They shall not be created automatically.

## 17. AI Execution Rules

AI auditors shall:

1. resolve scope and constraints
2. read repository context in order
3. inspect prior audits
4. gather evidence
5. distinguish fact from inference
6. classify and prioritize findings
7. capture strengths
8. document uncertainty
9. produce an immutable report
10. avoid unauthorized remediation

## 18. Acceptance Criteria

- [ ] Request and inferred defaults are recorded.
- [ ] Scope and exclusions are explicit.
- [ ] Discovery order was followed.
- [ ] Historical audits were considered.
- [ ] Findings use the canonical model.
- [ ] Evidence labels are present.
- [ ] Severity, confidence, status, effort, and impact are normalized.
- [ ] Positive observations are included.
- [ ] Uncertainty is visible.
- [ ] Partial and blocked reporting is supported.
- [ ] Historical reports remain immutable.
- [ ] No unauthorized modification occurred.

## 19. Related Artifacts

- `specfile`
- `reflector`
- `audit-repository`
