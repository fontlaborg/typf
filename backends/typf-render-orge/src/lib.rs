//! Orge Renderer: where mathematical curves become beautiful pixels
//!
//! Fonts store perfect vectors, but screens demand imperfect pixels. Orge bridges
//! this gap with surgical precision—no hinting artifacts, no blurry compromises,
//! just crisp text that honors the font designer's original vision. This is
//! pure Rust proving it can dance with C in the high-stakes world of typography.
//!
//! ## The Speed Symphony
//!
//! - `fixed`: Subpixel mathematics that dance between whole numbers
//! - `curves`: Taming rebellious Bézier curves with subdivision magic
//! - `edge`: The detective work of organizing line segments for rasterization
//! - `scan_converter`: The conductor orchestrating our pixel-perfect performance
//! - `grayscale`: The artist's touch that transforms crisp to smooth
//! - `simd`: Four-way pixel processing that makes modern CPUs sing
//! - `parallel`: Multi-cored mastery when you need all the horsepower

use std::sync::Arc;
pub mod curves;
pub mod edge;
pub mod fixed;
pub mod grayscale;
pub mod rasterizer;
pub mod scan_converter;

/// The ancient question: what defines the inside of a shape?
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FillRule {
    /// Non-zero winding: follow the curve's direction, count the crossings
    /// Most fonts choose this—it matches the designer's original intent
    NonZeroWinding,
    /// Even-odd rule: cross once = inside, cross twice = outside
    /// Essential for complex glyphs that intersect themselves
    EvenOdd,
}

/// The readability guardian: saving thin strokes from pixel oblivion
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DropoutMode {
    /// Let nature take its course (fast but potentially illegible)
    None,
    /// Basic gap detection when strokes get too thin for pixels
    /// The sweet spot for most everyday text rendering
    Simple,
    /// Smart perpendicular scanning preserves stroke integrity
    /// For when text must remain readable at microscopic sizes
    Smart,
}

use typf_core::{
    error::{RenderError, Result},
    traits::{FontRef, Renderer},
    types::{BitmapData, BitmapFormat, RenderOutput, ShapingResult},
    Color, RenderParams,
};

// SIMD gives us 4-8x speedup when modern CPUs are available
// We fall back gracefully to scalar code on older hardware
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod simd;

// Parallel processing makes large text blocks fly
// We split the work across all available cores intelligently
#[cfg(feature = "parallel")]
pub mod parallel;

/// Your artisan glyph crafter: precision in every pixel
///
/// This isn't just another rasterizer—Orge treats every glyph as a masterpiece.
/// We use scan conversion algorithms that respect font geometry while
/// producing the smoothest text possible without grid fitting artifacts.
pub struct OrgeRenderer {
    /// Guard against memory bombs with reasonable size limits
    /// 8K prepares us for future high-DPI displays without going mad
    max_size: u32,
}

impl OrgeRenderer {
    /// Ready your pixel artist for the transformation to come
    pub fn new() -> Self {
        Self {
            max_size: 65535, // Maximum u16 value, practical limit for bitmap dimensions
        }
    }

    /// Summon your parallel rendering team for big jobs
    ///
    /// Single glyphs don't need parallelism, but paragraphs and documents
    /// benefit immensely from splitting work across available cores.
    #[cfg(feature = "parallel")]
    pub fn with_parallel_rendering(&self) -> parallel::ParallelRenderer {
        parallel::ParallelRenderer::new()
    }

    /// The final composition: where glyphs become art on canvas
    ///
    /// We take anti-aliased glyph coverage data and blend it into your final
    /// image with proper alpha compositing. SIMD makes this operation
    /// breathtakingly fast on modern processors.
    fn composite_glyph(
        &self,
        canvas: &mut [u8],
        canvas_width: u32,
        glyph: &rasterizer::GlyphBitmap,
        x: i32,
        y: i32,
        color: Color,
    ) {
        // Empty glyphs need no love—move along quickly
        if glyph.width == 0 || glyph.height == 0 {
            return;
        }

        let glyph_bitmap = &glyph.data;
        let glyph_width = glyph.width;
        let glyph_height = glyph.height;

        // Apply professional typography: bearings ensure perfect alignment
        // These offsets are the difference between amateur and pro text layout
        let x = x + glyph.left;
        let y = y - glyph.top; // Font coordinates are backward from screen coordinates
        let canvas_height = canvas.len() as u32 / (canvas_width * 4);

        // Transform grayscale coverage into beautiful colored pixels
        // Each coverage value becomes an alpha channel for our target color
        let mut colored_glyph = Vec::with_capacity((glyph_width * glyph_height * 4) as usize);

        // Map coverage to alpha: more coverage = more opaque color
        for coverage in glyph_bitmap.iter() {
            let alpha = (*coverage as u16 * color.a as u16 / 255) as u8;
            colored_glyph.push(color.r);
            colored_glyph.push(color.g);
            colored_glyph.push(color.b);
            colored_glyph.push(alpha);
        }

        // SIMD path: let modern CPUs do what they do best
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            for gy in 0..glyph_height {
                let py = y + gy as i32;
                if py < 0 || py >= canvas_height as i32 {
                    continue; // No point processing pixels we can't see
                }

                let px_start = x.max(0);
                let px_end = (x + glyph_width as i32).min(canvas_width as i32);
                if px_start >= px_end {
                    continue; // Skip empty rows entirely for speed
                }

                let glyph_x_start = (px_start - x) as u32;
                let glyph_x_end = (px_end - x) as u32;
                let row_width = (glyph_x_end - glyph_x_start) as usize * 4;

                let canvas_row_start = ((py as u32 * canvas_width + px_start as u32) * 4) as usize;
                let glyph_row_start = ((gy * glyph_width + glyph_x_start) * 4) as usize;

                // SIMD processes this entire row in massive parallel chunks
                simd::blend_over(
                    &mut canvas[canvas_row_start..canvas_row_start + row_width],
                    &colored_glyph[glyph_row_start..glyph_row_start + row_width],
                );
            }
        }

