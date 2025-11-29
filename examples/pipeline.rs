//! Build once, render forever - TYPF's pipeline pattern in action
//!
//! The Pipeline builder pattern chains text processing stages into a single,
//! reusable object. Configure it once, then process any text through it.

use std::sync::Arc;
use typf_core::{traits::FontRef, Color, Pipeline, RenderParams, ShapingParams};
use typf_export::PnmExporter;
use typf_render_opixa::OpixaRenderer;
use typf_shape_none::NoneShaper;

/// Demo font with variable glyph spacing based on font size
///
/// This mock font adjusts its advance width proportionally to the requested size,
/// creating a simple scaling effect for demonstration purposes.
struct MockFont {
    size: f32,
}

impl FontRef for MockFont {
    fn data(&self) -> &[u8] {
        &[] // No font data - this is just a placeholder
    }

    fn units_per_em(&self) -> u16 {
        1000 // Standard font units
    }

    fn glyph_id(&self, ch: char) -> Option<u32> {
        Some(ch as u32) // Direct character-to-glyph mapping
    }

    fn advance_width(&self, _: u32) -> f32 {
        self.size * 0.5 // Scale spacing with font size
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Assemble our processing pipeline: shape → render → export
    let pipeline = Pipeline::builder()
        .shaper(Arc::new(NoneShaper::new()))      // Text shaping stage
        .renderer(Arc::new(OpixaRenderer::new()))  // Rasterization stage
        .exporter(Arc::new(PnmExporter::ppm()))   // File export stage
        .build()?;

    // Prepare text and font for processing
    let text = "Pipeline Example";
    let font = Arc::new(MockFont { size: 20.0 });

    // Configure how text gets shaped
    let shaping_params = ShapingParams {
        size: 20.0, // 20-point type
        ..Default::default()
    };

    // Configure how glyphs get rendered
    let render_params = RenderParams {
        foreground: Color::rgba(0, 0, 255, 255), // Blue text
        background: Some(Color::rgba(255, 255, 200, 255)), // Creamy background
        padding: 15,                             // Generous border
        ..Default::default()
    };

    // Execute the complete pipeline in one call
    let output = pipeline.process(text, font, &shaping_params, &render_params)?;

    // Write our rendered image to disk
    std::fs::write("examples/pipeline_output.ppm", output)?;
    println!("Pipeline output saved to examples/pipeline_output.ppm");

    Ok(())
}
