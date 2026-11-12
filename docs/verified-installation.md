# Install publisher-authenticated files from a clean machine

Publisher authentication precedes local native qualification. The new
`release_evidence::publisher` SDK verifies portable owner reports without needing
an installed provider, local native run, model or host identity file. It creates
an opaque `VerifiedPublisher` that permits only exact file copies. It cannot
authorize native execution or deserialize a serialized `verified` flag.

Supply a `flow.publisher-verification-request/v1` with exact `cohort` FilePin,
`timeout_ms`, and `selections`: each selects `row_id`, `report`, `sigstore_bundle`
and `archive` FilePins. The report comes from its independent owner's release
adapter and must already be attested. It binds an actual uploaded archive or raw
native asset, executable member, and archive-member/separately uploaded contract
assets. Required current GitHub asset digests, IDs, tag, source and signer must
match. The publisher verifier uses the same fixed signature/client protocol as
full native evidence ingestion.

Choose host trust outside that request. A `flow.publisher-policy/v1` contains
exact `owners` using the existing signer/workflow/source fields,
`maximum_age_seconds` (1–3600), and an optional explicit private
`github_credential` FilePin. It needs no local host/native evidence. Root CLI
options select policy and verifier paths/digests independently of any provider
report or pipeline. Verification requires a fresh audit directory and explicit
network acknowledgement; it may retrieve trust roots and release metadata.

```text
flow publisher-verify --publisher-request /private/publisher.json --verification-directory /private/fresh-audit --verification-policy /private/publisher-policy.json --policy-sha256 EXACT_POLICY_SHA --verifier-executable /absolute/trusted/gh --verifier-sha256 EXACT_VERIFIER_SHA --acknowledge-verification-network --format json
flow suite-install-released --request /private/install.json --publisher-request /private/publisher.json --verification-directory /private/fresh-install-audit --verification-policy /private/publisher-policy.json --policy-sha256 EXACT_POLICY_SHA --verifier-executable /absolute/trusted/gh --verifier-sha256 EXACT_VERIFIER_SHA --acknowledge-verification-network --format json
```

The second command authenticates again, then uses the existing immutable-copy
installer. Select the existing suite request's exact executable, contracts and
support files. Every selected byte identity must be admitted by that row's
authenticated report. Raw/extracted files may be obtained by the existing
explicit pinned acquisition/extraction helper; this operation does not download
them or launch installed programs. An expired or changed policy/report/archive/
audit refuses. No unqualified-candidate acknowledgement is manufactured.

For an existing pinned offline bundle, replace `--request` with `--bundle`,
`--bundle-sha256` and `--install-directory`, retaining the separate explicit
publisher inputs/trust. Opening/copying a bundle remains offline. Fresh-process
signature/publication verification is an explicit network operation; there is no
permanent trusted JSON cache. An embedding host can verify once and keep the
opaque short-lived authority while performing the offline copy.

The final existing installation receipt retains its request-bound publisher
admission, exact row/report/archive identities and audit pin. It records
`publisher_authenticated: true`, `candidate_acknowledged: false`,
`provider_invoked: false`, `native_qualified: false`. This is an installation-time
observation. Receipt status and activate/rollback/uninstall use the exact receipt
digest and current owned-file membership; they never contact a verifier. Old
candidate receipts retain their original encoding and acknowledgement semantics.
Retain external audit/signature material for later inspection; its presence or
serialized admission cannot grant native runtime authority.

After copying, use preparation/doctor and the separate positive-native evidence
path for actual local backend/profile readiness. Missing targets, dependencies,
models and native proof remain explicit. This source checkpoint ran no verifier,
networking, acquisition, installer, product/native program, test, build, compiler,
validator, CI, signing or publication. Existing qualification issues remain open;
Flow #88 is last and the known alignment failure remains unresolved.
