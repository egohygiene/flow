# Installed presented-window QC — November 15, 2026

A fresh four-tool candidate installation now executes a new synthetic Flow
project and the existing final-video QC facade with explicitly selected
`qc.presented_window: "exact_presented_audio"`. The separate analysis WAV contains
exactly 96,000 presented frames. All 96,256 complete decoded frames, including
the 256-frame nonpresented tail with 512 nonzero channel values, remain intact.
The omitted-selection control retains the full meter and creates no analysis view.

This closes the scoped integration gap under #75, coordinated with #78/#71.
It does not close their released-provider, native-host or full product acceptance.
The upstream Renderflow precision and Aniflow recovery work have independent
proofs; this checkpoint exercises their actual captured bytes through Flow.

## Exact installed subjects

| Tool | Measured source | Executable SHA-256 | Bytes |
| --- | --- | --- | ---: |
| Flow | `69eafaf74af8891bde1c7a769fb57dbca815476c` | `0de124114f96cab0256f09735b68ca56eecab646d608c051f1651892c20585e4` | 6,578,208 |
| Aniflow | `f851f9cc0f68291329ebc34acc36b16c07b9f82e` | `d776fd120ad0a77b21271c162815546062434f10e4df93e2f9686c664d1a666c` | 41,715,264 |
| Renderflow | `4fb2539fa686e966143be69b711ffeeccc59d3b4` | `d7b2f03670ed1262c9dfab67c9cf64a0314c7c9fb190d0ffc4a14e87ca96ca54` | 20,151,576 |
| Optiflow | released v0.1.1, `b82599a2231e997d42fd9f26f4b59587f4ae14cf` | `46af9399f3785607f835d9a805a8daa42c93597772c6ee812911615ea4b76280` | 11,098,688 |

Aniflow's installed descriptor is the separately pinned canonical projection of
the actual CLI result: SHA `939cfc732fed34775a4bb1b8618b73009f42dc29e92e83fcc649123f2aaa3027`,
12,538 bytes. The unmodified raw envelope (`b93ed244…`) and initially captured
unordered result (`a31b52fd…`) retain different physical identities. An explicit
projection receipt verifies parsed result equality without changing fields or
rewriting the immutable build record. Renderflow's compiled descriptor remains
`199bf790b2d78e7706b1e24b3e0fdb189b8286eb231f8be1c8da503d82922f58`, 372,295 bytes.

Both providers use their actual published source commits above. Aniflow's
optimized-dev build reused its warm cache; the subsequent build was fresh-cache
reuse, not an independent rebuild. Renderflow used its Debian Rust 1.94 release
cache. Later observer/docs tips are distinct from compiled source identities.

## Installation and source-context boundary

The new explicit v3 selection admits four physical binaries and exactly six
cohort rows. One Aniflow row declares `music-video-source-v1`, one
`final-video-audio-qc-v1`, and one `audio-policy-check-v1`. All three share the
exact executable/source/contract pins. The existing installer still owns copying
and immutable installation membership; no second installer or native runner is
introduced. The missing-acknowledgement refusal precedes destination creation.

The original Flow package and sealed two-provider cohort remain unchanged.
The historical v2 product projection still names selected source `2b18f11e…`.
A separate [current-context observation](evidence/20261115/audio-context.json)
binds actual current main `a6f01b9b0bfdf985027f3bcd8e0d729c20e1691f`, tree
`de1a2b1938a91409e7bbf54b518e7dfc7777419f`: 73 changed paths are confined to
`docs/`, `tools/`, `Taskfile.yml` and `config/local-ci.json`. The complete remaining
493-entry tracked mode/type/blob map is equal, and all 122 literal include
references retain the same exact target objects. Source/Cargo/build references
do not embed the changed control files. The closed new context schema retains
both inventories and hashes; Rust/Cargo/schema/other-config changes, changed
include evidence or an asserted source-authentication flag refuse.

This explains reuse of the measured source69 executable. It neither recompiles
current main nor reseals the original package for the newer provider cohort.
The old v1/v2 parsers and earlier installation/package/selection bytes retain
their original meaning and hashes.

## Actual observations

