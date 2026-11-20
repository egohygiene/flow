#!/usr/bin/env python3
"""Closed six-row audio candidate observation; historical v2 stays unchanged.

Four physical tools are installed. Three explicit Aniflow profile rows share
the same captured executable and descriptor. This is supplied local evidence,
without publisher authentication, native qualification or execution authority.
"""
import copy
from pathlib import Path
import re

import installed_music_candidate_subjects as music

cli = music.cli
TARGET = music.TARGET
QC_ROW = "aniflow.final-video-audio-qc.source-candidate." + TARGET
POLICY_ROW = "aniflow.audio-policy-check.source-candidate." + TARGET
CONTEXT_REVISION = "a6f01b9b0bfdf985027f3bcd8e0d729c20e1691f"
CONTEXT_TREE = "de1a2b1938a91409e7bbf54b518e7dfc7777419f"
CONTEXT_SCOPE = "supplied local tracked mode/type/blob comparison and literal-include/Cargo/build scan; not source authentication, a new executable or a package reseal"
CONTEXT_FIELDS = {"schema_version", "actual_build_revision", "actual_build_tree", "selected_revision", "selected_tree",
    "permitted_nonruntime_prefixes", "permitted_nonruntime_paths", "changed_paths", "changed_compilation_inputs",
    "runtime_inputs_equal", "changed_nonruntime_files_embedded", "compilation_input_count",
    "compilation_input_inventory_sha256", "compilation_input_inventory", "include_reference_count",
    "include_reference_inventory_sha256", "include_references", "source_authentication_performed", "scope"}
FIELDS = music.FIELDS | {"aniflow_final_qc_row_id", "aniflow_policy_row_id",
    "flow_context_equivalence", "flow_context_equivalence_sha256"}
PROFILE_ROWS = [(QC_ROW, "final-video-audio-qc-v1", "exact accepted final video",
    "fresh supporting QC record"), (POLICY_ROW, "audio-policy-check-v1",
    "exact final source and current QC", "read-only selected-policy assessment")]


def selection(value):
    cli.require(type(value) is dict and set(value) == FIELDS
        and all(type(v) is str and v for v in value.values())
        and value["schema_version"] == "flow.installed-candidate-selection/v3"
        and value["aniflow_final_qc_row_id"] == QC_ROW
        and value["aniflow_policy_row_id"] == POLICY_ROW
        and re.fullmatch(r"[0-9a-f]{64}", value["flow_context_equivalence_sha256"]) is not None,
        "closed explicit six-row audio candidate selection required")
    previous = {key: value[key] for key in music.FIELDS}
    previous["schema_version"] = "flow.installed-candidate-selection/v2"
    music.selection(previous)
    return dict(value)


def extra_rows(original):
    result = []
    for row_id, profile, input_scope, output_scope in PROFILE_ROWS:
        row = copy.deepcopy(original)
        row.update(id=row_id, profile=profile, input=input_scope, output=output_scope,
            next_action="explicit local audio candidate only; no release, meter, human or upload qualification")
        result.append(row)
    return result


def provider_projection(supplement, installed):
    cli.require(type(supplement) is dict and type(installed) is dict
        and len(supplement.get("rows", [])) == 5 and len(installed.get("rows", [])) == 6,
        "exact five provider rows and six installed rows required")
    rows = supplement["rows"]
    cli.require([r["tool"] for r in rows] == ["aniflow", "renderflow", "optiflow", "aniflow", "aniflow"]
        and [r["id"] for r in rows[-2:]] == [QC_ROW, POLICY_ROW]
        and rows[-2:] == extra_rows(rows[0])
        and len({r["id"] for r in installed["rows"]}) == 6
        and len({r["profile"] for r in rows if r["tool"] == "aniflow"}) == 3
        and installed["rows"][1:] == rows
        and supplement["retained_rows"] == [r["id"] for r in rows]
        and installed["retained_rows"] == [r["id"] for r in installed["rows"]],
        "audio profile rows changed, duplicated or selected different candidate bytes/contracts")
    # Join the original three-provider projection without changing either file.
    base_supplement = {**supplement, "rows": rows[:3], "retained_rows": [r["id"] for r in rows[:3]]}
    base_installed = {**installed, "rows": installed["rows"][:4],
        "retained_rows": [r["id"] for r in installed["rows"][:4]]}
    music.provider_projection(base_supplement, base_installed)


