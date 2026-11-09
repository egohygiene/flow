# Optional AI integration inventory

Source observation: November 9, 2026. Owner: [Flow #80](https://github.com/egohygiene/flow/issues/80).
This checkpoint inventories existing clients and recommends the smallest first
seam: reuse Renderflow's native catalog/profile/skill/summary owners. Add no
model router, gateway, shared crate, runtime installer or mandatory AI step.
It is a source recommendation, not accepted suite profile or live-provider proof.

## What already exists

| Owner and exact source | Existing source boundary | Still absent or unqualified |
| --- | --- | --- |
| Flow `2655158b3b55bccbf85767e5000c693710c671b6` | Extension manifests/locks, requested vs granted effects, opaque secret handles, prelaunch-only fallback, bounded process transport, durable intent/acceptance/recovery | AI-specific interoperable deployment/preparation contract; supported product CLI; released Renderflow adapter |
| Renderflow `9bfb49446f6cf570af01e33412b48cbd4ff03763` | Model-specific catalog, native skill runtime, Ollama and OpenAI-compatible HTTP adapters, closed optional runtime profile, source-grounded summary candidates and separate review/export | Verified local runtime/weights preparation, hardware/quality proof, released Flow consumption; broader modalities/tool loops and budget guarantees |
| Aniflow `8edb41e3a81ea0015e725144757f6efcb7d4fb0c` | Typed provider/runtime/toolchain/evidence owners; domain model-backed temporal/audio operations | Dedicated generic text/vision AMV review consumer under #116 and mapping of an agreed optional profile; RIFE/Upscayl/transcription are separate domain processors |
| Optiflow `733e7b08e9a80e0b23f3d30c401e75d1272fa5db` | Immutable inventory/report/planning evidence and typed in-process/process extension seams | Dedicated optional explanatory model consumer under #133; explanations must never grant mutation or replacement authority |

The [source inventory](inventory.json) pins each inspected public file by Git
revision, SHA-256 and line count. It is documentation data, not executable
configuration, a provider lock or an installable model catalog. Dedicated
Aniflow/Optiflow consumer absence is a bounded source-tree observation; their
existing domain providers are not reclassified as a generic chat harness.

GitHub release metadata was read separately: no published Flow, Renderflow or
Aniflow release was returned; Optiflow v0.1.1 and v0.1.0 were returned. The
existing supported Flow bridge is the read-only Optiflow v0.1.1 adapter. Source
package versions do not establish independently installable suite compatibility.
No releases, assets, models or services were installed or invoked.

## Distinguish the pieces

- A local inference runtime is the separately prepared service/process that
  loads a model. A loopback endpoint alone does not establish offline operation.
- A provider client adapts a bounded request to an existing runtime/API. The
  first consumer already has two such clients in Renderflow.
- A gateway is an optional additional service. Its routing, retries, credentials,
  residency and egress would need explicit policy and evidence.
- An inference step produces untrusted candidate data. It is not an agent loop.
- An iterative agent would need declared tools and finite calls/steps. No first
  summary requirement calls for one; no such loop is selected here.

## Compare the integration choices

| Option | Incremental source/dependency effect | Capabilities and limits | Recommendation |
| --- | --- | --- | --- |
| Reuse native Renderflow profile → skill → provider | No added crate/service. Existing core directly declares `ureq = 2` with JSON; its lock resolves 2.12.1. Workspace declares MIT and Rust 1.94 | Existing summary supports selected text and schema-constrained candidates. Profile scopes one provider/model and one inference attempt. Native validation/review owners remain local | First source seam; adapt only through a future immutable released interface |
| Introduce a Rust provider wrapper, e.g. genai or Rig core | New API/dependency and compatibility/migration work | Could matter for a concrete missing backend/modality. Current release/MSRV/license/maintenance and actual transitive/binary cost were not evaluated in this source-only inventory | Not selected; perform an exact-version comparison only if an existing client cannot satisfy an approved consumer |
| Introduce an optional gateway, e.g. LiteLLM | Adds a separately installed/configured service and extra operational/egress boundary | Hidden retries, provider aliases and cloud fallback could conflict with the exact invocation/authority contract. HTTP shape does not prove equivalent tools/modalities/schema behavior | Not selected for the first proof; no installed or qualified gateway is claimed |

This is not a completed upstream dependency audit. Source manifests and a lock
are observations, not evidence that any upstream dependency is maintained,
secure or suitable on the operator's hardware. No binary-size, startup, memory,
throughput or transitive-license measurements were run. Code and model-weight
licenses need separate review. The first seam requires no newly selected
external provider dependency, so those unknown comparisons need not delay its
bounded source mapping.

## Read the existing profile honestly

Renderflow `renderflow.ai-runtime-profile/v1` is explicitly documented in its
source as a **proposed Flow #80 mapping**, not a ratified suite specification.
It exposes disabled/local-only/remote modes and disabled/unavailable/incompatible/
refused/declared-ready profile results. `runtime_qualified` remains false.

Local-only requires a literal loopback socket and an explicit assertion that
the service's cloud/egress features are disabled. Its declared-ready path also
requires available model/catalog facts, a weight digest, runtime revision and
preparation reference. These are supplied declarations; the profile does not
open weights, attest a runtime, reserve VRAM or prove that egress is disabled.
The checked-in local example is deliberately not runnable as supplied.

The selected profile constrains one provider/model, clears provider-extension
settings, refuses unsupported sampler/schema options and sets retries to zero.
Bounded response mode disables redirects and ambient proxies for literal local
endpoints; read cancellation is checked between chunks. Ordinary runtime paths
outside this binding retain their own behavior and must not inherit these claims.

Finite input/output/response/token/deadline limits exist in this source mapping.
Hard monetary ceilings are **not supported**: a supplied cost ceiling is
refused, and remote unknown cost needs explicit permission. Unknown cost is not
zero or evidence of free hosted use. No model quality/reproducibility guarantee
follows from a seed, accepted schema or matching model name.

The native summary freezes selected UTF-8 source slices and records omitted
bytes. It checks references and produces a review-required candidate; reference
checks do not establish paraphrase accuracy or inspect audio/video/images.
Explicit source exposure and remote privacy approval remain separate. Subsequent
review/export is bound to the exact candidate and still grants no publication.

`RuntimeProfile::inspect` does not invoke inference. The existing summary
capture/plan path **does write source snapshots to its artifact store**; it
must not be advertised as a no-write Flow planning port. The AI doctor includes
a connectivity path, so it is not an offline source-inspection operation.
None of those commands were executed for this inventory.

## First consumer and remaining gates

Use the [native-field mapping](consumer-mapping.md) as a review proposal,
preserving existing Flow extension authority and domain validation owners.
The first eventual proof is a bounded synthetic textual summary candidate,
not a new generic router or automatic AMV edit/review tool.

The dependency order is: review this inventory/mapping → define exact
interoperable profile/preparation evidence → consume an immutable released
Renderflow capability through its Flow adapter → explicitly qualify one
prepared local runtime/model/hardware path. Remote comparison is a separate
opt-in. Aniflow #116 and Optiflow #133 remain owning domain consumers, not sibling
source imports or prerequisites for ordinary publishing.

Missing runtime/model preparation, accepted profile ownership, immutable
provider release, actual source permission and native/hardware/semantic review
stay explicit. Normal plan/run must not install dependencies, start services,
pull models or silently choose a paid remote fallback. An installed runtime,
credential or catalog entry does not enable AI.

Use [#15–#21 observability](../observability/README.md) for future operational
correlation/privacy; no parallel telemetry system is selected. Trace IDs,
timing/exporter settings and metrics never become artifact/cache/plan authority.
Prompt, source, private path and credential content stays out of portable logs.
The existing proposed observability profile is not claimed adopted by this work.

All tests/builds/checks/native/provider/media/model/CLI/SDK/CI execution remains
deferred to [Flow #88](https://github.com/egohygiene/flow/issues/88), scheduled
last with its original ledger intact. No inference demo, benchmark or provider
qualification ran. The known Aniflow alignment cancellation/start-marker failure
remains unresolved, cause unproven. #80 remains open for its full preparation,
consumer, operational and qualification criteria; no ADR or Identity adoption,
actual private content review, release or publication is asserted.
