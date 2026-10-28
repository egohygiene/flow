# Flow suite: the remaining path to usable tools

Updated October 28, 2026 (America/New_York), after the user-authorized PR merge
sweep. The [complete dependency map](flow-suite-dependency-map.md),
[visual issue atlas](flow-suite-completion.svg) and
[machine-readable snapshot](flow-suite-status.json) retain the full backlog.
This page explains which work gets the tools into use first.

All 16 PRs that were open at the start of the sweep are merged: 13 Optiflow,
3 Aniflow. Flow and Renderflow had no open PRs. Optiflow #151 additionally
refreshes the corpus lock binding after the dependency upgrades. All four PR
queues were observed empty before authoring this roadmap refresh.

## Where each repository stands

| Repository | Open issues | What exists | Main remaining work |
| --- | ---: | --- | --- |
| **Flow** | 31 | Extension/provider contracts, durable orchestration library, recovery foundations and a merged adapter for released read-only Optiflow v0.1.1 | Released Renderflow/Aniflow adapters; supported product CLI; real installed workflows; local-file profiles, packaging and release. |
| **Renderflow** | 27 | Rendering/CLI/SDK foundations, ordered-page PDF/EPUB, substantial merged creative/composition source | Finish locked baseline and document acceptance; qualify the merged capabilities; publish a verified integration release. Some broader composition/profile work remains. |
| **Optiflow** | 13 | Published read-only inventory/dedup planning; development PNG candidates, transaction foundations and partial video work | Usable video candidate production/validation; native transaction qualification; safe replacement and serial low-space folder processing. |
| **Aniflow** | 26 | Substantial temporal/media pipeline source, source-video recipe, upscaling/repair/recovery paths and synthetic demos | Remaining selected-profile entry/finishing/handoff work; accumulated qualification; immutable core and music-video releases. |

The **97 open issues are not 97 equal-sized features**. They include overlapping
epics, acceptance/qualification, optional capabilities and final audits. The
atlas marks **71 bounded completed issues NUKED** and preserves all 80 original
diagram nodes. Completion of an issue does not imply every platform or release
is qualified. No overall percentage is inferred from these counts.

Only Optiflow currently has a published release, and its latest v0.1.1 is
read-only. Source versions and merged PRs do not substitute for released,
verified tools. No supported product `flow` CLI exists yet.

## Three practical finish lines

