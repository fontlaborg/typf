//! Integration tests for SVG renderer
//!
//! Tests SVG output structure and validates with real fonts.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use typf_core::{
    traits::{FontRef, Renderer},
    types::{Direction, PositionedGlyph, RenderOutput, ShapingResult, VectorFormat},
    RenderParams,
};
use typf_render_svg::SvgRenderer;

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
        Some(0)
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
fn test_svg_renderer_creation() {
    let renderer = SvgRenderer::new();
    assert_eq!(renderer.name(), "svg");
}

#[test]
fn test_svg_render_with_real_font() {
    let font = match load_font("NotoSans-Regular.ttf") {
        Some(f) => f,
        None => {
            eprintln!("Skipping test: NotoSans-Regular.ttf not found");
            return;
        },
    };

    let renderer = SvgRenderer::new();
    let shaped = simple_shaping_result();
    let params = RenderParams::default();

    let result = renderer.render(&shaped, font, &params);
    assert!(result.is_ok(), "Render should succeed");

    if let Ok(RenderOutput::Vector(vector)) = result {
        assert_eq!(vector.format, VectorFormat::Svg);

        // Validate SVG structure
        let svg = &vector.data;
        assert!(svg.contains("<?xml"), "Should have XML declaration");
        assert!(svg.contains("<svg"), "Should have SVG element");
        assert!(svg.contains("</svg>"), "Should close SVG element");
        assert!(svg.contains("viewBox"), "Should have viewBox attribute");
        assert!(
            svg.contains("<path"),
            "Should have path elements for glyphs"
        );
    } else {
        panic!("Expected vector output");
    }
}

#[test]
fn test_svg_render_empty_text() {
    let font = match load_font("NotoSans-Regular.ttf") {
        Some(f) => f,
        None => return,
    };

    let renderer = SvgRenderer::new();
    let shaped = ShapingResult {
        glyphs: vec![],
        advance_width: 0.0,
        advance_height: 200.0,
        direction: Direction::LeftToRight,
    };
    let params = RenderParams::default();

    let result = renderer.render(&shaped, font, &params);
    assert!(result.is_ok(), "Empty text should render successfully");

    if let Ok(RenderOutput::Vector(vector)) = result {
        assert!(
            vector.data.contains("<svg"),
            "Should still produce valid SVG"
        );
        assert!(vector.data.contains("</svg>"), "Should close SVG");
        // Empty text should have no path elements
        assert!(
            !vector.data.contains("<path"),
            "Empty text should have no paths"
        );
    }
}

#[test]
fn test_svg_viewbox_dimensions() {
    let font = match load_font("NotoSans-Regular.ttf") {
        Some(f) => f,
        None => return,
    };

    let renderer = SvgRenderer::new();
    let shaped = simple_shaping_result();
    let params = RenderParams::default();

    let result = renderer.render(&shaped, font, &params);

    if let Ok(RenderOutput::Vector(vector)) = result {
        // Parse viewBox to verify dimensions are reasonable
        if let Some(viewbox_start) = vector.data.find("viewBox=\"") {
            let viewbox_content = &vector.data[viewbox_start + 9..];
            if let Some(viewbox_end) = viewbox_content.find('"') {
                let viewbox = &viewbox_content[..viewbox_end];
                let parts: Vec<f32> = viewbox
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();

                assert_eq!(parts.len(), 4, "viewBox should have 4 values");
                assert!(parts[2] > 0.0, "Width should be positive");
                assert!(parts[3] > 0.0, "Height should be positive");
            }
        }
    }
}

#[test]
fn test_svg_color_support() {
    let font = match load_font("NotoSans-Regular.ttf") {
        Some(f) => f,
        None => return,
    };

    let renderer = SvgRenderer::new();
    let shaped = simple_shaping_result();

    // Custom foreground color
    let mut params = RenderParams::default();
    params.foreground = typf_core::Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };

    let result = renderer.render(&shaped, font, &params);

    if let Ok(RenderOutput::Vector(vector)) = result {
        // Verify color is applied
        assert!(
            vector.data.contains("rgb(255,0,0)"),
            "Should apply foreground color"
        );
    }
}

#[test]
fn test_svg_consistency() {
    let font = match load_font("NotoSans-Regular.ttf") {
        Some(f) => f,
        None => return,
    };

    let renderer = SvgRenderer::new();
    let shaped = simple_shaping_result();
    let params = RenderParams::default();

    // Render the same input twice
    let result1 = renderer
        .render(&shaped, font.clone(), &params)
        .expect("First render should succeed");
    let result2 = renderer
        .render(&shaped, font, &params)
        .expect("Second render should succeed");

    // Extract SVGs and verify they're identical
    if let (RenderOutput::Vector(v1), RenderOutput::Vector(v2)) = (result1, result2) {
        assert_eq!(v1.data, v2.data, "SVG output should be consistent");
    } else {
        panic!("Expected vector outputs");
    }
}

#[test]
fn test_svg_supports_format() {
    let renderer = SvgRenderer::new();
    assert!(renderer.supports_format("svg"));
    assert!(renderer.supports_format("SVG"));
    assert!(renderer.supports_format("vector"));
    assert!(!renderer.supports_format("png")); // Vector renderer doesn't support PNG
}

#[test]
fn test_svg_with_padding() {
    let font = match load_font("NotoSans-Regular.ttf") {
        Some(f) => f,
        None => return,
    };

    let renderer = SvgRenderer::new().with_padding(20.0);
    let shaped = simple_shaping_result();
    let params = RenderParams::default();

    let result = renderer.render(&shaped, font, &params);
    assert!(result.is_ok(), "Render with padding should succeed");
}
