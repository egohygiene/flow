# Native Mac trial operator handoff

This kit prepares a **local, unqualified source candidate** through existing
release/package and suite-installation owners. Apple Silicon is first. Actual
Mac builds, installation and native workloads have **not been executed** by this
Linux authoring checkpoint. Linux fixture success supplies no Mac qualification.

`tools/macos_native_trial.py` provides a bounded prerequisite matrix and portable
metadata report. It never builds, installs, launches a product, downloads, starts
a service, acquires credentials/models, signs, publishes, edits shell profiles or
touches removable volumes. Its `plan` action invokes no programs. `preflight`
queries exact source Git state, installed toolchain versions and Command Line
Tools presence; `observe` additionally reads selected candidate bytes. Successful
preflight means only the checks named in that report passed. Offline Cargo-cache
completeness, disk headroom and all runtime/profile requirements remain explicit
unobserved gates until the owning operation runs.

The [existing distribution contract](distribution.md) owns copying, bundles,
status, activation, rollback and owned-only uninstall. The
[release-cohort contract](integrations/release-cohorts.md) keeps byte identity,
installation, publication and native qualification separate. This kit adds no
installer, workload orchestrator or new release authority.

## Exact input lock and one Task entry point

Prepare a private UTF-8 `flow.macos-native-trial-request/v1` document. All paths
are explicit canonical absolute paths. Each source is a distinct **clean** Git
checkout, with full lowercase HEAD revision, HEAD tree and raw `Cargo.lock`
SHA-256. Do not reuse these illustrative identity placeholders as actual pins:

```json
{
  "schema_version": "flow.macos-native-trial-request/v1",
  "target": "aarch64-apple-darwin",
  "sources": {
    "flow": {
      "directory": "/absolute/reviewed/flow",
      "revision": "EXACT_REVIEWED_40_CHARACTER_HEAD",
      "tree": "EXACT_REVIEWED_40_CHARACTER_TREE",
      "cargo_lock_sha256": "EXACT_64_CHARACTER_LOCK_SHA256"
    },
    "renderflow": {
      "directory": "/absolute/reviewed/renderflow",
      "revision": "EXACT_REVIEWED_40_CHARACTER_HEAD",
      "tree": "EXACT_REVIEWED_40_CHARACTER_TREE",
      "cargo_lock_sha256": "EXACT_64_CHARACTER_LOCK_SHA256"
    },
    "aniflow": {
      "directory": "/absolute/reviewed/aniflow",
      "revision": "EXACT_REVIEWED_SUCCESSOR_40_CHARACTER_HEAD",
      "tree": "EXACT_REVIEWED_SUCCESSOR_40_CHARACTER_TREE",
      "cargo_lock_sha256": "EXACT_64_CHARACTER_LOCK_SHA256"
    }
  }
}
```

Select reviewed source explicitly after all intended checkpoints are integrated.
The initial sprint snapshot is Flow
`2b18f11e8820cfb117c26651e28837e624cb6ef6`, Renderflow
`fe1054a87648f294c2fc8c58f75b26e87200685c` and Aniflow
`10df7c09ab1c10749788b4fe2bf77400442f8d32`. These identify the starting
inspection, **not future finished candidates**. In particular, the initial
Aniflow source has no reviewed successor candidate. Keep its missing prerequisite
visible; after successor preparation/review, replace its revision/tree/lock pins
together. Source changes require a new input lock and fresh report root.

Run this Task from the checked-out kit, using a fresh output directory with an
existing parent outside all selected sources and input files:

```bash
task macos:trial:preflight REQUEST="/absolute/private/mac-trial.json" OUTPUT="/absolute/private/fresh-preflight"
```

Use `ACTION=plan` with the same Task for a declaration-only report, including on
Linux. Use `ACTION=observe` after adding the exact `install_request` FilePin
described below. Blocked prerequisites exit 2 and retain a redacted matrix with
fixed reason/next-action codes. Missing files, dirty source, wrong HEAD/tree/lock,
missing candidate, unsupported owner target or mismatched installed Rust remain
actionable refusals. A malformed request exits 2 without creating a report root.
No raw native error, raw path, environment, credentials or subprocess output is
included in the portable report.

The selected Mac must match the target and installed Rust **host**. Apple Silicon
uses native `arm64`, not an Intel terminal under Rosetta. Flow and Aniflow use
Rust **1.85.0**; Renderflow uses **1.94.0**. Prepare Python **3.11+**, Git,
Xcode Command Line Tools, those already installed native toolchains and their
frozen Cargo dependency caches separately. Confirm free space on an ordinary
local private build volume before starting: each owner makes two isolated release
builds and preserves failed work; this kit cannot predict peak disk usage.
The Aniflow release declaration selects a `macos-15` runner; record the real
OS/SDK/deployment target in owner evidence rather than infer a supported OS range
from the runner name. FFmpeg/ffprobe and any selected rendering/audio provider
must be separately prepared and pinned by their existing domain owners. No model
is required for the CPU/manual-SRT trial.

