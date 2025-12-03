//! Example demonstrating HarfBuzz shaping with real font loading

use std::sync::Arc;
use typf_core::traits::Shaper;
use typf_core::{types::Direction, ShapingParams};
use typf_fontdb::TypfFontFace;
use typf_shape_hb::HarfBuzzShaper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Typf HarfBuzz Example");
    println!("=====================");

    // Load a font
    println!("\n1. Loading font...");
    let font_path = "/System/Library/Fonts/Helvetica.ttc"; // Common macOS font
    let font = TypfFontFace::from_file(font_path)?;
    let font_arc = Arc::new(font) as Arc<dyn typf_core::traits::FontRef>;

    println!("   Font loaded successfully!");
    println!("   Units per em: {}", font_arc.units_per_em());

    // Create the HarfBuzz shaper
    println!("\n2. Creating HarfBuzz shaper...");
    let shaper = HarfBuzzShaper::new();

    // Test text examples
    let test_texts = [
        "Hello, World!",
        "Typography is fun!",
        "Typf with HarfBuzz",
        "1234567890",
        "kerning: AV Ta We",
    ];

    println!("\n3. Shaping text examples:\n");

    for text in &test_texts {
        println!("   Text: \"{}\"", text);

        // Configure shaping parameters
        let params = ShapingParams {
            size: 12.0,
            direction: Direction::LeftToRight,
            ..Default::default()
        };

        // Shape the text
        let result = shaper.shape(text, font_arc.clone(), &params)?;

        println!("      Glyphs: {}", result.glyphs.len());
        println!("      Width: {:.2} pixels", result.advance_width);

        // Show first few glyph details
        for (i, glyph) in result.glyphs.iter().take(3).enumerate() {
            println!(
                "      Glyph[{}]: id={}, x={:.2}, advance={:.2}",
                i, glyph.id, glyph.x, glyph.advance
            );
        }
        if result.glyphs.len() > 3 {
            println!("      ... and {} more glyphs", result.glyphs.len() - 3);
        }
        println!();
    }

    // Demonstrate directional shaping
    println!("\n4. Testing different text directions:\n");

    let directions = [
        (Direction::LeftToRight, "Left-to-Right"),
        (Direction::RightToLeft, "Right-to-Left"),
    ];

    let sample_text = "Hello";

    for (direction, name) in &directions {
        println!("   Direction: {}", name);

        let params = ShapingParams {
            size: 12.0,
            direction: *direction,
            ..Default::default()
        };

        let result = shaper.shape(sample_text, font_arc.clone(), &params)?;

        println!("      Glyphs positioned:");
        for glyph in &result.glyphs {
            println!("        id={}, x={:.2}", glyph.id, glyph.x);
        }
        println!();
    }

    println!("\nSuccess! HarfBuzz shaping is working with real fonts.");
    println!("Community project by FontLab https://www.fontlab.org/");

    Ok(())
}
