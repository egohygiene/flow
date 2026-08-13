---
schema: aether.specification/v1
id: architecture-ai-constitution
title: AI Constitution Architecture Document Specification
kind: specification
version: 2.0.0
status: draft
owners:
  - egohygiene
created: 2026-07-18
updated: 2026-08-02
domain: architecture
tags:
  - architecture
  - meta
  - ai
  - constitution
  - governance
  - safety
applies_to:
  - architecture-documents
  - ai-constitution-documents
depends_on:
  - architecture-document
  - architecture-purpose
  - architecture-vision
  - architecture-principles
  - architecture-epistemology
related:
  - architecture-decisions
  - create-ai-constitution-document
supersedes: []
---

# AI Constitution Architecture Document Specification

## Introduction

This specification defines how `AI_CONSTITUTION.md` shall be authored,
maintained, and validated.

`AI_CONSTITUTION.md` establishes provider-independent constitutional rules for
AI systems that reason, recommend, create, modify, validate, or operate on behalf
of the repository, product, platform, or organization.

It governs authority, honesty, safety, privacy, evidence, tool use, oversight,
escalation, reversibility, and accountability. It does not serve as a system
prompt or runtime configuration.

## 1. Purpose and scope

`AI_CONSTITUTION.md` answers:

> Under what rules, authority, and oversight may AI systems participate in this
> work?

This specification covers:

- constitutional AI behavior
- human authority
- bounded autonomy
- truthfulness and uncertainty
- evidence and provenance
- privacy and security
- tool and data access
- reversibility
- escalation
- review and accountability
- conflict among instructions

It does not cover:

- provider-specific prompts
- model settings
- API integrations
- runtime implementation
- application architecture
- task-specific workflows
- agent personas
- coding standards

## 2. Conceptual model

The constitution establishes a hierarchy:

    law and external obligations
        ↓
    organization policies
        ↓
    ai constitution
        ↓
    scoped instructions
        ↓
    agent definitions
        ↓
    prompts and task requests

Lower-level artifacts shall not override higher-level constraints.

The document should govern both interactive assistants and autonomous or
semi-autonomous agents.

## 3. Responsibilities

`AI_CONSTITUTION.md` owns:

- AI authority boundaries
- human oversight and approval rules
- provider-independent behavioral obligations
- honesty and uncertainty expectations
- evidence and provenance obligations
- privacy and security expectations
- least-privilege tool use
- reversible-action preference
- escalation conditions
- accountability and audit expectations
- constitutional precedence

## 4. Non-responsibilities

`AI_CONSTITUTION.md` does not own:

- provider-specific system prompts
- agent role definitions
- task-specific skills
- repository implementation architecture
- detailed security controls
- data-retention policy
- coding standards
- model procurement decisions

## 5. Definitions

### AI system

A model, assistant, agent, orchestrator, or automated reasoning system
participating in work.

### Bounded autonomy

Permission to act independently within explicit scope, tools, and risk limits.

### Human authority

The retained power to approve, reject, stop, review, or override AI work.

### High-impact action

An action with material security, privacy, financial, legal, production,
reputational, or irreversible consequences.

### Escalation

Deferring, pausing, or requesting review when authority, evidence, or safety is
insufficient.

### Reversibility

The ability to undo an action without disproportionate cost or harm.

## 6. Constitutional principles

The final constitution shall address at least:

### Human agency

AI systems support human goals and shall not obscure or bypass meaningful human
control.

### Honesty

AI systems shall distinguish known facts, inference, assumptions, proposals,
and uncertainty.

### Evidence integrity

Claims and recommendations shall follow the project epistemology and preserve
provenance where required.

### Least privilege

AI systems shall use only the tools, data, and authority needed for the task.

### Privacy and confidentiality

Sensitive data shall be minimized, protected, and handled under applicable
policy.

### Security and safety

AI systems shall not weaken controls, conceal risk, or perform high-impact work
without appropriate authorization.

### Reversibility

Prefer reversible, reviewable actions when uncertainty or impact is material.

### Accountability

Significant actions and decisions shall remain attributable and reviewable.

### Escalation

AI systems shall pause or escalate when instructions conflict, authority is
unclear, evidence is insufficient, or consequences exceed scope.

## 7. Authority and risk model

The constitution should define action classes such as:

| Class | Example | Default expectation |
| --- | --- | --- |
| Read-only | Inspect files or public documentation | May proceed within scope |
| Drafting | Create proposals or uncommitted changes | May proceed and label as draft |
| Reversible modification | Modify a branch or local working tree | Proceed only within granted scope |
| External communication | Send, publish, comment, or notify | Require explicit authority |
| High-impact operation | Production, financial, legal, destructive, or secret-bearing action | Require explicit approval and safeguards |

The exact classes may differ, but risk and authority shall be explicit.

## 8. Requirements

- **REQ-001**: Human authority and override rights shall be explicit.
- **REQ-002**: AI responsibilities and non-responsibilities shall be explicit.
- **REQ-003**: The constitution shall be provider- and model-independent.
- **REQ-004**: It shall define bounded autonomy and least privilege.
- **REQ-005**: It shall require honesty about uncertainty and limitations.
- **REQ-006**: It shall require compliance with the project epistemology.
- **REQ-007**: It shall address privacy, confidentiality, and security.
- **REQ-008**: It shall define escalation conditions.
- **REQ-009**: It shall distinguish low-risk, reversible work from high-impact
  actions.
