# TYPF API Documentation

## Rustdoc Integration

The complete API documentation for TYPF is auto-generated using Rust's built-in documentation system. This includes:

- Module documentation
- Function signatures and parameters
- Type definitions and traits
- Usage examples from doc comments
- Implementation details

## Accessing the Documentation

### Local Generation

Generate the documentation locally:

```bash
# Generate docs for all crates
cargo doc --workspace --no-deps

# Generate with private items documented
cargo doc --workspace --document-private-items

# Open in browser
cargo doc --open
```

### Online Documentation

Once published to crates.io, documentation will be available at:
- https://docs.rs/typf-api
- https://docs.rs/typf-core
- https://docs.rs/typf-batch

## Key Modules

### typf-api
The main public API crate providing:
- `RasterizeOptions` - Configuration for text rendering
- `RenderBuilder` - Fluent API for rendering configuration
- Backend traits and implementations

### typf-core
Core rendering logic:
- `Font` - Font loading and management
- `GlyphCache` - Glyph caching system
- `TextLayout` - Text shaping and positioning
- Rendering pipelines

### typf-batch
Batch processing utilities:
- Parallel rendering
- Multi-font processing
- Performance optimizations

### typf-backend-*
Backend implementations:
- `typf-backend-zeno` - Pure Rust rasterizer
- `typf-backend-harfbuzz` - HarfBuzz integration
- `typf-backend-skia` - Skia graphics library
- `typf-backend-coretext` - macOS CoreText

## API Categories

### Rendering API
```rust
// Simple rendering
let image = typf::render_text("Hello", "path/to/font.ttf", 72.0)?;

// Advanced configuration
let image = RenderBuilder::new()
    .font_path("font.ttf")
    .text("Hello World")
    .size(120.0)
    .width(800)
    .height(200)
    .coordinates(vec![("wght", 700.0)])
    .backend("harfbuzz")
    .build()
    .render()?;
```

### Font Inspection
```rust
// Get font metrics
let metrics = typf::get_font_metrics("font.ttf")?;

// List variable axes
let axes = typf::get_variable_axes("font.ttf")?;

// Check glyph coverage
let has_glyph = typf::has_glyph("font.ttf", 'A')?;
```

### Batch Processing
```rust
// Render multiple texts
let results = typf::batch_render(texts, fonts, options)?;

// Parallel processing
let images = typf::parallel_render(configs)?;
```

## Type Safety

TYPF uses Rust's type system for safety:

### Result Types
All operations return `Result<T, TypfError>`:
```rust
pub enum TypfError {
    IoError(std::io::Error),
    FontError(String),
    RenderError(String),
    InvalidParameter(String),
    BackendUnavailable(String),
}
```

### Builder Pattern
Configuration uses the builder pattern for ergonomics:
```rust
impl RenderBuilder {
    pub fn font_path(mut self, path: impl AsRef<Path>) -> Self { ... }
    pub fn text(mut self, text: impl Into<String>) -> Self { ... }
    pub fn size(mut self, size: f32) -> Self { ... }
    // ... more configuration methods
}
```

### FFI Safety
C API with proper null checks and error codes:
```rust
#[no_mangle]
pub unsafe extern "C" fn typf_render(
    text: *const c_char,
    font_path: *const c_char,
    size: f32,
    out_width: *mut u32,
    out_height: *mut u32,
) -> *mut u8 { ... }
```

## Performance Characteristics

### Memory Usage
- Glyph cache: ~10-50MB per font
- Image buffers: width × height bytes
- Backend overhead: 5-20MB

### Rendering Speed
Typical performance for 100×30px text:
- Zeno: 0.5-1ms
- HarfBuzz: 1-2ms
- Skia: 2-3ms
- CoreText: 1-2ms

### Caching
- Glyph rasterization cached
- Shaping results cached for repeated text
- Thread-local caches for parallel rendering

## Examples

### Basic Usage
```rust
use typf::render_text;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let image = render_text("Hello, TYPF!", "Arial.ttf", 72.0)?;
    // image is a Vec<u8> of grayscale pixels
    Ok(())
}
```

### Variable Font
```rust
use typf::{RenderBuilder, Coordinate};

let image = RenderBuilder::new()
    .font_path("Roboto-Flex.ttf")
    .text("Variable!")
    .coordinates(vec![
        Coordinate::new("wght", 300.0),
        Coordinate::new("wdth", 125.0),
    ])
    .render()?;
```

### Custom Backend
```rust
use typf::{RenderBuilder, Backend};

let image = RenderBuilder::new()
    .font_path("font.ttf")
    .text("Custom rendering")
    .backend(Backend::Skia)
    .antialiasing(true)
    .hinting(false)
    .render()?;
```

## Thread Safety

TYPF is designed for concurrent use:
- Font objects are `Send + Sync`
- Rendering is thread-safe
- Caches use interior mutability with proper synchronization

## Error Handling

Comprehensive error types with context:
```rust
match typf::render_text("Test", "font.ttf", 72.0) {
    Ok(image) => { /* use image */ },
    Err(TypfError::FontError(msg)) => eprintln!("Font issue: {}", msg),
    Err(TypfError::BackendUnavailable(name)) => {
        eprintln!("Backend {} not available", name)
    },
    Err(e) => eprintln!("Rendering failed: {}", e),
}
```

## Platform Support

| Platform | Backends Available |
|----------|-------------------|
| Linux | Zeno, HarfBuzz, Skia |
| macOS | Zeno, HarfBuzz, Skia, CoreText |
| Windows | Zeno, HarfBuzz, Skia |
| WebAssembly | Zeno, HarfBuzz |

## Integration with FontSimi

TYPF is the rendering engine for FontSimi:
```python
# Python usage via PyO3 bindings
import typf

# Render text
image = typf.render("Hello", "font.ttf", size=72)

# Get font info
metrics = typf.get_metrics("font.ttf")
axes = typf.get_variable_axes("font.ttf")
```

## Contributing

See the main README for contribution guidelines. Key points:
- All public APIs must be documented
- Examples in doc comments are tested
- Maintain backward compatibility
- Add tests for new features