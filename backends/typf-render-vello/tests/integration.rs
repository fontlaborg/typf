//! Integration tests for Vello GPU renderer
//!
//! Tests rendering with real fonts and verifies output format/structure.
//! Note: These tests require GPU hardware and may be skipped in CI environments.

// this_file: backends/typf-render-vello/tests/integration.rs

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use typf_core::{
    traits::{FontRef, Renderer},
    types::{BitmapFormat, Direction, PositionedGlyph, RenderOutput, ShapingResult},
    RenderParams,
};
use typf_render_vello::{VelloConfig, VelloRenderer};

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

/// Try to create a GPU renderer, skipping test if no GPU available
fn try_create_renderer() -> Option<VelloRenderer> {
    match VelloRenderer::new() {
        Ok(r) => Some(r),
        Err(e) => {
            eprintln!("Skipping GPU test: {}", e);
            None
        },
    }
}

#[test]
fn test_vello_renderer_creation() {
    // This test verifies we can at least attempt to create a renderer
    // It may fail if no GPU is available, which is acceptable
    let result = VelloRenderer::new();
    match result {
        Ok(renderer) => {
            assert_eq!(renderer.name(), "vello");
        },
        Err(e) => {
            eprintln!("GPU renderer creation failed (expected in CI): {}", e);
        },
    }
}

#[test]
fn test_vello_render_with_real_font() {
    let renderer = match try_create_renderer() {
        Some(r) => r,
        None => return,
    };

    let font = match load_font("NotoSans-Regular.ttf") {
        Some(f) => f,
        None => {
            eprintln!("Skipping test: NotoSans-Regular.ttf not found");
            return;
        },
    };

    let shaped = simple_shaping_result();
    let params = RenderParams::default();

    let result = renderer.render(&shaped, font, &params);
    assert!(result.is_ok(), "Render should succeed: {:?}", result.err());

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
        unreachable!("Expected bitmap output");
    }
}

#[test]
fn test_vello_render_empty_text() {
    let renderer = match try_create_renderer() {
        Some(r) => r,
        None => return,
    };

    let font = match load_font("NotoSans-Regular.ttf") {
        Some(f) => f,
        None => return,
    };

    let shaped = ShapingResult {
        glyphs: vec![],
        advance_width: 0.0,
        advance_height: 200.0,
        direction: Direction::LeftToRight,
    };
    let params = RenderParams::default();

    let result = renderer.render(&shaped, font, &params);
    // Empty text with 0 advance_width will result in zero dimensions
    // This should either succeed with a minimal canvas or fail gracefully
    match result {
        Ok(RenderOutput::Bitmap(bitmap)) => {
            // If it succeeds, verify basic structure
            assert_eq!(bitmap.format, BitmapFormat::Rgba8);
        },
        Err(e) => {
            // Zero dimensions error is acceptable for zero-width canvas
            let err_str = e.to_string();
            assert!(
                err_str.contains("zero") || err_str.contains("dimension"),
                "Should fail with dimension error, got: {}",
                err_str
            );
        },
        _ => unreachable!("Unexpected result type"),
    }
}

#[test]
fn test_vello_render_different_sizes() {
    let renderer = match try_create_renderer() {
        Some(r) => r,
        None => return,
    };

    let font = match load_font("NotoSans-Regular.ttf") {
        Some(f) => f,
        None => return,
    };

    // Render at different sizes
    for size in [24.0, 48.0, 72.0, 200.0] {
        let mut shaped = simple_shaping_result();
        shaped.advance_height = size;

        let params = RenderParams::default();
        let result = renderer.render(&shaped, font.clone(), &params);

        assert!(result.is_ok(), "Size {} should render successfully", size);

        if let Ok(RenderOutput::Bitmap(bitmap)) = result {
            assert!(
                bitmap.height > 0,
                "Height should be positive for size {}",
                size
            );
        }
    }
}

#[test]
fn test_vello_bitmap_stability() {
    let renderer = match try_create_renderer() {
        Some(r) => r,
        None => return,
    };

    let font = match load_font("NotoSans-Regular.ttf") {
        Some(f) => f,
        None => return,
    };

    let shaped = simple_shaping_result();
    let params = RenderParams::default();

    // Render the same input twice
    let result1 = match renderer.render(&shaped, font.clone(), &params) {
        Ok(result) => result,
        Err(e) => unreachable!("first render should succeed: {e}"),
    };
    let result2 = match renderer.render(&shaped, font, &params) {
        Ok(result) => result,
        Err(e) => unreachable!("second render should succeed: {e}"),
    };

    // Extract bitmaps and verify they're identical
    match (result1, result2) {
        (RenderOutput::Bitmap(b1), RenderOutput::Bitmap(b2)) => {
            assert_eq!(b1.width, b2.width, "Width should be consistent");
            assert_eq!(b1.height, b2.height, "Height should be consistent");
            assert_eq!(b1.data, b2.data, "Bitmap data should be identical");
        },
        _ => unreachable!("Expected bitmap outputs"),
    }
}

#[test]
fn test_vello_supports_format() {
    let renderer = match try_create_renderer() {
        Some(r) => r,
        None => {
            // Even without GPU, we can test the trait implementation
            // by checking default behavior
            return;
        },
    };

    assert!(renderer.supports_format("bitmap"));
    assert!(renderer.supports_format("rgba"));
    assert!(!renderer.supports_format("svg")); // Raster renderer doesn't support SVG
}