Intel `x86_64-apple-darwin` remains a retained objective. All three exact starting
sources declare existing native release adapters for it; Aniflow names the
separate `macos-15-intel` runner and checks the x86-64 Mach-O header. It needs its
own native Intel host, build and workload evidence. The missing reviewed Aniflow
successor still blocks both profiles. A selected source that lacks a declared
target stays blocked until its owner gains a reviewed adapter/matrix change.
Do not cross-compile an Apple artifact, run through emulation, remove a retained
row or substitute a Linux ELF to make the Intel trial appear complete. An
unavailable Rosetta metadata query remains blocked rather than proving Intel
hardware readiness.

## Explicit native builds through existing owners

The following commands are operator steps on the matching Mac after review of
the exact lock. Set each quoted variable to an explicitly selected private path
or locked source identity. Work/output directories must be fresh, separate and
outside source. Run one owner at a time and retain failed logs. Neither this kit
nor the preflight invokes these commands.

Renderflow's existing owner builds two native Rust 1.94 candidates and captures
its compiled contract/SBOM. `RF_VERSION` is the selected source's Cargo authority:

```bash
RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true python3 -B "$RF_SOURCE/scripts/release/release_assets.py" cohort-build --source-dir "$RF_SOURCE" --target "$TARGET" --version "$RF_VERSION" --commit "$RF_REVISION" --work-directory "$RF_FRESH_WORK" --artifact-dir "$RF_FRESH_OUTPUT"
```

Aniflow's existing owner **requires a prepared and reviewed successor** whose
Cargo version, changelog and `.egohygiene/release-candidate.json` agree. Its
`verify` gate requires a clean exact source revision and no unpromoted entries.
`ANI_REVIEWED_VERSION` is that reviewed `v…` version, never an invented release or
new tag. First invoke the owner's read-only gate, then its native build explicitly:

```bash
RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true python3 -B "$ANI_SOURCE/scripts/release.py" verify --version "$ANI_REVIEWED_VERSION" --source-revision "$ANI_REVISION"
RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true python3 -B "$ANI_SOURCE/scripts/release.py" build --version "$ANI_REVIEWED_VERSION" --source-revision "$ANI_REVISION" --target "$TARGET" --work-directory "$ANI_FRESH_WORK" --output "$ANI_FRESH_OUTPUT"
```

