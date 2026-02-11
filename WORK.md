<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Post-v5.0.2 JSONL determinism/input-normalization micro-sprint

## Sprint Tasks

- [x] JSONL `font.instance.variations`: validate in stable sorted axis-tag order for deterministic diagnostics
- [x] JSONL parsing: trim surrounding whitespace for `version` and `text.direction` (blank direction defaults to LTR)
- [x] JSONL processing: reject non-finite/non-positive `font.size` before shaping with explicit error context

## Research Notes

- Rust `HashMap` iterators are in arbitrary order, so deterministic diagnostics require explicit sorting:
  https://doc.rust-lang.org/std/collections/hash_map/struct.HashMap.html
- Rust finite float checks for input validation:
  https://doc.rust-lang.org/core/primitive.f32.html#method.is_finite
- Rust string trimming behavior used for robust JSONL parser normalization:
  https://doc.rust-lang.org/std/primitive.str.html#method.trim

## Verification Results

- `cargo fmt --manifest-path crates/typf-cli/Cargo.toml`: PASS
- `cargo test --manifest-path crates/typf-cli/Cargo.toml -- --nocapture`: PASS
- `./test.sh --quick`: PASS

## Notes

- This sprint touched `crates/typf-cli/src/jsonl.rs` plus task-tracking docs.
- Existing repo-local changes outside this sprint were preserved as-is.

## Next

- No open items in this sprint.
