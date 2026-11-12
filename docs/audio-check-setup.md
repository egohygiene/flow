# Prepare audio checks from an installed provider

`delivery::audio_setup::preview` resolves the existing delivery package, its
original final-file locators, the selected installation and supplied Aniflow
QC/policy requests. It reads local files only. It creates no directories,
approval, provider process or audio measurements. Aniflow owns the policy and
current-QC comparison; Flow does not invent a platform loudness target.

The closed `flow.audio-check-setup/v1` request contains:

| Field | Selection |
| --- | --- |
| `package_manifest` | Exact delivery `manifest.json` FilePin |
| `installation` | Existing installation root and receipt digest, required for checks; `null` is allowed for archive-only preservation |
| `workspace` | Fresh private preparation bundle outside package, originals, providers/tools and Aniflow QC/native evidence |
| `selections` | Every media payload/destination, each with an explicit choice |
| `tool_directories` | Explicit existing canonical tool search directories |
| `limits` | Existing `flow.native-request/v1` limits for each plan |
| `timeout_ms` | Bound for the complete operation, at most one day |

A `check` choice supplies an `owner_request` FilePin for an existing
`aniflow.audio-policy-check-request/v1` and an optional exact installed `row_id`.
The installed Aniflow descriptor/cohort must advertise `audio-policy-check-v1`;
an old executable or an ambiguous row is not silently substituted. Its final
media source must still exist and match the exact bytes copied into the package.
The original native/QC request paths must be canonical and available. A later
Aniflow check still reobserves the actual QC owner; preview does not qualify it.

`keep_archive_master` requires a reason and is restricted to an Archive-assigned
clean master or unchanged original audio. It never exempts a platform derivative.
All assigned media/destination pairs must be covered once. Lyrics, artwork and
supporting documents are outside audio check coverage; manual SRT stays manual.

Preview lists each policy, source/configuration pin, installed provider/descriptor
and reserved configuration/plan/approval/native paths. It never infers approval
from a chosen path. Preparation, integrity readback, approval binding and command
delivery are separate checkpoints on top of this read-only resolver.

This source checkpoint and its focused coverage/scope cases are authored,
uncompiled/unrun. No tests, product validators, provider/native/media execution
or CI were run. Release/native/reference-meter/platform and musical qualification
remain with their existing owners. ADR-0019 remains proposed; Flow #88 stays last.
