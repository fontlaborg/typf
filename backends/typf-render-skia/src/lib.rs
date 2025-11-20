//! Skia Renderer - Professional-grade rasterization via tiny-skia
//!
//! When you need production-quality text rendering, Skia delivers.
//! This backend transforms font outlines into crisp anti-aliased bitmaps
//! using the same path rendering tech that powers Chrome and Android.
//!
//! ## What Makes Skia Special
//!
//! - Sub-pixel precision that makes text readable at any size
//! - True vector path rendering with proper Bézier curve handling
//! - Winding fill rules that match font designer expectations
//! - Clean alpha extraction for perfect compositing
//!
//! Crafted with care by FontLab - https://www.fontlab.org/

use kurbo::Shape;
use skrifa::MetadataProvider;
use std::sync::Arc;
use typf_core::{
    error::{RenderError, Result},
    traits::{FontRef, Renderer},
    types::{BitmapData, BitmapFormat, RenderOutput, ShapingResult},
    RenderParams,
};

/// tiny-skia powered renderer for pristine glyph output
///
/// This isn't just another bitmap renderer—it's a precision instrument
/// that extracts glyph outlines and renders them using industry-proven
/// algorithms. Perfect when quality matters more than raw speed.
pub struct SkiaRenderer {
    /// Maximum canvas dimension to prevent memory exhaustion
    /// Keeps even the most ambitious rendering jobs within bounds
    max_size: u32,
}

impl SkiaRenderer {
    /// Creates a renderer that treats every glyph with professional care
    pub fn new() -> Self {
        Self { max_size: 8192 }
    }

    /// Converts a single glyph from outline to bitmap with surgical precision
    ///
    /// This method extracts the glyph outline using skrifa, builds a path,
    /// and renders it with tiny-skia's advanced anti-aliasing. The result
    /// is a clean alpha bitmap ready for compositing.
    fn render_glyph(
        &self,
        font: &Arc<dyn FontRef>,
        glyph_id: u32,
        font_size: f32,
    ) -> Result<GlyphBitmap> {
        use kurbo::{BezPath, PathEl};
        use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

        // Pull raw font data for skrifa to parse
        let font_data = font.data();
        let font_ref = skrifa::FontRef::new(font_data).map_err(|_| RenderError::InvalidFont)?;

        // Navigate to the outline glyph collection
        let outlines = font_ref.outline_glyphs();
        let glyph_id = skrifa::GlyphId::from(glyph_id as u16);

        // Find the specific glyph we need to render
        let glyph = outlines
            .get(glyph_id)
            .ok_or_else(|| RenderError::GlyphNotFound(glyph_id.to_u32()))?;

        // Build a kurbo path from the glyph's outline data
        let mut path = BezPath::new();
        // skrifa's DrawSettings handles the tricky font-unit-to-pixel scaling
        // for us, so our PathPen can stay simple and focused
        let mut pen = PathPen {
            path: &mut path,
            scale: 1.0, // skrifa does the heavy lifting on scaling
        };

        // Request unhinted outlines at the exact size we need
        let size = skrifa::instance::Size::new(font_size);
        let location = skrifa::instance::LocationRef::default();
        let settings = skrifa::outline::DrawSettings::unhinted(size, location);

        // Trace the glyph outline into our kurbo path
        glyph
            .draw(settings, &mut pen)
            .map_err(|_| RenderError::OutlineExtractionFailed)?;

        // Figure out how much canvas space this glyph needs
        let bbox = path.bounding_box();

        // Guard against malformed glyphs that could crash the renderer
        if bbox.x0.is_infinite()
            || bbox.y0.is_infinite()
            || bbox.x1.is_infinite()
            || bbox.y1.is_infinite()
        {
            return Err(RenderError::PathBuildingFailed.into());
        }
        if bbox.width() == 0.0 || bbox.height() == 0.0 {
            return Err(RenderError::InvalidDimensions {
                width: bbox.width() as u32,
                height: bbox.height() as u32,
            }
            .into());
        }

        // Ensure we always have at least 1x1 pixels for rendering
        let width = (bbox.width().ceil() as u32).max(1);
        let height = (bbox.height().ceil() as u32).max(1);

        log::debug!(
            "Skia: glyph_id={}, bbox=({}, {}, {}, {}), size={}x{}",
            glyph_id,
            bbox.x0,
            bbox.y0,
            bbox.x1,
            bbox.y1,
            width,
            height
        );

        // Translate kurbo's path format into tiny-skia's native format
        let mut builder = PathBuilder::new();
        for element in path.elements() {
            match *element {
                PathEl::MoveTo(p) => builder.move_to(p.x as f32, p.y as f32),
                PathEl::LineTo(p) => builder.line_to(p.x as f32, p.y as f32),
                PathEl::QuadTo(ctrl, end) => {
                    builder.quad_to(ctrl.x as f32, ctrl.y as f32, end.x as f32, end.y as f32)
                },
                PathEl::CurveTo(c1, c2, end) => builder.cubic_to(
                    c1.x as f32,
                    c1.y as f32,
                    c2.x as f32,
                    c2.y as f32,
                    end.x as f32,
                    end.y as f32,
                ),
                PathEl::ClosePath => builder.close(),
            }
        }

        let skia_path = builder.finish().ok_or(RenderError::PathBuildingFailed)?;

        // Create our rendering surface
        let mut pixmap = Pixmap::new(width, height).ok_or(RenderError::PixmapCreationFailed)?;

        // Set up painter with anti-aliasing for smooth edges
        let paint = Paint {
            anti_alias: true,
            ..Default::default()
        };

        // Critical coordinate transform:
        // 1. Flip Y (fonts use y-up, bitmaps use y-down)
        // 2. Shift so bbox fits perfectly in our pixmap
        let transform =
            Transform::from_scale(1.0, -1.0).post_translate(-bbox.x0 as f32, bbox.y1 as f32);

        // Render the filled path to our pixmap
        pixmap.fill_path(&skia_path, &paint, FillRule::Winding, transform, None);

        // Extract just the alpha channel (tiny-skia gives us RGBA, we need grayscale)
        let data = pixmap.data();
        let mut alpha = vec![0u8; (width * height) as usize];
        for i in 0..(width * height) as usize {
            alpha[i] = data[i * 4 + 3]; // Alpha lives in channel 4
        }

        // Return positioning info so the glyph lands in the right place
        // bearing_x: how far from origin the leftmost pixel appears
        // bearing_y: how far above baseline the topmost pixel appears
        Ok(GlyphBitmap {
            width,
            height,
            data: alpha,
            bearing_x: bbox.x0.floor() as i32,
            bearing_y: bbox.y1.ceil() as i32,
        })
    }
}

