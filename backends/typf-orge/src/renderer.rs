//! High-level rendering API for the orge rasterizer.
//!
//! Provides a complete rendering pipeline from font outlines to grayscale bitmaps.

use crate::scan_converter::ScanConverter;
use crate::{DropoutMode, FillRule};
use read_fonts::TableProvider;
use skrifa::instance::Size;
use skrifa::outline::{DrawSettings, OutlinePen};
use thiserror::Error;

/// Error types for orge rendering
#[derive(Error, Debug)]
pub enum OrgeError {
    #[error("Invalid render parameters: {0}")]
    InvalidParams(String),

    #[error("Failed to rasterize glyph {glyph_id}: {reason}")]
    RasterizationFailed { glyph_id: u32, reason: String },

    #[error("Font error: {0}")]
    FontError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, OrgeError>;

/// Grayscale image output from orge rasterizer
#[derive(Clone, Debug)]
pub struct Image {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Image {
    /// Create a new image, validating dimensions and buffer size
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(OrgeError::InvalidParams(
                "Image dimensions must be non-zero".to_string(),
            ));
        }
        let expected = (width as usize) * (height as usize);
        if pixels.len() != expected {
            return Err(OrgeError::Internal(format!(
                "Pixel data size mismatch: expected {} bytes, got {}",
                expected,
                pixels.len()
            )));
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    /// Access raw grayscale pixels
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Consume the image and return the owned pixel buffer
    pub fn into_pixels(self) -> Vec<u8> {
        self.pixels
    }

    /// Width in pixels
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Return true when every pixel is zero (blank render)
    pub fn is_empty(&self) -> bool {
        self.pixels.iter().all(|&px| px == 0)
    }
}

/// Glyph rasterizer using orge scan conversion
#[derive(Clone, Debug)]
pub struct GlyphRasterizer {
    fill_rule: FillRule,
    dropout_mode: DropoutMode,
}

impl GlyphRasterizer {
    /// Create a new glyph rasterizer with default settings
    pub fn new() -> Self {
        Self {
            fill_rule: FillRule::NonZeroWinding,
            dropout_mode: DropoutMode::None,
        }
    }

    /// Set the fill rule (default: NonZeroWinding)
    pub fn with_fill_rule(mut self, rule: FillRule) -> Self {
        self.fill_rule = rule;
        self
    }

    /// Set dropout control mode (default: None)
    pub fn with_dropout_mode(mut self, mode: DropoutMode) -> Self {
        self.dropout_mode = mode;
        self
    }

    /// Render a single glyph from a font to a grayscale bitmap
    ///
    /// # Arguments
    ///
    /// * `font` - skrifa FontRef to render from
    /// * `glyph_id` - Glyph ID to render
    /// * `size` - Font size in pixels
    /// * `location` - Variable font coordinates (empty slice for non-variable fonts)
    /// * `width` - Output bitmap width
    /// * `height` - Output bitmap height
    ///
    /// # Returns
    ///
    /// Grayscale image with the rendered glyph
    pub fn render_glyph<'a>(
        &self,
        font: &(impl skrifa::MetadataProvider<'a> + TableProvider<'a>),
        glyph_id: u32,
        size: f32,
        location: &[(&str, f32)],
        width: u32,
        height: u32,
    ) -> Result<Image> {
        // Create scan converter
        let mut converter = ScanConverter::new(width as usize, height as usize);
        converter.set_fill_rule(self.fill_rule);
        converter.set_dropout_mode(self.dropout_mode);

        // Get font metrics
        let head = font
            .head()
            .map_err(|e| OrgeError::FontError(format!("Failed to read head table: {}", e)))?;
        let upem = head.units_per_em();
        let scale = size / upem as f32;

        // Set up drawing
        let axes = font.axes();
        let location_coords = location.to_vec();
        let loc = axes.location(location_coords.iter().copied());
        let location_ref = loc.coords();

        // Get outline
        let outline_glyphs = font.outline_glyphs();
        let glyph_outline =
            outline_glyphs
                .get(glyph_id.into())
                .ok_or_else(|| OrgeError::RasterizationFailed {
                    glyph_id,
                    reason: format!("Glyph {} not found in font", glyph_id),
                })?;

        // Draw outline to scan converter
        let baseline_y = height as f32 * 0.75;
        let mut pen = OrgePen::new(&mut converter, scale, 0.0, baseline_y);

        let draw_settings = DrawSettings::unhinted(Size::unscaled(), location_ref);
        glyph_outline.draw(draw_settings, &mut pen).map_err(|e| {
            OrgeError::RasterizationFailed {
                glyph_id,
                reason: format!("Failed to draw outline: {}", e),
            }
        })?;

        // Rasterize
        let bitmap = converter.rasterize();

        Image::new(width, height, bitmap)
    }
}

impl Default for GlyphRasterizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Outline pen that feeds skrifa outlines into orge scan converter
struct OrgePen<'a> {
    converter: &'a mut ScanConverter,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
}

impl<'a> OrgePen<'a> {
    fn new(converter: &'a mut ScanConverter, scale: f32, offset_x: f32, offset_y: f32) -> Self {
        Self {
            converter,
            scale,
            offset_x,
            offset_y,
        }
    }

    #[inline]
    fn transform_x(&self, x: f32) -> crate::fixed::F26Dot6 {
        crate::fixed::F26Dot6::from_float((x * self.scale) + self.offset_x)
    }

    #[inline]
    fn transform_y(&self, y: f32) -> crate::fixed::F26Dot6 {
        // Flip Y coordinate (skrifa uses positive-up, we use positive-down)
        crate::fixed::F26Dot6::from_float(self.offset_y - (y * self.scale))
    }
}

impl<'a> OutlinePen for OrgePen<'a> {
    fn move_to(&mut self, x: f32, y: f32) {
        let fx = self.transform_x(x);
        let fy = self.transform_y(y);
        self.converter.move_to(fx, fy);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let fx = self.transform_x(x);
        let fy = self.transform_y(y);
        self.converter.line_to(fx, fy);
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let fx0 = self.transform_x(cx0);
        let fy0 = self.transform_y(cy0);
        let fx = self.transform_x(x);
        let fy = self.transform_y(y);
        self.converter.quadratic_to(fx0, fy0, fx, fy);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let fx0 = self.transform_x(cx0);
        let fy0 = self.transform_y(cy0);
        let fx1 = self.transform_x(cx1);
        let fy1 = self.transform_y(cy1);
        let fx = self.transform_x(x);
        let fy = self.transform_y(y);
        self.converter.cubic_to(fx0, fy0, fx1, fy1, fx, fy);
    }

    fn close(&mut self) {
        self.converter.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_rejects_invalid_dimensions() {
        let result = Image::new(0, 10, vec![]);
        assert!(result.is_err());

        let result = Image::new(10, 10, vec![0u8; 5]);
        assert!(result.is_err());
    }

    #[test]
    fn image_is_empty_detects_blank_canvas() {
        let img = Image::new(4, 4, vec![0u8; 16]).unwrap();
        assert!(img.is_empty());

        let mut pixels = vec![0u8; 16];
        pixels[3] = 1;
        let img = Image::new(4, 4, pixels).unwrap();
        assert!(!img.is_empty());
    }

    #[test]
    fn rasterizer_has_sensible_defaults() {
        let rast = GlyphRasterizer::new();
        // Just verify it constructs without panic
        let _ = rast;
    }
}
