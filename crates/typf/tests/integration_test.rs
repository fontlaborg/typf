//! Integration tests for the Typf pipeline

use std::path::PathBuf;
use std::sync::Arc;

use typf_core::{
    traits::{Exporter, FontRef, Renderer, Shaper},
    types::{Direction, RenderOutput},
    Color, RenderParams, ShapingParams,
};
use typf_export::{PnmExporter, PnmFormat};
use typf_fontdb::TypfFontFace;
use typf_render_opixa::OpixaRenderer;
use typf_shape_none::NoneShaper;

/// Get path to test font fixtures
fn test_font_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test-fonts")
        .join(name)
}

/// Mock font for testing
struct MockFont;

impl FontRef for MockFont {
    fn data(&self) -> &[u8] {
        &[]
    }

    fn units_per_em(&self) -> u16 {
        1000
    }

    fn glyph_id(&self, ch: char) -> Option<u32> {
        if ch.is_ascii() {
            Some(ch as u32)
        } else {
            Some(0)
        }
    }

    fn advance_width(&self, _glyph_id: u32) -> f32 {
        500.0
    }
}

#[test]
fn test_full_pipeline() {
    // Setup
    let text = "Hello, Typf!";
    let font = Arc::new(MockFont);

    // Shaping parameters
    let shaping_params = ShapingParams {
        size: 16.0,
        direction: Direction::LeftToRight,
        language: Some("en".to_string()),
        script: None,
        features: Vec::new(),
        variations: Vec::new(),
        letter_spacing: 0.0,
    };

    // Create shaper
    let shaper = NoneShaper::new();

    // Shape the text
    let shaped = shaper
        .shape(text, font.clone(), &shaping_params)
        .expect("Shaping should succeed");

    // Verify shaping results
    assert_eq!(shaped.glyphs.len(), text.chars().count());
    assert!(shaped.advance_width > 0.0);
    assert_eq!(shaped.direction, Direction::LeftToRight);

    // Rendering parameters
    let render_params = RenderParams {
        foreground: Color::black(),
        background: Some(Color::white()),
        padding: 5,
        antialias: true,
        ..Default::default()
    };

    // Create renderer
    let renderer = OpixaRenderer::new();

    // Render to bitmap
    let rendered = renderer
        .render(&shaped, font, &render_params)
        .expect("Rendering should succeed");

    // Verify render output
    match &rendered {
        RenderOutput::Bitmap(bitmap) => {
            assert!(bitmap.width > 0);
            assert!(bitmap.height > 0);
            assert!(!bitmap.data.is_empty());
            assert_eq!(
                bitmap.data.len(),
                (bitmap.width * bitmap.height * 4) as usize
            );
        },
        _ => panic!("Expected bitmap output"),
    }

    // Test export
    let exporter = PnmExporter::ppm();
    let exported = exporter.export(&rendered).expect("Export should succeed");

    // Verify export output
    assert!(!exported.is_empty());
    let exported_str = String::from_utf8_lossy(&exported);
    assert!(exported_str.starts_with("P3")); // PPM header
    assert!(exported_str.contains("255")); // Max color value
}

#[test]
fn test_pipeline_with_different_formats() {
    let text = "Test";
    let font = Arc::new(MockFont);

    let shaping_params = ShapingParams {
        size: 12.0,
        ..Default::default()
    };

    let render_params = RenderParams::default();

    let shaper = NoneShaper::new();
    let shaped = shaper.shape(text, font.clone(), &shaping_params).unwrap();

    let renderer = OpixaRenderer::new();
    let rendered = renderer.render(&shaped, font, &render_params).unwrap();

    // Test different export formats
    for format in [PnmFormat::Ppm, PnmFormat::Pgm, PnmFormat::Pbm] {
        let exporter = PnmExporter::new(format);
        let exported = exporter
            .export(&rendered)
            .expect(&format!("Export to {:?} should succeed", format));

        assert!(!exported.is_empty());

        let header = match format {
            PnmFormat::Ppm => "P3",
            PnmFormat::Pgm => "P2",
            PnmFormat::Pbm => "P1",
        };

        let exported_str = String::from_utf8_lossy(&exported);
        assert!(
            exported_str.starts_with(header),
            "Export for {:?} should start with {}",
            format,
            header
        );
    }
}

