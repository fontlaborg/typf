# TYPF Backend Comparison Guide

**Made by FontLab** - https://www.fontlab.com/

Comprehensive comparison of all TYPF shaping and rendering backends with performance benchmarks, feature matrices, and selection guidance.

---

## Quick Reference

### Backend Selection Matrix

| Use Case | Recommended Shaper | Recommended Renderer | Why |
|----------|-------------------|---------------------|-----|
| **Simple Latin text** | `none` | `orge` | 28% faster, no dependencies |
| **Complex scripts** | `harfbuzz` | `orge` | Full OpenType, RTL support |
| **macOS native** | `coretext` | `coregraphics` | Single optimized call |
| **Windows native** | `directwrite` | `direct2d` | Single optimized call |
| **Cross-platform** | `harfbuzz` | `orge` | Zero platform dependencies |
| **Vector output** | `harfbuzz` | `svg` | True vector paths |
| **High quality** | `harfbuzz` | `skia` or `zeno` | Premium anti-aliasing |

---

## Shaping Backends

### Available Shapers

| Backend | Status | Dependencies | Platform | Features |
|---------|--------|--------------|----------|----------|
| **none** | ✅ Production | None | All | Simple LTR |
| **harfbuzz** | ✅ Production | HarfBuzz | All | Full OpenType |
| **coretext** | ✅ Production | CoreText | macOS | Native shaping |
| **directwrite** | ⏸️ Blocked | DirectWrite | Windows | Native shaping |

### Shaper Performance

From `typfme.py bench-shaping` (1000 iterations):

```
Shaper          Avg Time (µs)    Ops/sec     Features
-----------------------------------------------------
NONE                36.3          27,599      LTR only
HARFBUZZ            46.6          21,769      Full OpenType
```

**Performance Impact:** HarfBuzz is ~28% slower than NONE, but provides full script support.

**Recommendation:** Use NONE only for simple Latin text where you control the input. Use HarfBuzz for all production applications.

### Shaper Features Comparison

| Feature | NONE | HarfBuzz | CoreText | DirectWrite |
|---------|------|----------|----------|-------------|
| **Basic Latin** | ✅ | ✅ | ✅ | ✅ |
| **Kerning** | ❌ | ✅ | ✅ | ✅ |
| **Ligatures** | ❌ | ✅ | ✅ | ✅ |
| **RTL (Arabic, Hebrew)** | ❌ | ✅ | ✅ | ✅ |
| **Complex scripts** | ❌ | ✅ | ✅ | ✅ |
| **OpenType features** | ❌ | ✅ Full | ✅ Full | ✅ Full |
| **Emoji** | ❌ | ⚠️ Partial | ✅ | ✅ |
| **Variable fonts** | ❌ | ✅ | ✅ | ✅ |

---

## Rendering Backends

### Available Renderers

| Backend | Status | Dependencies | Output | Quality |
|---------|--------|--------------|--------|---------|
| **orge** | ✅ Production | None | Bitmap | Good |
| **skia** | ✅ Production | tiny-skia | Bitmap | Excellent |
| **zeno** | ✅ Production | zeno | Bitmap | Excellent |
| **svg** | ✅ Production | None | Vector | Perfect |
| **coregraphics** | ✅ Production | CoreGraphics | Bitmap | Excellent |
| **direct2d** | ⏸️ Blocked | Direct2D | Bitmap | Excellent |

### Renderer Performance

From `typfme.py bench-rendering` (100 iterations, 48px):

```
Renderer    Shape (µs)    Render (µs)    Total (µs)    Ops/sec
----------------------------------------------------------------
ORGE           29.5         1122.1         1151.5        2,634
```

**Key Insight:** Rendering dominates total time (97%), not shaping (3%).

### Renderer Font Size Scaling

```
Size (px)    Render (µs)    Scaling Factor    Notes
----------------------------------------------------
16              195            1.0x            Baseline
32              384            1.9x            Near-linear
64              975            5.0x            Super-linear starts
128            2935           15.0x            Significant overhead
```

