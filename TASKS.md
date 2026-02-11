<!-- this_file: TASKS.md -->
# Typf Quality Improvement Plan

**Version:** 5.1.0 (Target)
**Status:** Planning
**Last Updated:** 2026-02-11
**Source:** Derived from comprehensive code review ([REVIEW.md](./REVIEW.md))

This plan addresses 23 findings from the 2026-02-11 deep audit, organized by priority. Each task references the specific REVIEW.md section where the issue was identified.

---

## Phase 1: Critical — Must Fix Before Next Release

These issues cause incorrect behavior, misleading documentation, or non-functional features.

### 1.1 Fix Doc/Code Value Mismatches in `lib.rs` (ref: REVIEW §2.1)

Three documentation comments state incorrect default values:

- [ ] **Line 94**: Change doc comment from "4096 pixels" to match actual value of `DEFAULT_MAX_BITMAP_WIDTH` (16,777,216 pixels = 16 * 1024 * 1024)
- [ ] **Line 112**: Change doc comment from "default (65535)" to match actual `DEFAULT_MAX_BITMAP_WIDTH` value
- [ ] **Line 122**: Change doc comment from "default (4095)" to match actual `DEFAULT_MAX_BITMAP_HEIGHT` (16,384 = 16 * 1024)
- [ ] Alternatively, if the *docs* are correct and the *constants* are wrong, fix the constants. **Decide which is intended before changing.**

### 1.2 Implement `clear_all_caches()` (ref: REVIEW §2.10)

- [ ] `cache_config.rs` line 135: `clear_all_caches()` is a documented no-op. Either:
  - Implement it to actually clear the Moka caches, OR
  - Mark it `#[deprecated]` with a clear message, OR
  - Remove it and update all callers
- [ ] Add a test that verifies caches are actually cleared after calling this function

### 1.3 Fix WASM `render_text` MockFont (ref: REVIEW §4.4)

- [ ] `typf/src/wasm.rs`: `render_text` uses `MockFont` — WASM rendering is non-functional for real fonts
- [ ] Either implement proper font loading in WASM (e.g., accept font bytes via `Uint8Array`), or:
  - Add prominent documentation that WASM rendering is a stub
  - Add a compile-time warning or runtime error when `render_text` is called without real font data

---

## Phase 2: High Priority — Fix in Next Sprint

These issues affect reliability, correctness, or have significant technical debt impact.

### 2.1 Eliminate `unwrap()` in Non-Test Code (ref: REVIEW §5.2)

~8 instances of `unwrap()` in production code paths that will panic on malformed input:

- [ ] `crates/typf-cli/src/batch.rs`: Replace `BatchConfig::parse(&args).unwrap()` with `?` and proper error context
- [ ] `crates/typf-cli/src/jsonl.rs`: Replace all `unwrap()` calls in JSON deserialization with `Result`-based error handling
- [ ] `crates/typf-export/src/svg.rs`: Replace `unwrap()` in base64 encoding with `?` propagation
- [ ] **Workspace policy**: Change `unwrap_used = "warn"` to `unwrap_used = "deny"` in root `Cargo.toml` line 168 (for non-test code)

### 2.2 Fix Silent Error Swallowing in SVG Exporters (ref: REVIEW §5.2)

~6 instances of `let _ = write!(...)` that silently discard write errors:

- [ ] `crates/typf-export-svg/src/lib.rs`: Replace all `let _ = write!(...)` with `write!(...)?`
- [ ] `backends/typf-render-svg/src/lib.rs`: Replace all `let _ = write!(...)` with `write!(...)?`
- [ ] Ensure return types are updated to `Result<(), std::fmt::Error>` or equivalent

### 2.3 Consolidate Duplicate Cache Key Types (ref: REVIEW §2.3)

- [ ] `ShapingCacheKey` is defined in both `cache.rs:27` (hash-based) and `shaping_cache.rs:27` (field-based). Decide which is canonical and remove the other.
- [ ] `GlyphCacheKey` has the same duplication: `cache.rs:37` vs `glyph_cache.rs:37`. Same fix.
- [ ] Remove dead `CacheStats` type from `cache.rs` line 181 (never used)
- [ ] Update all internal callers to use the canonical type

### 2.4 Add Windows Variable Font Support (ref: REVIEW §3.3)

- [ ] `typf-os-win` line 236: Implement variable font (fvar/gvar) support via DirectWrite
- [ ] This is marked `TODO` in the code — significant feature gap on Windows where variable fonts are increasingly common
- [ ] Add tests for variable font rendering on Windows CI

### 2.5 Complete NEON SIMD Path (ref: REVIEW §3.2)

