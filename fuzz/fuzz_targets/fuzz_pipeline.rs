#![no_main]

use libfuzzer_sys::fuzz_target;
use std::sync::Arc;
use typf_core::{Pipeline, RenderParams, ShapingParams, traits::{FontRef, Shaper, Renderer, Exporter}, types::{RenderOutput, ShapingResult}};

// Minimal mock implementations for fuzzing
struct FuzzShaper;
impl typf_core::traits::Stage for FuzzShaper {
    fn name(&self) -> &'static str { "fuzz" }
    fn process(&self, ctx: typf_core::context::PipelineContext) -> Result<typf_core::context::PipelineContext, typf_core::error::TypfError> {
        Ok(ctx)
    }
}
impl Shaper for FuzzShaper {
    fn name(&self) -> &'static str { "fuzz" }
    fn shape(&self, text: &str, _font: Arc<dyn FontRef>, _params: &ShapingParams) -> typf_core::Result<ShapingResult> {
        Ok(ShapingResult {
            glyphs: vec![],
            advance_width: text.len() as f32 * 10.0,
            advance_height: 16.0,
            direction: typf_core::types::Direction::LeftToRight,
        })
    }
}

struct FuzzRenderer;
impl typf_core::traits::Stage for FuzzRenderer {
    fn name(&self) -> &'static str { "fuzz" }
    fn process(&self, ctx: typf_core::context::PipelineContext) -> Result<typf_core::context::PipelineContext, typf_core::error::TypfError> {
        Ok(ctx)
    }
}
impl Renderer for FuzzRenderer {
    fn name(&self) -> &'static str { "fuzz" }
    fn render(&self, _result: &ShapingResult, _font: Arc<dyn FontRef>, _params: &RenderParams) -> typf_core::Result<RenderOutput> {
        Ok(RenderOutput {
            width: 100,
            height: 100,
            data: vec![0; 100 * 100 * 4],
        })
    }
}

struct FuzzExporter;
impl typf_core::traits::Stage for FuzzExporter {
    fn name(&self) -> &'static str { "fuzz" }
    fn process(&self, ctx: typf_core::context::PipelineContext) -> Result<typf_core::context::PipelineContext, typf_core::error::TypfError> {
        Ok(ctx)
    }
}
impl Exporter for FuzzExporter {
    fn name(&self) -> &'static str { "fuzz" }
    fn export(&self, _output: &RenderOutput) -> typf_core::Result<Vec<u8>> {
        Ok(vec![])
    }
    fn extension(&self) -> &'static str { "bin" }
    fn mime_type(&self) -> &'static str { "application/octet-stream" }
}

struct FuzzFont;
impl FontRef for FuzzFont {
    fn data(&self) -> &[u8] { &[] }
    fn glyph_count(&self) -> usize { 100 }
    fn units_per_em(&self) -> u16 { 1000 }
}

fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);

    if text.is_empty() || text.len() > 1_000 {
        return;
    }

    // Build pipeline with fuzz backends
    let pipeline = Pipeline::builder()
        .shaper(Arc::new(FuzzShaper))
        .renderer(Arc::new(FuzzRenderer))
        .exporter(Arc::new(FuzzExporter))
        .build();

    if let Ok(pipeline) = pipeline {
        let font = Arc::new(FuzzFont);
        let shaping_params = ShapingParams::default();
        let render_params = RenderParams::default();

        // Process should not panic
        let _ = pipeline.process(&text, font, &shaping_params, &render_params);
    }
});
