<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** JSONL resource-limits and stream ID diagnostics micro-sprint

## Sprint Tasks

- [x] Add batch-size guardrail for JSONL specs (`MAX_BATCH_JOBS=10_000`) with explicit validation errors
- [x] Add per-job text payload-size guardrail (`MAX_TEXT_CONTENT_BYTES=1_000_000`) before shaping
- [x] Improve stream duplicate-ID diagnostics with first-seen line context and cap unique tracked IDs (`MAX_STREAM_UNIQUE_JOB_IDS=100_000`)

## Research Notes

- OWASP Input Validation Cheat Sheet (size/range limits for untrusted input):
  https://cheatsheetseries.owasp.org/cheatsheets/Input_Validation_Cheat_Sheet.html
- Rust `Result::expect_err` bound details (`T: Debug`) used while validating test robustness:
  https://doc.rust-lang.org/std/result/enum.Result.html#method.expect_err

## Verification Results

- `cargo test --manifest-path crates/typf-cli/Cargo.toml jsonl::tests:: -- --nocapture`: PASS
- `./test.sh --quick`: PASS

## Notes

- Touched code paths:
  - `crates/typf-cli/src/jsonl.rs`
- Updated tracking docs:
  - `TASKS.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `WORK.md`
- Existing unrelated repository changes were preserved.

## Next

- No open items in this micro-sprint.
