<!-- this_file: WORK.md -->
# Release 5.1.0 preparation

Date: 2026-10-15. Work remains local until the release controller verifies and publishes.

- Aligned all 27 workspace package versions and internal requirements at 5.1.0.
- Published Python identity is `typf`; `typfpy` remains a compatible API and CLI.
- Standard CPython >=3.11 stable ABI wheels, classifiers through 3.14.
- Replaced ignored Vello source paths with published 0.0.4 CPU/common/hybrid crates; aligned wgpu 26 and its poll API. No renderer removed.
- Rust minimum corrected to dependency-required 1.86 and inherited by all members.
- Replaced release automation with validated build-only and publication paths; default manual runs need no registry secret.
- Added topological publication order covering every workspace package, artifact payload checks, macOS native dependency/tag audit, installed-wheel identity checks and real-font rendering smoke tests.
- Excluded private agent documents, coordination state and bytecode from packages; disabled Maturin SBOM generation because it records absolute local Cargo source paths.
- Static bundled HarfBuzz selected by `HARFBUZZ_SYS_NO_PKG_CONFIG=1` in release builds. Maturin native repair remains enabled for other dependencies.

Verification completed before initial release-preparation commit:

- `cargo test --workspace`: PASS (full log retained locally).
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo fmt --all --check`: PASS.
- Release helper regression tests: 3 PASS; validator checks all 27 packages.
- `actionlint .github/workflows/release.yml`: PASS.
- Core/input `cargo publish --dry-run --locked --allow-dirty`: PASS, no uploads.
- Sdist payload audit: PASS; clean unpacked source builds offline without the ignored external checkout. Maturin prunes workspace members, so its source lockfile is refreshed by a normal source build; forcing `--locked` on the first unpacked build is unsupported.
- Final static HarfBuzz ABI3 wheel: PASS payload and macOS deployment/native link audit; no external Homebrew dylibs remain.
- Clean installs on CPython 3.11.17, 3.12.15, 3.13.16 and 3.14.8: 28 tests PASS each (112 total), no skips. Both imports and version metadata agree.
- Real-font opixa/skia/zeno/Vello CPU PNG and SVG smoke: PASS on each of the four Python versions; Vello GPU additionally rendered visible pixels.
- `cargo test --workspace --all-features`: 556 passed, 18 ignored, zero failures. Default workspace: 552 passed, 18 ignored.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: PASS.
- All 27 crate package file lists: PASS; no private agent notes or coordination state.
- `uvx twine check` on final wheel and sdist: PASS.
- Controller launched the GitHub platform build. The first musllinux jobs exposed an unnecessary package-manager install and an environment forwarding gap. Removed the system HarfBuzz install, explicitly forwarded the bundled-build switch into Docker, and selected PyPA musllinux images with CPython 3.11. Updated workflow passes actionlint. Controller must verify the rerun and downloaded artifacts before publication.

Official documentation consulted: PyO3 0.22 building/distribution, Maturin layout/distribution/config, Cargo publishing, wgpu 26 PollType, GitHub runner reference, and maturin-action inputs.

---

# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Script normalization + shared limit helper dedup micro-sprint

## Completed

- Added `crates/typf-cli/src/script.rs` with shared ISO 15924 script-tag normalization (`None`/blank/`auto` => unset, strict 4-letter ASCII validation, canonical titlecase)
- Rewired render CLI script parsing (`crates/typf-cli/src/commands/render.rs`) to use shared script normalization
- Rewired JSONL script parsing (`crates/typf-cli/src/jsonl.rs`) to use shared script normalization
- Added shared text-size constant + validator in `crates/typf-cli/src/limits.rs`:
  - `MAX_TEXT_CONTENT_BYTES`
  - `validate_text_size_limit(...)`
- Replaced duplicated text-size guards in render CLI and JSONL with shared helper calls
- Added boundary regression tests for text-size validation:
  - render CLI boundary accept test
  - JSONL boundary accept test
  - limits helper at-limit and over-limit tests
- Added missing `this_file` markers across `crates/typf-cli/src/*.rs` and `crates/typf-cli/src/commands/*.rs`

## Completed (Previous)

- Fixed doc comment for `get_max_bitmap_height()` in `crates/typf-core/src/lib.rs` to reflect the actual default value of 16384 pixels, resolving an inconsistency with `REVIEW.md` and `TASKS.md` (Task 1.1).

## Research Notes

- RFC 5646 / BCP 47 syntax and canonical casing guidance:
  https://www.rfc-editor.org/rfc/rfc5646
- `language-tags` crate canonicalization API used elsewhere in typf-cli:
  https://docs.rs/language-tags/latest/language_tags/struct.LanguageTag.html

## Verification Results

- `cargo test -p typf-cli --all-features`
  - Result: PASS (177 unit tests + 26 CLI smoke tests)
- `./test.sh --quick`
  - Result: PASS (workspace fmt, clippy, Rust tests/doc-tests, Python lint, Python tests)
  - Python tests: `27 passed`

## Notes

- No new dependencies added.
- Existing unrelated repository changes were preserved.

## Next

- No active scratch tasks in this session.
