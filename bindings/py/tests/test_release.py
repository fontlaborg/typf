"""Installed-wheel identity and compatibility checks."""
# this_file: bindings/py/tests/test_release.py

import sys
from importlib.metadata import metadata, version

import pytest


def test_distribution_and_both_imports_agree():
    import typf
    import typfpy

    assert version("typf") == typf.__version__ == typfpy.__version__ == "5.1.0"
    assert typf.Typf is typfpy.Typf
    assert typf.__all__ == typfpy.__all__
    assert metadata("typf")["Requires-Python"] == ">=3.11"


def test_macos_backends_match_the_runtime_platform():
    import typf

    if sys.platform == "darwin":
        assert typf.Typf(shaper="coretext", renderer="coregraphics") is not None
    else:
        with pytest.raises(ValueError, match="Unknown shaper"):
            typf.Typf(shaper="coretext")
        with pytest.raises(ValueError, match="Unknown renderer"):
            typf.Typf(shaper="none", renderer="coregraphics")
