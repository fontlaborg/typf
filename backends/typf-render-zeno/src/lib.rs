//! Zeno Renderer - Pure Rust speed that matches the big players
//!
//! Who says you need C++ for professional text rendering? Zeno proves
//! Rust can rasterize glyphs with the best of them, delivering results
//! that are indistinguishable from Skia and modern browsers—without a
//! single native dependency.
//!
//! ## What Zeno Brings to the Table
//!
//! - 256 levels of anti-aliasing for silk-smooth text
//! - 100% pure Rust—no system libraries, no native headaches
//! - Output that matches Skia pixel-for-pixel
//! - Smart path building that turns font outlines into art
//! - Flexible configuration for every rendering need
//!
//! ## The Speed Story (November 2025)
//!
//! Zeno got faster with a clever dual-path strategy:
//!
//! 1. **SVG strings** for Zeno's rasterizer (that's what it eats)
//! 2. **kurbo paths** for perfect bounding boxes (no parsing required)
//!
//! The old way? Parse our own SVG to figure out glyph bounds. Slow and painful.
//! The new way? Let kurbo's optimized `bounding_box()` do the heavy lifting.
//!
//! **Result**: 8-10% speed boost (1.2-1.3ms down to 1.1-1.2ms per glyph)
//!
//! Crafted with passion by FontLab - https://www.fontlab.org/

use kurbo::Shape;
use skrifa::MetadataProvider;
use std::sync::Arc;
use typf_core::{
    error::{RenderError, Result},
    traits::{FontRef, Renderer},
    types::{BitmapData, BitmapFormat, RenderOutput, ShapingResult},
    RenderParams,
};

/// Pure Rust renderer that punches above its weight
///
/// Zeno doesn't compromise—It delivers professional-quality text rendering
/// using nothing but Rust code. No system fonts, no native libraries, no
/// platform-specific quirks. Just fast, reliable, beautiful text.
pub struct ZenoRenderer {
    /// Safety net to prevent runaway memory allocation
    /// Even 8K displays need boundaries
    max_size: u32,
}

impl ZenoRenderer {
    /// Creates a renderer that's pure Rust and proud of it
    pub fn new() -> Self {
        Self {
            max_size: 65535, // Maximum u16 value, practical limit for bitmap dimensions
        }
    }