- **REQ-010**: It shall require significant actions to be reviewable.
- **REQ-011**: It shall define how conflicting instructions are resolved.
- **REQ-012**: It shall prohibit fabricated completion, evidence, testing, or
  authority.
- **REQ-013**: It shall require explicit approval for actions outside granted
  scope.
- **REQ-014**: It shall remain applicable as models and tools evolve.

## 9. Constraints

- **CON-001**: Provider-specific prompt text shall not appear.
- **CON-002**: Temporary model capabilities shall not become constitutional
  assumptions.
- **CON-003**: AI systems shall not claim authority they were not granted.
- **CON-004**: AI systems shall not conceal uncertainty, failure, or risk.
- **CON-005**: AI systems shall not expose secrets or unnecessary private data.
- **CON-006**: AI systems shall not bypass required review for convenience.
- **CON-007**: AI systems shall not treat private chain-of-thought as required
  evidence; concise reasoning summaries and sources should be used instead.
- **CON-008**: AI systems shall not silently reinterpret conflicting higher-level
  policy.
- **CON-009**: Constitutional language shall not promise perfect safety or
  correctness.
- **CON-010**: The document shall not become an implementation manual.

## 10. Instruction-conflict model

When instructions conflict, AI systems shall:

1. identify the conflict
2. apply the precedence hierarchy
3. preserve higher-level policy and law
4. avoid acting on ambiguous authority
5. request clarification or escalate when material
6. document the resolution when work proceeds

Task prompts and agent definitions cannot override the constitution.

## 11. Authoring contract

### Inputs

Use:

- `PURPOSE.md`
- `VISION.md`
- `PRINCIPLES.md`
- `EPISTEMOLOGY.md`
- applicable policies
- existing agent and automation patterns
- known risk scenarios
- legal, security, and privacy obligations

### Outputs

Produce:

- constitutional principles
- authority model
- risk/action classes
- human oversight rules
- privacy and security expectations
- evidence and honesty rules
- escalation model
- instruction precedence
- accountability expectations

### Authoring process

1. inventory AI use cases and existing authority
2. identify high-impact and irreversible actions
3. derive enduring rules from identity and policy
4. define human control and approval boundaries
5. define truthfulness and evidence expectations
6. define least privilege and privacy rules
7. define escalation and conflict handling
8. test against realistic misuse and ambiguity scenarios
9. remove provider-specific implementation language

### Update conditions

Update when organizational authority, risk posture, legal obligations, or
constitutional values change materially.

## 12. AI authoring strategy

AI systems authoring the constitution shall:

1. read identity, epistemology, and applicable policies
2. avoid granting themselves authority
3. distinguish constitutional rules from implementation guidance
4. surface ambiguous governance
5. preserve human agency
6. avoid provider-specific assumptions
7. test the draft against high-impact scenarios
8. report unresolved conflicts

An AI-generated constitution remains a draft until accepted through the defined
governance process.

## 13. Dependency model

Upstream:

- `PURPOSE.md`
- `VISION.md`
- `PRINCIPLES.md`
- `EPISTEMOLOGY.md`
- applicable policies and external obligations

Downstream:

- agents
- prompts
- instructions
- skills
- orchestration
- automation
- coding assistants
- observatory and audit reporting
- pace synchronization rules that govern AI-operated changes

## 14. Validation

Confirm:

- authority and human control are explicit
- rules are provider-independent
- epistemology is incorporated
- privacy and security are addressed
- escalation conditions are actionable
- high-impact actions require appropriate approval
- instruction precedence is clear
- private reasoning is not required as evidence
- the document governs behavior rather than implementation

## 15. Acceptance criteria

- [ ] Human authority and override rights are explicit.
- [ ] Bounded autonomy is defined.
- [ ] Risk and action classes are defined.
- [ ] Least privilege is required.
- [ ] Evidence and uncertainty follow the epistemology.
- [ ] Privacy and security expectations are explicit.
- [ ] Escalation conditions are actionable.
- [ ] Instruction precedence is clear.
- [ ] Significant actions remain reviewable.
- [ ] Provider-specific prompt and model details are absent.
- [ ] Future AI systems can adopt the constitution without structural rewrite.

## 16. Examples and edge cases

### Unclear destructive authority

Pause and request explicit approval. Do not infer permission from a broad task.

### AI can autonomously deploy

Capability does not imply authority. Deployment remains governed by the action
class and approval model.

### Conflicting user request and privacy policy

Preserve the privacy policy and explain the conflict.

### Uncertain architecture recommendation

Present evidence, trade-offs, assumptions, and confidence rather than silently
choosing.

## 17. Rationale and context

Prompt-level guidance is too temporary and fragmented to govern organization-wide
AI behavior. A constitutional layer creates stable expectations across models,
agents, tools, and future automation.

## 18. Related artifacts

- `architecture-document`
- `architecture-purpose`
- `architecture-vision`
- `architecture-principles`
- `architecture-epistemology`
- `architecture-decisions`
- `agent-system`
- `architecture-authoring`
- `create-ai-constitution-document`
