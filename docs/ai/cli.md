# Local AI preparation through Flow commands

The main Flow command SDK now delegates to the existing selected-file setup
owner. It adds no runtime, inference client, router, model installer or socket
probe. Existing profiles, private preparation and the separate provisional
`flow-ai-setup` utility retain their formats and behavior.

```bash
flow ai-doctor --profile "$AI_PROFILE" --preparation "$AI_PREPARATION" --format json
flow ai-prepare --request "$AI_SETUP_REQUEST" --output-directory "$AI_SETUP_DIRECTORY" --format json
flow ai-setup-status --directory "$AI_SETUP_DIRECTORY" --marker-sha256 "$AI_SETUP_MARKER_SHA256" --format json
```

`ai-doctor` reads an explicit profile and optional preparation using the existing
bounded no-follow document reader. Disabled profiles do not read preparation.
The portable doctor contains logical IDs and fixed next actions, not asset paths,
endpoint/credential dumps or raw model/source text. No model/service is contacted,
including a configured loopback endpoint. Missing/changed/unavailable assets stay
typed gaps; a supplied hard cost ceiling or unsupported modality stays refused.

`ai-prepare` observes only the existing files selected in
`flow.ai-local-setup/v1`. Select an existing empty canonical directory with Unix
private permissions (0700), outside request and all declared asset paths. Setup
writes its existing immutable request/profile/preparation/doctor members, then
the last `setup.json` marker. Partial documents remain on error; no retained
bundle or source file is adopted, overwritten, removed or downloaded.

The result wraps the existing bundle status and actual marker FilePin in
`flow.ai-setup-command/v1`. Use that real marker digest for `ai-setup-status`,
which reobserves member closure, the profile/request identity and current selected
asset bytes. The wrapper/marker locators are private local output; they are not
portable telemetry or authenticated runtime admission.

Matched/prepared bytes return the command's **partial outcome (exit 9)**: native
model identity, code/model license review, hardware/resource limits and actual
offline isolation remain unqualified. Disabled returns complete; unavailable,
changed, missing, refused or incompatible setup remains unavailable. None grants
inference, native provider, creative review or publication authority. No implicit
service start, remote/paid fallback or caller environment/secret inheritance exists.

Task aliases are `ai:doctor` (`PROFILE`, optional preparation uses the CLI),
`ai:prepare` (`REQUEST`, `OUTPUT`) and `ai:status` (`DIRECTORY`, `MARKER_SHA256`).
Each delegates to this same command surface. Asset acquisition, actual runtime
qualification and released consumer adoption remain explicit later work under
#80 and the existing provider owners.

The boundary/argv case and schemas are authored, uncompiled/unrun. No test,
compiler/build/dependencies/validators, model/native/SDK/CLI/task/installer or CI
was run. ADR-0015 remains proposed and final Flow #88 remains untouched and last.
