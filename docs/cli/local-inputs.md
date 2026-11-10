# Explicit local inputs and profile preview (#70)

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
| Renderflow publication / Aniflow temporal | Unavailable until exact immutable adapters exist |

The scope-plan result uses `scope_plan_only` and partial exit9. Inspection hashes
and budgets are **not** passed off as execution-input freezing or provider
runtime enforcement. The nested/native saved plan explicitly permits whole
read-only directory scope; the native scan owns actual member/content evidence
at execution. A new/changed member cannot be represented as an input-matched
transform by this wrapper. The existing operation approval/run/status path acts
only on that visible scope plan. Full input-bound publication/temporal execution,
provider-specific quotas and clean-install proof remain #70 acceptance.

Unavailable planning returns a structured reason and writes no executable plan.
Inspection-only output may save a no-clobber inventory snapshot; it cannot be
passed to `run` as an execution plan. Output snapshots and scope plans stay
separate from selected source/provider paths. Inputs/paths are private local
records, excluded from portable telemetry by default. No source is modified.

Seven local-input and one additional parser case are authored/uncompiled/unrun.
No tests/compiler/build/dependencies/lint/format/schema/docs/native/provider/model/
media/CLI/SDK or hosted CI execution occurred. ADR-0016 remains proposed; #53/#70
full dependency/release/qualification criteria remain. Flow #88 stays last.
