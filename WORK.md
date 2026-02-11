<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Post-v5.0.2 parser delimiter micro-sprint

## Sprint Tasks

- [x] Render CLI: accept comma/tab/newline-delimited OpenType feature lists
- [x] Render CLI: accept comma/tab/newline-delimited variation-axis lists
- [x] Render CLI: accept comma/tab/newline-delimited glyph-source lists

## Research Notes

- Rust `split_whitespace()` reference (whitespace tokenization contract):
  https://doc.rust-lang.org/std/primitive.str.html#method.split_whitespace
- Rust iterator `flat_map()` reference (CSV + whitespace composition):
  https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.flat_map

## Verification Results

- `cargo fmt --manifest-path crates/typf-cli/Cargo.toml`: PASS
- `cargo test --manifest-path crates/typf-cli/Cargo.toml commands::render::tests -- --nocapture`: PASS
- `./test.sh`: PASS

## Notes

- `cargo fmt --all` still fails in this repo snapshot due a missing vendored `external/vello/vello/Cargo.toml`; scoped formatting via `--manifest-path crates/typf-cli/Cargo.toml` is required.

## Next

- No open items in this sprint.
