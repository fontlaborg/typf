//! Glyph rasterizer - integrates scan converter with font outline extraction
//!
//! This module bridges skrifa's outline extraction with our scan converter
//! to produce anti-aliased glyph bitmaps.

use crate::fixed::F26Dot6;
use crate::grayscale::GrayscaleLevel;
use crate::scan_converter::ScanConverter;
use crate::{DropoutMode, FillRule};

use read_fonts::{FontRef as ReadFontsRef, TableProvider};
use skrifa::instance::Size;
use skrifa::outline::DrawSettings;
use skrifa::prelude::LocationRef;
use skrifa::{GlyphId as SkrifaGlyphId, MetadataProvider};

/// Glyph rasterizer combining scan conversion and anti-aliasing
///
/// This is the main entry point for rendering individual glyphs.
/// It handles:
/// - Outline extraction from skrifa fonts
/// - Scan conversion to monochrome bitmap
/// - Grayscale anti-aliasing via oversampling
///
/// # Example
/// ```ignore
/// let rasterizer = GlyphRasterizer::new(font_data, size)?;
/// let bitmap = rasterizer.render_glyph(glyph_id, fill_rule, dropout_mode)?;
/// ```
pub struct GlyphRasterizer<'a> {
    /// Font reference from read-fonts
    font: ReadFontsRef<'a>,
    /// Font size in pixels
    size: f32,
    /// Oversampling factor for anti-aliasing (typically 4 or 8)
    oversample: u8,
}

impl<'a> GlyphRasterizer<'a> {
    /// Create a new glyph rasterizer
    ///
    /// # Arguments
    ///
    /// * `font_data` - Raw font bytes (TTF/OTF)
    /// * `size` - Font size in pixels
    ///
    /// # Returns
    ///
    /// New `GlyphRasterizer` or error if font parsing fails
    pub fn new(font_data: &'a [u8], size: f32) -> Result<Self, String> {
        let font = ReadFontsRef::new(font_data).map_err(|e| format!("Failed to parse font: {}", e))?;

        Ok(Self {
            font,
            size,
            oversample: 4, // 4x oversampling by default
        })
    }

    /// Set the oversampling factor for anti-aliasing
    ///
    /// Higher values produce smoother edges but increase memory usage.
    /// Common values: 1 (no AA), 2, 4, 8
    pub fn with_oversample(mut self, oversample: u8) -> Self {
        self.oversample = oversample.max(1);
        self
    }

    /// Render a single glyph to a grayscale bitmap
    ///
    /// # Arguments
    ///
    /// * `glyph_id` - Glyph ID to render
    /// * `fill_rule` - Fill rule (NonZeroWinding or EvenOdd)
    /// * `dropout_mode` - Dropout control mode
    ///
    /// # Returns
    ///
    /// `GlyphBitmap` containing the rasterized glyph, or error
    pub fn render_glyph(
        &self,
        glyph_id: u32,
        fill_rule: FillRule,
        dropout_mode: DropoutMode,
    ) -> Result<GlyphBitmap, String> {
        // Get font metrics
        let upem = self.font.head().map_err(|e| format!("Failed to read head table: {}", e))?.units_per_em();

        // Calculate scaling factor from font units to pixels
        let scale = self.size / upem as f32;

        // Get glyph outlines
        let skrifa_gid = SkrifaGlyphId::from(glyph_id as u16);
        let outline_glyphs = self.font.outline_glyphs();

        let glyph = outline_glyphs
            .get(skrifa_gid)
            .ok_or_else(|| format!("Glyph {} not found", glyph_id))?;

        // Get glyph bounds by drawing it into a recording pen
        // This is simpler and more reliable than accessing glyf/loca directly
        struct BoundsCalculator {
            x_min: f32,
            y_min: f32,
            x_max: f32,
            y_max: f32,
            has_points: bool,
        }

        impl BoundsCalculator {
            fn new() -> Self {
                Self {
                    x_min: f32::MAX,
                    y_min: f32::MAX,
                    x_max: f32::MIN,
                    y_max: f32::MIN,
                    has_points: false,
                }
            }

            fn update(&mut self, x: f32, y: f32) {
                self.x_min = self.x_min.min(x);
                self.y_min = self.y_min.min(y);
                self.x_max = self.x_max.max(x);
                self.y_max = self.y_max.max(y);
                self.has_points = true;
            }
        }

        impl skrifa::outline::OutlinePen for BoundsCalculator {
            fn move_to(&mut self, x: f32, y: f32) {
                self.update(x, y);
            }

            fn line_to(&mut self, x: f32, y: f32) {
                self.update(x, y);
            }

            fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
                self.update(x1, y1);
                self.update(x, y);
            }

            fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
                self.update(x1, y1);
                self.update(x2, y2);
                self.update(x, y);
            }

