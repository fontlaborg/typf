//! Zeno rasterizer backend for TYPF
//!
//! This module provides high-quality grayscale glyph rasterization using the Zeno library.
//! Ported from haforu with exact algorithm preservation for pixel-perfect parity.

use read_fonts::TableProvider;
use skrifa::instance::Size;
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::MetadataProvider;
use std::path::Path;
use thiserror::Error;
use typf_fontdb::font_cache::FontInstance;
use zeno::{Command, Mask, Transform};

/// Error types for zeno rendering
#[derive(Error, Debug)]
pub enum ZenoError {
    #[error("Invalid render parameters: {0}")]
    InvalidParams(String),

    #[error("Failed to rasterize glyph {glyph_id}: {reason}")]
    RasterizationFailed { glyph_id: u32, reason: String },

    #[error("Font error: {0}")]
    FontError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, ZenoError>;

/// Fallback delta value for incomparable images
pub const PIXEL_DELTA_FALLBACK: f64 = 999_999.0;

/// Grayscale image with validation and metrics
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
            return Err(ZenoError::InvalidParams(
                "Image dimensions must be non-zero".to_string(),
            ));
        }
        let expected = (width as usize) * (height as usize);
        if pixels.len() != expected {
            return Err(ZenoError::Internal(format!(
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

    /// Total number of pixels
    fn len(&self) -> usize {
        self.pixels.len()
    }

    /// Return true when every pixel is zero (blank render)
    pub fn is_empty(&self) -> bool {
        self.pixels.iter().all(|&px| px == 0)
    }

    /// Calculate tight bounding box of non-zero pixels (x, y, width, height)
    #[inline]
    pub fn calculate_bbox(&self) -> (u32, u32, u32, u32) {
        let mut min_x = self.width;
        let mut min_y = self.height;
        let mut max_x = 0u32;
        let mut max_y = 0u32;

        for y in 0..self.height {
            let row_start = (y * self.width) as usize;
            let row_end = row_start + self.width as usize;
            let row = &self.pixels[row_start..row_end];

            let has_content = Self::has_nonzero_simd(row);
            if !has_content {
                continue;
            }

            min_y = min_y.min(y);
            max_y = max_y.max(y);

            for (x, &px) in row.iter().enumerate() {
                if px > 0 {
                    min_x = min_x.min(x as u32);
                    max_x = max_x.max(x as u32);
                }
            }
        }

        if min_x > max_x {
            return (0, 0, 0, 0);
        }

        (min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
    }

    /// Check if slice has any non-zero bytes (SIMD-accelerated on x86_64)
    #[inline]
    fn has_nonzero_simd(slice: &[u8]) -> bool {
        #[cfg(target_arch = "x86_64")]
        {
            use std::arch::x86_64::*;

            unsafe {
                let len = slice.len();
                let mut i = 0;
                let zeros = _mm256_setzero_si256();

                while i + 32 <= len {
                    let chunk = _mm256_loadu_si256(slice[i..].as_ptr() as *const __m256i);
                    let cmp = _mm256_cmpeq_epi8(chunk, zeros);
                    let mask = _mm256_movemask_epi8(cmp) as u32;

                    if mask != 0xFFFFFFFF {
                        return true;
                    }

                    i += 32;
                }

                slice[i..].iter().any(|&px| px > 0)
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            slice.iter().any(|&px| px > 0)
        }
    }

    /// Compute normalized pixel density (0.0 - 1.0)
    #[inline]
    pub fn density(&self) -> f64 {
        if self.is_empty() {
            return 0.0;
        }

        #[cfg(target_arch = "x86_64")]
        {
            self.density_simd()
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            self.density_scalar()
        }
    }

    #[inline]
    fn density_scalar(&self) -> f64 {
        let sum: u64 = self.pixels.iter().map(|&px| px as u64).sum();
        let denom = (self.len() as u64) * 255u64;
        if denom == 0 {
            return 0.0;
        }
        let density = sum as f64 / denom as f64;
        density.clamp(0.0, 1.0)
    }

    #[cfg(target_arch = "x86_64")]
    #[inline]
    fn density_simd(&self) -> f64 {
        use std::arch::x86_64::*;

        unsafe {
            let mut sum = _mm256_setzero_si256();
            let mut i = 0;
            let len = self.pixels.len();

            while i + 32 <= len {
                let chunk = _mm256_loadu_si256(self.pixels[i..].as_ptr() as *const __m256i);

                let zeros = _mm256_setzero_si256();
                let low = _mm256_unpacklo_epi8(chunk, zeros);
                let high = _mm256_unpackhi_epi8(chunk, zeros);

                let low_32 = _mm256_madd_epi16(low, _mm256_set1_epi16(1));
                let high_32 = _mm256_madd_epi16(high, _mm256_set1_epi16(1));

                sum = _mm256_add_epi32(sum, low_32);
                sum = _mm256_add_epi32(sum, high_32);

                i += 32;
            }

            let sum_array: [i32; 8] = std::mem::transmute(sum);
            let total: u64 = sum_array.iter().map(|&x| x as u64).sum();
            let remainder: u64 = self.pixels[i..].iter().map(|&px| px as u64).sum();
            let final_sum = total + remainder;

            let denom = (self.len() as u64) * 255u64;
            if denom == 0 {
                return 0.0;
            }
            let density = final_sum as f64 / denom as f64;
            density.clamp(0.0, 1.0)
        }
    }

    /// Compute longest contiguous non-zero run ratio (0.0 - 1.0)
    #[inline]
    pub fn beam(&self) -> f64 {
        if self.len() == 0 {
            return 0.0;
        }

        #[cfg(target_arch = "x86_64")]
        {
            self.beam_simd()
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            self.beam_scalar()
        }
    }

    #[inline]
    fn beam_scalar(&self) -> f64 {
        let mut best = 0usize;
        let mut current = 0usize;
        for &px in &self.pixels {
            if px > 0 {
                current += 1;
                best = best.max(current);
            } else {
                current = 0;
            }
        }
        let ratio = best as f64 / self.len() as f64;
        ratio.clamp(0.0, 1.0)
    }

    #[cfg(target_arch = "x86_64")]
    #[inline]
    fn beam_simd(&self) -> f64 {
        use std::arch::x86_64::*;

        unsafe {
            let mut best = 0usize;
            let mut current = 0usize;
            let mut i = 0;
            let len = self.pixels.len();
            let zeros = _mm256_setzero_si256();

            while i + 32 <= len {
                let chunk = _mm256_loadu_si256(self.pixels[i..].as_ptr() as *const __m256i);
                let cmp = _mm256_cmpeq_epi8(chunk, zeros);
                let mask = _mm256_movemask_epi8(cmp) as u32;

                for j in 0..32 {
                    let is_zero = (mask & (1 << j)) != 0;
                    if !is_zero {
                        current += 1;
                        best = best.max(current);
                    } else {
                        current = 0;
                    }
                }

                i += 32;
            }

            for &px in &self.pixels[i..] {
                if px > 0 {
                    current += 1;
                    best = best.max(current);
                } else {
                    current = 0;
                }
            }

            let ratio = best as f64 / self.len() as f64;
            ratio.clamp(0.0, 1.0)
        }
    }
}

/// Glyph rasterizer using Zeno
pub struct GlyphRasterizer;

impl GlyphRasterizer {
    /// Create a new glyph rasterizer
    pub fn new() -> Self {
        Self
    }

    /// Render shaped text to a grayscale image
    ///
    /// Shaped text structure expected to have:
    /// - glyphs: Vec<ShapedGlyph> with glyph_id, x_offset, y_offset, x_advance
    /// - font_size: f32
    pub fn render_text<T>(
        &self,
        font_instance: &FontInstance,
        shaped: &T,
        width: u32,
        height: u32,
        tracking: f32,
        _path: &Path,
    ) -> Result<Image>
    where
        T: ShapedTextAccess,
    {
        let mut canvas = vec![0u8; (width * height) as usize];

        if shaped.glyphs().is_empty() {
            return Image::new(width, height, canvas);
        }

        let font = font_instance.font_ref();
        let user_coords = font_instance.location();
        let axes = font.axes();
        let location = axes.location(user_coords.iter().copied());
        let location_ref = location.coords();

        let head = font
            .head()
            .map_err(|e| ZenoError::FontError(format!("Failed to read head table: {}", e)))?;
        let upem = head.units_per_em();
        let scale = shaped.font_size() / upem as f32;

        let baseline_y = height as f32 * 0.75;
        let mut cursor_x = 0.0f32;

        for glyph in shaped.glyphs() {
            let glyph_id = glyph.glyph_id();

            let outline = font.outline_glyphs();
            let Some(glyph_outline) = outline.get(glyph_id.into()) else {
                log::warn!("Glyph ID {} not found in font", glyph_id);
                cursor_x += (glyph.x_advance() as f32 + tracking) * scale;
                continue;
            };

            let mut path_commands = Vec::new();
            let mut pen = ZenoPen::new(&mut path_commands);

            let draw_settings = DrawSettings::unhinted(Size::unscaled(), location_ref);
            if let Err(e) = glyph_outline.draw(draw_settings, &mut pen) {
                return Err(ZenoError::RasterizationFailed {
                    glyph_id,
                    reason: format!("Failed to draw outline: {}", e),
                });
            }

            let glyph_x = cursor_x + (glyph.x_offset() as f32 * scale);
            let glyph_y = baseline_y - (glyph.y_offset() as f32 * scale);

            self.composite_glyph(
                &mut canvas,
                &path_commands,
                glyph_x,
                glyph_y,
                scale,
                width,
                height,
            )?;

            cursor_x += (glyph.x_advance() as f32 + tracking) * scale;
        }

        // Invert pixels (zeno renders white on black, we want black on white)
        for pixel in &mut canvas {
            *pixel = 255 - *pixel;
        }

        Image::new(width, height, canvas)
    }

    fn composite_glyph(
        &self,
        canvas: &mut [u8],
        path: &[Command],
        x: f32,
        y: f32,
        scale: f32,
        width: u32,
        height: u32,
    ) -> Result<()> {
        let transform = Transform::scale(scale, scale).then_translate(x, y);

        let mut mask = Mask::new(path);
        mask.size(width, height).transform(Some(transform));

        let (alpha_data, placement) = mask.render();

        let top = placement.top.max(0) as u32;
        let left = placement.left.max(0) as u32;
        let bottom = (placement.top + placement.height as i32).min(height as i32) as u32;
        let right = (placement.left + placement.width as i32).min(width as i32) as u32;

        for py in top..bottom {
            for px in left..right {
                let canvas_idx = (py * width + px) as usize;
                let mask_y = (py as i32 - placement.top) as u32;
                let mask_x = (px as i32 - placement.left) as u32;
                let mask_idx = (mask_y * placement.width + mask_x) as usize;

                if mask_idx < alpha_data.len() {
                    let alpha = alpha_data[mask_idx];
                    let src = canvas[canvas_idx];
                    let blended =
                        src.saturating_add(((alpha as u16 * (255 - src) as u16) / 255) as u8);
                    canvas[canvas_idx] = blended;
                }
            }
        }

        Ok(())
    }
}

impl Default for GlyphRasterizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Outline pen that converts skrifa outlines to Zeno commands
struct ZenoPen<'a> {
    commands: &'a mut Vec<Command>,
}

impl<'a> ZenoPen<'a> {
    fn new(commands: &'a mut Vec<Command>) -> Self {
        Self { commands }
    }
}

impl<'a> OutlinePen for ZenoPen<'a> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.commands.push(Command::MoveTo([x, -y].into()));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.commands.push(Command::LineTo([x, -y].into()));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.commands
            .push(Command::QuadTo([cx0, -cy0].into(), [x, -y].into()));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.commands.push(Command::CurveTo(
            [cx0, -cy0].into(),
            [cx1, -cy1].into(),
            [x, -y].into(),
        ));
    }

    fn close(&mut self) {
        self.commands.push(Command::Close);
    }
}

/// Trait for accessing shaped text data (abstraction for different shaped text types)
pub trait ShapedTextAccess {
    type Glyph: ShapedGlyphAccess;

    fn glyphs(&self) -> &[Self::Glyph];
    fn font_size(&self) -> f32;
}

/// Trait for accessing shaped glyph data
pub trait ShapedGlyphAccess {
    fn glyph_id(&self) -> u32;
    fn x_offset(&self) -> i32;
    fn y_offset(&self) -> i32;
    fn x_advance(&self) -> i32;
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
    fn calculate_bbox_handles_basic_shapes() {
        let mut pixels = vec![0u8; 100 * 50];
        assert_eq!(
            Image::new(100, 50, pixels.clone())
                .unwrap()
                .calculate_bbox(),
            (0, 0, 0, 0)
        );

        pixels[25 * 100 + 50] = 255;
        assert_eq!(
            Image::new(100, 50, pixels).unwrap().calculate_bbox(),
            (50, 25, 1, 1)
        );
    }

    #[test]
    fn density_computes_correctly() {
        let img = Image::new(4, 4, vec![0u8; 16]).unwrap();
        assert_eq!(img.density(), 0.0);

        let img = Image::new(4, 4, vec![255u8; 16]).unwrap();
        assert_eq!(img.density(), 1.0);

        let img = Image::new(4, 4, vec![128u8; 16]).unwrap();
        assert!((img.density() - 0.502).abs() < 0.01);
    }

    #[test]
    fn beam_computes_correctly() {
        let img = Image::new(4, 4, vec![0u8; 16]).unwrap();
        assert_eq!(img.beam(), 0.0);

        let img = Image::new(4, 4, vec![255u8; 16]).unwrap();
        assert_eq!(img.beam(), 1.0);

        let mut pixels = vec![0u8; 16];
        pixels[0] = 1;
        pixels[1] = 1;
        pixels[2] = 1;
        let img = Image::new(4, 4, pixels).unwrap();
        assert_eq!(img.beam(), 3.0 / 16.0);
    }
}
