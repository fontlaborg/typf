# Current Work Session

- Enforced `GlyphSourcePreference` in opixa/skia/zeno/svg renderers with outline-deny guards and SVG color selection respecting prefer/deny.
- Added renderer unit tests and CLI integration tests for glyph-source ordering; updated SVG integration tests for color-preferred runs.
- Tests: `cargo test --workspace --all-features --quiet` (pass).

## 2025-12-02 — Backend-neutral caches
- Added backend-tagged `ShapingCacheKey`, shared glyph/render cache, and pipeline-level cache policy with default-on wrappers for all shapers/renderers.
- CLI now exposes `--no-shaping-cache` / `--no-glyph-cache` and skips caches automatically when using linra; added cache flag unit tests.
- Hardened SVG renderer tests to skip when optional COLRv1 font asset is missing.
- Tests: `cargo test --workspace --all-features --quiet` (pass with warnings about unused cache fields).
