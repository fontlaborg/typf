//! Orge Renderer - Basic bitmap rasterization
//!
//! A simple renderer that produces bitmap output without external dependencies.
//! Includes SIMD optimizations for high-performance blending operations.

use std::sync::Arc;
use typf_core::{
    error::{RenderError, Result},
    traits::{FontRef, Renderer},
    types::{BitmapData, BitmapFormat, RenderOutput, ShapingResult},
    Color, RenderParams,
};

// SIMD optimizations for supported architectures
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod simd;

// Parallel rendering support
#[cfg(feature = "parallel")]
pub mod parallel;

/// A basic bitmap renderer
pub struct OrgeRenderer {
    /// Maximum canvas size
    max_size: u32,
}

impl OrgeRenderer {
    /// Create a new OrgeRenderer
    pub fn new() -> Self {
        Self {
            max_size: 8192, // 8K max dimension
        }
    }

    /// Enable parallel rendering for better performance on multi-core systems
    #[cfg(feature = "parallel")]
    pub fn with_parallel_rendering(&self) -> parallel::ParallelRenderer {
        parallel::ParallelRenderer::new()
    }

    /// Create a simple bitmap for a glyph (placeholder implementation)
    fn render_glyph(&self, glyph_id: u32, size: f32) -> Vec<u8> {
        // For now, just create a simple box for each glyph
        // In a real implementation, this would rasterize the actual glyph outline
        let glyph_size = (size * 0.8) as usize;
        let mut bitmap = vec![0u8; glyph_size * glyph_size];

        // Draw a simple box outline
        for i in 0..glyph_size {
            // Top and bottom edges
            bitmap[i] = 255;
            bitmap[(glyph_size - 1) * glyph_size + i] = 255;

            // Left and right edges
            bitmap[i * glyph_size] = 255;
            bitmap[i * glyph_size + glyph_size - 1] = 255;
        }

        // Add some internal pattern based on glyph ID
        let pattern = (glyph_id % 4) as usize;
        for y in 2..glyph_size - 2 {
            for x in 2..glyph_size - 2 {
                if (x + y) % (pattern + 2) == 0 {
                    bitmap[y * glyph_size + x] = 128;
                }
            }
        }

        bitmap
    }

    /// Composite a grayscale glyph onto an RGBA canvas
    /// Uses SIMD optimizations when available for high-performance blending
    #[allow(clippy::too_many_arguments)]
    fn composite_glyph(
        &self,
        canvas: &mut [u8],
        canvas_width: u32,
        glyph_bitmap: &[u8],
        glyph_size: u32,
        x: i32,
        y: i32,
        color: Color,
    ) {
        let canvas_height = canvas.len() as u32 / (canvas_width * 4);

        // Create a temporary buffer for the colored glyph
        let mut colored_glyph = Vec::with_capacity((glyph_size * glyph_size * 4) as usize);

        // Convert grayscale glyph to RGBA with the specified color
        for coverage in glyph_bitmap.iter() {
            let alpha = (*coverage as u16 * color.a as u16 / 255) as u8;
            colored_glyph.push(color.r);
            colored_glyph.push(color.g);
            colored_glyph.push(color.b);
            colored_glyph.push(alpha);
        }

        // Try to use SIMD for row-by-row blending if possible
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            for gy in 0..glyph_size {
                let py = y + gy as i32;
                if py < 0 || py >= canvas_height as i32 {
                    continue;
                }

                let px_start = x.max(0);
                let px_end = (x + glyph_size as i32).min(canvas_width as i32);
                if px_start >= px_end {
                    continue;
                }

                let glyph_x_start = (px_start - x) as u32;
                let glyph_x_end = (px_end - x) as u32;
                let row_width = (glyph_x_end - glyph_x_start) as usize * 4;

                let canvas_row_start = ((py as u32 * canvas_width + px_start as u32) * 4) as usize;
                let glyph_row_start = ((gy * glyph_size + glyph_x_start) * 4) as usize;

                // Use SIMD blend for this row
                simd::blend_over(
                    &mut canvas[canvas_row_start..canvas_row_start + row_width],
                    &colored_glyph[glyph_row_start..glyph_row_start + row_width],
                );
            }
        }

        // Fallback to scalar blending
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            for gy in 0..glyph_size {
                for gx in 0..glyph_size {
                    let px = x + gx as i32;
                    let py = y + gy as i32;

                    // Check bounds
                    if px < 0 || py < 0 || px >= canvas_width as i32 || py >= canvas_height as i32 {
                        continue;
                    }

                    let coverage = glyph_bitmap[(gy * glyph_size + gx) as usize];
                    if coverage == 0 {
                        continue;
                    }

                    let canvas_idx = ((py as u32 * canvas_width + px as u32) * 4) as usize;

                    // Simple alpha blending
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
        _font: Arc<dyn FontRef>,
        params: &RenderParams,
    ) -> Result<RenderOutput> {
        log::debug!("OrgeRenderer: Rendering {} glyphs", shaped.glyphs.len());

        // Calculate canvas size
        let padding = params.padding as f32;
        // Ensure minimum width even for empty text
        let min_width = if shaped.glyphs.is_empty() && shaped.advance_width == 0.0 {
            1 // Minimum 1 pixel width for empty text
        } else {
            (shaped.advance_width + padding * 2.0).ceil() as u32
        };
        let width = min_width.max(1); // Always at least 1 pixel wide
                                      // For empty text, use a minimum height based on font size
        let min_height = if shaped.glyphs.is_empty() {
            16.0 // Default minimum height for empty text
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

        // Create canvas
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

        // Render each glyph
        for glyph in &shaped.glyphs {
            let glyph_size = shaped.advance_height;
            let glyph_bitmap = self.render_glyph(glyph.id, glyph_size);
            let actual_glyph_size = (glyph_size * 0.8) as u32; // Match render_glyph's size

            let x = (glyph.x + padding) as i32;
            let y = padding as i32;

            self.composite_glyph(
                &mut canvas,
                width,
                &glyph_bitmap,
                actual_glyph_size,
                x,
                y,
                params.foreground,
            );
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
