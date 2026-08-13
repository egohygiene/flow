---
schema: aether.specification/v1
id: architecture-epistemology
title: Epistemology Architecture Document Specification
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
  - epistemology
  - knowledge
  - evidence
  - provenance
applies_to:
  - architecture-documents
  - epistemology-documents
depends_on:
  - architecture-document
  - architecture-purpose
  - architecture-principles
related:
  - architecture-ai-constitution
  - architecture-decisions
  - create-epistemology-document
supersedes: []
---

# Epistemology Architecture Document Specification

## Introduction

This specification defines how `EPISTEMOLOGY.md` shall be authored, maintained,
and validated.

`EPISTEMOLOGY.md` defines how the repository, product, platform, or organization
evaluates claims, evidence, provenance, confidence, uncertainty, disagreement,
and change in accepted knowledge.

It governs how conclusions should be justified. It does not prescribe which
conclusions must be reached.

## 1. Purpose and scope

`EPISTEMOLOGY.md` answers:

> How do we determine whether a claim is sufficiently supported, how certain we
> are, and how that knowledge may change?

This specification covers:

- claim classification
- evidence quality
- provenance
- source evaluation
- confidence and uncertainty
- observation and inference
- disagreement and conflict
- canonical working knowledge
- revision and deprecation
- human and AI use of evidence

It does not cover:

- implementation algorithms
- research conclusions
- project decisions
- provider-specific AI behavior
- prompt wording
- source databases
- documentation style
- architecture structure

## 2. Conceptual model

The epistemology should distinguish at least:

    source
        where information originated

    evidence
        material that supports or challenges a claim

    claim
        a statement that may be evaluated

    interpretation
        meaning derived from evidence

    confidence
        strength of support for a claim

    decision
        an accepted course of action, which may remain uncertain

    canonical working knowledge
        the current authoritative model within a defined scope

Canonical working knowledge is not declared infallible truth. It is the best
supported current model, with provenance and revision history.

## 3. Responsibilities

`EPISTEMOLOGY.md` owns:

- the claim-state vocabulary
- evidence and source evaluation criteria
- provenance expectations
- confidence and uncertainty conventions
- guidance for conflicting information
- rules for promoting claims into canonical working knowledge
- rules for revision, dispute, and deprecation
- expectations shared by humans and AI systems

## 4. Non-responsibilities

`EPISTEMOLOGY.md` does not own:

- specific research findings
- accepted project decisions
- data storage implementation
- knowledge-extraction algorithms
- AI prompts
- system architecture
- domain truth outside the project's scope
- mandatory security or privacy policy

## 5. Definitions

### Claim

A statement that can be supported, challenged, qualified, or revised.

### Evidence

Information relevant to evaluating a claim.

### Provenance

The origin, custody, transformation history, and version of information.

### Primary source

Evidence closest to the event, artifact, person, or system being described.

### Secondary source

An interpretation or synthesis of primary or other secondary sources.

### Confidence

A qualitative or calibrated quantitative assessment of evidentiary support.

### Uncertainty

A known limitation, ambiguity, missing observation, or unresolved disagreement.

### Canonical working knowledge

The currently authoritative model for a defined scope, subject to revision.

### Disputed claim

A claim for which materially credible evidence or interpretations conflict.

## 6. Claim states

The document shall define a usable vocabulary. A recommended starting set is:

| State | Meaning |
| --- | --- |
| Observed | Directly supported by recorded evidence |
| Inferred | Derived from observations through stated reasoning |
| Decided | Accepted as a course of action, not necessarily a truth claim |
| Proposed | Suggested but not accepted |
| Assumed | Temporarily treated as true for progress |
| Unverified | Not yet evaluated sufficiently |
| Disputed | Material credible disagreement remains |
| Deprecated | No longer accepted for current use |
| Open question | Intentionally unresolved |

The final vocabulary may be refined, but its meanings shall remain explicit.

## 7. Evidence and source evaluation

Evidence should be evaluated using relevant criteria such as:

- directness
- provenance completeness
- reproducibility
- independence
- expertise
- methodological quality
- recency
- consistency with other evidence
- known conflicts of interest
- transformation history
- applicability to the current context

No single criterion is universally decisive.

Authority alone shall not substitute for evidence, and raw evidence shall not be
assumed self-interpreting.

## 8. Confidence and uncertainty

The document shall define a confidence model. A qualitative model may use:

    high
    moderate
    low
    unknown

Confidence shall describe support, not rhetorical certainty.

When numerical confidence is used, the method shall be calibrated and explained.
False precision shall be avoided.

Uncertainty should identify:

- what is unknown
- why it is unknown
- what evidence is missing
- how the uncertainty affects decisions
- what would materially change confidence

## 9. Requirements

- **REQ-001**: The document shall define how claims are classified.
- **REQ-002**: It shall define provenance expectations.
- **REQ-003**: It shall define how evidence and sources are evaluated.
- **REQ-004**: It shall define confidence and uncertainty conventions.
- **REQ-005**: It shall distinguish observation, inference, assumption,
  proposal, and decision.
- **REQ-006**: It shall define how conflicting credible claims are preserved and
  reviewed.
- **REQ-007**: It shall define how knowledge becomes canonical within a scope.
- **REQ-008**: It shall define how canonical knowledge is revised, deprecated,
  or superseded.
