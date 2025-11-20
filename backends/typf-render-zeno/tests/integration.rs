//! Integration tests for Zeno renderer
//!
//! Community project by FontLab - https://www.fontlab.org/

use std::sync::Arc;
use typf_core::{
    traits::{FontRef, Renderer},
    types::{BitmapFormat, Direction, GlyphId, RenderOutput, ShapingResult},
    RenderParams,
};
use typf_render_zeno::ZenoRenderer;

/// Stub font for testing
struct StubFont {
    data: Vec<u8>,
}

impl FontRef for StubFont {
    fn data(&self) -> &[u8] {
        &self.data
    }

    fn units_per_em(&self) -> u16 {
        1000
    }

    fn glyph_id(&self, _ch: char) -> Option<GlyphId> {
        Some(0)
    }

    fn advance_width(&self, _glyph_id: GlyphId) -> f32 {
        500.0
    }
}

#[test]
fn test_renderer_creation() {
    let renderer = ZenoRenderer::new();
    assert_eq!(renderer.name(), "zeno");
}

#[test]
fn test_renderer_default() {
    let renderer = ZenoRenderer::default();
    assert_eq!(renderer.name(), "zeno");
}

#[test]
fn test_empty_rendering() {
    let renderer = ZenoRenderer::new();

    // Create empty shaping result
    let shaped = ShapingResult {
        glyphs: vec![],
        advance_width: 100.0,
        advance_height: 20.0,
        direction: Direction::LeftToRight,
    };

    // Create stub font with empty data (will fail but we're testing error handling)
    let font = Arc::new(StubFont { data: vec![] }) as Arc<dyn FontRef>;

    let params = RenderParams::default();

    // Should succeed with empty glyph list
    let result = renderer.render(&shaped, font, &params);
    assert!(result.is_ok());

    if let Ok(RenderOutput::Bitmap(bitmap)) = result {
        assert_eq!(bitmap.format, BitmapFormat::Rgba8);
        assert!(bitmap.width > 0);
        assert!(bitmap.height > 0);
    }
}
