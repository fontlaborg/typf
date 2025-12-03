# Benchmark Analysis

Comprehensive performance analysis based on extensive testing across different scripts, fonts, and backend combinations.

## Executive Summary

The benchmark reveals three categories of issues:
1. **CBDT bitmap font rendering failures** across multiple backends
2. **SVG export errors** for bitmap color fonts  
3. **Performance regressions** detected in several backend combinations

## Understanding the Metrics

### ns/op (Nanoseconds per Operation)

**ns/op** = nanoseconds per operation - the standard benchmark metric measuring how long a single rendering operation takes. Lower is better.

| Value | Human-readable | Assessment |
|-------|----------------|------------|
| 100 ns/op | 0.1ms | Excellent |
| 1,000 ns/op | 1ms | Good |
| 10,000 ns/op | 10ms | Acceptable |
| 100,000 ns/op | 100ms | Slow |
| 1,000,000+ ns/op | 1s+ | Problem |

### Ops/sec (Operations per Second)

Operations per second = `1,000,000,000 / ns_per_op`

Higher is better. This represents throughput.

## Backend Performance Rankings

### Shaping Performance (JSON output isolates shaping cost)

| Shaper | Avg Time | Ops/sec | Notes |
|--------|----------|---------|-------|
| none | 0.041ms | 24,673 | Fastest (no-op shaper) |
| HarfBuzz | 0.050ms | 20,630 | Best real shaper |
| ICU-HarfBuzz | 0.061ms | 17,368 | Unicode-accurate |
| CoreText | 0.065ms | 22,799 | macOS native |

**Winner**: HarfBuzz provides the best balance of speed and correctness.

### Rendering Performance (shaping + rendering)

| Backend Combo | Avg Time | Ops/sec | Notes |
|---------------|----------|---------|-------|
| none + coregraphics | 0.353ms | 5,045 | Fastest rasterizer |
| HarfBuzz + coregraphics | 0.367ms | 4,487 | Best real combo |
| none + opixa | 1.125ms | 2,538 | Pure Rust |
| HarfBuzz + opixa | 1.021ms | 2,584 | Good cross-platform |
| coretext + coregraphics | 0.956ms | 1,866 | macOS native |
| HarfBuzz + skia | 1.762ms | 913 | Color support |
| HarfBuzz + zeno | 1.738ms | 785 | Pure Rust alt |
| coretext + skia | 4.471ms | 349 | Slowest |

**Winner**: HarfBuzz + CoreGraphics for macOS; HarfBuzz + Opixa for cross-platform.

### Linra (Single-pass) Performance

linra-mac provides single-pass shape+render:
- Simple fonts: 100-500ns/op (excellent)
- Variable fonts: ~1.2ms/op
- Color fonts: 1.6-1.7ms/op (SVG parsing overhead)

## Error Analysis

### Error 1: CBDT Font Failures

**Affected Font**: `Nabla-Regular-CBDT.ttf` (bitmap color font)

**Symptoms**:
```
[CoreText] "Failed to create CGFont from data"
[CoreGraphics] "CoreGraphics rejected our font data"
[Skia/Zeno] "Rendering failed: Glyph not found: 93"
[SVG Export] "Glyph 37 not found"
```

**Root Cause**: CBDT (Color Bitmap Data Table) is a Google/Android format. Apple's CoreText/CoreGraphics don't support it natively. The bitmap glyph data isn't accessible via standard outline APIs.

**Impact**: CBDT fonts only work with:
- HarfBuzz shaping (works)
- Opixa rendering (works, but blank glyphs)
- Cannot export to SVG (no outlines)

### Error 2: SVG Export "Glyph not found"

**Pattern**: Consistent failure for CBDT font across all renderers when exporting to SVG.

**Root Cause**: The SVG exporter (`typf-export-svg`) tries to look up glyph outlines to generate SVG path data. CBDT fonts contain bitmap data, not outlines. The glyph IDs (37, 93, etc.) exist but have no outline data.

**Technical Detail**: The exporter calls `font.outline(glyph_id)` which returns `None` for bitmap glyphs.

### Error 3: Performance Regressions