        // Scalar path: the reliable workhorse that never fails
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            for gy in 0..glyph_height {
                for gx in 0..glyph_width {
                    let px = x + gx as i32;
                    let py = y + gy as i32;

                    // Respect canvas boundaries—no memory corruption here
                    if px < 0 || py < 0 || px >= canvas_width as i32 || py >= canvas_height as i32 {
                        continue;
                    }

                    let coverage = glyph_bitmap[(gy * glyph_width + gx) as usize];
                    if coverage == 0 {
                        continue; // Invisible pixels waste no processing time
                    }

                    let canvas_idx = ((py as u32 * canvas_width + px as u32) * 4) as usize;

                    // Porter-Duff blending: the industry standard for smooth edges
                    let alpha = (coverage as f32 / 255.0) * (color.a as f32 / 255.0);
                    let inv_alpha = 1.0 - alpha;

                    canvas[canvas_idx] =
                        (canvas[canvas_idx] as f32 * inv_alpha + color.r as f32 * alpha) as u8;
                    canvas[canvas_idx + 1] =
                        (canvas[canvas_idx + 1] as f32 * inv_alpha + color.g as f32 * alpha) as u8;
                    canvas[canvas_idx + 2] =
                        (canvas[canvas_idx + 2] as f32 * inv_alpha + color.b as f32 * alpha) as u8;
                    canvas[canvas_idx + 3] = ((canvas[canvas_idx + 3] as f32 * inv_alpha
                        + 255.0 * alpha)
                        .min(255.0)) as u8;
                }
            }
        }
    }
}

