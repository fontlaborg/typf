//! Monochrome rasterizer for Typf.
//!
//! Opixa is the pure-Rust renderer that turns shaped glyph outlines into pixel
//! coverage data. It is focused on predictable outline rasterization rather than
//! color-glyph support. The submodules divide that work into fixed-point math,
//! curve flattening, edge handling, scan conversion, and optional SIMD or
//! parallel acceleration.

use std::sync::Arc;

pub mod curves;
pub mod edge;
pub mod fixed;
pub mod glyph_cache;
pub mod grayscale;
pub mod rasterizer;
pub mod scan_converter;

/// Rule for deciding which parts of a path count as inside.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FillRule {
    /// Non-zero winding. Count edge crossings with direction.
    NonZeroWinding,
    /// Even-odd rule. An odd number of crossings means inside.
    EvenOdd,
}

/// Strategy for preserving very thin strokes.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DropoutMode {
    /// Do not apply dropout handling.
    None,
    /// Basic protection for thin strokes that might otherwise disappear.
    Simple,
    /// More expensive dropout handling for extreme small-size rendering.
    Smart,
}

use typf_core::{
    error::{RenderError, Result},
    traits::{FontRef, Renderer},
    types::{BitmapData, BitmapFormat, RenderOutput, ShapingResult},
    Color, GlyphSource, RenderParams,
};

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod simd;

#[cfg(feature = "parallel")]
pub mod parallel;

/// Renderer that rasterizes outline glyphs into bitmaps.
///
/// It applies scan conversion to glyph outlines and composites the resulting
/// coverage onto the target bitmap. Caching is optional and can avoid repeated
/// rasterization of the same glyph at the same size.
pub struct OpixaRenderer {
    max_width: u32,
    max_height: u32,
    max_pixels: u64,
    cache: Option<Arc<glyph_cache::GlyphCache>>,
}

impl OpixaRenderer {
    pub fn new() -> Self {
        Self {
            max_width: typf_core::get_max_bitmap_width(),
            max_height: typf_core::get_max_bitmap_height(),
            max_pixels: typf_core::get_max_bitmap_pixels(),
            cache: None,
        }
    }

    pub fn with_cache() -> Self {
        Self::with_cache_capacity(1000)
    }

    pub fn with_cache_capacity(capacity: usize) -> Self {
        Self {
            max_width: typf_core::get_max_bitmap_width(),
            max_height: typf_core::get_max_bitmap_height(),
            max_pixels: typf_core::get_max_bitmap_pixels(),
            cache: Some(Arc::new(glyph_cache::GlyphCache::new(capacity))),
        }
    }

    pub fn cache_stats(&self) -> Option<glyph_cache::GlyphCacheStats> {
        self.cache.as_ref().map(|c| c.stats())
    }

    pub fn cache_hit_rate(&self) -> Option<f64> {
        self.cache.as_ref().map(|c| c.hit_rate())
    }

    pub fn clear_cache(&self) {
        if let Some(ref cache) = self.cache {
            cache.clear();
        }
    }

    /// Create a parallel wrapper for larger rendering workloads.
    #[cfg(feature = "parallel")]
    pub fn with_parallel_rendering(&self) -> parallel::ParallelRenderer {
        parallel::ParallelRenderer::new()
    }

