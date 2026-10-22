# Provider capability matrix

## Current evidence — October 22, 2026

Source revisions, package versions, immutable releases and executed compatibility
are separate evidence. This matrix does not qualify arbitrary current source or
every platform. Re-query the [current handoff](../work/11/HANDOFF.md) and the
owning issue before implementing an adapter.

| Repository and inspected main | Release available to Flow | Supported integration evidence | Remaining owner |
| --- | --- | --- | --- |
| Flow `aadf053d48ce8826ee5b9ca4a134985f4e7910e3` | No published GitHub release; package `publish = false` | Durable library/kernel and native read-only Optiflow adapter merged; synthetic fixture binary only | Product CLI [#53](https://github.com/egohygiene/flow/issues/53), first candidate [#54](https://github.com/egohygiene/flow/issues/54), qualification [#34](https://github.com/egohygiene/flow/issues/34) |
| Aniflow `7a808ff99ca8f557f6a3ba7fd373a393de0feee6` | No published GitHub release | Merged temporal/audio/toolchain implementation is source evidence; Flow adapter compatibility is not qualified | Provider release [Aniflow #10](https://github.com/egohygiene/aniflow/issues/10), deferred qualification [#64](https://github.com/egohygiene/aniflow/issues/64), Flow adapter [#51](https://github.com/egohygiene/flow/issues/51) |
| Optiflow `d03e53f2e397af03920070d9bb6c076d7827e12b` | Read-only v0.1.1, source `b82599a2231e997d42fd9f26f4b59587f4ae14cf`; newer main is not that release | Native `optiflow.command-result.v1` CLI adapter: scan, report, exact-duplicate review plan. Pinned archive/executable identities and actual synthetic Linux execution; macOS archives pinned, native execution unverified | Completed Flow [#50](https://github.com/egohygiene/flow/issues/50)/[#81](https://github.com/egohygiene/flow/issues/81); later provider release [Optiflow #93](https://github.com/egohygiene/optiflow/issues/93), mutation/PNG adapters [#73](https://github.com/egohygiene/flow/issues/73)/[#74](https://github.com/egohygiene/flow/issues/74) |
| Renderflow `9dac5428df38691b2922e09ea02d51b02a7f43eb` | No published GitHub release | Merged publication/transform implementation is source evidence; Flow adapter compatibility is not qualified | Provider release [Renderflow #419](https://github.com/egohygiene/renderflow/issues/419), Flow adapter [#52](https://github.com/egohygiene/flow/issues/52) |

The [Optiflow integration guide](optiflow-v0.1.1-read-only.md) owns exact binary
digests, schemas, effects and native limits. Its Flow receipt is a separate
local audit record, not a generic graph checkpoint or mutation grant. Dry-run,
quarantine, restore and finalize remain explicitly unsupported by this pinned
adapter. The [#81 receipt](../validation/flow-81-local.json) records focused
Linux/Rust 1.85.0 evidence; broader clean-room qualification remains #34.

Source-only provider work and retained deferred checks cannot establish native
readiness. An adapter decision must record exact release/artifact identity,
interface and accepted schema versions, effects, runtime tools, observed
outputs, deterministic compatibility, and explicit unavailable/untested states.

## Prior architectural baseline — historical

The original table below preserves the August 13 Aniflow/Renderflow source
observations and the later pinned Optiflow entry. Package version labels and
earlier ordering decisions are not current release or compatibility evidence.
Use the current table and owning issues above for execution.

| Provider snapshot | Observed domain capability | Current integration evidence | First-slice role | Adapter decision |
| --- | --- | --- | --- | --- |
| Aniflow `20783d7374298e5fd44c782348a3c944c9318656` / package v0.2.0 | temporal inspection, decomposition, ordered processing, reconstruction, validation | single-crate behavior documented on 2026-08-13; final public library contract not yet proven | create and validate a new temporal master | pending release reinspection; library if stable, otherwise CLI |
| Optiflow immutable v0.1.1 (`b82599a2231e997d42fd9f26f4b59587f4ae14cf`) | scan, report, exact-duplicate review plan; dry-run/quarantine/restore/finalize unsupported | [release adapter and synthetic Linux fixture](optiflow-v0.1.1-read-only.md) pin binary/contract/schema/effects; macOS archive digests pinned but native run pending | read-only source inventory and later rescan | native `optiflow.command-result.v1` direct CLI adapter; Flow-owned receipt, no mutation authority |
| Renderflow v0.2.1 snapshot documented on 2026-08-13 | transform DAGs, documents, image/audio conversion, plugins and build caching | existing core/CLI/plugin-SDK split; suite result compatibility unproven | none | defer until restore-and-assess passes |

An adapter is compatible only when a fixture records the provider version,
interface kind and version, accepted Flow contract versions, side effects,
observed outputs, and a deterministic compatibility result.
