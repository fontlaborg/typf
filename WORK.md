<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Post-v5.0.2 JSONL/batch validation hardening micro-sprint

## Sprint Tasks

- [x] Reject blank and duplicate JSONL `job.id` values
- [x] Reject JSONL `rendering.width`/`rendering.height` values of `0`
- [x] Normalize/validate batch per-job optional input fields (`font`, `shaper`, `renderer`, `language`)
- [x] Normalize per-job batch backend tokens and reject blank per-job batch `format` values explicitly

## Research Notes

- JSON object member uniqueness interoperability guidance (RFC 8259):
  https://www.rfc-editor.org/rfc/rfc8259
- JSON Lines format reference:
  https://jsonlines.org/
- Rust string trimming behavior (`str::trim`) for robust input normalization:
  https://doc.rust-lang.org/std/primitive.str.html#method.trim
- Rust `HashSet` for duplicate detection:
  https://doc.rust-lang.org/std/collections/struct.HashSet.html

## Verification Results

- `cargo fmt --manifest-path crates/typf-cli/Cargo.toml`: PASS
- `cargo test --manifest-path crates/typf-cli/Cargo.toml -- --nocapture`: PASS
- `./test.sh --quick`: PASS

## Notes

- Touched code paths:
  - `crates/typf-cli/src/jsonl.rs`
  - `crates/typf-cli/src/commands/batch.rs`
- Updated project tracking docs:
  - `TASKS.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `WORK.md`
- Existing unrelated repository changes were preserved.

## Next

- No open items in this micro-sprint.
