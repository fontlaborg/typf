# Skia Renderer

Skia gives you hardware-accelerated rendering with subpixel precision. Cross-platform 2D graphics library used by Chrome, Android, and Flutter.

## What Skia Does

- Rasterizes glyph outlines to pixels
- Handles anti-aliasing and subpixel rendering
- Applies transforms, filters, and effects
- Renders to multiple formats (PNG, SVG, GPU surfaces)

```rust
pub struct SkiaRenderer {
    surface: skia_safe::Surface,
    paint: skia_safe::Paint,
    font: skia_safe::Font,
}
```

## Performance

| Text Size | vs Opixa Renderer | Memory | Quality |
|-----------|------------------|---------|---------|
| Small (12pt) | 2x faster | +50% | Better anti-aliasing |
| Medium (24pt) | 3x faster | +40% | Superior subpixel |
| Large (48pt) | 4x faster | +30% | Hardware acceleration |

## When to Use Skia

- **High-quality text** - Need the best rendering
- **Complex effects** - Filters, transforms, blending
- **GPU acceleration** - Large volumes or real-time
- **Cross-platform** - Consistent results everywhere

## Basic Usage

### Rust

```rust
let mut renderer = SkiaRenderer::new(width, height)?;
renderer.set_font(&font)?;
let result = renderer.render(&shaped_text, &font)?;
```

### Python

```python
import typf

renderer = typf.Typf(renderer="skia")
result = renderer.render_text("Hello World", "font.ttf")
```

## Anti-aliasing Options

```rust
#[derive(Debug, Clone)]
pub enum AntiAliasingMode {
    None,           // Hard edges, fastest
    Grayscale,      // Standard anti-aliasing
    Subpixel,       // LCD subpixel rendering
    LCD,            // Optimized for LCD displays
}

renderer.set_anti_aliasing(AntiAliasingMode::Subpixel)?;
```

## Transform Support

Skia handles complex transformations:

```rust
use skia_safe::Matrix;

// Scale text
let transform = Matrix::scale((2.0, 2.0));
renderer.set_transform(&transform);

// Rotate text
let transform = Matrix::rotate(45.0);
renderer.set_transform(&transform);

// Skew text
let transform = Matrix::skew((0.2, 0.0));
renderer.set_transform(&transform);
```

## Blend Modes

```rust
use skia_safe::BlendMode;

renderer.set_blend_mode(BlendMode::Multiply);
renderer.set_blend_mode(BlendMode::Screen);
renderer.set_blend_mode(BlendMode::Overlay);
```

## Color Management

```rust
// Set text color
renderer.set_color(skia_safe::Color::from_rgb(255, 0, 0));

// Gradient fills
let gradient = skia_safe::gradient::LinearGradient::new(
    &points,
    &colors,
    &tile_mode,
    transform,
    None,
);
renderer.set_shader(gradient);
```

## GPU Acceleration

Enable GPU backend for better performance:

```rust
#[cfg(feature = "render-skia-gpu")]
let renderer = SkiaRenderer::with_gpu(context, width, height)?;
```

### GPU Performance

| Operation | CPU vs GPU | Speedup |
|-----------|------------|---------|
| Large text (1000+ glyphs) | 5x faster | |
| Complex transforms | 8x faster | |
| Heavy blending | 10x faster | |

## Export Formats

Skia renders to multiple formats:

```rust
// PNG (bitmap)
let png_data = renderer.render_to_png(&shaped_text)?;
std::fs::write("output.png", png_data);

// SVG (vector)
let svg_data = renderer.render_to_svg(&shaped_text)?;
std::fs::write("output.svg", svg_data);

// PDF (document)
let pdf_data = renderer.render_to_pdf(&shaped_text)?;
std::fs::write("output.pdf", pdf_data);
```

## Configuration

Enable Skia renderer:

```toml
[dependencies.typf]
features = [
    "render-skia",
    "render-skia-gpu",  # Optional GPU support
]
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum SkiaRendererError {
    #[error("Surface creation failed: {0}")]
    SurfaceCreation(String),
    
    #[error("GPU context initialization failed: {0}")]
    GPUInit(String),
    
    #[error("Font loading failed: {0}")]
    FontLoad(String),
}
```

## Advanced Features

### Custom Shaders

```rust
let shader = skia_safe::shaders::gradient::Sweep::new(
    center,
    colors,
    positions,
    tile_mode,
    transform,
);
renderer.set_shader(shader);
```

### Path Effects

```rust
let path_effect = skia_safe::path_effects::dash::new(
    intervals,
    phase,
);
renderer.set_path_effect(path_effect);
```

### Mask Filters

```rust
let mask_filter = skia_safe::mask_filters::blur(
    skia_safe::BlurStyle::Normal,
    (2.0, 2.0),
);
renderer.set_mask_filter(mask_filter);
```

## Performance Tips

1. **Reuse surfaces** - expensive to create
2. **Batch operations** - minimize state changes
3. **Use GPU** - for large text or complex effects
4. **Cache fonts** - avoid repeated loading
5. **Optimize transforms** - combine when possible

## Memory Management

Skia manages GPU memory automatically:

```rust
// Release GPU resources when done
renderer.release_gpu_resources();

// Flush pending operations
renderer.flush();
```

## Debug Mode

Enable Skia debug info:

```rust
renderer.set_debug_mode(true);
let debug_info = renderer.get_debug_info();
println!("Draw calls: {}", debug_info.draw_calls);
println!("GPU memory: {} MB", debug_info.gpu_memory_mb);
```

## Migration from Opixa

Switching to Skia is straightforward:

```rust
// Before - Opixa renderer
let mut renderer = OpixaRenderer::new(width, height)?;
let bitmap = renderer.rasterize(&shaped_text)?;

// After - Skia renderer
let mut renderer = SkiaRenderer::new(width, height)?;
let bitmap = renderer.render(&shaped_text)?;

// Results should match, with better quality
```

---

Skia provides the highest quality text rendering with hardware acceleration. Use it when you need superior visual output or advanced graphics features.