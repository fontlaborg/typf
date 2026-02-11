<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Post-v5.0.2 verification-integrity micro-sprint

## Sprint Tasks

- [x] Canonicalize duplicate OpenType feature tags in both CLI render parsing and JSONL feature parsing (`last value wins`) with regression tests
- [x] Tighten JSONL `version` validation to reject malformed versions (`empty`, `non-numeric minor`, `extra segments`) while preserving `2`/`2.x` compatibility
- [x] Make `scripts/test.sh` fail when Python lint/tests fail (when those checks are executed) so success status is trustworthy

## Research Notes

- HarfBuzz feature parsing and precedence behavior reference:
  https://harfbuzz.github.io/harfbuzz-hb-common.html#hb-feature-from-string
- OpenType feature-tag byte constraints reference:
  https://learn.microsoft.com/en-us/typography/opentype/spec/featuretags
- Bash `pipefail` behavior reference:
  https://www.gnu.org/software/bash/manual/bash.html#The-Set-Builtin

## Verification Results

- `cargo fmt --manifest-path crates/typf-cli/Cargo.toml`: PASS
- `cargo test -p typf-cli -- --nocapture`: PASS
- `./test.sh --python --quick`: PASS
- `./test.sh --quick`: PASS

## Notes

- Touched code paths:
  - `crates/typf-cli/src/commands/render.rs`
  - `crates/typf-cli/src/jsonl.rs`
  - `scripts/test.sh`
- Existing unrelated repo changes were preserved.

## Next

- No open items in this sprint.