#[test]
fn test_vello_with_custom_config() {
    let config = VelloConfig {
        power_preference: wgpu::PowerPreference::LowPower,
        ..Default::default()
    };

    match VelloRenderer::with_config(config) {
        Ok(renderer) => {
            assert_eq!(renderer.name(), "vello");
        },
        Err(e) => {
            eprintln!(
                "GPU renderer with custom config failed (expected in CI): {}",
                e
            );
        },
    }
}

#[test]
fn test_vello_render_arabic_font() {
    let renderer = match try_create_renderer() {
        Some(r) => r,
        None => return,
    };

    let font = match load_font("NotoNaskhArabic-Regular.ttf") {
        Some(f) => f,
        None => {
            eprintln!("Skipping test: NotoNaskhArabic-Regular.ttf not found");
            return;
        },
    };

    // Create RTL shaping result (simplified - real RTL would have different glyph positions)
    let shaped = ShapingResult {
        glyphs: vec![PositionedGlyph {
            id: 100, // Arabic glyph
            x: 0.0,
            y: 0.0,
            advance: 80.0,
            cluster: 0,
        }],
        advance_width: 80.0,
        advance_height: 48.0,
        direction: Direction::RightToLeft,
    };

    let params = RenderParams::default();
    let result = renderer.render(&shaped, font, &params);
    assert!(
        result.is_ok(),
        "Arabic font should render: {:?}",
        result.err()
    );
}

#[test]
fn test_vello_render_variable_font() {
    let renderer = match try_create_renderer() {
        Some(r) => r,
        None => return,
    };

    let font = match load_font("Kalnia[wdth,wght].ttf") {
        Some(f) => f,
        None => {
            eprintln!("Skipping test: Kalnia[wdth,wght].ttf not found");
            return;
        },
    };

    let shaped = simple_shaping_result();
    let params = RenderParams::default();

    let result = renderer.render(&shaped, font, &params);
    assert!(
        result.is_ok(),
        "Variable font should render: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "Vendored vello_hybrid currently ignores bitmap/COLR glyph types; use vello-cpu for color fonts (see PLANSTEPS/01-rendering-quality-status.md)"]
fn test_vello_render_colr_color_font() {
    let renderer = match try_create_renderer() {
        Some(r) => r,
        None => return,
    };

    let font = match load_font("Nabla-Regular-COLR.ttf") {
        Some(f) => f,
        None => {
            eprintln!("Skipping test: Nabla-Regular-COLR.ttf not found");
            return;
        },
    };

    let shaped = simple_shaping_result();
    let params = RenderParams::default();

    let result = renderer.render(&shaped, font, &params);
    assert!(
        result.is_ok(),
        "COLR color font should render: {:?}",
        result.err()
    );

    if let Ok(RenderOutput::Bitmap(bitmap)) = result {
        assert_eq!(bitmap.format, BitmapFormat::Rgba8);
        assert!(bitmap.width > 0);
        assert!(bitmap.height > 0);
    }
}

#[test]
#[ignore = "Vendored vello_hybrid currently ignores bitmap/COLR glyph types; use vello-cpu for color fonts (see PLANSTEPS/01-rendering-quality-status.md)"]
fn test_vello_render_cbdt_color_font() {
    let renderer = match try_create_renderer() {
        Some(r) => r,
        None => return,
    };

    let font = match load_font("Nabla-Regular-CBDT.ttf") {
        Some(f) => f,
        None => {
            eprintln!("Skipping test: Nabla-Regular-CBDT.ttf not found");
            return;
        },
    };

    let shaped = simple_shaping_result();
    let params = RenderParams::default();

    let result = renderer.render(&shaped, font, &params);
    assert!(
        result.is_ok(),
        "CBDT bitmap color font should render: {:?}",
        result.err()
    );
}

#[test]
#[ignore = "Vendored vello_hybrid currently ignores bitmap/COLR glyph types; use vello-cpu for color fonts (see PLANSTEPS/01-rendering-quality-status.md)"]
fn test_vello_render_sbix_color_font() {
    let renderer = match try_create_renderer() {
        Some(r) => r,
        None => return,
    };

    let font = match load_font("Nabla-Regular-sbix.ttf") {
        Some(f) => f,
        None => {
            eprintln!("Skipping test: Nabla-Regular-sbix.ttf not found");
            return;
        },
    };

    let shaped = simple_shaping_result();
    let params = RenderParams::default();

    let result = renderer.render(&shaped, font, &params);
    assert!(
        result.is_ok(),
        "sbix bitmap color font should render: {:?}",
        result.err()
    );
}

#[test]
fn test_vello_render_math_font() {
    let renderer = match try_create_renderer() {
        Some(r) => r,
        None => return,
    };

    let font = match load_font("STIX2Math.otf") {
        Some(f) => f,
        None => {
            eprintln!("Skipping test: STIX2Math.otf not found");
            return;
        },
    };

    let shaped = simple_shaping_result();
    let params = RenderParams::default();

    let result = renderer.render(&shaped, font, &params);
    assert!(
        result.is_ok(),
        "Math font (STIX2Math) should render: {:?}",
        result.err()
    );

    if let Ok(RenderOutput::Bitmap(bitmap)) = result {
        assert_eq!(bitmap.format, BitmapFormat::Rgba8);
        assert!(bitmap.width > 0);
        assert!(bitmap.height > 0);
    }
}
