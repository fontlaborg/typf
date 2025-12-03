# Export Formats

Typf exports rendered text to multiple formats for different use cases.

## Available Formats

| Format | Type | Use Case | Size | Quality |
|--------|------|----------|------|---------|
| PNG | Raster | Web, documents | Medium | High |
| SVG | Vector | Web, print | Small | Excellent |
| PDF | Vector | Print, documents | Small | Excellent |
| PNM | Raster | Testing, debugging | Large | Medium |
| JSON | Data | Debugging, analysis | Medium | N/A |

## Quick Export

```rust
use typf_export::{PngExporter, SvgExporter, JsonExporter};

// PNG for raster output
let png_exporter = PngExporter::new();
let png_bytes = png_exporter.export(&render_output)?;

// SVG for vector output  
let svg_exporter = SvgExporter::new();
let svg_string = svg_exporter.export(&render_output)?;

// JSON for debugging
let json_exporter = JsonExporter::new();
let json_string = json_exporter.export(&render_output)?;
```

```python
import typf

# Simple export
renderer = typf.Typf()
renderer.render_text("Hello", "font.ttf", output="output.png")
renderer.render_text("Hello", "font.ttf", output="output.svg") 
renderer.render_text("Hello", "font.ttf", output="output.json")
```

## PNG Export

PNG provides compressed raster images with transparency support.

### PNG Options

```rust
pub struct PngOptions {
    pub compression: CompressionLevel,
    pub filter: FilterType,
    pub color_type: ColorType,
    pub bit_depth: BitDepth,
}

impl PngOptions {
    pub fn high_quality() -> Self {
        Self {
            compression: CompressionLevel::Best,
            filter: FilterType::Adaptive,
            color_type: ColorType::RGBA,
            bit_depth: BitDepth::Eight,
        }
    }
    
    pub fn web_optimized() -> Self {
        Self {
            compression: CompressionLevel::Fast,
            filter: FilterType::Sub,
            color_type: ColorType::RGB,
            bit_depth: BitDepth::Eight,
        }
    }
}
```

### PNG Usage

```rust
// High quality PNG for print
let png_options = PngOptions::high_quality();
let png_bytes = png_exporter.export_with_options(&output, png_options)?;

// Fast PNG for web
let web_options = PngOptions::web_optimized();
let web_png = png_exporter.export_with_options(&output, web_options)?;

// PNG with custom DPI metadata
let mut png_exporter = PngExporter::new();
png_exporter.set_dpi(300);
let print_png = png_exporter.export(&output)?;
```

## SVG Export

SVG creates scalable vector graphics perfect for web and print.

### Important Limitations

**CBDT Font Issue**: SVG export fails for CBDT bitmap fonts because they contain bitmap data, not vector outlines that can be converted to SVG paths.

```rust
// Error you'll see: "Glyph X not found"
// This happens because the SVG exporter looks for outline data that doesn't exist in CBDT fonts
```

**Solution**: Detect CBDT fonts and handle them appropriately:
```rust
fn safe_svg_export(output: &RenderOutput, font: &Font) -> Result<String> {
    if is_cbdt_font(font) {
        // Option 1: Return error with helpful message
        return Err(ExportError::CbdtNotSupported {
            message: "CBDT bitmap fonts cannot be exported as SVG paths".to_string(),
            suggestion: "Use PNG export for CBDT fonts or convert to COLR format".to_string(),
        });
        
        // Option 2: Embed bitmap as base64 images
        // return embed_cbdt_as_base64_png(output, font);
    }
    
    // Normal SVG export for outline fonts
    svg_exporter.export(output)
}
```

### SVG Options

```rust
pub struct SvgOptions {
    pub precision: u32,           // Decimal places
    pub optimize_paths: bool,     // Remove redundant points
    pub embed_fonts: bool,        // Include font data
    pub pretty_print: bool,       // Human-readable formatting
    pub viewbox: Option<Rect>,    // Custom viewbox
    pub handle_bitmap_fonts: BitmapFontStrategy, // New option
}

#[derive(Debug, Clone)]
pub enum BitmapFontStrategy {
    Error,        // Fail with descriptive error (default)
    Skip,         // Skip bitmap glyphs entirely
    EmbedAsPng,   // Embed bitmap glyphs as base64 PNG images
    Placeholder,  // Replace with placeholder rectangles
}

impl SvgOptions {
    pub fn web() -> Self {
        Self {
            precision: 6,
            optimize_paths: true,
            embed_fonts: false,
            pretty_print: true,
            viewbox: None,
            handle_bitmap_fonts: BitmapFontStrategy::Error,
        }
    }
    
    pub fn cbdt_compatible() -> Self {
        Self {
            precision: 6,
            optimize_paths: true,
            embed_fonts: false,
            pretty_print: true,
            viewbox: None,
            handle_bitmap_fonts: BitmapFontStrategy::EmbedAsPng,
        }
    }
    
    pub fn standalone() -> Self {
        Self {
            precision: 8,
            optimize_paths: false,
            embed_fonts: true,
            pretty_print: true,
            viewbox: None,
            handle_bitmap_fonts: BitmapFontStrategy::Error,
        }
    }
}
```