This produces local native candidate bytes. It does not create an immutable tag,
successful hosted candidate run, signed release, publisher proof or released
provider. Do not call a direct `cargo build` a replacement for this owner gate.
Retain the exact candidate source and original owner build evidence. Use the
existing bounded [asset extraction helper](distribution.md#explicit-asset-transport)
to copy the exact archive member only after recording its archive/member hashes
and byte count; do not launch an unpinned archive member.

Use the independent compiled provider descriptors retained by both native build
owners, then build the exact **two-provider**
`flow.release-cohort/v1` from actual Aniflow/Renderflow bytes, descriptors,
versions, source revisions and selected Mac target. Keep publication/native
qualification unestablished. Do not carry forward the prior Linux provider rows.
For explicit later CLI reobservation, Renderflow's `release-contract` returns JSON
directly; Aniflow's `--output json release-contract` returns a versioned command
envelope. Retain that successful envelope privately and compare its validated
`result` with the retained descriptor. The contract pin names exact retained
descriptor file bytes, not the command envelope. Flow's package owner and
Aniflow's native build owner already perform their own envelope/result capture;
Renderflow's native owner already retains its direct JSON descriptor.
The Flow owner packages that exact cohort unchanged:

```bash
RUSTUP_AUTO_INSTALL=0 CARGO_NET_OFFLINE=true python3 -B "$FLOW_SOURCE/scripts/release/package.py" prepare --source-dir "$FLOW_SOURCE" --source-revision "$FLOW_REVISION" --target "$TARGET" --cohort "$EXACT_PROVIDER_COHORT" --work-directory "$FLOW_FRESH_WORK" --output "$FLOW_FRESH_PACKAGE"
python3 -B "$FLOW_SOURCE/scripts/release/package.py" verify --package-directory "$FLOW_FRESH_PACKAGE" --manifest-sha256 "$FLOW_MANIFEST_SHA256"
```

Keep each owner's source revision/tree/lock, exact command, toolchain/host/SDK,
two-build observations, manifest/archive, binary and compiled descriptor pins.
Actual owner success and independent artifact inspection remain required.

## Install, bundle and manage through Flow

Prepare an existing `flow.suite-install-request/v1` with exactly Flow, Aniflow and
Renderflow for this selected trial profile. Its pinned installation cohort
retains the complete two packaged provider rows in order and adds one Flow row
bound to the actual packaged binary/descriptor. The packaged provider cohort and
three-tool installation cohort consequently have **different hashes**. Preserve
both exact documents; do not relabel one as the other.

Use the existing [distribution request schema](../schemas/suite-install-request-v1.schema.json)
and contract. Each selected executable and compiled descriptor uses
`{"path":{"encoding":"utf8","value":"/absolute/file"},"sha256":"…","byte_count":123}`.
No optional support assets are selected by this narrow profile. Add its exact
FilePin as `install_request` to a new private trial input lock and run the Task
with `ACTION=observe`. That action checks all three selected source/contract/byte
bindings, a closed row/tool inventory, aggregate logical-byte budget and native
Mach-O CPU/executable headers. It performs **no installation or product launch**.

The prior released Optiflow **0.1.1** read-only source is
`b82599a2231e997d42fd9f26f4b59587f4ae14cf`. Its Linux artifact cannot enter
this Mac profile. Keep released Optiflow as its separate existing read-only
release route until an exact native release artifact and its own proof are
explicitly selected. Newer Optiflow main/Rust 1.89 is not that immutable release.

After native byte observation, use the exact packaged Flow executable explicitly:

```bash
"$FLOW_PACKAGED_BINARY" suite-install --request "$EXACT_INSTALL_REQUEST" --acknowledge-unqualified-candidate --format json
"$INSTALLED_FLOW" suite-status --installation-root "$INSTALL_A" --receipt-sha256 "$RECEIPT_A_SHA256" --format json
"$INSTALLED_FLOW" suite-bundle --request "$EXACT_INSTALL_REQUEST" --output-directory "$FRESH_OFFLINE_BUNDLE" --format json
"$INSTALLED_FLOW" suite-install --bundle "$MOVED_OFFLINE_BUNDLE" --bundle-sha256 "$BUNDLE_RECEIPT_SHA256" --install-directory "$FRESH_INSTALL_B" --acknowledge-unqualified-candidate --format json
"$INSTALLED_FLOW" suite-activate --installation-root "$INSTALL_A" --receipt-sha256 "$RECEIPT_A_SHA256" --selection "$FRESH_SELECTION" --format json
"$INSTALLED_FLOW" suite-activate --installation-root "$INSTALL_B" --receipt-sha256 "$RECEIPT_B_SHA256" --selection "$SELECTION" --previous-selection-sha256 "$SELECTION_A_SHA256" --format json
"$INSTALLED_FLOW" suite-rollback --installation-root "$INSTALL_A" --receipt-sha256 "$RECEIPT_A_SHA256" --selection "$SELECTION" --previous-selection-sha256 "$SELECTION_B_SHA256" --format json
"$INSTALLED_FLOW" suite-uninstall --installation-root "$INACTIVE_INSTALL_B" --receipt-sha256 "$RECEIPT_B_SHA256" --removal-record "$FRESH_REMOVAL_RECORD" --format json
```

Use only fresh private test-owned copies for changed-byte, stale receipt,
unknown-member and missing-acknowledgement refusal controls. Preserve originals,
unknown material, retained installation A and interrupted work. Record exact
structured outcomes/current bytes, not just exit zero. Existing commands own
all mutations and refuse stale or unknown members; this walkthrough grants no
authority for unrelated roots. No OS installation or global selection is made.

## Reports and remaining native proof

The helper writes only `report.json` in a fresh `0700` root, with mode `0600`.
It emits the same allowlisted metadata on stdout: request digest, fixed reasons,
source revision/tree/lock/version, selected artifact/contract hashes and byte
counts, native host target or null, explicit remaining gates and false effect
flags. Its `native_execution` always remains `not_executed_by_this_helper`.
Copy this portable report only after reviewing it. Input locks, owner captures,
installation receipts, logs and actual media remain private because they contain
machine paths or content. Capture owner output separately under a private root
with finite per-command/time/total-byte limits; never dump the environment or
raw owner output into a public report. A full disk/failure preserves work for
explicit reconciliation, not an automatic delete or retry.

Run actual Mac native workloads through the existing Flow native adapter,
publishing project and Aniflow/Renderflow domain owners. Use the separately
selected exact source-candidate proof profile after its Mac support is reviewed;
the current combined installed-candidate observer is **Linux x86-64 only** and
must retain that guard. This kit does not weaken it. The separate CP6/CP7 owners
retain run/status/resume, ordinary MP4/manual-SRT, conversion, exact-final QC and
private package/readback behavior. Mac proof must bind its own exact subjects,
real local host and independently inspected output bytes. A Linux fixture,
caller-supplied JSON, a descriptor or copied files cannot establish those facts.

Use the existing [owner report assembler](owner-release-reports.md) only after
real positive native/domain acceptance evidence exists. An unsigned report stays
unsigned and a candidate stays unqualified until its separate release gates pass.
Human timing/word/appearance/rights/locale/accessibility review remains pending;
no review is synthesized. Signing, release/tag publication, credential/model
acquisition, content upload, CI dispatch and physical/removable-volume work are
outside this checkpoint. Flow #71/#72 and Renderflow #419 retain their remaining
acceptance, and Flow #88's original final-pass ledger remains unchanged and last.
