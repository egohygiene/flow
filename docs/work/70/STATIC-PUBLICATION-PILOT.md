# Installed static publication candidate proof

Relates to #52 and #70. Two separate single-locale `en-US` projects now execute
through the existing installed-provider/static/native owners: two explicitly
ordered PNGs become native fixed-layout EPUB or exact-provider print PDF.
No release, translation or human creative approval is inferred.

The 22-call native proof below remains the historical observation of its named
binary/source. Independent review then identified a missing aggregate bound for
private collection copies. The corrected candidate and its separate targeted
budget observations follow; the original native report has not been rewritten or
rerun under a different binary identity.

## Aggregate staging correction

`maximum_staged_bytes` has a separate 256 MiB default and an inclusive 64 GiB
maximum. Before reading selected content or creating the workspace, validation
checked-adds every collection-copy reference across all locale branches and
refuses an aggregate above the selected budget. A reused original counts again
for each physical copy. Single-file branches stage zero, so zero can explicitly
forbid collection staging. Native output, metadata and peak/total disk bounds
retain their separate semantics. The existing copier already refuses growth
beyond each source pin before writing the excess chunk.

Default serialization is omitted: an omitted or explicitly default-valued budget
preserves the earlier project identity and Record/Preview shapes. A nondefault
budget participates in project identity and retained readback. Existing records
that exceed the new default refuse; no serialized claim bypasses the guard.

Eight focused static Rust tests pass, including repeated references at 876/875
bytes, zero staging for single-file branches, default identity compatibility,
nondefault identity binding, upper-bound refusal and checked-add overflow. The
changed schema admits an actual explicit 438-byte project and refuses negative
or above-64-GiB budget values with local references resolved in memory.

Seven targeted public CLI calls on the corrected binary pass: preview and prepare
refuse 438 declared bytes under a 437-byte budget without creating a workspace;
two locales refuse 876 declared copies under an 875-byte budget without creating
a workspace; an exact 438-byte budget previews without writes, prepares exactly
438 copied bytes and reads back `not_started`/`partial`. No approval/provider
invocation or native acceptance is created by that preparation. The earlier
tiny default-budget native EPUB record still reads back accepted with creative
review pending, proving retained-record compatibility. Original input bytes stay
unchanged.

The corrected dev binary is 55,952,496 bytes, SHA-256
`19a99b81fc51f6bd47866b2c09f84503cf7b17063f189fa1f1b4552d6871d96b`,
built locked/offline at clean local revision
`830d331802cb00fb4c7d0d6441ed89e6819d5b0b`, tree
`993e5a595de93f4de61649f4f7b32aed8d7c99dc`. Later receipt/docs changes are
separate. The [portable budget receipt](receipts/staging-budget-check.json),
SHA-256 `5c0f77323f74e9ea8669811ef472ed9df71947b23bf87819f1abbddbb0f50f1a`,
binds targeted driver
`993757bf3cad65f8ca5e3d116921d51849b152032b5e3e7593ee44c3bff12658`
and the original static report. Local stdout/private preparation records remain
in `.work/flow-next-ten-20261114/static-publication/staging-budget-01/`.

```text
python3 -B tools/check_static_staging_budget.py --flow /absolute/corrected/flow --flow-build-evidence /private/corrected-build.json --prior-static-pilot /private/executed-static-pilot --output /private/fresh-budget-check
```

The source repair carries optional typed member geometry/media declarations and
explicit pinned native publication metadata. Ordered members are copied using
the existing constant-memory guarded copier into private configuration-relative
locations. Original pins remain in project readback; native plans bind exact
copied bytes and order. Publication artwork is rebound only for explicitly
selected originals, preserving the supplied metadata. Preview creates no copies.

Actual catalog readback also exposed an existing integration mismatch: Renderflow
publication verification reports `size_bytes`; the Flow inventory join incorrectly
looked for `byte_count`. The existing join now consumes its native versioned field
and still checks exact accepted file/digest/size. No new conversion engine,
installer, approval mechanism or sibling dependency is introduced.

## Executed evidence

