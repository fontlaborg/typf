///! Quick start example demonstrating all major backends
///!
///! This example shows how to use different shaping and rendering backends
///! with minimal code. Perfect for getting started with Typf.
///!
///! Run with: cargo run --example quickstart_backends --features shaping-hb,render-skia,render-zeno
use std::sync::Arc;
use typf_core::{
    traits::{FontRef, Renderer, Shaper},
    RenderParams, ShapingParams,
};
use typf_export::PnmExporter;
use typf_render_opixa::OpixaRenderer;
use typf_shape_none::NoneShaper;

// Stub font for demonstration
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
            Some(0)
        }
    }
    fn advance_width(&self, _glyph_id: u32) -> f32 {
        600.0
    }
}

fn main() -> typf::error::Result<()> {
    println!("Typf Backend Quickstart");
    println!("=======================\n");

    let text = "Hello, World!";
    let font = Arc::new(StubFont) as Arc<dyn FontRef>;

    // 1. Basic: None + Opixa (always available)
    println!("1. Basic rendering (none + opixa)...");
    render_with_backends(
        text,
        font.clone(),
        Arc::new(NoneShaper::new()),
        Arc::new(OpixaRenderer::new()),
        "output_basic.ppm",
    )?;

    // 2. HarfBuzz (if available)
    #[cfg(feature = "shaping-hb")]
    {
        println!("2. HarfBuzz shaping (hb + opixa)...");
        render_with_backends(
            text,
            font.clone(),
            Arc::new(typf_shape_hb::HarfBuzzShaper::new()),
            Arc::new(OpixaRenderer::new()),
            "output_harfbuzz.ppm",
        )?;
    }

    // 3. Skia renderer (if available)
    #[cfg(feature = "render-skia")]
    {
        println!("3. Skia rendering (none + skia)...");
        render_with_backends(
            text,
            font.clone(),
            Arc::new(NoneShaper::new()),
            Arc::new(typf_render_skia::SkiaRenderer::new()),
            "output_skia.ppm",
        )?;
    }

    // 4. Zeno renderer (if available)
    #[cfg(feature = "render-zeno")]
    {
        println!("4. Zeno rendering (none + zeno)...");
        render_with_backends(
            text,
            font.clone(),
            Arc::new(NoneShaper::new()),
            Arc::new(typf_render_zeno::ZenoRenderer::new()),
            "output_zeno.ppm",
        )?;
    }

    // 5. macOS native (if on macOS)
    #[cfg(all(target_os = "macos", feature = "shaping-ct", feature = "render-cg"))]
    {
        println!("5. macOS native (coretext + coregraphics)...");
        render_with_backends(
            text,
            font.clone(),
            Arc::new(typf_shape_ct::CoreTextShaper::new()),
            Arc::new(typf_render_cg::CoreGraphicsRenderer::new()),
            "output_macos.ppm",
        )?;
    }

    println!("\n✓ All backends tested!");
    println!("Check output_*.ppm files in the current directory");

    Ok(())
}

fn render_with_backends(
    text: &str,
    font: Arc<dyn FontRef>,
    shaper: Arc<dyn Shaper + Send + Sync>,
    renderer: Arc<dyn Renderer + Send + Sync>,
    output_file: &str,
) -> typf::error::Result<()> {
    use std::fs::File;
    use std::io::Write;
    use typf_core::traits::Exporter;
    use typf_core::types::Direction;
    use typf_core::Color;

    // Shape the text
    let shaping_params = ShapingParams {
        size: 48.0,
        direction: Direction::LeftToRight,
        ..Default::default()
    };
    let shaped = shaper.shape(text, font.clone(), &shaping_params)?;

    // Render the shaped glyphs
    let render_params = RenderParams {
        foreground: Color::rgba(0, 0, 0, 255),
        background: Some(Color::rgba(255, 255, 255, 255)),
        padding: 10,
        antialias: true,
        ..Default::default()
    };
    let rendered = renderer.render(&shaped, font, &render_params)?;

    // Export to PPM
    let exporter = PnmExporter::ppm();
    let data = exporter.export(&rendered)?;

    // Write to file
    let mut file = File::create(output_file)?;
    file.write_all(&data)?;

    println!("   → Saved to {}", output_file);
    Ok(())
}
