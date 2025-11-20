# Chapter 17: Export Formats

## Overview

TYPF provides a comprehensive suite of export formats that transform rendered text into various output types suitable for different use cases. From raster images for web graphics to vector formats for print production, TYPF's export system ensures consistent quality and compatibility across platforms. Each format is optimized for its specific target while maintaining the high quality output expected from a professional text rendering engine.

## Architecture

### Export Pipeline

```rust
#[derive(Debug, Clone)]
pub struct ExportEngine {
    pub exporters: HashMap<ExportFormat, Box<dyn Exporter>>,
    pub config: ExportConfig,
    metadata: ExportMetadata,
}

pub trait Exporter: Send + Sync {
    fn export(&self, data: &ExportData, config: &ExportConfig) -> Result<ExportResult>;
    fn format(&self) -> ExportFormat;
    fn capabilities(&self) -> ExportCapabilities;
}

#[derive(Debug, Clone)]
pub struct ExportData {
    pub render_output: RenderOutput,
    pub font_info: FontInfo,
    pub shaping_result: Option<ShapingResult>,
    pub metadata: ExportMetadata,
}
```

### Export Flow

```
Rendered Output → Export Engine → Format-Specific Exporter → Final Output
                     ↗                    ↘                    ↘
               PNG/JPEG Exporter    SVG/PDF Exporter    JSON/XML Exporter
```

1. **Input**: Rendered data from any renderer
2. **Format Detection**: Choose appropriate exporter
3. **Processing**: Apply format-specific transformations
4. **Output**: Generate final file/data

## Raster Export Formats

### PNG (Portable Network Graphics)

```rust
#[derive(Debug, Clone)]
pub struct PngExporter {
    compression: PngCompression,
    color_type: PngColorType,
    filter: PngFilter,
}

impl Exporter for PngExporter {
    fn export(&self, data: &ExportData, config: &ExportConfig) -> Result<ExportResult> {
        match &data.render_output {
            RenderOutput::Bitmap { bitmap, .. } => {
                let png_data = self.encode_png(bitmap, &config.png)?;
                
                Ok(ExportResult {
                    format: ExportFormat::PNG,
                    data: png_data,
                    metadata: self.generate_metadata(data, config),
                })
            },
            RenderOutput::Vector { .. } => {
                let rasterized = self.rasterize_vector(data, config)?;
                let png_data = self.encode_png(&rasterized, &config.png)?;
                
                Ok(ExportResult {
                    format: ExportFormat::PNG,
                    data: png_data,
                    metadata: self.generate_metadata(data, config),
                })
            },
        }
    }
}

impl PngExporter {
    fn encode_png(&self, bitmap: &BitmapData, config: &PngConfig) -> Result<Vec<u8>> {
        let mut encoder = png::Encoder::new(
            Vec::new(),
            bitmap.width,
            bitmap.height,
        );
        
        encoder.set_color(match (bitmap.channels, bitmap.has_alpha) {
            (1, false) => png::ColorType::Grayscale,
            (1, true) => png::ColorType::GrayscaleAlpha,
            (3, false) => png::ColorType::RGB,
            (4, true) => png::ColorType::RGBA,
            _ => return Err(ExportError::UnsupportedPixelFormat),
        });
        
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(self.compression.get_png_type());
        encoder.set_filter(self.filter);
        
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&bitmap.data)?;
        
        Ok(writer.finish()?)
    }
    
    pub fn encode_with_transparency(
        &self,
        bitmap: &BitmapData,
        transparent_color: Color,
    ) -> Result<Vec<u8>> {
        let mut processed_data = bitmap.data.clone();
        
        // Apply transparency channel
        for chunk in processed_data.chunks_exact_mut(bitmap.channels) {
            if chunk.len() >= 3 {
                let r = chunk[0];
                let g = chunk[1];
                let b = chunk[2];
                
                if (r, g, b) == (transparent_color.r, transparent_color.g, transparent_color.b) {
                    if chunk.len() > 3 {
                        chunk[3] = 0; // Set alpha to 0
                    }
                }
            }
        }
        
        let transparent_bitmap = BitmapData {
            data: processed_data,
            width: bitmap.width,
            height: bitmap.height,
            channels: bitmap.channels + 1,
            has_alpha: true,
        };
        
        self.encode_png(&transparent_bitmap, &Default::default())
    }
}
```

