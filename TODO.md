<!-- this_file: TODO.md -->
# TODO

## Phase 1: Fix Build & Sanity Checks (Immediate)

- [ ] **T1: Fix Compilation Error in `typf-core/src/cache.rs`**
  - Add `use std::sync::OnceLock;` to imports.
  - Make `shaping_cache` and `glyph_cache` fields in `CacheManager` struct public (or `pub(crate)`).

- [ ] **T2: Fix Formatting in `typf-core/src/cache_config.rs`**
  - Run `cargo fmt -- crates/typf-core/src/cache_config.rs` to fix the layout of `assert!` macros.

- [ ] **T3: Fix `sanity_fmt` failure in `test.py`**
  - Change `["cargo", "fmt", "--all", "--check"]` to `["cargo", "fmt", "--check"]` in `test.py` line 213.

- [ ] **T4: Fix `sanity_clippy` failure in `typf-core`**
  - Add `#[allow(clippy::expect_used, clippy::panic)]` to the `#[cfg(test)] mod tests` block in `crates/typf-core/src/lib.rs`.

- [ ] **T5: Verify Build Fixes**
  - Run `python3 test.py` (or the specific sanity checks) to confirm green.

## Phase 2: Critical Code Quality Fixes (From REVIEW.md)

- [ ] **T6: Fix Doc/Code Value Mismatches in `lib.rs`**
  - Update doc comments for `DEFAULT_MAX_BITMAP_WIDTH` and height to match actual values (16M and 16K).
  - File: `crates/typf-core/src/lib.rs`

- [ ] **T7: Verify `clear_all_caches()` Implementation**
  - Verify if `cache_config.rs` implementation works as expected with a test case.

- [ ] **T8: Fix WASM `render_text` MockFont**
  - `typf/src/wasm.rs` uses `MockFont`. Add warning or proper implementation.

## Phase 3: High Priority Debt (From REVIEW.md)

- [ ] **T9: Eliminate `unwrap()` in Non-Test Code**
  - `crates/typf-cli/src/batch.rs`: Replace `unwrap()` with `?`.
  - `crates/typf-cli/src/jsonl.rs`: Replace `unwrap()` with `Result`.
  - `crates/typf-export/src/svg.rs`: Replace `unwrap()` with `?`.
  - Change `unwrap_used = "warn"` to `deny` in root `Cargo.toml`.

- [ ] **T10: Fix Silent Error Swallowing in SVG Exporters**
  - `crates/typf-export-svg/src/lib.rs`: Replace `let _ = write!(...)` with `write!(...)?`.
  - `backends/typf-render-svg/src/lib.rs`: Replace `let _ = write!(...)` with `write!(...)?`.

- [ ] **T11: Consolidate Duplicate Cache Key Types**
  - Decide on canonical `ShapingCacheKey` and `GlyphCacheKey` (likely in `cache.rs` or specialized modules) and remove duplicates.

- [ ] **T12: Clean Up Vestigial Cache Naming**
  - Rename `MultiLevelCache` to `MokaCache`.
  - Remove L1/L2 fields from `CacheMetrics`.

## Phase 4: Final Verification

- [ ] **T13: Full Regression Test**
  - Run `python3 test.py` (all 18 steps).
