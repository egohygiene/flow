# Architecture Governance Specifications

The governance document family defines how important architectural and
organizational choices are preserved over time.

## Documents

| Document | Primary question | Canonical responsibility |
| --- | --- | --- |
| `DECISIONS.md` | Why is the project the way it is? | Significant accepted decisions, rationale, alternatives, consequences, and historical lineage |

## Boundary Rules

- `PRINCIPLES.md` defines how decisions should be evaluated.
- `EPISTEMOLOGY.md` defines how claims and evidence should be evaluated.
- Proposals describe choices that have not yet been accepted.
- GitHub issues coordinate implementation work.
- Meeting notes preserve discussion.
- `DECISIONS.md` preserves significant accepted choices and their rationale.
- Superseded decisions remain discoverable.
- Historical context must not be rewritten to make an old decision appear
  better informed than it actually was.

## Storage Modes

Small repositories may keep complete records directly in `DECISIONS.md`.

Larger repositories may use:

    DECISIONS.md
        canonical index and navigation

    docs/decisions/
        one detailed architecture decision record per decision

In linked-record mode, each decision must have exactly one canonical detailed
record. The index must not become a conflicting duplicate.

## Corresponding Skill

    decisions.spec.md
        ↔ create-decisions-document

## Change Propagation

A decision may require review of:

- architecture documents
- implementation plans
- roadmap initiatives
- policies and contracts
- documentation
- migrations
- tests and evaluations
- downstream repositories

A superseding decision must preserve a traceable link to the decision it
replaces.