| Finish line | Required outcome | Owning path |
| --- | --- | --- |
| **Try video optimization** | Generate a tiny input, produce one independently validated smaller candidate, compare/play it, retain the original | Optiflow [#146](https://github.com/egohygiene/optiflow/issues/146), then the selected-profile qualification/demo. |
| **Reclaim a video folder's space** | Recurse, process one file at a time, preserve recovery evidence, and dispose of originals only under explicit authority | Later #146 stages plus applicable native transaction/release work [#111](https://github.com/egohygiene/optiflow/issues/111)/[#93](https://github.com/egohygiene/optiflow/issues/93). PNG-first [#121](https://github.com/egohygiene/optiflow/issues/121) remains separately scoped. |
| **Use the suite from other projects** | Install independently versioned tools; declare ordered steps; inspect/run/resume; consume exact accepted artifacts | Provider releases → Flow [#51](https://github.com/egohygiene/flow/issues/51)/[#52](https://github.com/egohygiene/flow/issues/52) → [#53](https://github.com/egohygiene/flow/issues/53) → proofs/qualification → [#70](https://github.com/egohygiene/flow/issues/70)/[#71](https://github.com/egohygiene/flow/issues/71)/[#54](https://github.com/egohygiene/flow/issues/54). |

An optimization candidate and a source replacement are different milestones.
Serial processing still needs room for one candidate and bounded scratch;
physical free-space recovery can differ from encoded-byte savings.

## Recommended implementation order

The provider lanes can move in parallel. The order below is a planning
recommendation within existing issues, not a new dependency contract.

### Optiflow: prioritize the user's video-folder goal

1. **#146, first usable candidate:** finish provider memory/aggregate scratch
   policy, independent decode/preservation/lossy validation and versioned
   publication. Then complete the tiny synthetic inspect → candidate → compare
   walkthrough. The planner/observations/strict pipe capture are merged;
   candidate acceptance and an optimizer command are still absent.
2. **#111 → #93, native transaction readiness:** safe descriptor ACL adoption,
   remaining cross-volume/finalization work, disposable-data qualification and
   a qualified exact-dedup release. Merged same-volume Mac/APFS source alone
   does not finish this path.
3. **#95 → #121, PNG replacement and serial disposal:** connect accepted PNG
   candidates to qualified recovery, then complete explicit low-space folder
   operation. Keep restorable replacement separate from irreversible disposal.
4. **Later #146 stages:** serial recursive video batching and video-specific
   replacement/disposal after the appropriate transaction boundary exists.

#65 owns remaining corpus work; #136/#137 have merged source repairs with
execution acceptance outstanding. #135 is the ledger. Optional AI #133,
externally gated naming #115/ADR #60 and final audit #61 stay visible without
becoming prerequisites for the first source-preserving video trial.

### Aniflow: finish the selected music-video path

1. **#111:** finish recipe entry from an existing Upscayl spatial index, avoiding
   repeat upscaling. Target/bypass/preview source is already merged.
2. **#35 / #112:** settle the selected profile's audio offsets/tails, loudness
   observations and metadata semantics. The basic source-to-master/delivery
   recipe is merged, including the synthetic demo work.
3. **#113:** provide the exact accepted final-video evidence that Flow can pass
   to optional Renderflow packaging.
4. **#37/#38/#24 → #64:** finish selected support/corpus gaps and run the deferred
   refactor/qualification sprint, including the recorded cancellation failure.
5. **#10 / #40:** qualify and publish the core/music-video contracts, then Flow
   consumes them under #51/#75.

Watermark/provenance, destination export, subtitles, RIFE finishing, optional
AI and broader review/identity modes remain explicitly owned in the full map.
Do not wait for every optional lane before choosing a narrow useful release.
#36 is closed/completed despite stale older prose; native qualification remains
under its open owners. [Aniflow #117](https://github.com/egohygiene/aniflow/issues/117)
is the retained ledger.

### Renderflow: qualify and release the implemented path

1. **#458 / #459:** finish locked-baseline and documentation acceptance.
   Source fixes are merged; lock/lint/build/test/document acceptance is still
   owed when the deferred validation sprint begins.
2. **#419:** qualify the selected initial publication subset and publish its
   immutable integration candidate, with verified installation and provenance.
3. **Flow #52:** implement the released provider adapter in Flow.

The creative #422–430/#440 and composition #406 source should be qualified and
finished where their acceptance still requires work, not rebuilt from scratch.
Fixture depth #412/#413 remains. Broader composition, editorial/presentation,
semantic/AI/service capabilities and scheduled governance stay in the full map.
The initial ordered-page PDF/EPUB release need not wait for all of them.
[Renderflow #367](https://github.com/egohygiene/renderflow/issues/367) owns the
repository ledger; #421 owns creative integration acceptance.

### Flow: make independent tools one installed workflow

1. **#52 / #51:** consume qualified Renderflow/Aniflow releases. The existing
   #50 read-only Optiflow adapter is completed.
2. **#53 / #3:** ship the supported doctor/plan/run/status/resume experience and
   a real multi-tool vertical slice. #53's declared all-adapter dependencies and
   static-first acceptance wording still need an explicit closeout disposition.
3. **#32 / #33 / #34:** prove static and temporal installed workflows,
   interruption/recovery and compatibility. #32's Optiflow step remains
   read-only inspection, not optimization/replacement.
4. **#70 / #71 / #54:** simple local-file/folder profiles, installable suite
   packaging and the first immutable Flow candidate. #14's observability
   requirements need an explicit first-release disposition.
5. **#12 / #72 / #11:** final audit, supported stable capability set and suite
   epic closeout, with every unresolved optional issue visibly owned/disposed.

#15's proposed observability contract is merged; ADR/profile acceptance remains
open before #16's pilots. The #14–22 observability track stays explicit. Optional
mutating/PNG/music-video adapters #73/#74/#75, AMV preset #78, comic #10,
scenario docs #62 and AI #80 retain their scopes. The preset uses Flow's existing
contracts rather than introducing another orchestration engine.

Flow #70 captures ordering such as optimize → render or render → optimize.
Actual use requires compatible released operations and adapters; no current
general scenario CLI or automatic source-mutation authority is implied.

## Merge and evidence receipt

| Repository | Inspected integrated source | Merge sweep |
| --- | --- | --- |
| Flow | `58b06abc3f95da56df3b763e9ddc395656cc173a` | No prior open PR; this documentation refresh follows that source. |
| Renderflow | `c1d1bebfd48369d18e7c46295859d76e3f831499` | No open PR. |
| Optiflow | `741da2c96081518d421517df174776427a4f2e08` | #122, #127, #129, #130, #131, #139, #142, #143, #144, #145, #148–150 merged; #151 refreshes only the corpus lock binding. |
| Aniflow | `8008e0176f45b34d2a9e74d8a26d9b3fd374b81f` | #133–135 merged in order, retargeting children to main. |

Source/PR/issue/release inspection and exact-head merges were performed. The
only integration conflict was Optiflow ROADMAP.md; both native and video
histories were retained. No release/tag was created and no private media was
processed or deleted. Repository issue ledgers were refreshed while preserving
their histories and open acceptance criteria.

Tests, compiler/builds, dependency resolution, lint/format, product schema/drift
checks, native/provider/media execution and Actions checks remain deferred by
user request. The requested documentation diagram was rendered offline. That
rendering is not product validation. ADR not required: this update reconciles
roadmap observations and sequencing recommendations without changing runtime,
authority, preservation or public provider contracts.
