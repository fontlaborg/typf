//! Integration tests for orge rasterizer with real fonts

use read_fonts::FontRef;
use typf_orge::GlyphRasterizer;

/// Test rendering a simple glyph from a system font
#[test]
#[ignore] // Only run manually - requires system fonts
fn test_render_simple_glyph_from_font() {
    // Try to load a common system font
    let font_paths = vec![
        "/System/Library/Fonts/Helvetica.ttc",           // macOS
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", // Linux
        "C:\\Windows\\Fonts\\arial.ttf",                 // Windows
    ];

    let font_data = font_paths
        .iter()
        .find_map(|path| std::fs::read(path).ok())
        .expect("Could not load any system font for testing");

    let font = FontRef::new(&font_data).expect("Failed to parse font");

    // Render glyph 'A' (typically glyph ID 36 in many fonts, but let's use index 0 for safety)
    let rasterizer = GlyphRasterizer::new();
    let result = rasterizer.render_glyph(
        &font,
        1, // GID 1 (typically .notdef or first real glyph)
        64.0,   // 64px font size
        &[],    // No variable font coordinates
        128,    // width
        128,    // height
    );

    match result {
        Ok(image) => {
            assert_eq!(image.width(), 128);
            assert_eq!(image.height(), 128);
            assert_eq!(image.pixels().len(), 128 * 128);

            // Image should not be completely empty (assuming glyph 1 is valid)
            let has_pixels = image.pixels().iter().any(|&p| p > 0);
            if has_pixels {
                println!("✓ Successfully rendered glyph with visible pixels");
            } else {
                println!("⚠ Glyph rendered but appears empty (may be .notdef or whitespace)");
            }
        }
        Err(e) => {
            panic!("Failed to render glyph: {}", e);
        }
    }
}

/// Test that orge can handle different fill rules
#[test]
fn test_fill_rules() {
    use typf_orge::FillRule;

    let rasterizer_nonzero = GlyphRasterizer::new().with_fill_rule(FillRule::NonZeroWinding);
    let _rasterizer_evenodd = GlyphRasterizer::new().with_fill_rule(FillRule::EvenOdd);

    // Just verify construction works
    let _ = rasterizer_nonzero;
}

/// Test that orge can handle dropout modes
#[test]
fn test_dropout_modes() {
    use typf_orge::DropoutMode;

    let _rasterizer_simple = GlyphRasterizer::new().with_dropout_mode(DropoutMode::Simple);
    let _rasterizer_smart = GlyphRasterizer::new().with_dropout_mode(DropoutMode::Smart);
    let _rasterizer_none = GlyphRasterizer::new().with_dropout_mode(DropoutMode::None);
}

/// Test error handling for invalid parameters
#[test]
fn test_error_handling() {
    use typf_orge::Image;

    // Test invalid dimensions
    let result = Image::new(0, 100, vec![]);
    assert!(result.is_err());

    let result = Image::new(100, 0, vec![]);
    assert!(result.is_err());

    // Test mismatched buffer size
    let result = Image::new(10, 10, vec![0u8; 50]);
    assert!(result.is_err());

    // Test valid image
    let result = Image::new(10, 10, vec![0u8; 100]);
    assert!(result.is_ok());
}
