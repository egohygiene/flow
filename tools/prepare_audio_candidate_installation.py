#!/usr/bin/env python3
"""Select the existing installer with a closed six-row audio observation.

No acquisition, new installer, package rewrite or provider execution is added.
"""
import argparse
from pathlib import Path

from check_native_publication import pin, put
from check_released_read_only import OWNER_IO, canonical, document, require
from installed_audio_candidate_subjects import InstalledAudioCandidate, extra_rows
from prepare_music_candidate_installation import prepare


def run(config, output):
    rows = extra_rows(config["provider_rows"]["aniflow"])
    # prepare() validates the exact projection before any installation command.
    subject, original_report = prepare(config, output,
        candidate_type=InstalledAudioCandidate, extra_provider_rows=rows)
    report = {**original_report, "schema_version": "flow.audio-candidate-installation-observation/v1",
        "physical_tool_count": 4, "installed_cohort_rows": 6,
        "aniflow_profiles": [r["profile"] for r in subject.cohort["rows"] if r["tool"] == "aniflow"],
        "current_source_context": subject.current_context,
        "historical_v2_projection_preserved": True,
        "scope": "four physical tools, six exact candidate profile rows, separately pinned current-context observation; no new Flow package"}
    put(output / "audio-installation-report.json", report)
    return subject, report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inputs", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    _, report = run(document(OWNER_IO.regular(args.inputs, 1048576)), args.output)
    print(canonical(report).decode())
