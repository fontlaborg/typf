"""Unicode escape parsing regression tests for the Python CLI helpers."""
# this_file: bindings/python/tests/test_cli_unicode_escapes.py

from typfpy.cli import decode_unicode_escapes


def test_decode_unicode_escapes_when_basic_u4_then_decodes():
    assert decode_unicode_escapes(r"\u0041") == "A"


def test_decode_unicode_escapes_when_braced_escape_then_decodes():
    assert decode_unicode_escapes(r"\u{1F600}") == "😀"


def test_decode_unicode_escapes_when_surrogate_pair_then_decodes():
    assert decode_unicode_escapes(r"\uD83D\uDE00") == "😀"


def test_decode_unicode_escapes_when_uppercase_u8_then_decodes():
    assert decode_unicode_escapes(r"\U0001F600") == "😀"


def test_decode_unicode_escapes_when_malformed_then_preserves_literal():
    assert decode_unicode_escapes(r"\u12") == r"\u12"
    assert decode_unicode_escapes(r"\u{xyz}") == r"\u{xyz}"
    assert decode_unicode_escapes(r"\uD83D") == r"\uD83D"
    assert decode_unicode_escapes(r"\U0000ZZZZ") == r"\U0000ZZZZ"
    assert decode_unicode_escapes(r"\U00110000") == r"\U00110000"
