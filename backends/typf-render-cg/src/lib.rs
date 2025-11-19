//! CoreGraphics Renderer - macOS native bitmap rendering backend
//!
//! This backend uses CoreGraphics for high-quality bitmap rendering on macOS.
//! It supports antialiasing, custom colors, and proper baseline positioning.

#![cfg(target_os = "macos")]

use std::sync::Arc;
use typf_core::{
    error::{RenderError, Result, TypfError},
    traits::{FontRef, Renderer},
    types::{BitmapData, BitmapFormat, RenderOutput, ShapingResult},
    Color, RenderParams,
};

use core_graphics::{
    color_space::CGColorSpace,
    context::{CGContext, CGTextDrawingMode},
    data_provider::CGDataProvider,
    font::{CGFont, CGGlyph},
    geometry::{CGPoint, CGRect, CGSize},
    sys::CGContextRef,
};

/// Wrapper for font data to pass to CGDataProvider
struct ProviderData {
    bytes: Arc<[u8]>,
}

impl AsRef<[u8]> for ProviderData {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

/// CoreGraphics renderer with bitmap output
pub struct CoreGraphicsRenderer;

impl CoreGraphicsRenderer {
    /// Create a new CoreGraphics renderer
    pub fn new() -> Self {
        Self
    }

    /// Create CGFont from raw font data
    fn create_cg_font(data: &[u8]) -> Result<CGFont> {
        // Create Arc from font data
        let provider_data = Arc::new(ProviderData {
            bytes: Arc::from(data),
        });

        // Create CGDataProvider
        let provider = CGDataProvider::from_buffer(provider_data);

        // Create CGFont from data
        let cg_font = CGFont::from_data_provider(provider).map_err(|_| {
            TypfError::RenderingFailed(RenderError::BackendError(
                "Failed to create CGFont from data".to_string(),
            ))
        })?;

        Ok(cg_font)
    }

    /// Calculate dimensions for the rendered bitmap
    fn calculate_dimensions(shaped: &ShapingResult, params: &RenderParams) -> (u32, u32, f32) {
        // Calculate content dimensions from glyphs
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for glyph in &shaped.glyphs {
            min_x = min_x.min(glyph.x);
            max_x = max_x.max(glyph.x + glyph.advance);
            // Approximate glyph height based on font size (ascent + descent)
            min_y = min_y.min(glyph.y - shaped.advance_height * 0.8);
            max_y = max_y.max(glyph.y + shaped.advance_height * 0.2);
        }

        // Fall back to reasonable defaults if no glyphs
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

        // Calculate baseline offset (75% from top, matching old implementation)
        let baseline_y = height as f32 * 0.75;

        (width, height, baseline_y)
    }

    /// Convert Color to RGB components
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
        let (width, height, baseline_y) = Self::calculate_dimensions(shaped, params);

        log::debug!(
            "CoreGraphicsRenderer: Canvas size {}x{}, baseline_y={}",
            width,
            height,
            baseline_y
        );

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

        // Create CGFont
        let cg_font = Self::create_cg_font(font.data())?;

        // Set font and size
        context.set_font(&cg_font);
        context.set_font_size(shaped.advance_height as f64);

        // Set text drawing mode to fill
        context.set_text_drawing_mode(CGTextDrawingMode::CGTextFill);

        // Prepare glyph data
        let glyph_ids: Vec<CGGlyph> = shaped
            .glyphs
            .iter()
            .map(|g| g.id.min(u16::MAX as u32) as CGGlyph)
            .collect();

        log::debug!(
            "CoreGraphicsRenderer: Rendering {} glyphs, font_size={}, baseline_y={}",
            glyph_ids.len(),
            shaped.advance_height,
            baseline_y
        );
        if !glyph_ids.is_empty() {
            log::debug!(
                "CoreGraphicsRenderer: First glyph: id={}, x={}, y={}",
                glyph_ids[0],
                shaped.glyphs[0].x,
                shaped.glyphs[0].y
            );
        }

        // Calculate glyph positions
        // After translate(0, height) + scale(1, -1): Y=0 is at BOTTOM, Y increases upward
        // baseline_y is measured from top (75% * height), so in flipped coords it's at: height - baseline_y
        let glyph_positions: Vec<CGPoint> = shaped
            .glyphs
            .iter()
            .map(|g| CGPoint {
                x: (g.x + params.padding as f32) as f64,
                // In flipped coords: baseline is at (height - baseline_y), then add glyph offset
                y: (height as f32 - baseline_y + g.y) as f64,
            })
            .collect();

        if !glyph_positions.is_empty() {
            log::debug!(
                "CoreGraphicsRenderer: First glyph position: x={}, y={}",
                glyph_positions[0].x,
                glyph_positions[0].y
            );
        }

        // Render glyphs
        context.save();

        // CoreGraphics uses bottom-left origin, so we need to flip the coordinate system
        context.translate(0.0, height as f64);
        context.scale(1.0, -1.0);

        // Draw glyphs using CGContext
        if !glyph_ids.is_empty() {
            let context_ref: CGContextRef = &context as *const _ as *mut _;
            unsafe {
                CGContextShowGlyphsAtPositions(
                    context_ref,
                    glyph_ids.as_ptr(),
                    glyph_positions.as_ptr(),
                    glyph_ids.len(),
                );
            }
        }

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

// FFI declaration for CGContextShowGlyphsAtPositions
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGContextShowGlyphsAtPositions(
        c: CGContextRef,
        glyphs: *const CGGlyph,
        positions: *const CGPoint,
        count: usize,
    );
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