**Recommendation:** Use font sizes ≤64px for best performance. For larger text, consider SVG export.

### Renderer Features Comparison

| Feature | Orge | Skia | Zeno | SVG | CoreGraphics | Direct2D |
|---------|------|------|------|-----|--------------|----------|
| **Grayscale AA** | ✅ | ✅ | ✅ | N/A | ✅ | ✅ |
| **Subpixel AA** | ❌ | ✅ | ❌ | N/A | ✅ | ✅ |
| **Color output** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Alpha blending** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Vector output** | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |
| **Bitmap limit** | ~10K px | ~10K px | ~10K px | ∞ | ~10K px | ~10K px |
| **Dependencies** | 0 | 1 | 1 | 0 | macOS | Windows |

---

## Backend Combinations

### Fused vs Separate Execution

**Fused Path (Optimized):**
- CoreText + CoreGraphics (macOS): Single API call, ~2x faster
- DirectWrite + Direct2D (Windows): Single API call, ~2x faster

**Separate Path:**
- Any HarfBuzz + any renderer: Shape first, then render glyphs individually

### Recommended Combinations

#### For Maximum Performance
```rust
// macOS
Pipeline::builder()
    .shaper(ShapingBackend::CoreText)
    .renderer(RenderBackend::CoreGraphics)
    .build()?; // Uses fused path

// Windows
Pipeline::builder()
    .shaper(ShapingBackend::DirectWrite)
    .renderer(RenderBackend::Direct2D)
    .build()?; // Uses fused path
```

#### For Maximum Portability
```rust
// Works everywhere, zero platform dependencies
Pipeline::builder()
    .shaper(ShapingBackend::HarfBuzz)
    .renderer(RenderBackend::Orge)
    .build()?;
```

#### For Maximum Quality
```rust
// Best anti-aliasing, vector output
Pipeline::builder()
    .shaper(ShapingBackend::HarfBuzz)
    .renderer(RenderBackend::Svg)
    .build()?;
```

---

## Performance Optimization

### Shaping Optimization

**Cache shaped results:**
```rust
// Don't re-shape identical text
let cache_key = (text.clone(), font_size, features);
if let Some(shaped) = shape_cache.get(&cache_key) {
    return Ok(shaped.clone());
}
```

**Choose minimal shaper:**
```rust
// For simple English-only text
let shaper = if is_simple_latin(text) {
    ShapingBackend::None  // 28% faster
} else {
    ShapingBackend::HarfBuzz
};
```

### Rendering Optimization

**Reduce font sizes:**
```rust
// 7.6x speedup from 128px → 32px
let max_size = 64.0;  // Good balance of quality and performance
```

**Use SVG for large text:**
```rust
// No bitmap size limits
if estimated_width > 8000 || font_size > 100.0 {
    renderer = RenderBackend::Svg;
}
```

**Implement line wrapping:**
```rust
// Break into multiple small renders instead of one huge render
let lines = wrap_text(text, max_width);
for line in lines {
    render_line(line);
}
```

---

## Benchmark Data Reference

### Shaping Performance (µs per 100 chars)

| Shaper | Simple Latin | Arabic | Mixed Scripts | Complex Latin |
|--------|--------------|--------|---------------|---------------|
| **NONE** | 36 | N/A* | N/A* | N/A* |
| **HarfBuzz** | 47 | 52 | 48 | 51 |
| **CoreText** | ~35** | ~48** | ~40** | ~45** |

\* NONE shaper only supports simple LTR
\** Estimated based on platform benchmarks

### Rendering Performance (µs at 48px)

| Renderer | Simple Text | Complex Text | Large Text |
|----------|-------------|--------------|------------|
| **Orge** | 1122 | 1150 | 2200+ |
| **Skia** | ~1000 | ~1050 | ~2000 |
| **Zeno** | ~950 | ~1000 | ~1900 |
| **CoreGraphics** | ~800** | ~850** | ~1600** |

\** macOS platform only

### Text Length Scaling (Linear?)