- [ ] `backends/typf-render-opixa/src/simd.rs` line 169: NEON (ARM) SIMD is marked `TODO`
- [ ] Implement NEON intrinsics for scanline compositing (matching the AVX2/SSE4.1 paths)
- [ ] This affects all Apple Silicon Macs, Raspberry Pi, and ARM servers — significant performance impact
- [ ] Add ARM-specific benchmarks to verify speedup

---

## Phase 3: Medium Priority — Fix Within Quarter

These are maintainability, performance, and code hygiene improvements.

### 3.1 Clean Up Vestigial Cache Naming (ref: REVIEW §2.3, §2.4)

- [ ] Rename `MultiLevelCache` → `MokaCache` or `TinyLfuCache` to reflect single-level reality
- [ ] Remove `l1_hits`, `l2_hits`, `total_l1_time`, `total_l2_time` from `CacheMetrics` — replace with `hits`, `misses`, `total_access_time`
- [ ] Update stale doc comment in `shaping_cache.rs` line 89: remove "two-level cache (L1 hot cache + L2 LRU cache)" wording

### 3.2 Remove Dead Pipeline Stages (ref: REVIEW §2.2)

- [ ] Remove or implement `InputParsingStage`, `UnicodeProcessingStage`, `FontSelectionStage` (pipeline.rs lines 304-342) — currently they are pass-through no-ops that consume CPU
- [ ] If they are placeholders for future work, convert them to `todo!()` with tracking issue references
- [ ] Remove `#[allow(dead_code)]` on `cache_policy`, `shaping_cache`, `glyph_cache` fields (pipeline.rs lines 58-63) — either use these fields or remove them

### 3.3 Resolve Dead Code Modules in CLI (ref: REVIEW §4.1)

- [ ] `crates/typf-cli/src/batch.rs`: Either complete the batch processing module or remove it. Currently fully suppressed with `#[allow(dead_code)]`
- [ ] `crates/typf-cli/src/jsonl.rs`: Same — complete or remove
- [ ] `crates/typf-cli/src/repl.rs`: Same — complete or remove
- [ ] If keeping, remove the `#[allow(dead_code)]` suppressions and fix all resulting warnings

### 3.4 Decompose `typf-cli` Command Handler (ref: REVIEW §4.1)

- [ ] Extract font resolution logic from `commands/render.rs` `run()` into `resolver.rs`
- [ ] Extract parameter validation (colors, sizes, features) into `validation.rs`
- [ ] Group `RenderArgs` fields (lines 51-179, ~30 fields) into sub-structs: `FontOptions`, `ColorOptions`, `OutputOptions`, `ShapingOptions`
- [ ] Each extracted module should have its own unit tests

### 3.5 Replace Manual Base64 Implementation (ref: REVIEW §4.2)

- [ ] `crates/typf-export/src/svg.rs`: Replace manual `base64_encode` function with the workspace `base64` crate (already in workspace dependencies)
- [ ] Verify output is identical before and after (add a test with known input/output)

### 3.6 Fix Hardcoded Baseline in SVG Export (ref: REVIEW §4.3)

- [ ] `crates/typf-export-svg/src/lib.rs` line 109: Replace hardcoded `baseline_y = height * 0.8` with actual font metrics (ascender / (ascender - descender) * height)
- [ ] Requires passing font metrics through the export pipeline — may need to extend `RenderOutput` or export context

### 3.7 Optimize Font Data Hashing (ref: REVIEW §2.4)

- [ ] `shaping_cache.rs` line 64: Font data is re-hashed via `DefaultHasher` on every cache key creation. For large fonts (10MB+), this is ~5ms per key.
- [ ] Solution: Cache the font hash as part of the `FontRef` trait or as a lazy field on the font wrapper. Compute once, reuse across all cache key creations.

### 3.8 Reduce `ShapingCacheKey::new()` Parameter Count (ref: REVIEW §2.4)

- [ ] `shaping_cache.rs` line 52: `#[allow(clippy::too_many_arguments)]` with 8 parameters
- [ ] Refactor to accept a `&ShapingParams` struct or use a builder pattern
- [ ] Remove the `#[allow]` suppression

### 3.9 Fix Integer Overflow in `avg_access_time()` (ref: REVIEW §2.3)

- [ ] `cache.rs` line 146: `total_hits as u32` will overflow for long-running processes with billions of cache accesses
- [ ] Change to `u64` or use saturating arithmetic: `total_hits.min(u32::MAX as u64) as u32`

---

## Phase 4: Low Priority — Backlog

### 4.1 API Guidelines Compliance (ref: REVIEW §2.9)

