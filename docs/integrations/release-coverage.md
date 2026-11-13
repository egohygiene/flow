# Requested release coverage

Relates to #72. A matrix describes its declared rows; it cannot show a target
or profile omitted from the cohort. Supply a separate explicit coverage request
to make the desired scope inspectable:

```json
{
  "schema_version": "flow.release-coverage-request/v1",
  "cohort_id": "COPY_THE_SELECTED_COHORT_ID",
  "required": [
    {"tool": "aniflow", "interface": "flow_route", "target": "aarch64-apple-darwin", "profile": "music-video-source-v1", "row_id": null},
    {"tool": "optiflow", "interface": "standalone_cli", "target": "x86_64-unknown-linux-gnu", "profile": "read-only-v0.1.1", "row_id": null}
  ]
}
```

The selectors are operator intent, not claims that those named profiles have
released support. Use exact profile/row names from the selected provider cohort.
The one-to-512 selectors are closed and duplicate-checked. Multiple matching
rows stay ambiguous until an explicit `row_id` is supplied; no first match wins.

```bash
flow release-matrix --cohort /private/cohort.json --coverage-request /private/coverage.json --format json
```

The existing read-only matrix owner or explicitly selected authenticated evidence
path supplies observations unchanged. Output is `flow.release-coverage/v1`, with
the original matrix, required-row states (`missing`, `ambiguous`, `blocked`,
`unverified`, `green`), unrequested declared rows and an exact request identity.
Missing/ambiguous rows prevent complete requested coverage. All-requested-green
means only the selected scope; `whole_suite_qualified` remains false. This
projection runs no installer/provider and establishes no new native evidence.
Without `--coverage-request`, the existing v1 matrix output is unchanged.

Three meaningful source regressions are authored, uncompiled/unrun. No tests,
builds, dependencies, validators, CLI/native/media or CI were executed. #72's
actual all-applicable-row qualification, artifacts and stable release remain
owed; Flow #88 stays last and untouched.