impl Default for SkiaRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for SkiaRenderer {
    fn name(&self) -> &'static str {
        "skia"
    }

    fn render(
        &self,
        shaped: &ShapingResult,
        font: Arc<dyn FontRef>,
        params: &RenderParams,
    ) -> Result<RenderOutput> {
        // Calculate canvas dimensions
        let padding = params.padding as f32;
        let width = (shaped.advance_width + padding * 2.0).ceil() as u32;

        // Calculate height using font metrics approximation (matching CoreGraphics)
        // Ascent is approximately 80% of advance_height, descent is 20%
        let font_height = if shaped.glyphs.is_empty() {
            16.0 // Default minimum height for empty text
        } else {
            shaped.advance_height * 1.2 // Add extra space for descenders and diacritics
        };
        let height = (font_height + padding * 2.0).ceil() as u32;

        // Validate dimensions
        if width == 0 || height == 0 {
            return Err(RenderError::InvalidDimensions { width, height }.into());
        }

        if width > self.max_size || height > self.max_size {
            return Err(RenderError::InvalidDimensions { width, height }.into());
        }

        // Create RGBA canvas
        let mut canvas = vec![0u8; (width * height * 4) as usize];

        // Fill background if specified
        if let Some(bg) = params.background {
            for pixel in canvas.chunks_exact_mut(4) {
                pixel[0] = bg.r;
                pixel[1] = bg.g;
                pixel[2] = bg.b;
                pixel[3] = bg.a;
            }
        }

        // Use advance_height as the font size (same as Orge renderer)
        let glyph_size = shaped.advance_height;

        // Calculate baseline position using proper font metrics approximation
        // Use 0.75 ratio to match CoreGraphics reference implementation
        // In top-origin coordinates, baseline should be at padding + ascent
        let ascent = shaped.advance_height * 0.75;
        let baseline_y = padding + ascent;

        // Render each glyph
        for glyph in shaped.glyphs.iter() {
            match self.render_glyph(&font, glyph.id, glyph_size) {
                Ok(bitmap) => {
                    // Position glyph on canvas
                    // X: glyph.x + padding + bearing_x
                    // Y: baseline_y + glyph.y - bearing_y (baseline_y already includes padding)
                    //    (subtract bearing_y to position glyph correctly relative to baseline)
                    let x = (glyph.x + padding) as i32 + bitmap.bearing_x;
                    let y = (baseline_y + glyph.y) as i32 - bitmap.bearing_y;

                    // Composite glyph onto canvas
                    for gy in 0..bitmap.height {
                        for gx in 0..bitmap.width {
                            let canvas_x = x + gx as i32;
                            let canvas_y = y + gy as i32;

                            if canvas_x >= 0
                                && canvas_x < width as i32
                                && canvas_y >= 0
                                && canvas_y < height as i32
                            {
                                let canvas_idx =
                                    ((canvas_y as u32 * width + canvas_x as u32) * 4) as usize;
                                let glyph_idx = (gy * bitmap.width + gx) as usize;
                                let alpha = bitmap.data[glyph_idx];

                                // Alpha blending (glyph alpha over background)
                                let fg = &params.foreground;
                                canvas[canvas_idx] =
                                    ((canvas[canvas_idx] as u16 * (255 - alpha) as u16
                                        + fg.r as u16 * alpha as u16)
                                        / 255) as u8;
                                canvas[canvas_idx + 1] =
                                    ((canvas[canvas_idx + 1] as u16 * (255 - alpha) as u16
                                        + fg.g as u16 * alpha as u16)
                                        / 255) as u8;
                                canvas[canvas_idx + 2] =
                                    ((canvas[canvas_idx + 2] as u16 * (255 - alpha) as u16
                                        + fg.b as u16 * alpha as u16)
                                        / 255) as u8;
                            }
                        }
                    }
                },
                Err(e) => {
                    log::warn!("Skia: Failed to render glyph {}: {:?}", glyph.id, e);
                },
            }
        }

        Ok(RenderOutput::Bitmap(BitmapData {
            width,
            height,
            format: BitmapFormat::Rgba8,
            data: canvas,
        }))
    }

    fn supports_format(&self, format: &str) -> bool {
        matches!(format, "bitmap" | "rgba")
    }
}

