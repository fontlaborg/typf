<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Post-v5.0.2 render face-index/glyph-source + JSONL stream duplicate-id micro-sprint

## Sprint Tasks

- [x] Make `typf render` honor `--face-index` and return contextual `face_index=<n>` load errors
- [x] Harden `typf render --glyph-source` parsing (trim key/list whitespace + reject blank lists)
- [x] Reject duplicate JSONL stream `job.id` values across lines with line-aware diagnostics
- [x] Add regression tests for all above behaviors

## Research Notes

- OpenType collection model (`ttcf` header, `numFonts`, per-face offsets):
  https://learn.microsoft.com/en-us/typography/opentype/spec/otff#organization-of-an-opentype-font
- JSON Lines processing expectations for line-by-line records:
  https://jsonlines.org/

## Verification Results

- `cargo test -p typf-cli -- --nocapture`: PASS (`133` unit + `23` smoke)
- `cargo clippy -p typf-cli --all-targets -- -D warnings`: PASS
- `./test.sh --quick`: PASS

## Notes

- Touched code paths:
  - `crates/typf-cli/src/commands/render.rs`
  - `crates/typf-cli/src/jsonl.rs`
- Updated project tracking docs:
  - `TASKS.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `WORK.md`
- Existing unrelated repository changes were preserved.

## Next

- No open items in this micro-sprint.
