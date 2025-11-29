# Rust API

TYPF's Rust API provides high-performance text rendering with zero-copy operations and compile-time safety.

## Quick Start

```rust
use typf::{Pipeline, PipelineBuilder};

// Build a rendering pipeline
let mut pipeline = PipelineBuilder::new()
    .with_shaper("harfbuzz")?
    .with_renderer("opixa")?
    .build();

// Render text
let result = pipeline.render_text("Hello World", "font.ttf")?;
```

## Core Types

### Pipeline

The main interface for text rendering operations.

```rust
pub struct Pipeline {
    shaper: Arc<dyn Shaper>,
    renderer: Arc<dyn Renderer>,
    font_db: Arc<FontDatabase>,
    cache: Arc<RenderCache>,
}

impl Pipeline {
    /// Create a new pipeline with defaults
    pub fn new() -> Result<Self>;
    
    /// Render text to bitmap
    pub fn render_text(&mut self, text: &str, font_path: &str) 
        -> Result<RenderOutput>;
    
    /// Render with custom settings
    pub fn render_with_options(&mut self, 
        text: &str, 
        font_path: &str,
        options: &RenderOptions
    ) -> Result<RenderOutput>;
}
```

### PipelineBuilder

Configure your rendering pipeline with the builder pattern.

```rust
pub struct PipelineBuilder {
    shaper: Option<Arc<dyn Shaper>>,
    renderer: Option<Arc<dyn Renderer>>,
    font_db: Option<Arc<FontDatabase>>,
    cache: Option<Arc<RenderCache>>,
}

impl PipelineBuilder {
    pub fn new() -> Self;
    
    /// Select shaping backend
    pub fn with_shaper(mut self, name: &str) -> Result<Self>;
    
    /// Select rendering backend  
    pub fn with_renderer(mut self, name: &str) -> Result<Self>;
    
    /// Configure font database
    pub fn with_font_db(mut self, db: FontDatabase) -> Self;
    
    /// Set up cache
    pub fn with_cache(mut self, cache: RenderCache) -> Self;
    
    /// Build the pipeline
    pub fn build(self) -> Result<Pipeline>;
}
```

## Rendering Configuration

### RenderOptions

Control how text gets rendered.

```rust
pub struct RenderOptions {
    pub font_size: f32,           // Size in pixels
    pub dpi: f32,                 // Output resolution
    pub width: u32,               // Image width
    pub height: u32,              // Image height
    pub color: Color,             // Text color
    pub background: Color,        // Background color
    pub hinting: HintingMode,     // Font hinting
    pub antialiasing: bool,       // Edge smoothing
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            dpi: 72.0,
            width: 800,
            height: 600,
            color: Color::BLACK,
            background: Color::TRANSPARENT,
            hinting: HintingMode::Normal,
            antialiasing: true,
        }
    }
}
```

### Color

 RGBA color with alpha support.

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8, 
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };
    
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}
```

## Output Types

### RenderOutput

The result of a rendering operation.

```rust
pub struct RenderOutput {
    pub bitmap: BitmapData,       // Pixel data
    pub metadata: RenderMetadata, // Rendering info
    pub glyphs: Vec<GlyphInfo>,   // Glyph positions
}

pub struct BitmapData {
    pub width: u32,
    pub height: u32,
    pub stride: u32,              // Bytes per row
    pub format: PixelFormat,      // Color format
    pub data: Vec<u8>,            // Raw pixel data
}

pub enum PixelFormat {
    Gray,                         // 8-bit grayscale
    GrayAlpha,                    // 8-bit gray + 8-bit alpha
    RGB,                          // 24-bit RGB
    RGBA,                         // 32-bit RGBA
}
```

### GlyphInfo

Information about each rendered glyph.

```rust
pub struct GlyphInfo {
    pub glyph_id: u32,            // Glyph index in font
    pub codepoint: u32,           // Unicode character
    pub x: f32,                   // X position
    pub y: f32,                   // Y position
    pub width: f32,               // Glyph width
    pub height: f32,              // Glyph height
    pub advance: f32,             // X advance to next glyph
}
```

## Font Management

### FontDatabase

Load and manage fonts efficiently.

```rust
pub struct FontDatabase {
    fonts: DashMap<String, Arc<Font>>,
    system_fonts: bool,
}

impl FontDatabase {
    /// Create empty database
    pub fn new() -> Self;
    
    /// Create with system fonts
    pub fn with_system_fonts() -> Result<Self>;
    
    /// Load font from file
    pub fn load_font(&self, path: &str) -> Result<Arc<Font>>;
    
    /// Load font from bytes
    pub fn load_font_bytes(&self, 
        name: &str, 
        data: Vec<u8>
    ) -> Result<Arc<Font>>;
    
    /// Get font by name
    pub fn get_font(&self, name: &str) -> Option<Arc<Font>>;
    
    /// List available fonts
    pub fn list_fonts(&self) -> Vec<String>;
}
```

### Font

Represents a loaded font.

```rust
pub struct Font {
    pub name: String,
    pub family: String,
    pub style: FontStyle,
    pub metrics: FontMetrics,
    data: &'static [u8],          // Memory-mapped font data
}

pub struct FontMetrics {
    pub ascender: f32,            // Height of ascender
    pub descender: f32,           // Depth of descender  
    pub line_gap: f32,            // Space between lines
    pub units_per_em: u16,        // Font design units
}

pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
    Bold,
    BoldItalic,
}
```

## Shaping Backends

### Shaper Trait

All shapers implement this trait.

```rust
pub trait Shaper: Send + Sync {
    /// Shape text into glyphs
    fn shape(&self, 
        text: &str, 
        font: &Font, 
        options: &ShapeOptions
    ) -> Result<ShapingResult>;
    
