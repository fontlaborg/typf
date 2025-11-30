//! SVG export format
//!
//! Exports rendered text to Scalable Vector Graphics format.

use crate::png::encode_bitmap_to_png;
use typf_core::{
    error::{ExportError, Result},
    traits::Exporter,
    types::{BitmapData, RenderOutput},
};

/// SVG exporter for rendering results
///
/// Converts bitmap rendering output to SVG format with embedded base64 image data.
///
/// # Examples
///
/// ```ignore
/// use typf_export::SvgExporter;
///
/// let exporter = SvgExporter::new();
/// let svg_data = exporter.export(&render_output)?;
/// std::fs::write("output.svg", svg_data)?;
/// ```
pub struct SvgExporter {
    /// Whether to embed the bitmap as base64 or use data URI
    embed_image: bool,
}

impl SvgExporter {
    /// Create a new SVG exporter
    pub fn new() -> Self {
        Self { embed_image: true }
    }

    /// Create SVG exporter that references external images
    pub fn with_external_images() -> Self {
        Self { embed_image: false }
    }

    /// Export bitmap data to SVG
    pub fn export_bitmap(&self, bitmap: &BitmapData) -> Result<Vec<u8>> {
        let mut svg = String::new();

        // SVG header
        svg.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg"
     xmlns:xlink="http://www.w3.org/1999/xlink"
     width="{}"
     height="{}"
     viewBox="0 0 {} {}">
"#,
            bitmap.width, bitmap.height, bitmap.width, bitmap.height
        ));

        if self.embed_image {
            // Convert bitmap to base64 PNG using proper encoding
            let png_data = encode_bitmap_to_png(bitmap)?;
            let base64_data = base64_encode(&png_data);

            svg.push_str(&format!(
                r#"  <image width="{}" height="{}" xlink:href="data:image/png;base64,{}" />
"#,
                bitmap.width, bitmap.height, base64_data
            ));
        } else {
            // Reference external image
            svg.push_str(&format!(
                r#"  <image width="{}" height="{}" xlink:href="output.png" />
"#,
                bitmap.width, bitmap.height
            ));
        }

        svg.push_str("</svg>\n");

        Ok(svg.into_bytes())
    }
}

impl Default for SvgExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Exporter for SvgExporter {
    fn name(&self) -> &'static str {
        "svg"
    }

    fn export(&self, output: &RenderOutput) -> Result<Vec<u8>> {
        match output {
            RenderOutput::Bitmap(bitmap) => self.export_bitmap(bitmap),
            _ => Err(ExportError::FormatNotSupported(
                "SVG exporter only supports bitmap output".into(),
            )
            .into()),
        }
    }

    fn extension(&self) -> &'static str {
        "svg"
    }

    fn mime_type(&self) -> &'static str {
        "image/svg+xml"
    }
}