### JPEG (Joint Photographic Experts Group)

```rust
#[derive(Debug, Clone)]
pub struct JpegExporter {
    quality: u8,           // 1-100 quality setting
    chroma_subsampling: ChromaSubsampling,
    color_space: JpegColorSpace,
}

impl Exporter for JpegExporter {
    fn export(&self, data: &ExportData, config: &ExportConfig) -> Result<ExportResult> {
        let bitmap = match &data.render_output {
            RenderOutput::Bitmap { bitmap, .. } => bitmap,
            RenderOutput::Vector { .. } => {
                &self.rasterize_vector(data, config)?
            },
        };
        
        // JPEG requires RGB conversion
        let rgb_bitmap = self.convert_to_rgb(bitmap)?;
        let jpeg_data = self.encode_jpeg(&rgb_bitmap)?;
        
        Ok(ExportResult {
            format: ExportFormat::JPEG,
            data: jpeg_data,
            metadata: self.generate_metadata(data, config),
        })
    }
}

impl JpegExporter {
    fn encode_jpeg(&self, bitmap: &BitmapData) -> Result<Vec<u8>> {
        let mut encoder = jpeg::Encoder::new(Vec::new(), self.quality);
        
        match self.color_space {
            JpegColorSpace::YCbCr => {
                encoder.encode(&bitmap.data, bitmap.width, bitmap.height)?;
            },
            JpegColorSpace::Grayscale => {
                let grayscale_data = self.convert_to_grayscale(&bitmap.data);
                encoder.encode(&grayscale_data, bitmap.width, bitmap.height)?;
            },
        }
        
        Ok(encoder.finish()?)
    }
    
    fn optimize_for_web(&self, bitmap: &BitmapData) -> Result<Vec<u8>> {
        let mut config = self.clone();
        
        // Web-optimized settings
        config.quality = (self.quality * 85 / 100).max(70); // Reduce quality slightly for web
        config.chroma_subsampling = ChromaSubsampling::Subsample420; // Better compression
        
        config.encode_jpeg(bitmap)
    }
}
```

## Vector Export Formats

### SVG (Scalable Vector Graphics)

```rust
#[derive(Debug, Clone)]
pub struct SvgExporter {
    pub pretty_print: bool,
    pub precision: u8,
    pub embed_fonts: bool,
    pub include_metadata: bool,
}

impl Exporter for SvgExporter {
    fn export(&self, data: &ExportData, config: &ExportConfig) -> Result<ExportResult> {
        let svg_content = match &data.render_output {
            RenderOutput::Vector { paths, .. } => {
                self.export_vector_to_svg(paths, data, config)?
            },
            RenderOutput::Bitmap { bitmap, .. } => {
                self.export_bitmap_to_svg(bitmap, data, config)?
            },
        };
        
        Ok(ExportResult {
            format: ExportFormat::SVG,
            data: svg_content.into_bytes(),
            metadata: self.generate_metadata(data, config),
        })
    }
}

impl SvgExporter {
    fn export_vector_to_svg(
        &self,
        paths: &[StyledVectorPath],
        data: &ExportData,
        config: &ExportConfig,
    ) -> Result<String> {
        let mut svg = String::new();
        
        // SVG header
        svg.push_str(&self.generate_svg_header(data, config));
        
        // Style definitions
        if self.embed_fonts {
            svg.push_str(&self.embed_font_styles(data)?);
        }
        
        // Export paths
        for (index, path) in paths.iter().enumerate() {
            let path_element = self.convert_path_to_svg(path, index)?;
            svg.push_str(&path_element);
        }
        
        // Metadata
        if self.include_metadata {
            svg.push_str(&self.generate_metadata_section(data));
        }
        
        svg.push_str("</svg>");
        
        Ok(if self.pretty_print {
            self.pretty_print_svg(&svg)
        } else {
            svg
        })
    }
    
    fn embed_font_styles(&self, data: &ExportData) -> Result<String> {
        let mut styles = String::new();
        styles.push_str(r#"<style type="text/css">"#);
        
        // Extract font information and generate CSS
        if let Some(font) = &data.font_info {
            styles.push_str(&format!(
                r#"
                @font-face {{
                    font-family: '{}';
                    src: url('data:font/woff2;base64,{}');
                }}
                "#,
                font.family,
                self.base64_encode_font(font)?
            ));
        }
        
        styles.push_str("</style>");
        
        Ok(styles)
    }
    
    fn generate_svg_header(&self, data: &ExportData, config: &ExportConfig) -> String {
        let bounds = data.render_output.get_bounds();
        
        format!(
            r#"<svg width="{}" height="{}" viewBox="{} {} {} {}" xmlns="http://www.w3.org/2000/svg""#,
            config.width.unwrap_or(bounds.width as u32),
            config.height.unwrap_or(bounds.height as u32),
            bounds.min_x,
            bounds.min_y,
            bounds.width,
            bounds.height,
        )
    }
}
```

