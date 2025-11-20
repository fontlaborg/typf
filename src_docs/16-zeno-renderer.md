# Zeno Renderer

Zeno renders high-quality vector graphics with precise curve handling and smooth gradients.

## What Zeno Does

Zeno transforms shaped glyphs into vector paths with:
- Exact Bézier curve preservation
- Anti-aliased edge rendering
- Gradient and transparency support
- Subpixel precision positioning

```rust
#[cfg(feature = "render-zeno")]
pub struct ZenoRenderer {
    canvas: Canvas,
    transform: Transform,
    stroke_width: f32,
    fill_rule: FillRule,
}
```

## When to Use Zeno

Choose Zeno when you need:
- Vector output (SVG, PDF)
- High-quality printed typography
- Complex visual effects
- Precise curve control

Skip Zeno for:
- Simple bitmap rendering
- Maximum speed requirements
- Minimal binary size

## Performance Profile

| Task | Skia | Orge | Zeno |
|------|------|------|------|
| Small text (12pt) | 0.8ms | 0.3ms | 1.2ms |
| Medium text (24pt) | 1.5ms | 0.7ms | 2.1ms |
| Large text (48pt) | 3.2ms | 1.8ms | 3.8ms |
| Vector export | 0.5ms | N/A | 0.3ms |

Zeno excels at vector export quality but is slower than raster renderers.

## Basic Usage

```rust
use typf_core::traits::Renderer;
use backends::typf_render_zeno::ZenoRenderer;

// Create renderer
let mut renderer = ZenoRenderer::new(width, height)?;
renderer.set_quality(RenderQuality::High);

// Render text
let result = renderer.render(shaped_text, &font)?;

// Export to SVG
let svg_bytes = renderer.export_svg(&result)?;
```

```python
import typf

# Use Zeno for vector output
renderer = typf.Typf(renderer="zeno")
result = renderer.render_text("Hello World", "font.ttf", 
                              output_format="svg")
```

## Quality Settings

### Render Quality

```rust
#[derive(Debug, Clone, Copy)]
pub enum RenderQuality {
    Draft,      // Fast, lower precision
    Normal,     // Balance of speed/quality
    High,       // Maximum precision
    Print,      // Print-optimized curves
}
```

### Anti-aliasing

```rust
// Anti-aliasing levels
renderer.set_antialiasing(AntialiasingLevel::None);     // No AA
renderer.set_antialiasing(AntialiasingLevel::Low);      // 2x supersample
renderer.set_antialiasing(AntialiasingLevel::Medium);   // 4x supersample
renderer.set_antialiasing(AntialiasingLevel::High);     // 8x supersample
```

## Advanced Rendering

### Stroke Effects

```rust
// Custom stroke styling
let stroke_options = StrokeOptions {
    width: 2.0,
    line_cap: LineCap::Round,
    line_join: LineJoin::Round,
    dash_array: vec![5.0, 3.0],
    dash_offset: 0.0,
};
renderer.set_stroke_options(stroke_options);
```

### Fill Patterns

```rust
// Gradient fills
let gradient = LinearGradient::new(
    Point::new(0.0, 0.0),
    Point::new(width as f32, height as f32),
    vec![
        ColorStop::new(0.0, Color::rgb(255, 0, 0)),
        ColorStop::new(1.0, Color::rgb(0, 0, 255)),
    ],
);
renderer.set_fill_pattern(FillPattern::Linear(gradient));
```

### Filters and Effects

```rust
// Drop shadow
let shadow = DropShadow {
    offset: Vector::new(2.0, 2.0),
    blur_radius: 3.0,
    color: Color::rgba(0, 0, 0, 128),
};
renderer.add_effect(Box::new(shadow));

// Glow effect
let glow = Glow {
    radius: 5.0,
    color: Color::rgba(255, 255, 0, 64),
};
renderer.add_effect(Box::new(glow));
```

## Export Formats

### SVG Export

```rust
// Export with optimizations
let svg_options = SvgOptions {
    precision: 6,              // Decimal places
    optimize_paths: true,      // Remove redundant points
    embed_fonts: false,        // Reference external fonts
    pretty_print: true,        // Human-readable output
};

let svg_content = renderer.export_svg_with_options(&result, svg_options)?;
```

### PDF Export

```rust
// PDF for print
let pdf_options = PdfOptions {
    dpi: 300,                  // Print resolution
    embed_fonts: true,         // Include font subsets
    compress: true,            // Compress content
    version: PdfVersion::V1_7, // PDF version
};

let pdf_bytes = renderer.export_pdf(&result, pdf_options)?;
```

## Precision Control

### Coordinate Precision

```rust
// Set precision for different use cases
renderer.set_coordinate_precision(6);    // Web graphics
renderer.set_coordinate_precision(8);    // Desktop apps  
renderer.set_coordinate_precision(12);   // Print quality
```

