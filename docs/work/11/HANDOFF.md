# Flow #11 dependency-map documentation handoff

## Checkpoint and authority

- Parent: [Flow #11](https://github.com/egohygiene/flow/issues/11).
- Role: documentation and bounded live reconciliation.
- Request: capture the existing suite tree durably and update its observed state.
- Strategy: worker-strategy v0.1.0 at
  `db5f4bd339f9266758c74c3435d47c8005f89a8e`, implementation-first scheduling.
- Scope: documentation only; the proposed Renderflow #422 implementation remains
  unstarted. No sibling repository or tracker state is changed by this candidate.
- Stop: reviewable draft PR, then return to the maintainer; no merge or CI dispatch.

## Base and candidate

- Observed at: 2026-10-02T15:16:39.663Z.
- Verified Flow base: `2102c3d48998d6fa00da3751e2ae409dfa8b6d6c` (`main`).
- Candidate branch: `docs/flow-suite-dependency-map`.
- Candidate commit and PR: supplied by the containing GitHub branch/PR, not
  guessed in this file. A commit cannot contain its own eventual SHA.
- The [map snapshot](../../roadmaps/flow-suite-dependency-map.md) records all four
  observed heads, all open issues/PRs and the source/evidence limitations.

## Outcome and acceptance

- Preserve the original public Mermaid block byte-for-byte in
  `docs/roadmaps/archive/flow-suite-dependencies-2026-09-27.mmd`.
- Provide a maintained, readable current map in
  `docs/roadmaps/flow-suite-dependency-map.md`, covering all 80 original nodes and
  seven newly open checkpoints, with explicit relationship corrections.
- Keep authored/merged work, passing evidence, failures and release state
  separate. Preserve open parent criteria and known pending work.
- Link the map and this handoff from root navigation, and document future updates.

## Material changes and decisions

The map belongs in Flow because it coordinates independent holons. Domain
acceptance stays in each provider's issues. The original issue comment remains
provenance; the archived source is historical and the new views are maintained.
Root ROADMAP receives navigation and a metadata patch, not a new issue queue.
No architecture ownership, runtime interface, workflow or product behavior changes.

The original upload could not be retrieved, so image identity/preservation is
unverified. Source reconstruction uses the already public issue comment only.

## Checks and deferred work

One-off authoring inspection, `python .inspection/check_docs.py`: exit 0.
This temporary inspection is not a committed repository command. It checked:

- The archived Mermaid block exactly matches the recovered source and its
  recorded SHA-256 digest.
- All 80 original issues are retained; the current inventory contains 87 issues,
  all 76 observed open issues, and seven observed open PRs.
- All 125 relationships reference known issues; required edges have no cycle.
- Nine Mermaid blocks use the inspected syntax subset and resolve their nodes.
- All 43 relative links resolve against the observed base tree and these files.
- Markdown fences/headings, added-line whitespace, and root navigation diffs.

The inspected candidate is based on the Flow SHA above. Existing ROADMAP hard
breaks are preserved. These source checks are not a Mermaid parser, rendered
layout review, or the repository's full documentation validation suite.

Product test execution, Rust builds/lint, full local CI reproduction and fleet
audit are not applicable to this docs-only change. Their existing product
obligations are preserved in the map rather than reported as executed here.
Mermaid rendering is deferred because no local renderer is available; static
source checks do not prove layout or GitHub rendering. No renderer installation
or remote CI dispatch is included. The docs commit requests `[skip ci]` because
draft PRs otherwise trigger Flow's existing CI; the request is not a passing result.

## Reconciliation and next action

Known parallel work: Optiflow PR #113 includes merged-into-branch PR #114;
Aniflow draft PR #78 contains unqualified corpus work. Five Optiflow dependency
PRs remain open. The maps do not authorize their merge or recreate their code.

Before continuing, recheck Flow main and this draft PR against the recorded base.
Review the documentation and any changed upstream evidence. After maintainer
direction, either revise the map or start one separately bounded Renderflow #422
checkpoint, beginning with target deduplication in `planning.rs` and synthetic
regressions. Record its own acceptance, focused-check budget and deferred
tests/docs/local CI/audit before implementation; stop at its draft PR.

Current Flow main has no root CONTINUITY.md. This handoff is linked from ROADMAP;
adding a new continuity entrypoint is not silently bundled into this capture.