- [ ] `context.rs`: Change `exported()` return type from `Option<&Vec<u8>>` to `Option<&[u8]>` per [Rust API Guidelines C-DEREF](https://rust-lang.github.io/api-guidelines/interoperability.html#c-deref)

### 4.2 Consolidate Dimension Error Variants (ref: REVIEW §2.7)

- [ ] `error.rs`: Consider merging `InvalidDimensions` (legacy) and `InvalidBitmapSize` into the more specific `ZeroDimensions` / `DimensionsTooLarge` / `TotalPixelsTooLarge` variants
- [ ] This is a breaking API change — defer to next major version or gate behind `#[deprecated]`

### 4.3 Remove Redundant Filtering in `effective_order()` (ref: REVIEW §2.1)

- [ ] `lib.rs` line 643: `effective_order()` filters `deny` list, but `from_parts()` already ensures denied sources are not in `prefer`. The filtering is defensive but redundant.
- [ ] Either remove the filtering (add a comment explaining why it's safe) or add a debug assertion that no denied sources are present

### 4.4 Improve Test Code Quality (ref: REVIEW §2.5)

- [ ] Replace `unreachable!()` in test code (glyph_cache.rs lines 228, 234; shaping_cache.rs line 231) with `panic!("descriptive message about what failed")`
- [ ] This improves test failure diagnostics

### 4.5 Thread-Local Storage Bounds (ref: REVIEW §5.4)

- [ ] CoreText/CoreGraphics backends use thread-local storage for OS handles
- [ ] In rayon thread-pool scenarios, this could grow unbounded
- [ ] Consider: add a `flush_thread_locals()` function or limit TLS entries with an LRU policy

---

## Phase 5: Documentation & Testing Improvements

### 5.1 Fix Stale Inline Documentation

- [ ] `shaping_cache.rs` line 89: Remove "two-level cache" wording — now single-level Moka
- [ ] Add module-level `//!` documentation to backend crates that lack it
- [ ] Add `///` doc comments to all public functions in `typf-cli/src/limits.rs`
- [ ] Add `///` doc comments to all public functions in `typf-cli/src/commands/*.rs`

### 5.2 Enable Documentation Lints

- [ ] Add `#![warn(missing_docs)]` to `typf-core`
- [ ] Add `#![warn(missing_docs)]` to `typf-cli`
- [ ] Fix all resulting warnings

### 5.3 Expand Visual Regression Suite

- [ ] Add SSIM tests for Arabic (RTL) shaping
- [ ] Add SSIM tests for Devanagari (Hindi) shaping
- [ ] Add SSIM tests for Thai shaping
- [ ] Add SSIM tests for mixed-script lines (LTR + RTL in same line)

### 5.4 Expand Fuzzing

- [ ] Create `fuzz_batch_jsonl` target for the JSONL parser
- [ ] Create `fuzz_cache_keys` target to verify cache key collision properties
- [ ] Add coverage-guided fuzzing metrics to CI dashboard

### 5.5 Unsafe Code Documentation Standardization

- [ ] Audit all `unsafe` blocks in backend crates for `// SAFETY:` comments — use `ffi.rs` as the reference standard
- [ ] Specifically: `typf-shape-ct`, `typf-render-cg`, `typf-os-mac`, `typf-os-win` — verify `// SAFETY:` comments match the quality of `ffi.rs`
- [ ] Audit `Box::from_raw` usage in `typf-shape-ct` for correct ownership transfer

---

## Implementation Notes

### Ordering Dependencies
- Phase 1.1 (doc/code mismatches) blocks nothing — can be done immediately
- Phase 2.3 (cache key consolidation) should be done before Phase 3.1 (cache naming cleanup)
- Phase 3.4 (CLI decomposition) should be done before Phase 3.3 (dead code removal) — decomposition may reveal which "dead" modules are actually needed
- Phase 3.6 (baseline fix) depends on extending the export pipeline — coordinate with rendering backend teams

### Testing Strategy
- Every change in Phases 1-3 must include or update tests
- Run full `python3 test.py` (18 tests) after each phase completion
- Run `cargo clippy --workspace` after Phase 2.1 (lint strictness increase)
- SSIM visual regression tests (Phase 5.3) need reference images generated before any rendering changes

### Risk Assessment
| Phase | Risk | Mitigation |
|-------|------|------------|
| 1.1 | Changing constants may break downstream | Check if any external code depends on these values |
| 2.1 | `deny` on `unwrap_used` may break compilation | Do a dry-run with `cargo clippy` first, fix all violations |
| 2.3 | Removing duplicate types may break internal APIs | Use `lsp_find_references` to find all usages before removing |
| 3.1 | Renaming `MultiLevelCache` is a large refactor | Use LSP rename for safety |
| 3.6 | Baseline change alters rendering output | Generate new SSIM reference images after the change |
