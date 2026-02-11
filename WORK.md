<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Post-v5.0.2 batch hardening micro-sprint

## Sprint Tasks

- [x] Batch CLI: reject unsafe `output` paths (`..`, absolute paths, missing file names)
- [x] Batch CLI: validate output filename pattern requires `{}` placeholder
- [x] Batch CLI: reject unsupported `format` values and unknown JSON fields in batch jobs

## Research Notes

- Rust `Path::components` reference (path-segment validation):
  https://doc.rust-lang.org/std/path/struct.Path.html#method.components
- Rust `Path::is_absolute` reference (absolute-path rejection):
  https://doc.rust-lang.org/std/path/struct.Path.html#method.is_absolute
- Serde `deny_unknown_fields` reference (strict JSON schema):
  https://serde.rs/container-attrs.html#deny_unknown_fields

## Verification Results

- `cargo fmt --manifest-path crates/typf-cli/Cargo.toml`: PASS
- `cargo test --manifest-path crates/typf-cli/Cargo.toml -- --nocapture`: PASS
- `./test.sh --rust --quick`: PASS

## Notes

- `cargo fmt --all` still fails in this repo snapshot due missing vendored `external/vello/vello/Cargo.toml`; scoped formatting via `--manifest-path crates/typf-cli/Cargo.toml` remains required.

## Next

- No open items in this sprint.
