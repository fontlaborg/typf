---
title: Backend Architecture
icon: lucide/puzzle
tags:
  - Backends
  - Architecture
  - Implementation
---

# Backend Architecture

Backends implement the pipeline stages. Mix and match them for your needs.

## Core Traits

All backends implement these traits:

```rust
pub trait Shaper: Send + Sync {
    fn shape(&self, text: &ProcessedText, font: &FontHandle, options: &ShapeOptions) -> Result<ShapingResult>;
    fn name(&self) -> &str;
    fn supports_script(&self, script: Script) -> bool;
}

pub trait Renderer: Send + Sync {
    fn render(&self, glyphs: &[Glyph], options: &RenderOptions) -> Result<RenderOutput>;
    fn name(&self) -> &str;
    fn supports_format(&self, format: PixelFormat) -> bool;
}

pub trait Exporter: Send + Sync {
    fn export(&self, output: &RenderOutput, options: &ExportOptions) -> Result<ExportResult>;
    fn name(&self) -> &str;
    fn supported_formats(&self) -> &[ExportFormat];
}
```

## Shaping Backends

### NoneShaper
The simplest shaper. No shaping, just character-to-glyph mapping.

```rust
pub struct NoneShaper;
impl Shaper for NoneShaper {
    fn shape(&self, text: &ProcessedText, font: &FontHandle, options: &ShapeOptions) -> Result<ShapingResult> {
        // Map characters directly to glyph IDs
        let glyphs: Vec<Glyph> = text.text.chars()
            .map(|c| font.get_glyph(c, 0))
            .collect();
        
        Ok(ShapingResult {
            glyphs,
            advances: vec![font.units_per_em(); glyphs.len()],
            positions: vec![Position::default(); glyphs.len()],
            clusters: (0..text.text.len()).collect(),
            direction: text.base_direction,
            script: text.script.unwrap_or(Script::Latin),
        })
    }
}
```

**Use when:** You need basic Latin text or are testing other components.

**Limitations:** No ligatures, no complex script support, no kerning.

### HarfBuzz Shaper
Industry-standard text shaping.

```rust
pub struct HarfBuzzShaper {
    font_cache: Arc<HarfBuzzFontCache>,
    buffer_pool: Arc<BufferPool>,
}

impl Shaper for HarfBuzzShaper {
    fn shape(&self, text: &ProcessedText, font: &FontHandle, options: &ShapeOptions) -> Result<ShapingResult> {
        let mut buffer = self.buffer_pool.get();
        buffer.set_text(&text.text, text.script.unwrap_or(Script::Latin), text.base_direction);
        buffer.set_font(font.harfbuzz_font());
        
        // Apply features
        for feature in &options.features {
            buffer.add_feature(feature.tag, feature.value, feature.start, feature.end);
        }
        
        buffer.shape();
        
        Ok(self.convert_hb_result(&buffer))
    }
}
```

**Use when:** You need proper shaping for any script.

**Features:** Full Unicode support, ligatures, kerning, complex scripts.

### CoreText Shaper
macOS native shaping engine.

```rust
#[cfg(target_os = "macos")]
pub struct CoreTextShaper;
impl Shaper for CoreTextShaper {
    fn shape(&self, text: &ProcessedText, font: &FontHandle, options: &ShapeOptions) -> Result<ShapingResult> {
        let attributed_string = self.create_attributed_string(text, font, options);
        let line = CTLineCreateWithAttributedString(attrib_string);
        let runs = CTLineGetGlyphRuns(line);
        
        self.extract_glyphs(runs)
    }
}
```

**Use when:** You're on macOS and want native performance.

**Features:** Seamless macOS font integration, optimal performance.

## Rendering Backends

### Opixa Renderer
Pure Rust rasterizer. No external dependencies.

```rust
pub struct OpixaRenderer {
    rasterizer: OpixaRasterizer,
    scan_converter: ScanConverter,
}

impl Renderer for OpixaRenderer {
    fn render(&self, glyphs: &[Glyph], options: &RenderOptions) -> Result<RenderOutput> {
        // Convert glyphs to outlines
        let outlines: Vec<Outline> = glyphs.iter()
            .map(|g| self.glyph_to_outline(g))
            .collect();
        
        // Rasterize outlines to bitmap
        let bitmap = self.rasterizer.rasterize(&outlines, options)?;
        
        Ok(RenderOutput {
            data: RenderData::Bitmap(bitmap),
            width: bitmap.width,
            height: bitmap.height,
            format: options.pixel_format,
            dpi: options.dpi,
            transform: options.transform,
        })
    }
}
```

