#!/usr/bin/env python3
"""Validate the release contract, audit payloads, and order crate publication."""
# this_file: scripts/release.py

import argparse
import email
import json
import re
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PRIVATE_NAMES = {
    ".env",
    ".envrc",
    ".omc",
    ".superpowers",
    ".codegraph",
    "CLAUDE.md",
    "AGENTS.md",
    "WORK.md",
    "REVIEW.md",
    "TASKS.md",
}


def validate(expected=None):
    """Fail before building if versions or publication dependencies diverge."""
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())
    version = workspace["workspace"]["package"]["version"]
    if expected and version != expected.removeprefix("v"):
        raise ValueError(f"Expected {expected}; workspace is {version}")
    metadata = json.loads(
        subprocess.check_output(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=ROOT
        )
    )
    for package in metadata["packages"]:
        if package["id"] not in metadata["workspace_members"]:
            continue
        if package["version"] != version:
            raise ValueError(f"Version mismatch: {package['name']}")
        for dep in package["dependencies"]:
            if dep.get("path") and (
                dep["req"] == "*" or "external" in Path(dep["path"]).parts
            ):
                raise ValueError(
                    f"Unpublishable dependency: {package['name']} -> {dep['name']}"
                )
            if (
                dep["name"].startswith("typf")
                and dep.get("path")
                and dep["req"] != f"^{version}"
            ):
                raise ValueError(f"Dependency version mismatch: {dep['name']}")
    python = tomllib.loads((ROOT / "bindings/py/pyproject.toml").read_text())
    if (
        python["project"]["name"] != "typf"
        or python["project"]["requires-python"] != ">=3.11"
    ):
        raise ValueError("Python distribution must be typf for Python >=3.11")
    binding = tomllib.loads((ROOT / "bindings/py/Cargo.toml").read_text())
    if "abi3-py311" not in binding["dependencies"]["pyo3"]["features"]:
        raise ValueError("Python wheels must use abi3-py311")
    root_python = tomllib.loads((ROOT / "pyproject.toml").read_text())
    for key in (
        "name",
        "dynamic",
        "requires-python",
        "classifiers",
        "dependencies",
        "scripts",
    ):
        if root_python["project"][key] != python["project"][key]:
            raise ValueError(f"Root Python metadata diverges: {key}")
    if (
        root_python["tool"]["maturin"]["features"]
        != python["tool"]["maturin"]["features"]
    ):
        raise ValueError("Root and binding wheel features diverge")
    return metadata


def publish_order(metadata):
    """Topologically sort every publishable workspace crate, including bindings."""
    pending = {
        p["name"]: p
        for p in metadata["packages"]
        if p["id"] in metadata["workspace_members"] and p.get("publish") != []
    }
    ordered = []
    while pending:
        ready = sorted(
            name
            for name, package in pending.items()
            if not any(d["name"] in pending for d in package["dependencies"])
        )
        if not ready:
            raise ValueError("Workspace publication dependency cycle")
        ordered.extend(pending.pop(name) for name in ready)
    return ordered


def audit_macos(path, entries):
    """On macOS verify bundled native libraries honor their wheel deployment tag."""
    tag = re.search(r"macosx_(\d+)_(\d+)_", path.name)
    if sys.platform != "darwin" or not tag:
        return
    minimum = tuple(map(int, tag.groups()))
    with tempfile.TemporaryDirectory() as directory:
        for index, (name, data) in enumerate(entries):
            if not name.endswith((".so", ".dylib")):
                continue
            binary = Path(directory) / str(index)
            binary.write_bytes(data)
            commands = subprocess.check_output(["otool", "-l", str(binary)], text=True)
            versions = re.findall(r"\bminos (\d+)\.(\d+)", commands)
            versions += re.findall(
                r"cmd LC_VERSION_MIN_MACOSX\s+cmdsize \d+\s+version (\d+)\.(\d+)",
                commands,
            )
            if any(tuple(map(int, version)) > minimum for version in versions):
                raise ValueError(
                    f"Native library exceeds wheel macOS deployment tag: {name}"
                )
            links = subprocess.check_output(["otool", "-L", str(binary)], text=True)
            for line in links.splitlines()[1:]:
                link = line.strip().split(" (", 1)[0]
                if link.startswith("/") and not link.startswith(
                    ("/usr/lib/", "/System/Library/")
                ):
                    raise ValueError(f"Unbundled native dependency in {name}: {link}")


def audit_archive(path):
    """Reject coordination files, credentials and personal build paths in payloads."""
    if path.suffix == ".whl":
        archive = zipfile.ZipFile(path)
        entries = [
            (name, archive.read(name))
            for name in archive.namelist()
            if not name.endswith("/")
        ]
    else:
        archive = tarfile.open(path)  # noqa: SIM115 - shared context below
        entries = [
            (item.name, archive.extractfile(item).read())
            for item in archive
            if item.isfile()
        ]
    with archive:
        for name, data in entries:
            if PRIVATE_NAMES.intersection(Path(name).parts) or name.endswith(
                (".pyc", ".log")
            ):
                raise ValueError(f"Private file in {path.name}: {name}")
            if any(
                marker in data
                for marker in (
                    b"/Users/",
                    b"C:\\Users\\",
                    b"C:/Users/",
                    b"/home/runner/work/",
                    b"pypi-AgEI",
                )
            ):
                raise ValueError(
                    f"Private build path or credential in {path.name}: {name}"
                )
        if path.suffix == ".whl":
            if "-cp311-abi3-" not in path.name:
                raise ValueError(f"Expected cp311-abi3 wheel: {path.name}")
            metadata = [
                data for name, data in entries if name.endswith(".dist-info/METADATA")
            ]
            if len(metadata) != 1:
                raise ValueError("Expected one wheel METADATA")
            fields = email.message_from_bytes(metadata[0])
            version = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"][
                "package"
            ]["version"]
            if (fields["Name"], fields["Version"], fields["Requires-Python"]) != (
                "typf",
                version,
                ">=3.11",
            ):
                raise ValueError("Wheel metadata does not match release")
            names = {name for name, _ in entries}
            if not {"typf/__init__.py", "typfpy/__init__.py"}.issubset(names):
                raise ValueError("Wheel must contain typf and typfpy")
            audit_macos(path, entries)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["validate", "order", "audit"])
    parser.add_argument("paths", nargs="*")
    parser.add_argument("--version")
    args = parser.parse_args()
    metadata = validate(args.version)
    if args.command == "order":
        print("\n".join(p["name"] for p in publish_order(metadata)))
    elif args.command == "audit":
        if not args.paths:
            parser.error("audit requires at least one archive")
        for pattern in args.paths:
            path = Path(pattern)
            audit_archive(path)
            print(f"Audited {path.name}")
    else:
        print(f"Validated {len(metadata['workspace_members'])} workspace packages")


if __name__ == "__main__":
    main()
