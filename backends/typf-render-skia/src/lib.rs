//! Skia rendering backend for TYPF
//!
//! This backend uses tiny-skia for high-quality glyph rasterization with anti-aliasing.
//!
//! ## Features
//!
//! - Sub-pixel anti-aliasing via tiny-skia
//! - Vector path rendering with Bézier curves
//! - Winding fill rule for glyph outlines
//! - Grayscale alpha channel extraction
//!
//! Made by FontLab - https://www.fontlab.com/

use std::sync::Arc;
use kurbo::Shape;
use skrifa::MetadataProvider;
use typf_core::{
    error::{RenderError, Result},
    traits::{FontRef, Renderer},
    types::{BitmapData, BitmapFormat, RenderOutput, ShapingResult},
    RenderParams,
};

/// Skia-based renderer using tiny-skia
pub struct SkiaRenderer {
    /// Maximum canvas size
    max_size: u32,
}

impl SkiaRenderer {
    /// Create a new Skia renderer
    pub fn new() -> Self {
        Self { max_size: 8192 }
    }

    /// Render a single glyph to a bitmap
    fn render_glyph(
        &self,
        font: &Arc<dyn FontRef>,
        glyph_id: u32,
        font_size: f32,
    ) -> Result<GlyphBitmap> {
        use kurbo::{BezPath, PathEl};
        use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

        // Extract glyph outline using skrifa
        let font_data = font.data();
        let font_ref = skrifa::FontRef::new(font_data).map_err(|_| RenderError::InvalidFont)?;

        // Get outline glyphs collection
        let outlines = font_ref.outline_glyphs();
        let glyph_id = skrifa::GlyphId::from(glyph_id as u16);

        // Get the specific glyph
        let glyph = outlines
            .get(glyph_id)
            .ok_or_else(|| RenderError::GlyphNotFound(glyph_id.to_u32()))?;

        // Build path from glyph outline
        let mut path = BezPath::new();
        let scale = font_size / font.units_per_em() as f32;
        let mut pen = PathPen {
            path: &mut path,
            scale,
        };

        // Use unhinted drawing
        let size = skrifa::instance::Size::new(font_size);
        let location = skrifa::instance::LocationRef::default();
        let settings = skrifa::outline::DrawSettings::unhinted(size, location);

        glyph
            .draw(settings, &mut pen)
            .map_err(|_| RenderError::OutlineExtractionFailed)?;

        // Calculate bounding box
        let bbox = path.bounding_box();
        let width = (bbox.width().ceil() as u32).max(1);
        let height = (bbox.height().ceil() as u32).max(1);

        // Convert kurbo BezPath to tiny-skia Path
        let mut builder = PathBuilder::new();
        for element in path.elements() {
            match *element {
                PathEl::MoveTo(p) => builder.move_to(p.x as f32, p.y as f32),
                PathEl::LineTo(p) => builder.line_to(p.x as f32, p.y as f32),
                PathEl::QuadTo(ctrl, end) => {
                    builder.quad_to(ctrl.x as f32, ctrl.y as f32, end.x as f32, end.y as f32)
                }
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

        // Create pixmap
        let mut pixmap = Pixmap::new(width, height).ok_or(RenderError::PixmapCreationFailed)?;

        // Fill path with anti-aliasing
        let paint = Paint {
            anti_alias: true,
            ..Default::default()
        };

        pixmap.fill_path(
            &skia_path,
            &paint,
            FillRule::Winding,
            Transform::from_translate(-bbox.x0 as f32, -bbox.y0 as f32),
            None,
        );

        // Extract alpha channel (tiny-skia uses RGBA, we want grayscale alpha)
        let data = pixmap.data();
        let mut alpha = vec![0u8; (width * height) as usize];
        for i in 0..(width * height) as usize {
            alpha[i] = data[i * 4 + 3]; // Extract alpha channel
        }

        Ok(GlyphBitmap {
            width,
            height,
            data: alpha,
            bearing_x: bbox.x0 as i32,
            bearing_y: -bbox.y1 as i32,
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
        let min_height = if shaped.glyphs.is_empty() {
            16.0
        } else {
            shaped.advance_height
        };
        let height = (min_height + padding * 2.0).ceil() as u32;

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

        // Render each glyph
        for glyph in &shaped.glyphs {
            if let Ok(bitmap) = self.render_glyph(&font, glyph.id, glyph_size) {
                // Calculate position
                let x = (padding + glyph.x) as i32 + bitmap.bearing_x;
                let y = ((height as f32 * 0.8) + glyph.y) as i32 - bitmap.bearing_y;

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
                            let canvas_idx = ((canvas_y as u32 * width + canvas_x as u32) * 4) as usize;
                            let glyph_idx = (gy * bitmap.width + gx) as usize;
                            let alpha = bitmap.data[glyph_idx];

                            // Alpha blending (glyph alpha over background)
                            let fg = &params.foreground;
                            canvas[canvas_idx] = ((canvas[canvas_idx] as u16 * (255 - alpha) as u16
                                + fg.r as u16 * alpha as u16)
                                / 255) as u8;
                            canvas[canvas_idx + 1] = ((canvas[canvas_idx + 1] as u16
                                * (255 - alpha) as u16
                                + fg.g as u16 * alpha as u16)
                                / 255) as u8;
                            canvas[canvas_idx + 2] = ((canvas[canvas_idx + 2] as u16
                                * (255 - alpha) as u16
                                + fg.b as u16 * alpha as u16)
                                / 255) as u8;
                        }
                    }
                }
            }
        }

        Ok(RenderOutput::Bitmap(BitmapData {
            width,
            height,
            format: BitmapFormat::Rgba8,
            data: canvas,
        }))
    }
}

/// Glyph bitmap with positioning information
struct GlyphBitmap {
    width: u32,
    height: u32,
    data: Vec<u8>,
    bearing_x: i32,
    bearing_y: i32,
}

/// Pen for converting skrifa outline to kurbo path
struct PathPen<'a> {
    path: &'a mut kurbo::BezPath,
    scale: f32,
}

impl skrifa::outline::OutlinePen for PathPen<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path
            .move_to((x as f64 * self.scale as f64, y as f64 * self.scale as f64));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.path
            .line_to((x as f64 * self.scale as f64, y as f64 * self.scale as f64));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.path.quad_to(
            (cx0 as f64 * self.scale as f64, cy0 as f64 * self.scale as f64),
            (x as f64 * self.scale as f64, y as f64 * self.scale as f64),
        );
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.path.curve_to(
            (cx0 as f64 * self.scale as f64, cy0 as f64 * self.scale as f64),
            (cx1 as f64 * self.scale as f64, cy1 as f64 * self.scale as f64),
            (x as f64 * self.scale as f64, y as f64 * self.scale as f64),
        );
    }

    fn close(&mut self) {
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
}
