# Browse Flow's scenario evidence

## Plain-language recipe explanations

Every displayed recipe has a readable title, boundary purpose, setup description
and expected decision derived from its loaded canonical row. The stable scenario
ID remains visible and linkable. Reusable family descriptions explain why the
boundary matters; there is no duplicated scenario/count inventory. Lifecycle
summaries count only the declared final snapshot and never actual executions.
Unmapped future families are visibly unclassified until their copy is supplied.

A declaration that expects rejection says explicitly that no artifacts are
accepted and that correct refusal can pass a test. That explanatory sentence
does not establish that the test ran. The original typed expectation remains
expandable alongside missing/mismatched/unconfirmed report states.

The focused source test suite reads both actual canonical catalogs and requires
an ID-keyed explanation for every current row. It also checks declared refusal
and lifecycle semantics. Execution results belong in the closeout receipt;
authored tests alone do not establish browser, native or provider qualification.

## Stable links across catalogs

Scenario links now qualify both the canonical catalog family and scenario ID:
`#scenario=acceptance/scenario%3Aexample`. Existing ID-only links remain readable
when exactly one loaded catalog matches. Multiple matches stay ambiguous;
missing catalogs require explicit local selection. Links fetch nothing.

A linked recipe remains visible alongside filter matches, has a text marker
and keyboard/screen-reader focus, and can be cleared without changing loaded
evidence. Fragment selection never changes expected outcomes or turns supplied
report claims into current verification. Three focused link/refusal regressions
are included in the executed local model suite. The local browser walkthrough
also checks focus through successive catalog renders and back to search when
clearing the link. This does not establish provider execution.

## Publishing workflow source guides

The same local page now has a separate **Load publishing workflow guides**
control and search. Its [versioned source catalog](showcase/workflows.json)
covers the static publication preset, local AI preparation, tool-free video
folder preview, Sonic descriptions, manual-SRT pilot and publishing audio report.
Each guide links a pinned public source revision and owning issue/PR, with explicit
input/output intent, stage owners, prerequisites and remaining gates. The catalog
is a dated source snapshot; it does not query current PR, release or host status.

Guide counts are derived from that loaded catalog and remain separate from
canonical scenario counts and selected execution reports. Guides cannot claim
runtime qualification or observed executions, even if supplied through the local
file picker. They do not become provider capability, installation or acceptance
evidence. Human review, native/resource/model/platform qualification and actual
publishing remain separate work. The new parser rejects unknown records,
duplicate IDs, traversing source references and nonallowlisted links. Values are
rendered as text nodes; source links are not fetched automatically. The existing
same-origin/no-upload CSP, bounded reads and local report states remain intact.

The three source-guide model cases passed locally alongside the nine scenario
model cases. A headless Chromium walkthrough loaded and searched the guides,
kept their counts separate and checked a 360-pixel viewport. Broader browser and
assistive-technology qualification stays with #88; this adds no paid CI, runtime
engine, telemetry or publication authority.

The [scenario showcase](showcase/index.html) is a local, read-only documentation
browser for the current synthetic catalogs and indexed contract examples. Search
by stable scenario ID, expected outcome, recipe or recovery classification, filter
families, and expand expected typed traces without opening Rust source.

