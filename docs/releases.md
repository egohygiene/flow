# Flow artifact preparation

Flow's Cargo package version is the single CLI/library authority. `publish=false`
remains in force: the occupied crates.io name `flow` is not presented as ours.
The GitHub CLI artifact and portable tracked source archive are distinct channels;
neither is published by these tasks. A chosen release version/tag still needs
maintainer review and the existing #54 native/install/supply-chain gates.

The compiled library `release::describe()` and thin `flow release-contract
--format json` command expose source contract versions, schema digests, candidate
routes, default executable and explicit unqualified state without probes or I/O.
`cargo run` defaults to the real `flow` command. The native package builds only
`--bin flow`; no hermetic fixture or provisional setup binary is distributed.

```text
task release:plan
task release:prepare SOURCE_REVISION=FULL_REVIEWED_SHA TARGET=NATIVE_TARGET COHORT=PROVIDER_COHORT.json WORK=FRESH_EXTERNAL_WORK OUTPUT=FRESH_EXTERNAL_OUTPUT
task release:verify PACKAGE=PACKAGE_DIRECTORY MANIFEST_SHA256=EXACT_MANIFEST_SHA256
task release:publish PACKAGE=PACKAGE_DIRECTORY MANIFEST_SHA256=EXACT_MANIFEST_SHA256
```

Preparation requires an installed matching native Rust toolchain and prepared
offline dependencies. Linux x86-64 GNU, Apple Silicon and Intel Mac have source
adapters; Windows remains a retained objective. Two isolated native binaries and
their compiled descriptors must agree. No automatic toolchain/dependency download
occurs. The source archive uses only the exact selected Git revision's tracked
library/CLI/schema/test/docs/example files; it includes the original Cargo
authority and no sibling checkout, ignored model or private run workspace.

Each candidate package retains its executable, descriptor, portable source archive
and member inventory, explicit provider cohort, SHA256SUMS, SPDX Cargo dependency
inventory, declared license notices and product license. The inventory includes
dev/optional dependencies and declared licenses; it is not a native composition,
OS/tool/model dependency or license-adjudication claim. Native/profile dependencies
remain separately prepared. Source library consumers use an explicit extracted
path or exact reviewed Git revision with prepared dependencies; no registry claim.

Readback requires the externally selected manifest SHA, exact file membership,
current digest/size checks, coherent binary/library/descriptor/cohort roles and a
matching checksum inventory. It never executes the package. It establishes bytes
only; checksums do not authenticate a publisher or prove native functionality.
`release:publish` prints a draft-release argv and pending gates. It does not sign,
create/move tags, dispatch CI, create/upload a release or qualify the candidate.

Manual source review and authored cases only in this checkpoint. Native build,
package tasks, tests/compiler/dependencies/validators/CLI/SDK/CI and publication
were not run. #54/#71/#72 remain open. Flow #88 stays last with its original ledger
unchanged; known alignment cancellation/start-marker failure remains unresolved,
cause unproven. ADRs remain proposed.
