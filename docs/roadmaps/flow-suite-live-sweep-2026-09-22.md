# Flow Suite live-sweep checkpoint — 2026-09-22

Suite coordinator: [flow#11](https://github.com/egohygiene/flow/issues/11)

## Purpose

This checkpoint preserves the live audit state for Flow, Optiflow, Renderflow, and Aniflow independently of any chat session. The canonical execution coordinator remains flow#11; this file is a reviewable repository snapshot, not a second issue tracker.

## Verified repository state

| Repository | Verified main | Open issues | Open pull requests | Immediate state |
| --- | --- | ---: | ---: | --- |
| Flow | [3d854c7](https://github.com/egohygiene/flow/commit/3d854c79756b1dee5d3ca267a08ae27b76673a4b) | 30 | 2 | Draft PR #47 is based on main; stacked draft PR #48 depends on #47. |
| Optiflow | [7de8483](https://github.com/egohygiene/optiflow/commit/7de8483b64387542a05214004c19a9cd05628908) | 13 | 0 | Read-only v0.1 foundation is shipped; the removable-media and transaction chain is explicit. |
| Renderflow | [170c3d5](https://github.com/egohygiene/renderflow/commit/170c3d5985af073d3da8f6ac2e0c4b639cfd2b3e) | 17 | 0 | Ordered publication collections, PDF/EPUB generation, validation, and release truth are explicit gaps. |
| Aniflow | [47c9798](https://github.com/egohygiene/aniflow/commit/47c9798e78fd9a268e4584974948a11e81d9e7bb) | 9 | 0 | Provider SDK/Pipeline v3 foundation is merged; temporal correctness and delivery evidence remain. |

## Gap reconciliation

The live issue surfaces already contain the missing bounded work found by the sweep, so this checkpoint does not create duplicate issues.

- Flow: #49 through #54 cover durable run state, released-provider adapters, supported CLI closure, and the first immutable integration-candidate release.
- Optiflow: #87 through #96 define the operator-ready removable-drive path, exact-duplicate transaction safety, quarantine recovery/finalization, and lossless PNG optimization.
- Renderflow: #415 through #419 define ordered collections, deterministic print-interior PDF and fixed-layout EPUB production, independent validation, and an immutable integration-candidate release.
- Aniflow: #32 through #34 define stream-aware temporal correctness, layered delivery validation, and content-addressed operational reuse.

## Strict next-up queue

### Preserve the active Flow stack

1. Review and merge Flow PR #47; close Flow #44.
2. Retarget Flow PR #48 to main, review and merge it; close Flow #45.
3. Complete Flow #46 and reconcile parent Flow #29.

### Make Optiflow safe and useful on external media

4. Optiflow #88 — deterministic filesystem, path, and removable-volume corpus.
5. Optiflow #89 — read-only external-drive v0.1.1 pilot and operator runbook.
6. Optiflow #90 — immutable approval and non-mutating dry-run preflight.
7. Optiflow #91 — bounded exact-duplicate quarantine apply.
8. Optiflow #92 — status, resume, restore, cleanup, and fault recovery.
9. Optiflow #96 — separately authorized quarantine finalization.
10. Optiflow #93 — disposable-volume qualification and signed v0.2.0 release.
11. Optiflow #94 — bounded source-preserving OxiPNG candidates.
12. Optiflow #95 — validated lossless PNG replacement and v0.3.0.

Optiflow #89 is the first checkpoint suitable for scanning real external media without mutation. Actual duplicate-space reclamation begins only after #90 through #93. Lossless PNG replacement follows through #94 and #95.

### Finish publication providers and suite orchestration

13. Renderflow #415 — canonical ordered collections.
14. Renderflow #416 and #417 — print-interior PDF and fixed-layout EPUB generation; these may proceed independently after #415.
15. Renderflow #418 — fixed-layout EPUB validation and capability truth.
16. Renderflow #419 — immutable Renderflow integration-candidate release.
17. Aniflow #8, #13, #32, #33, #34, bounded #24 closeout, then #10.
18. Flow #30, #49, #31, released-provider adapters #50/#52/#51, then #53.
19. Flow #32, #33, #34, and comic-publication orchestration #10.
20. Flow #54 — first immutable Flow integration-candidate release.

## Remaining audit work

- Re-query CI and release assets before each issue branch; this checkpoint records repository and issue/PR state but does not freeze future GitHub state.
- Reconcile ROADMAP.md execution snapshots as each reviewed PR merges.
- Keep one bounded issue and review PR at a time, except where an issue explicitly documents safe independent provider work.
- Run repository-local post-roadmap audits only after the product/release chains above have landed; use them to close, supersede, or defer remaining backlog honestly.

## Safety boundary for the intended external-drive trial

Do not begin with Optiflow mutation features on irreplaceable media. Use the #89 read-only workflow first, keep state off the scanned volume, preserve source digests, and prove the transaction engine on disposable media before allowing quarantine or finalization against personal directories.