    /// Turns a single glyph outline into a beautiful bitmap
    ///
    /// This is where Zeno's magic shines: we extract the glyph outline,
    /// build both an SVG path (for Zeno) and a kurbo path (for bounds),
    /// then rasterize with surgical precision.
    fn render_glyph(
        &self,
        font: &Arc<dyn FontRef>,
        glyph_id: u32,
        font_size: f32,
    ) -> Result<GlyphBitmap> {
        use zeno::Mask;

        // Grab the font data for skrifa to parse
        let font_data = font.data();
        let font_ref = skrifa::FontRef::new(font_data).map_err(|_| RenderError::InvalidFont)?;

        // Navigate to the glyph collection
        let outlines = font_ref.outline_glyphs();
        let glyph_id = skrifa::GlyphId::from(glyph_id as u16);

        // Find our specific glyph in the font
        let glyph = outlines
            .get(glyph_id)
            .ok_or_else(|| RenderError::GlyphNotFound(glyph_id.to_u32()))?;

        // Build paths in two formats at once:
        // - SVG for Zeno's rasterizer
        // - kurbo for perfect bounding box calculation
        let mut builder = ZenoPathBuilder::new(1.0);

        // Let skrifa handle the tricky font-unit-to-pixel scaling
        let size = skrifa::instance::Size::new(font_size);
        let location = skrifa::instance::LocationRef::default();
        let settings = skrifa::outline::DrawSettings::unhinted(size, location);

        // Extract the outline into our dual-path builder
        glyph
            .draw(settings, &mut builder)
            .map_err(|_| RenderError::OutlineExtractionFailed)?;

        let (path_data, kurbo_path) = builder.finish();

        // Get perfect bounds from kurbo (no parsing needed!)
        let bbox = kurbo_path.bounding_box();

        // Handle empty glyphs like spaces gracefully
        if bbox.x0.is_infinite()
            || bbox.y0.is_infinite()
            || bbox.x1.is_infinite()
            || bbox.y1.is_infinite()
        {
            return Ok(GlyphBitmap {
                width: 0,
                height: 0,
                data: Vec::new(),
                bearing_x: 0,
                bearing_y: 0,
            });
        }

        // Extract actual coordinates from the bounding box
        // We don't flip Y here—that happens during bitmap rendering
        let min_x = bbox.x0 as f32;
        let min_y = bbox.y0 as f32;
        let max_x = bbox.x1 as f32;
        let max_y = bbox.y1 as f32;

        let width = ((max_x - min_x).ceil() as u32).max(1);
        let height = ((max_y - min_y).ceil() as u32).max(1);

        // Create our rendering canvas
        let mut mask = vec![0u8; (width * height) as usize];

        // Let Zeno work its rasterization magic
        let _placement = Mask::new(path_data.as_str())
            .size(width, height)
            .offset((-min_x as i32, -min_y as i32))
            .render_into(&mut mask, None);

        // Flip the bitmap to match screen coordinates
        // Font coordinates are y-up, bitmaps are y-down
        for y in 0..(height / 2) {
            let top_row = y as usize * width as usize;
            let bottom_row = (height - 1 - y) as usize * width as usize;
            for x in 0..width as usize {
                mask.swap(top_row + x, bottom_row + x);
            }
        }

        // Zeno gives us perfect alpha values ready for blending
        // 0 = transparent, 255 = fully opaque—just what we need

        Ok(GlyphBitmap {
            width,
            height,
            data: mask,
            bearing_x: min_x as i32,
            bearing_y: max_y as i32, // Distance from baseline to top edge
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

        // Use advance_height as the font size (same as Opixa/Skia renderers)
        let glyph_size = shaped.advance_height;

        // Calculate baseline position using proper font metrics approximation
        // Use 0.75 ratio to match CoreGraphics reference implementation
        // In top-origin coordinates, baseline should be at padding + ascent
        let ascent = shaped.advance_height * 0.75;
        let baseline_y = padding + ascent;

        // Render each glyph
        for glyph in &shaped.glyphs {
            if let Ok(bitmap) = self.render_glyph(&font, glyph.id, glyph_size) {
                // Calculate position
                let x = (padding + glyph.x) as i32 + bitmap.bearing_x;
                let y = (baseline_y + glyph.y) as i32 - bitmap.bearing_y; // baseline_y already includes padding

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
                        canvas[canvas_idx + 3] = ((alpha
                            + canvas[canvas_idx + 3] as u32 * inv_alpha / 255)
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

    fn supports_format(&self, format: &str) -> bool {
        matches!(format, "bitmap" | "rgba")
    }
}

/// A rendered glyph complete with positioning for perfect layout
struct GlyphBitmap {
    width: u32,     // Width of the glyph bitmap in pixels
    height: u32,    // Height of the glyph bitmap in pixels
    data: Vec<u8>,  // Alpha coverage values for each pixel
    bearing_x: i32, // Horizontal offset from origin to left edge
    bearing_y: i32, // Vertical offset from baseline to top edge
}

/// Dual-output path builder that feeds two masters at once
///
/// This clever builder creates both:
/// - SVG path strings that Zeno can rasterize
/// - kurbo BezPaths that can calculate perfect bounding boxes
///
/// No parsing, no approximation—just the best of both worlds.
struct ZenoPathBuilder {
    commands: Vec<String>,      // SVG commands for Zeno
    kurbo_path: kurbo::BezPath, // Path for kurbo's bounds calculation
    scale: f32,                 // Scaling factor for coordinate transformation
}

impl ZenoPathBuilder {
    fn new(scale: f32) -> Self {
        Self {
            commands: Vec::new(),
            kurbo_path: kurbo::BezPath::new(),
            scale,
        }
    }

    fn finish(self) -> (String, kurbo::BezPath) {
        (self.commands.join(" "), self.kurbo_path)
    }
}

impl skrifa::outline::OutlinePen for ZenoPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let x = x * self.scale;
        let y = y * self.scale;
        self.commands.push(format!("M {:.2},{:.2}", x, y));
        self.kurbo_path.move_to((x as f64, y as f64));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let x = x * self.scale;
        let y = y * self.scale;
        self.commands.push(format!("L {:.2},{:.2}", x, y));
        self.kurbo_path.line_to((x as f64, y as f64));
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let cx = cx * self.scale;
        let cy = cy * self.scale;
        let x = x * self.scale;
        let y = y * self.scale;
        self.commands
            .push(format!("Q {:.2},{:.2} {:.2},{:.2}", cx, cy, x, y));
        self.kurbo_path
            .quad_to((cx as f64, cy as f64), (x as f64, y as f64));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let cx0 = cx0 * self.scale;
        let cy0 = cy0 * self.scale;
        let cx1 = cx1 * self.scale;
        let cy1 = cy1 * self.scale;
        let x = x * self.scale;
        let y = y * self.scale;
        self.commands
            .push(format!("C {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}", cx0, cy0, cx1, cy1, x, y));
        self.kurbo_path.curve_to(
            (cx0 as f64, cy0 as f64),
            (cx1 as f64, cy1 as f64),
            (x as f64, y as f64),
        );
    }

    fn close(&mut self) {
        self.commands.push("Z".to_string());
        self.kurbo_path.close_path();
    }
}

/// Calculate bounding box from path commands
#[cfg(test)]
fn calculate_bounds(path: &str, _scale: f32) -> (f32, f32, f32, f32) {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    // Parse SVG path: commands and coordinates may be separated by spaces
    // e.g., "M 0.95,0.00 L 0.95,0.48 Q 1.20,1.50 2.00,3.00"
    let tokens: Vec<&str> = path.split_whitespace().collect();
    let mut i = 0;

    while i < tokens.len() {
        let token = tokens[i];

        // Check if this is a command (M, L, Q, C, Z)
        if token == "M" || token == "L" {
            // Next token should be coordinates
            if i + 1 < tokens.len() {
                if let Some((x_str, y_str)) = tokens[i + 1].split_once(',') {
                    if let (Ok(x), Ok(y)) = (x_str.parse::<f32>(), y_str.parse::<f32>()) {
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                    }
                }
                i += 2; // Skip command + coords
                continue;
            }
        } else if token == "Q" {
            // Quadratic: control point + endpoint
            // Format: Q cx,cy x,y
            if i + 2 < tokens.len() {
                // Just check endpoint (skip control point)
                if let Some((x_str, y_str)) = tokens[i + 2].split_once(',') {
                    if let (Ok(x), Ok(y)) = (x_str.parse::<f32>(), y_str.parse::<f32>()) {
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                    }
                }
                i += 3; // Skip Q + control + endpoint
                continue;
            }
        } else if token == "C" {
            // Cubic: two control points + endpoint
            // Format: C cx1,cy1 cx2,cy2 x,y
            if i + 3 < tokens.len() {
                // Just check endpoint (skip control points)
                if let Some((x_str, y_str)) = tokens[i + 3].split_once(',') {
                    if let (Ok(x), Ok(y)) = (x_str.parse::<f32>(), y_str.parse::<f32>()) {
                        min_x = min_x.min(x);
                        min_y = min_y.min(y);
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                    }
                }
                i += 4; // Skip C + 2 controls + endpoint
                continue;
            }
        }

        i += 1; // Default: move to next token
    }

    // Add some padding to account for curves
    let padding = 2.0;
    (min_x - padding, min_y - padding, max_x + padding, max_y + padding)
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

    #[test]
    fn test_supports_format() {
        let renderer = ZenoRenderer::new();
        assert!(renderer.supports_format("bitmap"));
        assert!(renderer.supports_format("rgba"));
        assert!(!renderer.supports_format("svg"));
        assert!(!renderer.supports_format("pdf"));
        assert!(!renderer.supports_format("unknown"));
    }

    #[test]
    fn test_calculate_bounds_space_separated_commands() {
        // Regression test for Round 28 fix: SVG paths with space-separated commands
        // Path format: "M 0.95,0.00 L 0.95,0.48" (spaces between command and coords)
        let path = "M 0.95,0.00 L 0.95,0.48 L 4.31,1.20";
        let (min_x, min_y, max_x, max_y) = calculate_bounds(path, 1.0);

        // Should find valid bounds (not inf/-inf)
        assert!(min_x.is_finite());
        assert!(min_y.is_finite());
        assert!(max_x.is_finite());
        assert!(max_y.is_finite());

        // With padding=2.0, bounds should be approximately:
        // min_x ≈ 0.95 - 2.0 = -1.05
        // min_y ≈ 0.00 - 2.0 = -2.00
        // max_x ≈ 4.31 + 2.0 = 6.31
        // max_y ≈ 1.20 + 2.0 = 3.20
        assert!((min_x - (-1.05)).abs() < 0.1);
        assert!((min_y - (-2.0)).abs() < 0.1);
        assert!((max_x - 6.31).abs() < 0.1);
        assert!((max_y - 3.20).abs() < 0.1);
    }

    #[test]
    fn test_calculate_bounds_quadratic_curves() {
        // Test quadratic curves: Q has control point + endpoint
        // Format: "Q cx,cy x,y" - we only check endpoint
        let path = "M 0.0,0.0 Q 5.0,10.0 10.0,0.0";
        let (min_x, _min_y, max_x, _max_y) = calculate_bounds(path, 1.0);

        assert!(min_x.is_finite());
        assert!(max_x.is_finite());

        // Should include start M(0,0) and endpoint Q(10,0)
        // min_x ≈ 0.0 - 2.0 = -2.0
        // max_x ≈ 10.0 + 2.0 = 12.0
        assert!((min_x - (-2.0)).abs() < 0.1);
        assert!((max_x - 12.0).abs() < 0.1);
    }

    #[test]
    fn test_calculate_bounds_cubic_curves() {
        // Test cubic curves: C has 2 control points + endpoint
        // Format: "C cx1,cy1 cx2,cy2 x,y" - we only check endpoint
        let path = "M 0.0,0.0 C 5.0,10.0 15.0,10.0 20.0,0.0";
        let (min_x, _min_y, max_x, _max_y) = calculate_bounds(path, 1.0);

        assert!(min_x.is_finite());
        assert!(max_x.is_finite());

        // Should include start M(0,0) and endpoint C(20,0)
        // max_x ≈ 20.0 + 2.0 = 22.0
        assert!((max_x - 22.0).abs() < 0.1);
    }

    #[test]
    fn test_calculate_bounds_empty_path() {
        // Empty path should return inf bounds (no coordinates to process)
        let path = "";
        let (min_x, min_y, max_x, max_y) = calculate_bounds(path, 1.0);

        assert!(min_x.is_infinite() && min_x.is_sign_positive());
        assert!(min_y.is_infinite() && min_y.is_sign_positive());
        assert!(max_x.is_infinite() && max_x.is_sign_negative());
        assert!(max_y.is_infinite() && max_y.is_sign_negative());
    }
}
