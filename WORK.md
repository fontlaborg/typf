<!-- this_file: WORK.md -->
# Current Work Session

**Documentation update session** - Analyzing and updating documentation to reflect v5.0.1 changes and improvements across the entire codebase.

## Work Log (Dec 16, 2025)

1. **Documentation comprehensive update** - Analyzed recent git changes (2579 additions, 885 deletions across 46 files) and updated README.md, caching section, and status to reflect v5.0.1 improvements including Moka TinyLFU caching, enhanced color font support, and zero-copy optimization.
2. **Planning docs reshaped** - Converted `PLAN.md` into an index and split the full content into `PLANSTEPS/01-..09-*.md`; rewrote `TODO.md` as a flat actionable backlog derived from the plan.
2. **Vello GPU color-font UX** - Added a CLI warning when `--renderer vello` is used with a font that has COLR/SVG/bitmap tables; added unit tests for color-table detection.
3. **Verification** - `cargo fmt --check`, `cargo test -p typf-cli`, `cargo clippy -p typf-cli -- -D warnings`, `cargo test` (workspace; one pre-existing `dead_code` warning in a test helper)
4. **Docs + tracking** - Corrected documentation to reflect the current Vello-GPU limitation for bitmap/COLR glyphs; added upstream tracking reference; marked Vello-GPU color-font tests as ignored (until vendored Vello is updated).
5. **Project hygiene** - Added `DEPENDENCIES.md` and aligned Cargo workspace versions with git tag/docs (`5.0.1`), updating `Cargo.lock` accordingly.
6. **Verification (full)** - `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test` (workspace) all pass.
7. **SVG exporter placeholder fallback** - `typf-export-svg` now emits a `typf-missing-glyph` placeholder box for bitmap-only glyphs when bitmap embedding is disabled/unavailable; added an integration test and verified with `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test`.
8. **Zero-copy font bytes (Stage 3 interop)** - Added `FontRef::data_shared()` and implemented it in `typf-fontdb`; updated Vello CPU/GPU renderers to use shared bytes when available (avoid per-render `to_vec()` copies); verified with `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test`.
9. **Font metrics API (Stage 3 interop)** - Added `types::FontMetrics` + `FontRef::metrics()` and implemented it in `typf-fontdb`; added a real-font regression test for table-derived ascent/descent/line_gap.
10. **Cache test stabilization** - Added `cache_config::scoped_caching_enabled()` and updated cache-related tests to use it; verified with `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test` (workspace) all pass.
11. **Bitmap PNG decoding hardened** - `typf-render-color` now decodes indexed/paletted PNGs for bitmap glyphs (sbix/CBDT/EBDT) and has a regression test; fixed `cargo test -p typf-render-color --features bitmap` by gating SVG-only examples behind `required-features`; verified with `cargo fmt --check`, `cargo test -p typf-render-color --features bitmap`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test` (workspace).
12. **Baseline standardization (spec)** - Audited current baseline math across Opixa/Skia/Zeno/Vello(-cpu)/Vello/CGBitmap, documented deltas, and wrote down a metrics-first contract in `src_docs/06-backend-architecture.md`; updated `TODO.md` accordingly.
13. **Baseline standardization (implementation)** - Updated Opixa/Skia/Zeno/Vello/Vello-CPU to prefer font ascent/descent for line box sizing and baseline placement, expanding to include glyph bounds when they exceed metrics; added a cross-renderer regression test in `crates/typf/tests/baseline_consistency.rs`.
14. **Verification (full)** - `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test` (workspace) all pass.

## Recent Completions (Dec 16, 2025)

1. **Added color_palette test** - Added assertion for `color_palette` field in `test_params_conversion` test
2. **Updated REVIEW.md** - Changed version from 2.5.4 to 5.0.1 and date to Dec 16, 2025
3. **Verified Python bindings** - `cargo check -p typf-py --all-features` passes

## Earlier Today (Dec 16, 2025)

1. **Added color_palette to LinraRenderParams** - Allows specifying CPAL palette index for COLR color glyphs
2. **Fixed flaky cache tests** - Applied parallel test interference fix to typf-shape-icu-hb and typf-shape-hr
3. **Fixed unused import warning** - Removed `get_svg_document` from svg_flip_test.rs example
4. **Fixed dead code warning** - Added `#[allow(dead_code)]` to MockFont in vello-cpu tests
5. **Updated version numbers** - PLAN.md and TODO.md now reflect v5.0.1 (matching git tag)
6. **Updated test count** - Now 414 tests across workspace

## Previous Session

1. **Cache architecture replaced** - Replaced custom L1/L2 (HashMap + LRU) with Moka TinyLFU
   - Fixes unbounded memory growth during scan workloads (font matching)
   - TinyLFU tracks frequency of both hits AND misses
   - Added 10-minute time-to-idle for automatic cleanup
2. **README.md caching section** - Added comprehensive caching documentation
3. **QUICKSTART.md created** - Rust usage guide with caching, variable fonts, color fonts