IDs and counts come directly from the selected canonical JSON. No duplicated
scenario registry or frozen total is maintained by the page. Loading source
recipes does not run tests, launch providers, build the suite or produce evidence.
The [local browser receipt](../work/62/BROWSER-OBSERVATIONS.md) records the bounded
model, source-drift and actual Chromium checks. Full suite qualification remains
with [flow #88](https://github.com/egohygiene/flow/issues/88).

## Open a local checkout

When local viewing is desired, serve the checkout root with the standard Python
tooling already used by Flow. This is an explicit local viewer command, not a
product Flow CLI or test command; it is not run by opening the Markdown.

```console
python3 -m http.server 8763 --bind 127.0.0.1 --directory .
```

Open <http://127.0.0.1:8763/docs/integrations/showcase/> in a modern browser and
choose **Load repository catalogs**. The page fetches only these same-origin
source files after that explicit action:

- [Acceptance catalog](../../tests/fixtures/acceptance-scenarios.v1.json):
  resolution, contract, artifact, provider and privacy recipes.
- [Lifecycle catalog](../../tests/fixtures/lifecycle-scenarios.v1.json):
  durable execution, stale state, recovery, authority and cleanup recipes.
- [Canonical example index](../../contracts/fixtures/scenarios/canonical-digests.v1.json)
  and its explicitly indexed scenario-manifest examples.

The JSON picker can instead select copied catalogs, manifest examples or existing
normalized report files. It reads them in the browser; filenames are display
labels and never become network locators. Each document is bounded to 8 MiB, with
at most eight selected files/32 MiB per selection. No data is uploaded and no
report is fetched automatically. Clear loaded data removes the current view.
Serving is needed for browser modules; opening `index.html` as `file://` is not
the supported path. No package install, frontend build or paid CI is required.

## Read the states honestly

| Display | Meaning |
| --- | --- |
| Source recipes / expected outcome | Authored test intent, including expected refusal; no execution observation |
| `unavailable` | No report supplied for this catalog |
| `catalog-mismatch` | A supplied report describes different catalog bytes/family; do not call it current |
| `incomplete-or-conflicting` | Report membership, status or outcomes disagree with the selected catalog |
| `source-unconfirmed` | Report declares a pass matching these catalog bytes; present implementation, freshness, authenticity and native receipt validity are unconfirmed |

The viewer displays the exact raw catalog/report SHA-256 and, when available,
the report's recorded `source_identity.digest` and toolchain declaration. That
source digest names the report's content-addressed test-source inventory; the
native reports do not provide an authenticated Git commit or execution time.
The page never invents one or upgrades an unsigned report into current verified
evidence. It performs bounded presentation checks, not the native report
validators. Missing or stale evidence stays visible. Existing
[acceptance](acceptance-scenarios.md) and [lifecycle](lifecycle-scenarios.md)
drivers own actual receipt validation and source inventory checks.

## Representative paths and their text equivalents

```mermaid
flowchart LR
  S[Supplied immutable source] --> R[Resolve provider]
  R --> P[Check declared authority and locked subjects]
  P --> E[Observe process transcript and artifacts]
  E --> A{Acceptance outcome}
  A --> C[Accepted exact artifacts]
  A --> F[Typed refusal with retained evidence]
```

Text equivalent: resolve the selected provider, check authority and locked
subjects, observe execution/artifacts, then assess acceptance. Process exit zero
or a declaration alone cannot select the accepted branch. A resolution-only
success such as `scenario:acceptance-resolution-compatible` still has
`accepted: false`; `scenario:acceptance-resolution-missing-provider` is an
expected unsupported outcome. Browse their exact current catalog declarations.

```mermaid
flowchart LR
  A[Step A] --> B[Dependent step B]
  C[Independent step C]
  A --> S[Freshness or interruption assessment]
  S --> D[Explicit recovery decision when required]
```

Text equivalent: B depends on A; C is independent. Staleness or interruption
requires the recipe's recorded assessment/recovery policy. The page displays
ordered expected snapshots and classifications for cases such as
`scenario:lifecycle-completed-restart`, `scenario:lifecycle-cancel-before-launch`
and `scenario:lifecycle-effect-stale-authority`. It does not execute recovery or
infer exactly-once effects, hardware crash behavior or real-provider support.

## Keep the presentation current

Edit the canonical catalogs through their normal fixture review. Reloading the
page reads the changed rows, expected traces, budgets and gaps directly, without
a generated copy or refreshed count. Unknown catalog/report versions refuse.
The contract-example index supplies only examples that it explicitly names;
its declared canonical digests remain distinct from raw file hashes shown here.

With Node and Python already available, the local model and deterministic source
checks are:

```console
node --test tools/test_scenario_showcase.mjs tools/test_workflow_guides.mjs
python3 -B tools/generate_scenario_sources.py --check
```

The optional actual-browser replay uses an existing Playwright installation and
native Chromium executable. It neither installs dependencies nor changes the
viewer. Start the server above, then select an existing output parent outside
the checkout and a fresh output directory:

```console
node tools/prove_scenario_browser.mjs \
  --url http://127.0.0.1:8763/docs/integrations/showcase/ \
  --chromium /path/to/chromium \
  --output /path/to/existing-parent/fresh-browser-proof
```

If Playwright is installed outside Node's normal module search path, supply
`--playwright-module /path/to/playwright`. The replay requires the exact local
viewer path, allows only same-origin GET requests and refuses an existing output
directory. It checks canonical IDs and rendered expectations, keyboard links and
details, narrow layout, source-guide separation and synthetic report states.
It hashes the browser executable and compares actual served source bodies with
the selected checkout; equal recipe counts alone are insufficient. The JSON
receipt and viewport screenshots contain only these public fixtures and browser
observations. Selected report fixtures are synthetic presentation inputs and
cannot establish execution, freshness or provider qualification.

The recorded run passed 12 model cases, five deterministic source recipes and
16 Chromium presentation checks. Other browser engines, assistive technologies,
deployed Pages behavior and actual native/provider report qualification remain
unproved. No CI workflow, frontend dependency or publication is added. Earlier
source-only receipts under `docs/work/62/` remain historical records of checks
that had not yet run.
