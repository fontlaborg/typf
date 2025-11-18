//! Parallel rendering support for OrgeRenderer
//!
//! Provides multi-threaded glyph rendering using Rayon for improved performance
//! on multi-core systems.

use rayon::prelude::*;
use typf_core::{
    types::{PositionedGlyph, ShapingResult},
    Color,
};

/// Parallel glyph renderer
pub struct ParallelRenderer {
    /// Number of threads to use (0 = auto)
    thread_count: usize,
}

impl ParallelRenderer {
    /// Create a new parallel renderer
    pub fn new() -> Self {
        Self {
            thread_count: 0, // Auto-detect
        }
    }

    /// Create with specific thread count
    pub fn with_threads(thread_count: usize) -> Self {
        Self { thread_count }
    }

    /// Render glyphs in parallel
    ///
    /// Splits the glyph list into chunks and renders each chunk in a separate thread.
    /// Results are then composited together.
    pub fn render_parallel(
        &self,
        glyphs: &[PositionedGlyph],
        canvas_width: u32,
        canvas_height: u32,
        glyph_renderer: impl Fn(&PositionedGlyph) -> Vec<u8> + Send + Sync,
        color: Color,
        background: Option<Color>,
    ) -> Vec<u8> {
        // Initialize canvas
        let canvas_size = (canvas_width * canvas_height * 4) as usize;
        let mut canvas = vec![0u8; canvas_size];

        // Fill background if specified
        if let Some(bg) = background {
            for pixel in canvas.chunks_exact_mut(4) {
                pixel[0] = bg.r;
                pixel[1] = bg.g;
                pixel[2] = bg.b;
                pixel[3] = bg.a;
            }
        }

        // Configure thread pool if needed
        if self.thread_count > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(self.thread_count)
                .build()
                .ok();
        }

        // Render glyphs in parallel
        let glyph_bitmaps: Vec<_> = glyphs
            .par_iter()
            .map(|glyph| {
                let bitmap = glyph_renderer(glyph);
                (glyph, bitmap)
            })
            .collect();

        // Composite results (must be sequential for correct blending)
        for (glyph, bitmap) in glyph_bitmaps {
            self.composite_glyph(
                &mut canvas,
                canvas_width,
                &bitmap,
                glyph.x as i32,
                glyph.y as i32,
                color,
            );
        }

        canvas
    }

    /// Render glyph regions in parallel
    ///
    /// Divides the canvas into regions and renders each region in parallel.
    /// This is useful for large text blocks where glyphs don't overlap.
    pub fn render_regions(
        &self,
        shaped: &ShapingResult,
        canvas_width: u32,
        canvas_height: u32,
        glyph_renderer: impl Fn(&PositionedGlyph) -> Vec<u8> + Send + Sync,
        color: Color,
    ) -> Vec<u8> {
        // Determine optimal region size
        let region_height = 64; // Typical line height
        let num_regions = (canvas_height / region_height).max(1);

        // Group glyphs by region
        let mut regions = vec![Vec::new(); num_regions as usize];
        for glyph in &shaped.glyphs {
            let region_idx = (glyph.y as u32 / region_height).min(num_regions - 1) as usize;
            regions[region_idx].push(glyph.clone());
        }

        // Render regions in parallel
        let rendered_regions: Vec<_> = regions
            .par_iter()
            .enumerate()
            .map(|(idx, glyphs)| {
                let region_y = idx as u32 * region_height;
                let mut region_canvas = vec![0u8; (canvas_width * region_height * 4) as usize];

                // Render glyphs in this region
                for glyph in glyphs {
                    let bitmap = glyph_renderer(glyph);
                    let local_y = glyph.y - region_y as f32;
                    self.composite_glyph(
                        &mut region_canvas,
                        canvas_width,
                        &bitmap,
                        glyph.x as i32,
                        local_y as i32,
                        color,
                    );
                }

                (idx, region_canvas)
            })
            .collect();

        // Combine regions
        let mut canvas = vec![0u8; (canvas_width * canvas_height * 4) as usize];
        for (idx, region_data) in rendered_regions {
            let region_y = idx as u32 * region_height;
            let start_idx = (region_y * canvas_width * 4) as usize;
            let region_size = region_data.len().min(canvas.len() - start_idx);
            canvas[start_idx..start_idx + region_size].copy_from_slice(&region_data[..region_size]);
        }

        canvas
    }

    /// Composite a glyph bitmap onto the canvas
    fn composite_glyph(
        &self,
        canvas: &mut [u8],
        canvas_width: u32,
        glyph_bitmap: &[u8],
        x: i32,
        y: i32,
        color: Color,
    ) {
        // Simple compositing - in production would use SIMD
        let glyph_size = (glyph_bitmap.len() as f32).sqrt() as u32;
        let canvas_height = canvas.len() as u32 / (canvas_width * 4);

        for gy in 0..glyph_size {
            for gx in 0..glyph_size {
                let px = x + gx as i32;
                let py = y + gy as i32;

                if px < 0 || py < 0 || px >= canvas_width as i32 || py >= canvas_height as i32 {
                    continue;
                }

                let coverage = glyph_bitmap[(gy * glyph_size + gx) as usize];
                if coverage == 0 {
                    continue;
                }

                let canvas_idx = ((py as u32 * canvas_width + px as u32) * 4) as usize;

                // Alpha blending
                let alpha = (coverage as f32 / 255.0) * (color.a as f32 / 255.0);
                let inv_alpha = 1.0 - alpha;

                canvas[canvas_idx] =
                    (canvas[canvas_idx] as f32 * inv_alpha + color.r as f32 * alpha) as u8;
                canvas[canvas_idx + 1] =
                    (canvas[canvas_idx + 1] as f32 * inv_alpha + color.g as f32 * alpha) as u8;
                canvas[canvas_idx + 2] =
                    (canvas[canvas_idx + 2] as f32 * inv_alpha + color.b as f32 * alpha) as u8;
                canvas[canvas_idx + 3] =
                    ((canvas[canvas_idx + 3] as f32 * inv_alpha + 255.0 * alpha).min(255.0)) as u8;
            }
        }
    }
}

