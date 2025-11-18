//! Example of using the Pipeline builder pattern

use std::sync::Arc;
use typf_core::{traits::FontRef, Color, Pipeline, RenderParams, ShapingParams};
use typf_export::PnmExporter;
use typf_render_orge::OrgeRenderer;
use typf_shape_none::NoneShaper;

/// Mock font implementation
struct MockFont {
    size: f32,
}

impl FontRef for MockFont {
    fn data(&self) -> &[u8] {
        &[]
    }
    fn units_per_em(&self) -> u16 {
        1000
    }
    fn glyph_id(&self, ch: char) -> Option<u32> {
        Some(ch as u32)
    }
    fn advance_width(&self, _: u32) -> f32 {
        self.size * 0.5
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create pipeline with builder pattern
    let pipeline = Pipeline::builder()
        .shaper(Arc::new(NoneShaper::new()))
        .renderer(Arc::new(OrgeRenderer::new()))
        .exporter(Arc::new(PnmExporter::ppm()))
        .build()?;

    // Process text
    let text = "Pipeline Example";
    let font = Arc::new(MockFont { size: 20.0 });

    let shaping_params = ShapingParams {
        size: 20.0,
        ..Default::default()
    };

    let render_params = RenderParams {
        foreground: Color::rgba(0, 0, 255, 255), // Blue text
        background: Some(Color::rgba(255, 255, 200, 255)), // Light yellow background
        padding: 15,
        ..Default::default()
    };

    // Run the pipeline
    let output = pipeline.process(text, font, &shaping_params, &render_params)?;

    // Save result
    std::fs::write("examples/pipeline_output.ppm", output)?;
    println!("Pipeline output saved to examples/pipeline_output.ppm");

    Ok(())
}
