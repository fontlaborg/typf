# TODO: Typf Rendering Quality & Backend Integration

**Version:** 2.5.0
**Updated:** Dec 11, 2025
**Reference:** See PLAN.md for detailed specifications

---

## 1. Current Status Summary

### Visual Inspection (Dec 4, 2025)

| Format | CoreGraphics | Opixa | Skia | Zeno | Vello-CPU | Vello-GPU |
|--------|--------------|-------|------|------|-----------|-----------|
| **CBDT** | N/A | N/A | ✅ | ✅ | ✅ Best | ❌ Upstream |
| **COLR** | N/A | N/A | ✅ | ✅ | ✅ | ❌ Upstream |
| **sbix** | ✅ | N/A | ✅ | ✅ | ✅ | ❌ Upstream |
| **SVG** | ✅ | ✅ Mono | ✅ | ✅ | ✅ Mono | ✅ Mono |

**Note:** Minor "T" cutoff at extreme COLR glyph edges is a known limitation.

---

## 2. Active Tasks

### 2.1. [-] Vello-GPU Color Fonts - DEFERRED (Upstream)

**Problem:** All color fonts render blank (~600 byte images)
**Root Cause:** `vello_hybrid` has stub implementations for Bitmap/COLR glyph types
**Status:** Requires upstream vello_hybrid work - not actionable in typf
**Workaround:** Use vello-cpu for color font rendering

---

## 3. Future Tasks

### 3.1. Baseline Standardization (Priority: LOW)

Cross-renderer baseline consistency improvements:
- [ ] Analyze baseline_y calculations across Opixa/Skia/Zeno
- [ ] Compare against CoreGraphics reference
- [ ] Standardize using font metrics if needed

### 3.2. External Integrations (Priority: LOW)

- [ ] Integrate `vello_svg` for OpenType-SVG in Vello backends
- [ ] Analyze `parley` and `swash` for script segmentation and multiline layout
- [ ] Enhance Python bindings with layout capabilities

### 3.3. SDF Rendering (Priority: LOW)

- [ ] Create `typf-sdf-core` crate with SDF types
- [ ] Implement SDF generation from glyph outlines
- [ ] Create `typf-render-sdf` CPU renderer

### 3.4. Platform Support (Priority: LOW)

- [ ] Test Vello GPU on Linux (Vulkan)
- [ ] Test Vello GPU on Windows (DX12/Vulkan)
- [ ] Add WASM/WebGPU support for Vello

---

## 4. Completed Work (Summary)

All critical rendering issues have been resolved:

### Color Font Fixes ✓
- **SVG scaling** - Fixed viewBox handling in `svg.rs`
- **sbix rendering** - Removed `!outline_empty` check in Skia/Zeno
- **CBDT/COLR** - Added 50% bbox padding for paint effects
- **CSS variables** - Implemented CPAL palette substitution for OpenType-SVG
- **Y-flip** - Added vertical flip in Skia/Zeno compositing
- **Black squares** - Fixed empty glyph handling (spaces)

### Vello Integration ✓
- **typf-tester** - Added vello/vello-cpu to renderer detection
- **Python bindings** - Added render-vello features to pyproject.toml
- **Variable fonts** - Fixed normalized coords in vello-cpu
- **Descenders** - Fixed "y" cutoff using proper font metrics
- **Benchmarks** - Documented performance (vello-cpu: 995 ops/sec)

### Zeno Precision ✓
- **Height padding** - Added +1 pixel to height calculation
- **bearing_y** - Using `max_y.ceil()` for proper alignment
- **Vertical flip** - Verified correct for odd/even heights

### Cache System ✓
- **Default disabled** - Caching now opt-in via `set_caching_enabled(true)` or `TYPF_CACHE=1`
- **Pipeline policy** - `CachePolicy` defaults to `false` for both shaping and glyph caches
- **Python bindings** - `typf.set_caching_enabled()` and `typf.is_caching_enabled()` exposed
- **Clippy clean** - Fixed derivable impl, Arc lint, and expect() warnings

### Documentation ✓
- **README.md** - Added caching section with Rust/Python/env var examples
- **QUICKSTART.md** - Created comprehensive Rust usage guide with caching docs

---

## 5. Performance Reference

| Renderer | Ops/sec | Notes |
|----------|---------|-------|
| JSON | 20,800 | No rasterization |
| CoreGraphics | 3,700 | macOS native |
| Zeno | 1,880 | Pure Rust |
| Opixa | 2,540 | Pure Rust, SIMD |
| Skia | 1,600 | tiny-skia |
| Vello-CPU | 995 | 256-level AA |
| Vello-GPU | 87 | High overhead |

---

## 6. Test Coverage

- **Workspace total:** 410 tests
- **Vello CPU:** 17 tests (4 unit + 13 integration)
- **Vello GPU:** 15 tests (3 unit + 12 integration)
- **Color fonts:** All 4 formats tested (COLR, SVG, sbix, CBDT)
