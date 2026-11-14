# Combined-source Linux package and suite candidate

Relates to #52, #53, #54, #70 and #71. The existing package and suite installation
owners now produce a local candidate from the actual merged Flow source, fresh
merged-source Renderflow candidate and the actual immutable Optiflow 0.1.1 bytes.
The earlier `71d034…`/`aa03c1…` package remains a historical baseline, not this
candidate. No owner/product source was changed for these package observations.

## Exact selected subjects

| Subject | Identity |
| --- | --- |
| Flow source | `69eafaf74af8891bde1c7a769fb57dbca815476c`, tree `376f175971da85fa7e894efc06a3ac836ce8b519` |
| Flow release executable | 6,578,208 bytes; SHA-256 `0de124114f96cab0256f09735b68ca56eecab646d608c051f1651892c20585e4` |
| Flow compiled contract | SHA-256 `592e52e39c9d1e77e9efd3b819b0fa8e48f9f8aa03178443f63383535d08ebbb` |
| Package manifest | SHA-256 `62ba25f3dd056b48fdfaf75a83b7f0dbdd3f29524b5ddc471b514246b85ccdbb` |
| Fresh Renderflow | Source `fe1054a87648f294c2fc8c58f75b26e87200685c`; binary SHA-256 `b8262204141e5317ea38f2e7ea25974e57e8e33794ca9bd83c071914a7683078` |
| Renderflow compiled v2 contract | SHA-256 `9ff99a410bcf98a82801eeadf3e35fdc40539c36d192ce066365769c98e9aa74` |
| Actual released Optiflow 0.1.1 | Source `b82599a2231e997d42fd9f26f4b59587f4ae14cf`; binary SHA-256 `46af9399f3785607f835d9a805a8daa42c93597772c6ee812911615ea4b76280` |

## Package and source observations

The unchanged `scripts/release/package.py prepare` owner ran with exact source,
target, sealed provider cohort, fresh external work and package roots. Prepared
Rust 1.85.0, frozen/offline release builds, two jobs, nonincremental targets and
the existing source/target remapping were selected. Two distinct compiler target
artifacts have the same executable bytes; separately captured executions expose
the same compiled descriptor. Source HEAD/tree and clean status were observed
before and after, and Cargo.lock SHA-256 remains
`b7a16689cb7ec533bb4a50bf14e1035a2bf5d123e0491f5f63d6cb675891c1ac`.

The [build observation](combined-source-20261114/build-observation.json), SHA-256
`9473da259a95026100b87b4c609832b09f13c7e64913b1bfd21499a7a0ceddf0`,
binds source/tree, selected release command/toolchain, owner/observer source
hashes, the manifest, all asset hashes and the two matching observations.
It is a supplied dated local observation, not authenticated build/source evidence.

The [existing independent package observer](combined-source-20261114/package-observation.json),
SHA-256 `b80da904b8573b4444577d554ea851d939922269e1b9cbca32b507e88180c4cb`,
matches all **376** tracked source archive members against the exact Git source,
the SPDX inventory against **40** unique lock packages, and declared notices
against that inventory. The source archive is 982,536 bytes, SHA-256
`8d94e5fe7e350a62d14371fb6ff2884293739ddcb7546299c9adbf41dbd725c6`.
The [manifest](combined-source-20261114/flow-release-manifest-v1.json) retains
eight role-bound assets, the separate checksum inventory, unavailable registry
channel and explicit unqualified/unpublished state.

Four fresh generated-copy controls require the exact existing verifier exit and
final refusal line: wrong manifest pin, changed executable, unknown member and
recomputed-pin qualification promotion. The original package passes before and
after, and packaged/installed version/help/contract commands match. Tampered
binary copies are never executed. SBOM timestamps remain variable: this is
same-host executable/descriptor reproducibility, not whole-package or cross-host
bit reproducibility, complete OS/tool/model inventory or license adjudication.

## Provider cohort and retained installation

