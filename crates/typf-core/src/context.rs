//! The traveling container that carries data through pipeline stages

use crate::{
    traits::{Exporter, FontRef, Renderer, Shaper},
    types::{RenderOutput, ShapingResult},
    RenderParams, ShapingParams,
};
use std::sync::Arc;

/// Everything a stage needs, nothing it doesn't
///
/// The context flows from stage to stage, accumulating the results
/// of each transformation. Text becomes glyphs, glyphs become pixels,
/// and pixels become files - all tracked here.
pub struct PipelineContext {
    // What we start with
    text: String,
    font_spec: String,

    // Who does the work
    shaper: Option<Arc<dyn Shaper>>,
    renderer: Option<Arc<dyn Renderer>>,
    exporter: Option<Arc<dyn Exporter>>,

    // What emerges along the way
    font: Option<Arc<dyn FontRef>>,
    shaped: Option<ShapingResult>,
    output: Option<RenderOutput>,
    exported: Option<Vec<u8>>,

    // How we want it done
    shaping_params: ShapingParams,
    render_params: RenderParams,
}

impl PipelineContext {
    /// Start fresh with text and a font specification
    pub fn new(text: String, font_spec: String) -> Self {
        Self {
            text,
            font_spec,
            shaper: None,
            renderer: None,
            exporter: None,
            font: None,
            shaped: None,
            output: None,
            exported: None,
            shaping_params: ShapingParams::default(),
            render_params: RenderParams::default(),
        }
    }

    // Read what's inside

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn font_spec(&self) -> &str {
        &self.font_spec
    }

    pub fn shaper(&self) -> Option<Arc<dyn Shaper>> {
        self.shaper.clone()
    }

    pub fn renderer(&self) -> Option<Arc<dyn Renderer>> {
        self.renderer.clone()
    }

    pub fn exporter(&self) -> Option<Arc<dyn Exporter>> {
        self.exporter.clone()
    }

    pub fn font(&self) -> Option<Arc<dyn FontRef>> {
        self.font.clone()
    }

    pub fn shaped(&self) -> Option<&ShapingResult> {
        self.shaped.as_ref()
    }

    pub fn output(&self) -> Option<&RenderOutput> {
        self.output.as_ref()
    }

    pub fn exported(&self) -> Option<&Vec<u8>> {
        self.exported.as_ref()
    }

    pub fn shaping_params(&self) -> &ShapingParams {
        &self.shaping_params
    }

    pub fn render_params(&self) -> &RenderParams {
        &self.render_params
    }

    // Change what's inside

    pub fn set_shaper(&mut self, shaper: Arc<dyn Shaper>) {
        self.shaper = Some(shaper);
    }

    pub fn set_renderer(&mut self, renderer: Arc<dyn Renderer>) {
        self.renderer = Some(renderer);
    }

    pub fn set_exporter(&mut self, exporter: Arc<dyn Exporter>) {
        self.exporter = Some(exporter);
    }

    pub fn set_font(&mut self, font: Arc<dyn FontRef>) {
        self.font = Some(font);
    }

    pub fn set_shaped(&mut self, shaped: ShapingResult) {
        self.shaped = Some(shaped);
    }

    pub fn set_output(&mut self, output: RenderOutput) {
        self.output = Some(output);
    }

    pub fn set_exported(&mut self, exported: Vec<u8>) {
        self.exported = Some(exported);
    }

    pub fn set_shaping_params(&mut self, params: ShapingParams) {
        self.shaping_params = params;
    }

    pub fn set_render_params(&mut self, params: RenderParams) {
        self.render_params = params;
    }
}
