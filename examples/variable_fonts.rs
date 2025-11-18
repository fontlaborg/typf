//! Variable font rendering example
//!
//! Demonstrates how to use variable fonts with different axis values.
//!
//! Variable fonts allow dynamic adjustment of typographic attributes like
//! weight, width, slant, and optical size through variation axes.
//!
//! Made by FontLab - https://www.fontlab.com/

use std::sync::Arc;
use typf_core::{
    traits::{FontRef, Shaper},
    types::Direction,
    ShapingParams,
};
use typf_shape_hb::HarfBuzzShaper;

/// Simple font wrapper for demonstration
struct DemoFont {
    data: Vec<u8>,
}

impl FontRef for DemoFont {
    fn data(&self) -> &[u8] {
        &self.data
    }

    fn units_per_em(&self) -> u16 {
        1000
    }

    fn glyph_id(&self, _ch: char) -> Option<u32> {
        Some(0)
    }

    fn advance_width(&self, _glyph_id: u32) -> f32 {
        500.0
    }
}

fn main() {
    println!("=== Variable Font Rendering Examples ===\n");

    let shaper = HarfBuzzShaper::new();
    let font = Arc::new(DemoFont { data: vec![] });

    // Example 1: Weight variation (wght)
    println!("1. Weight Variations:");
    for weight in &[100.0, 400.0, 700.0, 900.0] {
        let params = ShapingParams {
            size: 48.0,
            direction: Direction::LeftToRight,
            language: None,
            script: None,
            features: vec![],
            variations: vec![("wght".to_string(), *weight)],
            letter_spacing: 0.0,
        };

        match shaper.shape("Hello", font.clone(), &params) {
            Ok(result) => {
                println!("  Weight {}: {} glyphs, width: {:.2}px",
                    weight, result.glyphs.len(), result.advance_width);
            }
            Err(e) => println!("  Error at weight {}: {}", weight, e),
        }
    }

    println!();

    // Example 2: Width variation (wdth)
    println!("2. Width Variations:");
    for width in &[75.0, 100.0, 125.0] {
        let params = ShapingParams {
            size: 48.0,
            direction: Direction::LeftToRight,
            language: None,
            script: None,
            features: vec![],
            variations: vec![("wdth".to_string(), *width)],
            letter_spacing: 0.0,
        };

        match shaper.shape("Variable", font.clone(), &params) {
            Ok(result) => {
                println!("  Width {}%: {} glyphs, width: {:.2}px",
                    width, result.glyphs.len(), result.advance_width);
            }
            Err(e) => println!("  Error at width {}: {}", width, e),
        }
    }

    println!();

    // Example 3: Multiple axes (weight + width)
    println!("3. Combined Variations (Weight + Width):");
    let params = ShapingParams {
        size: 48.0,
        direction: Direction::LeftToRight,
        language: None,
        script: None,
        features: vec![],
        variations: vec![
            ("wght".to_string(), 700.0),
            ("wdth".to_string(), 125.0),
        ],
        letter_spacing: 0.0,
    };

    match shaper.shape("Bold Extended", font.clone(), &params) {
        Ok(result) => {
            println!("  Bold Extended: {} glyphs, width: {:.2}px",
                result.glyphs.len(), result.advance_width);
            for (i, glyph) in result.glyphs.iter().enumerate() {
                println!("    Glyph {}: id={}, x={:.2}, advance={:.2}",
                    i, glyph.id, glyph.x, glyph.advance);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!();

    // Example 4: Optical size variation (opsz)
    println!("4. Optical Size Variations:");
    for opsz in &[8.0, 12.0, 24.0, 72.0] {
        let params = ShapingParams {
            size: *opsz,
            direction: Direction::LeftToRight,
            language: None,
            script: None,
            features: vec![],
            variations: vec![("opsz".to_string(), *opsz)],
            letter_spacing: 0.0,
        };

        match shaper.shape("Optical", font.clone(), &params) {
            Ok(result) => {
                println!("  Size {}pt: {} glyphs, width: {:.2}px",
                    opsz, result.glyphs.len(), result.advance_width);
            }
            Err(e) => println!("  Error at size {}: {}", opsz, e),
        }
    }

    println!();

    // Example 5: Slant variation (slnt)
    println!("5. Slant Variations:");
    for slant in &[-15.0, 0.0, 15.0] {
        let params = ShapingParams {
            size: 48.0,
            direction: Direction::LeftToRight,
            language: None,
            script: None,
            features: vec![],
            variations: vec![("slnt".to_string(), *slant)],
            letter_spacing: 0.0,
        };

        match shaper.shape("Italic", font.clone(), &params) {
            Ok(result) => {
                println!("  Slant {}°: {} glyphs, width: {:.2}px",
                    slant, result.glyphs.len(), result.advance_width);
            }
            Err(e) => println!("  Error at slant {}: {}", slant, e),
        }
    }

    println!("\n=== Variable Font Examples Complete ===");
    println!("\nNote: These examples use a stub font. For real results,");
    println!("use an actual variable font file with the desired variation axes.");
    println!("\nCommon variation axes:");
    println!("  wght - Weight (100-900)");
    println!("  wdth - Width (50-200%)");
    println!("  slnt - Slant (-90 to 90 degrees)");
    println!("  opsz - Optical size (matches font size)");
    println!("  ital - Italic (0 or 1)");
    println!("\nMade by FontLab - https://www.fontlab.com/");
}
