"""Color parsing regression tests for Python CLI helpers."""
# this_file: bindings/python/tests/test_cli_color_parsing.py

import pytest

from typfpy.cli import parse_color


def test_parse_color_when_six_digit_with_whitespace_then_parses():
    assert parse_color("  #00ff7f\t") == (0x00, 0xFF, 0x7F, 0xFF)


def test_parse_color_when_rgb_shorthand_then_parses():
    assert parse_color("#0f8") == (0x00, 0xFF, 0x88, 0xFF)


def test_parse_color_when_rgba_shorthand_then_parses():
    assert parse_color("0f8c") == (0x00, 0xFF, 0x88, 0xCC)


def test_parse_color_when_invalid_length_then_errors_with_supported_formats():
    with pytest.raises(ValueError, match=r"RGB, RGBA, RRGGBB, or RRGGBBAA"):
        parse_color("#12")