- **REQ-009**: It shall apply to both human and AI contributions.
- **REQ-010**: It shall remain implementation-independent.
- **REQ-011**: It shall require material transformations of source information
  to remain traceable.
- **REQ-012**: It shall prevent decisions from being mislabeled as facts.
- **REQ-013**: It shall permit progress under uncertainty when assumptions are
  explicit and reversible.
- **REQ-014**: It shall define the scope in which a claim is considered
  canonical.

## 10. Constraints

- **CON-001**: The document shall not prescribe specific research conclusions.
- **CON-002**: It shall not imply that all sources are equally reliable.
- **CON-003**: It shall not equate consensus with proof or disagreement with
  falsehood.
- **CON-004**: It shall not conceal uncertainty to produce a simpler narrative.
- **CON-005**: It shall not require inaccessible private reasoning as evidence.
- **CON-006**: It shall not treat AI-generated text as self-validating.
- **CON-007**: It shall not use confidence labels without defined meaning.
- **CON-008**: It shall not make canonical knowledge immutable.
- **CON-009**: It shall not collapse observation, interpretation, and decision
  into one record.
- **CON-010**: It shall not fabricate provenance when it is unavailable.

## 11. Conflict-resolution model

When credible sources conflict:

1. preserve each claim and its provenance
2. identify the exact point of disagreement
3. evaluate source and evidence quality
4. distinguish factual conflict from interpretive conflict
5. record confidence and uncertainty
6. avoid manufacturing false consensus
7. state the current working position, if one is required
8. identify evidence that could change the position
9. preserve the dispute for later review

A working decision may be made under disagreement, but the decision shall not be
presented as proof that the underlying claim is true.

## 12. Authoring contract

### Inputs

Use:

- `PURPOSE.md`
- `PRINCIPLES.md`
- existing research and knowledge practices
- source-capture and provenance conventions
- decision records
- known cases of uncertainty or disagreement
- applicable privacy, security, and legal constraints

### Outputs

Produce:

- claim-state vocabulary
- evidence-quality criteria
- provenance requirements
- confidence model
- uncertainty model
- conflict-resolution process
- canonicalization and revision rules
- examples and edge cases

### Authoring process

1. inventory current knowledge practices
2. identify recurring failure modes
3. define claim states
4. define source and evidence criteria
5. define confidence and uncertainty language
6. define conflict handling
7. define canonicalization and revision
8. test the model against real examples
9. validate applicability to humans and AI systems

### Update conditions

Update when the philosophy of knowledge materially changes or repeated use shows
that the vocabulary or evaluation model produces ambiguity.

## 13. AI authoring strategy

AI systems shall:

1. inspect existing evidence and provenance practices
2. distinguish normative rules from current implementation
3. preserve disagreement and uncertainty
4. avoid claiming certainty unsupported by sources
5. define terms operationally
6. avoid provider-specific assumptions
7. test the proposed model against contradictory-source scenarios
8. surface missing governance rather than inventing it

## 14. Dependency model

Upstream:

- `PURPOSE.md`
- `PRINCIPLES.md`
- architecture document standard

Downstream:

- `AI_CONSTITUTION.md`
- decisions and ADRs
- research workflows
- knowledge extraction
- source capture
- documentation
- agents and prompts
- observatory and organizational reporting

A substantive change requires reviewing every artifact that classifies evidence,
confidence, or accepted knowledge.

## 15. Validation

Confirm:

- claim states are distinct and usable
- provenance expectations are explicit
- confidence labels have defined meaning
- uncertainty remains visible
- conflict handling avoids false consensus
- decisions and truth claims are separate
- canonicalization is scoped and revisable
- humans and AI systems can apply the model consistently
- examples expose likely failure modes

## 16. Acceptance criteria

- [ ] Claim states are explicitly defined.
- [ ] Evidence and source-evaluation criteria are documented.
- [ ] Provenance expectations are documented.
- [ ] Confidence and uncertainty are operationally defined.
- [ ] Conflicting credible information has a preservation and review process.
- [ ] Canonical working knowledge is scoped and revisable.
- [ ] Decisions are distinguishable from factual claims.
- [ ] AI-generated material is not treated as self-validating.
- [ ] The model remains implementation-independent.
- [ ] Both humans and AI systems can apply it.

## 17. Examples and edge cases

### Conflicting authoritative sources

Preserve both claims, provenance, and methods. Record the disagreement and avoid
inventing certainty.

### Decision under low confidence

A reversible decision may proceed with explicit assumptions, confidence, and a
review trigger.

### Missing provenance

Mark the claim as unverified or provenance-incomplete. Do not invent a source.

### New evidence overturns canonical knowledge

Create a traceable revision or supersession. Preserve historical context.

## 18. Rationale and context

A project cannot reliably govern knowledge if evidence, inference, decision,
and confidence are mixed together. A shared epistemology enables transparent
revision rather than brittle certainty.

## 19. Related artifacts

- `architecture-document`
- `architecture-purpose`
- `architecture-principles`
- `architecture-ai-constitution`
- `architecture-decisions`
- `knowledge-extraction`
- `source-capture`
- `architecture-authoring`
- `create-epistemology-document`