The final accepted observation runs 27 Flow CLI calls in total: three installation
calls, 22 project/QC calls and two before/after suite-status calls. Of the 22
project/QC calls, 19 complete and three intentionally refuse with the exact
`stale_plan` code/message and retained recovery directory. Each stale mutation is
followed by a successful restored readback. Thirty focused Python observer cases
pass, including existing v1/v2 admission regressions and saved Float32 WAV controls.

- A new two-stage project runs installed Aniflow video and installed Renderflow
  manual lyrics. It creates distinct clean-master/delivery artifacts and all eight
  selected lyric formats from two manually supplied SRT cues. The authored input
  clip is a reused redistribution-safe synthetic fixture; all project/provider
  execution and output namespaces are new. No private song or model is used.
- Both selected and omitted-selection QC go through Flow's existing native
  `plan` → separate `approve` → `run` → `status` → completed `resume` commands.
  Full request/source/artifact identities, direct and resumable intent hashes,
  domain acceptance and the actual saved QC report are independently joined.
- An independent RIFF reader admits only the observed Float32 `fmt` sizes 16 or
  18 with zero extension. It verifies every selected sample byte against the
  complete decoded prefix, finite samples, frame counts and the exact authored
  zero-origin two-second clock. AAC priming, block count, final presented samples,
  decoded extent and nonzero tail arithmetic remain separately visible.
- The selected WAV's PCM hash is `1fc7c404a80fc34828d4f33515b3b9f28c5f11164eb50157316f0937c2565e9f`.
  The complete PCM hash is `b70f04f7cd3c64fbb6c80784e69f5d10ef29c5df3643695dfe04be47d0c2a4ee`.
  Selected and legacy runs retain exactly equal full-meter observations. The
  selected numeric observations remain unqualified meter evidence.
- Changed window selection, final media and QC report bytes refuse current
  retained status. Restoring each owned synthetic subject restores the positive
  result. Completed resume does not repeat work or change retained native state.
  This status observation does not replace a fresh owner policy check.

The final report retains 511 relative artifact/command-state file hashes,
including 66 per-call stdout/stderr/exit files and four suite-status streams.
All nine executed repository Python helpers are bound before and after the run,
including the importlib-loaded file-I/O and package owners omitted by a plain
`sys.modules` traversal. Their complete closure was independently reviewed.

Private attempts 01 and 02 stopped on observer assumptions about omitted optional
fields and valid `fmt18` WAV headers. Attempt 03 passed semantic checks but did
not capture both dynamic helper pins. Those bytes/streams and that earlier
handoff remain retained and superseded. Final 04 uses the frozen complete
helper closure and its own exact subject/handoff; no earlier claim is retrofitted.

Portable records:

- [Installation subjects and preserved package](evidence/20261115/audio-installation.json).
- [Accepted project/QC observation](evidence/20261115/presented-qc.json).
- [Exact observer source and command identities](evidence/20261115/audio-observer-source.json).

## Reproduce the scoped workflow

Supply captured tools, their exact descriptors/build observations, the original
Flow package and explicit finite local evidence. Use fresh private directories;
the observers acquire no tools, models or network resources:

```sh
python3 tools/observe_audio_candidate_context.py \
  --repository /absolute/path/to/flow \
  --output /private/new-current-context.json
python3 tools/prepare_audio_candidate_installation.py \
  --inputs /private/explicit-audio-install-inputs.json \
  --output /private/new-installed-candidate
python3 tools/check_installed_presented_qc.py \
  --selection /private/new-installed-candidate/selection-v3.json \
  --source /private/authored-synthetic.mp4 \
  --recipe /private/pinned-source-recipe.json \
  --work-directory /private/new-project-and-qc \
  --handoff /private/new-cp4-handoff.json
```

The selected proof profile uses the exact authored two-second AAC/48-kHz stereo
fixture; this command is an observation driver, not a general arbitrary-video
converter. The installed Flow commands and owning providers remain the product.
Explicit candidate acknowledgements authorize mechanical local execution only.

No original-source mutation, gain, automatic lyric recognition, delivery trim/pad, source
disposal, hosted CI dispatch, native Mac/physical-volume test, signing, release,
Pages deployment or content upload was selected. Meter/provider/host/platform
qualification and genuine human words/timing/visual/audio/final-package reviews
remain pending. Existing #75/#78/#71 acceptance and final Flow #88 remain open.
