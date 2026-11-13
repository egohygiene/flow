# Static publication output catalog

Relates to #10. After preparing an explicit locale/source publication project
and running separately approved native branches, inspect current output references:

```bash
flow publication-project-catalog --workspace /private/static-project --record-sha256 RETAINED_RECORD_SHA256 --format json
task publication:project:catalog WORKSPACE=/private/static-project RECORD_SHA256=RETAINED_RECORD_SHA256
```

`flow.static-publication-artifact-catalog/v1` retains the existing full branch
status, missing required targets and exact file pins/purposes from the native
artifact owner. Each output is associated with its selected branch, declared
locale and required/optional target intent. A missing target never becomes an
output merely because it was requested. Incomplete branches supply no orphan
files. Changed native membership or branch status refuses the catalog.

This private read-only view performs no conversion, translation, review,
provider invocation, output copying or upload. Locale labels are declarations,
not proof that translation/content was reviewed. Retained native acceptance
does not refresh release qualification or grant publication authority. Use
the exact pins for later explicit packaging and keep all review gates.

Two focused source regressions are authored, uncompiled/unrun. Tests/builds,
dependencies/validators, native/provider/media/CLI and CI remain deferred.
#10 retains its full exhaustive/localization/recovery and actual released
multi-holon acceptance; Flow #88 stays last and untouched.
