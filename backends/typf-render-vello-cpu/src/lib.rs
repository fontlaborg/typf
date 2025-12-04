//! Vello CPU Renderer: High-quality 2D rendering without GPU
//!
//! This backend integrates [Vello CPU](https://github.com/linebender/vello),
//! a modern 2D graphics rendering engine, to provide high-quality text rendering
//! with excellent glyph caching and hinting support.
//!
//! ## Features
//!
//! - Pure Rust implementation (no GPU required)
//! - Native glyph caching for repeated text
//! - Font hinting support for crisp small text
//! - Color font support (outline, bitmap, COLR)
//! - SIMD optimizations on x86_64 and aarch64
//!
//! ## Usage
//!
//! ```ignore
//! use typf_render_vello_cpu::VelloCpuRenderer;
//! use typf_core::traits::Renderer;
//!
//! let renderer = VelloCpuRenderer::new();
//! let result = renderer.render(&shaped, font, &params)?;
//! ```

use std::sync::Arc;

use skrifa::MetadataProvider;
use thiserror::Error;
use typf_core::{
    error::{RenderError, Result},
    traits::{FontRef, Renderer},
    types::{BitmapData, BitmapFormat, RenderOutput, ShapingResult},
    Color, RenderParams,
};
use vello_common::glyph::Glyph as VelloGlyph;
use vello_common::peniko::FontData;
use vello_cpu::{
    color::AlphaColor,
    kurbo::{Affine, Rect},
    Pixmap, RenderContext, RenderMode,
};

/// Errors specific to the Vello CPU renderer
#[derive(Debug, Error)]
pub enum VelloCpuError {
    #[error("Failed to create font data: {0}")]
    FontDataError(String),

    #[error("Rendering failed: {0}")]
    RenderingFailed(String),
}

impl From<VelloCpuError> for RenderError {
    fn from(e: VelloCpuError) -> Self {
        RenderError::BackendError(e.to_string())
    }
}

/// Configuration for the Vello CPU renderer
#[derive(Debug, Clone)]
pub struct VelloCpuConfig {
    /// Enable font hinting for crisp small text
    pub hinting: bool,
    /// Render mode (speed vs quality)
    pub render_mode: RenderMode,
}

impl Default for VelloCpuConfig {
    fn default() -> Self {
        Self {
            hinting: true,
            render_mode: RenderMode::OptimizeSpeed,
        }
    }
}

/// High-quality CPU renderer powered by Vello
///
/// This renderer provides excellent text quality with efficient glyph caching,
/// making it suitable for applications that don't have GPU access or prefer
/// CPU-based rendering.
pub struct VelloCpuRenderer {
    config: VelloCpuConfig,
}

impl VelloCpuRenderer {
    /// Create a new Vello CPU renderer with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: VelloCpuConfig::default(),
        }
    }

    /// Create a renderer with custom configuration
    #[must_use]
    pub fn with_config(config: VelloCpuConfig) -> Self {
        Self { config }
    }

    /// Convert typf Color to Vello AlphaColor (RGBA8)
    fn to_vello_color(color: Color) -> AlphaColor<vello_cpu::color::Srgb> {
        AlphaColor::from_rgba8(color.r, color.g, color.b, color.a)
    }

    /// Convert ShapingResult glyphs to Vello Glyph format
    fn convert_glyphs(shaped: &ShapingResult) -> Vec<VelloGlyph> {
        shaped
            .glyphs
            .iter()
            .map(|g| VelloGlyph {
                id: g.id,
                x: g.x,
                y: g.y,
            })
            .collect()
    }
}

/// Vello's normalized coordinate type (plain i16, 2.14 fixed-point format)
type VelloNormalizedCoord = i16;

/// Build normalized variation coordinates from user-specified variations.
///
/// Converts variation settings like `[("wght", 700.0), ("wdth", 100.0)]` into
/// normalized coordinates suitable for Vello's glyph rendering.
///
/// Note: Vello uses raw i16 values in 2.14 fixed-point format, while skrifa
/// uses the F2Dot14 wrapper type. We convert by extracting the raw bits.
fn build_normalized_coords(
    font_data: &[u8],
    variations: &[(String, f32)],
) -> Vec<VelloNormalizedCoord> {
    if variations.is_empty() {
        return Vec::new();
    }

    let font_ref = match skrifa::FontRef::new(font_data) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let axes = font_ref.axes();
    let settings: Vec<(&str, f32)> = variations
        .iter()
        .map(|(tag, value)| (tag.as_str(), *value))
        .collect();

    let location = axes.location(settings);
    // Convert F2Dot14 to raw i16 values for Vello
    location.coords().iter().map(|c| c.to_bits()).collect()
}

