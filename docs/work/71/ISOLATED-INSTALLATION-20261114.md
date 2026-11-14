# Isolated pinned suite installation — checkpoint 5

Relates to Flow #71. Source baseline: `71d034263da17c8755cb2c7202cf68ca5b5df6fd`.
The existing `suite-*` CLI owners were executed against fresh private directories
outside the checkout; no new installer or global activation mechanism is added.
[The observation](ISOLATED-INSTALLATION-OBSERVATION-20261114.json) records the
exact candidate bytes, command-output digests and actual results.

The Flow executable is the separately prepared Rust 1.85 release candidate,
SHA-256 `aa03c1eb6a85316446edbed4d1127deee4e594f8b214d3080d34bac19b998eb4`
(6,539,600 bytes). It was built from the exact baseline in two independent targets;
package preparation/verification is reported by the following #54 checkpoint.
Renderflow is the captured Ubuntu candidate at source `c8f2b6896ec82c12e75ad6b8bcf7e4b319602db2`,
SHA-256 `b0a55e600e7888428b6971b99dc4566eac285cebf7338b99d6ed75415d1a3a56`;
its separately captured contract is `9ff99a410bcf98a82801eeadf3e35fdc40539c36d192ce066365769c98e9aa74`.
Optiflow is the actual extracted v0.1.1 executable at source
`b82599a2231e997d42fd9f26f4b59587f4ae14cf`, SHA-256
`46af9399f3785607f835d9a805a8daa42c93597772c6ee812911615ea4b76280`.
Inputs are copied into fresh singly-linked files before use; no original
candidate binary, provider asset or source checkout is changed.

## Actual results

Seventeen exact Flow command cases passed, plus installed Renderflow and Optiflow
version observations. The proof covers explicit candidate acknowledgement;
three-tool installation; installed status/help/version/compiled descriptor;
moved offline-bundle installation; A-to-B selection; exact-pointer rollback to A;
wrong receipt, stale selection, changed copied bytes and unknown-member refusal;
owned-only removal of inactive B; retained A status and original input hashes.
Refusal cases require the exact domain code and reason, not just a nonzero exit.
The unknown-member refusal preserves the canary and creates no removal record.
The final observer independently rehashes all three retained A binaries and both
contracts, requires exact installed provider version stdout, and confirms the
empty command working directory stays empty. Cohort projection rejects duplicate
provider tools/row IDs before constructing its lookup.
Only the test harness restores its own tamper fixture/removes its own canary.
The product never repairs it automatically or deletes unknown material.

Reproduce with `python3 -B tools/check_suite_installation.py` and explicit
`--flow`, `--flow-sha256`, `--descriptor`, `--renderflow`, `--render-descriptor`,
`--optiflow`, `--provider-cohort`, `--source-revision` and a fresh `--output`.
Inputs must be the selected exact package/provider files; the provider cohort
contains their real source/release references. The driver invokes the existing
product commands with absolute paths, an empty Unicode working directory and
no checkout path in the child's PATH. It retains bounded per-case captures
privately and projects only typed identities/results into the public observation.

## Limits retained

These are same-version candidate installation roots, so the rollback proves
selection/readback mechanics, not cross-version migration compatibility. The
host is Linux x86-64/glibc 2.41. No macOS, Windows, package manager, shell-profile
integration, publisher authentication or native/media/backend matrix is qualified.
The installer itself invokes no provider. Version/help/descriptor smoke is not
an operational media workload or authenticated release. #54/#71 and #88 retain
their remaining supply-chain, supported-host and deep acceptance criteria.

A separate Render-only installation is retained for the independent installed
publication checkpoint. It used that checkpoint's fixed Flow adapter candidate,
not this baseline package, and the exact canonical Render row/contract. Its
receipt explicitly records candidate acknowledgement and false provider-invoked,
native-qualified and publisher-authenticated fields. Do not transfer its later
Render-route results to this baseline executable; candidate identities remain
separate. No signing, release, upload, hosted workflow or physical-volume work
occurred. The known Flow timeout and Aniflow alignment/start-marker failure retain
their unresolved limits.
