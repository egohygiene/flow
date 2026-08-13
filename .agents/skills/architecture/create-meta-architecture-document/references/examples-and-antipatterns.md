# Create Meta Architecture Document — examples and anti-patterns

## Primary question

    How is this architecture-document system organized, navigated, validated, and evolved?

## Strong output characteristics

- specific enough to guide real work
- durable across provider and implementation changes
- aligned with upstream identity and governance
- honest about uncertainty and authority
- bounded against adjacent documents
- understandable without source-code knowledge

## Anti-patterns

- duplicating every architecture document
- relying only on directory order
- hiding missing documents
- making META.md an upstream dependency for everything

## Review questions

- Does the document answer the primary question directly?
- Is any content owned by another architecture document or policy?
- Would the guidance remain valid with different tools or providers?
- Are claims, authority, and provenance supported?
- Are conflicts and missing information visible?
