# Execute the fixed pilot with released provider authority

Use the same `pilot-plan.json` produced by project preparation or the existing
manual-SRT pilot planner. Released execution has a separate
`flow.released-pilot-approval/v1` contract; it never fabricates an
unqualified-candidate acknowledgement. Every selected stage requires the existing
opaque `VerifiedEvidence` for its exact row, installed files, host and runtime
requirements, including actual signed positive native evidence.

The commands are `pilot-approve-released`, `pilot-run-released`,
`pilot-status-released` and `pilot-continue-released`. Each takes the existing
independent trust/evidence options:

```text
--evidence-request /private/current-native-evidence.json --verification-directory /private/fresh-verification --verification-policy /private/native-policy.json --policy-sha256 EXACT_POLICY_SHA --verifier-executable /absolute/trusted/gh --verifier-sha256 EXACT_VERIFIER_SHA --acknowledge-verification-network
```

Approval additionally takes `--plan`, `--approved-by`, `--output` and
`--acknowledge-trusted-unconfined`. Run/continue take `--plan` and `--approval`.
Status takes `--run-directory`. Verification is explicit and may contact trust-root
and release metadata services; native media execution retains its existing
offline/environment boundary. A serialized verification result cannot substitute.

The same native supervisor and domain acceptance owner execute the fixed stages
sequentially. Start markers precede launches. A missing started directory or
partial native run requires explicit reconciliation, never automatic replay.
Use existing `resume-released` with the stage's run directory and exact native-run
directory, then `pilot-continue-released`. Completed stages are reused only after
current native status, accepted bytes, retained release record and opaque admission
match the exact pilot plan. Missing final release bookkeeping can be committed
through native resume without rerunning accepted media.

Each verification audit pins the exact request and captures. Execution authority
binds the selected cohort, independent policy/verifier, host identity, installed
executable/contracts and runtime requirement file pins. Freshly attested positive
proofs may renew readiness for that identical scope after older proofs expire;
their timestamps, witness locations and capture directories do not themselves
change the approved effects. Changed source/configuration/plan, provider, policy,
host or runtime byte identity still invalidates authority. No expired proof is
accepted as a passive cache. Original audits remain retained beside fresh status
verification; no old release record is silently replaced.

`released-handoff.json` binds current accepted native receipts, release records,
provider admissions and the original verification audit. Its
`provider_release_qualified: true` describes the selected provider rows at the
observation, not the entire Flow application or suite matrix. It retains
`creative_review_pending: true` and `upload_authority: false`. Status supplies its
fresh verification summary independently. Words, timing, visual/audio quality,
destination requirements and final publishing approval remain human/domain gates.

The exact-plan native `observe_receipt` API is read-only domain/payload readback
for either authority mode. It does not validate serialized approval or grant
execution/admission. Signed evidence consumers and local packaging use it without
recursively requiring a previous release-verifier object. Run/resume continue to
require their proper candidate or opaque released authority.

Source checkpoint only: no verification, provider/media/model execution, tests,
build/compiler/validator, CI, signing, acquisition or publication occurred. Real
qualified provider releases remain existing owner work; #88 remains last, and
the known alignment cancellation/start-marker failure remains unresolved.