The final pilot completed **22 public Flow CLI invocations in 77.540 seconds**.
It reobserves the selected installation, previews each project without writes,
prepares exact private member/configuration/native-plan copies without invoking a
provider or creating approvals, explicitly approves/runs the native candidate,
then verifies required-target status and current artifact catalog. Seven exact
refusal observations cover negative geometry, missing collection media types,
unselected artwork and changed staged bytes. Restoring the deliberate staged-copy
control recovers identical status; the 438 bytes of originals remain unchanged.

Both required native targets are observed with exact catalog payload pins.
Project status remains `partial`: `creative_review_pending: true`, no reverified
release admission, locale-content review or upload authority. The output EPUB is
4,951 bytes, SHA-256
`96bef187d48e01bb7ad70a32e55d2c18ede2c8d41d1b843bd10d6f4724e0a5c5`.
Its actual spine image bytes/order match the supplied selected originals. The PDF
is 2,226 bytes, SHA-256
`0ed5830ed858876a4bf2a7dc690181a1540a4995e797817bfbb5df952b522416`.
Independent pikepdf inspection verifies both pages' four finite MediaBox
coordinates, including origin and height, against the selected 100 mm bounds.
These hashes match the earlier direct candidate proof, not a release claim.

Five focused static-project Rust regressions pass: metadata rebinding refuses
unselected/traversing artwork; geometry refuses nonfinite/negative values;
existing locale, required-target and stale/orphan catalog boundaries remain.
Eight independent native observer cases pass. Both actual JSON project inputs
satisfy the changed schema, and negative geometry refuses with repository-local
references resolved in memory. The initial naive file-URI resolver invocation
failed before validation because the existing relative `$id` changes reference
scope; no input was called validated by that failed invocation. Full suite,
platform and hosted CI qualification were not run.

## Exact subjects

The Flow dev binary is 55,904,672 bytes, SHA-256
`3ea6ef6574093820d0b3fb1eef9ceaa65a796ad9e63392e19216756c47de7c6f`.
It was built locked/offline with Rust 1.85 at local source
`9cbbd6ccc84cfeb811ce6d087a0a51f1e3f49da7`, tree
`f45f9da221d69e8d4727a938e6521eabeb69381d`; later driver/docs changes are separate.
No remote tree-equivalence or fresh independent reproduction is claimed.

The installed candidate receipt SHA-256 is
`35f8ab91aa6281d965c30d0aa0f3c6d8b40d74b1c2f2db4f5abeeea218278e2e`.
Its Renderflow binary/contract retain the exact direct pilot's
`b0a55e60…`/`9ff99a41…` identities and independently captured source
`c8f2b6896ec82c12e75ad6b8bcf7e4b319602db2`. Candidate acknowledgement is true;
publisher/native qualification remain false. PDF uses already prepared img2pdf
0.6.3 and pikepdf; no implicit preparation/download occurs.

The [portable full report](receipts/static-pilot.json), SHA-256
`8ffe30fb9c75fdf93925184cbcd89a8530d690e064466e0e52d5ba7ddd3a78dc`,
binds driver
`67b15f01ccb9e7669a46c3e651bea49424366cba763053f034f95badb5b129aa`
and the three imported observer-owner source hashes. Local commands/plans/native
receipts and the earlier run remain under
`.work/flow-next-ten-20261114/static-publication/`; private paths/logs are not in
the portable report.

```text
python3 -B tools/check_static_publication.py --flow /absolute/captured/flow --installation /private/installed-candidate --receipt-sha256 EXACT_INSTALLATION_RECEIPT_SHA --img2pdf /absolute/prepared/python/bin/img2pdf --native-pilot /private/executed-direct-pilot --flow-build-evidence /private/build-observation.json --output /private/fresh-static-pilot
```

Native execution retains explicit trusted-unconfined candidate approval. These
synthetic policies neither establish a sandbox nor exercise private content,
models, native Mac/physical volumes, signatures, release/Pages or external uploads.
Broader composition/installation/release acceptance stays with existing owners;
Flow #88 remains unchanged and last.
