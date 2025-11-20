# TypF Performance Optimization Guide

This guide provides actionable recommendations for optimizing TypF performance based on comprehensive benchmarking analysis.

**Related Documentation:**
- **[Backend Comparison](BACKEND_COMPARISON.md)** - Choose the right backend for your use case
- **[Quick Start](../typf-tester/QUICKSTART.md)** - Get benchmarking tools running
- **[Examples](../examples/README.md)** - See optimization techniques in action

## Table of Contents

1. [Performance Characteristics](#performance-characteristics)
2. [Optimization Strategies](#optimization-strategies)
3. [Benchmarking Tools](#benchmarking-tools)
4. [Common Pitfalls](#common-pitfalls)
5. [Platform-Specific Optimizations](#platform-specific-optimizations)

## Performance Characteristics

### Pipeline Breakdown (Measured with typfme.py benchmarks)

From comprehensive benchmarking, we've identified where time is spent:

```
Shaping:    ~30-47µs   (3% of total time)
Rendering:  ~1122µs    (97% of total time)  ← PRIMARY BOTTLENECK
```

**Key Insight**: Rendering is 37x slower than shaping. Optimization efforts should focus on rendering, not shaping.

### Shaping Performance

| Shaper | Avg Time (µs) | Ops/sec | Notes |
|--------|---------------|---------|-------|
| NONE | 36.3 | 27,599 | Simple LTR advancement |
| HARFBUZZ | 46.6 | 21,769 | Full OpenType shaping |

- HarfBuzz is only ~28% slower than NONE (expected due to complex shaping)
- For most use cases, the shaping performance difference is negligible
- **Recommendation**: Use HarfBuzz unless you have extreme performance requirements

### Rendering Performance

| Font Size (px) | Render Time (µs) | Ops/sec | Scaling Factor |
|----------------|------------------|---------|----------------|
| 16 | 195 | 5,988 | 1.0x (baseline) |
| 32 | 384 | 2,982 | 2.0x |
| 64 | 975 | 1,167 | 5.0x |
| 128 | 2,935 | 399 | 15.1x |

**Analysis**:
- Rendering scales **super-linearly** with font size
- 8x font size increase (16→128) results in ~15x slower rendering
- This is expected due to O(size²) behavior (bitmap area grows quadratically)
- Actual scaling is slightly better than theoretical O(size²) due to fixed overhead

## Optimization Strategies

### 1. Choose the Right Font Size

**Problem**: Large font sizes have disproportionate rendering cost.

**Solution**:
```rust
// Instead of rendering at 128px and downscaling:
let result = render_text(text, font, 128.0); // ~2935µs

// Render at target size directly:
let result = render_text(text, font, 32.0);  // ~384µs (7.6x faster!)
```

**When to use**:
- Rendering for thumbnails or previews
- Batch processing multiple sizes
- Real-time rendering applications

### 2. Cache Shaped Results

**Problem**: Re-shaping the same text repeatedly wastes CPU cycles.

**Solution**:
```rust
use std::collections::HashMap;

struct ShapeCache {
    cache: HashMap<(String, String, u32), ShapingResult>,
}

impl ShapeCache {
    fn get_or_shape(&mut self, text: &str, font: &str, size: u32) -> &ShapingResult {
        let key = (text.to_string(), font.to_string(), size);
        self.cache.entry(key).or_insert_with(|| {
            // Perform expensive shaping operation
            shape_text(text, font, size as f32)
        })
    }
}
```

**Impact**: Up to 30-47µs saved per re-render of the same text.

### 3. Use SVG Export for Large/Long Texts

**Problem**: Bitmap rendering has width limits (~10,000px) and scales poorly for large text.

**Solution**:
```python
# For long texts or large sizes, use SVG:
svg_output = engine.render_to_svg(long_text, font, size=48)  # No width limits

# Instead of bitmap which would fail:
# bitmap = engine.render_text(long_text, font, size=48)  # InvalidDimensions error!
```

**Benefits**:
- No width/height limits
- Scalable output (infinite zoom)
- Smaller file sizes for text
- Faster generation for very large texts

### 4. Batch Processing Optimization

**Problem**: Repeated font loading and engine initialization overhead.

**Solution**:
```rust
// Bad: Create engine for each text
for text in texts {
    let engine = Typf::new()?;  // Overhead on each iteration
    engine.render(text)?;
}

// Good: Reuse engine instance
let engine = Typf::new()?;
for text in texts {
    engine.render(text)?;  // No initialization overhead
}
```

**Impact**: Eliminates repeated initialization, especially significant for short texts.

### 5. Choose Minimal Backend for Simple Use Cases

**Problem**: Using complex backends when simple ones suffice.

**Solution**:
```rust
// For Latin-only text without complex shaping:
let engine = Typf::builder()
    .shaper("none")      // 28% faster than HarfBuzz
    .renderer("orge")
    .build()?;

// Only use HarfBuzz when you need it:
let engine = Typf::builder()
    .shaper("harfbuzz")  // For Arabic, Devanagari, etc.
    .renderer("orge")
    .build()?;
```

**When to use NONE shaper**:
- Simple Latin text
- Monospaced fonts
- ASCII-only content
- Maximum performance requirement

### 6. Line Wrapping for Long Texts

**Problem**: Single-line rendering of long text hits bitmap width limits.

**Solution**:
```rust
fn wrap_and_render(text: &str, font: &Font, max_width_px: u32) -> Vec<BitmapData> {
    let max_chars = (max_width_px as f32 / (font_size * 0.55)) as usize;
    let lines = wrap_text(text, max_chars);

    lines.iter()
        .map(|line| render_text(line, font, font_size))
        .collect()
}
```

See `examples/long_text_handling.rs` for complete implementation.

## Benchmarking Tools

### Using typfme.py (Python Testing Tool)

TypF includes a comprehensive benchmarking tool:

```bash
cd typf-tester

# Test all backends with rendering
python typfme.py render --backend=harfbuzz --format=png

# Benchmark shaping performance only
python typfme.py bench-shaping --iterations=1000

# Benchmark rendering performance only
python typfme.py bench-rendering --iterations=100

# Test text length scaling limits
python typfme.py bench-scaling --iterations=50
```

**Output**: Detailed JSON reports in `typf-tester/output/`:
- `shaping_benchmark.json` - Shaping performance data
- `rendering_benchmark.json` - Rendering performance breakdown
- `scaling_benchmark.json` - Text length scaling analysis
- `benchmark_report.json` - Comprehensive benchmark results

### Interpreting Benchmark Results

**Good Performance Indicators**:
- Shaping < 50µs for typical text
- Rendering < 500µs at 16px
- Rendering < 2000µs at 64px
- Linear scaling with text length

**Performance Red Flags**:
- Shaping > 200µs (check for inefficient font loading)
- Rendering > 5000µs at 64px (check for large glyphs or complex paths)
- Super-linear scaling with text length (may indicate O(n²) algorithm)

## Common Pitfalls

### 1. Rendering at High DPI Without Scaling

❌ **Bad**:
```rust
// Rendering at 4x resolution for "better quality"
let result = render_text(text, font, 192.0);  // 50x slower than 32px!
```

✅ **Good**:
```rust
// Render at target size, let display handle DPI scaling
let result = render_text(text, font, 48.0);
```

### 2. Not Reusing Font Objects

❌ **Bad**:
```rust
for text in texts {
    let font = Font::from_file("font.ttf")?;  // Repeated I/O!
    render_text(text, &font, 48.0)?;
}
```

✅ **Good**:
```rust
let font = Font::from_file("font.ttf")?;  // Load once
for text in texts {
    render_text(text, &font, 48.0)?;
}
```

### 3. Using Bitmap Rendering for Very Long Texts

❌ **Bad**:
```rust
// Will fail with InvalidDimensions error
let long_text = "...1000+ characters...";
let result = render_text(long_text, font, 48.0)?;  // Error!
```

✅ **Good**:
```rust
// Use SVG for long texts
let svg = render_to_svg(long_text, font, 48.0)?;  // ✓ Works!

// Or implement line wrapping
let lines = wrap_text(long_text, 200);
for line in lines {
    render_text(line, font, 48.0)?;  // ✓ Each line fits
}
```

### 4. Over-Engineering for Simple Use Cases

❌ **Bad**:
```rust
// Using complex pipeline for simple static text
let engine = Typf::builder()
    .shaper("harfbuzz")
    .renderer("skia")
    .with_features(vec!["liga", "kern", "calt"])
    .build()?;

let result = engine.render("Hello")?;  // Overkill!
```

✅ **Good**:
```rust
// Use simple backend for simple text
let engine = Typf::builder()
    .shaper("none")
    .renderer("orge")
    .build()?;

let result = engine.render("Hello")?;  // 28% faster
```

## Platform-Specific Optimizations

### macOS: Use CoreText + CoreGraphics

When both shaping and rendering use native backends, TypF can optimize with a single call:

```rust
let engine = Typf::builder()
    .shaper("coretext")
    .renderer("coregraphics")
    .build()?;

// Single optimized path (no intermediate data structures)
let result = engine.render(text)?;
```

**Benefits**:
- Native platform integration
- Hardware-accelerated rendering
- Best quality on macOS

### Windows: Use DirectWrite + Direct2D

Similar optimization available on Windows:

```rust
let engine = Typf::builder()
    .shaper("directwrite")
    .renderer("direct2d")
    .build()?;
```

**Note**: Currently blocked pending Windows platform access.

### Linux/Cross-platform: HarfBuzz + Orge

For maximum portability:

```rust
let engine = Typf::builder()
    .shaper("harfbuzz")  // Pure Rust + C (widely available)
    .renderer("orge")    // Pure Rust (no external deps)
    .build()?;
```

**Benefits**:
- Zero platform dependencies
- Consistent behavior across platforms
- Smallest binary size

## Performance Targets & Achievements

| Metric | Target | Achieved | Status |
|--------|--------|----------|--------|
| Simple Latin shaping | <10µs/100 chars | ~3.6µs/100 chars | ✅ 2.8x better |
| Complex Arabic shaping | <50µs/100 chars | ~4.7µs/100 chars | ✅ 10.6x better |
| Glyph rasterization (16px) | <1µs/glyph | ~195µs total* | ⚠️ See note |
| Binary size (minimal) | ~500KB | 1.1MB | ⚠️ Close |

*Note: Total time includes composition of multiple glyphs. Per-glyph time varies by complexity.

## Recommendations Summary

**For Maximum Performance**:
1. Use NONE shaper for simple Latin text
2. Render at target size (avoid downscaling)
3. Cache shaped results for repeated text
4. Reuse font and engine objects
5. Use SVG export for long/large texts

**For Best Quality**:
1. Use HarfBuzz shaper for all scripts
2. Use platform-native renderers when available
3. Enable anti-aliasing (default)
4. Render at actual display size

**For Production Use**:
1. Implement line wrapping for arbitrary text
2. Add shaping cache for repeated text
3. Use SVG export for scalable output
4. Monitor rendering time in production
5. Set reasonable font size limits

---

*Last Updated: 2025-11-19*
*Based on comprehensive benchmarking with typfme.py*
*Community project by FontLab - https://www.fontlab.org/*
