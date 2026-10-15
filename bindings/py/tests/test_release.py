"""Installed-wheel identity and compatibility checks."""
# this_file: bindings/py/tests/test_release.py

from importlib.metadata import metadata, version


def test_distribution_and_both_imports_agree():
    import typf
    import typfpy

    assert version("typf") == typf.__version__ == typfpy.__version__ == "5.1.0"
    assert typf.Typf is typfpy.Typf
    assert typf.__all__ == typfpy.__all__
    assert metadata("typf")["Requires-Python"] == ">=3.11"
