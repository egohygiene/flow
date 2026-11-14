# Combined installed-candidate acceptance

`tools/check_combined_installed_candidate.py` observes the existing direct-native,
static-project, staging-budget and immutable released-Optiflow routes through an
already installed **packaged Flow executable**. It prepares no build, installation,
download, model or release. Each route keeps its existing owner and validators.

The historical standalone driver modes and dated receipts remain unchanged. This
mode requires an explicit closed `flow.installed-candidate-selection/v1` document
for the new package and installation; it never silently chooses the earlier Render
candidate or a development Flow executable. The selection contains:

- `target`, fixed to `x86_64-unknown-linux-gnu`;
- `installation_root` and `installation_receipt_sha256`;
- `flow_package_directory`, `flow_package_manifest_sha256`,
  `flow_package_build_observation`, `flow_package_build_observation_sha256`,
  `flow_source_revision` and `flow_source_tree`;
- `renderflow_source_revision`, `renderflow_source_tree`,
  `renderflow_binary_sha256`, `renderflow_contract_sha256`,
  `renderflow_candidate_proof`, `renderflow_candidate_proof_sha256` and
  `renderflow_row_id`;
- `optiflow_row_id` and `optiflow_release_bundle`.

Include `schema_version` as `flow.installed-candidate-selection/v1`. Paths are
absolute canonical existing paths. Digest and source fields use complete lowercase
SHA-256 and Git object identities. The Render row is the existing local
`renderflow.canonical-build.source-candidate.x86_64-unknown-linux-gnu` row.
The Optiflow bundle remains the exact public v0.1.1 bundle admitted by the
[released-route guide](optiflow-v0.1.1-read-only.md).

The private selection file contains machine paths and stays outside the repository.
Its exact raw-byte digest is frozen at admission and rechecked with the selected
package, candidate documents and installed bytes before each route. Public reports
project only named identities and evidence digests. Full command captures remain
local because installation receipts and source locators contain machine paths.

The existing package verifier checks the manifest and asset inventory. A separately
pinned local build observation binds the selected Flow revision/tree, clean source,
release profile and two isolated equal build results to that manifest. This is local
evidence, not authenticated source provenance. The public Flow identity's
`normalized_build_command` is a symbolic command shape; the separately pinned
package build observation retains the owner's actual invocation and environment.
Packaging seals two provider rows
before Flow is built. Installation retains those complete rows in order and adds
one Flow self row; the observer checks this exact projection instead of requiring
the legitimately different cohort documents to have equal hashes.

Run with an already prepared provider and a fresh proof directory whose parent
exists, outside all selected inputs and the observer checkout:

```bash
python3 tools/check_combined_installed_candidate.py \
  --selection /absolute/private/installed-candidate-selection.json \
  --img2pdf /absolute/prepared/bin/img2pdf \
  --output /absolute/private/fresh-acceptance-root
```

The observer first invokes installed `suite-status`. Direct and static routes use
two tiny ordered synthetic PNGs, explicit native approval, EPUB and PDF acceptance,
current artifact catalogs, status and exact resume. Independent observers inspect
EPUB spine order and source image bytes and every finite PDF media-box coordinate.
Refusal controls exercise stale/malformed pins, no-clobber and uncertain state;
the aggregate staging controls refuse over-budget requests before root creation
and count repeated collection copies. The released route verifies the original
immutable Optiflow bundle, compares its provider to the installed copy, then invokes
the installed provider. Each route reobserves selected and original bytes. A final
installed `suite-status` must still match the original receipt and copied bytes.

Each child report identifies `installed_packaged_cli` as its execution subject.
The combined report binds the selected source/package/installation and child-report
digests. A failed observer leaves its bounded local captures and produces no passing
combined report. Point-in-time reobservation does not establish that files could
never change and be restored between observations, and trusted execution is not an
operating-system sandbox.

The Linux host, Ubuntu userland candidate and locally prepared release-profile
Flow package are the measured subjects. Human creative approval, translation,
rights, accessibility, publisher authentication, release qualification and publishing
authority remain pending. This does not qualify hosted CI, Mac/Windows, physical
volumes, models, signed releases or a deployed site. Actual dated combined evidence
is added only after these newly prepared installed subjects complete the run.

## Measured combined checkpoint: 2026-11-14

