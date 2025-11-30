//! Integration tests for Opixa renderer
//!
//! Tests rendering with real fonts and verifies output format/structure.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use typf_core::{
    traits::{FontRef, Renderer},
    types::{BitmapFormat, Direction, PositionedGlyph, RenderOutput, ShapingResult},
    RenderParams,
};
use typf_render_opixa::OpixaRenderer;

/// Simple font wrapper for testing
struct TestFont {
    data: Vec<u8>,
    upem: u16,
}

impl FontRef for TestFont {
    fn data(&self) -> &[u8] {
        &self.data
    }

    fn units_per_em(&self) -> u16 {
        self.upem
    }

    fn glyph_id(&self, _ch: char) -> Option<u32> {
        Some(0) // Return dummy glyph ID
    }

    fn advance_width(&self, _glyph_id: u32) -> f32 {
        500.0
    }

    fn glyph_count(&self) -> Option<u32> {
        Some(100)
    }
}

/// Get path to a test font
fn test_font_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // backends
    path.pop(); // root
    path.push("test-fonts");
    path.push(name);
    path
}

/// Load a test font
fn load_font(name: &str) -> Option<Arc<dyn FontRef>> {
    let path = test_font_path(name);
    if !path.exists() {
        return None;
    }
    let data = fs::read(&path).ok()?;
    Some(Arc::new(TestFont { data, upem: 1000 }) as Arc<dyn FontRef>)
}

/// Create a simple shaping result for testing
fn simple_shaping_result() -> ShapingResult {
    let glyphs = vec![
        PositionedGlyph {
            id: 36, // 'H' glyph in many fonts
            x: 0.0,
            y: 0.0,
            advance: 100.0,
            cluster: 0,
        },
        PositionedGlyph {
            id: 72, // 'e' glyph
            x: 100.0,
            y: 0.0,
            advance: 80.0,
            cluster: 1,
        },
    ];

    ShapingResult {
        glyphs,
        advance_width: 180.0,
        advance_height: 200.0,
        direction: Direction::LeftToRight,
    }
}

#[test]
fn test_opixa_renderer_creation() {
    let renderer = OpixaRenderer::new();
    assert_eq!(renderer.name(), "opixa");
}

#[test]
fn test_opixa_render_with_real_font() {
    let font = match load_font("NotoSans-Regular.ttf") {
        Some(f) => f,
        None => {
            eprintln!("Skipping test: NotoSans-Regular.ttf not found");
            return;
        }
    };

    let renderer = OpixaRenderer::new();
    let shaped = simple_shaping_result();
    let params = RenderParams::default();

    let result = renderer.render(&shaped, font, &params);
    assert!(result.is_ok(), "Render should succeed");

    if let Ok(RenderOutput::Bitmap(bitmap)) = result {
        assert_eq!(bitmap.format, BitmapFormat::Rgba8);
        assert!(bitmap.width > 0, "Width should be positive");
        assert!(bitmap.height > 0, "Height should be positive");
        assert!(!bitmap.data.is_empty(), "Data should not be empty");

        // Verify RGBA8 format (4 bytes per pixel)
        let expected_size = (bitmap.width * bitmap.height * 4) as usize;
        assert_eq!(
            bitmap.data.len(),
            expected_size,
            "Data size should match dimensions"
        );
    } else {
        panic!("Expected bitmap output");
    }
}

#[test]
fn test_opixa_render_empty_text() {
    let font = match load_font("NotoSans-Regular.ttf") {
        Some(f) => f,
        None => return,
    };

    let renderer = OpixaRenderer::new();
    let shaped = ShapingResult {
        glyphs: vec![],
        advance_width: 0.0,
        advance_height: 200.0,
        direction: Direction::LeftToRight,
    };
    let params = RenderParams::default();

    let result = renderer.render(&shaped, font, &params);
    assert!(result.is_ok(), "Empty text should render successfully");
}

#[test]
fn test_opixa_render_different_sizes() {
    let font = match load_font("NotoSans-Regular.ttf") {
        Some(f) => f,
        None => return,
    };

    let renderer = OpixaRenderer::new();

    // Render at different sizes
    for size in [24.0, 48.0, 72.0, 200.0] {
        let mut shaped = simple_shaping_result();
        shaped.advance_height = size;

        let params = RenderParams::default();
        let result = renderer.render(&shaped, font.clone(), &params);

        assert!(result.is_ok(), "Size {} should render successfully", size);

        if let Ok(RenderOutput::Bitmap(bitmap)) = result {
            assert!(bitmap.height > 0, "Height should be positive for size {}", size);
        }
    }
}

#[test]
fn test_opixa_bitmap_hash_stability() {
    let font = match load_font("NotoSans-Regular.ttf") {
        Some(f) => f,
        None => return,
    };

    let renderer = OpixaRenderer::new();
    let shaped = simple_shaping_result();
    let params = RenderParams::default();

    // Render the same input twice
    let result1 = renderer
        .render(&shaped, font.clone(), &params)
        .expect("First render should succeed");
    let result2 = renderer
        .render(&shaped, font, &params)
        .expect("Second render should succeed");

    // Extract bitmaps and verify they're identical
    if let (RenderOutput::Bitmap(b1), RenderOutput::Bitmap(b2)) = (result1, result2) {
        assert_eq!(b1.width, b2.width, "Width should be consistent");
        assert_eq!(b1.height, b2.height, "Height should be consistent");
        assert_eq!(b1.data, b2.data, "Bitmap data should be identical");
    } else {
        panic!("Expected bitmap outputs");
    }
}

#[test]
fn test_opixa_supports_format() {
    let renderer = OpixaRenderer::new();
    assert!(renderer.supports_format("bitmap"));
    assert!(renderer.supports_format("rgba"));
    assert!(renderer.supports_format("rgb"));
    assert!(renderer.supports_format("gray"));
    assert!(!renderer.supports_format("svg")); // Raster renderer doesn't support SVG
    assert!(!renderer.supports_format("png")); // PNG encoding is done by exporter, not renderer
}
