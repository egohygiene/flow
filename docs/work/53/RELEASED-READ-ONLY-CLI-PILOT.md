# Actual released-provider CLI checkpoint

Owners: [Flow #53](https://github.com/egohygiene/flow/issues/53) and
[#70](https://github.com/egohygiene/flow/issues/70). The public Flow CLI now has
an executed, reproducible Linux synthetic checkpoint through the independently
published Optiflow **v0.1.1** read-only route. This adds a test driver and
independent negative observers; it changes no Rust runtime, provider pin,
approval semantics or release declaration. Both parent issues remain open.

## Source and executable identities

The selected Flow debug executable was built with Rust 1.85.0 using
`cargo +1.85.0 build --locked --offline --bin flow`. The actual build source was
local commit `3d4e98e34941250b72ee7e5944ea350dcc87ef57`, clean tree
`24fd09fb203a07a73a2cf60f289ee0adcffbd82e`. That tree is identical to remote
checkpoint #142, `65a6c962da4330ca733f2fc9fa19bb3b56ac8ae7`; Cargo did not run
at the remote commit ID. The executable contains **55,666,864 bytes**, SHA-256
`739382665293d6191bbd94c6479cb5fd417982710ba5f87a37e40eb050630516`.

The first driver proof was recorded at
`894f69b303bd6d6ab9a74d35d0d368d68bf0473a`. Independent review then tightened
its portable projection, final receipt binding and named refusal observations.
The corrected, actually executed driver has SHA-256
`e967ac193d620c55dc855e67de9023b06165a7cfa464bdc1b517c8ce0b2b4ac4`.
Its final committed bytes match the fresh post-review run; no Rust rebuild or
runtime-source change was required. Earlier local reports remain preserved.

The public [Optiflow release](https://github.com/egohygiene/optiflow/releases/tag/v0.1.1)
was downloaded separately and checked against the existing adapter lock:

| Selected identity | SHA-256 |
| --- | --- |
| Published `binary-v0.1.1.tar.gz`, 13,089,850 bytes | `f576131f2695a218fabfaa5167fd16f17641c267194254a9e624303a8cf23e07` |
| Nested Linux x86-64 archive, 4,588,627 bytes | `5e8a0eea84f5ab55fe75fc8a8abc2e9d36e3512537b754262b1f34a9c40aae31` |
| Released executable, 11,098,688 bytes | `46af9399f3785607f835d9a805a8daa42c93597772c6ee812911615ea4b76280` |

The published asset metadata and separate release evidence identify that same
outer bundle. Its embedded provenance declares source
`b82599a2231e997d42fd9f26f4b59587f4ae14cf` and the selected target archive;
the driver checks those exact declarations. It does not perform new signature
verification or authenticate the build producer. A current Optiflow development
binary is deliberately refused as a substitute for these release bytes.

## Reproduce the selected route

Supply an independently built Flow binary, the already downloaded immutable
release bundle, a small local build-observation JSON, and a fresh canonical
output directory. No download or install runs inside the driver.

```sh
python3 -B -m unittest discover --start-directory tools --pattern test_released_read_only.py
python3 -B tools/check_released_read_only.py \
  --flow /absolute/selected/flow \
  --provider-bundle /absolute/binary-v0.1.1.tar.gz \
  --flow-build-evidence /absolute/flow-build.json \
  --output /absolute/new-read-only-pilot
```

The build-observation document records the actual command/revision/tree and
tree-equivalent remote revision. Its `binary` member must be exactly the
selected executable's `{"sha256": "...", "bytes": 55666864}`. Only a closed
public projection enters the portable result: the three complete lowercase Git
identities, typed bounded binary identity, known development profile/command
and fixed authority description. Unknown/private fields are dropped, including
nested binary annotations. Identity grammar and the exact selected Cargo argv
are checked. Those supplied identities remain local observations rather than
signed or independently reproduced build proof. Recompute the binary identity
when selecting different bytes; never reuse this observation for another build.

Cargo can hard-link its debug executable to a dependency output. The harness
holds and rechecks that bounded build artifact, verifies its supplied digest,
then copies its exact bytes into a fresh private singly-linked executable before
execution. It retains strict nonalias admission for request/evidence documents;
it does not weaken the product's file guards. The release bundle, selected
nested archive and released executable must all match the fixed provider lock
before the released provider executes. It extracts only explicit regular
members, without archive-controlled output paths.

Each CLI call uses literal argv, a cleared bounded environment, separate output
streams capped at 1 MiB each and an owned Unix process group. Per-call deadlines
are at most 30 seconds within a cooperative 150-second operation budget. Timeout
and output-flood controls kill/reap the selected process group. This is not an
OS filesystem/network sandbox or proof against an escaped descendant. Initial
binary/archive observation and filesystem I/O have byte bounds, not hard kernel
time or global memory guarantees. The selected driver requires Linux x86-64.

## Observed results

The post-review driver run completed **16 cases / 18 public CLI invocations** in
**5.765448 seconds**. It generated three distinct source files (64 bytes total)
with spaces and Unicode paths. Two contain the same 21 bytes. The actual native
scan/report/review-plan receipt reports **21 duplicate logical bytes**, no
physical-savings measurement, no source-mutation authority and no native apply,
quarantine, restore or finalization support from this pinned release.

Observed cases include:

- Doctor matches the actual release bytes while keeping the suite **partial**
  with exit 9. Missing or unqualified routes do not become a green release.
- Exact local inspection retains unclassified-octet and nonfrozen-input meaning.
  Whole-directory planning stays a **partial scope plan**, not input freezing
  or native quota enforcement.
- Missing process acknowledgement and a mismatched plan approval are refused
  before attempt creation. The accepted synthetic operator declaration permits
  only the selected read-only scan and local evidence; it is not human creative
  approval or authenticated identity.
- Actual `run` invokes the existing released scan → report → review-plan path.
  The observer independently rehashes six native artifact files instead of
  trusting the CLI's completion label alone.
- Status and completed resume retain byte/identity/mtime-equal evidence. Removing
  only the outer completion marker gives recovery-required status; explicit
  resume reconstructs the identical marker without changing native files.
- Existing run output, altered completion metadata, changed native-plan bytes,
  overlapping source/state, an incompatible provider and an unsupported single
  file all refuse as declared. A file request never broadens into recursive
  directory authority. Each named refusal requires the Flow diagnostic domain,
  exact code, fixed reason and selected recovery subject; the same exit with an
  unrelated diagnostic does not pass. Typed partial cases retain their exact
  result domain/reason rather than being accepted as arbitrary failures.
- The final receipt locator must remain inside the selected run's native
  receipt namespace. The actual regular file must decode identically to the
  already validated native receipt before its hash is recorded; a locator to
  unrelated bytes cannot become a portable receipt identity.
- All original source identities/bytes and both selected executable digests
  remain unchanged. Final exact native evidence validates after the deliberate
  test-owned tamper mutations are restored.

The selected native receipt SHA-256 is
`b25dfc0a543578a0bf75686b2a099424bc68523789e81f2abeb7ef9f72119f2f`;
its accepted native plan SHA-256 is
`86fe6c6d7453db0b25010519209367c9d5ccbf5036e15d9597fae72b2f9cf627`.
Receipt IDs, paths and hashes differ on a new synthetic attempt. The report
retains exact selected binary identities, per-command stream hashes/statuses,
native file pins, captured UTC observation time and explicit qualification
limits. Raw local command logs remain private developer artifacts.

Eighteen independent observer test methods pass. Negative controls reject forged
completion/savings/approval, wrong provider identity, missing native work,
changed native bytes/roles or escaped locators, duplicate/nonfinite/multiple
JSON documents, dirty machine streams, aliases, a bogus release bundle,
subprocess output flood and timeout. Additional independent controls prove
private build annotations are omitted, invalid identity types/grammar refuse,
wrong/escaped or type-confused receipt files refuse, and same-exit unrelated
diagnostics cannot satisfy the named refusal cases. The actual post-review
pilot was supplied synthetic private annotations and confirmed their canary
was absent from its portable report. Compiler-artifact hardlinks are exercised
without relaxing the stricter owner-document policy.

Earlier development attempts remain in the local task evidence: the first
preflight correctly refused Cargo's hardlinked build output, and an early
observer expected an `invalid_input` outcome string where the native envelope
correctly uses outcome `refused` with diagnostic code `invalid_input`. The final
driver fixes those observer/admission assumptions; no product assertion was
disabled and no Rust source changed.

## Remaining boundaries

This is actual Linux read-only provider execution through a source-built Flow
CLI. It is not a published/installed Flow release, a clean-host installer test,
native macOS/Windows qualification, a mutation/recovery-volume trial or an
Aniflow/Renderflow released adapter. Captured source/producer declarations are
not signature verification. No private media/model, posting, signing, hosted CI,
release publication or creative approval is used. The known broader Flow
wrapper-timeout and Aniflow alignment/descendant-cancellation limits remain
unresolved; this small passing route does not generalize beyond its scope.
Flow #53/#70 retain their full acceptance; #88 stays unchanged and last.
