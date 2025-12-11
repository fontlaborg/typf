//! SVG export support for Typf
//!
//! This module provides SVG vector output from shaped text.
//!
//! ## Features
//!
//! - Direct glyph outline export to SVG paths
//! - Proper coordinate transformation and scaling
//! - Support for foreground colors and opacity
//! - Compact SVG output with optimized path commands
//! - Viewbox calculation for proper sizing
//! - Optional bitmap glyph embedding as base64 PNG (with `bitmap-embed` feature)
//!
//! ## Feature Flags
//!
//! - `bitmap-embed` - Enables embedding bitmap glyphs (CBDT/sbix) as base64 PNG `<image>` elements
//!
//! Community project by FontLab - https://www.fontlab.org/

#[cfg(feature = "bitmap-embed")]
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use skrifa::MetadataProvider;
use std::fmt::Write as FmtWrite;
use std::sync::Arc;
use typf_core::{
    error::{ExportError, Result},
    traits::FontRef,
    types::ShapingResult,
    Color,
};

/// SVG exporter for vector text output
#[derive(Debug)]
pub struct SvgExporter {
    /// SVG canvas padding
    padding: f32,
    /// Whether to embed bitmap glyphs as base64 PNG images
    #[cfg(feature = "bitmap-embed")]
    embed_bitmaps: bool,
}

impl SvgExporter {
    /// Create a new SVG exporter
    pub fn new() -> Self {
        Self {
            padding: 10.0,
            #[cfg(feature = "bitmap-embed")]
            embed_bitmaps: true,
        }
    }

    /// Set the padding around the SVG canvas
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Enable or disable bitmap glyph embedding (requires `bitmap-embed` feature)
    ///
    /// When enabled, bitmap-only glyphs (CBDT/sbix) are embedded as base64 PNG `<image>` elements.
    /// When disabled, bitmap glyphs are skipped (empty output).
    #[cfg(feature = "bitmap-embed")]
    pub fn with_bitmap_embedding(mut self, embed: bool) -> Self {
        self.embed_bitmaps = embed;
        self
    }

