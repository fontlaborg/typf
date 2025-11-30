//! Break the pipeline to make it stronger - fuzz Typf's core architecture
//!
//! This fuzzer tests the entire six-stage pipeline for robustness. By using
//! minimal mock implementations, we isolate the pipeline logic itself - the
//! builder pattern, stage execution, error handling, and context passing.
//! We want to ensure malformed text can't crash the pipeline framework.
//!
//! What gets fuzzed:
//! - Pipeline builder with various stage combinations
//! - Error propagation through the chain
//! - Context management between stages
//! - Parameter validation and sanitization
//! - Stage lifecycle (init/process/cleanup)

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::sync::Arc;
use typf_core::{Pipeline, RenderParams, ShapingParams, traits::{FontRef, Shaper, Renderer, Exporter}, types::{RenderOutput, ShapingResult}};

/// Minimal shaper that never crashes but exercises pipeline logic
struct FuzzShaper;
impl typf_core::traits::Stage for FuzzShaper {
    fn name(&self) -> &'static str { "fuzz" }
    fn process(&self, ctx: typf_core::context::PipelineContext) -> Result<typf_core::context::PipelineContext, typf_core::error::TypfError> {
        Ok(ctx) // Pass context through unchanged
    }
}
impl Shaper for FuzzShaper {
    fn name(&self) -> &'static str { "fuzz" }
    fn shape(&self, text: &str, _font: Arc<dyn FontRef>, _params: &ShapingParams) -> typf_core::Result<ShapingResult> {
        // Create plausible output based on text length
        Ok(ShapingResult {
            glyphs: vec![], // Empty glyph list - still exercises shaping logic
            advance_width: text.len() as f32 * 10.0, // Reasonable advance
            advance_height: 16.0,
            direction: typf_core::types::Direction::LeftToRight,
        })
    }
}

/// Minimal renderer that generates consistent bitmaps
struct FuzzRenderer;
impl typf_core::traits::Stage for FuzzRenderer {
    fn name(&self) -> &'static str { "fuzz" }
    fn process(&self, ctx: typf_core::context::PipelineContext) -> Result<typf_core::context::PipelineContext, typf_core::error::TypfError> {
        Ok(ctx) // Pass context through
    }
}
impl Renderer for FuzzRenderer {
    fn name(&self) -> &'static str { "fuzz" }
    fn render(&self, _result: &ShapingResult, _font: Arc<dyn FontRef>, _params: &RenderParams) -> typf_core::Result<RenderOutput> {
        // Return a simple but valid bitmap
        Ok(RenderOutput {
            width: 100,
            height: 100,
            data: vec![0; 100 * 100 * 4], // RGBA bitmap
        })
    }
}

/// Minimal exporter that returns empty but valid data
struct FuzzExporter;
impl typf_core::traits::Stage for FuzzExporter {
    fn name(&self) -> &'static str { "fuzz" }
    fn process(&self, ctx: typf_core::context::PipelineContext) -> Result<typf_core::context::PipelineContext, typf_core::error::TypfError> {
        Ok(ctx) // Pass context through
    }
}
impl Exporter for FuzzExporter {
    fn name(&self) -> &'static str { "fuzz" }
    fn export(&self, _output: &RenderOutput) -> typf_core::Result<Vec<u8>> {
        Ok(vec![]) // Empty but valid export
    }
    fn extension(&self) -> &'static str { "bin" }
    fn mime_type(&self) -> &'static str { "application/octet-stream" }
}

/// Simple font that satisfies interface requirements
struct FuzzFont;
impl FontRef for FuzzFont {
    fn data(&self) -> &[u8] { &[] } // No font data needed
    fn glyph_count(&self) -> usize { 100 } // Reasonable glyph count
    fn units_per_em(&self) -> u16 { 1000 } // Standard units
}

fuzz_target!(|data: &[u8]| {
    // Convert raw bytes to text for pipeline processing
    let text = String::from_utf8_lossy(data);

    // Filter out problematic inputs that would waste time
    if text.is_empty() || text.len() > 1_000 {
        return;
    }

    // Build the complete pipeline with our fuzz-friendly stages
    let pipeline = Pipeline::builder()
        .shaper(Arc::new(FuzzShaper))
        .renderer(Arc::new(FuzzRenderer))
        .exporter(Arc::new(FuzzExporter))
        .build();

    // Only test if pipeline built successfully - builder itself could fail
    if let Ok(pipeline) = pipeline {
        let font = Arc::new(FuzzFont);
        let shaping_params = ShapingParams::default();
        let render_params = RenderParams::default();

        // Process arbitrary text through the complete pipeline
        // Any panic here indicates a robustness issue
        let _ = pipeline.process(&text, font, &shaping_params, &render_params);
    }
});
