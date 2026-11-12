"""Authored source cases; execution is deferred with the suite's final pass."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location("flow_release_package",ROOT/"scripts/release/package.py")
package=importlib.util.module_from_spec(spec)
spec.loader.exec_module(package)


class PackageReaderTests(unittest.TestCase):
    def test_candidate_manifest_cannot_assert_publication_or_qualification(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory).resolve()
            value={"schema_version":package.SCHEMA,"repository":"egohygiene/flow","qualification":"qualified","publication":"published"}
            data=package.json_bytes(value)
            (root/package.MANIFEST).write_bytes(data)
            with self.assertRaisesRegex(ValueError,"cannot assert"):
                package.verify(root,package.sha(data))

    def test_untrusted_asset_path_is_refused_before_payload_read(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory).resolve()
            value={"schema_version":package.SCHEMA,"repository":"egohygiene/flow","version":"0.1.0","source_revision":"a"*40,
                "target":"x86_64-unknown-linux-gnu","registry_channel":"unavailable","qualification":"not_established_by_preparation","publication":"not_performed",
                "assets":[{"name":"../escape","sha256":"b"*64,"byte_count":1,"kind":"cli-binary"}]}
            data=package.json_bytes(value)
            (root/package.MANIFEST).write_bytes(data)
            with self.assertRaisesRegex(ValueError,"unsafe package asset"):
                package.verify(root,package.sha(data))
