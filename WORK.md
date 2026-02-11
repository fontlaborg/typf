<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Post-v5.0.2 JSONL font-loader/face-index/text-hint-normalization micro-sprint

## Sprint Tasks

- [x] Replace JSONL job font loading shim with real `TypfFontFace::from_file_index()` loading
- [x] Respect JSONL `font.source.face_index` and include `face_index=<n>` context in load-failure diagnostics
- [x] Normalize optional JSONL `text.language`/`text.script` hints (trim + blank to `None`) before shaping

## Research Notes

- JSON Lines format reference:
  https://jsonlines.org/
- Serde container attributes (`deny_unknown_fields`) reference:
  https://serde.rs/container-attrs.html#deny_unknown_fields
- Serde field attributes (alias/default compatibility):
  https://serde.rs/field-attrs.html#alias

## Verification Results

- `cargo clippy -p typf-cli --all-targets -- -D warnings`: PASS
- `cargo test -p typf-cli jsonl::tests`: PASS
- `./test.sh --quick`: PASS

## Notes

- Touched code paths:
  - `crates/typf-cli/src/jsonl.rs`
- Updated project tracking docs:
  - `TASKS.md`
  - `TODO.md`
  - `CHANGELOG.md`
  - `WORK.md`
- Existing unrelated repository changes were preserved.

## Next

- No open items in this micro-sprint.
