# Local Flow package candidate — checkpoint 6

Relates to Flow #54/#71. The existing `release:prepare`/`release:verify` owner,
`scripts/release/package.py`, was actually executed against clean source
`71d034263da17c8755cb2c7202cf68ca5b5df6fd`, using prepared offline dependencies,
Rust 1.85.0, two jobs and two fresh independent target directories. The owner
and product code are unchanged. [The observation](PACKAGE-CANDIDATE-OBSERVATION-20261114.json)
binds the actual candidate assets, independent outputs and subsequent checks.

Both native release compiler outputs are 6,539,600 bytes with SHA-256
`aa03c1eb6a85316446edbed4d1127deee4e594f8b214d3080d34bac19b998eb4`.
Their inodes/target roots are distinct, their bytes match, and independently
executed compiled release descriptors match the packaged descriptor SHA-256
`bc3b58373f19ff8b9bd849e39cc80585676c85177c74cd9dba3b596f26ce5db0`.
Cargo's internal executable hardlinks are observed as compiler outputs. The
existing preparation owner executes its own in-build descriptor observation;
subsequent proof invocations use fresh singly-linked captures, package files or
installed copies. No production file-admission defense is weakened.
The two target directories use 276 MiB combined; the candidate package is 7.2 MiB.

## Actual owner invocation and checks

The selected native target is `x86_64-unknown-linux-gnu`. The exact commands were
`python3 -B scripts/release/package.py prepare` with explicit `--source-revision`,
`--target`, `--cohort`, fresh external `--work-directory` and separate `--output`,
then `verify --package-directory ... --manifest-sha256 ...`. The existing Task
aliases expose the same owner. Environment selected the task-private installed
Rustup/Cargo directories and `CARGO_BUILD_JOBS=2`; the owner used frozen release
builds, isolated nonincremental targets and its existing source/target remapping.
No global toolchain/profile or Cargo.lock was changed. The host is Linux x86-64,
glibc 2.41; this is a measured local candidate, not a supported-host matrix pass.

The package manifest SHA-256 is
`b3d2b328a16f4905b2a4852fce1afbbff7bdf50b2766c3507e1e8e984d2ad941`.
Eight role-bound assets plus manifest and checksum inventory are present:
executable, compiled descriptor, tracked library/source archive, source-member
inventory, explicit provider cohort, SPDX dependency inventory, declared notices
and product license. All 355 archive members were independently compared with
the exact selected Git revision without extraction. The SPDX inventory matches
40 owning Cargo.lock packages and all recorded declared-license notices.
This inventory includes dev/optional packages and is not OS/tool/model composition
or license adjudication. Provider cohorts retain the real captured Renderflow
candidate and actual released Optiflow v0.1.1 identities; no Aniflow, model or
fictional release asset is added.

The existing owner verifier accepts the original exact-pin package before and
after four generated-copy refusal controls: wrong manifest pin, changed executable,
unknown member and a recomputed-pin manifest attempting to promote qualification.
Each control requires its specific verifier reason; tampered binaries are never
executed. The originals and failed-case captures remain retained separately.
`tools/check_package_candidate.py` repeats these observations against existing
prepared outputs; it does not implement another packager or silently rebuild.

The installation checkpoint's retained A executable has the exact packaged
binary hash. Its version, help and release-contract commands passed again with
an empty outside-checkout cwd and no checkout in the child PATH. This proves
that the packaged bytes can be copied/observed/executed in that selected local
setup; it does not create native provider, authenticated-install or media approval.
See [the installation proof](../71/ISOLATED-INSTALLATION-20261114.md) for moved
bundles, activation, rollback and owned-only uninstall controls.

## Observer precision amendment

The original measurement above and its JSON observation are retained unchanged.
A later [observer amendment](PACKAGE-OBSERVER-AMENDMENT-20261114.json) records
the original driver commit/hash, updated driver hash and exact capture hashes.
Refusal admission now requires Python exit **1**, empty stdout and the exact
newline-terminated final stderr line `ValueError: EXPECTED`; finding the text
somewhere in a crash traceback is insufficient. Seven in-memory observer cases
pass, including panic exit 101, unrelated error tails, missing final newline,
unexpected stdout and duplicate inventory controls.

All four retained actual refusal captures pass the stronger observer without
rerunning the builds or the whole proof. The actual package has exactly eight
unique asset names before dictionary projection, and its SPDX package list has
exactly 40 unique name/version pairs matching the owning lock, so duplicate
rows cannot disappear into a dictionary or set. No original receipt, measured
candidate byte or source-revision binding is rewritten by this amendment.

## Boundaries retained

The source revision above is the baseline candidate, not the later independent
Renderflow ordered-locator adapter repair. Do not transfer that repair's native
publication or installed-project results to these older executable bytes.
Cross-checkpoints may retain different exact candidate identities; a future
single release must build and qualify its actual selected final source.
Only same-host isolated executable/descriptor reproducibility was established.
SBOM creation timestamps are deliberately variable; the entire package is not
claimed bit-identical across future preparation runs, machines or compilers.

No signing, registry publication, immutable release/tag, upload, package-manager
install, macOS/Windows run, supported-target/native matrix, model/private media,
physical-volume operation or hosted CI was performed. Cargo's `publish=false`
and unavailable registry channel remain unchanged. #54/#71/#72 and #88 retain
remaining installation, supply-chain, native and deep acceptance criteria. The
known Flow timeout and Aniflow alignment/start-marker failure remain unresolved.
