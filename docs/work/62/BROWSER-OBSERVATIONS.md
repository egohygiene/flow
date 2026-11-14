# Local scenario-browser observations

Relates to Flow #62. This checkpoint starts from Flow main
`71d034263da17c8755cb2c7202cf68ca5b5df6fd` (tree
`c942b09089c63a8cb0371f61528be6c3e1c472e3`). The earlier source-only handoff and
closeout remain historical. The user authorized focused local source and browser
checks for this sprint; no hosted CI, publication or provider run was used.

## Repairs observed in an actual browser

The [baseline observations](baseline-observations.json) retain the original focus
failures: loading all catalogs for a stable deep link left focus on `BODY`, and
clearing a keyboard-selected linked recipe also left focus on `BODY`. Successive
catalog/manifest renders now preserve a focused linked article; clearing the link
returns focus to search. Ordinary filter changes do not force focus away from the
control the reader is using.

After loading publishing guides at 360 pixels, the document widened to 575
pixels. The [intermediate failing receipt](narrow-overflow-before-proof.json)
preserves that failure and the exact source identity before the wrapping repair.
Long workflow text and a raw SHA overflowed their containers. Inherited text
wrapping now keeps the same content within the narrow viewport. No frontend or
canonical scenario fixture was replaced.

## Executed checks

From this checkout:

```console
node --test tools/test_scenario_showcase.mjs tools/test_workflow_guides.mjs
python3 -B tools/generate_scenario_sources.py --check
git -c core.hooksPath=/dev/null diff --check
```

All 12 existing model/source-guide cases passed. All five deterministic scenario
source recipes validated without writing generated fixtures. The whitespace
check passed.

The optional [browser replay](../../../tools/prove_scenario_browser.mjs) used a
loopback Python HTTP server and already-installed Playwright 1.62.1 with actual
headless Chromium 151.0.7922.173. The native executable was 321,686,360 bytes,
SHA-256 `d387400aaf740ccb75e5e996a34aa0940e6e97683a4eabd6ae34c1eed6804723`;
the replay rehashed it after use. The cloud desktop tools had no desktop/browser
capability, so no desktop or assistive-technology observation is asserted.

The [passing browser receipt](evidence/20261114/browser-proof.json) contains 16 completed checks,
including exact canonical IDs, readable success/refusal decisions and an exact
lifecycle expected trace; keyboard search, family/link/clear behavior; deep-link
focus through every catalog/manifest render; narrow layout and expandable JSON;
workflow-source separation; reset and malformed-fragment handling. It presents
missing, mismatched, incomplete, conflicting and matching-but-unconfirmed report
states. The selected report inputs are explicitly synthetic and are never
accepted as current verified execution evidence.

The browser loaded 81 acceptance and 49 lifecycle source recipes, five indexed
manifest examples and six source workflow guides. These totals were derived
from the current source. The exact ordered scenario IDs and representative
rendered expectations were compared with those source rows. More strongly, all
16 actual HTTP response bodies for the viewer, modules, styles, catalogs, index,
manifests and workflow guide catalog matched the checkout's bytes and SHA-256.
The source identity also hashes the replay script and workflow schema, which the
viewer does not fetch. Thus this proof does not infer source equality from
equal counts. There were 39 same-origin GET requests, zero blocked/external
requests and zero page errors; no upload request occurred.

The report SHA-256 is
`9972b58a46f584d51607cb816bfd2e327d5ccbe762457683d4cd1da0e9a176a1`.
Its content-addressed source inventory digest is
`c2e2df39c089a28118d214e7a9ec9578664e3ad52ee229cb04d0cda6861b29ec`.
These identify the recorded source/observations, not an authenticated release.

## Screenshots and limits

Viewport screenshots were visually inspected after the successful run and are
preserved beside the receipt: [desktop](evidence/20261114/desktop-unconfirmed.png)
and [narrow](evidence/20261114/narrow-linked-expectation.png). Their relative names,
byte lengths and hashes are in the receipt. The desktop image shows textual
source-unconfirmed and unavailable states with raw identities; the narrow image
shows wrapped explanation text, an expanded JSON expectation and visible
keyboard focus. Earlier screenshots remain in the local workspace evidence
beside the failed run. No screenshot contains private user media or a native
provider result.

Other browser engines, assistive technologies, deployed Pages behavior,
authenticated report freshness and actual native/provider qualification remain
unproved. This bounded UI observation does not run Rust scenario workloads or
complete the final Flow #88 suite gate. #62 and #88 are not automatically closed;
maintainer reconciliation follows the draft checkpoint.
