# Typf Architecture

Typf is a modular text rendering library with a three-stage pipeline: **Shaping → Rendering → Export**.

## Overview

```
                    ┌─────────────────────────────────────────────────────┐
                    │                      Pipeline                       │
                    └─────────────────────────────────────────────────────┘
                                            │
         ┌──────────────────────────────────┼──────────────────────────────────┐
         │                                  │                                  │
         ▼                                  ▼                                  ▼
┌─────────────────┐              ┌─────────────────┐              ┌─────────────────┐
│     Shaper      │              │    Renderer     │              │    Exporter     │
│                 │              │                 │              │                 │
│ text → glyphs   │    ────▶     │ glyphs → pixels │    ────▶     │ pixels → file   │
└─────────────────┘              └─────────────────┘              └─────────────────┘
         │                                  │                                  │
    ┌────┴────┐                    ┌────────┼────────┐                   ┌─────┴─────┐
    │         │                    │        │        │                   │           │
    ▼         ▼                    ▼        ▼        ▼                   ▼           ▼
 HarfBuzz   ICU+HB              Opixa    Skia    Zeno    SVG          PNG         SVG
  (hb)     (icu-hb)            (opixa)  (skia)  (zeno)  (svg)        (png)       (svg)
```

## Core Crates

### `typf-core` — The Foundation

Central types and traits that all components share:

- **`Pipeline`**: Chains shaper → renderer → exporter
- **`Shaper` trait**: Text → positioned glyphs
- **`Renderer` trait**: Glyphs → pixels or vectors
- **`Exporter` trait**: Pixels/vectors → file bytes
- **`FontRef` trait**: Abstraction over font data access
- **`ShapingParams`**: Font size, features, variations, language
- **`RenderParams`**: Colors, padding, antialiasing, output mode
- **`ShapingResult`**: Positioned glyphs with metrics
- **`RenderOutput`**: Bitmap or vector data

### `typf-fontdb` — Font Loading

Loads fonts from files or memory with:
- TrueType Collection (TTC) face index support
- On-demand `FontRef` creation (no memory leaks)
- Units-per-em and glyph metrics access

### `typf-unicode` — Text Analysis

Unicode processing for complex scripts:
- Bidirectional text (Arabic, Hebrew)
- Script detection and segmentation
- Cluster boundaries

### `typf-export` — File Encoding

Converts rendered output to file formats:
- PNG export with proper IHDR/IDAT/IEND
- Format validation and error handling

## Backend Shapers

Shapers transform text into positioned glyphs, applying OpenType features and script rules.

| Backend | Crate | Description |
|---------|-------|-------------|
| **HarfBuzz** | `typf-shape-hb` | Industry-standard shaper via harfbuzz-rs |
| **ICU+HarfBuzz** | `typf-shape-icu-hb` | HarfBuzz with ICU for enhanced Unicode |
| **CoreText** | `typf-shape-ct` | macOS native shaping |
| **None** | `typf-shape-none` | Pass-through (testing only) |

All shapers support:
- OpenType features (`liga`, `kern`, `smcp`, etc.)
- Variable font axes (`wght`, `wdth`, etc.)
- Script/language tags
- Bidirectional text
- Shaping cache for repeated text

## Backend Renderers

Renderers convert positioned glyphs into visual output.

| Backend | Crate | Output | Description |
|---------|-------|--------|-------------|
| **Opixa** | `typf-render-opixa` | Bitmap | Pure Rust, high-quality antialiasing, SIMD |
| **Skia** | `typf-render-skia` | Bitmap | Uses tiny-skia for path rendering |
| **Zeno** | `typf-render-zeno` | Bitmap | Pure Rust, zeno rasterizer |
| **SVG** | `typf-render-svg` | Vector | Scalable vector output |
| **JSON** | `typf-render-json` | Data | Glyph data for debugging |
| **CoreGraphics** | `typf-render-cg` | Bitmap | macOS native rendering |
| **Color** | `typf-render-color` | Bitmap | Color glyph support (COLR/CPAL) |