The first fresh combined run passed **73 actual product CLI invocations** on the
Linux x86-64 host with glibc 2.41. The full observer was frozen at
`7512b16a279a45ee583dd622a38f3aa9204d499d` / tree
`8752a5c71a423926b1eeb2bb1b65be820db2598c`; the following amendment adds only
the already generated portable reports and this account. Observer validation
passed 15 installed-subject admission tests, eight native-publication tests and
18 released-route tests, plus Python compilation and whitespace checks.

The product subjects were the newly merged Flow
`69eafaf74af8891bde1c7a769fb57dbca815476c` / tree
`376f175971da85fa7e894efc06a3ac836ce8b519` and Renderflow
`fe1054a87648f294c2fc8c58f75b26e87200685c` / tree
`ff6a718004be916d5ddc45ef7d97ddfa2a00c531`. They are distinct from the earlier
development binaries and source trees in the historical pilot receipts.

| Measured route | CLI invocations | Observed result |
| --- | ---: | --- |
| Direct native publishing | 24 | Ordered two-page EPUB and PDF accepted; explicit approval, status, exact resume and named refusal controls passed |
| Static publishing projects | 22 | Both required artifacts joined to current catalogs; original and staged pins reobserved; creative review remains pending |
| Aggregate staging budget | 7 | 437/438-byte and repeated-reference 875/876-byte refusals occurred before workspace creation; exact 438-byte preparation passed |
| Actual released Optiflow v0.1.1 | 18 | Installed immutable provider completed read-only scan/report/plan; no-clobber, marker recovery and changed-artifact controls passed |
| Installation status before/after | 2 | Exact original receipt and current copied bytes matched |

Both publishing routes independently checked the EPUB spine and original image
bytes in order. Their EPUBs contain 4,951 bytes and SHA-256
`96bef187d48e01bb7ad70a32e55d2c18ede2c8d41d1b843bd10d6f4724e0a5c5`.
Both PDFs contain 2,226 bytes and SHA-256
`0ed5830ed858876a4bf2a7dc690181a1540a4995e797817bfbb5df952b522416`;
the independent page observer checked both pages and all four finite media-box
coordinates against the selected 100 mm square including bleed. Intentional
changed-config, publication, staged-copy and marker controls restored their
selected test state before final reobservation. This is no claim that those bytes
never changed during the controlled workload.

The locally packaged Flow binary contains 6,578,208 bytes and SHA-256
`0de124114f96cab0256f09735b68ca56eecab646d608c051f1651892c20585e4`.
Its manifest is `62ba25f3dd056b48fdfaf75a83b7f0dbdd3f29524b5ddc471b514246b85ccdbb`,
and the separately pinned two-build observation is
`9473da259a95026100b87b4c609832b09f13c7e64913b1bfd21499a7a0ceddf0`.
Installed receipt `29b5b16dfff0a507aa8c0e287dbfbb348b02bfa932198ab7275a844bedf1b2f0`
retained the same copied bytes before and after all workloads. The packaged
two-provider cohort `b9c1295f2b2e194051c16db8dc58513c7bd4e53a0cdd2af76039b5c2127ff711`
joined exactly to the installed three-tool cohort
`965ed55237926224055b65f361e9411ed891bf172d9d4c9f5cb0ab5856d7ef30`.

The fresh Renderflow installed binary contains 19,969,352 bytes and SHA-256
`b8262204141e5317ea38f2e7ea25974e57e8e33794ca9bd83c071914a7683078`;
its compiled contract is
`9ff99a410bcf98a82801eeadf3e35fdc40539c36d192ce066365769c98e9aa74`.
The pinned Ubuntu candidate summary
`d4b9326587b2ef8d4f5b63361ba06290c3476537ca8985c178978e74740ca4f6`
records caller-supplied target-cache reuse and **zero independent rebuilds**.
Its Ubuntu glibc 2.39 userland observation and ELF ceiling remain separate from
this workload's glibc 2.41 host; neither establishes whole-host/native qualification.
The prepared local provider was img2pdf 0.6.3 with pikepdf 10.18.1 and qpdf
12.4.2; transitive dependency authentication remains unestablished.

The exact portable
[combined report](../work/11/evidence/20261114/combined-installed/combined.json)
has SHA-256 `2241fce4dafbae1fbb2436f0b10b985c7da611407f828040819f640da291401e`.
It binds the four exact child reports retained beside it and the admitted selection
digest. The private selection, raw command captures, accepted native outputs,
staged sources and recovery markers remain in the local proof root; they contain
machine paths and are not copied into public evidence. Each child report states
`installed_packaged_cli`. Static catalog results stay `partial` with human creative
review pending; no upload, signing, release, CI or deployment was performed.
