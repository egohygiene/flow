# Pinned suite installation

The public `distribution` SDK and `suite-*` product commands copy explicitly
selected independent binaries and contract/support files into a fresh private
installation. They do not launch providers, fetch dependencies, infer publisher
identity or qualify native workflows. Candidate acknowledgement is explicit.

A `flow.suite-install-request/v1` selects a pinned cohort file, target, fresh
`install_directory`, 1–4 `tools`, timeout and logical-byte budget. Each tool names
its cohort `row_id`, exact executable `FilePin`, contract identifier/file pins and
optional named support assets. `FilePin` uses encoded absolute native `path`,
`sha256` and `byte_count`. Tool/target/executable/contracts must match the selected
cohort. The copies retain source assets and use fixed private bin/contract/asset
names. A successful receipt means exact copied bytes, not an installed working
media/model/backend environment.

```text
flow suite-install --request /private/install.json --acknowledge-unqualified-candidate --format json
flow suite-status --installation-root /private/suite-version --receipt-sha256 EXACT_INSTALLATION_RECEIPT_SHA --format json
flow suite-bundle --request /private/install.json --output-directory /private/fresh-offline-bundle --format json
flow suite-install --bundle /private/offline-bundle --bundle-sha256 EXACT_BUNDLE_RECEIPT_SHA --install-directory /private/new-version --acknowledge-unqualified-candidate --format json
```

Offline bundles retain exact ordinal copies and pinned request/receipt bytes.
Opening a moved bundle rebases only that bounded namespace, preserving every
source digest and explicit ordering; it never guesses external locations. The
new install intent uses the newly selected private destination. Missing/unknown
bundle members, duplicates, changed bytes and overlaps refuse.

Activation writes a separate version-selection record. An expected previous
selection digest and a private lock protect replacement. Rollback selects an
earlier retained, currently reobserved installation. No shell profile or global
system directory is edited; use the selected installation's `bin` and explicit
provider paths. Native plan/run authority remains a different operation.

```text
flow suite-activate --installation-root /private/version-a --receipt-sha256 RECEIPT_A --selection /private/current-suite.json --format json
flow suite-rollback --installation-root /private/version-a --receipt-sha256 RECEIPT_A --selection /private/current-suite.json --previous-selection-sha256 CURRENT_SELECTION_SHA --format json
flow suite-uninstall --installation-root /private/inactive-version --receipt-sha256 EXACT_RECEIPT_SHA --removal-record /private/fresh-removal-record.json --format json
```

Uninstall requires the exact receipt, unchanged owned files and complete membership,
including directories. It creates an external removal-intent record, then removes
only known copies and empty owned directories; it never removes original assets.
Unknown or changed material refuses. Interruption can leave a partial installation;
preserve the external record and remaining files for explicit reconciliation.
Local receipts/locks are trusted private application state, not authenticated
identity or a sandbox against a compromised/concurrently hostile host.

## Explicit asset transport

The separate standard-library transport helper can acquire an exact public GitHub
release asset with an expected digest and explicit `--allow-network`, or copy one
exact member from a digest-pinned gzip tar. It never installs/executes an asset.
Extraction rejects links, special nodes, duplicate/traversal members and bounded
compressed/expanded/member overflows, and writes a fresh non-executable file.

```text
python scripts/distribution/source_assets.py acquire --url EXACT_HTTPS_RELEASE_ASSET --sha256 EXPECTED_SHA --output FRESH_FILE --allow-network
python scripts/distribution/source_assets.py extract --archive PINNED_ARCHIVE --archive-sha256 ARCHIVE_SHA --member EXACT_RELATIVE_MEMBER --sha256 BINARY_SHA --byte-count EXACT_BYTES --output FRESH_BINARY_FILE
```

No models or optional external tools are implicitly installed. Existing-file local
preparation remains with its owning SDK. Tests/builds/validators/native/CLI/SDK/
installer/CI, acquisition and publication were not executed during authoring.
#71 stays open; #88 remains last and unchanged. Known alignment cancellation/start
marker failure remains unresolved, cause unproven.
