<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Post-v5.0.2 feature-tag diagnostics micro-sprint

## Sprint Tasks

- [x] Render CLI feature-tag validation: check printable-ASCII bytes before tag-length checks
- [x] JSONL feature-tag validation: align ordering with render CLI for deterministic multibyte diagnostics
- [x] Add multibyte non-ASCII feature-tag regression tests for both render CLI and JSONL parser paths

## Research Notes

- OpenType feature tags are 4-byte tags:
  https://learn.microsoft.com/en-us/typography/opentype/spec/featuretags
- Rust `str::len` returns bytes (important for 4-byte tag validation behavior):
  https://doc.rust-lang.org/std/primitive.str.html#method.len

## Verification Results

- `cargo fmt --manifest-path crates/typf-cli/Cargo.toml`: PASS
- `cargo test -p typf-cli -- --nocapture`: PASS
- `./test.sh --rust --quick`: FAIL (existing unrelated formatting drift in `crates/typf-core/src/lib.rs` blocks global fmt check)

## Notes

- This sprint touched `crates/typf-cli/src/commands/render.rs` and `crates/typf-cli/src/jsonl.rs`.
- Existing repo-local changes outside this sprint were preserved as-is.

## Next

- No open items in this sprint.