    /// Get shaper name
    fn name(&self) -> &str;
    
    /// Check if script is supported
    fn supports_script(&self, script: UnicodeScript) -> bool;
}
```

### ShapingResult

Output from the shaping stage.

```rust
pub struct ShapingResult {
    pub glyphs: Vec<PositionedGlyph>,
    pub clusters: Vec<TextCluster>,
    pub direction: TextDirection,
    pub script: UnicodeScript,
}

pub struct PositionedGlyph {
    pub glyph_id: u32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub x_advance: f32,
    pub y_advance: f32,
    pub cluster: u32,
}
```

## Rendering Backends

### Renderer Trait

All renderers implement this trait.

```rust
pub trait Renderer: Send + Sync {
    /// Render shaped glyphs to bitmap
    fn render(&self,
        glyphs: &[PositionedGlyph],
        font: &Font,
        options: &RenderOptions
    ) -> Result<BitmapData>;
    
    /// Get renderer name
    fn name(&self) -> &str;
    
    /// Check if format is supported
    fn supports_format(&self, format: PixelFormat) -> bool;
}
```

## Error Handling

### TypfError

Comprehensive error type for all operations.

```rust
#[derive(Debug, thiserror::Error)]
pub enum TypfError {
    #[error("Font loading failed: {0}")]
    FontLoad(String),
    
    #[error("Shaping failed: {0}")]  
    Shaping(String),
    
    #[error("Rendering failed: {0}")]
    Rendering(String),
    
    #[error("Backend not available: {0}")]
    BackendUnavailable(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Performance Features

### RenderCache

Cache rendered text for repeated use.

```rust
pub struct RenderCache {
    glyph_cache: LruCache<GlyphKey, BitmapData>,
    text_cache: LruCache<TextKey, RenderOutput>,
    max_memory: usize,
}

impl RenderCache {
    /// Create cache with memory limit
    pub fn new(max_memory: usize) -> Self;
    
    /// Get cached glyph
    pub fn get_glyph(&self, key: &GlyphKey) -> Option<BitmapData>;
    
    /// Cache glyph data
    pub fn put_glyph(&mut self, key: GlyphKey, data: BitmapData);
    
    /// Get cached text
    pub fn get_text(&self, key: &TextKey) -> Option<RenderOutput>;
    
    /// Cache text result
    pub fn put_text(&mut self, key: TextKey, output: RenderOutput);
    
    /// Clear all cached data
    pub fn clear(&mut self);
}
```

### Parallel Processing

Render multiple texts concurrently.

```rust
use rayon::prelude::*;

// Parallel batch rendering
let texts = vec!["Hello", "World", "TYPF"];
let results: Vec<Result<RenderOutput>> = texts
    .par_iter()
    .map(|text| pipeline.render_text(text, "font.ttf"))
    .collect();
```

## Advanced Usage

### Custom Shapers

```rust
struct CustomShaper {
    // Your shaping logic
}

impl Shaper for CustomShaper {
    fn shape(&self, 
        text: &str, 
        font: &Font, 
        options: &ShapeOptions
    ) -> Result<ShapingResult> {
        // Implement custom shaping
        todo!()
    }
    
    fn name(&self) -> &str {
        "custom"
    }
}

// Register with pipeline
let pipeline = PipelineBuilder::new()
    .with_custom_shaper(Arc::new(CustomShaper::new()))
    .build()?;
```

### Custom Renderers

```rust
struct CustomRenderer {
    // Your rendering logic
}

impl Renderer for CustomRenderer {
    fn render(&self,
        glyphs: &[PositionedGlyph],
        font: &Font,  
        options: &RenderOptions
    ) -> Result<BitmapData> {
        // Implement custom rendering
        todo!()
    }
    
    fn name(&self) -> &str {
        "custom"
    }
}
```

## Feature Flags

Control what gets compiled:

```toml
[dependencies.typf]
features = [
    "shaping-harfbuzz",     # HarfBuzz text shaping
    "render-opixa",          # Opixa rasterizer
    "render-skia",          # Skia renderer
    "export-png",           # PNG export
    "export-svg",           # SVG export
    "system-fonts",         # System font discovery
    "fontdb",               # Font database
    "cache",                # Caching system
]
```

Minimal build:

```toml
[dependencies.typf]
features = [
    "shaping-none",         # No shaping (identity only)
    "render-opixa",          # Basic rasterizer
    "export-pnm",           # PNM export for testing
]
```

## Examples

### Basic Text Rendering

```rust
use typf::{Pipeline, RenderOptions, Color};

fn main() -> Result<()> {
    let mut pipeline = Pipeline::new()?;
    
    let options = RenderOptions {
        font_size: 24.0,
        width: 400,
        height: 100,
        color: Color::BLACK,
        background: Color::WHITE,
        ..Default::default()
    };
    
    let result = pipeline.render_with_options(
        "Hello TYPF!", 
        "Roboto-Regular.ttf",
        &options
    )?;
    
    println!("Rendered {}x{} image", 
        result.bitmap.width, 
        result.bitmap.height);
    
    Ok(())
}
```

### Batch Processing

```rust
use typf::{Pipeline, FontDatabase};

fn process_texts(texts: &[String]) -> Result<Vec<RenderOutput>> {
    let font_db = FontDatabase::with_system_fonts()?;
    let mut pipeline = PipelineBuilder::new()
        .with_font_db(font_db)
        .build()?;
    
    let mut results = Vec::new();
    
    for text in texts {
        let result = pipeline.render_text(text, "Arial")?;
        results.push(result);
    }
    
    Ok(results)
}
```

---

The Rust API gives you maximum control over the text rendering pipeline with zero-copy operations and compile-time safety. Use the builder pattern to configure exactly what you need.