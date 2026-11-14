#!/usr/bin/env python3
"""Independent observer controls; these do not execute providers or qualify media."""
import copy
import io
import tempfile
import unittest
import zipfile
from pathlib import Path

import check_native_publication as pilot


class ObserverCases(unittest.TestCase):
    def test_epub_spine_checks_exact_original_bytes_and_order(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); sources = [root/'one.png', root/'two.png']
            for path, data in zip(sources, [b'first synthetic image', b'second synthetic image']): path.write_bytes(data)
            def fixture(reverse=False, changed=False):
                stream = io.BytesIO()
                with zipfile.ZipFile(stream, 'w') as archive:
                    archive.writestr('mimetype', 'application/epub+zip')
                    archive.writestr('META-INF/container.xml', '<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfile full-path="book.opf"/></container>')
                    archive.writestr('book.opf', '<package xmlns="http://www.idpf.org/2007/opf"><manifest><item id="a" href="one.xhtml"/><item id="b" href="two.xhtml"/></manifest><spine><itemref idref="'+('b' if reverse else 'a')+'"/><itemref idref="'+('a' if reverse else 'b')+'"/></spine></package>')
                    for name, source in zip(['one','two'], sources):
                        archive.writestr(name+'.xhtml', '<html xmlns="http://www.w3.org/1999/xhtml"><img src="'+name+'.png"/></html>')
                        archive.writestr(name+'.png', source.read_bytes() + (b'changed' if changed and name=='two' else b''))
                return stream.getvalue()
            self.assertEqual(pilot.epub_source_images(fixture(), sources), [pilot.pin(path)['sha256'] for path in sources])
            for value in [fixture(reverse=True), fixture(changed=True)]:
                with self.assertRaisesRegex(ValueError, 'bytes/order'): pilot.epub_source_images(value, sources)
    def test_pdf_geometry_rejects_wrong_height_shifted_origin_and_nonfinite_coordinates(self):
        box = [0.0, 0.0, 283.46456692913387, 283.46456692913387]
        pilot.pdf_geometry({"pages": 2, "boxes": [box, box]})
        for changed in [[0, 0, box[2], 200], [5, 0, box[2], box[3]], [0, -5, box[2], box[3]],
                        [0, 0, box[2], float('nan')], [False, 0, box[2], box[3]]]:
            with self.assertRaises(ValueError): pilot.pdf_geometry({"pages": 2, "boxes": [box, changed]})
    def build(self):
        return {"schema_version": "flow.local-build-observation/v1", "actual_build_revision": "a" * 40,
            "actual_build_tree": "b" * 40, "binary": {"sha256": "c" * 64, "bytes": 100},
            "profile": "dev", "build_command": ["cargo", "+1.85.0", "build", "--locked", "--offline", "--bin", "flow"]}

    def test_public_build_discards_private_and_remote_equivalence_claims(self):
        value = self.build(); value.update(private_path="/private/secret", equivalent_remote_revision="not a fact")
        result = pilot.public_candidate_build(value)
        self.assertNotIn("private_path", result); self.assertNotIn("equivalent_remote_revision", result)

    def test_malformed_or_boolean_build_identity_refuses(self):
        for field, value in [("actual_build_revision", "short"), ("actual_build_tree", 5), ("profile", "release")]:
            identity = self.build(); identity[field] = value
            with self.assertRaises(ValueError): pilot.public_candidate_build(identity)
        identity = self.build(); identity["binary"]["bytes"] = True
        with self.assertRaises(ValueError): pilot.public_candidate_build(identity)

    def test_same_exit_unrelated_refusal_does_not_pass_named_control(self):
        expected = ("stale_plan", "native pinned bytes changed; create a new reviewed plan", None)
        value = {"result": None, "diagnostics": [{"code": "stale_plan", "message": expected[1], "recovery_directory": None}]}
        pilot.refusal(value, expected)
        for changed in [{"code": "invalid_input"}, {"message": "unrelated failure"}, {"recovery_directory": pilot.native_path(Path('/other'))}]:
            actual = copy.deepcopy(value); actual["diagnostics"][0].update(changed)
            with self.assertRaises(ValueError): pilot.refusal(actual, expected)

    def test_candidate_cohort_has_real_source_url_and_no_release_claim(self):
        value = pilot.cohort({"sha256": "a" * 64, "byte_count": 10}, {"sha256": "b" * 64})
        row = value["rows"][0]
        self.assertIn('/commit/', row["artifact"]["release_url"])
        self.assertNotIn('/releases/download/', row["artifact"]["release_url"])
        self.assertIn('unestablished', row["next_action"])

    def test_native_collection_config_preserves_order_and_relative_member_scope(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); source = root / "source 作品"; source.mkdir()
            files = [source / "second.png", source / "first.png"]
            for p in files: p.write_bytes(pilot.png((10, 20, 30)))
            value = pilot.configuration(files, root / "run", "epub", Path('/prepared/img2pdf'))
            self.assertEqual([row["path"] for row in value["sources"][:2]], ['source 作品/second.png', 'source 作品/first.png'])
            self.assertEqual([row["path"] for row in value["publication"]["artwork"]], ['source 作品/second.png', 'source 作品/first.png'])
            self.assertEqual(value["sources"][2]["members"], ['source.page001', 'source.page002'])
            self.assertEqual(value["execution"]["requirements"], {"offline": True, "local_only": True})

    def test_acceptance_refuses_outside_receipt_and_unrelated_same_bytes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); run = root / "run"; native = run / "native/output"; native.mkdir(parents=True)
            output = native / "interior.pdf"; output.write_bytes(b"synthetic fixture, no PDF qualification")
            receipt = {"schema_version": "flow.native-receipt/v1", "state": "complete", "native_run": pilot.native_path(native),
                "artifacts": [pilot.pin(output)], "release_qualified": False, "creative_approval_inferred": False,
                "publication_authority": False, "acceptance_file": pilot.native_path(root / "outside.json"), "acceptance_sha256": "a" * 64}
            (root / "outside.json").write_bytes(b'{}')
            with self.assertRaisesRegex(ValueError, "escaped run"): pilot.accepted(receipt, run, "pdf", [])
            accepted = run / "domain.json"; accepted.write_bytes(b'{}')
            receipt["acceptance_file"] = pilot.native_path(accepted)
            with self.assertRaisesRegex(ValueError, "acceptance bytes differ"): pilot.accepted(receipt, run, "pdf", [])
            receipt["release_qualified"] = True
            with self.assertRaisesRegex(ValueError, "unqualified"): pilot.accepted(receipt, run, "pdf", [])


if __name__ == "__main__":
    unittest.main()