### PDF (Portable Document Format)

```rust
#[derive(Debug, Clone)]
pub struct PdfExporter {
    pub create_outline: bool,
    pub embed_fonts: bool,
    pub compression: PdfCompression,
    pub metadata: PdfMetadata,
}

impl Exporter for PdfExporter {
    fn export(&self, data: &ExportData, config: &ExportConfig) -> Result<ExportResult> {
        let mut pdf = PdfDocument::new(&self.metadata.title);
        
        // Set up page
        let page = pdf.new_page(config.width.unwrap_or(612), config.height.unwrap_or(792));
        
        // Add content
        match &data.render_output {
            RenderOutput::Vector { paths, .. } => {
                self.add_vector_content_to_pdf(page, paths, data, config)?;
            },
            RenderOutput::Bitmap { bitmap, .. } => {
                self.add_bitmap_content_to_pdf(page, bitmap, data, config)?;
            },
        }
        
        // Embed fonts if needed
        if self.embed_fonts {
            self.embed_fonts_in_pdf(&mut pdf, data)?;
        }
        
        // Create outline if requested
        if self.create_outline {
            self.create_pdf_outline(&mut pdf, data)?;
        }
        
        let pdf_data = pdf.finish()?;
        
        Ok(ExportResult {
            format: ExportFormat::PDF,
            data: pdf_data,
            metadata: self.generate_metadata(data, config),
        })
    }
}
```

## Specialized Formats

### PNM (Portable Anymap)

```rust
#[derive(Debug, Clone)]
pub struct PnmExporter {
    pub format: PnmFormat,
    pub ascii: bool,
}

#[derive(Debug, Clone)]
pub enum PnmFormat {
    PBM,  // Portable BitMap (binary)
    PGM,  // Portable GrayMap (grayscale)
    PPM,  // Portable PixMap (RGB)
}

impl Exporter for PnmExporter {
    fn export(&self, data: &ExportData, _config: &ExportConfig) -> Result<ExportResult> {
        let bitmap = match &data.render_output {
            RenderOutput::Bitmap { bitmap, .. } => bitmap,
            _ => return Err(ExportError::UnsupportedConversion),
        };
        
        let pnm_data = match self.format {
            PnmFormat::PBM => self.export_pbm(bitmap)?,
            PnmFormat::PGM => self.export_pgm(bitmap)?,
            PnmFormat::PPM => self.export_ppm(bitmap)?,
        };
        
        Ok(ExportResult {
            format: ExportFormat::PNM,
            data: pnm_data,
            metadata: self.generate_metadata(data, &Default::default()),
        })
    }
}

impl PnmExporter {
    fn export_ppm(&self, bitmap: &BitmapData) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        
        // PPM header
        let format_type = if self.ascii { "P3" } else { "P6" };
        writeln!(output, "{}", format_type)?;
        writeln!(output, "{} {}", bitmap.width, bitmap.height)?;
        writeln!(output, "255")?;
        
        // Pixel data
        if self.ascii {
            // ASCII format
            for chunk in bitmap.data.chunks_exact(3) {
                write!(output, "{} {} {} ", chunk[0], chunk[1], chunk[2])?;
            }
        } else {
            // Binary format
            output.extend_from_slice(&bitmap.data);
        }
        
        Ok(output)
    }
}
```

