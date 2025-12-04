# Typf Rendering Quality & Backend Integration Plan

**Version:** 2.5.1
**Status:** Active Development
**Reference Renderer:** CoreText + CoreGraphics (linra-mac)

---

## Executive Summary

This plan addresses rendering quality issues across typf backends and completes Vello integration:

1. **Bitmap Color Font Fixes** - Fix CBDT/sbix scaling and orientation issues
2. **COLR Space Glyph Fixes** - Fix yellow square rendering for space glyphs
3. **SVG Scaling Fixes** - Fix OpenType-SVG tiny rendering on Skia/Zeno
4. **Vertical Placement Consistency** - Fix shifted "T", "y" characters
5. **Vello Backend Finalization** - Complete vello and vello-cpu integration into typf-tester

---

## Part 0: Critical Rendering Fixes (Priority: CRITICAL)

### Visual Inspection Results (Dec 4, 2025)

| Format | CoreGraphics | Opixa | Skia | Zeno | Vello-CPU | Vello-GPU |
|--------|--------------|-------|------|------|-----------|-----------|
| **CBDT** | N/A | N/A | ⚠️ Vertical shift | ⚠️ Vertical shift | ✅ Best | ❌ Nothing |
| **COLR** | N/A (outline) | N/A (outline) | ⚠️ Vertical shift | ⚠️ Same as Skia | ⚠️ "y" cut off | ❌ Nothing |
| **sbix** | ✅ Perfect | N/A (outline) | ❌ Nothing | ❌ Nothing | ⚠️ "y" shifted | ❌ Nothing |
| **SVG** | ✅ Good | ✅ Mono fallback | ❌ Tiny artifacts | ❌ Same as Skia | ✅ Mono fallback | ✅ Mono fallback |

**Legend:** ✅ Works | ⚠️ Partial/Issues | ❌ Broken | N/A Expected

---

### Problem 0.1: SVG Glyphs Render as Tiny Artifacts (CRITICAL)

**Observed Issue:**
- `render-nabla-svg-coretext-skia-latn.png` shows tiny colorful dots instead of glyphs
- `render-nabla-svg-coretext-zeno-latn.png` has identical problem
- CoreGraphics renders correctly as reference

**Visual:** Tiny scattered colored dots across the canvas instead of full glyphs.

**Root Cause:**
OpenType-SVG documents use font units (e.g., 2048 UPM), but the SVG scaling in `svg.rs` calculates scale from the SVG tree size rather than font metrics. When the SVG viewBox is 2048x2048 font units and we try to fit it into a small glyph cell, the scale becomes extremely small (e.g., 0.06), making glyphs nearly invisible.

**Correct Scaling:**
```rust
// Get font's units per em
let upem = font.head()?.units_per_em() as f32;
// Target is: font_size pixels = 1em = upem font units
// SVG document is in font units, so scale = font_size / upem
let scale = font_size / upem;
```

**Files to modify:**
- `backends/typf-render-color/src/svg.rs`

**Tasks:**
- [ ] Fix SVG scaling to use ppem/upem ratio instead of tree size ratio
- [ ] Ensure SVG viewBox is properly handled (may be in font units)
- [ ] Test with Nabla-Regular-SVG.ttf - glyphs should be full-size

---

### Problem 0.2: sbix Bitmap Fonts Not Rendering in Skia/Zeno

**Observed Issue:**
- `render-nabla-sbix-coretext-skia-latn.png` renders nothing (blank)
- `render-nabla-sbix-coretext-zeno-latn.png` renders nothing (blank)
- CoreGraphics renders perfectly as reference
- vello-cpu renders correctly (proves sbix data is valid)

**Root Cause:**
Skia and Zeno are not attempting sbix rendering at all, or the sbix path is failing silently. Since CBDT works (same bitmap concept), the issue is likely in sbix-specific detection or extraction.

**Files to modify:**
- `backends/typf-render-skia/src/lib.rs`
- `backends/typf-render-zeno/src/lib.rs`
- `backends/typf-render-color/src/bitmap.rs`

**Tasks:**
- [ ] Add debug logging to trace sbix detection in Skia/Zeno
- [ ] Verify `try_color_glyph()` checks for sbix tables
- [ ] Ensure sbix bitmap extraction path is being called
- [ ] Test with Nabla-Regular-sbix.ttf

---

### Problem 0.3: CBDT/COLR Vertical Shifting in Skia/Zeno

