# Proposed first-consumer mapping

This maps **existing** Renderflow source fields to Flow concerns for #80.
It neither defines a second runtime-profile format nor adopts the native
Renderflow proposal as a suite standard. Exact released adapter calls and
configuration precedence remain later contract work.

| Existing owner/field | Flow concern | Preserved limit |
| --- | --- | --- |
| Extension manifest/lock and per-invocation authority | Capability/effect/secret/endpoint selection | Provider declarations never grant themselves permission; exact invocation and existing lock own authority |
| `RuntimeProfile.mode` | Disabled, approved loopback local-only or explicitly remote | Installed service/credentials do not enable AI; no implicit model pull or cloud fallback |
| `provider_id`, `model_id`, `allowed_reported_models` | Exact requested and declared observed identity | Catalog availability and aliases are declarations; unknown weight/revision evidence remains unknown |
| `endpoint`, `credential_env` | Deployment address and opaque credential-handle mapping | Keep separate from nonsecret portable process intent; no raw credential values copied to generic extension payloads |
| `local_service_egress_disabled` | Local-runtime egress policy | Explicit operator assertion only, not measured offline enforcement |
| `preparation_reference`, catalog runtime revision/weight digest/license | Preparation evidence handoff | Pointer/digest is not a verified runtime-native weight inspection, model/code license review or full preparation receipt |
| Profile generation and budgets | Explicit supported options, context/output/deadline/calls/cost policy | No arbitrary provider-extension options; one bound inference attempt; hard monetary ceilings refuse; unknown cost remains unknown |
| Native catalog modalities/features/limits | Model-specific capability negotiation | Text summary proof does not imply image/audio/video/tool-loop support |
| `SummaryRecipe.sources` exposure/privacy approvals | Exact approved textual source material | Preserve source SHA and selected byte ranges; remote requires its separate approval; no approval invented |
| Source evidence/slices and artifact store | Frozen input identity and omission coverage | Summary planning can snapshot bytes; a no-write external planning contract must handle that explicitly |
| `PreparedSummary`, skill/output schema and reference validator | Candidate generation/validation | Typed state and schema/reference checks never establish semantic truth, creative acceptance or publication |
| AI execution/cache/profile identities | Requested/observed provider/model/runtime/schema/settings/source/output evidence | Native cache identity exists but this path reports `cache_decision: not_requested`; do not advertise a qualified shared cache |
| Summary review/export | Supplied exact-candidate decision and local accepted-text export | Reviewer reference is a declaration, not authenticated legal authority; publication stays false |
| Existing operation/event/report seams | Observability vocabulary and safe local reporting | Future #14–#21 mapping; raw provider errors/source/prompt/path/secret data are not portable telemetry |

## Preparation evidence still needed

A future explicit preparation path should record runtime-native model source
and immutable revision, observed supported identities/digests, selected files
and storage estimates, code and weight licenses separately, quantization,
device/runtime compatibility, memory/context/concurrency guidance and removal/
rollback guidance. Runtime identity mechanisms differ; do not invent a universal
weight package or treat a text `preparation_reference` as a completed check.

A prepared model may be unavailable, incompatible or unsuitable for the chosen
task. An observed local endpoint still needs evidence for its own remote/cloud
features before an offline claim. No installation, download, service startup,
weight inspection, device probe or inference is performed by this proposal.

## Review and source-only cases

Existing Renderflow profile/runtime/summary source tests are inventoried, not
executed or declared passing here. Later shared conformance should cover AI
disabled, unavailable model, local-to-remote refusal, wrong reported model,
unsupported options/cost policy, malformed/schema-invalid output, injection-like
source text, timeout/cancellation, changed source/model/profile/cache identity,
bounded attempts and exporter failure. Synthetic declaration fixtures cannot
replace real model quality, hardware, egress or semantic review evidence.

Flow's existing fallback rule remains prelaunch-only. A timeout after invocation
must not become another paid/provider call without the existing recovery
authority. Domain consumers remain independently useful; Flow adds no sibling
path dependency and no holon imports Flow orchestration code.

Status: proposed documentation mapping; no new live contract, runtime package,
approved ADR, test result or release. Owner remains #80; final suite acceptance
is #88, scheduled last.

## Proposed preparation checkpoint

[Local preparation](preparation.md) now authors a separate Flow-owned proposed profile/preparation/report contract and inert byte inspection. This is not an adopted mapping or runtime-native verifier. No released domain adapter consumes it yet; the earlier inventory-only observation remains historical.
