#!/usr/bin/env python3
"""Negative completion observers; synthetic records do not qualify a provider."""
import copy
from pathlib import Path
import tempfile
import unittest

import check_installed_music_video as proof


class Cases(unittest.TestCase):
    def setUp(self):
        self.temporary = self.enterContext(tempfile.TemporaryDirectory())
        self.root = Path(self.temporary) / "run"
        native = self.root / "native/recipe"; native.mkdir(parents=True)
        self.source = Path(self.temporary) / "source.mp4"; self.source.write_bytes(b"synthetic, never decoded")
        self.provider = Path(self.temporary) / "bin/aniflow"
        payload={"source_video":str(self.source),"provider_executable":str(self.provider)}
        intent={"schema":"aniflow.source-music-video-intent/v1","payload":payload,"intent_sha256":proof.sha(proof.canonical(payload))}
        artifacts = []
        for name in ["master.mkv", "delivery.mp4"]:
            path = native / name; path.write_bytes(name.encode()); artifacts.append(proof.pin(path))
        declared = [{"variant": variant, "artifact_id": variant, "locator": artifact["path"]["value"],
            "sha256": artifact["sha256"], "byte_count": artifact["byte_count"]}
            for variant, artifact in zip(["clean_archival_master", "destination_copy"], artifacts)]
        self.evidence = {"schema_version": "flow.aniflow-domain-acceptance/v1", "source_pins": [proof.pin(self.source)],
            "native_plan":{"intent":intent},
            "native_status": {"complete": True, "intent_sha256": intent['intent_sha256'],"run_directory":str(native)}, "native_completion": {
                "schema": "aniflow.final-video-handoff/v1", "publication_authority": "none", "proof_state": "not_asserted", "artifacts": declared}}
        self.receipt = {"schema_version": "flow.native-receipt/v1", "state": "complete", "native_run": proof.native_path(native),
            "artifacts": artifacts, "release_qualified": False, "creative_approval_inferred": False, "publication_authority": False,
            "native_intent_sha256": intent['intent_sha256'], "acceptance_file": proof.native_path(self.root / "domain.json")}
        (native/'intent.json').write_bytes(proof.canonical(intent)+b'\n')
        self.emit()

    def emit(self):
        raw = proof.canonical(self.evidence) + b"\n"
        (self.root / "domain.json").write_bytes(raw); self.receipt["acceptance_sha256"] = proof.sha(raw)
        (self.root / "accepted.json").write_bytes(proof.canonical(self.receipt) + b"\n")

    def test_receipt_requires_distinct_actual_roles_and_current_exact_bytes(self):
        proof.accepted(self.receipt, self.root, self.source,self.provider)
        self.evidence["native_completion"]["artifacts"][1]["variant"] = "clean_archival_master"; self.emit()
        with self.assertRaisesRegex(ValueError, "distinct accepted final roles"): proof.accepted(self.receipt, self.root, self.source,self.provider)

    def test_current_payload_and_source_identity_are_independent(self):
        path = Path(self.receipt["artifacts"][0]["path"]["value"]); path.write_bytes(b"different bytes")
        with self.assertRaisesRegex(ValueError, "byte join"): proof.accepted(self.receipt, self.root, self.source,self.provider)
        path.write_bytes(b"master.mkv"); self.source.write_bytes(b"different source")
        with self.assertRaisesRegex(ValueError, "source/status"): proof.accepted(self.receipt, self.root, self.source,self.provider)

    def test_outside_acceptance_and_wrong_accepted_receipt_refuse(self):
        original = copy.deepcopy(self.receipt)
        self.receipt["acceptance_file"] = proof.native_path(Path(self.temporary) / "outside.json")
        with self.assertRaisesRegex(ValueError, "escaped"): proof.accepted(self.receipt, self.root, self.source,self.provider)
        self.receipt = original
        (self.root / "accepted.json").write_bytes(b"{}\n")
        with self.assertRaisesRegex(ValueError, "validated receipt"): proof.accepted(self.receipt, self.root, self.source,self.provider)

    def test_same_completion_cannot_claim_a_different_installed_provider(self):
        with self.assertRaisesRegex(ValueError,"exact installed provider"):
            proof.accepted(self.receipt,self.root,self.source,self.provider.with_name('other'))

if __name__ == "__main__": unittest.main()
