//! CoreGraphics Renderer - Apple's own text rendering muscle, now in TYPF
//!
//! When you're on macOS, why settle for less? This renderer taps directly
//! into CoreGraphics, the same engine that powers macOS's text rendering.
//! Perfect antialiasing, native performance, and results that match exactly
//! what users see in their native apps.

#![cfg(target_os = "macos")]

use std::sync::Arc;
use typf_core::{
    error::{RenderError, Result, TypfError},
    traits::{FontRef, Renderer},
    types::{BitmapData, BitmapFormat, RenderOutput, ShapingResult},
    Color, RenderParams,
};

use core_foundation::base::{TCFType, TCFTypeRef};
use core_graphics::{
    color_space::CGColorSpace,
    context::{CGContext, CGTextDrawingMode},
    data_provider::CGDataProvider,
    font::{CGFont, CGGlyph},
    geometry::{CGPoint, CGRect, CGSize},
};

/// Bridge between our font bytes and CoreGraphics' data expectations
///
/// CoreGraphics needs data that lives as long as the font object does.
/// This wrapper keeps our font data alive with proper reference counting.
struct ProviderData {
    bytes: Arc<[u8]>,
}

impl AsRef<[u8]> for ProviderData {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

/// Direct access to macOS's professional text rendering pipeline
///
/// This isn't just another renderer—it's a first-class citizen on macOS,
/// using the exact same APIs that Safari, Pages, and the system text renderer
/// use. When you need pixel-perfect consistency with the platform, this is it.
pub struct CoreGraphicsRenderer;

impl CoreGraphicsRenderer {
    /// Creates a renderer ready to harness macOS's text rendering power
    pub fn new() -> Self {
        Self
    }

    /// Turns raw font bytes into a CoreGraphics-ready font object
    ///
    /// This is where we bridge TYPF's font loading with CoreGraphics' expectations.
    /// The data provider pattern ensures the font data stays alive as long as
    /// CoreGraphics needs it.
    fn create_cg_font(data: &[u8]) -> Result<CGFont> {
        // Wrap our font data in an Arc for proper lifetime management
        let provider_data = Arc::new(ProviderData {
            bytes: Arc::from(data),
        });

        // Hand the data to CoreGraphics through its provider interface
        let provider = CGDataProvider::from_buffer(provider_data);

        // Let CoreGraphics parse and validate the font data
        let cg_font = CGFont::from_data_provider(provider).map_err(|_| {
            TypfError::RenderingFailed(RenderError::BackendError(
                "CoreGraphics rejected our font data".to_string(),
            ))
        })?;

        Ok(cg_font)
    }

    /// Figures out how much canvas space our shaped text needs
    ///
    /// CoreGraphics needs explicit dimensions, so we calculate the bounding box
    /// that contains all our positioned glyphs plus any requested padding.
    fn calculate_dimensions(shaped: &ShapingResult, params: &RenderParams) -> (u32, u32) {
        // Track the extremes of our glyph layout
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for glyph in &shaped.glyphs {
            min_x = min_x.min(glyph.x);
            max_x = max_x.max(glyph.x + glyph.advance);
            // Estimate vertical bounds using font proportions (80% ascent, 20% descent)
            min_y = min_y.min(glyph.y - shaped.advance_height * 0.8);
            max_y = max_y.max(glyph.y + shaped.advance_height * 0.2);
        }

        // Don't crash on empty text—give it a minimal reasonable size
        if shaped.glyphs.is_empty() {
            min_x = 0.0;
            max_x = 1.0;
            min_y = 0.0;
            max_y = 1.0;
        }

        let content_width = (max_x - min_x).max(1.0);
        let content_height = (max_y - min_y).max(1.0);

        let padding = params.padding as f32;
        let width = (content_width + padding * 2.0).ceil() as u32;
        let height = (content_height + padding * 2.0).ceil() as u32;

        (width, height)
    }

    /// Convert TYPF's Color type to CoreGraphics' normalized float format
    fn color_to_rgb(color: &Color) -> (f64, f64, f64, f64) {
        (
            color.r as f64 / 255.0,
            color.g as f64 / 255.0,
            color.b as f64 / 255.0,
            color.a as f64 / 255.0,
        )
    }
}

impl Default for CoreGraphicsRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for CoreGraphicsRenderer {
    fn name(&self) -> &'static str {
        "coregraphics"
    }

