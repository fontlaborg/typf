// this_file: backends/typf-icu-hb/src/renderer.rs

//! Glyph rendering trait and implementations

use kurbo::BezPath;
use typf_core::cache::RenderedGlyph;

/// Trait for glyph rendering backends
pub trait GlyphRenderer {
    /// Render a glyph from a BezPath outline
    fn render_glyph(
        &self,
        path: &BezPath,
        width: u32,
        height: u32,
        antialias: bool,
    ) -> Option<RenderedGlyph>;
}

/// TinySkia-based renderer
#[cfg(feature = "tiny-skia-renderer")]
pub struct TinySkiaRenderer;

#[cfg(feature = "tiny-skia-renderer")]
impl Default for TinySkiaRenderer {
    fn default() -> Self {
        Self
    }
}

#[cfg(feature = "tiny-skia-renderer")]
impl TinySkiaRenderer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "tiny-skia-renderer")]
impl GlyphRenderer for TinySkiaRenderer {
    fn render_glyph(
        &self,
        path: &BezPath,
        width: u32,
        height: u32,
        antialias: bool,
    ) -> Option<RenderedGlyph> {
        use kurbo::PathEl;
        use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

        // Convert kurbo BezPath to tiny-skia Path
        let mut builder = PathBuilder::new();
        for element in path.elements() {
            match *element {
                PathEl::MoveTo(p) => builder.move_to(p.x as f32, p.y as f32),
                PathEl::LineTo(p) => builder.line_to(p.x as f32, p.y as f32),
                PathEl::QuadTo(ctrl, end) => {
                    builder.quad_to(ctrl.x as f32, ctrl.y as f32, end.x as f32, end.y as f32)
                }
                PathEl::CurveTo(c1, c2, end) => builder.cubic_to(
                    c1.x as f32,
                    c1.y as f32,
                    c2.x as f32,
                    c2.y as f32,
                    end.x as f32,
                    end.y as f32,
                ),
                PathEl::ClosePath => builder.close(),
            }
        }
        let skia_path = builder.finish()?;

        // Create pixmap
        let mut pixmap = Pixmap::new(width, height)?;

        // Fill path
        let paint = Paint {
            anti_alias: antialias,
            ..Default::default()
        };
        pixmap.fill_path(
            &skia_path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );

        // Extract alpha channel (tiny-skia uses RGBA, we want grayscale alpha)
        let data = pixmap.data();
        let mut alpha = vec![0u8; (width * height) as usize];
        for i in 0..(width * height) as usize {
            alpha[i] = data[i * 4 + 3]; // Extract alpha channel
        }

        Some(RenderedGlyph {
            bitmap: alpha,
            width,
            height,
            left: 0.0,
            top: 0.0,
        })
    }
}

/// Create the appropriate renderer - skiahb always uses TinySkia
pub fn create_renderer() -> Box<dyn GlyphRenderer> {
    // skiahb backend ALWAYS uses TinySkia for rasterization
    #[cfg(feature = "tiny-skia-renderer")]
    {
        Box::new(TinySkiaRenderer::new())
    }
    #[cfg(not(feature = "tiny-skia-renderer"))]
    {
        compile_error!(
            "tiny-skia-renderer feature must be enabled for skiahb backend"
        )
    }
}