### JSON Export

```rust
#[derive(Debug, Clone)]
pub struct JsonExporter {
    pub pretty_print: bool,
    pub include_metrics: bool,
    pub include_positions: bool,
}

impl Exporter for JsonExporter {
    fn export(&self, data: &ExportData, _config: &ExportConfig) -> Result<ExportResult> {
        let json_data = self.serialize_to_json(data)?;
        
        Ok(ExportResult {
            format: ExportFormat::JSON,
            data: json_data.into_bytes(),
            metadata: self.generate_metadata(data, &Default::default()),
        })
    }
}

impl JsonExporter {
    fn serialize_to_json(&self, data: &ExportData) -> Result<String> {
        let mut json = json::JsonValue::new_object();
        
        // Basic information
        json["format"] = json::JsonValue::String("TYPF Export".to_string());
        json["version"] = json::JsonValue::String(env!("CARGO_PKG_VERSION").to_string());
        json["timestamp"] = json::JsonValue::String(
            chrono::Utc::now().to_rfc3339()
        );
        
        // Font information
        if let Some(font) = &data.font_info {
            json["font"] = json::JsonValue::Object({
                let mut font_obj = json::JsonObject::new();
                font_obj.insert("family".to_string(), json::JsonValue::String(font.family.clone()));
                font_obj.insert("style".to_string(), json::JsonValue::String(font.style.clone()));
                font_obj.insert("weight".to_string(), json::JsonValue::Number(font.weight.into()));
                font_obj
            });
        }
        
        // Shaping results
        if let Some(shaping) = &data.shaping_result {
            json["shaping"] = self.serialize_shaping_result(shaping)?;
        }
        
        // Render output
        json["render"] = self.serialize_render_output(&data.render_output)?;
        
        Ok(if self.pretty_print {
            json::stringify_pretty(json, 4)
        } else {
            json::stringify(json)
        })
    }
    
    fn serialize_shaping_result(&self, shaping: &ShapingResult) -> Result<json::JsonValue> {
        let mut shaping_obj = json::JsonObject::new();
        
        shaping_obj.insert("glyph_count".to_string(), shaping.glyphs.len().into());
        
        if self.include_metrics {
            shaping_obj.insert("metrics".to_string(), {
                let mut metrics = json::JsonObject::new();
                metrics.insert("advance_width".to_string(), shaping.metrics.advance_width.into());
                metrics.insert("advance_height".to_string(), shaping.metrics.advance_height.into());
                metrics.insert("ascent".to_string(), shaping.metrics.ascent.into());
                metrics.insert("descent".to_string(), shaping.metrics.descent.into());
                json::JsonValue::Object(metrics)
            });
        }
        
        if self.include_positions {
            shaping_obj.insert("glyphs".to_string(), {
                let mut glyphs = json::Array::new();
                
                for (i, glyph_id) in shaping.glyphs.iter().enumerate() {
                    let mut glyph_obj = json::JsonObject::new();
                    glyph_obj.insert("id".to_string(), (*glyph_id).into());
                    
                    if let Some(pos) = shaping.positions.get(i) {
                        glyph_obj.insert("x".to_string(), pos.x_offset.into());
                        glyph_obj.insert("y".to_string(), pos.y_offset.into());
                        glyph_obj.insert("advance".to_string(), pos.x_advance.into());
                    }
                    
                    glyphs.push(json::JsonValue::Object(glyph_obj));
                }
                
                json::JsonValue::Array(glyphs)
            });
        }
        
        Ok(json::JsonValue::Object(shaping_obj))
    }
}
```

