# TODO: Rendering Quality & Backend Integration

**Version:** 2.5.2
**Updated:** Dec 4, 2025
**Reference:** See PLAN.md for detailed specifications

---

## 1. Visual Inspection Summary (Dec 4, 2025)

| Format | CoreGraphics | Opixa | Skia | Zeno | Vello-CPU | Vello-GPU |
|--------|--------------|-------|------|------|-----------|-----------|
| **CBDT** | N/A | N/A | ⚠️ Shift | ⚠️ Shift | ✅ Best | ❌ Blank |
| **COLR** | N/A | N/A | ⚠️ Shift | ⚠️ Shift | ⚠️ "y" cut | ❌ Blank |
| **sbix** | ✅ Perfect | N/A | ❌ Blank | ❌ Blank | ⚠️ "y" shift | ❌ Blank |
| **SVG** | ✅ Good | ✅ Mono | ❌ Tiny | ❌ Tiny | ✅ Mono | ✅ Mono |

---

## 2. Linear Work Queue (Priority Order)

### 2.1. [!] SVG Scaling Fix (Skia/Zeno) - CRITICAL

**Problem:** SVG glyphs render as tiny colorful dots instead of full glyphs
**File:** `backends/typf-render-color/src/svg.rs`

- [ ] 1.1 Debug current SVG scale calculation
- [ ] 1.2 Fix scaling to use ppem/upem ratio instead of tree_size ratio
- [ ] 1.3 Handle SVG viewBox in font units correctly
- [ ] 1.4 Test with Nabla-Regular-SVG.ttf - verify full-size glyphs

### 2.2. [!] sbix Rendering Missing (Skia/Zeno) - HIGH

**Problem:** sbix fonts render as blank (nothing) in Skia/Zeno
**Files:** `backends/typf-render-skia/src/lib.rs`, `backends/typf-render-zeno/src/lib.rs`

- [ ] 2.1 Add debug logging to trace sbix table detection
- [ ] 2.2 Verify `try_color_glyph()` checks for sbix tables (not just CBDT/COLR)
- [ ] 2.3 Fix sbix bitmap extraction path if not being called
- [ ] 2.4 Test with Nabla-Regular-sbix.ttf - verify bitmap rendering

### 2.3. [!] CBDT/COLR Vertical Shifting (Skia/Zeno) - HIGH

**Problem:** Color glyphs shift up/down relative to each other (inconsistent baseline)
**Files:** `backends/typf-render-skia/src/lib.rs`, `backends/typf-render-zeno/src/lib.rs`

- [ ] 3.1 Analyze current bearing_y calculation for color glyphs
- [ ] 3.2 Calculate baseline from font metrics (ascender), not per-glyph bounds
- [ ] 3.3 Position color glyphs relative to consistent baseline
- [ ] 3.4 Test CBDT and COLR with Nabla fonts - verify aligned baseline

### 2.4. [ ] Vello-CPU "y" Cutoff/Shift - MEDIUM

**Problem:** "y" glyph bottom cut off (COLR) or shifted up (sbix)
**File:** `backends/typf-render-vello-cpu/src/lib.rs`

- [ ] 4.1 Review descender handling in bitmap allocation
- [ ] 4.2 Add padding for descender glyphs (g, j, p, q, y)
- [ ] 4.3 Verify bearing_y includes full descender extent
- [ ] 4.4 Test with descender-heavy text ("gypsy jumping")

### 2.5. [-] Vello-GPU Color Fonts - DEFERRED (Upstream)

**Problem:** All color fonts render blank (~600 byte images)
**Root Cause:** vello_hybrid has stub implementations for Bitmap/COLR glyph types
**Status:** Requires upstream vello_hybrid changes - not actionable in typf

---

## 3. Completed Tasks ✓

