# Opixa Renderer

Opixa rasterizes glyph outlines with pure Rust. Simple, fast, and dependency-free - the foundation of Typf's minimal build.

## What Opixa Does

- Converts bezier curves to pixels
- Handles anti-aliasing and grayscale
- Works with any font format
- No external dependencies

```rust
pub struct OpixaRenderer {
    rasterizer: Rasterizer,
    scan_converter: ScanConverter,
    grayscale_filter: GrayscaleFilter,
}
```

## Performance

| Text Size | Speed | Memory | Quality |
|-----------|-------|---------|---------|
| Small (12pt) | Baseline | Minimal | Good |
| Medium (24pt) | 2x CPU | 100KB | Better |
| Large (48pt) | 3x CPU | 200KB | Excellent |

## When to Use Opixa

- **Minimal builds** - No heavy dependencies
- **Embedded systems** - Small binary size  
- **Simple text** - Basic rendering needs
- **Debugging** - Predictable output

## Basic Usage

### Rust

```rust
let mut renderer = OpixaRenderer::new(width, height)?;
let bitmap = renderer.rasterize(&shaped_text, &font)?;
```

### Python

```python
import typf

renderer = typf.Typf(renderer="opixa")
result = renderer.render_text("Hello", "font.ttf")
```

## Rasterization Pipeline

Opixa processes glyphs in three stages:

```
Outlines → Fill Coverage → Grayscale → Bitmap
```

1. **Outline processing** - Bezier curves to edges
2. **Scan conversion** - Edge accumulation into coverage
3. **Filtering** - Coverage to pixel intensities

## Anti-aliasing Options

```rust
#[derive(Debug, Clone)]
pub enum AntiAliasingMode {
    None,           // Hard edges, fastest
    Grayscale,      // 8-bit coverage values
    Supersample4x,  // 4x supersampling
    Supersample16x, // 16x supersampling
}

renderer.set_aa_mode(AntiAliasingMode::Grayscale)?;
```

### Quality vs Speed

| Mode | Quality | Speed | Memory |
|------|---------|-------|---------|
| 16x SS | Best | Slow | High |
| Grayscale | Good | Fast | Low |
| 4x SS | Better | Medium | Medium |
| None | Poor | Fastest | Minimal |

## Memory Management

Opixa handles memory efficiently:

```rust
// Reuse buffers for better performance
renderer.reuse_buffers(true);

// Pool for multi-threaded rendering
let pool = OpixaRenderPool::new(4);
```

## Configuration

Minimal dependencies required:

```toml
[dependencies.typf]
features = ["render-opixa"]  # Always enabled in minimal build
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum OpixaRendererError {
    #[error("Buffer allocation failed: {size} bytes")]
    BufferAlloc { size: usize },
    
    #[error("Glyph outline parsing failed: {glyph_id}")]
    OutlineParse { glyph_id: GlyphId },
    
    #[error("Invalid dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
}
```

## Advanced Features

### Custom Filters

```rust
trait PixelFilter {
    fn apply(&self, coverage: f32) -> u8;
}

// Sharpening filter
struct SharpenFilter;
impl PixelFilter for SharpenFilter {
    fn apply(&self, coverage: f32) -> u8 {
        (coverage * 1.2).min(255.0) as u8
    }
}

renderer.set_filter(Box::new(SharpenFilter));
```

### Subpixel Positioning

```rust
// Position glyphs at subpixel precision
renderer.enable_subpixel(true);
renderer.set_ppem(24.0); // Pixels per EM
```

### Gamma Correction

```rust
// Apply gamma for better perceived quality
renderer.set_gamma(2.2);
```

## Performance Tips

1. **Reuse renderers** - expensive to create
2. **Pool buffers** - avoid allocations
3. **Batch glyphs** - process runs together
4. **Choose AA carefully** - balance quality/speed
5. **Prefer grayscale** - best quality/speed trade-off

## Debug Mode

Enable performance monitoring:

```rust
renderer.set_debug_mode(true);
let stats = renderer.get_stats();

println!("Rasterized {} glyphs", stats.glyph_count);
println!("Buffer size: {} KB", stats.buffer_size_kb);
println!("Render time: {} μs", stats.render_time_us);
```

## Benchmarks

Opixa performance on different platforms:

| Platform | Glyphs/sec | Memory/Glyph |
|----------|------------|--------------|
| x86_64 (Linux) | 50K | 200B |
| ARM64 (Mac) | 45K | 200B |
| x86_64 (Windows) | 48K | 200B |

## Memory Layout

Opixa uses efficient memory layout:

```rust
#[repr(C)]
pub struct Bitmap {
    width: u32,
    height: u32,
    stride: u32,
    data: Vec<u8>,  // Row-major, grayscale
}

// Direct memory access for copying
unsafe extern "C" fn copy_bitmap(src: *const u8, dst: *mut u8, len: usize) {
    std::ptr::copy_nonoverlapping(src, dst, len);
}
```

## Optimization Features

### SIMD Acceleration

```rust
#[cfg(target_arch = "x86_64")]
renderer.enable_simd(true);

#[cfg(target_arch = "aarch64")]
renderer.enable_neon(true);
```

### Parallel Processing

```rust
use rayon::prelude_;

// Process multiple glyphs in parallel
let bitmaps: Vec<Bitmap> = glyphs
    .par_iter()
    .map(|g| renderer.rasterize_glyph(g))
    .collect();
```

## Export Options

Save rendered output:

```rust
// PPM format (uncompressed)
let ppm_data = renderer.export_ppm(&bitmap)?;
std::fs::write("output.ppm", ppm_data);

// Raw bitmap data
let raw = bitmap.as_raw();
std::fs::write("output.raw", raw);
```

## Migration Pattern

Start with Opixa, upgrade when needed:

```rust
// Phase 1: Prototype with Opixa
let mut renderer = Box::new(OpixaRenderer::new(width, height)?);

// Phase 2: Test rendering pipeline
let result = render_text_with_pipeline(renderer.as_mut(), text, font)?;

// Phase 3: Upgrade if needed
if needs_high_quality() {
    renderer = Box::new(SkiaRenderer::new(width, height)?);
}
```

## Testing

Test Opixa rendering consistency:

```rust
#[test]
fn test_rendering_consistency() {
    let renderer = OpixaRenderer::new(100, 100).unwrap();
    let text = "Test";
    
    let result1 = renderer.rasterize(text, &font);
    let result2 = renderer.rasterize(text, &font);
    
    assert_eq!(result1.as_bytes(), result2.as_bytes());
}
```

---

Opixa provides solid text rendering with minimal dependencies. Use it for embedded systems, minimal builds, or when you need predictable, lightweight rasterization.