The package consumes a sealed **two-provider**
[cohort](combined-source-20261114/provider-cohort.json), SHA-256
`b9c1295f2b2e194051c16db8dc58513c7bd4e53a0cdd2af76039b5c2127ff711`.
Renderflow has one unambiguous `flow_route` / `canonical-build-v1` row. Its
source-commit location is real; no release-download URL is fabricated. The
pending-at-packaging-start wording records when those pins were sealed. The
separate fresh Ubuntu owner smoke subsequently passed against the same binary
and contract, including native two-page EPUB/PDF and current-source refusals.

Optiflow is extracted afresh from the original public release bundle after exact
outer, nested target archive, embedded source declarations and executable hashes
are checked. The [release-byte observation](combined-source-20261114/released-optiflow-evidence.json)
retains the real `b82599a…` release source, never current Optiflow development
source. It performs no signature verification.

The unchanged installation driver derives exactly one Flow self-row after the
package binary hash exists. Thus the [installed cohort](combined-source-20261114/installed-cohort.json)
has three rows and SHA-256
`965ed55237926224055b65f361e9411ed891bf172d9d4c9f5cb0ab5856d7ef30`.
Its two provider rows equal the package rows exactly; the added Flow row binds
the actual package executable, descriptor and source. Raw cohort byte equality
would incorrectly reject this existing projection; no package mutation or third
build is needed.

The [17-case installation observation](combined-source-20261114/installation-observation.json)
exercises existing public `suite-*` owners from an empty outside-checkout cwd:
explicit candidate acknowledgement, exact copies, status/receipt refusal,
installed contract/help/version and provider versions, a moved offline bundle,
same-version activate/rollback, stale pointer refusal, changed-byte refusal,
unknown-member preservation and owned-only removal of disposable installation B.
Installation A is retained with receipt SHA-256
`29b5b16dfff0a507aa8c0e287dbfbb348b02bfa932198ab7275a844bedf1b2f0`.
Original candidate files, A and the offline directory are preserved. No global
shell profile, system directory or package manager is modified.

Local retained artifacts under `.work/flow-combined-candidate-20261114/flow-package/`:

- `package-01/`: executable, compiled contract, source archive, inventories and exact manifest.
- `installation-01/installed suite A 作品/`: working installed binaries, contracts, cohort and receipt.
- `installation-01/moved offline bundle 作品/`: ready offline suite directory; exact `bundle.json` SHA-256 `0d8ee0535582e11ed36d90102ad40a90063fd3c4be565fe99a78e13b2e06629d`.

Transfer the complete offline directory together; its receipt alone is not the
payload. Existing `suite-install --bundle ... --bundle-sha256 ...` verifies its
bounded copied namespace and requires a fresh private destination and explicit
candidate acknowledgement. It does not acquire or launch runtime dependencies.

## Actual runtime and compatibility scope

The separate [combined installed-runtime guide](../../integrations/combined-installed-candidate.md)
records **73** actual CLI calls using installed A directly: direct native
publication, static projects, staging-budget guards and actual released
Optiflow read-only work, plus suite status before and after. Its combined report
SHA-256 is `2241fce4dafbae1fbb2436f0b10b985c7da611407f828040819f640da291401e`.
These runtime observations are separate from the 17 management cases and the
package builds; installing bytes alone is not a media-workflow pass.

Read-only [ELF inspection](combined-source-20261114/flow-elf-observation.json)
finds maximum required Flow GLIBC symbol version **2.39**. The measured product
host is Linux x86-64/glibc **2.41**. Fresh Renderflow was built/smoked in Ubuntu
24.04 userland/glibc 2.39; Flow's symbol observation does not substitute for a
separate supported-host run. macOS, Windows and the full compatibility matrix
remain unqualified. External PDF tools remain explicitly prepared; no runtime,
model, account or provider is installed implicitly.

No release/tag/signature, registry publication, upload, model/private media,
native Mac/physical-volume or hosted CI operation was performed. Candidate
acknowledgement remains distinct from authenticated installation, native-matrix
qualification, human creative/rights/locale review and publishing authority.
#54/#71/#72 and other parent acceptance remain open; #88 is unchanged and last.
