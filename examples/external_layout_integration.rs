//! Integration pattern for external layout engines (cosmic-text, parley)
//!
//! This example demonstrates how typf can serve as a **rasterization backend**
//! for external layout engines. The key insight is that engines like cosmic-text
//! and parley already handle shaping internally - typf's value is in its
//! diverse rendering backends (Opixa, Skia, Vello, etc.).
//!
//! ## Integration Pattern
//!
//! 1. External layout engine produces positioned glyphs (glyph_id, x, y)
//! 2. Convert to `typf_core::types::PositionedGlyph`
//! 3. Build a `ShapingResult` from the external positions
//! 4. Pass to any typf renderer (Opixa, Skia, Vello-CPU, etc.)
//!
//! This decoupling allows users to choose typf for rendering while keeping
//! their preferred layout engine.

use std::sync::Arc;

use typf_core::{
    traits::{Exporter, FontRef, Renderer},
    types::{Direction, PositionedGlyph, ShapingResult},
    Color, RenderParams,
};
use typf_export::{PnmExporter, PnmFormat};
use typf_render_opixa::OpixaRenderer;

/// Simulates positioned glyph output from an external layout engine
///
/// In a real integration, this would come from:
/// - `cosmic_text::LayoutRun::glyphs` → map to `PositionedGlyph`
/// - `parley::Layout::runs()` → iterate `GlyphRun` → map positions
struct ExternalLayoutEngine;

/// Glyph position from an external engine (simplified)
struct ExternalGlyph {
    glyph_id: u32,
    x: f32,
    y: f32,
    advance: f32,
    cluster: u32,
}

impl ExternalLayoutEngine {
    /// Simulate layout output from cosmic-text or parley
    ///
    /// Real code would call:
    /// ```ignore
    /// for run in buffer.layout_runs() {
    ///     for glyph in &run.glyphs {
    ///         external_glyphs.push(ExternalGlyph {
    ///             glyph_id: glyph.glyph_id,
    ///             x: glyph.x + run.line_x,
    ///             y: glyph.y + run.line_y,
    ///             advance: glyph.w,
    ///             cluster: glyph.start,
    ///         });
    ///     }
    /// }
    /// ```
    fn layout_text(&self, text: &str, size: f32) -> Vec<ExternalGlyph> {
        // Simulate simple LTR layout
        let advance = size * 0.6; // ~60% of font size
        let mut x = 0.0;

        text.chars()
            .enumerate()
            .map(|(i, ch)| {
                let glyph = ExternalGlyph {
                    glyph_id: ch as u32,
                    x,
                    y: 0.0,
                    advance,
                    cluster: i as u32,
                };
                x += advance;
                glyph
            })
            .collect()
    }
}

/// Stub font for demonstration
///
/// In production, use `typf_fontdb::TypfFontFace::from_file()` or
/// share the font reference from the layout engine.
struct StubFont;

impl FontRef for StubFont {
    fn data(&self) -> &[u8] {
        &[]
    }

    fn units_per_em(&self) -> u16 {
        1000
    }

    fn glyph_id(&self, ch: char) -> Option<u32> {
        Some(ch as u32)
    }

    fn advance_width(&self, _glyph_id: u32) -> f32 {
        600.0
    }
}

/// Convert external glyph positions to typf's ShapingResult
///
/// This is the key integration point - transform the layout engine's
/// output into typf's internal format for rendering.
fn to_shaping_result(glyphs: Vec<ExternalGlyph>) -> ShapingResult {
    let total_advance: f32 = glyphs.iter().map(|g| g.advance).sum();

    ShapingResult {
        glyphs: glyphs
            .into_iter()
            .map(|g| PositionedGlyph {
                id: g.glyph_id,
                x: g.x,
                y: g.y,
                advance: g.advance,
                cluster: g.cluster,
            })
            .collect(),
        advance_width: total_advance,
        advance_height: 0.0,
        direction: Direction::LeftToRight,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = "Hello from external layout!";
    let font_size = 24.0;

    println!("=== External Layout Engine Integration Demo ===\n");

    // Step 1: External layout engine produces glyph positions
    // (In production, this is cosmic-text, parley, or similar)
    let layout_engine = ExternalLayoutEngine;
    let external_glyphs = layout_engine.layout_text(text, font_size);
    println!("External layout produced {} glyphs", external_glyphs.len());

    // Step 2: Convert to typf ShapingResult
    let shaped = to_shaping_result(external_glyphs);
    println!(
        "Converted to ShapingResult: advance_width={:.1}px",
        shaped.advance_width
    );

    // Step 3: Use any typf renderer
    // Options: OpixaRenderer, SkiaRenderer, ZenoRenderer, VelloCpuRenderer, etc.
    let renderer = OpixaRenderer::new();
    let font: Arc<dyn FontRef> = Arc::new(StubFont);

    let render_params = RenderParams {
        foreground: Color::black(),
        background: Some(Color::white()),
        padding: 10,
        ..Default::default()
    };

    println!("Rendering with typf-render-opixa...");
    let rendered = renderer.render(&shaped, font, &render_params)?;

    // Step 4: Export using typf exporters
    let exporter = PnmExporter::new(PnmFormat::Ppm);
    let exported = exporter.export(&rendered)?;

    let output_path = "examples/external_layout_output.ppm";
    std::fs::write(output_path, exported)?;
    println!("Saved to {}\n", output_path);

    // Show the integration pattern summary
    println!("=== Integration Pattern ===");
    println!("1. cosmic-text/parley → produces glyph positions");
    println!("2. Map to typf_core::types::PositionedGlyph");
    println!("3. Build ShapingResult from positions");
    println!("4. Pass to any typf Renderer (Opixa/Skia/Vello/etc.)");
    println!("5. Export via typf exporters (PNG/SVG/etc.)");

    Ok(())
}