#[test]
fn test_empty_text() {
    let font = Arc::new(MockFont);
    let shaping_params = ShapingParams::default();
    let render_params = RenderParams::default();

    let shaper = NoneShaper::new();
    let shaped = shaper.shape("", font.clone(), &shaping_params).unwrap();

    assert_eq!(shaped.glyphs.len(), 0);
    assert_eq!(shaped.advance_width, 0.0);

    // Rendering empty text should still produce a valid (though small) bitmap
    let renderer = OpixaRenderer::new();
    let rendered = renderer.render(&shaped, font, &render_params).unwrap();

    match rendered {
        RenderOutput::Bitmap(bitmap) => {
            // Should have minimum dimensions for empty text (at least 1x16 with no padding)
            assert!(bitmap.width >= 1, "Width should be at least 1");
            assert!(
                bitmap.height >= 16,
                "Height should be at least 16 for empty text"
            );
        },
        _ => panic!("Expected bitmap output"),
    }
}

#[test]
fn test_large_text() {
    let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let font = Arc::new(MockFont);

    let shaping_params = ShapingParams {
        size: 24.0,
        ..Default::default()
    };

    let render_params = RenderParams {
        padding: 10,
        ..Default::default()
    };

    let shaper = NoneShaper::new();
    let shaped = shaper.shape(text, font.clone(), &shaping_params).unwrap();

    assert_eq!(shaped.glyphs.len(), text.len());

    let renderer = OpixaRenderer::new();
    let rendered = renderer.render(&shaped, font, &render_params).unwrap();

    match rendered {
        RenderOutput::Bitmap(bitmap) => {
            // Large text should produce a wide bitmap
            assert!(bitmap.width > 100);
            assert!(bitmap.height > 20);
        },
        _ => panic!("Expected bitmap output"),
    }
}

#[test]
fn test_pipeline_with_colors() {
    let text = "Color Test";
    let font = Arc::new(MockFont);

    let shaping_params = ShapingParams::default();

    // Test with different foreground/background colors
    let test_cases = vec![
        (Color::black(), Some(Color::white())),
        (Color::white(), Some(Color::black())),
        (Color::rgba(255, 0, 0, 255), None), // Red on transparent
    ];

    let shaper = NoneShaper::new();
    let shaped = shaper.shape(text, font.clone(), &shaping_params).unwrap();

    let renderer = OpixaRenderer::new();

    for (fg, bg) in test_cases {
        let render_params = RenderParams {
            foreground: fg,
            background: bg,
            padding: 10, // Add padding so we can test background
            antialias: false,
            ..Default::default()
        };

        let rendered = renderer
            .render(&shaped, font.clone(), &render_params)
            .unwrap();

        match &rendered {
            RenderOutput::Bitmap(bitmap) => {
                if let Some(bg_color) = bg {
                    // Check that background color is applied in the top-left corner (padding area)
                    // The first pixel should be in the padding area, thus background color
                    assert_eq!(bitmap.data[0], bg_color.r, "Red channel mismatch");
                    assert_eq!(bitmap.data[1], bg_color.g, "Green channel mismatch");
                    assert_eq!(bitmap.data[2], bg_color.b, "Blue channel mismatch");
                    assert_eq!(bitmap.data[3], bg_color.a, "Alpha channel mismatch");
                }
            },
            _ => panic!("Expected bitmap output"),
        }

        // Verify export works with the colored output
        let exporter = PnmExporter::ppm();
        let exported = exporter.export(&rendered).unwrap();
        assert!(!exported.is_empty());
    }
}

// =============================================================================
// Real Font Integration Tests
// =============================================================================

