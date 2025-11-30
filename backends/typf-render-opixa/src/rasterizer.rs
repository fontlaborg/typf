//! Where fonts meet pixels: the final transformation
//!
//! Font files store mathematical curves, but screens need pixels. This module
//! orchestrates the delicate dance between skrifa's outline extraction and
//! our scan converter, turning vector curves into beautiful anti-aliased
//! bitmaps that humans can read.

use crate::fixed::F26Dot6;
use crate::grayscale::GrayscaleLevel;
use crate::scan_converter::ScanConverter;
use crate::{DropoutMode, FillRule};

use read_fonts::FontRef as ReadFontsRef;
use skrifa::instance::Size;
use skrifa::outline::DrawSettings;
use skrifa::{GlyphId as SkrifaGlyphId, MetadataProvider};

/// Your personal glyph artist: turning outlines into masterpieces
///
/// Every glyph starts as a mathematical blueprint in font files. This rasterizer
/// brings them to life through three careful steps:
/// - Extract perfect outlines from any font format
/// - Convert curves to crisp monochrome pixels
/// - Apply anti-aliasing magic for smooth edges
///
/// # Your First Rendering
/// ```ignore
/// let rasterizer = GlyphRasterizer::new(font_data, size)?;
/// let bitmap = rasterizer.render_glyph(glyph_id, fill_rule, dropout_mode)?;
/// // bitmap.data now contains pixels ready for your screen
/// ```
pub struct GlyphRasterizer<'a> {
    /// The font we're bringing to life
    font: ReadFontsRef<'a>,
    /// How big to make the glyphs (in pixels, not font units)
    size: f32,
    /// Our smoothing level: 1=crisp, 4=balanced, 8=perfect
    oversample: u8,
    /// Variable font coordinates for infinite font variation
    location: skrifa::instance::Location,
}

impl<'a> GlyphRasterizer<'a> {
    /// Ready your glyph artist
    ///
    /// Give us font data and your desired size, we'll prepare everything
    /// needed to transform those mathematical curves into beautiful pixels.
    ///
    /// # What You Need
    ///
    /// * `font_data` - Raw bytes from your TTF/OTF file
    /// * `size` - Target pixel size (12 for body text, 48+ for headlines)
    ///
    /// # What You Get
    ///
    /// A ready-to-use rasterizer or a helpful error message
    pub fn new(font_data: &'a [u8], size: f32) -> Result<Self, String> {
        let font =
            ReadFontsRef::new(font_data).map_err(|e| format!("Failed to parse font: {}", e))?;

        Ok(Self {
            font,
            size,
            oversample: 4, // 4x oversampling by default
            location: skrifa::instance::Location::default(),
        })
    }

    /// Shape your variable font: bend axes to your will
    ///
    /// Variable fonts contain infinite styles. This function lets you specify
    /// exactly which variation you want—weight, width, slant, or custom axes.
    /// All subsequent renderings will use this beautiful new shape.
    pub fn set_variations(&mut self, variations: &[(String, f32)]) -> Result<(), String> {
        if variations.is_empty() {
            self.location = skrifa::instance::Location::default();
            return Ok(());
        }

        // Use AxisCollection::location() which properly handles user-space
        // coordinates including axis variation remapping (avar table).
        // This is the recommended API instead of manually calling axis.normalize().
        let axes = self.font.axes();

        // Convert (String, f32) tuples to (&str, f32) for skrifa's location API
        let settings: Vec<(&str, f32)> = variations
            .iter()
            .map(|(tag, value)| (tag.as_str(), *value))
            .collect();

        self.location = axes.location(settings);
        Ok(())
    }

    /// Choose your smoothness: from razor-sharp to buttery-smooth
    ///
    /// Anti-aliasing is the art of compromise between speed and beauty.
    /// Higher oversampling creates smoother edges but demands more memory
    /// and processing time. Pick your sweet spot.
    pub fn with_oversample(mut self, oversample: u8) -> Self {
        self.oversample = oversample.max(1);
        self
    }