### SVG Features

```rust
// SVG with embedded fonts
let standalone_svg = SvgOptions::standalone();
let svg_content = svg_exporter.export_with_options(&output, standalone_svg)?;

// Optimized web SVG
let web_svg = SvgOptions::web();
let optimized = svg_exporter.export_with_options(&output, web_svg)?;

// CBDT-compatible SVG (embeds bitmaps as PNG)
let cbdt_svg = SvgOptions::cbdt_compatible();
let bitmap_svg = svg_exporter.export_with_options(&output, cbdt_svg)?;

// SVG with custom dimensions
let mut custom_svg = SvgOptions::web();
custom_svg.viewbox = Some(Rect::new(0.0, 0.0, 800.0, 600.0));
let sized_svg = svg_exporter.export_with_options(&output, custom_svg)?;

// SVG that gracefully handles bitmap fonts
let mut safe_svg = SvgOptions::web();
safe_svg.handle_bitmap_fonts = BitmapFontStrategy::Placeholder;
let robust_svg = svg_exporter.export_with_options(&output, safe_svg)?;
```

## PDF Export

PDF generates print-ready documents with proper typography and fonts.

### PDF Options

```rust
pub struct PdfOptions {
    pub page_size: PageSize,
    pub margins: Margins,
    pub embed_fonts: bool,
    pub compress: bool,
    pub version: PdfVersion,
    pub metadata: Option<DocumentMetadata>,
}

pub struct PageSize {
    pub width: f64,
    pub height: f64,
    pub units: Units,
}

pub struct Margins {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
    pub units: Units,
}
```

### PDF Usage

```rust
// Standard letter size
let letter_opts = PdfOptions {
    page_size: PageSize::letter(),
    margins: Margins::inches(0.5, 0.5, 0.5, 0.5),
    embed_fonts: true,
    compress: true,
    version: PdfVersion::V1_7,
    metadata: None,
};

let letter_pdf = pdf_exporter.export_with_options(&output, letter_opts)?;

// Custom page size
let custom_page = PdfOptions {
    page_size: PageSize::new(210.0, 297.0, Units::Millimeters), // A4
    margins: Margins::millimeters(10.0, 10.0, 10.0, 10.0),
    embed_fonts: true,
    compress: true,
    version: PdfVersion::V1_7,
    metadata: Some(DocumentMetadata {
        title: "Rendered Text".to_string(),
        author: "Typf".to_string(),
        subject: "Text Rendering".to_string(),
    }),
};
```

## PNM Export

PNM provides simple uncompressed raster formats for testing.

### PNM Types

```rust
pub enum PnmFormat {
    PBM,    // Portable bitmap (binary)
    PGM,    // Portable grayscale (8-bit)
    PPM,    // Portable pixmap (RGB)
    PAM,    // Portable arbitrary map (RGBA)
}
```

### PNM Usage

```rust
// Binary bitmap (1-bit)
let pbm_exporter = PnmExporter::new(PnmFormat::PBM);
let pbm_bytes = pbm_exporter.export(&output)?;

// Grayscale image
let pgm_exporter = PnmExporter::new(PnmFormat::PGM);
let pgm_bytes = pgm_exporter.export(&output)?;

// Color image
let ppm_exporter = PnmExporter::new(PnmFormat::PPM);
let ppm_bytes = ppm_exporter.export(&output)?;

// RGBA with transparency
let pam_exporter = PnmExporter::new(PnmFormat::PAM);
let pam_bytes = pam_exporter.export(&output)?;
```

## JSON Export

JSON exports structured data for debugging and analysis.

### JSON Content

```json
{
  "metadata": {
    "width": 800,
    "height": 600,
    "format": "rgba",
    "dpi": 72
  },
  "glyphs": [
    {
      "gid": 1,
      "unicode": "H",
      "x": 0,
      "y": 0,
      "width": 45,
      "height": 60,
      "advance": 48
    }
  ],
  "image": {
    "data": "base64-encoded-pixel-data",
    "stride": 3200
  }
}
```

### JSON Usage

```rust
// Export with all metadata
let json_exporter = JsonExporter::new();
let full_json = json_exporter.export(&output)?;

// Export minimal data
let minimal_json = json_exporter.export_minimal(&output)?;

// Export with custom formatting
let pretty_json = json_exporter.export_pretty(&output, 2)?;
```

## Format Selection

Choose the right format for your needs:

### Web Use
- **PNG**: For raster images with transparency
- **SVG**: For scalable icons and graphics
- **Size**: Prefer PNG for photos, SVG for text/shapes

### Print Production
- **PDF**: For final documents with proper fonts
- **SVG**: For vector graphics in design software
- **High-DPI PNG**: For raster images in layouts

### Development/Testing
- **JSON**: For debugging and analysis
- **PNM**: Simple format for unit tests
- **SVG**: Easy to inspect in browsers

### Data Processing
- **JSON**: Structured data for pipelines
- **PNG**: Compressed image data
- **Raw buffers**: For further processing

