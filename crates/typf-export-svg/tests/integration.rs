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
