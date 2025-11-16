// this_file: typf/examples/direct_orge_single_glyph.rs

//! Example: Direct orge usage for fast single glyph rendering
//!
//! This demonstrates the high-performance path:
//! Unicode codepoint → skrifa → orge → bitmap
//!
//! NO ICU segmentation, NO HarfBuzz shaping overhead.
//! Perfect for rendering individual glyphs or glyph atlases.

use typf_orge::fixed::F26Dot6;
use typf_orge::grayscale::{render_grayscale_direct, GrayscaleLevel};
use typf_orge::scan_converter::ScanConverter;
use skrifa::instance::Size;
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, GlyphId, MetadataProvider};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load font with skrifa
    let font_data = std::fs::read("testdata/fonts/NotoSans-Regular.ttf")?;
    let font = FontRef::new(&font_data)?;

    // Get glyph ID for Unicode codepoint 'A'
    let codepoint = 'A';
    let glyph_id = font.charmap().map(codepoint).unwrap_or(GlyphId::NOTDEF);

    println!("Rendering glyph for '{}' (GID: {:?})", codepoint, glyph_id);

    // Get font metrics for sizing
    let upem = font.head()?.units_per_em();
    let size_px = 48.0;
    let scale = size_px / upem as f32;

    // Calculate bitmap dimensions from glyph metrics
    let metrics = font.glyph_metrics(Size::unscaled(), glyph_id);
    let width = (metrics.advance_width * scale).ceil() as usize;
    let height = (size_px * 1.5) as usize; // Include ascent/descent

    println!("Bitmap size: {}x{}", width, height);

    // Method 1: Monochrome rendering
    {
        let mut sc = ScanConverter::new(width, height);

        // Extract outline and feed directly to scan converter
        if let Some(glyph) = font.outline_glyphs().get(glyph_id) {
            glyph.draw(DrawSettings::unhinted(Size::new(size_px), &[]), &mut sc)?;
        }

        let mut bitmap = vec![0u8; width * height];
        sc.render_mono(&mut bitmap);

        println!("Monochrome rendering complete: {} bytes", bitmap.len());
    }

    // Method 2: Grayscale rendering with 4x4 oversampling
    {
        let bitmap = render_grayscale_direct(width, height, GrayscaleLevel::Level4x4, |sc| {
            if let Some(glyph) = font.outline_glyphs().get(glyph_id) {
                glyph
                    .draw(DrawSettings::unhinted(Size::new(size_px), &[]), sc)
                    .ok();
            }
        });

        println!("Grayscale 4x4 rendering complete: {} bytes", bitmap.len());
    }

    // Method 3: Variable font with axis variations
    {
        // This works even with static fonts (variations are ignored)
        use skrifa::instance::Location;

        let mut location = Location::default();

        // If font has 'wght' axis, set it to 700 (Bold)
        if let Ok(fvar) = font.fvar() {
            for axis in fvar.axes() {
                if axis.axis_tag().to_string() == "wght" {
                    location.coords.push(700.0);
                    println!("Setting wght axis to 700.0");
                }
            }
        }

        let bitmap = render_grayscale_direct(width, height, GrayscaleLevel::Level4x4, |sc| {
            if let Some(glyph) = font.outline_glyphs().get(glyph_id) {
                let settings = DrawSettings::unhinted(Size::new(size_px), &location);
                glyph.draw(settings, sc).ok();
            }
        });

        println!("Variable font rendering complete: {} bytes", bitmap.len());
    }

    println!("\n✅ Direct orge rendering successful!");
    println!("   • No ICU overhead");
    println!("   • No HarfBuzz overhead");
    println!("   • Direct skrifa → orge path");
    println!("   • ~100μs per glyph target");

    Ok(())
}
