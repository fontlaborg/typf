//! Basic example of using TYPF to render text

use std::fs;
use std::sync::Arc;

use typf_core::{
    traits::{Exporter, FontRef, Renderer, Shaper},
    Color, RenderParams, ShapingParams,
};
use typf_export::{PnmExporter, PnmFormat};
use typf_render_orge::OrgeRenderer;
use typf_shape_none::NoneShaper;

/// A simple stub font for demonstration
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
    // Text to render
    let text = "Hello, TYPF!";

    // Create a stub font (in real usage, load an actual font)
    let font = Arc::new(StubFont);

    // Configure shaping parameters
    let shaping_params = ShapingParams {
        size: 24.0,
        ..Default::default()
    };

    // Configure rendering parameters
    let render_params = RenderParams {
        foreground: Color::black(),
        background: Some(Color::white()),
        padding: 10,
        ..Default::default()
    };

    // Create pipeline components
    let shaper = NoneShaper::new();
    let renderer = OrgeRenderer::new();
    let exporter = PnmExporter::new(PnmFormat::Ppm);

    // Execute the pipeline
    println!("Shaping text: {}", text);
    let shaped = shaper.shape(text, font.clone(), &shaping_params)?;
    println!("  Generated {} glyphs", shaped.glyphs.len());

    println!("Rendering glyphs...");
    let rendered = renderer.render(&shaped, font, &render_params)?;

    println!("Exporting to PPM format...");
    let exported = exporter.export(&rendered)?;

    // Save to file
    let output_path = "examples/output.ppm";
    fs::write(output_path, exported)?;
    println!("Saved to {}", output_path);

    Ok(())
}
