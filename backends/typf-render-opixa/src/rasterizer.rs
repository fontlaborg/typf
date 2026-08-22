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
    /// Outline hinting mode (see `set_hinting`)
    hinting: typf_core::HintingMode,
    /// Integer ppem to hint at (`None` = rounded size)
    ppem: Option<u32>,
    /// Lazily built typftth hinter for the current size/location
    #[cfg(feature = "tth")]
    hinter: Option<typftth::hinter::Hinter<'a>>,
    /// Raw font bytes, for the hinter
    #[cfg(feature = "tth")]
    font_data: &'a [u8],
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
            hinting: typf_core::HintingMode::None,
            ppem: None,
            #[cfg(feature = "tth")]
            hinter: None,
            #[cfg(feature = "tth")]
            font_data,
        })
    }

    /// Select outline hinting. With `HintingMode::TrueType` (feature `tth`)
    /// glyphs with TrueType instructions are hinted by `typftth` at `ppem`
    /// (default: the render size rounded) before scan conversion; fonts or
    /// glyphs without hints fall back to the unhinted path.
    pub fn set_hinting(&mut self, hinting: typf_core::HintingMode, ppem: Option<u32>) {
        self.hinting = hinting;
        self.ppem = ppem;
        #[cfg(feature = "tth")]
        {
            self.hinter = None;
        }
    }

    /// Integer ppem used for hinting.
    pub fn hint_ppem(&self) -> u32 {
        self.ppem.unwrap_or_else(|| self.size.round().max(1.0) as u32)
    }

    /// Lazily build the typftth hinter for the current size/location.
    #[cfg(feature = "tth")]
    fn hinter(&mut self) -> Option<&mut typftth::hinter::Hinter<'a>> {
        if self.hinting != typf_core::HintingMode::TrueType {
            return None;
        }
        if self.hinter.is_none() {
            let font = typftth::loader::HintFont::parse(self.font_data, 0).ok()?;
            if !font.is_hinted() {
                return None;
            }
            let coords: Vec<typftth::F2Dot14> =
                self.location.coords().iter().map(|c| typftth::F2Dot14(c.to_bits())).collect();
            let ppem = self.hint_ppem() as i32;
            // Behave like FreeType's v35 grayscale target: fonts that gate
            // hinting on the rasterizer version take their usual branches
            // and out-of-range CVT indices do not abort the glyph program.
            let variation = !font.axes.is_empty();
            let options = typftth::hinter::HinterOptions::freetype(typftth::GetInfoProfile::freetype_v35(true, variation));
            match typftth::hinter::Hinter::with_options(font, ppem, &coords, options, &mut typftth::NoTrace) {
                Ok(h) => self.hinter = Some(h),
                Err(e) => {
                    log::warn!("typftth setup failed, rendering unhinted: {e}");
                    return None;
                },
            }
        }
        self.hinter.as_mut()
    }

    /// Dropout mode actually used: the caller's choice, unless it is `None`
    /// and the font's `prep` enabled dropout control through SCANCTRL /
    /// SCANTYPE for this ppem (TrueType hinting active only).
    fn effective_dropout_mode(&mut self, requested: DropoutMode) -> DropoutMode {
        if requested != DropoutMode::None {
            return requested;
        }
        #[cfg(feature = "tth")]
        {
            let ppem = self.hint_ppem();
            if let Some(h) = self.hinter() {
                return dropout_from_scan_control(h.machine().params.scan_control, ppem);
            }
        }
        DropoutMode::None
    }

    /// Draw the glyph outline into `pen`, hinted when configured.
    /// Returns `Ok(true)` if the hinted path was used.
    fn draw_outline(
        &mut self,
        glyph_id: u32,
        pen: &mut impl skrifa::outline::OutlinePen,
    ) -> Result<bool, String> {
        #[cfg(feature = "tth")]
        if let Some(hinter) = self.hinter() {
            match hinter.hint_glyph(glyph_id, &mut typftth::NoTrace) {
                Ok(g) => {
                    emit_truetype_contours(&g.zone, pen);
                    return Ok(true);
                },
                Err(e) => log::debug!("typftth: glyph {glyph_id} unhinted ({e})"),
            }
        }
        let skrifa_gid = SkrifaGlyphId::new(glyph_id);
        let outline_glyphs = self.font.outline_glyphs();
        let glyph = outline_glyphs
            .get(skrifa_gid)
            .ok_or_else(|| format!("Glyph {} not found", glyph_id))?;
        let location = self.location.clone();
        let draw_settings = DrawSettings::unhinted(Size::new(self.size), location.coords());
        glyph
            .draw(draw_settings, pen)
            .map_err(|e| format!("Failed to draw outline: {:?}", e))?;
        Ok(false)
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
        &mut self,
        glyph_id: u32,
        fill_rule: FillRule,
        dropout_mode: DropoutMode,
    ) -> Result<GlyphBitmap, String> {
        // Note: Scaling is handled by DrawSettings, which uses self.size directly

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

        let mut bounds_calc = BoundsCalculator::new();
        self.draw_outline(glyph_id, &mut bounds_calc)?;

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

        // Calculate OUTPUT dimensions first (ensuring at least 1x1), then derive
        // oversampled dimensions. This ensures render_grayscale's mono_bitmap size
        // matches the scan_converter dimensions.
        let out_width = ((x_max - x_min) as usize).max(1);
        let out_height = ((y_max - y_min) as usize).max(1);
        let width = out_width * self.oversample as usize;
        let height = out_height * self.oversample as usize;

        // Guard against memory bombs (malicious fonts or giant sizes)
        if width > 4096 || height > 4096 {
            return Err(format!(
                "Glyph bitmap too large: {}x{} (max 4096x4096)",
                width, height
            ));
        }

        // Prepare our canvas, oversized for smooth anti-aliasing
        let mut scan_converter = ScanConverter::new(width, height);
        scan_converter.set_fill_rule(fill_rule);
        scan_converter.set_dropout_mode(self.effective_dropout_mode(dropout_mode));

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

        // Draw the glyph outline (hinted when configured)
        self.draw_outline(glyph_id, &mut transform_pen)?;

        // The final touch: smooth those crisp pixels into beauty
        // This downsampling creates the anti-aliased effect readers love
        let grayscale_level = match self.oversample {
            2 => GrayscaleLevel::Level2x2,
            4 => GrayscaleLevel::Level4x4,
            8 => GrayscaleLevel::Level8x8,
            _ => GrayscaleLevel::Level4x4, // Default to 4x4
        };

        // Use the grayscale module's downsample function
        // out_width and out_height were calculated at the top of this function
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

/// Emit a hinted TrueType zone as pen callbacks, in pixel units. Applies the
/// TrueType implied-on-curve rule between consecutive off-curve points.
#[cfg(feature = "tth")]
fn emit_truetype_contours(zone: &typftth::Zone, pen: &mut impl skrifa::outline::OutlinePen) {
    let px = |v: i32| v as f32 / 64.0;
    let n_outline = zone.outline_points;
    for c in 0..zone.contour_count {
        let start = zone.sp[c] as usize;
        let end = zone.ep[c] as usize;
        if start > end || end >= n_outline {
            continue;
        }
        let count = end - start + 1;
        let on = |i: usize| zone.on_curve[start + (i % count)] & typftth::zone::ONCURVE != 0;
        let pt = |i: usize| {
            let k = start + (i % count);
            (px(zone.x[k]), px(zone.y[k]))
        };
        // Find a starting on-curve point (or synthesize the midpoint of the first two off-curve points).
        let first_on = (0..count).find(|&i| on(i));
        let (sx, sy, start_idx) = match first_on {
            Some(i) => {
                let (x, y) = pt(i);
                (x, y, i)
            },
            None => {
                let (x0, y0) = pt(0);
                let (x1, y1) = pt(1);
                ((x0 + x1) / 2.0, (y0 + y1) / 2.0, 0)
            },
        };
        pen.move_to(sx, sy);
        let mut prev_off: Option<(f32, f32)> = None;
        let steps = if first_on.is_some() { count } else { count + 1 };
        for i in (start_idx + 1..).take(steps) {
            let (x, y) = pt(i);
            if on(i) {
                match prev_off.take() {
                    Some((cx, cy)) => pen.quad_to(cx, cy, x, y),
                    None => pen.line_to(x, y),
                }
            } else if let Some((cx, cy)) = prev_off {
                let mx = (cx + x) / 2.0;
                let my = (cy + y) / 2.0;
                pen.quad_to(cx, cy, mx, my);
                prev_off = Some((x, y));
            } else {
                prev_off = Some((x, y));
            }
        }
        if let Some((cx, cy)) = prev_off {
            pen.quad_to(cx, cy, sx, sy);
        }
        pen.close();
    }
}

/// Map TrueType SCANCTRL (low 16 bits) + SCANTYPE (high 16 bits) to a
/// [`DropoutMode`] the way FreeType's `tt_loader` does: dropout is enabled when
/// the threshold is 0xFF, or bit 8 is set and `ppem <= threshold`; bit 11
/// disables it above the threshold. SCANTYPE 0–3 = simple, 4–5 = smart,
/// 6–7 = none.
#[cfg(feature = "tth")]
fn dropout_from_scan_control(sc: typftth::gs::ScanControl, ppem: u32) -> DropoutMode {
    let ctrl = u32::from(sc.low());
    let threshold = ctrl & 0xFF;
    let enabled = threshold == 0xFF
        || (ctrl & 0x100 != 0 && ppem <= threshold)
        || (ctrl & 0x800 != 0 && ppem <= threshold);
    let disabled = ctrl & 0x800 != 0 && ppem > threshold;
    if !enabled || disabled {
        return DropoutMode::None;
    }
    match sc.kind() {
        0..=3 => DropoutMode::Simple,
        4 | 5 => DropoutMode::Smart,
        _ => DropoutMode::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real font testing happens in integration tests
    // Unit tests would need embedded font data, which we avoid

    #[cfg(feature = "tth")]
    #[test]
    fn test_dropout_from_scan_control() {
        use typftth::gs::ScanControl;
        let sc = |ctrl: u32, kind: u16| ScanControl(ctrl | (u32::from(kind) << 16));
        assert_eq!(dropout_from_scan_control(sc(0, 0), 12), DropoutMode::None, "SCANCTRL 0 = off");
        assert_eq!(dropout_from_scan_control(sc(0xFF, 0), 200), DropoutMode::Simple, "0xFF = always");
        assert_eq!(dropout_from_scan_control(sc(0x111, 0), 17), DropoutMode::Simple, "ppem <= 17");
        assert_eq!(dropout_from_scan_control(sc(0x111, 0), 18), DropoutMode::None, "ppem > 17");
        assert_eq!(dropout_from_scan_control(sc(0x1FF, 4), 9), DropoutMode::Smart, "SCANTYPE 4 = smart");
        assert_eq!(dropout_from_scan_control(sc(0x1FF, 6), 9), DropoutMode::None, "SCANTYPE 6 = off");
    }
}