## Export Configuration

### Unified Configuration

```rust
#[derive(Debug, Clone)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub dpi: Option<f32>,
    pub quality: Option<u8>,
    pub compression: Option<CompressionLevel>,
    pub metadata: ExportMetadata,
    
    // Format-specific configs
    pub png: PngConfig,
    pub jpeg: JpegConfig,
    pub svg: SvgConfig,
    pub pdf: PdfConfig,
    pub json: JsonConfig,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    // Raster formats
    PNG,
    JPEG,
    WEBP,
    TIFF,
    BMP,
    
    // Vector formats
    SVG,
    PDF,
    EPS,
    
    // Specialized formats
    PNM,
    JSON,
    XML,
    
    // Raw formats
    RAW,
}

#[derive(Debug, Clone)]
pub struct ExportMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
    pub tags: Vec<String>,
    pub custom: HashMap<String, String>,
}
```

### Python Configuration

```python
import typf
from datetime import datetime

# PNG export configuration
png_config = typf.ExportConfig(
    format="png",
    width=800,
    height=600,
    dpi=300,
    png=typf.PngConfig(
        compression=typf.PngCompression.DEFLATE,
        color_type=typf.PngColorType.RGBA,
        filter=typf.PngFilter.ADAPTIVE,
    ),
    metadata=typf.ExportMetadata(
        title="TYPF Text Rendering",
        description="High-quality text rendering with TYPF",
        author="TYPF Team",
        created=datetime.utcnow(),
    )
)

# SVG export configuration
svg_config = typf.ExportConfig(
    format="svg",
    svg=typf.SvgConfig(
        pretty_print=True,
        precision=2,
        embed_fonts=False,
        include_metadata=True,
    ),
    metadata=typf.ExportMetadata(
        title="TYPF Vector Output",
        tags=["text", "rendering", "typf"],
    )
)

# JSON export configuration
json_config = typf.ExportConfig(
    format="json",
    json=typf.JsonConfig(
        pretty_print=True,
        include_metrics=True,
        include_positions=True,
    ),
)

# Export with different formats
renderer = typf.Typf(shaper="harfbuzz", renderer="skia")

# PNG export
png_result = renderer.export_text("Hello, World!", export_config=png_config)
with open("output.png", "wb") as f:
    f.write(png_result.data)

# SVG export
svg_result = renderer.export_text("Hello, World!", export_config=svg_config)
with open("output.svg", "w") as f:
    f.write(svg_result.data.decode())

# JSON export
json_result = renderer.export_text("Hello, World!", export_config=json_config)
with open("output.json", "w") as f:
    f.write(json_result.data.decode())
```

## Performance Optimization

### Streaming Exports

```rust
impl ExportEngine {
    pub fn export_streaming(
        &self,
        data: DataStream,
        format: ExportFormat,
        config: ExportConfig,
    ) -> Result<tokio::io::DuplexStream> {
        let (tx, rx) = tokio::io::duplex(64 * 1024);
        
        let exporter = self.get_exporter_for_format(format)?;
        
        tokio::spawn(async move {
            let mut buffer = Vec::new();
            
            while let Some(chunk) = data.next().await {
                match chunk {
                    Ok(data_chunk) => {
                        let export_chunk = exporter.export_chunk(&data_chunk, &config)?;
                        buffer.extend_from_slice(&export_chunk);
                    },
                    Err(e) => return Err(e),
                }
            }
            
            // Send final data
            tx.write_all(&buffer).await?;
            
            Ok::<(), ExportError>(())
        });
        
        Ok(rx)
    }
}
```

### Parallel Processing