/// A rendered glyph with everything needed for proper positioning
struct GlyphBitmap {
    width: u32,      // Pixel width of the glyph bitmap
    height: u32,     // Pixel height of the glyph bitmap
    data: Vec<u8>,   // Grayscale alpha values for each pixel
    bearing_x: i32,  // Horizontal offset from origin to left edge
    bearing_y: i32,  // Vertical offset from baseline to top edge
}

/// Bridge between skrifa's outline commands and kurbo's path format
///
/// This pen receives drawing commands from skrifa and translates them
/// into kurbo's path representation, handling scaling along the way.
struct PathPen<'a> {
    path: &'a mut kurbo::BezPath,
    scale: f32,
}

impl skrifa::outline::OutlinePen for PathPen<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        // Start a new subpath at this position
        self.path
            .move_to((x as f64 * self.scale as f64, y as f64 * self.scale as f64));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        // Draw a straight line to this point
        self.path
            .line_to((x as f64 * self.scale as f64, y as f64 * self.scale as f64));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        // Draw a quadratic Bézier curve with one control point
        self.path.quad_to(
            (cx0 as f64 * self.scale as f64, cy0 as f64 * self.scale as f64),
            (x as f64 * self.scale as f64, y as f64 * self.scale as f64),
        );
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        // Draw a cubic Bézier curve with two control points
        self.path.curve_to(
            (cx0 as f64 * self.scale as f64, cy0 as f64 * self.scale as f64),
            (cx1 as f64 * self.scale as f64, cy1 as f64 * self.scale as f64),
            (x as f64 * self.scale as f64, y as f64 * self.scale as f64),
        );
    }

    fn close(&mut self) {
        // Close the current subpath, connecting back to the start
        self.path.close_path();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = SkiaRenderer::new();
        assert_eq!(renderer.name(), "skia");
    }

    #[test]
    fn test_renderer_default() {
        let renderer = SkiaRenderer::default();
        assert_eq!(renderer.name(), "skia");
        assert_eq!(renderer.max_size, 8192);
    }

    #[test]
    fn test_supports_format() {
        let renderer = SkiaRenderer::new();
        assert!(renderer.supports_format("bitmap"));
        assert!(renderer.supports_format("rgba"));
        assert!(!renderer.supports_format("svg"));
        assert!(!renderer.supports_format("pdf"));
        assert!(!renderer.supports_format("unknown"));
    }
}