    /// Blend one rasterized glyph bitmap onto the destination canvas.
    fn composite_glyph(
        &self,
        canvas: &mut [u8],
        canvas_width: u32,
        glyph: &rasterizer::GlyphBitmap,
        x: i32,
        y: i32,
        color: Color,
    ) {
        if glyph.width == 0 || glyph.height == 0 {
            return;
        }

        let glyph_bitmap = &glyph.data;
        let glyph_width = glyph.width;
        let glyph_height = glyph.height;

        let x = x + glyph.left;
        let y = y - glyph.top;
        let canvas_height = canvas.len() as u32 / (canvas_width * 4);

        let mut colored_glyph = Vec::with_capacity((glyph_width * glyph_height * 4) as usize);

        for coverage in glyph_bitmap.iter() {
            let alpha = (*coverage as u16 * color.a as u16 / 255) as u8;
            colored_glyph.push(color.r);
            colored_glyph.push(color.g);
            colored_glyph.push(color.b);
            colored_glyph.push(alpha);
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            for gy in 0..glyph_height {
                let py = y + gy as i32;
                if py < 0 || py >= canvas_height as i32 {
                    continue;
                }

                let px_start = x.max(0);
                let px_end = (x + glyph_width as i32).min(canvas_width as i32);
                if px_start >= px_end {
                    continue;
                }

                let glyph_x_start = (px_start - x) as u32;
                let glyph_x_end = (px_end - x) as u32;
                let row_width = (glyph_x_end - glyph_x_start) as usize * 4;

                let canvas_row_start = ((py as u32 * canvas_width + px_start as u32) * 4) as usize;
                let glyph_row_start = ((gy * glyph_width + glyph_x_start) * 4) as usize;

                simd::blend_over(
                    &mut canvas[canvas_row_start..canvas_row_start + row_width],
                    &colored_glyph[glyph_row_start..glyph_row_start + row_width],
                );
            }
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            for gy in 0..glyph_height {
                for gx in 0..glyph_width {
                    let px = x + gx as i32;
                    let py = y + gy as i32;

                    if px < 0 || py < 0 || px >= canvas_width as i32 || py >= canvas_height as i32 {
                        continue;
                    }

                    let coverage = glyph_bitmap[(gy * glyph_width + gx) as usize];
                    if coverage == 0 {
                        continue;
                    }

                    let canvas_idx = ((py as u32 * canvas_width + px as u32) * 4) as usize;

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

impl Default for OpixaRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for OpixaRenderer {
    fn name(&self) -> &'static str {
        "opixa"
    }

    fn render(
        &self,
        shaped: &ShapingResult,
        font: Arc<dyn FontRef>,
        params: &RenderParams,
    ) -> Result<RenderOutput> {
        log::debug!("OpixaRenderer: Rendering {} glyphs", shaped.glyphs.len());

        let allows_outline = params
            .glyph_sources
            .effective_order()
            .iter()
            .any(|s| matches!(s, GlyphSource::Glyf | GlyphSource::Cff | GlyphSource::Cff2));
        if !allows_outline {
            return Err(RenderError::BackendError(
                "opixa renderer requires outline glyph sources".to_string(),
            )
            .into());
        }

        let font_data = font.data();
        let padding = params.padding as f32;
        let glyph_size = shaped.advance_height;

        let mut rendered_glyphs: Vec<RenderedGlyph> = Vec::new();
        let mut min_y: f32 = 0.0;
        let mut max_y: f32 = 0.0;

        let mut rasterizer = if !shaped.glyphs.is_empty() {
            match rasterizer::GlyphRasterizer::new(font_data, glyph_size) {
                Ok(mut r) => {
                    if !params.variations.is_empty() {
                        if let Err(e) = r.set_variations(&params.variations) {
                            log::warn!("Variable font setup failed: {}", e);
                        }
                    }
                    Some(r)
                },
                Err(e) => {
                    log::warn!("Failed to create rasterizer: {}", e);
                    None
                },
            }
        } else {
            None
        };

        for glyph in &shaped.glyphs {
            let glyph_bitmap = if let Some(ref cache) = self.cache {
                let cache_key = glyph_cache::GlyphCacheKey::new(
                    font_data,
                    glyph.id,
                    glyph_size,
                    &params.variations,
                );

                if let Some(cached) = cache.get(&cache_key) {
                    cached
                } else {
                    let Some(ref mut rast) = rasterizer else {
                        log::warn!("Skipping glyph {} (no rasterizer available)", glyph.id);
                        continue;
                    };

                    let bitmap = match rast.render_glyph(
                        glyph.id,
                        FillRule::NonZeroWinding,
                        DropoutMode::None,
                    ) {
                        Ok(b) => b,
                        Err(e) => {
                            log::warn!("Glyph {} rasterization failed: {}", glyph.id, e);
                            continue;
                        },
                    };

                    cache.insert(cache_key, bitmap.clone());
                    bitmap
                }
            } else {
                let Some(ref mut rast) = rasterizer else {
                    log::warn!("Skipping glyph {} (no rasterizer available)", glyph.id);
                    continue;
                };

                match rast.render_glyph(glyph.id, FillRule::NonZeroWinding, DropoutMode::None) {
                    Ok(bitmap) => bitmap,
                    Err(e) => {
                        log::warn!("Glyph {} rasterization failed: {}", glyph.id, e);
                        continue;
                    },
                }
            };

            if glyph_bitmap.width == 0 || glyph_bitmap.height == 0 {
                continue;
            }

            let glyph_top = glyph.y + glyph_bitmap.top as f32;
            let glyph_bottom = glyph.y + glyph_bitmap.top as f32 - glyph_bitmap.height as f32;

            max_y = max_y.max(glyph_top);
            min_y = min_y.min(glyph_bottom);

            rendered_glyphs.push(RenderedGlyph {
                bitmap: glyph_bitmap,
                glyph_x: glyph.x,
                glyph_y: glyph.y,
            });
        }

        let min_width = if shaped.glyphs.is_empty() && shaped.advance_width == 0.0 {
            1
        } else {
            (shaped.advance_width + padding * 2.0).ceil() as u32
        };
        let width = min_width.max(1);

        let (metrics_ascent, metrics_descent) = font
            .metrics()
            .filter(|m| m.units_per_em > 0 && (m.ascent != 0 || m.descent != 0))
            .map(|m| {
                let scale = glyph_size / (m.units_per_em as f32);
                let ascent = (m.ascent as f32).max(0.0) * scale;
                let descent = (m.descent as f32).abs() * scale;
                (ascent, descent)
            })
            .unwrap_or((0.0, 0.0));

        let glyph_top = max_y.max(0.0);
        let glyph_bottom = (-min_y).max(0.0);
        let top = glyph_top.max(metrics_ascent);
        let bottom = glyph_bottom.max(metrics_descent);

        let content_height = if rendered_glyphs.is_empty() {
            16.0
        } else {
            top + bottom
        };
        let height = (content_height + padding * 2.0).ceil() as u32;

        if width == 0 || height == 0 {
            return Err(RenderError::ZeroDimensions { width, height }.into());
        }

        if width > self.max_width || height > self.max_height {
            return Err(RenderError::DimensionsTooLarge {
                width,
                height,
                max_width: self.max_width,
                max_height: self.max_height,
            }
            .into());
        }

        let total_pixels = width as u64 * height as u64;
        if total_pixels > self.max_pixels {
            return Err(RenderError::TotalPixelsTooLarge {
                width,
                height,
                total: total_pixels,
                max: self.max_pixels,
            }
            .into());
        }

        let mut canvas = vec![0u8; (width * height * 4) as usize];

        if let Some(bg) = params.background {
            for pixel in canvas.chunks_exact_mut(4) {
                pixel[0] = bg.r;
                pixel[1] = bg.g;
                pixel[2] = bg.b;
                pixel[3] = bg.a;
            }
        }

        let baseline_y = if rendered_glyphs.is_empty() {
            padding
        } else {
            padding + top
        };

        for rg in rendered_glyphs {
            let x = (rg.glyph_x + padding) as i32;
            let y = (baseline_y + rg.glyph_y) as i32;

            self.composite_glyph(&mut canvas, width, &rg.bitmap, x, y, params.foreground);
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

struct RenderedGlyph {
    bitmap: rasterizer::GlyphBitmap,
    glyph_x: f32,
    glyph_y: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use typf_core::{
        types::{Direction, PositionedGlyph},
        GlyphSource, GlyphSourcePreference,
    };

    #[test]
    fn test_basic_rendering() {
        let renderer = OpixaRenderer::new();

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
                assert_eq!(
                    bitmap.data.len(),
                    (bitmap.width * bitmap.height * 4) as usize
                );
            },
            _ => panic!("Expected bitmap output"),
        }
    }

    #[test]
    fn errors_when_outlines_denied() {
        let renderer = OpixaRenderer::new();

        let shaped = ShapingResult {
            glyphs: vec![PositionedGlyph {
                id: 1,
                x: 0.0,
                y: 0.0,
                advance: 10.0,
                cluster: 0,
            }],
            advance_width: 10.0,
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
                Some(1)
            }
            fn advance_width(&self, _glyph_id: u32) -> f32 {
                500.0
            }
        }

        let font = Arc::new(MockFont);
        let params = RenderParams {
            glyph_sources: GlyphSourcePreference::from_parts(vec![GlyphSource::Colr1], []),
            ..RenderParams::default()
        };

        let result = renderer.render(&shaped, font, &params);
        assert!(result.is_err(), "outline denial should be an error");
    }

    #[test]
    fn test_with_background() {
        let renderer = OpixaRenderer::new();

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
