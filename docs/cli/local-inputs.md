# Explicit local inputs and profile preview (#70)

## Measured installed entry points

The [November 15 checkpoint](../work/70/LOCAL-INPUT-ENTRY-20261115.md)
observes 31 actual installed Flow calls through these existing entry points:
single-file Aniflow selection, ordered-collection Renderflow selection and the
immutable Optiflow v0.1.1 recursive-directory route. It includes actual accepted
outputs, status/completed resume and 12 refusal/unavailable controls. The measured
Flow executable remains its exact older package build; current source-equivalence
and each provider/installation pin are recorded separately. This establishes a
bounded Linux candidate experience, without a new build, installation, immutable
Flow/Aniflow/Renderflow release or broader supported-host qualification.

For an ordinary single file, use the existing pinned native selection and save
the executable plan explicitly:

```console
flow plan --input '/private/ordinary source 作品.mp4' --profile aniflow-temporal --native-selection /private/aniflow-selection.json --output /private/video-plan.json --format json
flow approve --plan /private/video-plan.json --approved-by 'execution operator' --acknowledge-trusted-unconfined --acknowledge-unqualified-candidate --output /private/video-approval.json --format json
flow run --plan /private/video-plan.json --approval /private/video-approval.json --format json
```

For an ordered collection, the closed `flow.local-input-native-request/v1` file
contains `request` (local input/profile/limits) and `selection` (the existing
pinned native request). Select `renderflow_publication` in JSON, with the same
ordered member paths in both fields. Use the [ordered-input](ordered-input.example.json)
and [native-request](native-request.example.json) templates for their respective
objects. The selection references the already installed cohort; do not edit that
cohort to register a source or invent a provider identity.

```console
flow plan --request /private/ordered-local-native.json --output /private/publication-plan.json --format json
flow approve --plan /private/publication-plan.json --approved-by 'execution operator' --acknowledge-trusted-unconfined --acknowledge-unqualified-candidate --output /private/publication-approval.json --format json
flow run --plan /private/publication-plan.json --approval /private/publication-approval.json --format json
```

Plan stdout contains the local inventory and its inner native command plan.
`--output` saves **that inner executable plan**, which is the file to pass to
`approve` and `run`; the outer inventory response is not an execution plan.
`status` and `resume` use the native run locator retained in the actual receipt:

```console
flow status --run-directory /private/selected-run --format json
flow resume --run-directory /private/selected-run --native-run-directory /private/selected-run/native/EXACT-RETAINED-NATIVE-RUN --format json
```

These acknowledgements authorize only the explicit candidate execution. They do
not supply human words, timing, visual, audio, rights or publishing approval.

## Inspection contract

The public `local_input` SDK observes one regular file, an explicitly recursive
or nonrecursive directory, or an ordered collection of distinct file locators.
It performs two bounded observations of current membership, native identities
and streamed SHA-256 bytes. Collection order is authored, never inferred or
silently deduplicated; directory order follows stable native path-component
order. Empty files/directories remain visible. The inventory is private local
evidence, not authenticated origin, a media-format claim or a frozen snapshot.

Ordinary file/folder inspection needs no hand-edited provider manifest:

```console
flow inspect --input '/selected/source with spaces.mp4' --format json
flow inspect --input '/selected/folder' --format json
flow inspect --request /private/ordered-input.json --format json
flow plan --input /selected/folder --profile optiflow-read-only --provider-executable /selected/optiflow-0.1.1 --run-directory /private/new-run --output /private/scope-plan.json --format json
```

The [ordered-input example](ordered-input.example.json) is inert. Existing
`--request flow.command-request/v1` operation planning retains its meaning;
`flow.local-input-request/v1` selects the new input/profile boundary. Request
files cannot be mixed with command-line overrides. `--input` inspects filesystem
kind without trusting filename extension. No semantic probing, media processing,
model, installer, download, network request or provider subprocess is started.

Explicit limits cover files, entries, depth, per-file/total logical bytes and a
cooperative deadline/cancellation guard. Streaming uses at most64KiB file chunks;
no whole source file is loaded in memory. Descriptor/path checks surround reads.
Symlinks, file hard links, special nodes, directory mount escapes, inaccessible
members, mutation and exceeded limits refuse rather than being silently skipped.
Nonrecursive subdirectories are named as explicitly excluded subtrees. Initial
native identity support is Unix; unsupported platform guarantees remain visible.
These checks are not a filesystem sandbox, atomic snapshot or hard kernel I/O/
memory quota. Logical sizes do not prove physical free-space requirements.

## Route truth

| Selection/profile | Current result |
| --- | --- |
| Any supported input, `inspect-only` | Filesystem identity inventory, no domain acceptance |
| Whole recursive directory, exact released Optiflow | Partial scope plan using the existing read-only adapter |
| File, collection or nonrecursive directory for Optiflow | Unavailable: cannot broaden into parent/recursive scope |
| Renderflow publication / Aniflow temporal without native selection | Unavailable: select exact cohort/artifact/descriptor/configuration pins |
| Native file / explicit ordered collection with matching selection | Source-candidate plan via `plan_native` or `--native-selection`; explicit candidate approval required |

See [native candidate commands](native-candidates.md). The additive
`flow.local-input-native-request/v1` wraps `request` (this local-input v1 contract)
and `selection` (`flow.native-request/v1`). `plan_native` requires identical file
paths/order and hashes before returning the native command plan. It never expands
a directory into a native collection. Native selected-input limits and local
inspection limits remain separately visible; large files/collections can use a
direct native request instead of this smaller preview profile.

The scope-plan result uses `scope_plan_only` and partial exit9. Inspection hashes
and budgets are **not** passed off as execution-input freezing or provider
runtime enforcement. The nested/native saved plan explicitly permits whole
read-only directory scope; the native scan owns actual member/content evidence
at execution. A new/changed member cannot be represented as an input-matched
transform by this wrapper. The existing operation approval/run/status path acts
only on that visible scope plan. The separately selected native wrapper binds
the current file/member pins to its command plan; this remains an observation,
not atomic input freezing. Released routes, broader clean-host evidence,
provider-specific enforcement and the full #70 acceptance remain open.

Unavailable planning returns a structured reason and writes no executable plan.
Inspection-only output may save a no-clobber inventory snapshot; it cannot be
passed to `run` as an execution plan. Output snapshots and scope plans stay
separate from selected source/provider paths. Inputs/paths are private local
records, excluded from portable telemetry by default. No source is modified.

The initial source-authoring checkpoint authored seven local-input and one parser
case without running them. The later installed observation above records only
its 31 actual CLI calls and three independent observer controls; it does not
claim execution of every Rust case, new product compilation or a green hosted
matrix. ADR-0016 remains proposed; #53/#70 full dependency/release/qualification
criteria remain. Flow #88 stays last.
