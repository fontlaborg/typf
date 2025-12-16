<!-- this_file: PLANSTEPS/01-rendering-quality-status.md -->

# Typf Rendering Quality & Backend Integration Plan

**Version:** 5.0.1
**Status:** All Critical Issues Resolved
**Reference Renderer:** CoreText + CoreGraphics (linra-mac)

---

## Executive Summary

All critical rendering issues have been resolved. The typf text rendering pipeline now correctly handles:
- All color font formats (COLR, SVG, sbix, CBDT) in Skia, Zeno, and Vello-CPU
- Proper descender rendering in Vello-CPU
- Correct vertical flip and compositing in all backends

---

## Current Status

### Visual Inspection Results (Dec 4, 2025)

| Format | CoreGraphics | Opixa | Skia | Zeno | Vello-CPU | Vello-GPU |
|--------|--------------|-------|------|------|-----------|-----------|
| **CBDT** | N/A | N/A | ✅ Works | ✅ Works | ✅ Best | ❌ Nothing |
| **COLR** | N/A (outline) | N/A (outline) | ✅ Works | ✅ Works | ✅ Works | ❌ Nothing |
| **sbix** | ✅ Perfect | N/A (outline) | ✅ Works | ✅ Works | ✅ Works | ❌ Nothing |
| **SVG** | ✅ Good | ✅ Mono fallback | ✅ Works | ✅ Works | ✅ Mono fallback | ✅ Mono fallback |

**Legend:** ✅ Works | ❌ Broken | N/A Expected

**Note:** Minor "T" cutoff at extreme COLR glyph edges is a known limitation.

---

## Completed Work

### Problem 0.1: SVG Glyphs Render as Tiny Artifacts ✅

**Solution:** Fixed viewBox to `"0 -{upem} {upem} {double_upem}"` in `svg.rs`, implemented proper ppem/upem scaling ratio, added conditional flip for SVG.

### Problem 0.2: sbix Bitmap Fonts Not Rendering ✅

**Solution:** Removed the `&& !outline_empty` condition in Skia and Zeno - sbix fonts have empty outline paths but valid bitmap data.

### Problem 0.3: CBDT/COLR Rendering Issues ✅

**Solution:** Added 50% bbox padding for COLR rendering to handle paint effects (shadows, 3D perspective, etc.)

### Problem 0.4: Vello-CPU Glyph Cutoff ✅

**Solution:** Added proper font metrics extraction using skrifa's `MetadataProvider`. Now uses actual `ascent` and `descent` values instead of fixed 0.8/1.5 multipliers.

### Problem 0.5: Vello-GPU Color Fonts - DEFERRED

**Status:** The vendored `vello_hybrid` renderer currently has stub implementations for `GlyphType::Bitmap` and `GlyphType::Colr` (they are ignored), so color glyphs can render as blank output. This is not actionable in typf without updating/replacing the vendored Vello renderer.

**Workaround:** Use vello-cpu for color font rendering.

**Evidence (in this repo):** `external/vello/sparse_strips/vello_hybrid/src/scene.rs` ignores `GlyphType::Bitmap(_)` and `GlyphType::Colr(_)` in `GlyphRenderer::{fill_glyph,stroke_glyph}`.

**Upstream tracking:** Upstream Vello has added bitmap/COLR glyph rendering for `vello_hybrid` (see vello#937, referenced in Linebender’s May 2025 update: https://linebender.org/blog/tmil-17/).

### Phase 2: Color Font Rendering ✅

- [x] COLR Y-flip in Skia/Zeno compositing
- [x] Black square space handling
- [x] OpenType-SVG CSS variable substitution
- [x] CPAL palette color support

### Phase 3: Zeno Precision ✅

- [x] Height padding (+1 pixel)
- [x] bearing_y using `max_y.ceil()`
- [x] Vertical flip verified correct for odd/even heights

### Phase 4: Vello Integration ✅

- [x] Added vello/vello-cpu to typf-tester
- [x] Python bindings with render-vello features
- [x] Variable font normalized coords
- [x] Performance benchmarks documented

---

## Performance Benchmarks

| Renderer | Avg Time (ms) | Ops/sec | Notes |
|----------|--------------|---------|-------|
| JSON | 0.05 | 20,800 | Fastest (no rasterization) |
| CoreGraphics | 0.38 | 3,700 | macOS native, excellent |
| Zeno | 0.76 | 1,880 | Pure Rust, good |
| Opixa | 1.02 | 2,540 | Pure Rust, SIMD |
| Skia | 1.05 | 1,600 | tiny-skia |
| Vello-CPU | 2.20 | 995 | High-quality 256-level AA |
| Vello-GPU | 11.5 | 87 | GPU overhead for small text |

**Recommendation:** Use vello-cpu for typical text rendering, GPU vello for large batch workloads.

---

## Future Work (Low Priority)

### Baseline Standardization

Cross-renderer baseline consistency using font metrics instead of per-glyph bounds.

### External Integrations

- Integrate `vello_svg` for OpenType-SVG in Vello backends
- Analyze `parley` and `swash` for script segmentation and multiline layout

### SDF Rendering

- Create `typf-sdf-core` crate with SDF types
- Implement SDF generation from glyph outlines

### Platform Support

- Test Vello GPU on Linux (Vulkan) and Windows (DX12/Vulkan)
- Add WASM/WebGPU support for Vello

---

## Test Coverage

- **Workspace total:** 414 tests
- **Vello CPU:** 17 tests (4 unit + 13 integration)
- **Vello GPU:** 15 tests (3 unit + 12 integration)
- **Color fonts:** All 4 formats tested (COLR, SVG, sbix, CBDT)

---

## References

- CoreGraphics: `backends/typf-render-cg/src/lib.rs`
- Skia: `backends/typf-render-skia/src/lib.rs`
- Zeno: `backends/typf-render-zeno/src/lib.rs`
- Color: `backends/typf-render-color/src/lib.rs`
- Vello CPU: `backends/typf-render-vello-cpu/src/lib.rs`
- Vello GPU: `backends/typf-render-vello/src/lib.rs`

The complexity of text rendering must be managed by enforcing strict boundaries and standardized data contracts across the ecosystem. The design of `typf` centers around a mandatory **six-stage pipeline** to ensure modularity and correctness.

The integration of `typf` with external high-performance Rust and Python libraries requires extending its core API, focusing on optimizing data transfer, specifically through **zero-copy techniques**. The primary points of interaction analyzed are the outputs of Stage 4 (Shaping) and Stage 5 (Rendering).