**Observed Issue:**
- In CBDT renders, individual glyphs shift up/down relative to each other
- In COLR renders, same vertical inconsistency between adjacent glyphs
- Skia and Zeno exhibit identical behavior (shared root cause)
- vello-cpu renders with consistent baseline

**Root Cause:**
Color glyph bearings are calculated from the rendered bitmap bounds, but the positioning calculation doesn't account for the difference between outline glyph bounds and color glyph bounds. Each glyph's bearing_y is computed independently, leading to inconsistent vertical placement.

**Solution:**
Use font-level baseline metrics instead of per-glyph computed bearings for vertical positioning. The color glyph should be positioned relative to a consistent baseline, not its individual bounds.

**Files to modify:**
- `backends/typf-render-skia/src/lib.rs`
- `backends/typf-render-zeno/src/lib.rs`

**Tasks:**
- [ ] Calculate baseline from font metrics (ascender), not glyph bounds
- [ ] Position color glyphs relative to baseline, not their individual bearing_y
- [ ] Add visual comparison test: all glyphs should sit on same baseline

---

### Problem 0.4: Vello-CPU Glyph Cutoff ("y" bottom/shift)

**Observed Issue:**
- COLR: Bottom of "y" glyph is cut off
- sbix: Final "y" glyph is shifted up relative to others
- Other glyphs render correctly

**Root Cause:**
Vello-cpu's bitmap height calculation or compositing position has an off-by-one or rounding error specifically affecting descenders (glyphs that extend below baseline).

**Files to modify:**
- `backends/typf-render-vello-cpu/src/lib.rs`

**Tasks:**
- [ ] Review descender handling in vello-cpu bitmap allocation
- [ ] Add padding for descender glyphs (g, j, p, q, y)
- [ ] Verify bearing_y calculation includes descender extent

---

### Problem 0.5: Vello-GPU Renders Nothing for Color Fonts

**Observed Issue:**
- All CBDT, COLR, sbix, SVG renders produce ~600 byte blank images
- Monochrome fonts render correctly

**Root Cause (KNOWN):**
GPU Vello (`vello_hybrid`) has stub implementations for color glyphs:
```rust
// external/vello/sparse_strips/vello_hybrid/src/scene.rs
GlyphType::Bitmap(_) => {}  // Empty stub!
GlyphType::Colr(_) => {}    // Empty stub!
```

This is an **upstream vello_hybrid limitation**, not a typf bug.

**Status:** DEFERRED - Requires upstream vello_hybrid work

**Workaround:** Use vello-cpu for color font rendering

---

## Part 1: Vertical Placement Consistency (Priority: HIGH)

### Problem Description

Different renderers place glyphs at inconsistent vertical positions:
- **CoreGraphics** (reference): Correct baseline positioning
- **Opixa/Skia/Zeno**: Glyphs shifted vertically relative to CoreGraphics

### Root Cause Analysis

Each renderer calculates `baseline_y` differently:
- CoreGraphics uses native macOS text layout metrics
- Other renderers compute from `max_y` (highest ascent) but may have off-by-one errors

### Implementation

#### Phase 1.1: Analyze Baseline Calculations

**Files to modify:**
- `backends/typf-render-opixa/src/lib.rs`
- `backends/typf-render-skia/src/lib.rs`
- `backends/typf-render-zeno/src/lib.rs`

**Tasks:**
- [ ] Add debug logging for baseline_y calculation in each renderer
- [ ] Compare glyph bounds (min_y, max_y) across renderers for same input
- [ ] Identify the delta between CoreGraphics and other renderers

#### Phase 1.2: Standardize Baseline Calculation

**Approach:** Use consistent formula across all renderers:
```rust
// Standard baseline calculation matching CoreGraphics behavior:
// 1. Get font metrics (ascender, descender) from skrifa
// 2. Scale to font_size
// 3. Position baseline at: padding + scaled_ascender
let baseline_y = padding + (font_metrics.ascender * scale);
```

**Tasks:**
- [ ] Extract font metrics (ascender, descender) via skrifa in each renderer
- [ ] Replace per-glyph bounds-based baseline with font-metrics-based baseline
- [ ] Add tests comparing output positions across renderers

---

## Part 2: Color Font Rendering Fixes (Priority: HIGH)

### Problem Description

Nabla COLR font via Skia and Zeno shows:
1. **Vertically flipped glyphs** - Glyphs appear upside down
2. **Black squares for spaces** - Empty glyphs render as black rectangles

