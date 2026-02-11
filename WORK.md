<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** BCP47 language-tag validation parity micro-sprint (render + JSONL + batch)

## Completed

- [x] Added shared BCP 47 language-tag normalization utility in `crates/typf-cli/src/language.rs` using `language-tags`
- [x] Updated `typf render --language` parsing to validate BCP 47 and canonicalize tag casing
- [x] Updated JSONL `text.language` parsing to validate/canonicalize with explicit `Invalid text.language` diagnostics
- [x] Updated `typf batch` per-job `language` parsing to validate/canonicalize with explicit `Invalid batch language tag` diagnostics
- [x] Added regression tests for valid canonicalization and invalid-tag error paths across render, JSONL, batch, and shared language utility

## Research Notes

- RFC 5646 (BCP 47 language-tag grammar and casing conventions):
  https://www.rfc-editor.org/rfc/rfc5646
- `language-tags` crate API (`LanguageTag::parse`, `canonicalize`):
  https://docs.rs/language-tags/latest/language_tags/struct.LanguageTag.html

## Verification Results

- `cargo test -p typf-cli --all-features`
  - Result: PASS
- `./test.sh --quick`
  - Result: PASS
- `./test.sh`
  - Result: PASS (Rust fmt, clippy, full workspace Rust tests/doc-tests, Python lint, Python tests)
  - Python tests: `27 passed`

## Notes

- Added dependency: `language-tags v0.3.2`
- Existing unrelated repository changes were preserved.

## Next

- No active scratch tasks in this session.