**Use when:** You need fast rasterization without dependencies.

**Features:** Anti-aliasing, subpixel rendering, parallel processing.

### Skia Renderer
Cross-platform GPU-accelerated rendering.

```rust
pub struct SkiaRenderer {
    surface: Surface,
    canvas: Canvas,
    paint: Paint,
}

impl Renderer for SkiaRenderer {
    fn render(&self, glyphs: &[Glyph], options: &RenderOptions) -> Result<RenderOutput> {
        let mut canvas = self.canvas.clone();
        canvas.clear(&options.background_color);
        
        for glyph in glyphs {
            let positioned_glyph = self.position_glyph(glyph, &options.transform);
            canvas.draw_glyph(positioned_glyph, &self.paint);
        }
        
        let image = canvas.surface().image_snapshot();
        let data = image.encode_to_data(options.pixel_format)?;
        
        Ok(RenderOutput {
            data: RenderData::SkiaImage(data),
            width: image.width(),
            height: image.height(),
            format: options.pixel_format,
            dpi: options.dpi,
            transform: options.transform,
        })
    }
}
```

**Use when:** You need GPU acceleration or advanced effects.

**Features:** GPU rendering, complex effects, cross-platform.

### Vello Renderer
Modern compute-centric GPU renderer using wgpu.

```rust
pub struct VelloRenderer {
    gpu: GpuContext,       // wgpu Device + Queue
    config: VelloConfig,
}

impl Renderer for VelloRenderer {
    fn render(&self, glyphs: &[Glyph], options: &RenderOptions) -> Result<RenderOutput> {
        let mut scene = Scene::new(options.width, options.height);
        scene.set_paint(options.foreground_color);

        // Build glyph run from shaped glyphs
        let font_data = FontData::from_bytes(font.data());
        let glyph_run = scene.glyph_run(&font_data)
            .font_size(options.font_size)
            .glyphs(glyphs.iter().map(to_vello_glyph));

        scene.fill(glyph_run);

        // GPU render and readback
        let bitmap = self.render_to_bitmap(&scene)?;
        Ok(RenderOutput::Bitmap(bitmap))
    }
}
```

**Use when:** You need maximum throughput on GPU-equipped systems.

**Features:** GPU compute rendering, COLR/bitmap color fonts, glyph caching.

### Vello CPU Renderer
Pure Rust CPU renderer using vello_cpu. No GPU required.

```rust
pub struct VelloCpuRenderer {
    config: VelloCpuConfig,
}

impl Renderer for VelloCpuRenderer {
    fn render(&self, glyphs: &[Glyph], options: &RenderOptions) -> Result<RenderOutput> {
        let mut ctx = RenderContext::new();
        let mut pixmap = Pixmap::new(options.width, options.height);

        // Render glyphs via RenderContext
        ctx.glyph_run(&font_data)
            .font_size(options.font_size)
            .glyphs(glyphs.iter().map(to_vello_glyph))
            .render_into(&mut pixmap);

        Ok(RenderOutput::Bitmap(pixmap.into()))
    }
}
```

**Use when:** You need high-quality rendering without GPU dependencies.

**Features:** Pure Rust, no GPU, server-friendly, COLR/bitmap color fonts.

## Backend Registry

Find and create backends by name:

```rust
pub struct BackendRegistry {
    shapers: HashMap<String, Box<dyn Fn() -> Box<dyn Shaper>>>,
    renderers: HashMap<String, Box<dyn Fn() -> Box<dyn Renderer>>>,
    exporters: HashMap<String, Box<dyn Fn() -> Box<dyn Exporter>>>,
}

impl BackendRegistry {
    pub fn register_shaper<F>(&mut self, name: &str, factory: F) 
    where F: Fn() -> Box<dyn Shaper> + 'static {
        self.shapers.insert(name.to_string(), Box::new(factory));
    }
    
    pub fn create_shaper(&self, name: &str) -> Result<Box<dyn Shaper>> {
        self.shapers.get(name)
            .ok_or_else(|| BackendError::NotFound(name.to_string()))
            .map(|factory| factory())
    }
}
```

