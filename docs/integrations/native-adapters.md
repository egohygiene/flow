# Pinned native adapters

## Renderflow canonical file and collection profile

`canonical-build-v1` consumes `renderflow.release-contract/v2` from the independent
Renderflow contract/readback checkpoint. Its configuration is bounded JSON with
`schema: renderflow/v2`, exactly one artifact or one ordered collection of declared
artifact IDs, and paths matching `Request.inputs` in that order. No unused sources
or remote URIs are admitted. The configuration must explicitly set
`execution.requirements.offline` and `local_only` to true. Network and AI remain
denied; custom transform YAML and scoped composition/presentation/briefing packages
require their own future routes. These retained inventory rows remain visible.

Set `output.bundle_root` to the exact absolute `RUN/native/output` path and keep
collision policy `error`. This separates every native publication from selected
source files and preserves existing output for recovery. Flow reuses native
`build --dry-run`, `build`, graph `build --resume`, and `publication-status`.
Native planning may perform the provider's bounded local tool probes only after
Flow's explicit run/resume approval; Flow preparation/doctor do not launch it.

Acceptance binds native source bytes/order, the admitted forest identity, engine
version, complete native receipt and exact output root. The existing native
publication verifier must report all selected targets with accepted validation.
Flow hashes their actual confined bytes and rechecks the receipt. It preserves
lossy/partial/path-dependent/unknown fidelity and warnings; accepted publication
bytes alone do not establish quality, native application opening, creative review,
publisher authenticity, release qualification or external publication authority.
An unavailable/unsupported/pruned requested branch cannot be hidden by a successful
subset. Explicit resume must name `RUN/native/output` and retain the original
forest. Frozen native package resume is not substituted for graph recovery.

The candidate APIs can operate once exact independently built artifacts/descriptors
are supplied and explicitly approved. They do not install or build those artifacts
or relabel the still-unpublished provider releases. Renderflow #419 and Flow
#52/#70/#71/#72 retain packaging, installation and native qualification work.

The `native` library API prepares and executes a source-candidate route from
explicit operator pins. Initial support is Aniflow `music-video-source-v1`, using
`aniflow.release-contract/v1` from the provider's separate draft checkpoint. It
requires an independently built binary and exact descriptor; no sibling checkout,
PATH-based provider selection, download, installation or paid/cloud fallback.

1. Select a `flow.release-cohort/v1` row with `flow_route`, matching host target,
   implemented profile, immutable binary version/revision/hash/size, and descriptor
   contract/hash. Keep publication and qualification unobserved until verified.
2. Supply a closed `flow.native-request/v1` with the cohort path/hash, `row_id`,
   `provider_executable`, `release_descriptor`, ordered `inputs`, `configuration`,
   fresh `run_directory`, explicit `tool_directories` and bounded `limits`.
   Native paths use the existing lossless `optiflow::NativePath` encoding. Aniflow
   accepts one source video and native `aniflow.source-music-video-recipe/v1` JSON.
   Recipe tool/model and manually supplied timed-text pins remain native-owned.
3. `native::prepare` only reads selected files and hashes bytes. `native::approve`
   requires an operator declaration bound to that plan, acknowledging both
   `trusted_unconfined` and `unqualified_candidate`. No release qualification is
   inferred from this approval. `native::run` is the explicit launch operation.
4. Run performs the compiled descriptor handshake, correlates native source plan,
   original source hash, persisted intent, outcome and fresh complete status. It
   requires the native final-video completion, both master and delivery, matching
   assembly identity, confined exact output hashes and stable native status. Exit
   zero, a partial recipe or an existing file cannot commit Flow acceptance.
5. `native::status` observes saved native acceptance and current source/config/
   provider/output bytes. It does **not** claim a fresh native domain probe.
   `native::resume(root, native_run, token)` explicitly names an existing native
   run under the reserved output parent, binds its original intent and delegates
   prefix recovery to Aniflow. Unknown runs, moved/stale sources, incomplete
   initialization and exhausted attempt bounds require recovery, not replay.

The operator chooses explicit canonical tool search directories. Child environment
is cleared, with only that selected PATH added. Tool discovery/versions and exact
models remain subject to native configuration and current native admission.
Flow's deadline/output/hash limits are cooperative process/I/O controls; they are
not OS quotas, descriptor-bound launch, network isolation, sandboxing or descendant
containment. The common direct-child supervisor reaps the child and bounds waiting
for retained pipes. Descendant-held stream readers may be detached at deadline;
this does not terminate those descendants.

Operation state is private, create-new and synchronized. A per-operation lock
rejects concurrent writers. Transcripts, native plans, started markers and accepted
domain evidence remain distinct. Interrupted/cancelled attempts preserve evidence
and require explicit inspection/resume; no source or retained attempt is deleted.
Local approvals/receipts are unauthenticated operator records, not signatures.
Native acceptance and current hashes do not grant creative approval, publishing,
signing, timestamps, original replacement/disposal or release qualification.

Authored source only; no tests, builds, validators, native/provider/media/model,
CLI/SDK or CI execution occurred. Actual published providers, clean installation,
positive native workflows and platform/dependency qualification remain under
Aniflow #212 / Flow #51/#70/#71/#72. #88 remains last and unchanged; known alignment
cancellation/start-marker failure is unresolved with cause unproven.
