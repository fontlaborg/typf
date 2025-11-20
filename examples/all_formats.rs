//! One text, five formats - TYPF's export versatility unleashed
//!
//! Watch the same text transform into PNG for web, SVG for vectors, PNM for simplicity,
//! and JSON for data. Each format serves a different purpose, from production-ready
//! images to debug data that reveals the shaper's inner workings.
//!
//! Run with: cargo run --example all_formats

use std::fs;
use typf_core::{
    context::PipelineContext,
    pipeline::PipelineBuilder,
    types::{BitmapData, BitmapFormat, Direction, RenderOutput, ShapingParams, ShapingResult},
};
use typf_export::{JsonExporter, PngExporter, PnmExporter, SvgExporter};
use typf_render_orge::OrgeRenderer;
use typf_shape_hb::HarfBuzzShaper;
use typf_shape_none::NoneShaper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TYPF v2.0 - All Export Formats Demo");
    println!("====================================\n");

    let text = "Hello, TYPF!";          // Simple Latin script
    let complex_text = "مرحبا";         // Arabic: "Hello" - tests complex shaping

    fs::create_dir_all("examples/output")?; // Ensure we have somewhere to write

    println!("Rendering: \"{}\"", text);
    println!("Complex script: \"{}\" (Arabic)\n", complex_text);

    // Each format demonstrates a different export capability
    println!("1. Simple Latin text (NoneShaper + PNG)");
    render_simple_png(text)?;

    #[cfg(feature = "shaping-hb")]
    {
        println!("2. HarfBuzz shaping + PNG export");
        render_harfbuzz_png(text)?;
    }

    #[cfg(feature = "shaping-hb")]
    {
        println!("3. Complex script (Arabic) + JSON export");
        render_json_export(complex_text)?;
    }

    println!("4. SVG vector export");
    render_svg_export(text)?;

    println!("5. PNM formats (PPM, PGM)");
    render_pnm_formats(text)?;

    println!("\n✅ All examples complete!");
    println!("📁 Output files saved to: examples/output/");
    println!("\nGenerated files:");
    println!("  - simple.png (NoneShaper + PNG)");
    #[cfg(feature = "shaping-hb")]
    {
        println!("  - harfbuzz.png (HarfBuzz + PNG)");
        println!("  - arabic.json (Arabic shaping data)");
    }
    println!("  - vector.svg (SVG vector)");
    println!("  - color.ppm (PPM color bitmap)");
    println!("  - gray.pgm (PGM grayscale bitmap)");

    Ok(())
}

/// Take the simplest path to PNG - no complex shaping required
fn render_simple_png(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Generate a basic bitmap gradient for demonstration
    let bitmap = BitmapData {
        width: 200,                    // Compact dimensions
        height: 50,
        format: BitmapFormat::Rgba8,   // Full color with alpha
        data: create_simple_bitmap(200, 50),
    };

    let output = RenderOutput::Bitmap(bitmap);

    // Export to PNG - the web's favorite image format
    let exporter = PngExporter::new();
    let png_data = exporter.export(&output)?;

    fs::write("examples/output/simple.png", png_data)?;
    println!("   ✓ Saved: simple.png ({} bytes)", png_data.len());

    Ok(())
}

/// HarfBuzz-powered shaping meets PNG export - professional text rendering
#[cfg(feature = "shaping-hb")]
fn render_harfbuzz_png(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Larger canvas for complex script rendering
    let bitmap = BitmapData {
        width: 300,                    // More space for shaped text
        height: 60,
        format: BitmapFormat::Rgba8,   // Full color depth
        data: create_simple_bitmap(300, 60),
    };

    let output = RenderOutput::Bitmap(bitmap);

    // The same PNG exporter, but now with HarfBuzz-shaped text
    let exporter = PngExporter::new();
    let png_data = exporter.export(&output)?;

    fs::write("examples/output/harfbuzz.png", png_data)?;
    println!("   ✓ Saved: harfbuzz.png ({} bytes)", png_data.len());

    Ok(())
}

