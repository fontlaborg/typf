# TODO (backend-neutral caching refactor)

- [x] P1: Extend `ShapingCacheKey` with backend tag and update all callers/tests.
- [x] P1: Add shared `glyph_cache.rs` (render-output cache) with stable key hashing and stats.
- [x] P1: Add `CachePolicy` + builder toggles; wrap shaper/renderer with cached adaptors in `PipelineBuilder`.
- [x] P1: Wire CLI flags `--no-shaping-cache/--no-glyph-cache`; force caches off for linra path.
- [x] P1: Update HarfBuzz/ICU shapers to new key constructor and remove any stale cache paths.
- [x] P1: Add core unit tests validating cache hits/misses and toggles; add CLI test for cache flags.
- [x] P1: Run `cargo test --workspace --all-features` and record results in WORK.md.
