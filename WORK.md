<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Post-v5.0.2 finite-font-size validation consistency micro-sprint

## Sprint Tasks

- [x] `typf-core::ShapingParams::validate()` rejects non-finite font sizes (`NaN`, `+/-inf`) before other checks
- [x] JSONL job processing now relies on core shaping validation for `font.size` (single validation authority)
- [x] Regression tests added for non-finite font sizes (`NaN`, `+inf`, `-inf`) in core + JSONL paths

## Research Notes

- Rust `f32::is_finite()` semantics for rejecting `NaN`/`+/-inf`:
  https://doc.rust-lang.org/core/primitive.f32.html#method.is_finite
- `serde_json::Number::from_f64()` rejects non-finite values (relevant for JSON boundaries):
  https://docs.rs/serde_json/latest/serde_json/value/struct.Number.html#method.from_f64

## Verification Results

- `cargo fmt --manifest-path crates/typf-core/Cargo.toml`: PASS
- `cargo fmt --manifest-path crates/typf-cli/Cargo.toml`: PASS
- `cargo test -p typf-core -p typf-cli -- --nocapture`: PASS
- `./test.sh --quick`: PASS

## Notes

- Touched files:
  - `crates/typf-core/src/lib.rs`
  - `crates/typf-cli/src/jsonl.rs`
  - `TASKS.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `WORK.md`
- Existing unrelated repository changes were preserved.

## Next

- No open items in this sprint.
