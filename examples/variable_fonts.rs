//! One font, infinite styles - variable fonts adapt to any design need
//!
//! Variable fonts eliminate the need for separate font files for every weight,
//! width, or style. Instead, they contain a continuum of designs that you can
//! access through variation axes. Think of it as morphing between different
//! font styles in real-time.
//!
//! This example shows how Typf exposes that power through variation parameters.

use std::sync::Arc;
use typf_core::{
    traits::{FontRef, Shaper},
    types::Direction,
    ShapingParams,
};
use typf_shape_hb::HarfBuzzShaper;

/// Demo font container - in production, load actual variable font files
struct DemoFont {
    data: Vec<u8>,
}

impl FontRef for DemoFont {
    fn data(&self) -> &[u8] {
        &self.data // Font data would contain variation tables here
    }

    fn units_per_em(&self) -> u16 {
        1000 // Standard font coordinate space
    }

    fn glyph_id(&self, _ch: char) -> Option<u32> {
        Some(0) // Stub implementation
    }

    fn advance_width(&self, _glyph_id: u32) -> f32 {
        500.0 // Default glyph width
    }
}

fn main() {
    println!("=== Variable Font Rendering Examples ===\n");

    let shaper = HarfBuzzShaper::new();
    let font = Arc::new(DemoFont { data: vec![] });

    // --- Axis 1: Weight ---
    // The wght axis transitions from thin (100) to black (900)
    println!("1. Weight Variations (wght axis):");
    for weight in &[100.0, 400.0, 700.0, 900.0] {
        let params = ShapingParams {
            size: 48.0,
            direction: Direction::LeftToRight,
            language: None,
            script: None,
            features: vec![],
            variations: vec![("wght".to_string(), *weight)], // Weight axis only
            letter_spacing: 0.0,
        };

        match shaper.shape("Hello", font.clone(), &params) {
            Ok(result) => {
                println!(
                    "  Weight {}: {} glyphs, {:.2}px wide",
                    weight,
                    result.glyphs.len(),
                    result.advance_width
                );
            },
            Err(e) => println!("  Error at weight {}: {}", weight, e),
        }
    }

    println!();

    // --- Axis 2: Width ---
    // The wdth axis compresses or expands letter spacing
    println!("2. Width Variations (wdth axis):");
    for width in &[75.0, 100.0, 125.0] {
        let params = ShapingParams {
            size: 48.0,
            direction: Direction::LeftToRight,
            language: None,
            script: None,
            features: vec![],
            variations: vec![("wdth".to_string(), *width)], // Width axis only
            letter_spacing: 0.0,
        };

        match shaper.shape("Variable", font.clone(), &params) {
            Ok(result) => {
                println!(
                    "  Width {}%: {} glyphs, {:.2}px wide",
                    width,
                    result.glyphs.len(),
                    result.advance_width
                );
            },
            Err(e) => println!("  Error at width {}: {}", width, e),
        }
    }

    println!();

    // --- Combined: Weight + Width ---
    // Mix multiple axes for precise control
    println!("3. Combined Variations (Weight + Width):");
    let params = ShapingParams {
        size: 48.0,
        direction: Direction::LeftToRight,
        language: None,
        script: None,
        features: vec![],
        variations: vec![
            ("wght".to_string(), 700.0), // Bold weight
            ("wdth".to_string(), 125.0), // Extended width
        ],
        letter_spacing: 0.0,
    };

    match shaper.shape("Bold Extended", font.clone(), &params) {
        Ok(result) => {
            println!(
                "  Bold Extended: {} glyphs, {:.2}px wide",
                result.glyphs.len(),
                result.advance_width
            );
            // Show individual glyph positioning
            for (i, glyph) in result.glyphs.iter().enumerate() {
                println!(
                    "    Glyph {}: id={}, pos={:.2}, width={:.2}",
                    i, glyph.id, glyph.x, glyph.advance
                );
            }
        },
        Err(e) => println!("  Error: {}", e),
    }

    println!();

    // --- Axis 3: Optical Size ---
    // The opsz axis optimizes glyphs for different display sizes
    println!("4. Optical Size Variations (opsz axis):");
    for opsz in &[8.0, 12.0, 24.0, 72.0] {
        let params = ShapingParams {
            size: *opsz, // Font size matches optical size
            direction: Direction::LeftToRight,
            language: None,
            script: None,
            features: vec![],
            variations: vec![("opsz".to_string(), *opsz)],
            letter_spacing: 0.0,
        };

        match shaper.shape("Optical", font.clone(), &params) {
            Ok(result) => {
                println!(
                    "  Size {}pt: {} glyphs, {:.2}px wide",
                    opsz,
                    result.glyphs.len(),
                    result.advance_width
                );
            },
            Err(e) => println!("  Error at size {}: {}", opsz, e),
        }
    }

    println!();

    // --- Axis 4: Slant ---
    // The slnt axis creates italic-like tilting
    println!("5. Slant Variations (slnt axis):");
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
                println!(
                    "  Slant {}°: {} glyphs, {:.2}px wide",
                    slant,
                    result.glyphs.len(),
                    result.advance_width
                );
            },
            Err(e) => println!("  Error at slant {}: {}", slant, e),
        }
    }

    println!("\n=== Variable Font Examples Complete ===");
    println!("\n💡 Note: These examples use a stub font for API demonstration.");
    println!("For real typographic variation, load an actual variable font file");
    println!("with the variation axes you want to explore.");
    println!("\n🎛️  Standard OpenType variation axes:");
    println!("  wght - Weight (100-900): Light to Black");
    println!("  wdth - Width (50-200%): Compressed to Extended");
    println!("  slnt - Slant (-90° to 90°): Backward to forward tilt");
    println!("  opsz - Optical size: Optimizes for display size");
    println!("  ital - Italic (0 or 1): Roman to Italic switch");
    println!("  GRAD - Grade: Similar to weight but affects spacing more");
    println!("\nCommunity project by FontLab - https://www.fontlab.org/");
}
