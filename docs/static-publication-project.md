# Static publication projects

`flow.static-publication-project/v1` is a bounded preset for explicit local
Renderflow publication branches. It supports one default/source locale plus
separately supplied locale branches, each with ordered source FilePins, declared
formats, a pinned `flow.renderflow-options/v1` document and required native target
IDs. Options keep Renderflow's own `targets` and `execution` vocabulary. Preview
is local/read-only; unknown format/target combinations remain owner decisions.

The project selects an exact installed receipt, optional unambiguous Renderflow
`canonical-build-v1` row, fresh workspace, explicit tool directories, native
limits and whole-operation deadline. At most 16 locale branches, 512 files per
branch and 2048 file references total are admitted. A single source stays a file;
multiple files become an explicit ordered collection. Source-byte totals count
references across branches and are not measured disk/peak-memory forecasts.

Locale labels do not translate content or establish translation quality. Supply
the actual chosen locale sources/options. The default metadata language needs its
own branch. Rights, visual, translation, accessibility and final review remain
human work. AI, remote/network behavior and custom native transforms are outside
this first local preset; unsupported requested behavior refuses rather than drops.

```bash
flow publication-project-preview --project "$STATIC_PROJECT" --format json
flow publication-project-prepare --project "$STATIC_PROJECT" --format json
flow publication-project-status --workspace "$STATIC_WORKSPACE" --record-sha256 "$STATIC_RECORD_SHA256" --format json
```

Preparation copies exact local options/source bindings into configuration/native
plans, using private fresh directories and no-clobber files, then writes the final
`prepared-static-project.json` record. It never invokes Renderflow, creates
approval or accepts partial retained state. A failure preserves its recovery root.
Use the returned native plan files through the existing explicit candidate or
authenticated released approve/run/status/resume paths. There is no new runner.

Static status reobserves current source/options/provider/plan identities and, if
available, existing native acceptance and target IDs. Missing required targets
remain listed even when a native run completed. Unknown or uncertain native state
does not become success. Status retains creative review pending, no reverified
release admission and no upload authority. Config changes need new plans/review.

This checkpoint does not finish #10's exhaustive multi-holon comic proof,
automatic locale production, cost/resource forecasts or governed human gates.
Those remain existing scope. The focused source refusal case and contracts are
authored, uncompiled/unrun. No compiler/tests/validators/native/SDK/CLI/task/CI or
actual media, review, acquisition, signatures/timestamps/publication were run.
ADR-0019 remains proposed; Flow #88 remains last and untouched.