### Renderer Features

All raster renderers share:
- Canvas sizing from actual glyph bounds (ascent/descent/bbox)
- 32-bit glyph ID support (>65k glyphs)
- Foreground/background colors
- Configurable padding
- Antialiasing options

The SVG renderer additionally supports:
- Variable font variations
- Perfect scaling (vector output)

## Data Flow

```
1. Input
   └─► "Hello مرحبا" + font.ttf + params

2. Shaping (HarfBuzz)
   ├─► Bidi analysis: [LTR: "Hello "] [RTL: "مرحبا"]
   ├─► Cluster mapping: H→glyph72, e→glyph68, ...
   ├─► OpenType features: ligatures, kerning
   └─► ShapingResult: [{id:72, x:0, y:0}, {id:68, x:12, y:0}, ...]

3. Rendering (Opixa)
   ├─► Load glyph outlines from font
   ├─► Calculate canvas from actual bounds
   ├─► Rasterize with antialiasing
   └─► RenderOutput::Bitmap { width, height, data }

4. Export (PNG)
   ├─► Encode bitmap as PNG
   └─► Vec<u8> ready to write
```

## Configuration

### ShapingParams

```rust
ShapingParams {
    size: 24.0,                           // Font size in pixels
    language: Some("ar".into()),          // Language tag
    script: Some("arab".into()),          // Script tag
    features: vec![("liga".into(), 1)],   // OpenType features
    variations: vec![("wght".into(), 700.0)], // Variable font axes
    direction: Direction::Auto,           // Text direction
    letter_spacing: 0.0,                  // Extra spacing
}
```

### RenderParams

```rust
RenderParams {
    foreground: Color::black(),           // Text color
    background: Some(Color::white()),     // Canvas color
    padding: 10,                          // Pixels around text
    antialias: true,                      // Smooth edges
    variations: vec![("wght".into(), 700.0)], // For SVG renderer
    color_palette: 0,                     // CPAL palette index
    output: RenderMode::Bitmap,           // Vector: RenderMode::Vector(VectorFormat::Svg)
}
```

## Platform Support

| Feature | macOS | Linux | Windows |
|---------|-------|-------|---------|
| HarfBuzz shaping | ✓ | ✓ | ✓ |
| CoreText shaping | ✓ | - | - |
| Opixa rendering | ✓ | ✓ | ✓ |
| Skia rendering | ✓ | ✓ | ✓ |
| CoreGraphics rendering | ✓ | - | - |
| System font discovery | ✓ | ✓ | ✓ |

## Caching

### Shaping Cache

The `ShapingCache` avoids re-shaping identical text:
- Key: text + font ID + size + features + variations
- Value: `ShapingResult` (positioned glyphs)
- Configurable capacity
- Hit/miss statistics

### Glyph Cache

Renderers can cache rasterized glyphs:
- Key: font ID + glyph ID + size + style
- Value: rasterized bitmap
- Per-renderer implementation

## Error Handling

All operations return `Result<T, TypfError>`:

```rust
pub enum TypfError {
    FontError(FontError),     // Font loading/parsing
    ShapingError(ShapingError), // Text shaping
    RenderError(RenderError),  // Rasterization
    ExportError(ExportError),  // File encoding
    ConfigError(String),       // Invalid configuration
}
```

## CLI Usage

```bash
# Render text to PNG
typf render -f font.ttf -t "Hello" -o hello.png

# Get font info
typf info -f font.ttf --shapers --renderers

# Batch processing
typf batch -c config.json
```

## Python Bindings

```python
import typf

# Simple rendering
png_data = typf.render_simple("Hello", font_path, size=24)

# With full control
result = typf.render(
    text="Hello",
    font=font_path,
    size=24,
    features={"liga": 1},
    variations={"wght": 700},
)
```

## See Also

- [README.md](README.md) — Getting started
- [CONTRIBUTING.md](CONTRIBUTING.md) — Development guide
- [API Documentation](https://docs.rs/typf) — Crate docs
