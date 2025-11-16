// this_file: typf/examples/full_text_icu_hb_orge.rs

//! Example: Full Unicode text line rendering with ICU-HB + orge
//!
//! This demonstrates the complete text processing pipeline:
//! Unicode text → ICU segmentation → HarfBuzz shaping → skrifa outlines → orge rendering
//!
//! Handles:
//! - Complex scripts (Arabic, Devanagari, etc.)
//! - Bidirectional text
//! - OpenType features
//! - Variable fonts
//! - TrueType and CFF outlines

use typf_core::{Backend, Font, RenderFormat, RenderOptions, SegmentOptions};
use typf_icu_hb::IcuHbBackend;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create backend (uses orge renderer by default)
    let backend = IcuHbBackend::new();

    println!("Backend: {}", backend.name());

    // Test 1: Simple Latin text
    {
        println!("\n=== Test 1: Latin Text ===");
        let text = "Hello, World!";
        let font = Font::from_family("Noto Sans")?;

        let runs = backend.segment(text, &SegmentOptions::default())?;
        println!("Text runs: {}", runs.len());

        for (i, run) in runs.iter().enumerate() {
            let shaped = backend.shape(run, &font)?;
            println!("Run {}: {} glyphs", i, shaped.glyphs.len());

            let output = backend.render(
                &shaped,
                &RenderOptions {
                    format: RenderFormat::Png,
                    ..Default::default()
                },
            )?;

            println!("Rendered: {}x{} pixels", output.width, output.height);
        }
    }

    // Test 2: Arabic text (complex shaping)
    {
        println!("\n=== Test 2: Arabic Text ===");
        let text = "مرحبا بالعالم"; // "Hello World" in Arabic
        let font = Font::from_family("Noto Naskh Arabic")?;

        let runs = backend.segment(text, &SegmentOptions::default())?;
        println!("Text runs: {} (may include BiDi)", runs.len());

        for run in &runs {
            let shaped = backend.shape(run, &font)?;
            println!(
                "Shaped glyphs: {} (with contextual forms)",
                shaped.glyphs.len()
            );

            let output = backend.render(&shaped, &RenderOptions::default())?;
            println!("Rendered: {}x{} pixels", output.width, output.height);
        }
    }

    // Test 3: Variable font
    {
        println!("\n=== Test 3: Variable Font ===");
        let text = "Variable";
        let mut font = Font::from_file("testdata/fonts/RobotoFlex-VariableFont_wght.ttf")?;

        // Set weight axis to Bold (700)
        font.set_variation("wght", 700.0);

        let runs = backend.segment(text, &SegmentOptions::default())?;
        let shaped = backend.shape(&runs[0], &font)?;
        let output = backend.render(&shaped, &RenderOptions::default())?;

        println!(
            "Rendered at wght=700: {}x{} pixels",
            output.width, output.height
        );
    }

    // Test 4: OpenType features
    {
        println!("\n=== Test 4: OpenType Features ===");
        let text = "Ligatures: fi fl ffi";
        let mut font = Font::from_family("Noto Sans")?;

        // Enable ligatures
        font.enable_feature("liga");

        let runs = backend.segment(text, &SegmentOptions::default())?;
        let shaped = backend.shape(&runs[0], &font)?;

        println!(
            "With ligatures: {} glyphs (fewer than characters)",
            shaped.glyphs.len()
        );

        let output = backend.render(&shaped, &RenderOptions::default())?;
        println!("Rendered: {}x{} pixels", output.width, output.height);
    }

    println!("\n✅ Full Unicode text rendering successful!");
    println!("   • ICU segmentation");
    println!("   • HarfBuzz shaping");
    println!("   • skrifa font parsing");
    println!("   • orge ultra-smooth rendering");

    Ok(())
}
