//! Example: Render a color emoji glyph to PNG
//!
//! This example demonstrates how to use typf-render-color to render
//! color glyphs from emoji fonts.
//!
//! Run with:
//! ```sh
//! cargo run --example render_emoji --features "bitmap,svg" -- path/to/emoji.ttf 42 output.png
//! ```

use std::env;
use std::fs;
use typf_render_color::{detect_color_font_types, render_glyph, RenderMethod};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage: {} <font.ttf> <glyph_id> <output.png>", args[0]);
        eprintln!("Example: {} NotoColorEmoji.ttf 42 emoji.png", args[0]);
        std::process::exit(1);
    }

    let font_path = &args[1];
    let glyph_id: u32 = args[2].parse().expect("glyph_id must be a number");
    let output_path = &args[3];

    // Load font
    let font_data = fs::read(font_path).expect("Failed to read font file");

    // Detect color capabilities
    let color_types = detect_color_font_types(&font_data);
    println!("Detected color support: {:?}", color_types);

    if color_types.is_empty() {
        eprintln!("Warning: Font has no color glyph support");
    }

    // Render glyph at 128x128 pixels
    let size = 128;
    match render_glyph(&font_data, glyph_id, size, size, size as f32, 0) {
        Ok(result) => {
            let method_name = match result.method {
                RenderMethod::ColrV1 => "COLR v1 (gradients)",
                RenderMethod::ColrV0 => "COLR v0 (layered)",
                RenderMethod::Svg => "SVG table",
                RenderMethod::Bitmap => "Embedded bitmap",
                RenderMethod::Outline => "Outline fallback",
            };
            println!("Rendered glyph {} using {}", glyph_id, method_name);

            // Save as PNG
            result
                .pixmap
                .save_png(output_path)
                .expect("Failed to save PNG");
            println!("Saved to {}", output_path);
        },
        Err(e) => {
            eprintln!("Failed to render glyph {}: {}", glyph_id, e);
            std::process::exit(1);
        },
    }
}
