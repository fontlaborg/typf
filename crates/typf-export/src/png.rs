//! PNG export format
//!
//! Exports rendered text to PNG format using the `image` crate.

use image::{ImageBuffer, ImageEncoder, RgbaImage};
use typf_core::{
    error::{ExportError, Result},
    traits::Exporter,
    types::{BitmapData, BitmapFormat, RenderOutput},
};

/// Encode bitmap data to PNG format.
///
/// This is the shared implementation used by both `PngExporter` and `SvgExporter`
/// (for embedded images). Handles all bitmap formats: RGBA8, RGB8, Gray8, Gray1.
///
/// Returns a valid PNG with proper IHDR, IDAT, and IEND chunks.
pub fn encode_bitmap_to_png(bitmap: &BitmapData) -> Result<Vec<u8>> {
    // Validate buffer size before processing
    let expected_size = match bitmap.format {
        BitmapFormat::Rgba8 => (bitmap.width * bitmap.height * 4) as usize,
        BitmapFormat::Rgb8 => (bitmap.width * bitmap.height * 3) as usize,
        BitmapFormat::Gray8 => (bitmap.width * bitmap.height) as usize,
        BitmapFormat::Gray1 => ((bitmap.width * bitmap.height + 7) / 8) as usize,
    };

    if bitmap.data.len() < expected_size {
        return Err(ExportError::EncodingFailed(format!(
            "Buffer too small: expected {} bytes for {}x{} {:?}, got {}",
            expected_size,
            bitmap.width,
            bitmap.height,
            bitmap.format,
            bitmap.data.len()
        ))
        .into());
    }

    // Create RGBA image buffer
    let img: RgbaImage = match bitmap.format {
        BitmapFormat::Rgba8 => {
            // Direct RGBA data
            ImageBuffer::from_raw(bitmap.width, bitmap.height, bitmap.data.clone()).ok_or_else(
                || {
                    ExportError::EncodingFailed(
                        "Failed to create image buffer from RGBA data".into(),
                    )
                },
            )?
        },
        BitmapFormat::Rgb8 => {
            // Convert RGB to RGBA
            let mut rgba_data = Vec::with_capacity((bitmap.width * bitmap.height * 4) as usize);
            for chunk in bitmap.data.chunks(3) {
                if chunk.len() < 3 {
                    break; // Guard against malformed data
                }
                rgba_data.push(chunk[0]); // R
                rgba_data.push(chunk[1]); // G
                rgba_data.push(chunk[2]); // B
                rgba_data.push(255); // A (fully opaque)
            }
            ImageBuffer::from_raw(bitmap.width, bitmap.height, rgba_data).ok_or_else(|| {
                ExportError::EncodingFailed("Failed to create image buffer from RGB data".into())
            })?
        },
        BitmapFormat::Gray8 => {
            // Convert grayscale to RGBA
            let mut rgba_data = Vec::with_capacity((bitmap.width * bitmap.height * 4) as usize);
            for &gray in &bitmap.data {
                rgba_data.push(gray); // R
                rgba_data.push(gray); // G
                rgba_data.push(gray); // B
                rgba_data.push(255); // A
            }
            ImageBuffer::from_raw(bitmap.width, bitmap.height, rgba_data).ok_or_else(|| {
                ExportError::EncodingFailed(
                    "Failed to create image buffer from grayscale data".into(),
                )
            })?
        },
        BitmapFormat::Gray1 => {
            // Convert 1-bit to RGBA
            let mut rgba_data = Vec::with_capacity((bitmap.width * bitmap.height * 4) as usize);
            for y in 0..bitmap.height {
                for x in 0..bitmap.width {
                    let byte_idx = ((y * bitmap.width + x) / 8) as usize;
                    let bit_idx = ((y * bitmap.width + x) % 8) as usize;
                    if byte_idx >= bitmap.data.len() {
                        // Guard against out-of-bounds access
                        rgba_data.extend_from_slice(&[0, 0, 0, 255]);
                        continue;
                    }
                    let bit = (bitmap.data[byte_idx] >> (7 - bit_idx)) & 1;
                    let value = if bit == 1 { 255 } else { 0 };
                    rgba_data.push(value); // R
                    rgba_data.push(value); // G
                    rgba_data.push(value); // B
                    rgba_data.push(255); // A
                }
            }
            ImageBuffer::from_raw(bitmap.width, bitmap.height, rgba_data).ok_or_else(|| {
                ExportError::EncodingFailed("Failed to create image buffer from 1-bit data".into())
            })?
        },
    };

    // Encode to PNG
    let mut png_data = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        &mut png_data,
        image::codecs::png::CompressionType::Default,
        image::codecs::png::FilterType::Sub,
    );

    encoder
        .write_image(img.as_raw(), bitmap.width, bitmap.height, image::ExtendedColorType::Rgba8)
        .map_err(|e| ExportError::EncodingFailed(format!("PNG encoding failed: {}", e)))?;

    Ok(png_data)
}

/// PNG exporter for rendering results
///
/// Converts bitmap rendering output to PNG format.
///
/// # Examples
///
/// ```
/// use typf_export::PngExporter;
/// let exporter = PngExporter::new();
/// ```
pub struct PngExporter;

impl PngExporter {
    /// Create a new PNG exporter
    pub fn new() -> Self {
        Self
    }

    /// Convert bitmap data to PNG format
    fn export_bitmap(&self, bitmap: &BitmapData) -> Result<Vec<u8>> {
        encode_bitmap_to_png(bitmap)
    }
}

impl Exporter for PngExporter {
    fn name(&self) -> &'static str {
        "png"
    }

    fn export(&self, output: &RenderOutput) -> Result<Vec<u8>> {
        match output {
            RenderOutput::Bitmap(bitmap) => self.export_bitmap(bitmap),
            _ => Err(ExportError::FormatNotSupported(
                "PNG exporter only supports bitmap output".into(),
            )
            .into()),
        }
    }

    fn extension(&self) -> &'static str {
        "png"
    }

    fn mime_type(&self) -> &'static str {
        "image/png"
    }
}

impl Default for PngExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_exporter_creation() {
        let exporter = PngExporter::new();
        assert_eq!(exporter.name(), "png");
        assert_eq!(exporter.extension(), "png");
        assert_eq!(exporter.mime_type(), "image/png");
    }

    #[test]
    fn test_png_export_rgba() {
        let exporter = PngExporter::new();

        // Create a small 2x2 RGBA test bitmap
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

        let output = RenderOutput::Bitmap(bitmap);
        let png_data = exporter.export(&output).unwrap();

        // PNG should start with PNG magic bytes
        assert_eq!(&png_data[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
        assert!(png_data.len() > 50); // Should have reasonable size for 2x2 image
    }

    #[test]
    fn test_png_export_grayscale() {
        let exporter = PngExporter::new();

        let bitmap = BitmapData {
            width: 2,
            height: 2,
            format: BitmapFormat::Gray8,
            data: vec![0, 128, 192, 255],
        };

        let output = RenderOutput::Bitmap(bitmap);
        let png_data = exporter.export(&output).unwrap();

        // Verify PNG magic bytes
        assert_eq!(&png_data[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn test_png_default() {
        let exporter = PngExporter::default();
        assert_eq!(exporter.name(), "png");
    }
}