def context(value, subject):
    reason = "explicit current audio-candidate context differs from the measured Flow inputs"
    cli.require(type(value) is dict and set(value) == CONTEXT_FIELDS, reason)
    cli.require(value["schema_version"] == "flow.audio-candidate-build-context/v1"
        and value["actual_build_revision"] == subject.selection["flow_source_revision"]
        and value["actual_build_tree"] == subject.selection["flow_source_tree"]
        and value["selected_revision"] == CONTEXT_REVISION and value["selected_tree"] == CONTEXT_TREE
        and value["permitted_nonruntime_prefixes"] == ["docs/", "tools/"]
        and value["permitted_nonruntime_paths"] == ["Taskfile.yml", "config/local-ci.json"]
        and value["runtime_inputs_equal"] is True and value["changed_compilation_inputs"] == []
        and value["changed_nonruntime_files_embedded"] is False
        and value["source_authentication_performed"] is False and value["scope"] == CONTEXT_SCOPE
        and type(value["compilation_input_count"]) is int and value["compilation_input_count"] > 0
        and type(value["compilation_input_inventory_sha256"]) is str
        and re.fullmatch(r"[0-9a-f]{64}", value["compilation_input_inventory_sha256"]) is not None
        and type(value["include_reference_inventory_sha256"]) is str
        and re.fullmatch(r"[0-9a-f]{64}", value["include_reference_inventory_sha256"]) is not None
        and type(value["include_reference_count"]) is int and value["include_reference_count"] > 0
        and type(value["compilation_input_inventory"]) is dict
        and value["compilation_input_count"] == len(value["compilation_input_inventory"])
        and cli.sha(cli.canonical(value["compilation_input_inventory"])) == value["compilation_input_inventory_sha256"]
        and type(value["include_references"]) is list
        and value["include_reference_count"] == len(value["include_references"])
        and cli.sha(cli.canonical(value["include_references"])) == value["include_reference_inventory_sha256"]
        and type(value["changed_paths"]) is list and all(type(p) is str for p in value["changed_paths"])
        and value["changed_paths"] == sorted(set(value["changed_paths"]))
        and all(type(p) is str and ".." not in Path(p).parts
            and (p.startswith(("docs/", "tools/")) or p in ["Taskfile.yml", "config/local-ci.json"])
            for p in value["changed_paths"]),
        reason)
    def tracked(path, item):
        return (type(path) is str and path and not path.startswith("/") and ".." not in Path(path).parts
            and type(item) is dict and set(item) == {"mode", "type", "sha"}
            and item["mode"] in ["100644", "100755", "120000"] and item["type"] == "blob"
            and type(item["sha"]) is str
            and re.fullmatch(r"[0-9a-f]{40}", item["sha"]) is not None)
    cli.require(all(tracked(path, item) and not path.startswith(("docs/", "tools/"))
        and path not in ["Taskfile.yml", "config/local-ci.json"]
        for path, item in value["compilation_input_inventory"].items()), reason)
    cli.require(all(type(item) is dict and set(item) == {"owner", "kind", "target", "object"}
        and item["kind"] in ["include!", "include_str!", "include_bytes!"]
        and type(item["owner"]) is str and item["owner"] in value["compilation_input_inventory"]
        and tracked(item["target"], item["object"]) and item["target"] not in value["changed_paths"]
        for item in value["include_references"]), reason)
    return {"actual_build_revision": value["actual_build_revision"],
        "actual_build_tree": value["actual_build_tree"], "selected_revision": value["selected_revision"],
        "selected_tree": value["selected_tree"], "runtime_inputs_equal": True,
        "scope": "separate complete local Git input comparison; not a rebuild, package reseal or source authentication"}


class InstalledAudioCandidate(music.InstalledMusicCandidate):
    SELECTION_SCHEMA = "flow.installed-candidate-selection/v3"
    selection_parser = staticmethod(selection)
    projection_checker = staticmethod(provider_projection)

    def reobserve(self):
        super().reobserve()
        value = music.previous.read(Path(self.selection["flow_context_equivalence"]),
            self.selection["flow_context_equivalence_sha256"], 131072)
        self.current_context = context(value, self)
        self.current_context["observation_sha256"] = self.selection["flow_context_equivalence_sha256"]
        for _, profile, _, _ in PROFILE_ROWS:
            description = music.previous.read(self.aniflow_descriptor)
            cli.require(any(p["id"] == profile and p["source_mutation"] is False for p in description["profiles"]),
                "actual installed descriptor lacks selected audio profile")
        return self
