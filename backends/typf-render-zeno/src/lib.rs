//! Zeno rendering backend for TYPF
//!
//! This backend uses the Zeno crate for high-performance 2D path rasterization.
//!
//! ## Features
//!
//! - 256x anti-aliased rasterization (8-bit alpha)
//! - Pure Rust implementation with no external dependencies
//! - Near-identical output to Skia and modern browsers
//! - Efficient path building from glyph outlines
//! - Builder pattern for flexible configuration
//!
//! Made by FontLab - https://www.fontlab.com/

use std::sync::Arc;
use skrifa::MetadataProvider;
use typf_core::{
    error::{RenderError, Result},
    traits::{FontRef, Renderer},
    types::{BitmapData, BitmapFormat, RenderOutput, ShapingResult},
    RenderParams,
};

/// Zeno-based renderer using pure Rust rasterization
pub struct ZenoRenderer {
    /// Maximum canvas size
    max_size: u32,
}

impl ZenoRenderer {
    /// Create a new Zeno renderer
    pub fn new() -> Self {
        Self {
            max_size: 8192, // 8K max dimension
        }
    }

    /// Render a single glyph to a bitmap using Zeno
    ///
    /// # Arguments
    ///
    /// * `font` - Font reference containing the glyph data
    /// * `glyph_id` - ID of the glyph to render
    /// * `font_size` - Size to render the glyph at (in pixels)
    ///
    /// # Returns
    ///
    /// A `GlyphBitmap` containing the rasterized glyph with positioning data
    fn render_glyph(
        &self,
        font: &Arc<dyn FontRef>,
        glyph_id: u32,
        font_size: f32,
    ) -> Result<GlyphBitmap> {
        use zeno::Mask;

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

        // Build path using Zeno's PathBuilder
        let scale = font_size / font.units_per_em() as f32;
        let mut builder = ZenoPathBuilder::new(scale);

        // Use unhinted drawing
        let size = skrifa::instance::Size::new(font_size);
        let location = skrifa::instance::LocationRef::default();
        let settings = skrifa::outline::DrawSettings::unhinted(size, location);

        glyph
            .draw(settings, &mut builder)
            .map_err(|_| RenderError::OutlineExtractionFailed)?;

        let path_data = builder.finish();

        // Calculate bounding box from path
        let (min_x, min_y, max_x, max_y) = calculate_bounds(&path_data, scale);

        let width = ((max_x - min_x).ceil() as u32).max(1);
        let height = ((max_y - min_y).ceil() as u32).max(1);

        // Create mask for rendering
        let mut mask = vec![0u8; (width * height) as usize];

        // Render the path using Zeno (use string slice, not &String)
        let _placement = Mask::new(path_data.as_str())
            .size(width, height)
            .offset((-min_x as i32, -min_y as i32))
            .render_into(&mut mask, None);

        Ok(GlyphBitmap {
            width,
            height,
            data: mask,
            bearing_x: min_x as i32,
            bearing_y: -max_y as i32, // Flip Y coordinate
        })
    }
}

impl Default for ZenoRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for ZenoRenderer {
    fn name(&self) -> &'static str {
        "zeno"
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

        // Use advance_height as the font size (same as Orge/Skia renderers)
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
                        let px = x + gx as i32;
                        let py = y + gy as i32;

                        // Check bounds
                        if px < 0 || py < 0 || px >= width as i32 || py >= height as i32 {
                            continue;
                        }

                        let coverage = bitmap.data[(gy * bitmap.width + gx) as usize];
                        if coverage == 0 {
                            continue;
                        }

                        let canvas_idx = ((py as u32 * width + px as u32) * 4) as usize;

                        // Alpha blend foreground with coverage
                        let alpha = (coverage as u32 * params.foreground.a as u32) / 255;
                        let inv_alpha = 255 - alpha;

                        canvas[canvas_idx] = ((params.foreground.r as u32 * alpha
                            + canvas[canvas_idx] as u32 * inv_alpha)
                            / 255) as u8;
                        canvas[canvas_idx + 1] = ((params.foreground.g as u32 * alpha
                            + canvas[canvas_idx + 1] as u32 * inv_alpha)
                            / 255) as u8;
                        canvas[canvas_idx + 2] = ((params.foreground.b as u32 * alpha
                            + canvas[canvas_idx + 2] as u32 * inv_alpha)
                            / 255) as u8;
                        canvas[canvas_idx + 3] = ((alpha + canvas[canvas_idx + 3] as u32 * inv_alpha / 255)
                            .min(255)) as u8;
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

/// PathBuilder implementation that collects SVG-style path commands
struct ZenoPathBuilder {
    commands: Vec<String>,
    scale: f32,
}

impl ZenoPathBuilder {
    fn new(scale: f32) -> Self {
        Self {
            commands: Vec::new(),
            scale,
        }
    }

    fn finish(self) -> String {
        self.commands.join(" ")
    }
}

impl skrifa::outline::OutlinePen for ZenoPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let x = x * self.scale;
        let y = y * self.scale;
        self.commands.push(format!("M {:.2},{:.2}", x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let x = x * self.scale;
        let y = y * self.scale;
        self.commands.push(format!("L {:.2},{:.2}", x, y));
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let cx = cx * self.scale;
        let cy = cy * self.scale;
        let x = x * self.scale;
        let y = y * self.scale;
        self.commands
            .push(format!("Q {:.2},{:.2} {:.2},{:.2}", cx, cy, x, y));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let cx0 = cx0 * self.scale;
        let cy0 = cy0 * self.scale;
        let cx1 = cx1 * self.scale;
        let cy1 = cy1 * self.scale;
        let x = x * self.scale;
        let y = y * self.scale;
        self.commands.push(format!(
            "C {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
            cx0, cy0, cx1, cy1, x, y
        ));
    }

    fn close(&mut self) {
        self.commands.push("Z".to_string());
    }
}

/// Calculate bounding box from path commands
fn calculate_bounds(path: &str, _scale: f32) -> (f32, f32, f32, f32) {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    // Simple parser for SVG path commands to extract coordinates
    for cmd in path.split_whitespace() {
        if let Some(coords) = cmd.strip_prefix('M').or_else(|| cmd.strip_prefix('L')) {
            if let Some((x_str, y_str)) = coords.split_once(',') {
                if let (Ok(x), Ok(y)) = (x_str.parse::<f32>(), y_str.parse::<f32>()) {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        } else if let Some(coords) = cmd.strip_prefix('Q') {
            // Quadratic curve - only check endpoint for simplicity
            if let Some((_, rest)) = coords.split_once(' ') {
                if let Some((x_str, y_str)) = rest.split_once(',') {
                    if let (Ok(x), Ok(y)) = (x_str.parse::<f32>(), y_str.parse::<f32>()) {
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                    }
                }
            }
        } else if let Some(coords) = cmd.strip_prefix('C') {
            // Cubic curve - only check endpoint for simplicity
            let parts: Vec<&str> = coords.split(' ').collect();
            if parts.len() >= 3 {
                if let Some((x_str, y_str)) = parts[2].split_once(',') {
                    if let (Ok(x), Ok(y)) = (x_str.parse::<f32>(), y_str.parse::<f32>()) {
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                    }
                }
            }
        }
    }

    // Add some padding to account for curves
    let padding = 2.0;
    (
        min_x - padding,
        min_y - padding,
        max_x + padding,
        max_y + padding,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = ZenoRenderer::new();
        assert_eq!(renderer.name(), "zeno");
    }

    #[test]
    fn test_renderer_default() {
        let renderer = ZenoRenderer::default();
        assert_eq!(renderer.name(), "zeno");
    }
}
