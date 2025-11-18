//! Golden tests for HarfBuzz shaping output
//!
//! These tests save shaping results to text files and compare against
//! known-good "golden" outputs to detect regressions.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use typf_core::{
    traits::{FontRef, Shaper},
    types::{Direction, ShapingResult},
    ShapingParams,
};
use typf_shape_hb::HarfBuzzShaper;

/// Mock font for testing
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
        Some(ch as u32)
    }

    fn advance_width(&self, _glyph_id: u32) -> f32 {
        500.0
    }
}

/// Format shaping result as human-readable text
fn format_result(result: &ShapingResult) -> String {
    let mut output = String::new();
    output.push_str(&format!("Glyph count: {}\n", result.glyphs.len()));
    output.push_str(&format!("Advance width: {:.2}\n", result.advance_width));
    output.push_str(&format!("Advance height: {:.2}\n", result.advance_height));
    output.push_str(&format!("Direction: {:?}\n", result.direction));
    output.push_str("Glyphs:\n");
    for (i, glyph) in result.glyphs.iter().enumerate() {
        output.push_str(&format!(
            "  {}: id={}, advance={:.2}, pos=({:.2}, {:.2}), cluster={}\n",
            i, glyph.id, glyph.advance, glyph.x, glyph.y, glyph.cluster
        ));
    }
    output
}

/// Get the path to a golden file
fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(format!("{}.txt", name))
}

/// Load golden file or create if UPDATE_GOLDEN env var is set
fn check_golden(name: &str, actual: &str) {
    let path = golden_path(name);

    if std::env::var("UPDATE_GOLDEN").is_ok() {
        // Update mode: write new golden file
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, actual).unwrap();
        eprintln!("Updated golden file: {}", path.display());
    } else {
        // Test mode: compare against existing golden file
        let expected = fs::read_to_string(&path).unwrap_or_else(|_| {
            panic!(
                "Golden file not found: {}. Run with UPDATE_GOLDEN=1 to create it.",
                path.display()
            )
        });

        if actual.trim() != expected.trim() {
            eprintln!("=== EXPECTED ===");
            eprintln!("{}", expected);
            eprintln!("=== ACTUAL ===");
            eprintln!("{}", actual);
            panic!("Golden test failed for {}", name);
        }
    }
}

#[test]
fn test_golden_simple_latin() {
    let shaper = HarfBuzzShaper::new();
    let font = Arc::new(MockFont { data: vec![] });
    let params = ShapingParams {
        size: 16.0,
        direction: Direction::LeftToRight,
        ..Default::default()
    };

    let result = shaper.shape("Hello", font, &params).unwrap();
    let formatted = format_result(&result);
    check_golden("simple_latin", &formatted);
}

#[test]
fn test_golden_empty_text() {
    let shaper = HarfBuzzShaper::new();
    let font = Arc::new(MockFont { data: vec![] });
    let params = ShapingParams::default();

    let result = shaper.shape("", font, &params).unwrap();
    let formatted = format_result(&result);
    check_golden("empty_text", &formatted);
}

#[test]
fn test_golden_single_char() {
    let shaper = HarfBuzzShaper::new();
    let font = Arc::new(MockFont { data: vec![] });
    let params = ShapingParams {
        size: 16.0,
        ..Default::default()
    };

    let result = shaper.shape("A", font, &params).unwrap();
    let formatted = format_result(&result);
    check_golden("single_char", &formatted);
}

#[test]
fn test_golden_numbers() {
    let shaper = HarfBuzzShaper::new();
    let font = Arc::new(MockFont { data: vec![] });
    let params = ShapingParams {
        size: 16.0,
        ..Default::default()
    };

    let result = shaper.shape("1234567890", font, &params).unwrap();
    let formatted = format_result(&result);
    check_golden("numbers", &formatted);
}

#[test]
fn test_golden_punctuation() {
    let shaper = HarfBuzzShaper::new();
    let font = Arc::new(MockFont { data: vec![] });
    let params = ShapingParams {
        size: 16.0,
        ..Default::default()
    };

    let result = shaper.shape("Hello, World!", font, &params).unwrap();
    let formatted = format_result(&result);
    check_golden("punctuation", &formatted);
}