impl Default for VelloCpuRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for VelloCpuRenderer {
    fn name(&self) -> &'static str {
        "vello-cpu"
    }

    fn render(
        &self,
        shaped: &ShapingResult,
        font: Arc<dyn FontRef>,
        params: &RenderParams,
    ) -> Result<RenderOutput> {
        log::debug!(
            "VelloCpuRenderer: Rendering {} glyphs at size {}",
            shaped.glyphs.len(),
            shaped.advance_height
        );

        let padding = params.padding as f32;
        let font_size = shaped.advance_height;

        // Get actual font metrics for proper baseline and height calculation
        let font_bytes = font.data();
        let (ascent, descent) = if let Ok(font_ref) = skrifa::FontRef::new(font_bytes) {
            let size = skrifa::instance::Size::new(font_size);
            let location = skrifa::instance::LocationRef::default();
            let metrics = font_ref.metrics(size, location);
            // ascent is positive (above baseline), descent is negative (below baseline)
            (metrics.ascent, metrics.descent.abs())
        } else {
            // Fallback to approximate values if font parsing fails
            (font_size * 0.8, font_size * 0.2)
        };

        // Calculate canvas dimensions using actual font metrics
        let width = (shaped.advance_width + padding * 2.0).ceil() as u32;
        // Height covers full ascent + descent + padding on both sides
        let height = (ascent + descent + padding * 2.0).ceil() as u32;

        // Sanity check dimensions
        if width == 0 || height == 0 {
            return Err(RenderError::ZeroDimensions { width, height }.into());
        }

        // Create font data from raw bytes
        // FontData requires Vec<u8> (not &[u8]) and a font collection index
        let font_bytes = font.data().to_vec();
        let font_data = FontData::new(font_bytes.clone().into(), 0);

        // Build normalized variation coordinates for variable fonts
        let normalized_coords = build_normalized_coords(&font_bytes, &params.variations);

        // Create render context
        let mut context = RenderContext::new(width as u16, height as u16);

        // Fill background if specified
        if let Some(bg) = params.background {
            context.set_paint(Self::to_vello_color(bg));
            context.fill_rect(&Rect::new(0.0, 0.0, width as f64, height as f64));
        }

        // Set foreground color
        context.set_paint(Self::to_vello_color(params.foreground));

        // Calculate baseline position using actual font ascent
        // Baseline is at padding + ascent (top of canvas + space for ascenders)
        let baseline_y = padding + ascent;

        // Set transform for glyph positioning
        context.set_transform(Affine::translate((padding as f64, baseline_y as f64)));

        // Convert glyphs
        let glyphs = Self::convert_glyphs(shaped);

        // Build and render glyph run using RenderContext's built-in glyph support
        let mut glyph_run = context
            .glyph_run(&font_data)
            .font_size(font_size)
            .hint(self.config.hinting);

        // Apply variable font coordinates if specified
        if !normalized_coords.is_empty() {
            glyph_run = glyph_run.normalized_coords(&normalized_coords);
        }

        glyph_run.fill_glyphs(glyphs.into_iter());

        // Flush and render to pixmap
        context.flush();
        let mut pixmap = Pixmap::new(width as u16, height as u16);
        context.render_to_pixmap(&mut pixmap);

        // Convert pixmap to RGBA8 bitmap data
        let rgba_data: Vec<u8> = pixmap
            .data()
            .iter()
            .flat_map(|pixel| [pixel.r, pixel.g, pixel.b, pixel.a])
            .collect();

        Ok(RenderOutput::Bitmap(BitmapData {
            width,
            height,
            format: BitmapFormat::Rgba8,
            data: rgba_data,
        }))
    }

    fn supports_format(&self, format: &str) -> bool {
        matches!(format, "bitmap" | "rgba" | "rgb")
    }

    fn clear_cache(&self) {
        // RenderContext manages its own glyph caches internally
        // For stateless rendering (new context per call), this is a no-op
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typf_core::types::{Direction, PositionedGlyph};

    struct MockFont {
        data: Vec<u8>,
    }

    impl FontRef for MockFont {
        fn data(&self) -> &[u8] {
            &self.data
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

    #[test]
    fn test_renderer_creation() {
        let renderer = VelloCpuRenderer::new();
        assert_eq!(renderer.name(), "vello-cpu");
    }

    #[test]
    fn test_config_default() {
        let config = VelloCpuConfig::default();
        assert!(config.hinting);
    }

    #[test]
    fn test_glyph_conversion() {
        let shaped = ShapingResult {
            glyphs: vec![
                PositionedGlyph {
                    id: 72,
                    x: 0.0,
                    y: 0.0,
                    advance: 10.0,
                    cluster: 0,
                },
                PositionedGlyph {
                    id: 105,
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

        let glyphs = VelloCpuRenderer::convert_glyphs(&shaped);
        assert_eq!(glyphs.len(), 2);
        assert_eq!(glyphs[0].id, 72);
        assert_eq!(glyphs[1].id, 105);
    }

    #[test]
    fn test_color_conversion() {
        let color = Color::rgba(255, 128, 64, 200);
        let vello_color = VelloCpuRenderer::to_vello_color(color);
        // Alpha is stored in components[3], should be approximately 200/255 ≈ 0.784
        let alpha = vello_color.components[3];
        assert!(alpha > 0.7 && alpha < 0.8, "alpha was {alpha}");
    }
}
