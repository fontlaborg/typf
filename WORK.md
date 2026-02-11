<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Post-v5.0.2 stream-diagnostics/color-input micro-sprint

## Sprint Tasks

- [x] Add line-aware JSONL stream diagnostics for parse and `job.id` validation failures
- [x] Prefix JSONL stream execution errors with source line numbers while preserving job IDs
- [x] Improve render CLI input parsing: support shorthand hex colors (`RGB`/`RGBA`) and contextual invalid font-size diagnostics

## Research Notes

- JSON Lines format reference:
  https://jsonlines.org/
- MDN hex-color shorthand/full notation reference (`#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`):
  https://developer.mozilla.org/en-US/docs/Web/CSS/hex-color
- Rayon indexed `collect()` ordering behavior:
  https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.collect

## Verification Results

- `cargo fmt --manifest-path crates/typf-cli/Cargo.toml`: PASS
- `cargo clippy -p typf-cli --all-features -- -D warnings`: PASS
- `cargo test -p typf-cli`: PASS
- `./test.sh --quick`: PASS

## Notes

- Touched code paths:
  - `crates/typf-cli/src/jsonl.rs`
  - `crates/typf-cli/src/commands/render.rs`
- Updated project tracking docs:
  - `TASKS.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `WORK.md`
- Existing unrelated repository changes were preserved.

## Next

- No open items in this micro-sprint.
