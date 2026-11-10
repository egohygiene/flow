# Proposed local AI preparation evidence (#80)

This is a proposed interoperability checkpoint, not an adopted suite runtime or installable harness. `flow::ai_preparation::{Profile,LocalPreparation,inspect}` is an explicit read-only SDK surface. Flow still has no supported product CLI for this feature. Domain holons do not import Flow or sibling source. The existing native Renderflow profile and extension contracts retain their wire meanings; adapters require a separate reviewed immutable released interface.

The closed nonsecret `flow.ai-preparation-profile/v1` records exact provider/model/runtime/template/output-schema identity and one bounded text/structured inference intent. Its finite byte/token/deadline/context/concurrency/memory/working limits are selected declarations. One attempt and one concurrent invocation are supported; unsupported modality, non-structured output and hard monetary ceilings produce `incompatible`. Unknown cost remains unknown. An ordinary configuration resolver must produce the selected explicit profile: defaults < user < project < recognized environment < explicit CLI remains the integration mapping, not a second resolver here. No environment is read by this module, and no profile enables processing by merely finding credentials or a service.

The separate PRIVATE `flow.ai-local-preparation/v1` declares a literal loopback socket, operator egress assertion, runtime and model licenses independently, quantization, exact profile pin and1..32 canonical local files. There is exactly one runtime executable plus weight/config subjects. Each file is pinned by SHA-256 and byte count (at most8GiB each); declared total is at most128GiB. Observation reads each selected regular file at most twice, with cancellation checked each64KiB. Paths and endpoint are not portable recipe fields. Keys/credential values are not accepted anywhere in these shapes. The private preparation itself must remain local.

`inspect` never opens a socket, invokes a service/executable, starts inference, installs/downloads, writes artifacts or retries elsewhere. Disabled mode ignores even an unavailable preparation without accessing files. Remote mode explicitly refuses this local-only surface. Missing/unavailable/changed assets remain actionable states; callers can retain the structured report themselves. Diagnostics are fixed codes and portable asset IDs/digests/byte counts. They omit raw errors, local paths, endpoint, source prompts and license text. No generic extension gets a credential or inherited environment.

`prepared_bytes` means selected current file bytes matched the supplied pins. Runtime/model revision, licenses and quantization remain declarations. It does not verify a runtime-native manifest, GGUF structure, shard relationships, signed origin, license compliance, VRAM/RAM reservation, schema support, service health, actual weight loading, local egress isolation or useful model quality. All qualification/authority flags remain false. Two observations detect ordinary changes, not an atomic filesystem snapshot, descriptor-bound launch or adversarial race immunity. There is no reusable invocation acceptance or persistent cache in this API.

Preparation is operator-managed: obtain the exact selected runtime/model using its supported tooling after reviewing model/code licenses, disable its hosted features, record actual runtime-native identity and observed files, then inspect the explicit declaration. No product action performs these instructions automatically. Rollback/removal belongs to the runtime's supported tooling and operator, after checking other references; Flow neither deletes model bundles nor claims that removal reclaims physical disk. A failed preparation does not affect ordinary deterministic media processing.

The [synthetic profile](examples/preparation-profile.json) is reviewable and not a download/install recipe. Match a private preparation's `profile_sha256` to `Profile::sha256` using Flow's existing JSON-value/SHA-256 convention; asset digests must describe actual selected bytes. It grants no launch. Extension manifests/locks, explicit invocation authority, secret handles, prelaunch fallback and isolation remain their existing owners. Operational reporting uses the existing #14–#21 proposal, with telemetry excluded from profile identity. No new client, exporter, gateway or shared crate is added.

Next gates remain runtime-native/hardware/egress proof, an immutable released Renderflow adapter, safe configuration/credential handoff and an authorized synthetic consumer. AMV/Optiflow domain evidence packets can progress independently without adopting this proposal. Seven authored SDK cases are uncompiled/unrun. All execution/checks remain deferred to flow #88 last; no ADR ratification, Identity adoption or model/hardware readiness is asserted.

## Selected local setup and doctor source

`ai_preparation::setup::prepare` accepts an explicit private request selecting
existing runtime/weight/configuration files. It derives complete SHA-256/size
pins, respects independent expected pins when supplied, and reuses the existing
inspector to confirm the set before returning a private preparation. One shared
cooperative setup deadline checks cancellation between 64 KiB reads. Hashing may
read each selected asset three times (derivation plus existing two-observation
confirmation); no runtime-native identity, file format, shard relation, loaded
model, hardware or offline enforcement is inferred.

`setup::diagnose` projects existing observations into fixed manual actions for
missing, unreadable or changed assets and unqualified runtime/license/egress/
resource checks. Portable doctor output contains no paths, endpoint, license
text, raw errors, shell commands, prompts or secrets. Selected logical bytes
are not physical storage measurements. All automatic-action flags are false.

Disabled and remote/incompatible profiles stop before selection observation.
Setup never starts/contacts a service, installs/downloads or grants invocation.
Five synthetic SDK cases and the new closed request/result/doctor schemas are
authored, uncompiled/unrun. No checks, product/native/model calls or CI ran.
