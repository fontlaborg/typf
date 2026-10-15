"""Release metadata and artifact regression tests."""
# this_file: scripts/test_release.py

import tempfile
import unittest
import zipfile
from pathlib import Path

import release


class ReleaseTests(unittest.TestCase):
    def test_workspace_contract_and_dependency_order(self):
        metadata = release.validate()
        ordered = release.publish_order(metadata)
        self.assertEqual(len(ordered), len(metadata["workspace_members"]))
        positions = {p["name"]: i for i, p in enumerate(ordered)}
        for package in ordered:
            for dep in package["dependencies"]:
                if dep["name"] in positions:
                    self.assertLess(positions[dep["name"]], positions[package["name"]])

    def test_private_payload_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory) / "typf-5.1.0-cp311-abi3-test.whl"
            with zipfile.ZipFile(archive, "w") as wheel:
                wheel.writestr("typf/.env", "secret")
            with self.assertRaisesRegex(ValueError, "Private"):
                release.audit_archive(archive)

    def test_wrong_release_version_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "Expected"):
            release.validate("0.0.0")


if __name__ == "__main__":
    unittest.main()
