<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Post-v5.0.2 JSONL quality sprint

## Sprint Tasks

- [x] JSONL: Parse `text.features` into shaping features (`Vec<(String, u32)>`)
- [x] JSONL: Accept canonical `version` with legacy `_version` alias for compatibility
- [x] JSONL: Parallelize `run_batch` with deterministic result ordering

## Verification Results

- `cargo fmt --manifest-path crates/typf-cli/Cargo.toml`: PASS
- `cargo test --manifest-path crates/typf-cli/Cargo.toml jsonl -- --nocapture`: PASS
- `cargo test --workspace`: PASS
- `cargo clippy --manifest-path crates/typf-cli/Cargo.toml --all-targets -- -D warnings`: PASS

## Notes

- `cargo fmt --all` still fails in this repo snapshot due a missing vendored `external/vello/vello/Cargo.toml`; scoped formatting via `--manifest-path crates/typf-cli/Cargo.toml` is required.

## Next

- No open items in this sprint.
