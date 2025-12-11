# Typf Quickstart Guide

Get typf running in your Rust project in 5 minutes.

## Add to Cargo.toml

```toml
[dependencies]
# Minimal: just shaping and rendering
typf = { git = "https://github.com/fontlaborg/typf.git", features = ["minimal"] }

# Recommended: HarfBuzz shaping + Opixa rendering + PNG export
typf = { git = "https://github.com/fontlaborg/typf.git", features = ["shaping-hb", "render-opixa", "export-png"] }

# Full: all backends
typf = { git = "https://github.com/fontlaborg/typf.git", features = ["full"] }
```

### Feature flags

| Feature | Description |
|---------|-------------|
| `minimal` | NoneShaper + OpixaRenderer (pure Rust, no deps) |
| `shaping-hb` | HarfBuzz shaper (complex scripts) |
| `shaping-ct` | CoreText shaper (macOS only) |
| `shaping-icu-hb` | ICU + HarfBuzz (best Unicode support) |
| `render-opixa` | Opixa rasterizer (pure Rust, SIMD) |
| `render-zeno` | Zeno rasterizer (pure Rust, 256-level AA) |
| `render-skia` | tiny-skia renderer (color fonts) |
| `render-vello-cpu` | Vello CPU renderer (high quality) |
| `render-cg` | CoreGraphics renderer (macOS only) |
| `export-png` | PNG export via image crate |
| `export-svg` | SVG export |
| `fontdb` | System font discovery |

## Basic usage

```rust
use std::sync::Arc;
use typf::prelude::*;
use typf_fontdb::TypfFontFace;
use typf_shape_hb::HarfBuzzShaper;
use typf_render_opixa::OpixaRenderer;
use typf_export::PngExporter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load a font
    let font = TypfFontFace::from_file("path/to/font.ttf")?;
    let font: Arc<dyn FontRef> = Arc::new(font);

    // 2. Create shaper and renderer
    let shaper = HarfBuzzShaper::new();
    let renderer = OpixaRenderer::new();
    let exporter = PngExporter::new();

    // 3. Configure shaping
    let shaping_params = ShapingParams {
        size: 48.0,
        direction: Direction::LeftToRight,
        ..Default::default()
    };

    // 4. Shape text into positioned glyphs
    let shaped = shaper.shape("Hello, World!", font.clone(), &shaping_params)?;

    // 5. Render glyphs to bitmap
    let render_params = RenderParams {
        foreground: Color::black(),
        background: Some(Color::white()),
        padding: 10,
        ..Default::default()
    };
    let rendered = renderer.render(&shaped, font, &render_params)?;

    // 6. Export to PNG
    let png_bytes = exporter.export(&rendered)?;
    std::fs::write("output.png", png_bytes)?;

    Ok(())
}
```

## RTL and complex scripts

```rust
use typf::prelude::*;

// Arabic text (right-to-left)
let shaping_params = ShapingParams {
    size: 48.0,
    direction: Direction::RightToLeft,
    language: Some("ar".to_string()),
    script: Some("Arab".to_string()),
    ..Default::default()
};

let shaped = shaper.shape("مرحبا بالعالم", font.clone(), &shaping_params)?;
```

## Variable fonts

```rust
let shaping_params = ShapingParams {
    size: 48.0,
    variations: vec![
        ("wght".to_string(), 700.0),  // Bold
        ("wdth".to_string(), 75.0),   // Condensed
    ],
    ..Default::default()
};

let render_params = RenderParams {
    variations: vec![
        ("wght".to_string(), 700.0),
        ("wdth".to_string(), 75.0),
    ],
    ..Default::default()
};
```

## Caching

Typf includes two-level caches (L1 hot + L2 LRU) for both shaping and rendering results. **Caching is disabled by default.**

### Enable caching

```rust
use typf::cache_config;

// Enable caching globally
cache_config::set_caching_enabled(true);

// Now shape/render operations will cache results
let shaped1 = shaper.shape("Hello", font.clone(), &params)?;
let shaped2 = shaper.shape("Hello", font.clone(), &params)?; // Cache hit!

// Check if caching is enabled
if cache_config::is_caching_enabled() {
    println!("Caching is ON");
}

// Disable caching
cache_config::set_caching_enabled(false);
```

### Environment variable

```bash
# Enable caching at startup
TYPF_CACHE=1 cargo run --release
```

### When to enable caching

| Use case | Enable caching? |
|----------|-----------------|
| One-shot renders (CLI) | No (default) |
| Interactive UI with re-renders | Yes |
| Server rendering same content | Yes |
| Batch processing unique texts | No |
| Memory-constrained systems | No |

### Cache behavior

- **Shaping cache**: Keys on text + font + size + language + features + variations
- **Glyph cache**: Keys on shaped result + render params + font
- **L1 cache**: Fast HashMap (~50ns access), small capacity
- **L2 cache**: LRU eviction, larger capacity

When caching is disabled, `get()` returns `None` and `insert()` is a no-op.

## Using the Pipeline builder

For more complex setups, use the Pipeline builder:

```rust
use typf::{Pipeline, ShapingParams, RenderParams};
use std::sync::Arc;

let pipeline = Pipeline::builder()
    .shaper(Arc::new(HarfBuzzShaper::new()))
    .renderer(Arc::new(OpixaRenderer::new()))
    .exporter(Arc::new(PngExporter::new()))
    .build()?;

let output = pipeline.process(
    "Hello, World!",
    font,
    &ShapingParams::default(),
    &RenderParams::default(),
)?;
```

## Linra (single-pass rendering)

For maximum performance on macOS, use linra which combines shaping and rendering:

```rust
#[cfg(target_os = "macos")]
{
    use typf_os_mac::CoreTextLinraRenderer;
    use typf_core::linra::{LinraRenderer, LinraRenderParams};

    let linra = CoreTextLinraRenderer::new();

    let params = LinraRenderParams {
        size: 48.0,
        direction: Direction::LeftToRight,
        foreground: Color::black(),
        background: Some(Color::white()),
        padding: 10,
        ..Default::default()
    };

    let output = linra.render_text("Hello", font, &params)?;
}
```

## Color fonts

Control which glyph sources are used:

```rust
use typf::{GlyphSource, GlyphSourcePreference, RenderParams};

// Prefer COLR over SVG
let render_params = RenderParams {
    glyph_sources: GlyphSourcePreference::from_parts(
        vec![GlyphSource::Colr1, GlyphSource::Colr0, GlyphSource::Svg],
        [],  // no denied sources
    ),
    ..Default::default()
};

// Force outline-only (disable color)
let render_params = RenderParams {
    glyph_sources: GlyphSourcePreference::from_parts(
        vec![GlyphSource::Glyf, GlyphSource::Cff, GlyphSource::Cff2],
        [GlyphSource::Colr0, GlyphSource::Colr1, GlyphSource::Svg,
         GlyphSource::Sbix, GlyphSource::Cbdt],
    ),
    ..Default::default()
};
```

## Error handling

```rust
use typf::error::{TypfError, Result};

fn render_text(text: &str) -> Result<Vec<u8>> {
    let shaped = shaper.shape(text, font.clone(), &params)
        .map_err(|e| TypfError::ShapingFailed(e))?;

    let rendered = renderer.render(&shaped, font, &render_params)
        .map_err(|e| TypfError::RenderingFailed(e))?;

    exporter.export(&rendered)
        .map_err(|e| TypfError::ExportFailed(e))
}
```

## Next steps

- [README.md](./README.md) - Full feature overview
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Pipeline and backend details
- [src_docs/](./src_docs/) - Comprehensive documentation
- [examples/](./examples/) - More code samples