### Root Cause Analysis

The `typf-render-color` module renders COLR glyphs in font coordinate space (Y-up). When Skia/Zeno composite these into their canvas (Y-down), the Y-flip is not applied.

**Coordinate Systems:**
- Font coordinates: Y increases upward (origin at baseline)
- Bitmap coordinates: Y increases downward (origin at top-left)
- `typf-render-color` outputs in font coordinates
- Skia/Zeno expect Y-down for compositing

### Implementation

#### Phase 2.1: Fix COLR Y-Flip

**Files to modify:**
- `backends/typf-render-skia/src/lib.rs` (`try_color_glyph` and compositing)
- `backends/typf-render-zeno/src/lib.rs` (`try_color_glyph` and compositing)

**Solution:** Apply vertical flip to color glyph bitmaps before compositing:
```rust
// After receiving color bitmap from typf-render-color:
fn flip_vertical(data: &mut [u8], width: u32, height: u32) {
    for y in 0..(height / 2) {
        let top_row = y as usize * (width * 4) as usize;
        let bottom_row = (height - 1 - y) as usize * (width * 4) as usize;
        for x in 0..(width * 4) as usize {
            data.swap(top_row + x, bottom_row + x);
        }
    }
}
```

**Tasks:**
- [x] Add vertical flip to Skia's `RgbaPremul` compositing path
- [x] Add vertical flip to Zeno's `RgbaPremul` compositing path
- [x] Add tests with Nabla-Regular-COLR.ttf

#### Phase 2.2: Fix Black Square Spaces

**Root cause:** Empty glyphs (spaces) have zero-dimension bitmaps but the fallback code creates 1x1 black pixels.

**Solution:** Handle empty color glyphs gracefully:
```rust
// In try_color_glyph:
if pixmap.data().iter().all(|&b| b == 0) {
    // Empty color glyph - skip, don't composite black
    return Ok(None);
}
```

**Tasks:**
- [x] Check for fully-transparent color glyphs and return None
- [x] Ensure space glyphs fall through to outline path (which correctly returns empty bitmap)

#### Phase 2.3: OpenType-SVG Rendering ✓ COMPLETED

**Files modified:**
- `backends/typf-render-color/src/svg.rs`
- `backends/typf-render-color/src/lib.rs`
- `backends/typf-render-skia/src/lib.rs`
- `backends/typf-render-zeno/src/lib.rs`

**Completed Tasks:**
- [x] Implemented CSS variable substitution (`substitute_css_variables()`) for OpenType-SVG
  - Replaces `var(--colorN, fallback)` with CPAL palette colors
  - Supports color indices 0-15
- [x] Added `render_svg_glyph_with_palette()` function to pass palette colors to SVG renderer
- [x] Updated render pipeline to pass CPAL colors to SVG glyph rendering
- [x] Fixed Skia/Zeno compositing formula bug (incorrect division by 255 was zeroing colors)
- [x] Added unit tests for CSS variable substitution
- [x] Verified with Nabla-Regular-SVG.ttf - colors now display correctly

---

## Part 3: Zeno Glyph Precision Fixes (Priority: MEDIUM)

### Problem Description

Zeno renderer cuts off the bottom 1-2 pixels of glyphs and shifts them down slightly.

Comparison:
- TinySkia: Glyphs fully visible, correct position
- Zeno: Bottom of S, a, o, e cut off, shifted down

### Root Cause Analysis

Zeno's bounding box calculation or bitmap placement has an off-by-one error:
1. The `bearing_y` calculation may be ceiling/flooring incorrectly
2. The Y position calculation in compositing may have rounding errors
3. The vertical flip loop may miss the last row

### Implementation

#### Phase 3.1: Audit Zeno Positioning

**File:** `backends/typf-render-zeno/src/lib.rs`

**Investigation:**
```rust
// Current code at line 227:
bearing_y: max_y as i32,  // Should this be ceil()?

// Compositing at line 496:
let y = (baseline_y + rg.glyph_y) as i32 - bitmap.bearing_y;
```

**Tasks:**
- [x] Add +1 pixel padding to height calculation: `height = ((max_y - min_y).ceil() as u32).max(1) + 1`
- [x] Review bearing_y calculation - use `max_y.ceil() as i32`
- [ ] Compare canvas dimensions with Skia for same input
- [ ] Add visual regression test comparing Zeno vs Skia output