    /// Export shaped text to SVG
    ///
    /// # Arguments
    ///
    /// * `shaped` - Shaping result containing glyph positions
    /// * `font` - Font reference for glyph outline extraction
    /// * `foreground` - Text color
    ///
    /// # Returns
    ///
    /// Complete SVG document as a string
    pub fn export(
        &self,
        shaped: &ShapingResult,
        font: Arc<dyn FontRef>,
        foreground: Color,
    ) -> Result<String> {
        // Calculate viewBox dimensions
        let width = shaped.advance_width + self.padding * 2.0;
        let height = shaped.advance_height + self.padding * 2.0;

        let mut svg = String::new();

        // SVG header
        writeln!(&mut svg, r#"<?xml version="1.0" encoding="UTF-8"?>"#)
            .map_err(|e| ExportError::WriteFailed(e.to_string()))?;

        writeln!(
            &mut svg,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {:.2} {:.2}" width="{:.0}" height="{:.0}">"#,
            width, height, width, height
        )
        .map_err(|e| ExportError::WriteFailed(e.to_string()))?;

        // Background (optional - only if not fully transparent)
        // Skip for now to keep output minimal

        // Extract and render each glyph
        let baseline_y = height * 0.8; // Position baseline at 80% of height
        let scale = shaped.advance_height / font.units_per_em() as f32;

        for glyph in &shaped.glyphs {
            // Calculate glyph position
            let x = self.padding + glyph.x;
            let y = baseline_y + glyph.y;

            // Try to extract outline path
            let path_result = self.extract_glyph_path(&font, glyph.id, scale);

            match path_result {
                Ok(path) if !path.is_empty() => {
                    // Write path element with transform
                    writeln!(
                        &mut svg,
                        r#"  <path d="{}" fill="rgb({},{},{})" fill-opacity="{:.2}" transform="translate({:.2},{:.2})"/>"#,
                        path,
                        foreground.r,
                        foreground.g,
                        foreground.b,
                        foreground.a as f32 / 255.0,
                        x,
                        y
                    )
                    .map_err(|e| ExportError::WriteFailed(e.to_string()))?;
                },
                Ok(_) => {
                    // Empty path - try bitmap embedding if enabled
                    #[cfg(feature = "bitmap-embed")]
                    if self.embed_bitmaps {
                        if let Some(image_element) = self.try_embed_bitmap_glyph(
                            &font,
                            glyph.id,
                            shaped.advance_height,
                            x,
                            y,
                        )? {
                            writeln!(&mut svg, "{}", image_element)
                                .map_err(|e| ExportError::WriteFailed(e.to_string()))?;
                        }
                    }
                    // If bitmap embedding disabled or failed, skip the glyph
                },
                Err(_) => {
                    // Glyph extraction failed - try bitmap as fallback
                    #[cfg(feature = "bitmap-embed")]
                    if self.embed_bitmaps {
                        if let Some(image_element) = self.try_embed_bitmap_glyph(
                            &font,
                            glyph.id,
                            shaped.advance_height,
                            x,
                            y,
                        )? {
                            writeln!(&mut svg, "{}", image_element)
                                .map_err(|e| ExportError::WriteFailed(e.to_string()))?;
                            continue;
                        }
                    }
                    // No bitmap fallback available - skip silently
                },
            }
        }

        // SVG footer
        writeln!(&mut svg, "</svg>").map_err(|e| ExportError::WriteFailed(e.to_string()))?;

        Ok(svg)
    }

    /// Extract glyph outline as SVG path string
    ///
    /// Returns `Ok(empty string)` for bitmap-only glyphs that have no outline data.
    /// This allows graceful handling of CBDT/CBLC bitmap fonts.
    fn extract_glyph_path(
        &self,
        font: &Arc<dyn FontRef>,
        glyph_id: u32,
        scale: f32,
    ) -> Result<String> {
        let font_data = font.data();
        let font_ref = skrifa::FontRef::new(font_data)
            .map_err(|_| ExportError::EncodingFailed("Invalid font".to_string()))?;

        let outlines = font_ref.outline_glyphs();
        let glyph_id_obj = skrifa::GlyphId::new(glyph_id);

        // Try to get the outline - bitmap glyphs won't have one
        let glyph = match outlines.get(glyph_id_obj) {
            Some(g) => g,
            None => {
                // Check if this might be a bitmap-only glyph
                // If the font has bitmap strikes, this is likely a CBDT/sbix glyph
                let bitmap_strikes = font_ref.bitmap_strikes();
                let has_bitmap_data = !bitmap_strikes.is_empty();
                if has_bitmap_data {
                    // Bitmap glyph - return empty path (graceful skip)
                    // SVG paths can't represent bitmap data directly
                    return Ok(String::new());
                }
                // No bitmap data either - this glyph truly doesn't exist
                return Err(ExportError::EncodingFailed(format!(
                    "Glyph {} not found (no outline or bitmap data)",
                    glyph_id
                ))
                .into());
            },
        };

        // Extract at units_per_em size, then apply scale in the path builder
        // This avoids double-scaling issues
        let mut path_builder = SvgPathBuilder::new(scale);

        let size = skrifa::instance::Size::new(font.units_per_em() as f32);
        let location = skrifa::instance::LocationRef::default();
        let settings = skrifa::outline::DrawSettings::unhinted(size, location);

        glyph
            .draw(settings, &mut path_builder)
            .map_err(|_| ExportError::EncodingFailed("Outline extraction failed".to_string()))?;

        Ok(path_builder.finish())
    }

    /// Try to render a bitmap glyph and embed it as a base64 PNG image element
    ///
    /// Returns `Ok(Some(image_element))` if successful, `Ok(None)` if no bitmap available.
    #[cfg(feature = "bitmap-embed")]
    fn try_embed_bitmap_glyph(
        &self,
        font: &Arc<dyn FontRef>,
        glyph_id: u32,
        font_size: f32,
        x: f32,
        y: f32,
    ) -> Result<Option<String>> {
        use typf_render_color::bitmap::{has_bitmap_glyphs, render_bitmap_glyph};

        let font_data = font.data();

        // Check if font has bitmap glyphs
        if !has_bitmap_glyphs(font_data) {
            return Ok(None);
        }

        // Try to render the bitmap glyph
        let pixmap = match render_bitmap_glyph(font_data, glyph_id, font_size) {
            Ok(p) => p,
            Err(_) => return Ok(None), // No bitmap for this glyph
        };

        // Encode to PNG
        let png_data = pixmap
            .encode_png()
            .map_err(|e| ExportError::EncodingFailed(format!("PNG encoding failed: {}", e)))?;

        // Base64 encode
        let base64_data = BASE64_STANDARD.encode(&png_data);

        // Create SVG image element
        // Position: x is horizontal position, y needs adjustment for baseline
        // The image should be positioned so its bottom aligns with the baseline
        let img_width = pixmap.width() as f32;
        let img_height = pixmap.height() as f32;

        // Adjust y position: move up by image height to align bottom with baseline
        let img_y = y - img_height;

        let image_element = format!(
            r#"  <image x="{:.2}" y="{:.2}" width="{}" height="{}" href="data:image/png;base64,{}"/>"#,
            x, img_y, img_width, img_height, base64_data
        );

        Ok(Some(image_element))
    }
}

impl Default for SvgExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// SVG path builder implementing skrifa's OutlinePen
struct SvgPathBuilder {
    commands: String,
    scale: f32,
}

impl SvgPathBuilder {
    fn new(scale: f32) -> Self {
        Self {
            commands: String::new(),
            scale,
        }
    }

    fn finish(self) -> String {
        self.commands
    }
}

impl skrifa::outline::OutlinePen for SvgPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let x = x * self.scale;
        let y = -y * self.scale; // Flip Y for SVG coordinate system
        let _ = write!(&mut self.commands, "M{:.2},{:.2}", x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let x = x * self.scale;
        let y = -y * self.scale;
        let _ = write!(&mut self.commands, "L{:.2},{:.2}", x, y);
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let cx = cx * self.scale;
        let cy = -cy * self.scale;
        let x = x * self.scale;
        let y = -y * self.scale;
        let _ = write!(&mut self.commands, "Q{:.2},{:.2} {:.2},{:.2}", cx, cy, x, y);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let cx0 = cx0 * self.scale;
        let cy0 = -cy0 * self.scale;
        let cx1 = cx1 * self.scale;
        let cy1 = -cy1 * self.scale;
        let x = x * self.scale;
        let y = -y * self.scale;
        let _ = write!(
            &mut self.commands,
            "C{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
            cx0, cy0, cx1, cy1, x, y
        );
    }

    fn close(&mut self) {
        self.commands.push('Z');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exporter_creation() {
        let exporter = SvgExporter::new();
        assert_eq!(exporter.padding, 10.0);
    }

    #[test]
    fn test_exporter_with_padding() {
        let exporter = SvgExporter::new().with_padding(20.0);
        assert_eq!(exporter.padding, 20.0);
    }

    #[test]
    fn test_exporter_default() {
        let exporter = SvgExporter::default();
        assert_eq!(exporter.padding, 10.0);
    }
}
