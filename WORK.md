<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Render/JSONL script-hint + text-size validation parity micro-sprint

## Completed

- [x] Verified render CLI text payload-size guardrail (`MAX_TEXT_CONTENT_BYTES=1_000_000`) before shaping
- [x] Verified render CLI `--script` parsing enforces ISO 15924-style 4 ASCII letters and canonical titlecase (`auto`/blank => unset)
- [x] Verified render CLI `--language` normalization (`trim` + blank => unset) before auto-direction resolution
- [x] Verified JSONL `text.script` validation/canonicalization parity with render CLI
- [x] Verified regression coverage for payload-size/script/language normalization paths in render + JSONL tests

## Research Notes

- RFC 5646 language-tag syntax + script subtags (`script = 4ALPHA`, case-insensitive, canonical titlecase):
  https://www.rfc-editor.org/rfc/rfc5646
- IANA Language Subtag Registry metadata:
  https://www.iana.org/assignments/language-subtags-tags-extensions/language-subtags-tags-extensions.xhtml
- ISO 15924 script code list:
  https://www.unicode.org/iso15924/iso15924-codes.html

## Verification Results

- `cargo test --manifest-path crates/typf-cli/Cargo.toml render::tests:: -- --nocapture`
  - Result: PASS (`46` render-module unit tests)
- `cargo test --manifest-path crates/typf-cli/Cargo.toml jsonl::tests:: -- --nocapture`
  - Result: PASS (`71` JSONL-module unit tests)
- `./test.sh --quick`
  - Result: PASS (Rust fmt, clippy, quick Rust workspace tests/doc-tests, Python lint, Python tests)
  - Python tests: `27 passed`

## Notes

- Existing unrelated repository changes were preserved.

## Next

- No active scratch tasks in this session.