### 3.1. CBDT/sbix Bitmap Scaling ✓
- [x] Added `render_bitmap_glyph_scaled()` with bilinear interpolation
- [x] Added `scale_pixmap_bilinear()` and `flip_pixmap_vertical()` helpers
- [x] Applied proper scale factor to bearings
- [x] Tested with Nabla-Regular-CBDT.ttf and sbix.ttf

### 3.2. COLR Space Glyphs (Yellow Squares) ✓
- [x] Skip COLR rendering when outline is empty (space characters)
- [x] Modified Skia and Zeno with `!outline_empty` condition

### 3.3. OpenType-SVG CSS Variables ✓
- [x] Implemented CSS variable substitution from CPAL palette
- [x] Added `render_svg_glyph_with_palette_and_ppem()` function
- [x] Fixed Skia/Zeno compositing formula bug

### 3.4. Vello typf-tester Integration ✓
- [x] Added vello features to Python bindings pyproject.toml
- [x] Added "vello" and "vello-cpu" to renderer detection
- [x] Benchmarked performance (vello-cpu: 995 ops/sec, vello: 87 ops/sec)

---

## 4. Phase 2.1: Fix COLR Y-Flip (Priority: HIGH) ✓

- [x] Add `flip_vertical()` function to `typf-render-skia/src/lib.rs`
- [x] Apply vertical flip in Skia's `RgbaPremul` compositing path
- [x] Add `flip_vertical()` function to `typf-render-zeno/src/lib.rs`
- [x] Apply vertical flip in Zeno's `RgbaPremul` compositing path
- [x] Add tests with Nabla-Regular-COLR.ttf

---

## 5. Phase 2.2: Fix Black Square Spaces (Priority: HIGH) ✓

- [x] Check for fully-transparent color glyphs in `try_color_glyph()`
- [x] Return `Ok(None)` for empty color glyphs (skip compositing)
- [x] Ensure space glyphs fall through to outline path
- [x] Test with Nabla font space characters

---

## 6. Phase 2.3: OpenType-SVG Rendering (Priority: MEDIUM) ✓

- [x] Verify SVG glyph extraction in `typf-render-color/src/svg.rs` - Works
- [x] Test with Nabla-Regular-SVG.ttf - Renders but colors wrong (black)
- [x] Implement CSS variable substitution from CPAL palette
- [ ] Add SVG color glyph tests to typf-tester
- [ ] Fix SVG scaling issue (see Part 0.3)

---

## 7. Phase 3.1: Audit Zeno Positioning (Priority: MEDIUM) ✓

- [x] Add debug logging for bearing_y calculation
- [x] Review `bearing_y: max_y as i32` - should use `max_y.ceil() as i32`
- [x] Add +1 pixel padding to height calculation
- [ ] Compare canvas dimensions with Skia for same input
- [ ] Add visual regression test comparing Zeno vs Skia output

---

## 8. Phase 3.2: Fix Zeno Vertical Flip (Priority: MEDIUM)

- [ ] Review vertical flip loop at lines 209-217
- [ ] Ensure complete flip including middle row for odd heights
- [ ] Test with various glyph heights (even and odd)

---

## 9. Phase 1.1: Analyze Baseline Calculations (Priority: HIGH)

- [ ] Add debug logging for baseline_y in Opixa renderer
- [ ] Add debug logging for baseline_y in Skia renderer
- [ ] Add debug logging for baseline_y in Zeno renderer
- [ ] Compare glyph bounds (min_y, max_y) across renderers
- [ ] Identify delta between CoreGraphics and other renderers

---

## 10. Phase 1.2: Standardize Baseline Calculation (Priority: HIGH)

- [ ] Extract font metrics (ascender, descender) via skrifa in Opixa
- [ ] Extract font metrics (ascender, descender) via skrifa in Skia
- [ ] Extract font metrics (ascender, descender) via skrifa in Zeno
- [ ] Replace per-glyph bounds-based baseline with font-metrics-based
- [ ] Add tests comparing output positions across renderers

---

## 11. Phase 4.1: Add Vello to typf-tester (Priority: HIGH) ✓