**Detected Regressions (>10% slowdown from baseline)**:

| Backend | Script | Size | Slowdown |
|---------|--------|------|----------|
| coretext + JSON | arab | 128px | **+786%** |
| coretext + JSON | latn | 64px | +136% |
| HarfBuzz + zeno | mixd | 64px | +129% |
| HarfBuzz + zeno | mixd | 128px | +101% |
| HarfBuzz + zeno | mixd | 16px | +94% |
| HarfBuzz + zeno | arab | 128px | +39% |

**Possible Causes**:
1. CoreText JSON regression may be measurement noise (sub-millisecond variations)
2. Zeno regressions suggest a real performance issue in the zeno backend
3. Mixed script handling overhead increased

## Successful Combinations

The following backend combinations work correctly for all test fonts:

| Shaper | Renderer | COLR | SVG | sbix | CBDT | Notes |
|--------|----------|------|-----|------|------|-------|
| HarfBuzz | opixa | ✓ | ✓ | ✓ | ⚠️ | CBDT blank |
| HarfBuzz | skia | ✓ | ✓ | ✓ | ✗ | CBDT fails |
| HarfBuzz | zeno | ✓ | ✓ | ✓ | ✗ | CBDT fails |
| HarfBuzz | coregraphics | ✓ | ✓ | ✓ | ✗ | CBDT fails |
| CoreText | opixa | ✓ | ✓ | ✓ | ✗ | CBDT can't shape |
| CoreText | coregraphics | ✓ | ✓ | ✓ | ✗ | CBDT can't shape |

**Key Finding**: CBDT fonts are only partially supported. All other color font formats (COLR, SVG table, sbix) work correctly.

## Performance Recommendations

### Short-term (Bug Fixes)

1. **Add CBDT glyph type detection** - Before attempting SVG export, check if glyph is bitmap-only
2. **Graceful degradation** - Return placeholder or skip glyph instead of hard error
3. **Better error messages** - "CBDT bitmap glyphs cannot be exported as SVG paths"

### Medium-term (Feature Gaps)

1. **Add CBDT rendering support** to skia/zeno backends using skrifa's bitmap APIs
2. **Investigate zeno performance regression** - Profile the backend
3. **Add CBDT → PNG embedding** in SVG export (base64 images)

### Long-term (Architecture)

1. **Glyph type awareness** - Track glyph source (outline, COLR, SVG, bitmap) throughout pipeline
2. **Capability matrix** - Backends declare what glyph types they support
3. **Automatic fallback** - If preferred renderer can't handle glyph type, fall back

## Conclusion

The typf pipeline handles 4 out of 5 color font formats correctly (COLR, SVG table, sbix, and standard outlines). CBDT bitmap fonts have limited support due to the fundamentally different data format.

**Performance-wise**:
- **Fastest**: HarfBuzz + CoreGraphics (macOS) at ~4,500 ops/sec
- **Best cross-platform**: HarfBuzz + Opixa at ~2,500 ops/sec
- **linra-mac**: Best for simple fonts when shaping data isn't needed

The detected "performance regressions" should be validated with more iterations, as some may be measurement noise at sub-millisecond scales.

## Benchmark Methodology

### Test Setup

- **Fonts**: Standard TTF, CBDT bitmap, COLR vector, SVG table, sbix bitmap, emoji
- **Scripts**: Latin (latn), Arabic (arab), Mixed scripts (mixd)
- **Sizes**: 16px, 64px, 128px
- **Iterations**: 1000 per configuration
- **Warmup**: 100 iterations before measurement
- **Platform**: macOS with M1 processor

### Measurement Process

1. Load font into memory
2. Warmup phase with repeated renders
3. Actual measurement phase
4. Collect ns/op and ops/sec metrics
5. Verify output correctness
6. Record any errors or failures

### Data Analysis

- Outliers removed (top/bottom 5%)
- Statistical significance calculated
- Regression detection at >10% slowdown
- Error patterns analyzed across backends

---

This benchmark analysis provides the foundation for performance optimization decisions and backend selection strategies. Use these findings to guide your implementation choices and identify areas needing improvement.