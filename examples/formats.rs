//! Example demonstrating different export formats
//!
//! Shows PNG, SVG, PNM (PPM/PGM/PBM) export capabilities

use std::sync::Arc;
use typf_core::{
    traits::{Exporter, FontRef, Renderer, Shaper},
    types::RenderOutput,
    RenderParams, ShapingParams,
};
use typf_export::{PngExporter, PnmExporter, PnmFormat, SvgExporter};
use typf_render_orge::OrgeRenderer;
use typf_shape_none::NoneShaper;

struct SimpleFont;

impl FontRef for SimpleFont {
    fn data(&self) -> &[u8] {
        &[]
    }
    fn units_per_em(&self) -> u16 {
        1000
    }
    fn glyph_id(&self, ch: char) -> Option<u32> {
        if ch.is_alphanumeric() || ch.is_whitespace() {
            Some(ch as u32)
        } else {
            Some(0)
        }
    }
    fn advance_width(&self, _: u32) -> f32 {
        500.0
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    std::fs::create_dir_all("examples/output")?;

    let text = "Format Test";
    let font = Arc::new(SimpleFont);

    // Shape the text once
    let shaper = NoneShaper::new();
    let shaped = shaper.shape(text, font.clone(), &ShapingParams::default())?;

    // Render once
    let renderer = OrgeRenderer::new();
    let rendered = renderer.render(&shaped, font, &RenderParams::default())?;

    // Export to different formats
    let formats = [
        (PnmFormat::Ppm, "examples/output/test.ppm", "PPM (color)"),
        (PnmFormat::Pgm, "examples/output/test.pgm", "PGM (grayscale)"),
        (PnmFormat::Pbm, "examples/output/test.pbm", "PBM (black/white)"),
    ];

    for (format, path, description) in formats {
        let exporter = PnmExporter::new(format);
        let data = exporter.export(&rendered)?;
        std::fs::write(path, data)?;
        println!("Exported {} format to {}", description, path);
    }

    // Export to PNG
    let png_exporter = PngExporter::new();
    let png_data = png_exporter.export(&rendered)?;
    std::fs::write("examples/output/test.png", png_data)?;
    println!("Exported PNG format to examples/output/test.png");

    // Export to SVG
    let svg_exporter = SvgExporter::new();
    let svg_data = svg_exporter.export(&rendered)?;
    std::fs::write("examples/output/test.svg", svg_data)?;
    println!("Exported SVG format to examples/output/test.svg");

    // Demonstrate bitmap info
    if let RenderOutput::Bitmap(ref bitmap) = rendered {
        println!("\nBitmap information:");
        println!("  Size: {}x{}", bitmap.width, bitmap.height);
        println!("  Format: {:?}", bitmap.format);
        println!("  Data size: {} bytes", bitmap.data.len());
    }

    Ok(())
}
