# Authenticated release evidence and native admission

The default `release-matrix` and `doctor` commands remain read-only local
observation. Candidate execution still requires its explicit candidate approval.
The new release path consumes actual published release/native evidence; it cannot
turn a source package, copied binary, receipt JSON or unsigned `passed` flag green.

## Independent host trust

An evidence request selects a cohort and up to 32 exact row reports, Sigstore
bundles, installed files and contract files. It cannot select a policy or verifier.
The embedding application supplies an opaque `TrustedConfiguration` from its host
configuration. Root CLI options select policy and verifier paths and SHA-256 pins
independently of the request:

```text
--verification-policy /private/trust.json --policy-sha256 EXACT_POLICY_SHA --verifier-executable /absolute/trusted/gh --verifier-sha256 EXACT_VERIFIER_SHA
```

Pin a reviewed GitHub CLI implementation with attestation verification support.
An executable hash alone does not certify that implementation: choosing the host
trust configuration is the operator/embedding application's authority. Never derive
these options from a provider result or untrusted pipeline. The policy identifies
exact owner repository, workflow, signer commit, source commit/ref and current
host identity, with native evidence age limited to at most 24 hours. Workflows
must belong to the corresponding Egohygiene owner. The decision remains proposed.

`release-evidence` or authenticated `release-matrix` explicitly requires
`--acknowledge-verification-network` and a fresh private
`--verification-directory`. The pinned client verifies the exact report with
`gh attestation verify`, exact signer/source flags, retained Sigstore bundle and
`--deny-self-hosted-runners`. It also reads the actual GitHub release metadata.
The release must be non-draft with its exact version tag, asset ID, uploaded state,
download URL, archive size and GitHub SHA-256 content digest. Missing digest or
metadata refuses; an owner signature alone is insufficient publication evidence.
No artifact is downloaded or installed by this command.

The verifier receives an empty private HOME and no inherited PATH, configuration,
tokens or secrets. For GitHub clients requiring authentication, policy can select
one exact private credential file (at most 4 KiB, Unix mode excluding group/other
access); only that token is passed as `GH_TOKEN`. No credential value is written
to the audit. Failed verification preserves its bounded private audit directory.
Networking is scoped to this explicit verification operation; it grants no media
provider networking. Native processes retain their existing cleared environment.

## Positive records

The closed schemas are
[request](../schemas/release-evidence-request-v1.schema.json),
[policy](../schemas/release-evidence-policy-v1.schema.json), and
[owner report](../schemas/authenticated-release-report-v1.schema.json).
Actual release/native gate owners must emit and attest these exact records after
the corresponding work. This checkpoint does not create an attestation workflow,
publish an asset, invent a gate record, or imply existing releases already emit
these new records.

The report binds one exact cohort/row subject, published archive and executable
member, contract pins, every named runtime requirement, positive clean-install
proof, native qualification proof and native acceptance witness. Current files,
target, explicit host identity and freshness are rechecked. An installation copy
receipt cannot substitute for `flow.positive-clean-install/v1`: an authenticated
positive native plan and acceptance must also bind the smoke result. Missing or
failed qualification checks and synthetic/unit/refusal evidence are refused.

For implemented Aniflow/Renderflow Flow routes, the witness is the exact retained
candidate native `accepted.json`; the existing native status and domain validators
reobserve the plan, acceptance and all output bytes. Keep that private run in its
original approved location. For standalone CLI/library interfaces, the independent
owner supplies an authenticated `flow.owner-native-release-gate/v1` record covering
native build, positive contract probe, clean install, positive smoke and domain
acceptance. This adapter consumes owner evidence without importing sibling crates.

Runtime files and host identity require exact pins. This is current installed-byte
readback plus recent authenticated native qualification, not a continuous GPU or
service health monitor. The actual provider retains live domain admission and can
refuse a later run. Other hosts/targets require their own positive records and
current host selection; absent Windows/native coverage stays visible. Matrix
projection retains every declared coverage row, including those without reports.

## Commands and execution

Supply the independent host trust options above to each effectful command along
with its exact evidence request, fresh verification directory and network
acknowledgement. Ordinary local matrix observation needs none of them.

```text
flow release-evidence --evidence-request /private/evidence.json --verification-directory /private/fresh-verification --verification-policy /private/trust.json --policy-sha256 EXACT_POLICY_SHA --verifier-executable /absolute/trusted/gh --verifier-sha256 EXACT_VERIFIER_SHA --acknowledge-verification-network --format json
flow release-matrix --cohort /private/cohort.json --evidence-request /private/evidence.json --verification-directory /private/fresh-matrix-verification --verification-policy /private/trust.json --policy-sha256 EXACT_POLICY_SHA --verifier-executable /absolute/trusted/gh --verifier-sha256 EXACT_VERIFIER_SHA --acknowledge-verification-network --format json
```

`approve-released` takes `--plan`, `--approved-by`, `--output` and
`--acknowledge-trusted-unconfined` plus those verification options.
`run-released` takes `--plan` and `--approval` plus verification options.
`status-released` takes `--run-directory`; `resume-released` additionally requires
the explicit `--native-run-directory`. Every invocation verifies again into a
fresh audit directory. Approval binds stable request, trust, report and exact
row identity; it cannot consume a serialized verification summary as authority.

Released and candidate approvals have different schemas. The released path does
not manufacture an unqualified-candidate acknowledgement. It reuses the same
native process supervision, stage recovery and domain validators, rechecks opaque
release admission at provider boundaries, and wraps the inner native receipt with
an authenticated admission record retained as `released-acceptance.json`. Missing
final bookkeeping requires explicit released resume and never replays accepted
media. The inner receipt's own native-only scope
remains unchanged. Status/resume use the corresponding authority path. Neither
path grants creative review, upload, signing, timestamp or disposal authority.

Source review only: no verification client/network, tests, compiler, builds,
validators, product CLI/SDK, native/provider/model/media/installer execution,
hosted CI or publication was run. Real signed reports and positive native gates
remain work for existing release/qualification issues. Flow #88 remains last;
the known alignment cancellation/start-marker failure is unresolved, cause unproven.