impl Default for ParallelRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Parallel rendering statistics
#[derive(Debug, Clone, Default)]
pub struct ParallelStats {
    pub total_glyphs: usize,
    pub threads_used: usize,
    pub render_time_ms: u128,
    pub composite_time_ms: u128,
}

impl ParallelStats {
    /// Calculate speedup vs single-threaded
    pub fn speedup(&self) -> f32 {
        if self.threads_used <= 1 {
            1.0
        } else {
            let total_time = self.render_time_ms + self.composite_time_ms;
            let estimated_sequential =
                self.total_glyphs as u128 * total_time / self.threads_used as u128;
            estimated_sequential as f32 / total_time as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typf_core::types::Direction;

    #[test]
    fn test_parallel_renderer() {
        let renderer = ParallelRenderer::new();

        let glyphs = vec![
            PositionedGlyph {
                id: 42,
                x: 0.0,
                y: 0.0,
                advance: 10.0,
                cluster: 0,
            },
            PositionedGlyph {
                id: 43,
                x: 10.0,
                y: 0.0,
                advance: 10.0,
                cluster: 1,
            },
        ];

        // Mock glyph renderer
        let glyph_renderer = |_glyph: &PositionedGlyph| {
            vec![255u8; 100] // 10x10 glyph
        };

        let canvas = renderer.render_parallel(
            &glyphs,
            100,
            20,
            glyph_renderer,
            Color::rgba(0, 0, 0, 255),
            None,
        );

        assert_eq!(canvas.len(), 100 * 20 * 4);
    }

    #[test]
    fn test_region_rendering() {
        let renderer = ParallelRenderer::new();

        let shaped = ShapingResult {
            glyphs: (0..10)
                .map(|i| PositionedGlyph {
                    id: 42,
                    x: (i * 10) as f32,
                    y: (i / 5 * 20) as f32, // Two rows
                    advance: 10.0,
                    cluster: i as u32,
                })
                .collect(),
            advance_width: 100.0,
            advance_height: 40.0,
            direction: Direction::LeftToRight,
        };

        let glyph_renderer = |_glyph: &PositionedGlyph| {
            vec![128u8; 100] // 10x10 glyph
        };

        let canvas =
            renderer.render_regions(&shaped, 100, 40, glyph_renderer, Color::rgba(0, 0, 0, 255));

        assert_eq!(canvas.len(), 100 * 40 * 4);
    }

    #[test]
    fn test_parallel_stats() {
        let stats = ParallelStats {
            total_glyphs: 100,
            threads_used: 4,
            render_time_ms: 10,
            composite_time_ms: 5,
        };

        let speedup = stats.speedup();
        assert!(speedup > 1.0); // Should show speedup with 4 threads
    }
}