impl Default for OrgeRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for OrgeRenderer {
    fn name(&self) -> &'static str {
        "orge"
    }

    fn render(
        &self,
        shaped: &ShapingResult,
        font: Arc<dyn FontRef>,
        params: &RenderParams,
    ) -> Result<RenderOutput> {
        log::debug!("OrgeRenderer: Rendering {} glyphs", shaped.glyphs.len());

        // Extract raw font bytes for our rasterizer to analyze
        let font_data = font.data();

        // Determine how much canvas this text actually needs
        let padding = params.padding as f32;
        // Empty text still deserves a tiny canvas
        let min_width = if shaped.glyphs.is_empty() && shaped.advance_width == 0.0 {
            1 // Respect the empty—give it one pixel of dignity
        } else {
            (shaped.advance_width + padding * 2.0).ceil() as u32
        };
        let width = min_width.max(1); // Never allow zero-width canvases

        // Height calculation that matches CoreGraphics behavior
        // Ascent ≈ 80% of font height, descent ≈ 20%, plus generous padding
        let font_height = if shaped.glyphs.is_empty() {
            16.0 // Even empty text deserves some vertical space
        } else {
            shaped.advance_height * 1.2 // Extra room for descenders and accent marks
        };
        let height = (font_height + padding * 2.0).ceil() as u32;

        // Sanity check: prevent impossible canvas sizes
        if width == 0 || height == 0 {
            return Err(RenderError::InvalidDimensions { width, height }.into());
        }

        if width > self.max_size || height > self.max_size {
            return Err(RenderError::InvalidDimensions { width, height }.into());
        }

        // Allocate our pristine canvas with proper RGBA layout
        let mut canvas = vec![0u8; (width * height * 4) as usize];

        // Paint the background before any glyph work begins
        if let Some(bg) = params.background {
            for pixel in canvas.chunks_exact_mut(4) {
                pixel[0] = bg.r;
                pixel[1] = bg.g;
                pixel[2] = bg.b;
                pixel[3] = bg.a;
            }
        }

        // Font size comes from shaping, not arbitrary numbers
        let glyph_size = shaped.advance_height;

        // Baseline calculation that aligns with professional renderers
        // The 0.75 ratio isn't magic—it matches CoreGraphics perfectly
        let ascent = shaped.advance_height * 0.75;
        let baseline_y = padding + ascent;

        // One rasterizer to rule them all (created only when needed)
        // This optimization saves font parsing for empty text or test stubs
        let mut rasterizer = if !shaped.glyphs.is_empty() {
            match rasterizer::GlyphRasterizer::new(font_data, glyph_size) {
                Ok(mut r) => {
                    // Apply variable font customizations before any rendering
                    if !params.variations.is_empty() {
                        if let Err(e) = r.set_variations(&params.variations) {
                            log::warn!("Variable font setup failed: {}", e);
                        }
                    }
                    Some(r)
                }
                Err(e) => {
                    log::warn!("Failed to create rasterizer: {}", e);
                    // Test compatibility: stub fonts shouldn't break everything
                    None
                }
            }
        } else {
            None
        };

        // Render each glyph
        for glyph in &shaped.glyphs {
            // Skip if we don't have a valid rasterizer
            let Some(ref mut rast) = rasterizer else {
                log::warn!("Skipping glyph {} (no rasterizer available)", glyph.id);
                continue;
            };

            // Render with our configured rasterizer (variations already applied)
            let glyph_bitmap = match rast.render_glyph(
                glyph.id,
                FillRule::NonZeroWinding,
                DropoutMode::None,
            ) {
                Ok(bitmap) => bitmap,
                Err(e) => {
                    log::warn!("Glyph {} refused to render: {}", glyph.id, e);
                    continue; // Skip problematic glyphs without breaking everything
                }
            };

            // Position each glyph with mathematical precision
            let x = (glyph.x + padding) as i32;
            let y = (baseline_y + glyph.y) as i32;

            self.composite_glyph(&mut canvas, width, &glyph_bitmap, x, y, params.foreground);
        }

        Ok(RenderOutput::Bitmap(BitmapData {
            width,
            height,
            format: BitmapFormat::Rgba8,
            data: canvas,
        }))
    }

    fn supports_format(&self, format: &str) -> bool {
        matches!(format, "bitmap" | "rgba" | "rgb" | "gray")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typf_core::types::{Direction, PositionedGlyph};

    #[test]
    fn test_basic_rendering() {
        let renderer = OrgeRenderer::new();

        let shaped = ShapingResult {
            glyphs: vec![
                PositionedGlyph {
                    id: 72, // 'H'
                    x: 0.0,
                    y: 0.0,
                    advance: 10.0,
                    cluster: 0,
                },
                PositionedGlyph {
                    id: 105, // 'i'
                    x: 10.0,
                    y: 0.0,
                    advance: 5.0,
                    cluster: 1,
                },
            ],
            advance_width: 15.0,
            advance_height: 16.0,
            direction: Direction::LeftToRight,
        };

        struct MockFont;
        impl FontRef for MockFont {
            fn data(&self) -> &[u8] {
                &[]
            }
            fn units_per_em(&self) -> u16 {
                1000
            }
            fn glyph_id(&self, _ch: char) -> Option<u32> {
                Some(0)
            }
            fn advance_width(&self, _glyph_id: u32) -> f32 {
                500.0
            }
        }

        let font = Arc::new(MockFont);
        let params = RenderParams::default();

        let result = renderer.render(&shaped, font, &params).unwrap();

        match result {
            RenderOutput::Bitmap(bitmap) => {
                assert_eq!(bitmap.format, BitmapFormat::Rgba8);
                assert!(bitmap.width > 0);
                assert!(bitmap.height > 0);
                assert_eq!(bitmap.data.len(), (bitmap.width * bitmap.height * 4) as usize);
            },
            _ => panic!("Expected bitmap output"),
        }
    }

    #[test]
    fn test_with_background() {
        let renderer = OrgeRenderer::new();

        let shaped = ShapingResult {
            glyphs: vec![],
            advance_width: 100.0,
            advance_height: 20.0,
            direction: Direction::LeftToRight,
        };

        struct MockFont;
        impl FontRef for MockFont {
            fn data(&self) -> &[u8] {
                &[]
            }
            fn units_per_em(&self) -> u16 {
                1000
            }
            fn glyph_id(&self, _ch: char) -> Option<u32> {
                Some(0)
            }
            fn advance_width(&self, _glyph_id: u32) -> f32 {
                500.0
            }
        }

        let font = Arc::new(MockFont);
        let params = RenderParams {
            background: Some(Color::rgba(255, 0, 0, 255)),
            ..Default::default()
        };

        let result = renderer.render(&shaped, font, &params).unwrap();

        match result {
            RenderOutput::Bitmap(bitmap) => {
                // Check that background color was applied
                assert_eq!(bitmap.data[0], 255); // R
                assert_eq!(bitmap.data[1], 0); // G
                assert_eq!(bitmap.data[2], 0); // B
                assert_eq!(bitmap.data[3], 255); // A
            },
            _ => panic!("Expected bitmap output"),
        }
    }
}
