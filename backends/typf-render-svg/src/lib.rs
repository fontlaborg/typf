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
//!
//! ## Canvas Sizing
//!
//! Uses two-phase rendering to ensure proper viewBox dimensions:
//! - Phase 1: Extract all glyph paths, track actual bounds
//! - Phase 2: Generate SVG with accurate viewBox from bounds

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

    /// Extract glyph outline as SVG path string with bounds
    ///
    /// Returns (path_string, min_y, max_y) where min_y/max_y are in scaled
    /// SVG coordinates (y-flipped, relative to glyph origin).
    fn extract_glyph_path_with_bounds(
        &self,
        font: &Arc<dyn FontRef>,
        glyph_id: u32,
        scale: f32,
        location: &skrifa::instance::Location,
    ) -> Result<GlyphPath> {
        let font_data = font.data();
        let font_ref = skrifa::FontRef::new(font_data).map_err(|_| RenderError::InvalidFont)?;

        let outlines = font_ref.outline_glyphs();
        // Use GlyphId::new to support full u32 range (>65k glyph IDs)
        let glyph_id = skrifa::GlyphId::new(glyph_id);

        let glyph = match outlines.get(glyph_id) {
            Some(g) => g,
            None => {
                return Ok(GlyphPath {
                    path: String::new(),
                    min_y: 0.0,
                    max_y: 0.0,
                })
            },
        };

        let mut path_builder = SvgPathBuilder::new(scale);

        let size = skrifa::instance::Size::new(font.units_per_em() as f32);
        // Use provided location for variable font support
        let settings = skrifa::outline::DrawSettings::unhinted(size, location.coords());

        glyph
            .draw(settings, &mut path_builder)
            .map_err(|_| RenderError::OutlineExtractionFailed)?;

        let (path, min_y, max_y) = path_builder.finish_with_bounds();
        Ok(GlyphPath { path, min_y, max_y })
    }

    /// Build variation location from params
    fn build_location(
        font: &Arc<dyn FontRef>,
        variations: &[(String, f32)],
    ) -> skrifa::instance::Location {
        if variations.is_empty() {
            return skrifa::instance::Location::default();
        }

        let font_data = font.data();
        let font_ref = match skrifa::FontRef::new(font_data) {
            Ok(f) => f,
            Err(_) => return skrifa::instance::Location::default(),
        };

        let axes = font_ref.axes();
        let settings: Vec<(&str, f32)> = variations
            .iter()
            .map(|(tag, value)| (tag.as_str(), *value))
            .collect();

        axes.location(settings)
    }
}

/// Extracted glyph path with vertical bounds
struct GlyphPath {
    path: String,
    min_y: f32, // In SVG coords (y-flipped), relative to glyph origin
    max_y: f32,
}

/// A glyph ready for SVG output with its position
struct ExtractedGlyph {
    path: String,
    x: f32,
    y: f32,
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
        log::debug!(
            "SvgRenderer: Rendering {} glyphs as vector paths",
            shaped.glyphs.len()
        );

        let padding = params.padding as f32;
        let foreground = params.foreground;
        let scale = shaped.advance_height / font.units_per_em() as f32;

        // Build variable font location from params.variations
        let location = Self::build_location(&font, &params.variations);

        // Phase 1: Extract all glyph paths and compute actual bounds
        // min_y/max_y are in SVG coordinates relative to baseline (y=0)
        let mut extracted_glyphs: Vec<ExtractedGlyph> = Vec::new();
        let mut min_y: f32 = 0.0; // Below baseline (positive in SVG coords)
        let mut max_y: f32 = 0.0; // Above baseline (negative in SVG coords, but we track magnitude)

        for glyph in &shaped.glyphs {
            let glyph_path = self.extract_glyph_path_with_bounds(&font, glyph.id, scale, &location)?;

            if glyph_path.path.is_empty() {
                continue; // Skip glyphs with no outline (e.g., space)
            }

            // Glyph bounds relative to baseline at this position
            // glyph.y is the vertical offset from baseline (usually 0 for base glyphs)
            let glyph_min_y = glyph_path.min_y + glyph.y;
            let glyph_max_y = glyph_path.max_y + glyph.y;

            min_y = min_y.min(glyph_min_y);
            max_y = max_y.max(glyph_max_y);

            extracted_glyphs.push(ExtractedGlyph {
                path: glyph_path.path,
                x: glyph.x,
                y: glyph.y,
            });
        }

        // Phase 2: Calculate viewBox from actual content bounds
        let width = shaped.advance_width + padding * 2.0;

        // In SVG coords: min_y is topmost (most negative), max_y is bottommost (most positive)
        // Content height spans from min_y to max_y
        let content_height = if extracted_glyphs.is_empty() {
            shaped.advance_height // Fallback for empty text
        } else {
            max_y - min_y
        };
        let height = content_height + padding * 2.0;

        // Baseline position: distance from top of viewBox to baseline
        // min_y is the topmost point (most negative in SVG), so baseline is at:
        // padding + |min_y| = padding - min_y (since min_y is typically negative for ascenders)
        let baseline_y = padding - min_y;

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

        // Phase 3: Render each glyph with correct positioning
        for eg in &extracted_glyphs {
            let x = padding + eg.x;
            let y = baseline_y + eg.y;

            writeln!(
                &mut svg,
                r#"  <path d="{}" fill="rgb({},{},{})" fill-opacity="{:.2}" transform="translate({:.2},{:.2})"/>"#,
                eg.path,
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
///
/// Tracks vertical bounds while building the path for proper viewBox sizing.
struct SvgPathBuilder {
    commands: String,
    scale: f32,
    min_y: f32,
    max_y: f32,
    has_points: bool,
}

impl SvgPathBuilder {
    fn new(scale: f32) -> Self {
        Self {
            commands: String::new(),
            scale,
            min_y: 0.0,
            max_y: 0.0,
            has_points: false,
        }
    }

    /// Track a Y coordinate for bounds calculation
    fn track_y(&mut self, y: f32) {
        if !self.has_points {
            self.min_y = y;
            self.max_y = y;
            self.has_points = true;
        } else {
            self.min_y = self.min_y.min(y);
            self.max_y = self.max_y.max(y);
        }
    }

    fn finish_with_bounds(self) -> (String, f32, f32) {
        (self.commands, self.min_y, self.max_y)
    }
}

impl skrifa::outline::OutlinePen for SvgPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let x = x * self.scale;
        let y = -y * self.scale; // Flip Y for SVG coordinate system
        self.track_y(y);
        let _ = write!(&mut self.commands, "M{:.2},{:.2}", x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let x = x * self.scale;
        let y = -y * self.scale;
        self.track_y(y);
        let _ = write!(&mut self.commands, "L{:.2},{:.2}", x, y);
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let cx = cx * self.scale;
        let cy = -cy * self.scale;
        let x = x * self.scale;
        let y = -y * self.scale;
        // Track control point and endpoint
        self.track_y(cy);
        self.track_y(y);
        let _ = write!(&mut self.commands, "Q{:.2},{:.2} {:.2},{:.2}", cx, cy, x, y);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let cx0 = cx0 * self.scale;
        let cy0 = -cy0 * self.scale;
        let cx1 = cx1 * self.scale;
        let cy1 = -cy1 * self.scale;
        let x = x * self.scale;
        let y = -y * self.scale;
        // Track all control points and endpoint
        self.track_y(cy0);
        self.track_y(cy1);
        self.track_y(y);
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