## Runtime Backend Selection

Choose backends at runtime:

```rust
pub struct PipelineBuilder {
    shaper_name: Option<String>,
    renderer_name: Option<String>,
    exporter_name: Option<String>,
    registry: Arc<BackendRegistry>,
}

impl PipelineBuilder {
    pub fn with_shaper(mut self, name: &str) -> Result<Self> {
        self.registry.create_shaper(name)?;
        self.shaper_name = Some(name.to_string());
        Ok(self)
    }
    
    pub fn build(self) -> Result<Pipeline> {
        let shaper = self.registry.create_shaper(self.shaper_name.unwrap_or("none"))?;
        let renderer = self.registry.create_renderer(self.renderer_name.unwrap_or("opixa"))?;
        let exporter = self.registry.create_exporter(self.exporter_name.unwrap_or("pnm"))?;
        
        Ok(Pipeline::new(shaper, renderer, exporter))
    }
}
```

## Platform Defaults

Automatic backend selection by platform:

```rust
#[cfg(target_os = "macos")]
fn default_shaper() -> &'static str { "mac" }  // CoreText

#[cfg(target_os = "windows")]
fn default_shaper() -> &'static str { "directwrite" }

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn default_shaper() -> &'static str { "harfbuzz" }

fn default_renderer() -> &'static str {
    if gpu_available() { "skia" } else { "opixa" }
}
```

## Backend Combinations

Common combinations:

| Use Case | Shaper | Renderer | Exporter | Performance |
|----------|--------|----------|----------|-------------|
| Fastest data | none | json | json | 25K ops/sec |
| GPU high-throughput | harfbuzz | vello | png | 10K+ ops/sec |
| Pure Rust quality | harfbuzz | vello-cpu | png | 3.5K ops/sec |
| Complex scripts | harfbuzz | zeno | png | 3K ops/sec |
| macOS best | mac | mac | png | 4K ops/sec |
| Pure Rust minimal | harfbuzz | opixa | pnm | 2K ops/sec |
| Web rendering | harfbuzz | skia | svg | 3.5K ops/sec |
| Mobile apps | mac | skia | png | 4K ops/sec |

## Performance Characteristics

| Backend | Memory | Speed | Quality | Platform |
|---------|--------|-------|---------|----------|
| NoneShaper | Low | 25K ops/sec | Basic | All |
| HarfBuzz | Medium | 4K ops/sec | High | All |
| ICU-HarfBuzz | Medium | 3.5K ops/sec | High | All |
| CoreText (mac) | Medium | 4.5K ops/sec | High | macOS only |
| Opixa | Low | 2K ops/sec | Medium | All |
| Skia | High | 3.5K ops/sec | High | All |
| Zeno | Medium | 3K ops/sec | High | All |
| Vello CPU | Medium | 3.5K ops/sec | High | All (pure Rust) |
| Vello GPU | Medium | 10K+ ops/sec | High | GPU required |
| CoreGraphics (mac) | High | 4K ops/sec | High | macOS only |
| JSON | Low | 25K ops/sec | Data only | All |

## Adding New Backends

Implement the trait and register:

```rust
// 1. Implement the trait
pub struct MyCustomShaper;
impl Shaper for MyCustomShaper {
    fn shape(&self, text: &ProcessedText, font: &FontHandle, options: &ShapeOptions) -> Result<ShapingResult> {
        // Your shaping logic
    }
}

// 2. Register the backend
registry.register_shaper("my_custom", || Box::new(MyCustomShaper));
```

## Error Handling

Backend-specific errors:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ShapingError {
    #[error("Font not supported by backend: {backend}")]
    UnsupportedFont { backend: String },
    #[error("Script not supported by backend: {backend}")]
    UnsupportedScript { backend: String },
    #[error("Backend internal error: {message}")]
    InternalError { backend: String, message: String },
}
```

## Testing Backends

Each backend includes tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_shaping() {
        let shaper = HarfBuzzShaper::new();
        let text = create_test_text("Hello");
        let font = load_test_font();
        
        let result = shaper.shape(&text, &font, &ShapeOptions::default());
        assert!(result.is_ok());
        
        let shaped = result.unwrap();
        assert!(!shaped.glyphs.is_empty());
    }
}
```

---

Backends implement pipeline stages. Choose the right combination for your needs.
