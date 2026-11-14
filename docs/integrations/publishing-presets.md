# Reusable manual-SRT publishing presets

Relates to #78. Reuse explicit installed-provider/configuration settings across
Akashic, Dingus and Exist while selecting each project's video, manual lyrics,
optional audio, metadata and workspace separately. This is configuration
materialization over the existing project/native owners, not another workflow
engine or permission to process/publish those works.

Start with `examples/publishing/preset.json.in` and `selection.json.in`. They
contain placeholder paths and an intentionally unfilled installation digest;
replace them with your exact retained installation and selected local paths.
Do not invent an installation/release or copy a synthetic test digest.

```bash
flow project-resolve --preset /private/manual-amv.json --selection /private/akashic-selection.json --output /private/new-akashic-project.json --format json
task project:resolve PRESET=/private/manual-amv.json SELECTION=/private/akashic-selection.json OUTPUT=/private/new-akashic-project.json
flow project-preview --project /private/new-akashic-project.json --format json
```

Resolution writes a fresh ordinary `flow.publishing-project/v1` file and reports
the exact file pin plus semantic preset/selection/effective-project identities.
It reads no media, inspects no provider and creates no approval. Preview and
later explicit preparation obtain current source/provider pins through their
existing owners. Execution, recovery and human review stay separate.

The selection must supply video, manually supplied SRT, metadata and workspace;
audio is explicitly optional. It may replace `destinations`, `rows` or
`video_configuration`. Each override replaces that whole named value. For
example, a supplied `rows` object with `timed_text: null` does not inherit a
hidden preset row. No arbitrary deep merge, automatic lyric detection,
normalization, watermarking, model download, signing or upload is introduced.
Effective configuration changes produce a different project identity; formatting
changes alone are not a new semantic configuration. Retain the reported
resolution when reviewing how a project was assembled.

Three meaningful resolution/refusal regressions are authored, uncompiled/unrun.
No tests/builds/dependencies/validators, product CLI/SDK, native/provider/media,
model or CI execution ran. #78 retains its real released orchestration,
selection/quality/recovery and human-review acceptance. Flow #88 stays last.
