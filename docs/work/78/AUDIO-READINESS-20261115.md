# Installed audio-readiness operator checkpoint

The existing installed setup → separate approval binding → current owner check
→ publishing-readiness path is now observed on authored synthetic media. The
final selected run passes **61 Flow workload calls**, two separate suite-status
observations and **15 exact refusal controls**. It uses the new separately pinned
CP4 handoff04 and its exact Flow-generated final video; no Flow runtime change or
new audio engine was required.

The [portable observation](audio-readiness-observed-20261115.json) and
[source/capture bindings](audio-readiness-bindings-20261115.json) retain exact
identities. This is a bounded checkpoint under Flow #78/#70, with the installed
provider prerequisite under #75/#71. Full parent acceptance remains open.

## What ran

The private delivery package has three payloads: the exact accepted clean master
assigned to Archive, the exact delivery video assigned to Archive and TikTok, and
the supplied manual SRT assigned to Archive. The two delivery-video assignments
require separate current checks. Only the unchanged clean Archive master uses
explicit preservation; its policy, quantities, result and native record remain
null and its qualification flags remain false. SRT does not become audio.

Actual `audio-setup-preview`, `audio-setup-prepare` and `audio-setup-status` resolve
and save the exact installed provider/configuration/plans without launching an
owner or creating approvals. Two later explicit `approve` calls supply the exact
reserved candidate execution acknowledgements. `audio-setup-bind` saves the raw
readiness request; preparation/binding does not create human review or release
authority.

`audio-readiness-preview` keeps both checks unmeasured. The explicit run performs
two fresh Aniflow policy checks, and `--output` saves the actual raw summary.
The matching fixture deliberately selects the observed finite synthetic loudness
as its comparison goal, with explicit tolerance and peak maximum. An independent
numeric comparison verifies the returned bounds and result. A separate policy
moves the loudness goal by ten LU and yields two genuine outside-target results.
These are selected technical comparisons, not platform requirements or mastering
recommendations.

Both summaries retain all five human review scopes as awaiting and meter/provider
qualification as false. Both runs correctly return partial exit 9. The later
`publishing-readiness` reports correlate the exact saved summary and package
bytes, while `current_owner_checks_performed`, `release_admission_reverified`,
`current_platform_compliance_verified` and `upload_authority` remain false. The
publishing report reads a historical snapshot; it does not refresh domain QC.

## Measured subjects

| Subject | Actual measured source | Executable SHA-256 |
| --- | --- | --- |
| Flow release package | `69eafaf74af8891bde1c7a769fb57dbca815476c` | `0de124114f96cab0256f09735b68ca56eecab646d608c051f1651892c20585e4` |
| Aniflow candidate | `f851f9cc0f68291329ebc34acc36b16c07b9f82e` | `d776fd120ad0a77b21271c162815546062434f10e4df93e2f9686c664d1a666c` |
| Renderflow candidate | `4fb2539fa686e966143be69b711ffeeccc59d3b4` | `d7b2f03670ed1262c9dfab67c9cf64a0314c7c9fb190d0ffc4a14e87ca96ca54` |

The v3 selection has six exact cohort rows over four physical tools. Aniflow's
music, final-video-QC and policy rows share one executable and compiled contract;
the actual released Optiflow v0.1.1 remains its separate installed subject.
Flow's earlier package/build and historical v2 context remain intact. A separate
complete Git input comparison binds the current `a6f01b9b…` context, including the
two explicit nonruntime paths `Taskfile.yml` and `config/local-ci.json`. This does
not rebuild/repackage Flow or transfer qualification to the newer context.

The new exact delivery SHA-256 is
`f38216769371c8e8218ca5d47e973991b86ca64731805ee24261352767f38513`.
Its selected 96,000-frame analysis view retains presented PCM SHA-256
`1fc7c404a80fc34828d4f33515b3b9f28c5f11164eb50157316f0937c2565e9f`
within all 96,256 complete decoded frames. Current checks preserve the full
nonpresented tail, original packets, native artifacts and source bytes.

## Refusal and readback evidence

The 15 named controls require the owning exit, diagnostic code, exact reason and
recovery scope; an unrelated failure cannot satisfy them:

- Missing/duplicate/unassigned payload/destination coverage and attempting to
  exempt a delivery derivative refuse before setup workspace creation.
- Missing, extra, wrong-location and wrong-plan approvals refuse binding without
  writing a readiness request.
- A stale external QC digest, wrong producer/artifact and omitted presented-window
  selection each reach their exact retained inner Aniflow state error. The first
  failed check preserves its attempt and stops the later destination check.
- A repeated current-check request refuses reuse of completed attempts.
- Changed file/policy summary claims and a separately valid second package
  context refuse historical publishing readback.

Four current native snapshot locators independently join the exact source FilePin,
canonical assessment digest, selected QC, actual quantities and policy bounds.
The final observation hashes 411 runtime inventory members and 122 direct workload
streams; its two reports and two suite-status calls are separately pinned. All nine repository helper
sources are pinned before and after execution, including explicitly loaded
`scripts/owner_reports.py` and `scripts/release/package.py`. Exact original
installation, project, source/QC and copied-package bytes remain unchanged.

Earlier runs01/02 passed their semantic workload but had incomplete explicit
helper-source lists. Their bytes remain preserved as superseded observations.
The accepted run03 uses the complete nine-source closure and the new CP4 handoff04;
no before-run identity is retroactively inferred for the earlier runs.

## Reproduce

First retain the actual CP4 source-candidate installation, generated project and
selected QC. The handoff contains private canonical locators; copying only the
JSON does not recreate its subjects. Choose a new output root with an existing
canonical parent:

```sh
python3 tools/check_installed_audio_readiness.py \
  --handoff /absolute/path/to/CP4-HANDOFF-04.json \
  --handoff-sha256 529c10a0b170d26a207dabdd6e66a3a8c53a507ad0b462ae11d102b0bfbdda9a \
  --work-directory /absolute/path/to/new-audio-readiness-observation
```

No provider acquisition, implicit normalization, signing, timestamp networking,
upload or Pages deployment is performed. The observation uses authored fixtures,
not private song/model assets. Actual supported-host/native/reference-meter,
release and creative acceptance remain separate. Flow #88 stays unchanged and
scheduled last; broad strict-Clippy debt is not claimed repaired by this path.