#### Phase 3.2: Fix Vertical Flip

**Current code (lines 209-217):**
```rust
for y in 0..(height / 2) {
    // This misses middle row for odd heights
}
```

**Fix:** Ensure complete flip including middle row handling.

---

## Part 4: Vello Backend Finalization (Priority: HIGH)

### Current Status

- **typf-render-vello-cpu**: ✓ Implemented, 16 tests passing
- **typf-render-vello**: ✓ Implemented, 15 tests passing
- **typf-tester integration**: ✓ COMPLETE

### Implementation

#### Phase 4.1: Add Vello to typf-tester ✓ COMPLETE

**File:** `typf-tester/typfme.py`

**Completed Tasks:**
- [x] Add "vello" and "vello-cpu" to renderer list in `_detect_available_backends()`
- [x] Add vello features to Python bindings `pyproject.toml`
- [x] Test Vello renderers with all font types (Latin, Arabic, Variable, Color)
- [x] Vello renderings now appear in typf-tester output

**Note:** GPU `vello` renders color fonts as ~600 byte blank images while `vello-cpu` renders them correctly (33KB). This suggests GPU path needs color font support work.

#### Phase 4.2: Verify Vello Color Font Support

**Files:**
- `backends/typf-render-vello-cpu/src/lib.rs`
- `backends/typf-render-vello/src/lib.rs`

**Tasks:**
- [ ] Verify COLR rendering works via vello's native color font support
- [ ] Compare output quality with CoreGraphics reference
- [ ] Add color font tests to Vello test suites

#### Phase 4.3: Performance Benchmarks ✓ COMPLETE

**Benchmark Results (Dec 4, 2025, 50 iterations):**

| Renderer | Avg Time (ms) | Ops/sec | Notes |
|----------|--------------|---------|-------|
| JSON | 0.05 | 20,800 | Fastest (no rasterization) |
| CoreGraphics | 0.38 | 3,700 | macOS native, excellent |
| Zeno | 0.76 | 1,880 | Pure Rust, good |
| Opixa | 1.02 | 2,540 | Pure Rust, SIMD |
| Skia | 1.05 | 1,600 | tiny-skia |
| **Vello-CPU** | **2.20** | **995** | High-quality 256-level AA |
| **Vello-GPU** | **11.5** | **87** | GPU overhead for small text |

**Key Findings:**
- Vello-CPU: ~2ms/render, good quality, ~2x slower than Opixa but 256-level AA
- Vello-GPU: ~12ms/render, significant overhead - best for large batch/GPU workloads
- GPU vello has high per-render overhead; not suitable for single small text renders
- For typical use, prefer vello-cpu over vello (GPU) for small text operations

---

## Testing Strategy

### Visual Regression Tests

Create reference images from CoreGraphics and compare:
```bash
# Generate reference
typf render --shaper coretext --renderer coregraphics "Test" -o ref.png

# Generate test output
typf render --shaper coretext --renderer zeno "Test" -o test.png

# Compare (using image diff tool)
compare ref.png test.png diff.png
```

### Unit Tests

- Baseline position consistency across renderers
- Color glyph Y-flip correctness
- Empty glyph handling (no black squares)
- Glyph bounds accuracy

### Integration Tests

- Full pipeline with all font types
- All shaper × renderer combinations
- Color font rendering (COLR, SVG, sbix, CBDT)

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Vertical placement | ±1 pixel vs CoreGraphics baseline |
| COLR rendering | Correct orientation, no black squares |
| Zeno precision | No bottom cutoff, aligned with Skia |
| Vello integration | Full typf-tester support |

---

## Implementation Order

1. **Phase 2.1**: Fix COLR Y-flip (most visible issue)
2. **Phase 2.2**: Fix black square spaces
3. **Phase 3.1-3.2**: Fix Zeno precision
4. **Phase 1.1-1.2**: Standardize vertical placement
5. **Phase 4.1-4.3**: Complete Vello integration

---

## References

- CoreGraphics rendering: `backends/typf-render-cg/src/lib.rs`
- Skia rendering: `backends/typf-render-skia/src/lib.rs`
- Zeno rendering: `backends/typf-render-zeno/src/lib.rs`
- Color rendering: `backends/typf-render-color/src/lib.rs`
- Vello CPU: `backends/typf-render-vello-cpu/src/lib.rs`
- Vello GPU: `backends/typf-render-vello/src/lib.rs`
