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
from a chosen path.

`delivery::audio_setup::prepare` explicitly creates a fresh private bundle. It
copies the exact supplied owner-request bytes, writes the setup intent and
delegates each plan to the existing `native::prepare`. It reobserves sources,
configuration, installation and package identity before writing the final
`prepared-audio-checks.json` record. The result contains that record's FilePin and
each exact native plan and proposed approval path. No approval file is generated.

Configuration and plans are immutable bundle members. The empty `approvals/`
directory is reserved for separately supplied operator authority; `native/`
reserves later existing native attempts. Setup does not create check workspaces
or invoke a provider. A failed operation preserves its partial root and reports
that recovery directory. It never adopts or repairs that root; use a fresh
workspace for another preparation.

`delivery::audio_setup::status` selects the workspace and exact final record
digest. It reconstructs the local selection and verifies source/configuration/
installation/package pins, native plans and exact immutable core membership.
Missing, changed, unknown or symbolic core members refuse and remain preserved.
`approvals/` and `native/` are separate later-owner namespaces; setup status does
not adopt or validate their contents as execution evidence. Status neither runs
Aniflow nor rechecks the QC owner.

For each prepared check, separately obtain its existing exact native candidate
or released approval at the preview's reserved approval path. No preparation or
binding call creates one. A closed `flow.audio-check-approvals/v1` document lists
`payload_id`, `destination` and the existing audio-readiness `authority` containing
that approval's FilePin. Archive-only preservation needs an empty approval list.

`delivery::audio_setup::bind` consumes that complete, nonduplicated set and
returns the existing `flow.audio-readiness-request/v1`. Approval pins and exact
plan identity are checked. `save_binding` writes only outside the bundle and all
selected package/source/provider/QC evidence. Missing, extra or wrong-plan
approvals refuse. Binding a released approval does not authenticate it: the
existing audio-readiness run must still obtain current opaque release evidence
and its ordinary explicit verification acknowledgement before any invocation.
The resulting request uses the existing read-only owner-check runner; there is
no new process supervisor or acceptance database.

## Commands

These examples describe a later explicitly selected local workflow. Paths and
digests come from the operator's real files and command results; no sample pin
or review is evidence. None of these commands was executed for this source work.

```bash
flow audio-setup-preview --request "$AUDIO_SETUP_REQUEST" --format json
flow audio-setup-prepare --request "$AUDIO_SETUP_REQUEST" --format json
flow audio-setup-status --workspace "$AUDIO_SETUP_WORKSPACE" --record-sha256 "$AUDIO_SETUP_RECORD_SHA256" --format json
```

After inspecting the returned native plans, supply their approvals through the
existing [candidate](cli/native-candidates.md) or separately authenticated
released path. Collect the actual approval FilePins into the approval-set
document. Setup commands have no `--approved-by`, execution or network switch.

```bash
flow audio-setup-bind --workspace "$AUDIO_SETUP_WORKSPACE" --record-sha256 "$AUDIO_SETUP_RECORD_SHA256" --approvals "$AUDIO_SETUP_APPROVALS" --output "$AUDIO_READINESS_REQUEST" --format json
flow audio-readiness-preview --request "$AUDIO_READINESS_REQUEST" --format json
```

Only a later explicit `audio-readiness-run` performs owner checks. Released
choices require that command's full existing verification request, host policy,
verifier pins and explicit network acknowledgement when selected; setup does not
supply them. Every byte-changing gain, encode, caption mux or metadata edit needs
fresh final-file QC and another policy check. An old setup/check cannot qualify
new media bytes. A successful setup/bind command means that local operation
completed, not that audio, human review, platform compliance or publishing is ready.

Equivalent task aliases are `audio:setup:preview` / `audio:setup:prepare` with
`REQUEST`, `audio:setup:status` with `WORKSPACE` and `RECORD_SHA256`, and
`audio:setup:bind` with those two plus `APPROVALS` and `OUTPUT`. Each delegates
to the same Flow command, without installing anything or running extra checks.

These source checkpoints and their focused coverage/scope, archive-only roundtrip
and stale-pin-before-write cases are authored,
uncompiled/unrun. No tests, product validators, provider/native/media execution
or CI were run. Release/native/reference-meter/platform and musical qualification
remain with their existing owners. ADR-0019 remains proposed; Flow #88 stays last.
