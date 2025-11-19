//! SVG export support for TYPF
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
//!
//! Made by FontLab - https://www.fontlab.com/

use std::fmt::Write as FmtWrite;
use std::sync::Arc;
use skrifa::MetadataProvider;
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
}

impl SvgExporter {
    /// Create a new SVG exporter
    pub fn new() -> Self {
        Self { padding: 10.0 }
    }

    /// Set the padding around the SVG canvas
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
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
        writeln!(
            &mut svg,
            r#"<?xml version="1.0" encoding="UTF-8"?>"#
        )
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
            let path = self.extract_glyph_path(&font, glyph.id, scale)?;

            if path.is_empty() {
                continue; // Skip glyphs with no outline (e.g., space)
            }

            // Calculate glyph position
            let x = self.padding + glyph.x;
            let y = baseline_y + glyph.y;

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
        }

        // SVG footer
        writeln!(&mut svg, "</svg>")
            .map_err(|e| ExportError::WriteFailed(e.to_string()))?;

        Ok(svg)
    }

    /// Extract glyph outline as SVG path string
    fn extract_glyph_path(&self, font: &Arc<dyn FontRef>, glyph_id: u32, scale: f32) -> Result<String> {
        let font_data = font.data();
        let font_ref = skrifa::FontRef::new(font_data)
            .map_err(|_| ExportError::EncodingFailed("Invalid font".to_string()))?;

        let outlines = font_ref.outline_glyphs();
        let glyph_id = skrifa::GlyphId::from(glyph_id as u16);

        let glyph = outlines
            .get(glyph_id)
            .ok_or_else(|| ExportError::EncodingFailed(format!("Glyph {} not found", glyph_id.to_u32())))?;

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