    fn render(
        &self,
        shaped: &ShapingResult,
        font: Arc<dyn FontRef>,
        params: &RenderParams,
    ) -> Result<RenderOutput> {
        log::debug!("CoreGraphicsRenderer: Rendering {} glyphs", shaped.glyphs.len());

        // Handle empty glyph case
        if shaped.glyphs.is_empty() {
            log::debug!("CoreGraphicsRenderer: No glyphs to render");
            return Ok(RenderOutput::Bitmap(BitmapData {
                width: 1,
                height: 1,
                format: BitmapFormat::Rgba8,
                data: vec![0, 0, 0, 255], // Transparent pixel
            }));
        }

        // Calculate dimensions
        let (width, height) = Self::calculate_dimensions(shaped, params);

        log::debug!("CoreGraphicsRenderer: Canvas size {}x{}", width, height);

        // Create bitmap buffer (RGBA, premultiplied alpha)
        let bytes_per_row = width as usize * 4;
        let mut buffer = vec![0u8; height as usize * bytes_per_row];

        // Create CGContext
        let color_space = CGColorSpace::create_device_rgb();
        let context = CGContext::create_bitmap_context(
            Some(buffer.as_mut_ptr() as *mut _),
            width as usize,
            height as usize,
            8, // bits per component
            bytes_per_row,
            &color_space,
            core_graphics::base::kCGImageAlphaPremultipliedLast,
        );

        // Configure antialiasing
        context.set_should_antialias(params.antialias);
        if params.antialias {
            context.set_should_smooth_fonts(true);
        } else {
            context.set_should_smooth_fonts(false);
        }

        // Fill background (default to transparent if not specified)
        if let Some(bg_color) = &params.background {
            let (r, g, b, a) = Self::color_to_rgb(bg_color);
            context.set_rgb_fill_color(r, g, b, a);
            context.fill_rect(CGRect::new(
                &CGPoint::new(0.0, 0.0),
                &CGSize::new(width as f64, height as f64),
            ));
        } else {
            // Clear to transparent (RGBA all zeros)
            context.clear_rect(CGRect::new(
                &CGPoint::new(0.0, 0.0),
                &CGSize::new(width as f64, height as f64),
            ));
        }

        // Set text color
        let (r, g, b, a) = Self::color_to_rgb(&params.foreground);
        context.set_rgb_fill_color(r, g, b, a);

        // Validate font size before creating CTFont
        // CoreText requires a positive, finite font size
        let font_size = shaped.advance_height as f64;
        if !font_size.is_finite() || font_size <= 0.0 {
            return Err(TypfError::RenderingFailed(RenderError::BackendError(
                format!("Invalid font size: {}. Font size must be positive and finite.", font_size),
            )));
        }

        // Create CGFont and CTFont
        let cg_font = Self::create_cg_font(font.data())?;
        let ct_font = core_text::font::new_from_CGFont(&cg_font, font_size);

        // Verify CTFont creation succeeded
        // CoreText might return a CTFont with NULL internal pointer on failure
        // which would later crash when CFRelease is called during drop
        if ct_font.as_concrete_TypeRef().as_void_ptr().is_null() {
            return Err(TypfError::RenderingFailed(RenderError::BackendError(
                "CTFont creation failed: CoreText returned NULL font object".to_string(),
            )));
        }

        // Prepare glyph data
        let glyph_ids: Vec<CGGlyph> = shaped
            .glyphs
            .iter()
            .map(|g| g.id.min(u16::MAX as u32) as CGGlyph)
            .collect();

        log::debug!(
            "CoreGraphicsRenderer: Rendering {} glyphs, font_size={}",
            glyph_ids.len(),
            shaped.advance_height
        );
        if !glyph_ids.is_empty() {
            log::debug!(
                "CoreGraphicsRenderer: First glyph: id={}, x={}, y={}",
                glyph_ids[0],
                shaped.glyphs[0].x,
                shaped.glyphs[0].y
            );
        }

        // Calculate glyph positions relative to origin (after translate)
        // CoreGraphics uses bottom-left origin. Calculate baseline position.
        // Use 0.75 ratio: baseline at 75% from top = 25% from bottom
        const BASELINE_RATIO: f64 = 0.75;
        let baseline_y = (height as f64) * (1.0 - BASELINE_RATIO);

        let glyph_positions: Vec<CGPoint> = shaped
            .glyphs
            .iter()
            .map(|g| CGPoint {
                x: g.x as f64,
                y: g.y as f64,
            })
            .collect();

        if !glyph_positions.is_empty() {
            log::debug!(
                "CoreGraphicsRenderer: First glyph position: x={}, y={}, baseline_y={}",
                glyph_positions[0].x,
                glyph_positions[0].y,
                baseline_y
            );
        }

        // Render glyphs using CTFont
        context.save();
        context.translate(params.padding as f64, baseline_y);
        context.set_text_drawing_mode(CGTextDrawingMode::CGTextFill);
        ct_font.draw_glyphs(&glyph_ids, &glyph_positions, context.clone());
        context.restore();

        // Return bitmap data
        Ok(RenderOutput::Bitmap(BitmapData {
            width,
            height,
            format: BitmapFormat::Rgba8,
            data: buffer,
        }))
    }

    fn supports_format(&self, format: &str) -> bool {
        matches!(format, "bitmap" | "rgba")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typf_core::types::Direction;

    // Mock font for testing
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

        fn glyph_id(&self, ch: char) -> Option<u32> {
            if ch.is_ascii() {
                Some(ch as u32)
            } else {
                None
            }
        }

        fn advance_width(&self, _glyph_id: u32) -> f32 {
            500.0
        }
    }

    #[test]
    fn test_renderer_creation() {
        let renderer = CoreGraphicsRenderer::new();
        assert_eq!(renderer.name(), "coregraphics");
    }

    #[test]
    fn test_supports_format() {
        let renderer = CoreGraphicsRenderer::new();
        assert!(renderer.supports_format("bitmap"));
        assert!(renderer.supports_format("rgba"));
        assert!(!renderer.supports_format("svg"));
    }

    #[test]
    fn test_empty_glyphs() {
        let renderer = CoreGraphicsRenderer::new();
        let font = Arc::new(MockFont { data: vec![] });
        let shaped = ShapingResult {
            glyphs: vec![],
            advance_width: 0.0,
            advance_height: 16.0,
            direction: Direction::LeftToRight,
        };
        let params = RenderParams::default();

        let result = renderer.render(&shaped, font, &params);
        assert!(result.is_ok());

        if let Ok(RenderOutput::Bitmap(bitmap)) = result {
            assert_eq!(bitmap.width, 1);
            assert_eq!(bitmap.height, 1);
            assert_eq!(bitmap.format, BitmapFormat::Rgba8);
        }
    }
}
