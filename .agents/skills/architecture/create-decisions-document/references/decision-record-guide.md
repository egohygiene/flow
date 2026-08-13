# Decision Record Guide

## Record the Decision, Not the Discussion

A decision record preserves:

- the relevant context
- the accepted choice
- the rationale
- meaningful alternatives
- trade-offs
- consequences
- lineage

It does not preserve every comment, meeting statement, or implementation
step.

## Significance Test

Record a choice when it is durable, consequential, expensive to reverse,
cross-cutting, or likely to be questioned again.

Do not record every local refactor or routine implementation choice.

## Historical Integrity

Write the original context using evidence available at the time.

Later information belongs in:

    Observed Outcomes

or:

    Review Notes

Do not retrofit later knowledge into the original rationale.

## Evidence Levels

Strong evidence may include:

- accepted proposals
- approved pull requests
- explicit maintainer statements
- architecture review records
- migration plans
- release or implementation history

Weak evidence may include:

- current repository structure
- inferred intent from code
- undocumented convention
- memory without corroboration

Weak evidence may support a provisional reconstruction, but not a confident
historical claim.

## Decision Lineage

    adr-0004
        status: superseded
        superseded_by: adr-0018

    adr-0018
        status: accepted
        supersedes: adr-0004

Preserve both directions.

## Index Versus Detailed Record

In indexed ADR mode:

- `DECISIONS.md` owns navigation and status summary.
- the ADR file owns complete context and rationale.
- the index must not contain a conflicting duplicate of the full record.
