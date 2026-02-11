<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Script normalization + shared limit helper dedup micro-sprint

## Completed

- Added `crates/typf-cli/src/script.rs` with shared ISO 15924 script-tag normalization (`None`/blank/`auto` => unset, strict 4-letter ASCII validation, canonical titlecase)
- Rewired render CLI script parsing (`crates/typf-cli/src/commands/render.rs`) to use shared script normalization
- Rewired JSONL script parsing (`crates/typf-cli/src/jsonl.rs`) to use shared script normalization
- Added shared text-size constant + validator in `crates/typf-cli/src/limits.rs`:
  - `MAX_TEXT_CONTENT_BYTES`
  - `validate_text_size_limit(...)`
- Replaced duplicated text-size guards in render CLI and JSONL with shared helper calls
- Added boundary regression tests for text-size validation:
  - render CLI boundary accept test
  - JSONL boundary accept test
  - limits helper at-limit and over-limit tests
- Added missing `this_file` markers across `crates/typf-cli/src/*.rs` and `crates/typf-cli/src/commands/*.rs`

## Completed (Previous)

- Fixed doc comment for `get_max_bitmap_height()` in `crates/typf-core/src/lib.rs` to reflect the actual default value of 16384 pixels, resolving an inconsistency with `REVIEW.md` and `TASKS.md` (Task 1.1).

## Research Notes

- RFC 5646 / BCP 47 syntax and canonical casing guidance:
  https://www.rfc-editor.org/rfc/rfc5646
- `language-tags` crate canonicalization API used elsewhere in typf-cli:
  https://docs.rs/language-tags/latest/language_tags/struct.LanguageTag.html

## Verification Results

- `cargo test -p typf-cli --all-features`
  - Result: PASS (177 unit tests + 26 CLI smoke tests)
- `./test.sh --quick`
  - Result: PASS (workspace fmt, clippy, Rust tests/doc-tests, Python lint, Python tests)
  - Python tests: `27 passed`

## Notes

- No new dependencies added.
- Existing unrelated repository changes were preserved.

## Next

- No active scratch tasks in this session.
