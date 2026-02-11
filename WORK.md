<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Post-v5.0.2 CLI validation micro-sprint

## Sprint Tasks

- [x] JSONL: reject unsupported batch `version` values (accept only major `2.x`)
- [x] JSONL: validate `text.direction` (`ltr|rtl|ttb|btt`) and reject unknown values
- [x] CLI render: enforce OpenType feature-tag validation (exactly 4 printable ASCII chars)

## Research Notes

- OpenType feature tags are defined as four-byte tags:
  https://learn.microsoft.com/en-us/typography/opentype/spec/featuretags
- HarfBuzz feature syntax (`+kern`, `-kern`, `liga=0`) uses OpenType feature tags:
  https://harfbuzz.github.io/harfbuzz-hb-common.html#hb-feature-from-string
- OpenType variation axis tags are also 4-byte tags (kept parser behavior aligned):
  https://learn.microsoft.com/en-us/typography/opentype/spec/dvaraxisreg

## Verification Results

- `cargo fmt --manifest-path crates/typf-cli/Cargo.toml`: PASS
- `cargo test --manifest-path crates/typf-cli/Cargo.toml --all-features -- --nocapture`: PASS
- `cargo clippy --manifest-path crates/typf-cli/Cargo.toml --all-features --all-targets -- -D warnings`: PASS
- `./test.sh --rust --quick`: PASS

## Notes

- `cargo fmt --all` still fails in this repo snapshot due a missing vendored `external/vello/vello/Cargo.toml`; scoped formatting via `--manifest-path crates/typf-cli/Cargo.toml` is required.

## Next

- No open items in this sprint.