- [x] Add "vello" to renderer list in `_detect_available_backends()`
- [x] Add "vello-cpu" to renderer list in `_detect_available_backends()`
- [x] Test Vello renderers with Latin fonts
- [x] Test Vello renderers with Arabic fonts (RTL) - Works correctly
- [x] Test Vello renderers with variable fonts - Fixed ✓
- [x] Test Vello renderers with color fonts (COLR, sbix, CBDT) - All work correctly
- [x] Add vello features to Python bindings pyproject.toml

**Bug Found & Fixed:** Vello CPU renderer was ignoring `--instance` / variable font coordinates.
Added `build_normalized_coords()` helper and `.normalized_coords()` call to glyph run builder.
See commit for `typf-render-vello-cpu/src/lib.rs` changes.

**Python Bindings Fix (Dec 4):** Added `render-vello-cpu` and `render-vello` to pyproject.toml features list.
Without these, vello renderers were compiled as dependencies but not enabled via cfg flags.

---

## 12. Phase 4.2: Verify Vello Color Font Support (Priority: MEDIUM)

- [x] Verify COLR rendering via vello's native color font support
- [ ] Compare output quality with CoreGraphics reference
- [ ] Add color font tests to Vello test suites

**Finding (Dec 4):** GPU Vello (`vello_hybrid`) has stub implementations for color glyphs:
```rust
// external/vello/sparse_strips/vello_hybrid/src/scene.rs
GlyphType::Bitmap(_) => {}  // Empty!
GlyphType::Colr(_) => {}    // Empty!
```

CPU Vello (`vello_cpu`) has full COLR/bitmap support via `ColrPainter` and bitmap rendering.
This is an upstream vello_hybrid limitation, not a typf bug.

---

## 13. Phase 4.3: Performance Benchmarks (Priority: LOW) ✓

- [x] Run `typfme.py bench` with Vello renderers
- [x] Compare Vello GPU vs CPU vs Opixa performance
- [x] Document performance characteristics in PLAN.md

**Results Summary:**
- Vello-CPU: ~995 ops/sec (2.2ms avg) - 256-level AA, high quality
- Vello-GPU: ~87 ops/sec (11.5ms avg) - high overhead, best for batch/GPU workloads
- Recommendation: Use vello-cpu for typical text rendering, GPU for large batches

---

## 14. Deferred / Future Tasks

### 14.1. SDF Rendering (Priority: LOW)
- [ ] Create `typf-sdf-core` crate with SDF types
- [ ] Implement SDF generation from glyph outlines
- [ ] Create `typf-render-sdf` CPU renderer

### 14.2. Platform Support
- [ ] Test Vello GPU on Linux (Vulkan)
- [ ] Test Vello GPU on Windows (DX12/Vulkan)
- [ ] Add WASM/WebGPU support for Vello

---

## 15. Notes

### 15.1. Implementation Order
1. Phase 2.1: Fix COLR Y-flip (most visible issue)
2. Phase 2.2: Fix black square spaces
3. Phase 3.1-3.2: Fix Zeno precision
4. Phase 1.1-1.2: Standardize vertical placement
5. Phase 4.1-4.3: Complete Vello integration

### 15.2. Success Criteria
- Vertical placement: ±1 pixel vs CoreGraphics baseline
- COLR rendering: Correct orientation, no black squares
- Zeno precision: No bottom cutoff, aligned with Skia
- Vello integration: Full typf-tester support

### 15.3. Testing
- Vello CPU: 16 tests (4 unit + 12 integration)
- Vello GPU: 15 tests (3 unit + 12 integration)
- Total workspace: 380 tests (verified Dec 4, 2025)


## 16. svg

- [ ] integrate @./external/vello_svg to render OpenType-SVG in Vello + Vello-CPU
- [ ] analyze @./external/parley and @./external/swash and integrate into typf (for better script segmentation and multiline layout). Especially interesting for Python bindings. 
