# Provider capability matrix

This is an architectural evidence baseline, not a compatibility guarantee.
Implementation work must reinspect named provider releases and replace each
`pending` interface decision with tested evidence.

| Provider snapshot | Observed domain capability | Current integration evidence | First-slice role | Adapter decision |
| --- | --- | --- | --- | --- |
| Aniflow `20783d7374298e5fd44c782348a3c944c9318656` / package v0.2.0 | temporal inspection, decomposition, ordered processing, reconstruction, validation | single-crate behavior documented on 2026-08-13; final public library contract not yet proven | create and validate a new temporal master | pending release reinspection; library if stable, otherwise CLI |
| Optiflow v0.1.0 snapshot documented on 2026-08-13 | read-only collection discovery, identity/relationship evidence, reporting | mutation is explicitly outside the v0.1 safety boundary | scan source before processing; rescan source plus output | pending release reinspection; prefer structured read-only CLI if library is not stable |
| Renderflow v0.2.1 snapshot documented on 2026-08-13 | transform DAGs, documents, image/audio conversion, plugins and build caching | existing core/CLI/plugin-SDK split; suite result compatibility unproven | none | defer until restore-and-assess passes |

An adapter is compatible only when a fixture records the provider version,
interface kind and version, accepted Flow contract versions, side effects,
observed outputs, and a deterministic compatibility result.
