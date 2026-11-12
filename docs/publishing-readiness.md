# Inspect the publishing handoff

`flow publishing-readiness` reports the exact delivery package files, supplied
review state, missing metadata/destination copies and optional recorded audio
checks. It lists remaining operator actions without running a provider,
transforming media, creating approval or uploading anything.

The closed `flow.publishing-readiness-request/v1` request selects:

| Field | Meaning |
| --- | --- |
| `package_manifest` | FilePin for the exact current delivery `manifest.json` |
| `audio_summary` | Optional FilePin for the actual raw `flow.audio-readiness-summary/v1` result; omit it or use `null` before checks |
| `timeout_ms` | Bound for the complete read, at most one day |

Use the raw audio summary in the prior command envelope's `result`, as saved by
`audio-readiness-run --output`. Redirecting a command's whole machine envelope
does not produce that raw summary. Paths and FilePins must come from actual local
files/results; an example or hand-written positive flag is not execution evidence.

```bash
flow publishing-readiness --request "$PUBLISHING_READINESS_REQUEST" --format json
flow publishing-readiness --request "$PUBLISHING_READINESS_REQUEST" --output "$PUBLISHING_READINESS_REPORT" --format json
```

`task publishing:readiness REQUEST=/selected/request.json` delegates to the same
read-only command. It returns the existing partial outcome (exit 9): fresh owner,
qualification and destination/platform work remains explicit. It does not return
an all-green publishing certificate merely because local evidence can be read.

Every assigned audio/video payload and destination in a supplied summary must
match the current package, with no missing or duplicate row. Recorded quantities,
policy and state must agree with the exact retained native domain snapshot. Its
request/source/digest/unit/channel/availability/authority framing is checked by
the existing audio adapter boundary; Flow does not recompute LUFS or true peak.
A preview cannot invent measured or qualified quantities. Only an explicit
unchanged Archive master/original can omit a target check.

The report labels the numeric state and meter/provider flags as recorded. In
particular, `provider_release_qualified_in_summary` is a historical summary flag;
this report does not authenticate fresh opaque release evidence. Its
`current_owner_checks_performed`, `release_admission_reverified`,
`current_platform_compliance_verified` and `upload_authority` remain false.
Exact package bytes are reobserved; QC, musical quality and human identity are
not established by that byte check.

Unknown or changed package/core source mappings, stale summary/package pins and
inconsistent record claims refuse. Report output must be a new file outside the
package, selected originals, audio summary and known native/QC evidence roots.
It never rewrites the package, supplied checks or reviews. Operator-owned paths,
metadata and review notes remain private local output.

## Journey for the pilot

1. Prepare the selected publishing project and inspect its exact native plans.
   Supply manual SRT; automatic lyric recognition is not a prerequisite.
2. Separately approve and run the existing candidate or authenticated released
   pilot path. Preserve the clean master and distinct delivery identities.
3. Select accepted native artifacts, metadata and supplied review records for a
   private delivery package; inspect its exact copied-byte manifest.
4. Obtain the relevant Aniflow final-file QC for each selected media source.
   [Audio-check setup](audio-check-setup.md) resolves the installed owner row,
   writes plans only, and binds separately supplied exact approvals.
5. Explicitly run those current owner checks. If a goal is outside target or
   measurements are missing, address it through the owning Aniflow workflow.
   Gain candidates preserve originals and need new QC; AAC encoding, caption mux,
   metadata changes or any other new media bytes also require fresh final QC.
6. Inspect this report for outstanding words/timing/visual/audio/final review,
   metadata, destination and qualification work. Supplied reviews bind the exact
   package. A changed package manifest requires a new matching summary even when
   its audio payload happens to remain byte-identical.
7. Recheck current destination requirements and applicable release/native/meter
   evidence. Actual account publication is a later separate authorized operation;
   record its real URL/receipt only after it occurs.

The source drafts describe this journey; it has not been executed or qualified.
The new focused report/preview/argv and setup scope/roundtrip cases are authored,
uncompiled/unrun. No tests/compiler/build/dependencies/validators/native/media/
provider/model/SDK/CLI/task/installer/CI ran. No real media, approval, acquisition,
original replacement/disposal, signing, timestamping or publication occurred.
Existing release/native/reference/platform and creative review gates stay open.
Flow #88 remains last; known alignment cancellation/start-marker failure remains
unresolved with unproven cause. ADR-0019 remains proposed.
