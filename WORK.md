<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Post-v5.0.2 unicode-escape reliability micro-sprint

## Sprint Tasks

- [x] Decode UTF-16 surrogate pairs in Rust CLI `decode_unicode_escapes` (`\uXXXX\uXXXX`)
- [x] Preserve malformed `\u` escapes verbatim in both Rust and Python CLI decoders
- [x] Add Rust + Python regression coverage for basic/braced/surrogate/malformed Unicode escapes

## Research Notes

- JSON string escaping and surrogate-pair model:
  https://www.rfc-editor.org/rfc/rfc8259#section-7
- Rust Unicode UTF-16 decoding reference:
  https://doc.rust-lang.org/std/char/fn.decode_utf16.html

## Verification Results

- `cargo test -p typf-cli decode_unicode_escapes -- --nocapture`: PASS
- `uv run pytest bindings/python/tests/test_cli_unicode_escapes.py -q`: PASS
- `./test.sh --quick`: PASS

## Notes

- Touched code paths:
  - `crates/typf-cli/src/commands/render.rs`
  - `bindings/python/python/typfpy/cli.py`
  - `bindings/python/tests/test_cli_unicode_escapes.py`
- Existing unrelated repository changes were preserved.

## Next

- No open items in this sprint.
