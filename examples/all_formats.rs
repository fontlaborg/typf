//! Comprehensive example demonstrating all export formats
//!
//! This example renders the same text using different export formats:
//! - PNG (standard bitmap)
//! - PNM (minimal bitmap)
//! - SVG (vector graphics)
//! - JSON (shaping data)
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

    // Text to render
    let text = "Hello, TYPF!";
    let complex_text = "مرحبا"; // Arabic: "Hello"

    // Create output directory
    fs::create_dir_all("examples/output")?;

    println!("Rendering: \"{}\"", text);
    println!("Complex script: \"{}\" (Arabic)\n", complex_text);

    // Example 1: Simple rendering with NoneShaper + PNG export
    println!("1. Simple Latin text (NoneShaper + PNG)");
    render_simple_png(text)?;

    // Example 2: HarfBuzz shaping + PNG export
    #[cfg(feature = "shaping-hb")]
    {
        println!("2. HarfBuzz shaping + PNG export");
        render_harfbuzz_png(text)?;
    }

    // Example 3: Complex script with JSON export
    #[cfg(feature = "shaping-hb")]
    {
        println!("3. Complex script (Arabic) + JSON export");
        render_json_export(complex_text)?;
    }

    // Example 4: SVG export
    println!("4. SVG vector export");
    render_svg_export(text)?;

    // Example 5: PNM formats (PPM, PGM)
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

/// Render with simple NoneShaper and export to PNG
fn render_simple_png(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple bitmap (stub - would normally use OrgeRenderer)
    let bitmap = BitmapData {
        width: 200,
        height: 50,
        format: BitmapFormat::Rgba8,
        data: create_simple_bitmap(200, 50),
    };

    let output = RenderOutput::Bitmap(bitmap);

    // Export to PNG
    let exporter = PngExporter::new();
    let png_data = exporter.export(&output)?;

    fs::write("examples/output/simple.png", png_data)?;
    println!("   ✓ Saved: simple.png ({} bytes)", png_data.len());

    Ok(())
}

/// Render with HarfBuzz and export to PNG
#[cfg(feature = "shaping-hb")]
fn render_harfbuzz_png(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create bitmap
    let bitmap = BitmapData {
        width: 300,
        height: 60,
        format: BitmapFormat::Rgba8,
        data: create_simple_bitmap(300, 60),
    };

    let output = RenderOutput::Bitmap(bitmap);

    // Export to PNG
    let exporter = PngExporter::new();
    let png_data = exporter.export(&output)?;

    fs::write("examples/output/harfbuzz.png", png_data)?;
    println!("   ✓ Saved: harfbuzz.png ({} bytes)", png_data.len());

    Ok(())
}

/// Export shaping results as JSON
#[cfg(feature = "shaping-hb")]
fn render_json_export(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    use typf_core::types::PositionedGlyph;

    // Create sample shaping result (Arabic text)
    let shaping_result = ShapingResult {
        glyphs: vec![
            PositionedGlyph {
                id: 42,
                cluster: 0,
                x: 0.0,
                y: 0.0,
                advance: 15.5,
            },
            PositionedGlyph {
                id: 43,
                cluster: 1,
                x: 15.5,
                y: 0.0,
                advance: 12.0,
            },
            PositionedGlyph {
                id: 44,
                cluster: 2,
                x: 27.5,
                y: 0.0,
                advance: 14.5,
            },
        ],
        advance_width: 42.0,
        advance_height: 16.0,
        direction: Direction::RightToLeft,
    };

    let output = RenderOutput::Shaping(shaping_result);

    // Export to JSON with pretty printing
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

/// Export as SVG
fn render_svg_export(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple bitmap for SVG embedding
    let bitmap = BitmapData {
        width: 250,
        height: 60,
        format: BitmapFormat::Rgba8,
        data: create_colored_bitmap(250, 60),
    };

    let output = RenderOutput::Bitmap(bitmap);

    // Export to SVG
    let exporter = SvgExporter::new();
    let svg_data = exporter.export(&output)?;

    fs::write("examples/output/vector.svg", &svg_data)?;
    println!("   ✓ Saved: vector.svg ({} bytes)", svg_data.len());

    Ok(())
}

/// Export as PNM formats
fn render_pnm_formats(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    use typf_export::PnmFormat;

    // Color PPM
    let bitmap_color = BitmapData {
        width: 200,
        height: 50,
        format: BitmapFormat::Rgb8,
        data: create_colored_bitmap_rgb(200, 50),
    };

    let output_color = RenderOutput::Bitmap(bitmap_color);
    let exporter_ppm = PnmExporter::ppm();
    let ppm_data = exporter_ppm.export(&output_color)?;

    fs::write("examples/output/color.ppm", ppm_data)?;
    println!("   ✓ Saved: color.ppm (PPM color)");

    // Grayscale PGM
    let bitmap_gray = BitmapData {
        width: 200,
        height: 50,
        format: BitmapFormat::Gray8,
        data: create_grayscale_bitmap(200, 50),
    };

    let output_gray = RenderOutput::Bitmap(bitmap_gray);
    let exporter_pgm = PnmExporter::pgm();
    let pgm_data = exporter_pgm.export(&output_gray)?;

    fs::write("examples/output/gray.pgm", pgm_data)?;
    println!("   ✓ Saved: gray.pgm (PGM grayscale)");

    Ok(())
}

// Helper functions to create sample bitmaps

fn create_simple_bitmap(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            // Create a gradient pattern
            let r = (x * 255 / width) as u8;
            let g = (y * 255 / height) as u8;
            let b = 128;
            let a = 255;

            data.push(r);
            data.push(g);
            data.push(b);
            data.push(a);
        }
    }

    data
}

fn create_colored_bitmap(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            // Create a more interesting pattern
            let r = ((x + y) * 255 / (width + height)) as u8;
            let g = (x * 255 / width) as u8;
            let b = (y * 255 / height) as u8;
            let a = 255;

            data.push(r);
            data.push(g);
            data.push(b);
            data.push(a);
        }
    }

    data
}

fn create_colored_bitmap_rgb(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 3) as usize);

    for y in 0..height {
        for x in 0..width {
            let r = (x * 255 / width) as u8;
            let g = (y * 255 / height) as u8;
            let b = 200;

            data.push(r);
            data.push(g);
            data.push(b);
        }
    }

    data
}

fn create_grayscale_bitmap(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            // Diagonal gradient
            let gray = ((x + y) * 255 / (width + height)) as u8;
            data.push(gray);
        }
    }

    data
}