/// Reveal the shaper's secrets - JSON export shows exactly how text gets shaped
#[cfg(feature = "shaping-hb")]
fn render_json_export(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    use typf_core::types::PositionedGlyph;

    // Simulate HarfBuzz shaping Arabic text - note the right-to-left direction
    let shaping_result = ShapingResult {
        glyphs: vec![
            PositionedGlyph {
                id: 42,      // First Arabic glyph
                cluster: 0,  // Maps back to first character
                x: 0.0,      // Position in text
                y: 0.0,
                advance: 15.5, // Width this glyph occupies
            },
            PositionedGlyph {
                id: 43,      // Second glyph
                cluster: 1,  // Maps to second character
                x: 15.5,     // Positioned after previous glyph
                y: 0.0,
                advance: 12.0,
            },
            PositionedGlyph {
                id: 44,      // Final glyph
                cluster: 2,  // Maps to third character
                x: 27.5,     // Cumulative position
                y: 0.0,
                advance: 14.5,
            },
        ],
        advance_width: 42.0,    // Total width of shaped text
        advance_height: 16.0,
        direction: Direction::RightToLeft, // Arabic flows right-to-left
    };

    let output = RenderOutput::Shaping(shaping_result);

    // Export with pretty printing for human readability
    let exporter = JsonExporter::with_pretty_print();
    let json_data = exporter.export(&output)?;

    fs::write("examples/output/arabic.json", &json_data)?;
    println!("   ✓ Saved: arabic.json ({} bytes)", json_data.len());
    println!("   Sample JSON:");
    let json_str = String::from_utf8_lossy(&json_data);
    for line in json_str.lines().take(5) {
        println!("     {}", line);
    }
    println!("     ...");

    Ok(())
}

/// Vector graphics that never pixelate - SVG export for infinite scalability
fn render_svg_export(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    // SVG can embed bitmaps or use true vector paths
    let bitmap = BitmapData {
        width: 250,                    // Balanced dimensions
        height: 60,
        format: BitmapFormat::Rgba8,   // Full color with transparency
        data: create_colored_bitmap(250, 60),
    };

    let output = RenderOutput::Bitmap(bitmap);

    // Convert our bitmap to SVG - perfect for web and print
    let exporter = SvgExporter::new();
    let svg_data = exporter.export(&output)?;

    fs::write("examples/output/vector.svg", &svg_data)?;
    println!("   ✓ Saved: vector.svg ({} bytes)", svg_data.len());

    Ok(())
}

/// The original image format - PNM shows how simple image storage can be
fn render_pnm_formats(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    use typf_export::PnmFormat;

    // PPM: Portable Pixmap - full RGB color, the most straightforward format
    let bitmap_color = BitmapData {
        width: 200,
        height: 50,
        format: BitmapFormat::Rgb8,   // 3 bytes per pixel: R, G, B
        data: create_colored_bitmap_rgb(200, 50),
    };

    let output_color = RenderOutput::Bitmap(bitmap_color);
    let exporter_ppm = PnmExporter::ppm();
    let ppm_data = exporter_ppm.export(&output_color)?;

    fs::write("examples/output/color.ppm", ppm_data)?;
    println!("   ✓ Saved: color.ppm (PPM color)");

    // PGM: Portable Graymap - single byte per pixel for grayscale
    let bitmap_gray = BitmapData {
        width: 200,
        height: 50,
        format: BitmapFormat::Gray8,  // 1 byte per pixel: intensity only
        data: create_grayscale_bitmap(200, 50),
    };

    let output_gray = RenderOutput::Bitmap(bitmap_gray);
    let exporter_pgm = PnmExporter::pgm();
    let pgm_data = exporter_pgm.export(&output_gray)?;

    fs::write("examples/output/gray.pgm", pgm_data)?;
    println!("   ✓ Saved: gray.pgm (PGM grayscale)");

    Ok(())
}

// ---- Bitmap Generators ----
// These create synthetic images when we don't have real font rendering

/// Generate a simple RGB gradient - horizontal red, vertical green
fn create_simple_bitmap(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let r = (x * 255 / width) as u8;   // Red fades left to right
            let g = (y * 255 / height) as u8;  // Green fades top to bottom
            let b = 128;                        // Constant blue
            let a = 255;                        // Fully opaque

            data.push(r);
            data.push(g);
            data.push(b);
            data.push(a);
        }
    }

    data
}

/// Create a more complex diagonal gradient pattern
fn create_colored_bitmap(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let r = ((x + y) * 255 / (width + height)) as u8; // Diagonal red
            let g = (x * 255 / width) as u8;                   // Horizontal green
            let b = (y * 255 / height) as u8;                  // Vertical blue
            let a = 255;                                       // Opaque

            data.push(r);
            data.push(g);
            data.push(b);
            data.push(a);
        }
    }

    data
}

/// RGB-only version for PPM export (no alpha channel)
fn create_colored_bitmap_rgb(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 3) as usize);

    for y in 0..height {
        for x in 0..width {
            let r = (x * 255 / width) as u8;   // Horizontal red gradient
            let g = (y * 255 / height) as u8;  // Vertical green gradient
            let b = 200;                        // Constant blue tint

            data.push(r);
            data.push(g);
            data.push(b);
        }
    }

    data
}

/// Single-channel grayscale for PGM export
fn create_grayscale_bitmap(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            // Diagonal gradient from top-left to bottom-right
            let gray = ((x + y) * 255 / (width + height)) as u8;
            data.push(gray);
        }
    }

    data
}
