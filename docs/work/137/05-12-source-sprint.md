# Flow suite CI source sprint — November 13, 2026

The maintainer expanded [Flow #137](https://github.com/egohygiene/flow/issues/137)
to the remaining checkpoints 5–12, delivered as stacked draft PRs for later
reconciliation. Nothing is merged, released, deployed or qualified by this sprint.
The issue's acceptance boxes remain separate from source delivery.

| Checkpoint | Owning draft | Source result and remaining limit |
| --- | --- | --- |
| 5 | [Aniflow #232](https://github.com/egohygiene/aniflow/pull/232) | Bounded source-fixture snapshots preserve dot metadata without weakening production bundle inventory. The repaired 31-case synthetic release suite passes. |
| 6 | [Optiflow #210](https://github.com/egohygiene/optiflow/pull/210) | Cargo-deny container and local invocation explicitly select Rust 1.85. The SQLite advisory remains blocked on the compatibility decision. No dependency-policy scan ran. |
| 7 | [Optiflow #211](https://github.com/egohygiene/optiflow/pull/211) | Seven public example assets are explicitly mounted with bounded staging. Five focused regressions and local composition/link verification pass; no deployment occurred. |
| 8 | [Relay #182](https://github.com/egohygiene/relay/pull/182) → [Optiflow #212](https://github.com/egohygiene/optiflow/pull/212) | Narrow literal-path backport at the previously consumed Relay revision, then immutable consumer pin. Seven guard methods pass; unrelated historical catalog failures remain recorded. |
| 9 | [Optiflow #213](https://github.com/egohygiene/optiflow/pull/213) | Two reviewed source bindings change; recipes and expected payloads stay intact. Fourteen Python helper regressions pass and all 17 source bindings are current. Dual-host corpus execution remains pending. |
| 10 | [Optiflow #214](https://github.com/egohygiene/optiflow/pull/214) | Existing canonical temp-parent fix retained; one aliased-parent regression authored. Rustfmt passes; the Rust regression and native macOS/APFS qualification remain unrun. |
| 11 | [Renderflow #514](https://github.com/egohygiene/renderflow/pull/514) | Explicit locked CLI workspace selection and coherent Snap/installer/AUR source assumptions. Metadata/syntax checks pass; actual build, installer and packaging execution remain unrun. |
| 12 | This Flow draft; [Aniflow #233](https://github.com/egohygiene/aniflow/pull/233), [Optiflow #216](https://github.com/egohygiene/optiflow/pull/216), [Renderflow #515](https://github.com/egohygiene/renderflow/pull/515) | One cheap source gate, explicit heavy/scheduled opt-ins, timeouts/concurrency and bounded honest plans. Release admission requires actual prerequisite-job evidence rather than aggregate success. See [the budget handoff](12-ci-budget.md). |

Within each repository, reconcile the still-open checkpoint 1–4 stack first.
Then use these orders:

- Aniflow: #232 → #233.
- Optiflow: #210 → #211 → #212 → #213 → #214 → #216.
- Renderflow: #514 → #515.
- Flow: checkpoint 12 after #140.

Relay #182 is the owner dependency for Optiflow #212. Its commit parent is the
old immutable consumer pin, avoiding unrelated later Relay features; its PR is
against Relay main for review. Reconcile that backport before adopting its pin.

[Optiflow #215](https://github.com/egohygiene/optiflow/pull/215) is a separate,
optional Rust 1.89 / rusqlite 0.41.1 / libsqlite3-sys 0.39.1 compatibility
proposal branching from #210. It is **not adopted and not a primary-stack
dependency**. It requires an explicit maintainer compatibility decision, then
rebasing subsequent repairs and reviewing the resulting corpus lock binding.
The primary stack retains Rust 1.85 and the unresolved advisory; no ignore or
implicit system-SQLite substitution is introduced.

## Evidence boundaries

The task-private tools and bounded source checks establish only their named
results. Final integrated local composition verified 86 HTML documents and seven example
downloads with the existing missing-Intelligence waiver; live producer/feed
capture and Pages deployment remain unobserved. Checkpoint 12 adds synthetic
workflow-budget and release-admission regressions, not a product test pass.

No Rust product type-check/build/test, Clippy, cargo-deny scan, hosted dispatch
or rerun, native macOS/APFS/physical-volume test, installer/Snap execution,
signing, release or publication occurred. The known Flow bounded-wrapper timeout
and Aniflow alignment cancellation/start-marker failure remain unresolved and
their causes unproven. The existing final-pass issue #88, including its original
45-ticket ledger, remains untouched and last. Aniflow #35 stays closed;
Optiflow #95 stays open. Historical dated handoffs remain history.
