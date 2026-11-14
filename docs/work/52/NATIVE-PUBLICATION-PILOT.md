# Synthetic native publication candidate pilot

Relates to #52, #53 and #70. This checkpoint runs the existing public Flow CLI
through the existing independently captured Renderflow candidate adapter. It
does not consume a released Renderflow build or qualify an installed Flow suite.

The first actual attempt exposed a source-contract mismatch: Flow required
absolute paths while Renderflow ordered members require paths below the config
parent. The adapter now resolves normal relative locators against that pinned
parent and still compares exact absolute selected inputs. Parent traversal,
changed order and unselected inputs refuse. Domain conversion remains Renderflow's.

## Executed scope

The final bounded driver completed 24 Flow invocations in 53.323 seconds: both
two-page native fixed-layout EPUB and exact-provider print PDF followed
plan/approve/run/status/explicit completed resume/final status. Twelve negative
observations require exact diagnostic codes, reasons and recovery subjects:
missing candidate acknowledgement, existing run, wrong resume selection, changed
configuration, changed publication bytes and reordered source selection, for each
format. Restored state is reaccepted; completed resume changes no retained bytes.

Two 100-pixel synthetic RGB PNGs use spaces/Unicode paths. Originals and selected
provider/contract bytes remain unchanged. EPUB ZIP membership and the actual
native receipt/publication joins are independently checked. PDF readback uses the
already prepared pikepdf to check two pages and their 100 mm media geometry;
img2pdf is explicitly selected at version 0.6.3. No provider installation occurs.

Eight independent Python observer cases pass, including privacy projection,
malformed identities, same-exit unrelated refusals, wrong receipt confinement and
collection order. Three focused Rust adapter regressions pass. Their first run
was blocked by existing undeclared `tempfile` references; a separate test-only
owned-directory repair restores compilation with no dependency/lockfile change.
Eight focused audio-setup/policy/readiness tests also pass after that repair.
These selected checks are not full Rust/CI/native matrix qualification.

## Exact subjects

The Flow development binary is 55,671,224 bytes,
SHA-256 `7bcaa12b72d57f12f3321156f18f12cb10d29803a4848d59c8a6acad489125a0`,
built with locked/offline Rust 1.85 at local revision
`a057cd46057710c935f4fd560ecfec8339438465`, tree
`e21b30f54e56808b3d83c70e323dd6b5b02f9d9b`. Later observer/docs and test-only
repair changes are separate source; no remote tree-equivalence is claimed.

The Renderflow binary is 19,965,256 bytes,
SHA-256 `b0a55e600e7888428b6971b99dc4566eac285cebf7338b99d6ed75415d1a3a56`.
Its captured source revision is `c8f2b6896ec82c12e75ad6b8bcf7e4b319602db2`;
the compiled v2 contract SHA-256 is
`9ff99a410bcf98a82801eeadf3e35fdc40539c36d192ce066365769c98e9aa74`.
The earlier Ubuntu/glibc 2.39 userland build receipt is distinct from this local
CLI proof. The cohort's location field points to that real source commit, never
to an invented release asset. Publication/authentication remain unestablished.

Independent review tightened PDF inspection to require all four finite MediaBox
coordinates `[0, 0, 283.46456692913387, 283.46456692913387]` within 0.001 point
on both pages. Wrong height, shifted origins, nonfinite and boolean coordinates
refuse in observer controls. Only this changed inspection was rerun against the
unchanged retained PDF and provider. Its stdout SHA-256 is
`3e8cb87cd49321de7870eb65f26be7c4de456f3f4efc13236ac425bb177392ab`.
`PDF-INSPECTION-AMENDMENT.json` alongside the pilot roots binds the original full
report SHA-256 `26e74a7f5189a78d1b4b1bde1a7802a4e1967e695149907e5564e26dea8bd0a6`
and driver `554c8ee1b86c79efe7dba1776e5cebfda6323548243fcf49ea4565a0cd54d330`
to amended driver `b0c51e782dce07be909f89b051d6d8b8c6d669751a50549287b939a518905206`.
The original 24-invocation report is preserved and was not rerun by the amendment.
The subsequent `CONTENT-INSPECTION-AMENDMENT.json` also verifies both archived
PNG payload hashes in actual EPUB spine order against the exact originals;
reordered pages and changed image bytes refuse. It binds amended driver
`66c3da37e9d5f544813972014b16b4b2e1f28542f4962b916104bb4cc5410a0c`
and the prior PDF amendment. These targeted content checks preserve the full
pilot's original driver/report identity and do not claim a new full run.

Portable receipts are committed as the [original full pilot](receipts/direct-pilot.json),
[PDF inspection amendment](receipts/pdf-inspection-amendment.json) and
[final content inspection amendment](receipts/content-inspection-amendment.json).
The local final report is also retained under
`.work/flow-next-ten-20261114/native-publication/pilot-08-final/report.json`.
Earlier failed attempts remain in sibling directories; a new invocation requires
a fresh evidence root. The report contains projected public identity/hashes,
not private paths, supplied free-form metadata, raw logs or source media.

## Repeat with explicitly selected local assets

```text
python3 -B tools/test_native_publication.py
python3 -B tools/check_native_publication.py --flow /absolute/captured/flow --provider /absolute/captured/renderflow --descriptor /absolute/captured/renderflow-contract.json --img2pdf /absolute/prepared/python/bin/img2pdf --flow-build-evidence /private/build-observation.json --output /private/fresh-pilot
```

The build observation uses `flow.local-build-observation/v1`: actual local build
revision/tree, binary SHA-256/byte count, profile `dev`, and exact
`["cargo","+1.85.0","build","--locked","--offline","--bin","flow"]` argv.
Unknown/private fields and supplied remote-equivalence claims are dropped.
The driver admits only the exact captured Renderflow candidate above. The
selected Python environment must already supply img2pdf 0.6.3 and pikepdf.

This is synthetic candidate execution under explicit trusted-unconfined approval,
not a sandbox, release admission, signing, creative/rights approval or upload.
No hosted workflow, model, private content, Mac/native-volume or Pages task ran.
Immutable releases, installed-suite/platform evidence and full parent acceptance
remain open. Flow #88 stays unchanged and last.
