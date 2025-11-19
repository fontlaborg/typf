//! SVG Export Example
//!
//! This example demonstrates how to use TYPF's SVG export functionality
//! with real font files to generate scalable vector graphics.
//!
//! Run with:
//! ```bash
//! cargo run --example svg_export_example
//! ```
//!
//! Made by FontLab - https://www.fontlab.com/

use std::sync::Arc;
use typf_core::{
    traits::FontRef,
    types::{Direction, PositionedGlyph, ShapingResult},
    Color,
};
use typf_export_svg::SvgExporter;

/// Example showing SVG export usage
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TYPF SVG Export Example ===\n");

    // For demonstration, create a simple shaped result
    // In a real application, this would come from a Shaper
    let shaped = ShapingResult {
        glyphs: vec![
            PositionedGlyph {
                id: 72, // 'H'
                x: 0.0,
                y: 0.0,
                advance: 600.0,
                cluster: 0,
            },
            PositionedGlyph {
                id: 101, // 'e'
                x: 600.0,
                y: 0.0,
                advance: 500.0,
                cluster: 1,
            },
            PositionedGlyph {
                id: 108, // 'l'
                x: 1100.0,
                y: 0.0,
                advance: 300.0,
                cluster: 2,
            },
            PositionedGlyph {
                id: 108, // 'l'
                x: 1400.0,
                y: 0.0,
                advance: 300.0,
                cluster: 3,
            },
            PositionedGlyph {
                id: 111, // 'o'
                x: 1700.0,
                y: 0.0,
                advance: 500.0,
                cluster: 4,
            },
        ],
        advance_width: 2200.0,
        advance_height: 64.0,
        direction: Direction::LeftToRight,
    };

    println!("Shaped text: 5 glyphs, {}px wide", shaped.advance_width);
    println!("Glyph IDs: {:?}\n", shaped.glyphs.iter().map(|g| g.id).collect::<Vec<_>>());

    // Note: SVG export requires a real font with outline data
    // This example demonstrates the API, but won't produce valid output
    // with an empty stub font
    println!("Note: To generate actual SVG output, you need:");
    println!("  1. Load a real font file (TTF/OTF) with outline data");
    println!("  2. Pass the font to the SvgExporter");
    println!("  3. The exporter will extract glyph outlines and convert to SVG paths\n");

    println!("Example with real font (pseudo-code):");
    println!("```rust");
    println!("// Load font from file");
    println!("let font_data = std::fs::read(\"path/to/font.ttf\")?;");
    println!("let font = Arc::new(RealFont::new(font_data));");
    println!();
    println!("// Create SVG exporter");
    println!("let exporter = SvgExporter::new()");
    println!("    .with_padding(20.0);  // Optional padding");
    println!();
    println!("// Export to SVG");
    println!("let svg = exporter.export(&shaped, font, Color::black())?;");
    println!();
    println!("// Save to file");
    println!("std::fs::write(\"output.svg\", svg)?;");
    println!("```\n");

    println!("The SVG output will contain:");
    println!("  ✓ XML declaration and SVG namespace");
    println!("  ✓ ViewBox for responsive scaling");
    println!("  ✓ <path> elements with glyph outlines");
    println!("  ✓ RGB color and opacity");
    println!("  ✓ Transform attributes for positioning");
    println!();
    println!("Benefits of SVG export:");
    println!("  • Scalable to any resolution");
    println!("  • Small file size (text-based)");
    println!("  • Editable in vector graphics software");
    println!("  • Perfect for web and print");

    Ok(())
}
