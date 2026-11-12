# Manual SRT publishing pilot

`pilot` is a fixed Flow preset over the existing native execution SDK. It composes
an Aniflow `music-video-source-v1` request, a Renderflow `canonical-build-v1`
manual-SRT request, and an optional Renderflow audio request. All stages select
one exact cohort and host target. There is no automatic lyric recognition.

Supply a `flow.manual-srt-pilot-request/v1` document:

| Field | Selection |
| --- | --- |
| `metadata` | `work_id`, `title`, `language` strings; for example Akashic |
| `manual_srt` | Exact `FilePin` of the manually prepared `.srt` original |
| `audio_source` | Exact `FilePin` of original song audio, or `null` |
| `video` | Existing native request selecting source video and Aniflow configuration |
| `timed_text` | Existing native request selecting only that SRT and Renderflow configuration |
| `audio` | Existing native request selecting only that audio source, or `null` |
| `run_directory` | Fresh private pilot directory as a `NativePath` |

Each native request retains its normal provider, descriptor, input, configuration,
preparation, cohort, output and budget pins. Native output directories and the
pilot directory must be separate trees with existing canonical parents. Planning
reads selected files without creating run directories or launching providers.
The native request schema is [here](../schemas/flow-native-request-v1.schema.json).

For timed text, use one source with explicit `format: srt`, native output root
`NATIVE_RUN_DIRECTORY/native/output`, and explicit offline/local-only requirements.
The existing Renderflow `lyrics` profile can select its native supported lyric
formats; bind `execution.lyrics.source_sha256` to the exact manual SRT and retain
its actual review state. Native planning and acceptance decide available targets.
For audio, select the original mix and explicit native derivative targets. No
automatic normalization, replacement mix, transcoding claim or distributor
approval is inferred by the preset. See Renderflow's `examples/lyrics` for its
domain configuration; Flow does not fork the conversion implementation.

```text
flow pilot-plan --request /private/akashic.json --output /private/pilot-plan.json --format json
flow pilot-approve --plan /private/pilot-plan.json --approved-by OPERATOR --acknowledge-trusted-unconfined --acknowledge-unqualified-candidate --output /private/pilot-approval.json --format json
flow pilot-run --plan /private/pilot-plan.json --approval /private/pilot-approval.json --format json
flow pilot-status --run-directory /private/akashic-pilot --format json
```

Stages run sequentially through the existing native supervisor and domain receipt
validators. The preset never decides success from process exit alone. A start
marker precedes each launch; failed/uncertain stages retain their native evidence.
Identify a partial native run explicitly with the existing `flow resume` command,
then call `pilot-continue` with the exact original plan and approval. Continue
reobserves completed stages and starts only stages that have never started; it
never guesses a provider run or replays a missing started stage.

```text
flow resume --run-directory /private/partial-stage --native-run-directory /private/partial-stage/native/EXACT_RUN --format json
flow pilot-continue --plan /private/pilot-plan.json --approval /private/pilot-approval.json --format json
```

The local `handoff.json` indexes exact accepted artifacts and native acceptance
receipts by stage, retains manual SRT/audio pins and metadata, and remains
`review_pending: true`, `upload_authority: false`, `release_qualified: false`.
It references retained native outputs; it is not a self-contained copied media
archive. Clean video, caption exports and original-audio derivatives keep their
native identities. The preset does not automatically burn captions into video,
attach Aniflow lyric review, create a human word/timing/color/audio review,
publish, upload, timestamp or sign anything. Those remain explicit existing
domain/review operations after native completion.

This source checkpoint used no private Akashic media and ran no build, test,
validator, CLI, provider, model, media, installation or publishing operation.
Qualification and final sweep remain open in the existing issues; #88 is last.
