#!/usr/bin/env python3
"""Closed observer controls; no provider, media or installer is executed."""
import copy
from pathlib import Path
import re
import tempfile
import unittest

import installed_audio_candidate_subjects as audio
import installed_music_candidate_subjects as music
import test_installed_music_candidate_subjects as previous


class Cases(unittest.TestCase):
    def setUp(self):
        self.temp = self.enterContext(tempfile.TemporaryDirectory())
        self.fixture = previous.Fixture(Path(self.temp))
        self.installed = copy.deepcopy(self.fixture.cohort)
        self.installed["rows"].extend(audio.extra_rows(self.installed["rows"][1]))
        self.installed["retained_rows"] = [r["id"] for r in self.installed["rows"]]
        self.supplement = {**copy.deepcopy(self.installed), "rows": self.installed["rows"][1:],
            "retained_rows": [r["id"] for r in self.installed["rows"][1:]]}

    def test_six_profile_rows_are_explicit_and_legacy_v2_stays_closed(self):
        audio.provider_projection(self.supplement, self.installed)
        with self.assertRaisesRegex(ValueError, "^exact three supplemented providers and one derived Flow row required$"):
            music.provider_projection(self.supplement, self.installed)
        for row in self.supplement["rows"][-2:]:
            self.assertEqual(row["artifact"], self.supplement["rows"][0]["artifact"])
            self.assertEqual(row["contracts"], self.supplement["rows"][0]["contracts"])

    def test_conflicting_alias_or_duplicate_profile_cannot_admit_a_different_provider(self):
        mutations = [lambda r: r.update(profile="music-video-source-v1"),
            lambda r: r.update(id=audio.QC_ROW), lambda r: r["artifact"].update(sha256="0" * 64),
            lambda r: r["artifact"].update(source_revision="0" * 40),
            lambda r: r["contracts"][0].update(sha256="0" * 64),
            lambda r: r["effects"].append("replace_originals")]
        for mutate in mutations:
            installed = copy.deepcopy(self.installed); mutate(installed["rows"][-1])
            installed["retained_rows"] = [r["id"] for r in installed["rows"]]
            supplement = {**copy.deepcopy(installed), "rows": installed["rows"][1:],
                "retained_rows": installed["retained_rows"][1:]}
            with self.subTest(mutation=mutations.index(mutate)):
                with self.assertRaisesRegex(ValueError, "^" + re.escape("audio profile rows changed, duplicated or selected different candidate bytes/contracts") + "$"):
                    audio.provider_projection(supplement, installed)

    def test_v3_selection_cannot_be_v2_or_an_authority_cache(self):
        value = {**self.fixture.value, "schema_version": "flow.installed-candidate-selection/v3",
            "aniflow_final_qc_row_id": audio.QC_ROW, "aniflow_policy_row_id": audio.POLICY_ROW,
            "flow_context_equivalence": "/supplied/context.json", "flow_context_equivalence_sha256": "a" * 64}
        audio.selection(value)
        with self.assertRaisesRegex(ValueError, "^closed four-tool candidate selection required$"):
            music.selection(value)
        for key, replacement in [("schema_version", "flow.installed-candidate-selection/v2"),
                ("aniflow_policy_row_id", audio.QC_ROW), ("verified", "true")]:
            with self.assertRaisesRegex(ValueError, "^closed explicit six-row audio candidate selection required$"):
                audio.selection({**value, key: replacement})

    def test_current_context_cannot_hide_changed_compilation_or_included_inputs(self):
        class Subject:
            selection = {"flow_source_revision": "a" * 40, "flow_source_tree": "b" * 40}
        value = {"schema_version": "flow.audio-candidate-build-context/v1",
            "actual_build_revision": "a" * 40, "actual_build_tree": "b" * 40,
            "selected_revision": audio.CONTEXT_REVISION, "selected_tree": audio.CONTEXT_TREE,
            "permitted_nonruntime_prefixes": ["docs/", "tools/"],
            "permitted_nonruntime_paths": ["Taskfile.yml", "config/local-ci.json"],
            "runtime_inputs_equal": True, "changed_compilation_inputs": [],
            "changed_nonruntime_files_embedded": False, "compilation_input_count": 1,
            "compilation_input_inventory": {"src/main.rs": {"mode": "100644", "type": "blob", "sha": "e" * 40}},
            "compilation_input_inventory_sha256": "c" * 64, "include_reference_count": 1,
            "include_references": [{"owner": "src/main.rs", "kind": "include_str!", "target": "schemas/example.json",
                "object": {"mode": "100644", "type": "blob", "sha": "f" * 40}}],
            "include_reference_inventory_sha256": "d" * 64,
            "source_authentication_performed": False, "scope": audio.CONTEXT_SCOPE,
            "changed_paths": sorted(["docs/proof.md", "tools/observer.py", "Taskfile.yml", "config/local-ci.json"])}
        value["compilation_input_inventory_sha256"] = audio.cli.sha(audio.cli.canonical(value["compilation_input_inventory"]))
        value["include_reference_inventory_sha256"] = audio.cli.sha(audio.cli.canonical(value["include_references"]))
        reason = "^explicit current audio-candidate context differs from the measured Flow inputs$"
        audio.context(value, Subject())
        for path in ["src/native/audio_policy.rs", "Cargo.toml", "schemas/native-request-v1.schema.json", "config/other.json"]:
            wrong = copy.deepcopy(value); wrong["changed_paths"].append(path)
            wrong["changed_paths"].sort()
            with self.assertRaisesRegex(ValueError, reason): audio.context(wrong, Subject())
        for key, replacement in [("runtime_inputs_equal", False),
                ("changed_compilation_inputs", ["src/lib.rs"]),
                ("changed_nonruntime_files_embedded", True), ("source_authentication_performed", True),
                ("include_reference_count", 2), ("compilation_input_inventory_sha256", "0" * 64), ("verified", True)]:
            with self.assertRaisesRegex(ValueError, reason): audio.context({**value, key: replacement}, Subject())


if __name__ == "__main__": unittest.main()