```rust
impl ExportEngine {
    pub fn export_batch_parallel(
        &self,
        batch: Vec<ExportData>,
        config: ExportConfig,
    ) -> Result<Vec<ExportResult>> {
        use rayon::prelude::*;
        
        batch
            .into_par_iter()
            .map(|data| {
                let exporter = self.get_exporter_for_format(config.format.clone())?;
                exporter.export(&data, &config)
            })
            .collect()
    }
}
```

## Error Handling

### Export-Specific Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("Export format {format} not supported")]
    UnsupportedFormat { format: String },
    
    #[error("Encoding failed: {message}")]
    EncodingFailed { message: String },
    
    #[error("File size exceeds limit: {size} > {limit}")]
    FileSizeExceeded { size: u64, limit: u64 },
    
    #[error("Quality setting out of range: {quality}. Must be 1-100")]
    InvalidQuality { quality: u8 },
    
    #[error("Conversion from {from} to {to} not supported")]
    UnsupportedConversion { from: String, to: String },
    
    #[error("Export permissions denied: {message}")]
    PermissionDenied { message: String },
    
    #[error("Disk space insufficient. Required: {required}, Available: {available}")]
    InsufficientSpace { required: u64, available: u64 },
}
```

## Testing and Validation

### Format Compliance Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_png_compliance() {
        let exporter = PngExporter::default();
        let data = create_test_bitmap_data();
        let result = exporter.export(&data, &Default::default()).unwrap();
        
        // Verify PNG signature
        assert_eq!(&result.data[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
        
        // Verify CRC checksums
        let png_tester = png_Validator::new();
        assert!(png_tester.validate(&result.data).is_ok());
    }
    
    #[test]
    fn test_svg_structure() {
        let exporter = SvgExporter::default();
        let data = create_test_vector_data();
        let result = exporter.export(&data, &Default::default()).unwrap();
        
        let svg_content = String::from_utf8(result.data).unwrap();
        
        // Verify SVG structure
        assert!(svg_content.starts_with("<svg"));
        assert!(svg_content.ends_with("</svg>"));
        assert!(svg_content.contains("xmlns=\"http://www.w3.org/2000/svg\""));
        
        // Verify XML validity
        let xml_doc = xml::Document::parse(&svg_content).unwrap();
        assert_eq!(xml_doc.root_element().tag_name().name(), "svg");
    }
    
    #[test]
    fn test_json_serialization() {
        let exporter = JsonExporter::default();
        let data = create_test_export_data();
        let result = exporter.export(&data, &Default::default()).unwrap();
        
        let json_content = String::from_utf8(result.data).unwrap();
        
        // Verify JSON validity
        let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();
        
        assert!(parsed.get("format").is_some());
        assert!(parsed.get("version").is_some());
        assert!(parsed.get("timestamp").is_some());
    }
}
```

## Best Practices

### Format Selection Guidelines

1. **Web Graphics**: PNG for transparency, JPEG for photos, SVG for vectors
2. **Print Production**: PDF with embedded fonts, high DPI
3. **Embedded Systems**: PNM for simplicity, RAW for maximum compression
4. **Data Exchange**: JSON for programmatic use, XML for structured data
5. **Debugging**: JSON with full metrics, PPM for visual verification

### Quality Optimization

1. **Resolution**: Use appropriate DPI (72 web, 150 print, 300 high quality)
2. **Compression**: Balance file size vs. quality
3. **Color Spaces**: sRGB for web, CMYK for print considerations
4. **Metadata**: Include only necessary information
5. **Font Embedding**: Embed for PDF, omit for web SVGs

### Performance Considerations

1. **Batch Processing**: Use parallel exports for multiple files
2. **Memory Management**: Process large files in streams
3. **Caching**: Cache converted data for repeated exports
4. **Format Selection**: Choose simplest suitable format

TYPF's export system provides comprehensive format support while maintaining consistent quality and performance across all output types. The modular architecture allows for easy addition of new formats while ensuring backward compatibility with existing workflows.