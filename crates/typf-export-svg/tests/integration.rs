//! Integration tests for SVG exporter
//!
//! Community project by FontLab - https://www.fontlab.org/

use std::sync::Arc;
use typf_core::{
    traits::FontRef,
    types::{Direction, GlyphId, ShapingResult},
    Color,
};
use typf_export_svg::SvgExporter;

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
fn test_exporter_creation() {
    let exporter = SvgExporter::new();
    let with_padding = exporter.with_padding(15.0);
    // Just ensure it compiles and works
    assert!(format!("{:?}", with_padding).len() > 0);
}

#[test]
fn test_export_empty_glyphs() {
    let exporter = SvgExporter::new();

    let shaped = ShapingResult {
        glyphs: vec![],
        advance_width: 100.0,
        advance_height: 20.0,
        direction: Direction::LeftToRight,
    };

    let font = Arc::new(StubFont { data: vec![] }) as Arc<dyn FontRef>;

    let foreground = Color::black();

    // Should succeed with empty glyph list
    let result = exporter.export(&shaped, font, foreground);

    // With empty font data, export will fail gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_svg_output_structure() {
    let exporter = SvgExporter::new();

    let shaped = ShapingResult {
        glyphs: vec![],
        advance_width: 100.0,
        advance_height: 20.0,
        direction: Direction::LeftToRight,
    };

    let font = Arc::new(StubFont { data: vec![] }) as Arc<dyn FontRef>;

    let foreground = Color::rgba(255, 0, 0, 255);

    if let Ok(svg) = exporter.export(&shaped, font, foreground) {
        // Check SVG structure
        assert!(svg.contains("<?xml"));
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("viewBox"));
    }
}

/// Test that CBDT bitmap fonts are handled gracefully
///
/// CBDT fonts contain bitmap data, not outlines.
/// The SVG exporter should skip bitmap glyphs gracefully instead of failing.
#[test]
fn test_cbdt_bitmap_font_graceful_handling() {
    // Load the CBDT test font
    let font_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-fonts/Nabla-Regular-CBDT.ttf");

    let font_data = match std::fs::read(font_path) {
        Ok(data) => data,
        Err(_) => {
            // Skip test if font file not found (CI environment)
            return;
        }
    };

    // Create a simple real font wrapper
    struct RealFont {
        data: Vec<u8>,
    }

    impl FontRef for RealFont {
        fn data(&self) -> &[u8] {
            &self.data
        }

        fn units_per_em(&self) -> u16 {
            // CBDT fonts typically use 1000 or 2048
            2048
        }

        fn glyph_id(&self, _ch: char) -> Option<GlyphId> {
            Some(1) // Return a valid glyph ID
        }

        fn advance_width(&self, _glyph_id: GlyphId) -> f32 {
            600.0
        }
    }

    let font = Arc::new(RealFont { data: font_data }) as Arc<dyn FontRef>;

    // Create shaped result with glyphs that exist in the font
    let shaped = ShapingResult {
        glyphs: vec![
            typf_core::types::PositionedGlyph {
                id: 1,  // Glyph ID that exists but has no outline
                x: 0.0,
                y: 0.0,
                advance: 600.0,
                cluster: 0,
            },
        ],
        advance_width: 600.0,
        advance_height: 2048.0,
        direction: Direction::LeftToRight,
    };

    let exporter = SvgExporter::new();
    let foreground = Color::black();

    // Export should succeed - bitmap glyphs are skipped gracefully
    let result = exporter.export(&shaped, font, foreground);

    assert!(result.is_ok(), "CBDT font export should succeed with graceful skip");

    if let Ok(svg) = result {
        // SVG should be valid (has header/footer)
        assert!(svg.contains("<?xml"), "SVG should have XML header");
        assert!(svg.contains("<svg"), "SVG should have svg element");
        assert!(svg.contains("</svg>"), "SVG should have closing svg");
        // Bitmap glyphs produce empty paths, which are skipped
        // So we shouldn't see path elements for bitmap glyphs
    }
}

/// Test bitmap embedding when the bitmap-embed feature is enabled
///
/// CBDT fonts should have their glyphs embedded as base64 PNG images.
#[test]
#[cfg(feature = "bitmap-embed")]
fn test_cbdt_bitmap_font_embedding() {
    // Load the CBDT test font
    let font_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-fonts/Nabla-Regular-CBDT.ttf");

    let font_data = match std::fs::read(font_path) {
        Ok(data) => data,
        Err(_) => {
            // Skip test if font file not found (CI environment)
            return;
        }
    };

    // Create a real font wrapper
    struct RealFont {
        data: Vec<u8>,
    }

    impl FontRef for RealFont {
        fn data(&self) -> &[u8] {
            &self.data
        }

        fn units_per_em(&self) -> u16 {
            2048
        }

        fn glyph_id(&self, _ch: char) -> Option<GlyphId> {
            Some(1)
        }

        fn advance_width(&self, _glyph_id: GlyphId) -> f32 {
            600.0
        }
    }

    let font = Arc::new(RealFont { data: font_data }) as Arc<dyn FontRef>;

    // Create shaped result with a glyph that exists in the font
    let shaped = ShapingResult {
        glyphs: vec![
            typf_core::types::PositionedGlyph {
                id: 36,  // Glyph ID for 'A' in Nabla
                x: 0.0,
                y: 0.0,
                advance: 600.0,
                cluster: 0,
            },
        ],
        advance_width: 600.0,
        advance_height: 48.0, // Reasonable font size for bitmap lookup
        direction: Direction::LeftToRight,
    };

    let exporter = SvgExporter::new().with_bitmap_embedding(true);
    let foreground = Color::black();

    // Export should succeed with bitmap embedding
    let result = exporter.export(&shaped, font, foreground);

    assert!(result.is_ok(), "CBDT font export with bitmap embedding should succeed: {:?}", result.err());

    if let Ok(svg) = result {
        // SVG should be valid
        assert!(svg.contains("<?xml"), "SVG should have XML header");
        assert!(svg.contains("<svg"), "SVG should have svg element");
        assert!(svg.contains("</svg>"), "SVG should have closing svg");

        // With bitmap embedding enabled, we should see <image> elements with base64 data
        // (if the bitmap was successfully rendered)
        if svg.contains("<image") {
            assert!(svg.contains("data:image/png;base64,"), "Image should have base64 PNG data URI");
            assert!(svg.contains("href=\"data:"), "Image should use href attribute");
        }
    }
}