/// Simple base64 encoding
fn base64_encode(data: &[u8]) -> String {
    use std::fmt::Write;

    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &byte) in chunk.iter().enumerate() {
            buf[i] = byte;
        }

        let b1 = (buf[0] >> 2) as usize;
        let b2 = (((buf[0] & 0x03) << 4) | (buf[1] >> 4)) as usize;
        let b3 = (((buf[1] & 0x0f) << 2) | (buf[2] >> 6)) as usize;
        let b4 = (buf[2] & 0x3f) as usize;

        write!(&mut result, "{}", TABLE[b1] as char).unwrap();
        write!(&mut result, "{}", TABLE[b2] as char).unwrap();

        if chunk.len() > 1 {
            write!(&mut result, "{}", TABLE[b3] as char).unwrap();
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            write!(&mut result, "{}", TABLE[b4] as char).unwrap();
        } else {
            result.push('=');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use typf_core::types::BitmapFormat;

    #[test]
    fn test_svg_exporter_creation() {
        let exporter = SvgExporter::new();
        assert!(exporter.embed_image);
    }

    #[test]
    fn test_svg_export_basic() {
        let bitmap = BitmapData {
            width: 10,
            height: 10,
            format: BitmapFormat::Rgba8,
            data: vec![255u8; 10 * 10 * 4],
        };

        let exporter = SvgExporter::new();
        let result = exporter.export_bitmap(&bitmap);
        assert!(result.is_ok());

        let svg = String::from_utf8(result.unwrap()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("width=\"10\""));
        assert!(svg.contains("height=\"10\""));
    }

    #[test]
    fn test_svg_embedded_png_is_valid() {
        // Create a small 2x2 RGBA bitmap
        let bitmap = BitmapData {
            width: 2,
            height: 2,
            format: BitmapFormat::Rgba8,
            data: vec![
                255, 0, 0, 255, // Red
                0, 255, 0, 255, // Green
                0, 0, 255, 255, // Blue
                255, 255, 255, 255, // White
            ],
        };

        let exporter = SvgExporter::new();
        let svg_bytes = exporter.export_bitmap(&bitmap).unwrap();
        let svg = String::from_utf8(svg_bytes).unwrap();

        // Extract base64 data from SVG
        let start = svg.find("base64,").unwrap() + 7;
        let end = svg[start..].find('"').unwrap() + start;
        let base64_data = &svg[start..end];

        // Decode base64 and verify PNG magic bytes
        use base64::{engine::general_purpose::STANDARD, Engine};
        let png_data = STANDARD.decode(base64_data).unwrap();

        // PNG signature: 137 80 78 71 13 10 26 10
        assert_eq!(
            &png_data[0..8],
            &[137, 80, 78, 71, 13, 10, 26, 10],
            "PNG should start with correct magic bytes"
        );

        // Verify IHDR chunk exists (first chunk after signature)
        assert_eq!(&png_data[12..16], b"IHDR", "PNG should have IHDR chunk");
    }

    #[test]
    fn test_svg_export_gray1_format() {
        // 8x8 bitmap = 8 bytes for Gray1 format
        let bitmap = BitmapData {
            width: 8,
            height: 8,
            format: BitmapFormat::Gray1,
            data: vec![0xAA; 8], // Alternating pattern
        };

        let exporter = SvgExporter::new();
        let result = exporter.export_bitmap(&bitmap);
        assert!(result.is_ok(), "Gray1 export should succeed");

        let svg = String::from_utf8(result.unwrap()).unwrap();
        assert!(svg.contains("data:image/png;base64,"));
    }

    #[test]
    fn test_svg_export_grayscale() {
        let bitmap = BitmapData {
            width: 4,
            height: 4,
            format: BitmapFormat::Gray8,
            data: vec![
                0, 64, 128, 192, 255, 200, 100, 50, 0, 64, 128, 192, 255, 200, 100, 50,
            ],
        };

        let exporter = SvgExporter::new();
        let result = exporter.export_bitmap(&bitmap);
        assert!(result.is_ok(), "Grayscale export should succeed");
    }

    #[test]
    fn test_svg_export_short_buffer_fails() {
        // Create a bitmap with insufficient data
        let bitmap = BitmapData {
            width: 10,
            height: 10,
            format: BitmapFormat::Rgba8,
            data: vec![255u8; 10], // Should be 10*10*4 = 400 bytes
        };

        let exporter = SvgExporter::new();
        let result = exporter.export_bitmap(&bitmap);
        assert!(result.is_err(), "Short buffer should fail");

        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("Buffer too small"),
            "Error should mention buffer size: {}",
            err_msg
        );
    }

    #[test]
    fn test_svg_external_image_mode() {
        let bitmap = BitmapData {
            width: 10,
            height: 10,
            format: BitmapFormat::Rgba8,
            data: vec![255u8; 10 * 10 * 4],
        };

        let exporter = SvgExporter::with_external_images();
        let result = exporter.export_bitmap(&bitmap).unwrap();
        let svg = String::from_utf8(result).unwrap();

        assert!(svg.contains("xlink:href=\"output.png\""));
        assert!(!svg.contains("base64"));
    }

    #[test]
    fn test_base64_encode() {
        let data = b"Hello, World!";
        let encoded = base64_encode(data);
        assert!(!encoded.is_empty());
        assert!(encoded
            .chars()
            .all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='));
    }
}
