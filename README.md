# Flow

> One coherent way to inspect, plan, transform, validate, and explain content.

Flow is the suite and future orchestration layer for Ego Hygiene's independently
useful Rust tools:

| Holon | Owns | Does not own |
| --- | --- | --- |
| [Aniflow](docs/architecture/holons/aniflow/README.md) | Time-based video decomposition, ordered processing, reconstruction, and temporal validation | Collection optimization or general document rendering |
| [Optiflow](docs/architecture/holons/optiflow/README.md) | Local inventory, evidence-backed relationships, safe optimization plans, and collection normalization | Video-frame processing or transform-graph execution |
| [Renderflow](docs/architecture/holons/renderflow/README.md) | Spec-driven transform graphs and publication-ready document, image, and audio derivatives | Collection policy or temporal video reconstruction |
| Flow | Cross-holon planning, compatibility, execution state, provenance, validation, recovery, and suite experience | Reimplementing a holon's specialized engine |

Each holon remains usable as both a Rust library and a standalone CLI. Flow may
compose them through stable public library interfaces or versioned CLI
contracts, without introducing direct dependencies between sibling holons.

This repository is currently architecture-first. It intentionally contains no
imported tool source or working orchestrator yet. The first implementation work
begins only after the boundaries and migration gates in the [roadmap](ROADMAP.md)
are accepted.

## Architecture

- [Architecture map](docs/architecture/README.md)
- [Suite system model](docs/architecture/foundation/SYSTEM.md)
- [Suite structural architecture](docs/architecture/foundation/ARCHITECTURE.md)
- [Decision index](docs/architecture/governance/DECISIONS.md)
- [Roadmap](ROADMAP.md)

The `.agents/` directory contains the repository-local Aether specifications,
skills, agents, templates, and validators used to maintain these documents.

## Status

Flow is in the **definition** phase. Current descriptions of Aniflow, Optiflow,
and Renderflow are grounded in their default branches as inspected on
2026-08-13. Import mechanics, workspace layout, crate names, and history
preservation remain intentionally open until the migration phase.

## License

MIT