## Performance Comparison

| Format | Export Speed | File Size | Memory |
|--------|--------------|-----------|---------|
| PNG | 15ms | 45KB | 8MB |
| SVG | 5ms | 12KB | 2MB |
| PDF | 25ms | 18KB | 6MB |
| PNM | 2ms | 1.5MB | 8MB |
| JSON | 8ms | 89KB | 4MB |

## Advanced Configuration

### Custom Exporters

```rust
pub struct CustomExporter {
    format: ExportFormat,
    options: ExportOptions,
}

impl Exporter for CustomExporter {
    fn export(&self, output: &RenderOutput) -> Result<Vec<u8>> {
        match self.format {
            ExportFormat::Custom => self.custom_export(output),
            _ => fallback_exporter().export(output),
        }
    }
}
```

### Batch Export

```rust
// Export to multiple formats
let batch_exporter = BatchExporter::new();
batch_exporter.add_format(PngExporter::new());
batch_exporter.add_format(SvgExporter::new());
batch_exporter.add_format(JsonExporter::new());

let results = batch_exporter.export_all(&output)?;
// Returns HashMap<Format, Vec<u8>>
```

### Streaming Export

```rust
// Export large images without loading entirely in memory
let streaming_exporter = StreamingPngExporter::new(file_path);

streaming_exporter.begin_image(width, height)?;
for row in image_rows() {
    streaming_exporter.write_row(row)?;
}
streaming_exporter.finish()?;
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("Format not supported: {0}")]
    UnsupportedFormat(String),
    
    #[error("Encoding failed: {0}")]
    EncodingError(String),
    
    #[error("File write error: {0}")]
    FileError(std::io::Error),
    
    #[error("Memory allocation failed")]
    OutOfMemory,
    
    #[error("Invalid options: {0}")]
    InvalidOptions(String),
    
    #[error("CBDT font not compatible with SVG export: {message}")]
    CbdtNotSupported {
        message: String,
        suggestion: String,
    },
    
    #[error("Bitmap font glyph not found for SVG paths: glyph {glyph_id}")]
    BitmapGlyphNotFound {
        glyph_id: u32,
        font_name: String,
    },
}

// Enhanced error handling for font compatibility
fn safe_export_with_fallback(output: &RenderOutput, format: ExportFormat, font: &Font) -> Result<Vec<u8>> {
    match format {
        ExportFormat::SVG if is_cbdt_font(font) => {
            // Try PNG instead for CBDT fonts
            info!("CBDT font detected, falling back to PNG export");
            let png_exporter = PngExporter::new();
            png_exporter.export(output)
        }
        _ => {
            // Use requested format
            format.exporter().export(output)
        }
    }
}
```

## Testing Exports

```rust
#[test]
fn test_png_export_roundtrip() {
    let exporter = PngExporter::new();
    let original = create_test_output();
    
    let png_bytes = exporter.export(&original)?;
    let loaded = load_png_from_bytes(&png_bytes)?;
    
    assert_images_equal(&original, &loaded);
}

#[test]
fn test_svg_validity() {
    let exporter = SvgExporter::new();
    let output = create_test_output();
    
    let svg_content = exporter.export(&output)?;
    
    // Verify valid XML
    let doc = XmlDocument::parse(&svg_content).unwrap();
    assert_eq!(doc.root_tag(), "svg");
    
    // Verify paths present
    assert!(svg_content.contains("<path"));
}

#[test]
fn test_cbdt_svg_error_handling() {
    let exporter = SvgExporter::new();
    let cbdt_output = create_cbdt_test_output();
    let cbdt_font = load_cbdt_font();
    
    let result = exporter.export(&cbdt_output);
    assert!(result.is_err());
    
    match result.unwrap_err() {
        ExportError::CbdtNotSupported { message, suggestion } => {
            assert!(message.contains("CBDT"));
            assert!(suggestion.contains("PNG"));
        }
        _ => panic!("Expected CBDT-specific error"),
    }
}

#[test]
fn test_cbdt_embed_as_png() {
    let mut exporter = SvgExporter::new();
    let cbdt_output = create_cbdt_test_output();
    let cbdt_font = load_cbdt_font();
    
    // Configure to embed bitmaps as PNG
    let options = SvgOptions::cbdt_compatible();
    let svg_content = exporter.export_with_options(&cbdt_output, options)?;
    
    // Should contain base64 PNG data instead of paths
    assert!(svg_content.contains("data:image/png;base64,"));
    assert!(!svg_content.contains("<path")); // No vector paths for CBDT
}

#[test]
fn test_json_structure() {
    let exporter = JsonExporter::new();
    let output = create_test_output();
    
    let json = exporter.export(&output)?;
    let parsed: serde_json::Value = serde_json::from_str(&json)?;
    
    assert!(parsed["metadata"].is_object());
    assert!(parsed["glyphs"].is_array());
    assert!(parsed["image"].is_object());
}
```

---

Export formats let you deliver rendered text exactly where it's needed. Pick the right format for your use case and configure the options for optimal results.
