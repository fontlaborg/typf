<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Post-v5.0.2 cross-CLI unicode/color parser-parity micro-sprint

## Sprint Tasks

- [x] Add Rust CLI decoding support for 8-digit uppercase Unicode escapes (`\UXXXXXXXX`)
- [x] Add Python CLI decoding support for `\UXXXXXXXX` with malformed-literal preservation parity
- [x] Align Python CLI `parse_color()` with Rust shorthand/trim behavior (`RGB`/`RGBA`, trimmed input) and add regression tests

## Research Notes

- Python string escape semantics (`\u`, `\U`) in lexical analysis:
  https://docs.python.org/3/reference/lexical_analysis.html
- CSS hex color shorthand/longhand forms (`#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`):
  https://developer.mozilla.org/en-US/docs/Web/CSS/hex-color
- Rust Unicode scalar validity (`char::from_u32`) used for escape decoding checks:
  https://doc.rust-lang.org/std/primitive.char.html#method.from_u32

## Verification Results

- `cargo fmt --manifest-path crates/typf-cli/Cargo.toml`: PASS
- `cargo test --manifest-path crates/typf-cli/Cargo.toml decode_unicode_escapes -- --nocapture`: PASS
- `cd bindings/python && uv run --isolated --with pytest pytest tests/test_cli_unicode_escapes.py tests/test_cli_color_parsing.py -v`: PASS
- `./test.sh --quick`: PASS

## Notes

- Touched code paths:
  - `crates/typf-cli/src/commands/render.rs`
  - `bindings/python/python/typfpy/cli.py`
  - `bindings/python/tests/test_cli_unicode_escapes.py`
  - `bindings/python/tests/test_cli_color_parsing.py`
- Updated project tracking docs:
  - `TASKS.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `WORK.md`
- Existing unrelated repository changes were preserved.

## Next

- No open items in this micro-sprint.
