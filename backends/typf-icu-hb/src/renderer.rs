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

/// orge-based renderer (ultra-smooth unhinted scan converter)
#[cfg(feature = "orge")]
pub struct OrgeRenderer;

#[cfg(feature = "orge")]
impl Default for OrgeRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl OrgeRenderer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "orge")]
impl GlyphRenderer for OrgeRenderer {
    fn render_glyph(
        &self,
        path: &BezPath,
        width: u32,
        height: u32,
        antialias: bool,
    ) -> Option<RenderedGlyph> {
        use typf_orge::fixed::F26Dot6;
        use typf_orge::grayscale::{render_grayscale_direct, GrayscaleLevel};
        use typf_orge::scan_converter::ScanConverter;

        let width = width as usize;
        let height = height as usize;

        if antialias {
            // Grayscale rendering with 4x4 oversampling
            let bitmap = render_grayscale_direct(width, height, GrayscaleLevel::Level4x4, |sc| {
                // Build outline from BezPath
                use kurbo::PathEl;
                for element in path.elements() {
                    match *element {
                        PathEl::MoveTo(p) => sc.move_to(
                            F26Dot6::from_float(p.x as f32),
                            F26Dot6::from_float(p.y as f32),
                        ),
                        PathEl::LineTo(p) => sc.line_to(
                            F26Dot6::from_float(p.x as f32),
                            F26Dot6::from_float(p.y as f32),
                        ),
                        PathEl::QuadTo(ctrl, end) => sc.quadratic_to(
                            F26Dot6::from_float(ctrl.x as f32),
                            F26Dot6::from_float(ctrl.y as f32),
                            F26Dot6::from_float(end.x as f32),
                            F26Dot6::from_float(end.y as f32),
                        ),
                        PathEl::CurveTo(c1, c2, end) => sc.cubic_to(
                            F26Dot6::from_float(c1.x as f32),
                            F26Dot6::from_float(c1.y as f32),
                            F26Dot6::from_float(c2.x as f32),
                            F26Dot6::from_float(c2.y as f32),
                            F26Dot6::from_float(end.x as f32),
                            F26Dot6::from_float(end.y as f32),
                        ),
                        PathEl::ClosePath => sc.close(),
                    }
                }
            });

            Some(RenderedGlyph {
                bitmap,
                width: width as u32,
                height: height as u32,
                left: 0.0,
                top: 0.0,
            })
        } else {
            // Monochrome rendering
            let mut sc = ScanConverter::new(width, height);

            // Build outline from BezPath
            use kurbo::PathEl;
            for element in path.elements() {
                match *element {
                    PathEl::MoveTo(p) => sc.move_to(
                        F26Dot6::from_float(p.x as f32),
                        F26Dot6::from_float(p.y as f32),
                    ),
                    PathEl::LineTo(p) => sc.line_to(
                        F26Dot6::from_float(p.x as f32),
                        F26Dot6::from_float(p.y as f32),
                    ),
                    PathEl::QuadTo(ctrl, end) => sc.quadratic_to(
                        F26Dot6::from_float(ctrl.x as f32),
                        F26Dot6::from_float(ctrl.y as f32),
                        F26Dot6::from_float(end.x as f32),
                        F26Dot6::from_float(end.y as f32),
                    ),
                    PathEl::CurveTo(c1, c2, end) => sc.cubic_to(
                        F26Dot6::from_float(c1.x as f32),
                        F26Dot6::from_float(c1.y as f32),
                        F26Dot6::from_float(c2.x as f32),
                        F26Dot6::from_float(c2.y as f32),
                        F26Dot6::from_float(end.x as f32),
                        F26Dot6::from_float(end.y as f32),
                    ),
                    PathEl::ClosePath => sc.close(),
                }
            }

            let mut bitmap = vec![0u8; width * height];
            sc.render_mono(&mut bitmap);

            // Convert 0/1 to 0/255 for consistency
            for byte in &mut bitmap {
                *byte = if *byte > 0 { 255 } else { 0 };
            }

            Some(RenderedGlyph {
                bitmap,
                width: width as u32,
                height: height as u32,
                left: 0.0,
                top: 0.0,
            })
        }
    }
}

/// Create the appropriate renderer based on enabled features
pub fn create_renderer() -> Box<dyn GlyphRenderer> {
    #[cfg(feature = "orge")]
    {
        Box::new(OrgeRenderer::new())
    }
    #[cfg(not(feature = "orge"))]
    {
        #[cfg(feature = "tiny-skia-renderer")]
        {
            Box::new(TinySkiaRenderer::new())
        }
        #[cfg(not(feature = "tiny-skia-renderer"))]
        {
            compile_error!(
                "At least one renderer feature must be enabled: orge or tiny-skia-renderer"
            )
        }
    }
}