#[test]
fn test_real_font_noto_sans_latin() {
    let font_path = test_font_path("NotoSans-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: font not found at {:?}", font_path);
        return;
    }

    // Load real font
    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load NotoSans");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    // Verify font properties
    assert!(font.units_per_em() > 0, "Font should have valid units_per_em");
    assert!(
        font.glyph_id('A').is_some(),
        "Font should contain glyph for 'A'"
    );

    // Shape text
    let shaper = NoneShaper::new();
    let shaping_params = ShapingParams {
        size: 24.0,
        direction: Direction::LeftToRight,
        ..Default::default()
    };

    let text = "Hello, World!";
    let shaped = shaper
        .shape(text, font.clone(), &shaping_params)
        .expect("Shaping should succeed with real font");

    assert_eq!(shaped.glyphs.len(), text.chars().count());
    assert!(shaped.advance_width > 0.0);

    // Render
    let renderer = OpixaRenderer::new();
    let render_params = RenderParams {
        foreground: Color::black(),
        background: Some(Color::white()),
        padding: 10,
        ..Default::default()
    };

    let rendered = renderer
        .render(&shaped, font, &render_params)
        .expect("Rendering should succeed with real font");

    match &rendered {
        RenderOutput::Bitmap(bitmap) => {
            assert!(bitmap.width > 100, "Bitmap should have reasonable width");
            assert!(bitmap.height > 20, "Bitmap should have reasonable height");
            assert!(!bitmap.data.is_empty(), "Bitmap should have pixel data");
        }
        _ => panic!("Expected bitmap output"),
    }

    // Export
    let exporter = PnmExporter::ppm();
    let exported = exporter.export(&rendered).expect("Export should succeed");
    assert!(!exported.is_empty());
}

#[test]
fn test_real_font_arabic_rtl() {
    let font_path = test_font_path("NotoNaskhArabic-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: font not found at {:?}", font_path);
        return;
    }

    // Load Arabic font
    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load Arabic font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    // Shape with RTL direction
    let shaper = NoneShaper::new();
    let shaping_params = ShapingParams {
        size: 24.0,
        direction: Direction::RightToLeft,
        language: Some("ar".to_string()),
        ..Default::default()
    };

    // Arabic text "مرحبا" (Hello)
    let text = "مرحبا";
    let shaped = shaper
        .shape(text, font.clone(), &shaping_params)
        .expect("Shaping should succeed with Arabic font");

    assert!(!shaped.glyphs.is_empty(), "Should produce glyphs for Arabic");
    assert_eq!(shaped.direction, Direction::RightToLeft);

    // Render
    let renderer = OpixaRenderer::new();
    let render_params = RenderParams::default();

    let rendered = renderer
        .render(&shaped, font, &render_params)
        .expect("Rendering should succeed with Arabic font");

    match &rendered {
        RenderOutput::Bitmap(bitmap) => {
            assert!(bitmap.width > 0);
            assert!(bitmap.height > 0);
        }
        _ => panic!("Expected bitmap output"),
    }
}

#[test]
fn test_real_font_variable() {
    let font_path = test_font_path("Kalnia[wdth,wght].ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: font not found at {:?}", font_path);
        return;
    }

    // Load variable font
    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load variable font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    // Shape with variation settings
    let shaper = NoneShaper::new();
    let shaping_params = ShapingParams {
        size: 32.0,
        variations: vec![
            ("wght".to_string(), 700.0), // Bold
            ("wdth".to_string(), 100.0), // Normal width
        ],
        ..Default::default()
    };

    let text = "Variable";
    let shaped = shaper
        .shape(text, font.clone(), &shaping_params)
        .expect("Shaping should succeed with variable font");

    assert_eq!(shaped.glyphs.len(), text.chars().count());

    // Render
    let renderer = OpixaRenderer::new();
    let render_params = RenderParams::default();

    let rendered = renderer
        .render(&shaped, font, &render_params)
        .expect("Rendering should succeed with variable font");

    match &rendered {
        RenderOutput::Bitmap(bitmap) => {
            assert!(bitmap.width > 0);
            assert!(bitmap.height > 0);
        }
        _ => panic!("Expected bitmap output"),
    }
}
