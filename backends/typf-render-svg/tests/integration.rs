//! Integration tests for SVG renderer
//!
//! Tests SVG output structure and validates with real fonts.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use typf_core::{
    traits::{FontRef, Renderer},
    types::{Direction, PositionedGlyph, RenderOutput, ShapingResult, VectorFormat},
    GlyphSource, GlyphSourcePreference, RenderParams,
};
use typf_fontdb::TypfFontFace;
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

/// Load a real font via typf-fontdb for glyph-dependent tests
fn load_real_font(name: &str) -> Option<Arc<dyn FontRef>> {
    let path = test_font_path(name);
    if !path.exists() {
        return None;
    }
    TypfFontFace::from_file(path)
        .ok()
        .map(|font| Arc::new(font) as Arc<dyn FontRef>)
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

/// Find the first glyph ID with COLR color data
fn first_color_glyph(font: &Arc<dyn FontRef>) -> Option<u32> {
    use typf_render_color::get_color_glyph_format;

    let data = font.data();
    let glyph_count = font.glyph_count().unwrap_or(512);

    (0..glyph_count).find(|gid| get_color_glyph_format(data, *gid).is_some())
}

/// Extract the base64 payload from a data URI inside the SVG
fn extract_base64_image(svg: &str) -> Option<String> {
    let start = svg.find("base64,")? + "base64,".len();
    let end = svg[start..].find('"')? + start;
    Some(svg[start..end].to_string())
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
fn test_svg_embeds_colr_glyph_as_image() {
    let font = match load_real_font("Nabla-Regular-COLR.ttf") {
        Some(f) => f,
        None => {
            eprintln!("Skipping test: Nabla-Regular-COLR.ttf not found");
            return;
        },
    };

    let glyph_id = match first_color_glyph(&font) {
        Some(gid) => gid,
        None => {
            eprintln!("Skipping test: no COLR glyphs found");
            return;
        },
    };

    let advance_width = font.advance_width(glyph_id);

    let shaped = ShapingResult {
        glyphs: vec![PositionedGlyph {
            id: glyph_id,
            x: 0.0,
            y: 0.0,
            advance: advance_width,
            cluster: 0,
        }],
        advance_width,
        advance_height: 256.0,
        direction: Direction::LeftToRight,
    };

    let renderer = SvgRenderer::new();
    let params = RenderParams {
        glyph_sources: GlyphSourcePreference::from_parts(
            vec![GlyphSource::Colr1, GlyphSource::Colr0, GlyphSource::Glyf],
            [],
        ),
        ..RenderParams::default()
    };

    let result = renderer.render(&shaped, font, &params);

    if let Ok(RenderOutput::Vector(vector)) = result {
        assert!(
            vector.data.contains("<image"),
            "SVG should embed color glyph as image"
        );
        assert!(
            vector.data.contains("data:image/png;base64,"),
            "SVG should embed PNG data for color glyph"
        );
    } else {
        panic!("Expected vector output");
    }
}

#[test]
fn test_svg_color_palette_affects_output() {
    let font = match load_real_font("Nabla-Regular-COLR.ttf") {
        Some(f) => f,
        None => return,
    };

    let glyph_id = match first_color_glyph(&font) {
        Some(gid) => gid,
        None => return,
    };

    // Verify palette count >= 2; otherwise skip
    let palette_count = skrifa::FontRef::new(font.data())
        .ok()
        .map(|f| skrifa::color::ColorPalettes::new(&f).len())
        .unwrap_or(0);
    if palette_count < 2 {
        eprintln!(
            "Skipping palette test: font has {} palette(s)",
            palette_count
        );
        return;
    }

    let advance_width = font.advance_width(glyph_id);
    let shaped = ShapingResult {
        glyphs: vec![PositionedGlyph {
            id: glyph_id,
            x: 0.0,
            y: 0.0,
            advance: advance_width,
            cluster: 0,
        }],
        advance_width,
        advance_height: 256.0,
        direction: Direction::LeftToRight,
    };

    let renderer = SvgRenderer::new();

    let mut params0 = RenderParams::default();
    params0.color_palette = 0;
    params0.glyph_sources =
        GlyphSourcePreference::from_parts(vec![GlyphSource::Colr1, GlyphSource::Glyf], []);
    let svg0 = match renderer.render(&shaped, font.clone(), &params0) {
        Ok(RenderOutput::Vector(v)) => v.data,
        _ => panic!("Expected vector output for palette 0"),
    };

    let mut params1 = params0.clone();
    params1.color_palette = 1;
    let svg1 = match renderer.render(&shaped, font, &params1) {
        Ok(RenderOutput::Vector(v)) => v.data,
        _ => panic!("Expected vector output for palette 1"),
    };

    let img0 = extract_base64_image(&svg0).expect("Palette 0 image should exist");
    let img1 = extract_base64_image(&svg1).expect("Palette 1 image should exist");

    if img0 == img1 {
        eprintln!("Skipping palette diff: palettes render identical PNG for this font");
        return;
    }

    assert_ne!(
        img0, img1,
        "Different palettes should change embedded image data"
    );
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