    /// The moment of truth: curves become pixels
    ///
    /// This is where the magic happens. We take everything you've configured—
    /// font, size, variations, smoothing—and transform a single glyph from
    /// mathematical curves into actual pixels you can display.
    ///
    /// # The Recipe
    ///
    /// * `glyph_id` - Which character to bring to life
    /// * `fill_rule` - How to decide what's inside vs outside
    /// * `dropout_mode` - How to handle tiny details at small sizes
    ///
    /// # Your Reward
    ///
    /// A complete bitmap with alpha values, ready for blending into any surface
    ///
    /// # Before You Call
    ///
    /// Set up variable variations with `set_variations()` if needed
    pub fn render_glyph(
        &self,
        glyph_id: u32,
        fill_rule: FillRule,
        dropout_mode: DropoutMode,
    ) -> Result<GlyphBitmap, String> {
        // Note: Scaling is handled by DrawSettings, which uses self.size directly

        // Get glyph outlines - use GlyphId::new for full u32 range (>65k glyph IDs)
        let skrifa_gid = SkrifaGlyphId::new(glyph_id);
        let outline_glyphs = self.font.outline_glyphs();

        let glyph = outline_glyphs
            .get(skrifa_gid)
            .ok_or_else(|| format!("Glyph {} not found", glyph_id))?;

        // Bounds detection: how much canvas do we really need?
        // We'll draw into a temporary pen to find the glyph's natural size
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

        // Variable font location comes from our stored coordinates
        let location_ref = self.location.coords();
        let draw_settings = DrawSettings::unhinted(size_setting, location_ref);

        let mut bounds_calc = BoundsCalculator::new();
        glyph
            .draw(draw_settings, &mut bounds_calc)
            .map_err(|e| format!("Failed to calculate bounds: {:?}", e))?;

        if !bounds_calc.has_points {
            // Empty glyph (spaces, tabs, etc.) - perfectly valid, just needs no canvas
            return Ok(GlyphBitmap {
                width: 0,
                height: 0,
                left: 0,
                top: 0,
                data: Vec::new(),
            });
        }

        // DrawSettings already scaled from font units to pixels
        // Now we convert to integer pixel coordinates
        let x_min = bounds_calc.x_min.floor() as i32;
        let y_min = bounds_calc.y_min.floor() as i32;
        let x_max = bounds_calc.x_max.ceil() as i32;
        let y_max = bounds_calc.y_max.ceil() as i32;

        let width = ((x_max - x_min) as u32 * self.oversample as u32).max(1);
        let height = ((y_max - y_min) as u32 * self.oversample as u32).max(1);

        // Guard against memory bombs (malicious fonts or giant sizes)
        if width > 4096 || height > 4096 {
            return Err(format!("Glyph bitmap too large: {}x{} (max 4096x4096)", width, height));
        }

        // Prepare our canvas, oversized for smooth anti-aliasing
        let mut scan_converter = ScanConverter::new(width as usize, height as usize);
        scan_converter.set_fill_rule(fill_rule);
        scan_converter.set_dropout_mode(dropout_mode);

        // Transform magic: where do pixels go in our oversized canvas?
        // skrifa handled font units → pixels, now we apply oversampling
        let oversample_scale = self.oversample as f32;
        let x_offset = -x_min as f32 * self.oversample as f32;
        // Y-flip: fonts go bottom-up, bitmaps go top-down
        let y_offset = y_max as f32 * self.oversample as f32;

        // Our coordinate transformer: shapes the canvas for the scan converter
        struct TransformPen<'p> {
            inner: &'p mut ScanConverter,
            scale: f32,
            x_offset: f32,
            y_offset: f32,
        }

        impl<'p> skrifa::outline::OutlinePen for TransformPen<'p> {
            fn move_to(&mut self, x: f32, y: f32) {
                let tx = x * self.scale + self.x_offset;
                let ty = -y * self.scale + self.y_offset; // Flip Y for bitmap coordinates
                self.inner
                    .move_to(F26Dot6::from_float(tx), F26Dot6::from_float(ty));
            }

            fn line_to(&mut self, x: f32, y: f32) {
                let tx = x * self.scale + self.x_offset;
                let ty = -y * self.scale + self.y_offset; // Flip Y
                self.inner
                    .line_to(F26Dot6::from_float(tx), F26Dot6::from_float(ty));
            }

            fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
                let tx1 = x1 * self.scale + self.x_offset;
                let ty1 = -y1 * self.scale + self.y_offset; // Flip Y
                let tx = x * self.scale + self.x_offset;
                let ty = -y * self.scale + self.y_offset; // Flip Y
                self.inner.quadratic_to(
                    F26Dot6::from_float(tx1),
                    F26Dot6::from_float(ty1),
                    F26Dot6::from_float(tx),
                    F26Dot6::from_float(ty),
                );
            }

            fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
                let tx1 = x1 * self.scale + self.x_offset;
                let ty1 = -y1 * self.scale + self.y_offset; // Flip Y
                let tx2 = x2 * self.scale + self.x_offset;
                let ty2 = -y2 * self.scale + self.y_offset; // Flip Y
                let tx = x * self.scale + self.x_offset;
                let ty = -y * self.scale + self.y_offset; // Flip Y
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
        let location_ref = self.location.coords(); // Use stored variations
        let draw_settings = DrawSettings::unhinted(size_setting, location_ref);

        glyph
            .draw(draw_settings, &mut transform_pen)
            .map_err(|e| format!("Failed to draw outline: {:?}", e))?;

        // The final touch: smooth those crisp pixels into beauty
        // This downsampling creates the anti-aliased effect readers love
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
            top: y_max, // TrueType origins are bottom-left, we prefer top-left
            data: gray_bitmap,
        })
    }
}

/// Your rendered glyph: pixels, position, and purpose
///
/// This isn't just a bitmap—it's a complete rendering package. We give you
/// the pixels plus exact positioning information so you can place each glyph
/// perfectly in your text layout.
#[derive(Debug, Clone)]
pub struct GlyphBitmap {
    /// How wide this glyph wants to be (in pixels)
    pub width: u32,
    /// How tall this glyph wants to be (in pixels)
    pub height: u32,
    /// How far from the origin to start drawing (left edge)
    pub left: i32,
    /// How far from the baseline to the top (for proper alignment)
    pub top: i32,
    /// The actual alpha values: 0=invisible air, 255=solid ink
    pub data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    // Real font testing happens in integration tests
    // Unit tests would need embedded font data, which we avoid
}
