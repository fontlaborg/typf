<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Post-v5.0.2 CLI/JSONL input-hardening micro-sprint

## Sprint Tasks

- [x] CLI `--instance` parsing: reject unsupported named-instance tokens and validate axis tags as 4 printable ASCII bytes
- [x] CLI variation parsing: canonicalize axis list (sorted, duplicate axis tags resolve deterministically with last value)
- [x] CLI font-size parsing: reject non-finite/non-positive values and values above `MAX_FONT_SIZE`
- [x] JSONL validation: enforce supported `rendering.encoding` values (`base64|plain`) and validate/sort `font.instance.variations`
- [x] JSONL text-feature parsing: accept comma/tab/newline delimiters consistently

## Research Notes

- OpenType axis tags are 4-byte tags:
  https://learn.microsoft.com/en-us/typography/opentype/spec/dvaraxisreg
- OpenType feature tags are 4-byte tags:
  https://learn.microsoft.com/en-us/typography/opentype/spec/featuretags
- HarfBuzz feature parsing reference:
  https://harfbuzz.github.io/harfbuzz-hb-common.html#hb-feature-from-string
- Rust finite-float checks (`f32::is_finite`):
  https://doc.rust-lang.org/core/primitive.f32.html#method.is_finite

## Verification Results

- `cargo fmt --manifest-path crates/typf-cli/Cargo.toml`: PASS
- `cargo test --manifest-path crates/typf-cli/Cargo.toml -- --nocapture`: PASS
- `./test.sh --quick`: PASS

## Notes

- This sprint touched only `crates/typf-cli/src/commands/render.rs` and `crates/typf-cli/src/jsonl.rs`.
- Existing repo-local changes outside this sprint were preserved as-is.

## Next

- No open items in this sprint.
