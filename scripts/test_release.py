"""Release metadata and artifact regression tests."""
# this_file: scripts/test_release.py

import io
import os
import re
import shlex
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path

import release


class ReleaseTests(unittest.TestCase):
    def test_tag_assets_do_not_depend_on_registry_publication(self):
        workflow = (release.ROOT / ".github/workflows/release.yml").read_text()
        for job in ("publish-crates", "publish-pypi"):
            # Inspect the job condition independently of nested step conditions.
            condition = re.search(
                rf"  {job}:\n(?:    [^\n]*\n)*?    if: ([^\n]+)", workflow
            )
            self.assertIsNotNone(condition, job)
            self.assertEqual(
                condition[1],
                "github.event_name == 'workflow_dispatch' && !inputs.build_only",
            )
        release_section = workflow.split("  github-release:\n", 1)[1]
        self.assertIn("needs: [validate, binaries, wheels, sdist]", release_section)
        self.assertNotIn("publish-pypi", release_section)

    def test_binary_private_paths_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "typf.exe"
            path.write_bytes(b"MZpublic executable")
            release.audit_binary(path)
            path.write_bytes(b"MZ C:\\Users\\runneradmin\\.cargo\\source.rs")
            with self.assertRaisesRegex(ValueError, "Private build path"):
                release.audit_binary(path)

    def make_sdist(self, path, metadata):
        with tarfile.open(path, "w:gz") as archive:
            for name, data in metadata:
                payload = data.encode()
                member = tarfile.TarInfo(name)
                member.size = len(payload)
                archive.addfile(member, io.BytesIO(payload))

    def test_sdist_release_identity(self):
        valid = "Metadata-Version: 2.1\nName: typf\nVersion: 5.1.0\nRequires-Python: >=3.11\n"
        cases = [
            ("valid", [("typf-5.1.0/PKG-INFO", valid)], False),
            (
                "old version",
                [("typf-5.1.0/PKG-INFO", valid.replace("5.1.0", "5.0.16"))],
                True,
            ),
            (
                "other project",
                [
                    (
                        "typf-5.1.0/PKG-INFO",
                        valid.replace("Name: typf", "Name: unrelated"),
                    )
                ],
                True,
            ),
            (
                "old Python",
                [("typf-5.1.0/PKG-INFO", valid.replace(">=3.11", ">=3.9"))],
                True,
            ),
            ("missing metadata", [("typf-5.1.0/README.md", "hello")], True),
            ("nested metadata", [("typf-5.1.0/nested/PKG-INFO", valid)], True),
            (
                "ambiguous metadata",
                [("typf-5.1.0/PKG-INFO", valid), ("other/PKG-INFO", valid)],
                True,
            ),
        ]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "typf-5.1.0.tar.gz"
            for name, records, rejected in cases:
                with self.subTest(name=name):
                    self.make_sdist(path, records)
                    if rejected:
                        with self.assertRaisesRegex(ValueError, "[Mm]etadata|PKG-INFO"):
                            release.audit_archive(path)
                    else:
                        release.audit_archive(path)

    @unittest.skipIf(sys.platform == "win32", "Shell publisher is tested on Unix")
    def test_real_publish_error_stops_release(self):
        with tempfile.TemporaryDirectory() as directory:
            cargo = Path(directory) / "cargo"
            cargo.write_text(
                '#!/bin/sh\nif [ "$1" = publish ]; then exit 91; fi\n'
                f'exec {shlex.quote(shutil.which("cargo"))} "$@"\n'
            )
            cargo.chmod(0o755)
            result = subprocess.run(
                ["bash", "scripts/publish.sh", "--crates"],
                cwd=release.ROOT,
                env={
                    **os.environ,
                    "PATH": f"{directory}{os.pathsep}{os.environ['PATH']}",
                    "PYTHON": sys.executable,
                },
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(result.returncode, 91, result.stdout + result.stderr)

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
