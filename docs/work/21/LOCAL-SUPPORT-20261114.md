# Four-CLI local support checkpoint

November 14, 2026 (America/New_York). Relates to
[Flow #21](https://github.com/egohygiene/flow/issues/21) in the approved
ten-checkpoint continuation under [#137](https://github.com/egohygiene/flow/issues/137).
These are independently built source candidates, not four published releases.

## Actual selected observations

The existing shared fixture now passes all **44 cases**, eleven per product,
through the actual native CLIs. The portable
[report](LOCAL-SUPPORT-20261114.json) records each executable SHA-256 and byte
count, observation time and passed case. All four were selected;
`all_selected_passed` and `complete` are true only for this metadata-only scope.

The cases exercise preview without writes, allowlisted private bundle creation,
unsigned inspection, no-clobber behavior, unknown input, changed digests,
traversal, duplicate members, oversized input, symlink refusal and explicitly
unknown health measurements. Generated bundle files remain mode 0600 on this
Linux host. A synthetic private sentinel in environment, paths and input never
appears in emitted metadata or command output.

Eight stdlib driver regression cases also pass. They independently reject
panics, parser failures, unrelated reasons, valid-looking JSON with crash exit
codes or wrong typed categories, incomplete selections, symlink-loop inputs
and existing report destinations. The driver now preserves completed cases and
fixed failure rows while checking other selected products. Captures remain
private; native output text is not copied into portable failure records.

## Source and build bindings

| Product | Built source and reviewed boundary |
| --- | --- |
| Flow | Clean local `3d4e98e34941250b72ee7e5944ea350dcc87ef57`, tree `24fd09fb203a07a73a2cf60f289ee0adcffbd82e`, exactly matching draft #142 at `65a6c962da4330ca733f2fc9fa19bb3b56ac8ae7`; Rust 1.85 locked/offline debug CLI build passes |
| Aniflow | The separate three-destination CLI repair from draft #238, source `f33d51e678f2da976f6b02cc31db925e3ec83094`, tree `0703b91666de62ebada9768b839c599f998d4591`; Rust 1.85 locked/offline diagnostic build with opt-level 1, debug information and LTO disabled |
| Optiflow | Clean local `d1dcd62b971e2a4d6fbeeb01fc96e6d9b343bf8b`, tree `6bc599d061fb37ef5838d7de935e2c9fd9dc6ddf`, exactly matching draft #218 at `6f0821bdd968d5f907ae575f575f8126b0a7ab1e`; Rust 1.89 locked/offline debug CLI build passes with four existing warnings |
| Renderflow | Captured local package candidate from draft #516, source `729f24a23eff8a990201a385777dd2b7eee2e3f7`, tree `848614021aefcd3e151dc89a394664e341213cb7`; the previously retained Rust 1.94 release-profile executable is selected unchanged |

These source trees identify the actual builds. Different local Git aliases are
not represented as the remote commit at which Cargo ran. Later support-driver
and receipt documentation do not change those selected product binaries. The
Aniflow diagnostic build is separately labeled; its older canonical pilot
executable and accepted media remain untouched.

## Faults found and retained

The first full attempt exposed a real Aniflow Clap collision between global
`--output human|json` and local destination fields. Aniflow now uses
`--output-file` for diagnostics bundles and the two recovered-RIFE binding
files. Global presentation stays unchanged. Both unique-ID-only and legacy-alias
alternatives reproduced Clap failures, so the broken spelling is not kept as
an alias. Actual corrected bundle/inspect invocations pass with global JSON
selection before and after the subcommand; old/missing destination spellings
refuse without creating a file. The shared driver/profile follows that repair.

The original driver accepted any nonzero exit as a negative-case pass and lost
all results on the first failed tool. Exact native command, exit, typed error
classification and domain message are now required. Partial or failed reports
cannot claim all-tool completion, and existing report paths refuse before any
launch. `task diagnostics:conformance` selects the four exact binaries;
`task diagnostics:conformance:test` runs only the observer regressions.

An earlier Flow bundle attempt returned nonzero before the original driver
retained its native error. Eight immediate independent Flow repeats, the
stronger three-tool runs and the final four-tool run passed. Its cause remains
unproven; this checkpoint does not label it fixed or establish an intermittent
failure rate. The new failure-retaining observer makes a future occurrence
visible. The initial failed log is retained privately.

## Limits

This is the metadata-only support profile on Linux x86-64, not the broader
observability/exporter pilots, native-media acceptance, collector/network
isolation proof, native Mac/Windows, telemetry package adoption or release
qualification. Renderflow's native error presentation remains its existing
fixed stderr text; this checkpoint does not invent a new machine-error envelope.
The per-command fifteen-second timeout and post-capture byte checks are not a
process-memory sandbox. Use only the exact trusted build under review.

No source media, model, collector, upload, signing or publication is selected.
Original Flow #88 and its final-pass ledger remain unchanged and last. No
architecture decision is ratified by these conformance observations.
