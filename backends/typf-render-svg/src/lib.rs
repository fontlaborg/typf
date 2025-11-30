//! SVG Renderer: where glyphs become scalable vector paths
//!
//! Unlike raster renderers that produce pixels, the SVG renderer extracts
//! glyph outlines directly from the font and emits perfect vector paths.
//! The result scales infinitely without quality loss.
//!
//! ## How it works
//!
//! 1. Takes shaped glyph positions from any shaper
//! 2. Extracts outline curves from the font using skrifa
//! 3. Converts curves to SVG path commands
//! 4. Returns complete SVG document as RenderOutput::Vector

use skrifa::MetadataProvider;
use std::fmt::Write as FmtWrite;
use std::sync::Arc;
use typf_core::{
    error::{RenderError, Result},
    traits::{FontRef, Renderer},
    types::{RenderOutput, ShapingResult, VectorData, VectorFormat},
    RenderParams,
};

/// SVG vector renderer
///
/// Produces scalable vector graphics from shaped text by extracting
/// glyph outlines directly from the font file.
#[derive(Debug, Default)]
pub struct SvgRenderer {
    /// SVG canvas padding
    padding: f32,
}

impl SvgRenderer {
    /// Create a new SVG renderer with default padding
    pub fn new() -> Self {
        Self { padding: 10.0 }
    }

    /// Set the padding around the SVG canvas
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Extract glyph outline as SVG path string
    fn extract_glyph_path(
        &self,
        font: &Arc<dyn FontRef>,
        glyph_id: u32,
        scale: f32,
    ) -> Result<String> {
        let font_data = font.data();
        let font_ref = skrifa::FontRef::new(font_data).map_err(|_| RenderError::InvalidFont)?;

        let outlines = font_ref.outline_glyphs();
        let glyph_id = skrifa::GlyphId::from(glyph_id as u16);

        let glyph = match outlines.get(glyph_id) {
            Some(g) => g,
            None => return Ok(String::new()), // Missing glyph = empty path
        };

        let mut path_builder = SvgPathBuilder::new(scale);

        let size = skrifa::instance::Size::new(font.units_per_em() as f32);
        let location = skrifa::instance::LocationRef::default();
        let settings = skrifa::outline::DrawSettings::unhinted(size, location);

        glyph
            .draw(settings, &mut path_builder)
            .map_err(|_| RenderError::OutlineExtractionFailed)?;

        Ok(path_builder.finish())
    }
}

impl Renderer for SvgRenderer {
    fn name(&self) -> &'static str {
        "svg"
    }

    fn render(
        &self,
        shaped: &ShapingResult,
        font: Arc<dyn FontRef>,
        params: &RenderParams,
    ) -> Result<RenderOutput> {
        log::debug!("SvgRenderer: Rendering {} glyphs as vector paths", shaped.glyphs.len());

        let padding = params.padding as f32;
        let foreground = params.foreground;

        // Calculate viewBox dimensions
        let width = shaped.advance_width + padding * 2.0;
        let height = shaped.advance_height + padding * 2.0;

        let mut svg = String::new();

        // SVG header
        writeln!(&mut svg, r#"<?xml version="1.0" encoding="UTF-8"?>"#)
            .map_err(|_| RenderError::PathBuildingFailed)?;

        writeln!(
            &mut svg,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {:.2} {:.2}" width="{:.0}" height="{:.0}">"#,
            width, height, width, height
        )
        .map_err(|_| RenderError::PathBuildingFailed)?;

        // Baseline positioning (80% down from top)
        let baseline_y = height * 0.8;
        let scale = shaped.advance_height / font.units_per_em() as f32;

        // Render each glyph as an SVG path
        for glyph in &shaped.glyphs {
            let path = self.extract_glyph_path(&font, glyph.id, scale)?;

            if path.is_empty() {
                continue; // Skip glyphs with no outline (e.g., space)
            }

            let x = padding + glyph.x;
            let y = baseline_y + glyph.y;

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
            .map_err(|_| RenderError::PathBuildingFailed)?;
        }

        // SVG footer
        writeln!(&mut svg, "</svg>").map_err(|_| RenderError::PathBuildingFailed)?;

        Ok(RenderOutput::Vector(VectorData {
            format: VectorFormat::Svg,
            data: svg,
        }))
    }

    fn supports_format(&self, format: &str) -> bool {
        matches!(format.to_lowercase().as_str(), "svg" | "vector")
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
    fn test_renderer_creation() {
        let renderer = SvgRenderer::new();
        assert_eq!(renderer.name(), "svg");
    }

    #[test]
    fn test_renderer_with_padding() {
        let renderer = SvgRenderer::new().with_padding(20.0);
        assert_eq!(renderer.padding, 20.0);
    }

    #[test]
    fn test_supports_format() {
        let renderer = SvgRenderer::new();
        assert!(renderer.supports_format("svg"));
        assert!(renderer.supports_format("SVG"));
        assert!(renderer.supports_format("vector"));
        assert!(!renderer.supports_format("png"));
    }
}
