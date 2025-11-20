//! True vector graphics from fonts - not just bitmaps in SVG clothing
//!
//! This example shows how to extract real glyph outlines from fonts and convert
//! them to SVG paths. The result isn't a bitmap wrapped in SVG - it's genuine
//! vector graphics that scale to any resolution without pixelation.
//!
//! Run with: cargo run --example svg_export_example

use std::sync::Arc;
use typf_core::{
    traits::FontRef,
    types::{Direction, PositionedGlyph, ShapingResult},
    Color,
};
use typf_export_svg::SvgExporter;

/// Demonstrate SVG vector export from glyph outlines
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TYPF SVG Export Example ===\n");

    // Simulate a shaped "Hello" - in real usage, a shaper creates this
    let shaped = ShapingResult {
        glyphs: vec![
            PositionedGlyph {
                id: 72,      // Glyph ID for 'H'
                x: 0.0,      // Position at start of line
                y: 0.0,      // Baseline position
                advance: 600.0, // Width this glyph occupies
                cluster: 0,  // Maps to first character
            },
            PositionedGlyph {
                id: 101,     // Glyph ID for 'e'
                x: 600.0,    // Positioned after 'H'
                y: 0.0,
                advance: 500.0,
                cluster: 1,  // Maps to second character
            },
            PositionedGlyph {
                id: 108,     // Glyph ID for first 'l'
                x: 1100.0,   // Positioned after 'e'
                y: 0.0,
                advance: 300.0,
                cluster: 2,
            },
            PositionedGlyph {
                id: 108,     // Glyph ID for second 'l'
                x: 1400.0,   // Positioned after first 'l'
                y: 0.0,
                advance: 300.0,
                cluster: 3,
            },
            PositionedGlyph {
                id: 111,     // Glyph ID for 'o'
                x: 1700.0,   // Final position
                y: 0.0,
                advance: 500.0,
                cluster: 4,
            },
        ],
        advance_width: 2200.0,  // Total width of shaped text
        advance_height: 64.0,   // Line height
        direction: Direction::LeftToRight,
    };

    println!("Shaped text: 5 glyphs, {}px wide", shaped.advance_width);
    println!("Glyph IDs: {:?}\n", shaped.glyphs.iter().map(|g| g.id).collect::<Vec<_>>());

    // SVG export needs actual font data to extract outlines
    // This demo shows the API, but you'll need real fonts for production
    println!("⚠️  To generate real SVG output, you need:");
    println!("  1. Load an actual font file (TTF/OTF) with vector outlines");
    println!("  2. Pass the font to SvgExporter along with shaping data");
    println!("  3. The exporter extracts outlines and converts to SVG paths\n");

    println!("Real-world example:");
    println!("```rust");
    println!("// Load a proper font file");
    println!("let font_data = std::fs::read(\"fonts/inter.ttf\")?;");
    println!("let font = Arc::new(RealFont::from_data(font_data)?);");
    println!();
    println!("// Configure SVG exporter");
    println!("let exporter = SvgExporter::new()");
    println!("    .with_padding(20.0)      // Add breathing room");
    println!("    .with_precision(2);      // Control path detail");
    println!();
    println!("// Export true vector graphics");
    println!("let svg = exporter.export(&shaped, font, Color::black())?;");
    println!();
    println!("// Save your infinitely scalable text");
    println!("std::fs::write(\"hello.svg\", svg)?;");
    println!("```\n");

    println!("The generated SVG contains:");
    println!("  ✓ Proper XML header and SVG namespace");
    println!("  ✓ Responsive ViewBox for perfect scaling");
    println!("  ✓ <path> elements with actual glyph outlines");
    println!("  ✓ Color, opacity, and stroke properties");
    println!("  ✓ Precise positioning via transform attributes");
    println!();
    println!("Why choose SVG export:");
    println!("  • Infinite resolution - zoom forever without pixelation");
    println!("  • Tiny file sizes - text-based compression beats raster");
    println!("  • Fully editable - open in Illustrator, Inkscape, or Figma");
    println!("  • Web and print ready - the same file works everywhere");

    Ok(())
}
