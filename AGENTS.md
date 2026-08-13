# Flow Contributor Instructions

## Start here

Before changing architecture or source boundaries, read:

1. `docs/architecture/README.md`
2. `docs/architecture/foundation/SYSTEM.md`
3. `docs/architecture/foundation/ARCHITECTURE.md`
4. The affected holon's contract under `docs/architecture/holons/`
5. `docs/architecture/governance/DECISIONS.md`
6. The applicable horizon in `ROADMAP.md`

Use the repository-local specifications and skills in `.agents/`. Their
supported validation commands are documented in `.agents/README.md`.

## Architectural rules

- Preserve originals and evidence before transformation.
- Keep Aniflow, Optiflow, and Renderflow independently useful as Rust libraries
  and CLIs.
- Do not add direct dependencies between sibling holons.
- Put cross-holon planning, compatibility, execution state, recovery, and suite
  provenance coordination in Flow.
- Put specialized algorithms and domain validation in the owning holon.
- Use typed public APIs or versioned structured CLI contracts.
- Invoke external programs through executable-plus-argv APIs, never interpolated
  shell strings.
- Treat pipeline definitions and external outputs as untrusted input.
- Never report completion solely because a process exited successfully or a
  file exists.
- Never upload content, mutate originals, sign provenance, or perform destructive
  collection actions without explicit product behavior and user authority.

## Change method

Ground claims in named revisions and executed validation. Update the canonical
architecture owner before downstream documents. Record significant boundary,
safety, contract, provenance, release, or workspace decisions. Keep pull
requests scoped to one independently reviewable outcome.

Use long-form CLI arguments in documentation and examples. Preserve paths with
spaces and Unicode. Structured output must be stable, versioned, and free of
interleaved human presentation.
