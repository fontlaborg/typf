//! One rendering, five exports - see the same text transform into every format
//!
//! Shape and render once, export multiple ways. This demonstrates TYPF's flexibility:
//! PNG for web, SVG for vectors, and the entire PNM family for simple, reliable
//! image storage that's been working since the 1980s.

use std::sync::Arc;
use typf_core::{
    traits::{Exporter, FontRef, Renderer, Shaper},
    types::RenderOutput,
    RenderParams, ShapingParams,
};
use typf_export::{PngExporter, PnmExporter, PnmFormat, SvgExporter};
use typf_render_opixa::OpixaRenderer;
use typf_shape_none::NoneShaper;

/// A font that handles basic ASCII characters gracefully
struct SimpleFont;

impl FontRef for SimpleFont {
    fn data(&self) -> &[u8] {
        &[] // No font data - demonstration stub
    }

    fn units_per_em(&self) -> u16 {
        1000 // Standard font unit space
    }

    fn glyph_id(&self, ch: char) -> Option<u32> {
        // Map alphanumeric and whitespace to glyph IDs
        if ch.is_alphanumeric() || ch.is_whitespace() {
            Some(ch as u32)
        } else {
            Some(0) // .notdef glyph for unsupported characters
        }
    }

    fn advance_width(&self, _: u32) -> f32 {
        500.0 // Consistent width for all characters
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("examples/output")?; // Prepare output directory

    let text = "Format Test"; // The same text will appear in every format
    let font = Arc::new(SimpleFont);

    // Process the text once - then reuse the result for all exports
    let shaper = NoneShaper::new();
    let shaped = shaper.shape(text, font.clone(), &ShapingParams::default())?;

    let renderer = OpixaRenderer::new();
    let rendered = renderer.render(&shaped, font, &RenderParams::default())?;

    // The PNM family: three formats, one simple approach
    let formats = [
        (PnmFormat::Ppm, "examples/output/test.ppm", "PPM (color)"), // 3 bytes/pixel: RGB
        (PnmFormat::Pgm, "examples/output/test.pgm", "PGM (grayscale)"), // 1 byte/pixel: intensity
        (PnmFormat::Pbm, "examples/output/test.pbm", "PBM (black/white)"), // 1 bit/pixel: binary
    ];

    for (format, path, description) in formats {
        let exporter = PnmExporter::new(format);
        let data = exporter.export(&rendered)?;
        std::fs::write(path, data)?;
        println!("Exported {} format to {}", description, path);
    }

    // PNG: The web's workhorse image format
    let png_exporter = PngExporter::new();
    let png_data = png_exporter.export(&rendered)?;
    std::fs::write("examples/output/test.png", png_data)?;
    println!("Exported PNG format to examples/output/test.png");

    // SVG: Vector graphics that scale forever
    let svg_exporter = SvgExporter::new();
    let svg_data = svg_exporter.export(&rendered)?;
    std::fs::write("examples/output/test.svg", svg_data)?;
    println!("Exported SVG format to examples/output/test.svg");

    // Peek inside the rendered bitmap to see what we generated
    if let RenderOutput::Bitmap(ref bitmap) = rendered {
        println!("\nGenerated bitmap details:");
        println!("  Dimensions: {}×{} pixels", bitmap.width, bitmap.height);
        println!("  Color format: {:?}", bitmap.format);
        println!("  Raw data: {} bytes", bitmap.data.len());
    }

    Ok(())
}
