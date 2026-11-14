# Static publication projects

`flow.static-publication-project/v1` is a bounded preset for explicit local
Renderflow publication branches. It supports one default/source locale plus
separately supplied locale branches, each with ordered source FilePins, declared
formats, a pinned `flow.renderflow-options/v1` document and required native target
IDs. Options keep Renderflow's own `targets` and `execution` vocabulary. Preview
is local/read-only; unknown format/target combinations remain owner decisions.

An ordered collection also supplies each member's `media_type` and, when its
native policy requires it, explicit `geometry` (`width`, `height`, `unit` and
optional `margin`, `bleed`, `safe_area`). These values enter the native source
declaration and identity; Flow does not infer page geometry or MIME from a suffix.
Single-file sources keep these fields optional.

For native fixed-layout EPUB, the pinned options document also supplies a
`publication` JSON object using Renderflow's publication contract: title,
contributors, language/date, geometry, artwork/alt text, rights and accessibility.
Native validation stays with Renderflow. Artwork paths must name explicitly
selected original sources, either absolutely or relative to the options file's
parent without parent traversal. Preparation rebinds those exact paths to the
selected private copies; supplied words, rights and accessibility remain unchanged.

The project selects an exact installed receipt, optional unambiguous Renderflow
`canonical-build-v1` row, fresh workspace, explicit tool directories, native
limits and whole-operation deadline. At most 16 locale branches, 512 files per
branch and 2048 file references total are admitted. A single source stays a file;
multiple files become an explicit ordered collection. Source-byte totals count
references across branches and are not measured disk/peak-memory forecasts.

`maximum_staged_bytes` separately bounds the sum of all private ordered-member
copies before any workspace is created. Its default is 256 MiB; explicit values
range from 0 to 64 GiB. Reusing an original in another locale counts again because
each branch creates another copy. Single-file branches stage zero, so a zero
budget can allow them while refusing every nonempty collection. This budget does
not limit native outputs, metadata or total/peak disk consumption; those retain
their existing bounds. Omitting the field, or choosing its default, preserves the
earlier project identity; a nondefault limit enters its identity and readback.

Preview exposes original pins and planned native input locations without writing
files. Preparation uses the existing bounded copier to stage ordered members
under `configuration/source-NN/`, allowing native root-relative locators without
symlinks or parent traversal. Native plans bind those copied bytes; project
readback continues to reobserve original source/options/installation pins too.
Changed originals, copies or metadata need a new reviewed plan. Older prepared
configurations that differ from the corrected mapping refuse readback; they are
not silently migrated or replayed.

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
Those remain existing scope. The [executed synthetic pilot](work/70/STATIC-PUBLICATION-PILOT.md)
now proves both separate single-locale EPUB/PDF policies on an exact installed
Renderflow candidate. Current native policy cannot combine print PDF and fixed
EPUB in one route: use separate project workspaces for this proof. This is not
exhaustive multilingual composition or a released suite. ADR-0019 remains
proposed; Flow #88 remains last and untouched.
