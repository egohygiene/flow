# Architecture Relationship Model

## Relationship Types

- `depends_on`: required upstream artifact
- `related`: relevant but nonrequired artifact
- `supersedes`: replaced artifact
- `implements`: specification operationalized by a skill
- `recommended_agents`: agents suited to execute a skill

## Rules

- Use stable artifact identifiers.
- Keep dependency graphs acyclic.
- Do not treat directory placement as a dependency declaration.
- Preserve superseded artifacts for historical discovery.
- Surface conflicting ownership.
- Treat unresolved identifiers as validation failures.
