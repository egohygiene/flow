# Distribution sprint handoff — 2026-11-11

User authorized seven source checkpoints on separate draft branches after merging
the preceding six drafts. Those six were merged in dependency order: Flow #98,
Aniflow #213, Renderflow #503, then Flow #99, #100 and #101. New checkpoints remain
drafts for sync/review; their publication does not authorize their merge.

| Checkpoint | Draft / source branch | Existing issue owners |
| --- | --- | --- |
| Aniflow three-host native and portable library bundle | Aniflow #214, `codex/212-native-bundles-20261111` | Aniflow #212 |
| Renderflow native v2 package/installer handoff | Renderflow #504, `codex/419-release-v2-20261111` | Renderflow #419 |
| Flow compiled descriptor and release candidate package | Flow #102, `codex/54-release-package-20261111` | Flow #54 |
| Pinned independent suite install/bundle/activate/rollback/uninstall | Flow #103, `codex/71-suite-install-20261111` | Flow #71, #70, #72 |
| Existing preparation SDK connected to main commands and native requests | Flow #104, `codex/80-preparation-cli-20261111` | Flow #80, #70 |
| Manual-SRT video/text/optional-audio pilot and local handoff | Flow #105, `codex/78-manual-srt-pilot-20261111` | Flow #78, #75, Aniflow #212 |
| Explicit authenticated evidence and released native admission | `codex/72-authenticated-evidence-20261111` | Flow #72, #51, #52 |

Provider drafts base on their independent mains. Flow drafts stack in the table's
order beginning at #102; review/merge in that order when later authorized.
The #102 follow-up preserves the original LICENSE copyright exactly.

All source is unexecuted. No tests, compiler, build, dependency resolution,
format/lint/schema/docs/drift/product validators, package/installer invocation,
CLI/SDK/native/provider/model/media/browser/container execution, hosted CI,
acquisition, signing, timestamping or publication ran. Authored synthetic cases
are future checks, not qualification. Source and Git transport/manual review are
the only evidence from this sprint. Never enable paid CI implicitly.

Actual release/native owners still need to produce real artifacts and native
records, publish approved releases, and attest the new evidence records. No new
release workflow was enabled. Candidate acknowledgements remain explicit. The
evidence adapter requires separately selected host trust; report JSON cannot
choose the verifier/policy or construct verified authority. Its ADR-0018 remains
proposed. Read the three new preparation/pilot/evidence guides for exact limits.

Keep Flow #88 last and its original 45-ticket title/body/history/criteria intact.
The known alignment cancellation/start-marker failure is unresolved; its cause
is unproven. Aniflow #35 stays closed; Optiflow #95 stays open. Aniflow #212 is its
single remaining owner after eighteen predecessor scopes were transferred, not
declared qualified. Do not reopen historical tickets from old snapshots. No issue
is closed by these draft PRs. Leave unrelated Optiflow Dependabot drafts alone.

The next sync should review the seven source checkpoints and then choose the
remaining release/native qualification or cross-suite issue work from live
trackers. Missing targets and unsupported coverage remain visible; nothing in
this sprint establishes an all-green release matrix or publishing readiness.
