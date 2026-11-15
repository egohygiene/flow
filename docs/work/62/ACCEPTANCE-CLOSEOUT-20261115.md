# Scenario showcase acceptance closeout — November 15, 2026

This checkpoint reconciles the six original acceptance criteria in
[Flow #62](https://github.com/egohygiene/flow/issues/62) against implemented source
and retained actual observations. All six documentation-presentation criteria are
satisfied in their bounded scope. Close #62 after review of this closeout and
the integration-index navigation link; retain the separate final #88 gates.

The audited base is merged main `80127ca4f9ad14ebb8e8c5a8f7321f242aa36607`,
tree `e50ff08a3828b7c5febc6ded5fb3d131aed7a9f2`. No viewer, model, runtime,
scenario catalog or expected outcome was changed. README already links the guide;
the integration index now does too. Older source-only handoffs remain history.

| Original criterion | Implemented owner and observed evidence |
| --- | --- |
| Browse all current scenarios and understand success/refusal without Rust source | The [viewer](../../integrations/showcase/index.html), [row explanations](../../integrations/showcase/explanations.mjs) and expandable expectations read the canonical catalogs. The retained browser checks compare every ordered ID and representative acceptance/refusal/lifecycle text. The current Node cases check an explanation for every canonical row. |
| Stable canonical IDs, synchronized counts and expected outcomes | The [model](../../integrations/showcase/model.mjs) reads canonical rows directly; it does not keep another scenario/count registry. Current Node checks exercise actual catalogs; all five existing deterministic source recipes validate. The observed snapshot is 81 acceptance plus 49 lifecycle recipes, not a permanent count or 130 native passes. |
| Separate expectations, observed verification and coverage gaps | Text labels distinguish source intent, unavailable reports, catalog mismatch, incomplete/conflicting reports and source-unconfirmed supplied claims. Known gaps/budgets remain expandable. The retained browser observes every state and never promotes a matching synthetic report into native acceptance. |
| Identify tested revision and stale/unavailable evidence | Selected reports expose their recorded tested-source SHA-256, raw report SHA-256 and toolchain declaration. These identify the supplied content-addressed source revision; authenticated Git commit/time and present native validity stay unconfirmed. Changed catalog bytes cannot appear current. [The guide](../../integrations/scenario-showcase.md#read-the-states-honestly) explains the source/report owners and limits. |
| Readable diagram text equivalents and non-color-only status | The [guide's two diagrams](../../integrations/scenario-showcase.md#representative-paths-and-their-text-equivalents) each have a plain-text equivalent. Evidence labels are words, not color keys; HTML has labeled controls/status regions. The retained desktop/narrow screenshots show readable state text and visible keyboard focus. This does not claim assistive-technology qualification. |
| Local existing tooling, refresh instructions and no paid CI | The [documented Python-stdlib server](../../integrations/scenario-showcase.md#open-a-local-checkout) serves a static page: no frontend install/build is needed. Canonical refresh and existing Node/Python checks are documented. A fresh loopback smoke served all 16 browser-requested sources plus README, integration index and guide, matching current bytes. No hosted CI or deployment was used. |

## Exact observation scope

The [portable closeout receipt](evidence/20261115/closeout.json) records current
source and log hashes. On this base, the existing 12 Node model/guide cases and
five deterministic source recipes pass. Nineteen current loopback GET responses
match the checkout's bytes and hashes; the server is closed after the smoke.

The [actual Chromium receipt](evidence/20261114/browser-proof.json) and its
[walkthrough](BROWSER-OBSERVATIONS.md) remain the November 14 execution. Its
18-file content-addressed subject is byte-identical to the corresponding current
files. All 16 retained served-body identities and both screenshot hashes also
match. The receipt records 16 passing browser checks, 39 same-origin GETs, zero
blocked requests and zero page errors. The current audit reuses that exact source
observation; it did **not** rerun Chromium or install a browser. The preserved
screenshots were read back for their textual states, wrapping and keyboard focus.

This closeout concerns the documentation surface. It does not qualify native
provider execution, authenticate supplied reports, establish current report
freshness, approve content, deploy Pages, test other engines/assistive technology,
or finish Flow #88. Pages deployment was not one of #62's six original criteria.
The original #88 acceptance ledger remains unchanged and last.

To refresh, use the existing commands in the [guide](../../integrations/scenario-showcase.md#keep-the-presentation-current).
A later change to the observed viewer source or browser needs its own observation;
a matching saved report must still remain source-unconfirmed in the viewer.
