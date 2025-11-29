//! Backend Comparison Example
//!
//! This example demonstrates the differences between TYPF's three rendering backends:
//! - Opixa: Pure Rust grayscale rasterizer (minimal dependencies)
//! - Skia: High-quality anti-aliased rendering (tiny-skia)
//! - Zeno: Pure Rust 256x anti-aliased rendering
//!
//! Run with:
//! ```bash
//! cargo run --example backend_comparison --features shaping-hb
//! ```
//!
//! Community project by FontLab - https://www.fontlab.org/

use std::sync::Arc;
use typf_core::{
    traits::{FontRef, Renderer, Shaper},
    types::{BitmapFormat, Direction, PositionedGlyph, RenderOutput, ShapingResult},
    Color, RenderParams,
};

// Import all three renderers
use typf_render_opixa::OpixaRenderer;
use typf_render_skia::SkiaRenderer;
use typf_render_zeno::ZenoRenderer;

/// Simple stub font for demonstration
struct DemoFont {
    data: Vec<u8>,
}

impl FontRef for DemoFont {
    fn data(&self) -> &[u8] {
        &self.data
    }

    fn units_per_em(&self) -> u16 {
        1000
    }

    fn glyph_id(&self, _ch: char) -> Option<u32> {
        Some(0)
    }

    fn advance_width(&self, _glyph_id: u32) -> f32 {
        500.0
    }
}

fn main() {
    println!("=== TYPF Backend Comparison ===\n");

    // Create a simple shaping result
    let shaped = ShapingResult {
        glyphs: vec![
            PositionedGlyph {
                id: 1,
                x: 0.0,
                y: 0.0,
                advance: 500.0,
                cluster: 0,
            },
            PositionedGlyph {
                id: 2,
                x: 500.0,
                y: 0.0,
                advance: 500.0,
                cluster: 1,
            },
        ],
        advance_width: 1000.0,
        advance_height: 64.0,
        direction: Direction::LeftToRight,
    };

    let font = Arc::new(DemoFont { data: vec![] }) as Arc<dyn FontRef>;

    let params = RenderParams {
        foreground: Color::black(),
        background: Some(Color::white()),
        padding: 10,
        antialias: true,
    };

    println!("Text: 2 glyphs, 1000px width, 64px height\n");

    // 1. Opixa Renderer
    println!("1. Opixa Renderer (Pure Rust Grayscale)");
    println!("   - Minimal dependencies");
    println!("   - Grayscale anti-aliasing via oversampling");
    println!("   - Best for embedded systems or minimal builds");

    let opixa = OpixaRenderer::default();
    match opixa.render(&shaped, font.clone(), &params) {
        Ok(RenderOutput::Bitmap(bitmap)) => {
            println!("   ✓ Rendered: {}x{} {:?}", bitmap.width, bitmap.height, bitmap.format);
            println!("   ✓ Size: {} bytes\n", bitmap.data.len());
        }
        Err(e) => println!("   ✗ Error: {}\n", e),
    }

    // 2. Skia Renderer
    println!("2. Skia Renderer (tiny-skia)");
    println!("   - High-quality sub-pixel anti-aliasing");
    println!("   - Vector path rendering with Bézier curves");
    println!("   - Best for high-quality desktop applications");

    let skia = SkiaRenderer::default();
    match skia.render(&shaped, font.clone(), &params) {
        Ok(RenderOutput::Bitmap(bitmap)) => {
            println!("   ✓ Rendered: {}x{} {:?}", bitmap.width, bitmap.height, bitmap.format);
            println!("   ✓ Size: {} bytes\n", bitmap.data.len());
        }
        Err(e) => println!("   ✗ Error: {}\n", e),
    }

    // 3. Zeno Renderer
    println!("3. Zeno Renderer (Pure Rust 256x AA)");
    println!("   - 256x anti-aliased rasterization");
    println!("   - Zero C dependencies (pure Rust)");
    println!("   - Browser-compatible output quality");
    println!("   - Best for cross-platform consistency");

    let zeno = ZenoRenderer::default();
    match zeno.render(&shaped, font.clone(), &params) {
        Ok(RenderOutput::Bitmap(bitmap)) => {
            println!("   ✓ Rendered: {}x{} {:?}", bitmap.width, bitmap.height, bitmap.format);
            println!("   ✓ Size: {} bytes\n", bitmap.data.len());
        }
        Err(e) => println!("   ✗ Error: {}\n", e),
    }

    // Comparison summary
    println!("=== Comparison Summary ===\n");
    println!("Backend  | Quality      | Dependencies | Use Case");
    println!("---------|--------------|--------------|------------------");
    println!("Opixa     | Good         | Minimal      | Embedded, minimal");
    println!("Skia     | Excellent    | tiny-skia    | Desktop, quality");
    println!("Zeno     | Excellent    | Pure Rust    | Cross-platform");
    println!();
    println!("All backends implement the same Renderer trait,");
    println!("making it easy to switch between them!");
}