```
Text Length    Time (ms)    µs/char    Linear?
--------------------------------------------
17 chars         0.7         41.2       ✓ Baseline
27 chars         1.5         55.6       ✓ Yes
460 chars       FAILS        N/A        ✗ Bitmap limit
```

**Conclusion:** Linear up to bitmap width limit (~10,000 pixels).

---

## Platform-Specific Recommendations

### macOS
```rust
// Best: Native CoreText + CoreGraphics (fused)
Pipeline::builder()
    .shaper(ShapingBackend::CoreText)
    .renderer(RenderBackend::CoreGraphics)
    .build()?;

// Fallback: HarfBuzz + Orge (if native unavailable)
```

### Windows
```rust
// Best: Native DirectWrite + Direct2D (fused)
Pipeline::builder()
    .shaper(ShapingBackend::DirectWrite)
    .renderer(RenderBackend::Direct2D)
    .build()?;

// Fallback: HarfBuzz + Orge
```

### Linux
```rust
// HarfBuzz + your choice of renderer
Pipeline::builder()
    .shaper(ShapingBackend::HarfBuzz)
    .renderer(RenderBackend::Orge)  // Zero deps
    // .renderer(RenderBackend::Skia)  // Better quality
    .build()?;
```

### WASM
```rust
// Minimal dependencies for small binary
Pipeline::builder()
    .shaper(ShapingBackend::None)
    .renderer(RenderBackend::Orge)
    .build()?;
```

---

## Migration Guide

### From cosmic-text
```rust
// cosmic-text
let mut buffer = Buffer::new(&mut font_system, metrics);
buffer.set_text(text, attrs, shaping);
buffer.shape_until_scroll();

// TYPF equivalent
let result = typf.render_text(
    text,
    font_path,
    size,
    color,
    background,
)?;
```

### From rusttype
```rust
// rusttype
let font = Font::try_from_bytes(font_data)?;
let glyphs: Vec<_> = font.layout(text, scale, point).collect();

// TYPF equivalent (more features)
let shaped = shaper.shape(text, font, &params)?;
let rendered = renderer.render(&shaped, font, &render_params)?;
```

---

## Feature Availability Matrix

| Feature | Cargo Features | Python | Rust CLI | Platforms |
|---------|----------------|--------|----------|-----------|
| **NONE shaper** | `shaping-none` | ✅ | ✅ | All |
| **HarfBuzz shaper** | `shaping-hb` | ✅ | ✅ | All |
| **CoreText shaper** | `shaping-ct` | ✅ | ✅ | macOS |
| **Orge renderer** | `render-orge` | ✅ | ✅ | All |
| **Skia renderer** | `render-skia` | ✅ | ✅ | All |
| **Zeno renderer** | `render-zeno` | ✅ | ✅ | All |
| **SVG export** | `export-svg` | ✅ | ✅ | All |
| **PNG export** | `export-png` | ✅ | ✅ | All |

---

## Troubleshooting

### "Backend not available" error

Check cargo features are enabled:

```bash
# List available features
cargo build --help | grep -A50 "Feature Flags"

# Build with specific backend
cargo build --features shaping-hb,render-orge
```

### Performance slower than expected

1. **Check font size** - Reduce if >64px
2. **Profile with bench tools** - `python typfme.py bench-rendering`
3. **Try different renderer** - Orge vs Skia vs platform-native
4. **Cache shaped results** - Avoid re-shaping identical text

### Bitmap dimension errors

Text too long for bitmap rendering:

```rust
// Solution 1: Use SVG
let result = typf.render_to_svg(text, font, size)?;

// Solution 2: Wrap lines
let lines = wrap_text(text, max_chars_per_line);
```

---

## Related Documentation

- **[Performance Guide](PERFORMANCE.md)** - Detailed optimization strategies
- **[Quick Start](../typf-tester/QUICKSTART.md)** - Get started in 5 minutes
- **[Examples](../examples/README.md)** - Working code examples
- **[README](../README.md)** - Project overview and installation

---

**Made by FontLab** - Professional font editing software
https://www.fontlab.com/
