//! Watch TYPF turn text into pixels - the simplest possible way
//!
//! This example shows TYPF's six-stage pipeline in action: text goes in,
//! gets shaped, rendered, and exported as an image. No complex setup required.

use std::fs;
use std::sync::Arc;

use typf_core::{
    traits::{Exporter, FontRef, Renderer, Shaper},
    Color, RenderParams, ShapingParams,
};
use typf_export::{PnmExporter, PnmFormat};
use typf_render_orge::OrgeRenderer;
use typf_shape_none::NoneShaper;

/// A bare-bones font that maps ASCII characters to glyph IDs
///
/// In real applications, you'd load actual font files. This stub exists
/// purely to demonstrate the pipeline mechanics without file dependencies.
struct StubFont;

impl FontRef for StubFont {
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
            Some(0) // .notdef
        }
    }

    fn advance_width(&self, _glyph_id: u32) -> f32 {
        600.0
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = "Hello, TYPF!"; // The raw string we'll transform

    // In production, you'd load a real font file here
    let font = Arc::new(StubFont);

    // Stage 1: Shape the text - convert characters to positioned glyphs
    let shaping_params = ShapingParams {
        size: 24.0,          // 24-point text
        ..Default::default()
    };

    // Stage 2: Render the glyphs - turn them into pixels
    let render_params = RenderParams {
        foreground: Color::black(),      // Black text on...
        background: Some(Color::white()), // ...white background
        padding: 10,                      // 10-pixel border
        ..Default::default()
    };

    // Build our pipeline: shape → render → export
    let shaper = NoneShaper::new();      // Handles character-to-glyph conversion
    let renderer = OrgeRenderer::new();  // Rasterizes glyphs to bitmap
    let exporter = PnmExporter::new(PnmFormat::Ppm); // Saves as PPM image

    // Execute the complete pipeline
    println!("Shaping text: {}", text);
    let shaped = shaper.shape(text, font.clone(), &shaping_params)?;
    println!("  Generated {} glyphs", shaped.glyphs.len());

    println!("Rendering glyphs...");
    let rendered = renderer.render(&shaped, font, &render_params)?;

    println!("Exporting to PPM format...");
    let exported = exporter.export(&rendered)?;

    // Write the final image to disk
    let output_path = "examples/output.ppm";
    fs::write(output_path, exported)?;
    println!("Saved to {}", output_path);

    Ok(())
}