            fn close(&mut self) {}
        }

        let size_setting = Size::new(self.size);
        let location_ref: LocationRef = Default::default();
        let draw_settings = DrawSettings::unhinted(size_setting, location_ref);

        let mut bounds_calc = BoundsCalculator::new();
        glyph
            .draw(draw_settings, &mut bounds_calc)
            .map_err(|e| format!("Failed to calculate bounds: {:?}", e))?;

        if !bounds_calc.has_points {
            // Empty glyph (e.g., space)
            return Ok(GlyphBitmap {
                width: 0,
                height: 0,
                left: 0,
                top: 0,
                data: Vec::new(),
            });
        }

        // bounds_calc already has scaled coordinates (from DrawSettings with size)
        // So we just convert to integers
        let x_min = bounds_calc.x_min.floor() as i32;
        let y_min = bounds_calc.y_min.floor() as i32;
        let x_max = bounds_calc.x_max.ceil() as i32;
        let y_max = bounds_calc.y_max.ceil() as i32;

        let width = ((x_max - x_min) as u32 * self.oversample as u32).max(1);
        let height = ((y_max - y_min) as u32 * self.oversample as u32).max(1);

        // Prevent excessive memory allocation
        if width > 4096 || height > 4096 {
            return Err(format!(
                "Glyph bitmap too large: {}x{} (max 4096x4096)",
                width, height
            ));
        }

        // Create scan converter with oversampled dimensions
        let mut scan_converter = ScanConverter::new(width as usize, height as usize);
        scan_converter.set_fill_rule(fill_rule);
        scan_converter.set_dropout_mode(dropout_mode);

        // Calculate transform: font units → oversampled pixel coordinates
        let oversample_scale = scale * self.oversample as f32;
        let x_offset = -x_min as f32 * self.oversample as f32;
        let y_offset = -y_min as f32 * self.oversample as f32;

        // Create a pen that transforms coordinates and feeds to ScanConverter
        struct TransformPen<'p> {
            inner: &'p mut ScanConverter,
            scale: f32,
            x_offset: f32,
            y_offset: f32,
        }

        impl<'p> skrifa::outline::OutlinePen for TransformPen<'p> {
            fn move_to(&mut self, x: f32, y: f32) {
                let tx = x * self.scale + self.x_offset;
                let ty = y * self.scale + self.y_offset;
                self.inner.move_to(F26Dot6::from_float(tx), F26Dot6::from_float(ty));
            }

            fn line_to(&mut self, x: f32, y: f32) {
                let tx = x * self.scale + self.x_offset;
                let ty = y * self.scale + self.y_offset;
                self.inner.line_to(F26Dot6::from_float(tx), F26Dot6::from_float(ty));
            }

            fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
                let tx1 = x1 * self.scale + self.x_offset;
                let ty1 = y1 * self.scale + self.y_offset;
                let tx = x * self.scale + self.x_offset;
                let ty = y * self.scale + self.y_offset;
                self.inner.quadratic_to(
                    F26Dot6::from_float(tx1),
                    F26Dot6::from_float(ty1),
                    F26Dot6::from_float(tx),
                    F26Dot6::from_float(ty),
                );
            }

            fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
                let tx1 = x1 * self.scale + self.x_offset;
                let ty1 = y1 * self.scale + self.y_offset;
                let tx2 = x2 * self.scale + self.x_offset;
                let ty2 = y2 * self.scale + self.y_offset;
                let tx = x * self.scale + self.x_offset;
                let ty = y * self.scale + self.y_offset;
                self.inner.cubic_to(
                    F26Dot6::from_float(tx1),
                    F26Dot6::from_float(ty1),
                    F26Dot6::from_float(tx2),
                    F26Dot6::from_float(ty2),
                    F26Dot6::from_float(tx),
                    F26Dot6::from_float(ty),
                );
            }

            fn close(&mut self) {
                self.inner.close();
            }
        }

        let mut transform_pen = TransformPen {
            inner: &mut scan_converter,
            scale: oversample_scale,
            x_offset,
            y_offset,
        };

        // Draw the glyph outline
        let size_setting = Size::new(self.size);
        let location_ref: LocationRef = Default::default(); // No variations for now
        let draw_settings = DrawSettings::unhinted(size_setting, location_ref);

        glyph
            .draw(draw_settings, &mut transform_pen)
            .map_err(|e| format!("Failed to draw outline: {:?}", e))?;

        // Apply grayscale anti-aliasing by downsampling
        // (render_grayscale will call scan_converter.rasterize() internally)
        let grayscale_level = match self.oversample {
            2 => GrayscaleLevel::Level2x2,
            4 => GrayscaleLevel::Level4x4,
            8 => GrayscaleLevel::Level8x8,
            _ => GrayscaleLevel::Level4x4, // Default to 4x4
        };

        let out_width = width as usize / self.oversample as usize;
        let out_height = height as usize / self.oversample as usize;

        // Use the grayscale module's downsample function
        // We need to import it properly
        let gray_bitmap = crate::grayscale::render_grayscale(
            &mut scan_converter,
            out_width,
            out_height,
            grayscale_level,
        );

        Ok(GlyphBitmap {
            width: out_width as u32,
            height: out_height as u32,
            left: x_min,
            top: y_max, // Note: TrueType uses bottom-left origin, we use top-left
            data: gray_bitmap,
        })
    }
}

/// A rasterized glyph bitmap
#[derive(Debug, Clone)]
pub struct GlyphBitmap {
    /// Bitmap width in pixels
    pub width: u32,
    /// Bitmap height in pixels
    pub height: u32,
    /// Left bearing (offset from origin to left edge)
    pub left: i32,
    /// Top bearing (offset from origin to top edge)
    pub top: i32,
    /// Grayscale bitmap data (0 = transparent, 255 = opaque)
    pub data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    // Tests require actual font data
    // Integration tests with real fonts are in the CLI tests
}
