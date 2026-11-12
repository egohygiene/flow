# Prepare a publishing project

Copy the inert [project template](../examples/publishing/project.json.in) into a
private project file. Select the installed suite root and exact installation
receipt digest, source video, manually supplied SRT, existing Aniflow source-entry
recipe, fresh workspace and requested destinations. Optional song audio selects
its original file, explicit format and native Renderflow target options. No
private source paths or lyrics are included in these repository examples.

```text
flow project-preview --project /private/song-project.json --format json
flow project-prepare --project /private/song-project.json --format json
flow project-status --workspace /private/song-work --record-sha256 EXACT_PREPARED_RECORD_SHA --format json
```

Preview is read-only. It reports observed source/recipe/options identities, exact
installed provider/descriptor and selected rows, requested destinations and pending
review. Installation is selected through the public distribution SDK, not ambient
PATH or sibling checkouts. Defaults use only a unique installed implemented Flow
row for `music-video-source-v1` or `canonical-build-v1`. If multiple directions
match, select exact `rows.video`, `rows.timed_text` and optional `rows.audio` IDs;
missing or ambiguous profiles refuse. A receipt's publisher authentication is an
installation-time observation, separately from native readiness.

Prepare explicitly creates the fresh private workspace, immutable configuration
snapshots and existing native/pilot plans. It copies the selected Aniflow recipe
bytes unchanged. The Renderflow manual-SRT configuration uses its existing lyrics
profile with unreviewed words/timing and an exact source digest. The optional
[audio options template](../examples/publishing/audio-options.json) selects native
targets/execution settings; Flow supplies only the single chosen source and fresh
output location, with network and AI denied. Normalization or other audio
processing stays an explicit domain choice, never inferred from a destination.

The workspace contains `configuration/`, the empty native-run parent `native/`,
`pilot-plan.json` and a last `prepared-project.json` record. It does not create
provider runs or the reserved `pilot/` run. Generated configuration and every
original/installation identity are pinned. Status requires the printed record
digest and reobserves those identities. It reports configuration readiness only;
use the corresponding pilot status commands for actual run progress. A changed
source, options, installation or preparation requires a new reviewed project
workspace. Partial preparation is retained and never overwritten automatically.

Destinations are requested handoff goals (`archive`, `soundcloud`, `distrokid`,
`tiktok`), not proof of current platform compliance, automatic crop/trim, human
approval or permission to publish. The later delivery-package operation selects
actual accepted final payloads and explicit reviews. No transcription, automatic
lyric recognition, provider/tool/model acquisition, creative approval or upload is
introduced. The selected native owners retain conversion and temporal algorithms.

Use the printed plan path with the existing candidate pilot approval/run path,
or the independently authenticated released path when its real evidence exists.
No provider, CLI/SDK, model, test, build, compiler, validator, CI or publishing
operation was executed for this source checkpoint. Flow #88 remains last.