### Curve Optimization

```rust
// Curve tolerance for simplification
let curve_options = CurveOptions {
    tolerance: 0.01,           // Maximum deviation
    min_segments: 4,           // Minimum curve segments
    max_segments: 100,         // Maximum curve segments
    preserve_corners: true,    // Keep sharp corners
};
renderer.set_curve_options(curve_options);
```

## Memory Management

### Canvas Pooling

```rust
// Reuse canvases for better performance
let canvas_pool = CanvasPool::new(10);    // Pool of 10 canvases
renderer.set_canvas_pool(canvas_pool);

// Clear pool when done
renderer.clear_canvas_pool();
```

### Buffer Management

```rust
// Optimize buffer sizes
renderer.set_buffer_size(BufferSize::Auto);     // Auto-detect
renderer.set_buffer_size(BufferSize::Fixed(4096)); // 4KB buffers
renderer.set_buffer_size(BufferSize::Huge(65536));  // 64KB buffers
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ZenoRendererError {
    #[error("Canvas creation failed: {0}")]
    CanvasCreation(String),
    
    #[error("Invalid curve parameters: {0}")]
    InvalidCurve(String),
    
    #[error("Export format not supported: {0}")]
    UnsupportedExport(String),
    
    #[error("Memory allocation failed: {0}")]
    MemoryError(String),
}
```

## Integration Examples

### Web Graphics

```rust
// Generate SVG for web pages
let web_renderer = ZenoRenderer::new(800, 600)?;
web_renderer.set_quality(RenderQuality::Normal);
web_renderer.set_coordinate_precision(6);

let svg = web_renderer.export_svg(&result)?;
web_page.insert_svg(&svg, "#text-container");
```

### Print Production

```rust
// High-quality PDF for printing
let print_renderer = ZenoRenderer::new(2400, 3300)?; // 8" x 11" at 300 DPI
print_renderer.set_quality(RenderQuality::Print);
print_renderer.set_coordinate_precision(12);

let pdf = print_renderer.export_pdf(&result, pdf_options)?;
send_to_printer(&pdf);
```

### Desktop Applications

```rust
// Render to window surface
let window_renderer = ZenoRenderer::new(window_width, window_height)?;
window_renderer.set_quality(RenderQuality::High);

let frame = window_renderer.render_frame(&result)?;
window_surface.display_frame(&frame);
```

## Performance Optimization

### Batching

```rust
// Group similar operations
let batch = renderer.begin_batch();
for glyph in glyphs {
    batch.add_glyph(&glyph);
}
let result = batch.finish()?;
```

### Caching

```rust
// Cache complex curves
let cache_key = format!("{}:{}:{}", text, font_name, size);
if let Some(cached) = renderer.get_cached_curve(&cache_key) {
    return cached;
}

let curve = renderer.generate_curve(text, font, size)?;
renderer.cache_curve(cache_key, curve.clone());
Ok(curve)
```

## Testing

### Quality Tests

```rust
#[test]
fn test_curve_precision() {
    let renderer = ZenoRenderer::new(1000, 1000)?;
    renderer.set_quality(RenderQuality::Print);
    
    // Test complex curves
    let complex_glyph = load_complex_glyph();
    let result = renderer.render_glyph(&complex_glyph)?;
    
    // Verify curve smoothness
    assert!(is_curve_smooth(&result, tolerance: 0.001));
}
```

### Export Tests

```rust
#[test]
fn test_svg_export() {
    let renderer = ZenoRenderer::new(800, 600)?;
    let result = renderer.render(sample_text, &font)?;
    
    let svg = renderer.export_svg(&result)?;
    
    // Verify SVG validity
    assert!(is_valid_svg(&svg));
    assert!(svg.contains(r#"<svg"#));
    assert!(svg.contains(r#"</svg>"#));
}
```

## Migration

### From Orge Renderer

```rust
// Before - Orge (raster only)
let orge = OrgeRenderer::new(width, height)?;
let bitmap = orge.render(text, &font)?;

// After - Zeno (vector + raster)
let zeno = ZenoRenderer::new(width, height)?;
let vector = zeno.render(text, &font)?;
let bitmap = zeno.rasterize(&vector)?; // Optional rasterization
```

### From Skia

```rust
// Skia and Zeno have similar APIs
let renderer = ZenoRenderer::new(width, height)?;
renderer.set_quality(RenderQuality::High); // Similar to Skia quality

// Most Skia options have Zeno equivalents
renderer.set_antialiasing(AntialiasingLevel::High); // Like Skia anti-aliasing
```

---

Zeno provides the highest quality vector rendering with precise curve control and advanced effects. Use it when visual quality matters more than raw speed.