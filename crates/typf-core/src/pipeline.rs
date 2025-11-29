//! The engine that drives text through six stages to become images

use crate::{
    context::PipelineContext,
    error::{Result, TypfError},
    traits::{Exporter, FontRef, Renderer, Shaper, Stage},
    RenderParams, ShapingParams,
};
use std::sync::Arc;

/// Six stages, one purpose: turn text into rendered output
///
/// The Pipeline orchestrates text's journey through:
/// 1. Input Parsing - Raw text becomes structured data
/// 2. Unicode Processing - Scripts normalize, bidi resolves
/// 3. Font Selection - The right font finds each character
/// 4. Shaping - Characters transform into positioned glyphs
/// 5. Rendering - Glyphs become pixels or vectors
/// 6. Export - Final output emerges as files
///
/// ```ignore
/// use typf_core::Pipeline;
///
/// let pipeline = Pipeline::builder()
///     .shaper(my_shaper)
///     .renderer(my_renderer)
///     .exporter(my_exporter)
///     .build()?;
///
/// let result = pipeline.process(
///     "Hello, world!",
///     font,
///     &shaping_params,
///     &render_params,
/// )?;
/// ```
pub struct Pipeline {
    stages: Vec<Box<dyn Stage>>,
    shaper: Option<Arc<dyn Shaper>>,
    renderer: Option<Arc<dyn Renderer>>,
    exporter: Option<Arc<dyn Exporter>>,
}

impl Pipeline {
    /// Start building a new pipeline
    pub fn builder() -> PipelineBuilder {
        PipelineBuilder::new()
    }

    /// Send text through all six stages and get the final bytes
    pub fn process(
        &self,
        text: &str,
        font: Arc<dyn FontRef>,
        shaping_params: &ShapingParams,
        render_params: &RenderParams,
    ) -> Result<Vec<u8>> {
        // Grab the three essential backends
        let shaper = self
            .shaper
            .as_ref()
            .ok_or_else(|| TypfError::ConfigError("No shaper configured".into()))?;
        let renderer = self
            .renderer
            .as_ref()
            .ok_or_else(|| TypfError::ConfigError("No renderer configured".into()))?;
        let exporter = self
            .exporter
            .as_ref()
            .ok_or_else(|| TypfError::ConfigError("No exporter configured".into()))?;

        // Shape → Render → Export
        let shaped = shaper.shape(text, font.clone(), shaping_params)?;
        let rendered = renderer.render(&shaped, font, render_params)?;
        let exported = exporter.export(&rendered)?;

        Ok(exported)
    }

    /// Run the full pipeline with a prepared context
    pub fn execute(&self, mut context: PipelineContext) -> Result<PipelineContext> {
        // Backends need to know where they live
        if let Some(shaper) = &self.shaper {
            context.set_shaper(shaper.clone());
        }
        if let Some(renderer) = &self.renderer {
            context.set_renderer(renderer.clone());
        }
        if let Some(exporter) = &self.exporter {
            context.set_exporter(exporter.clone());
        }

        // One stage at a time, each transforms the context
        for stage in &self.stages {
            log::debug!("Executing stage: {}", stage.name());
            context = stage.process(context)?;
        }

        Ok(context)
    }
}

/// Build pipelines your way, piece by piece
///
/// Chain together the backends you need, customize stages, or use defaults.
/// The builder pattern makes configuration clear and explicit.
///
/// ```ignore
/// use typf_core::Pipeline;
///
/// // Quick start with defaults
/// let pipeline = Pipeline::builder()
///     .shaper(Arc::new(HarfBuzzShaper::new()))
///     .renderer(Arc::new(OpixaRenderer::new()))
///     .exporter(Arc::new(PnmExporter::new(PnmFormat::Ppm)))
///     .build()?;
///
/// // Full control with custom stages
/// let pipeline = Pipeline::builder()
///     .stage(Box::new(CustomInputStage))
///     .shaper(my_shaper)
///     .renderer(my_renderer)
///     .build()?;
/// ```
pub struct PipelineBuilder {
    stages: Vec<Box<dyn Stage>>,
    shaper: Option<Arc<dyn Shaper>>,
    renderer: Option<Arc<dyn Renderer>>,
    exporter: Option<Arc<dyn Exporter>>,
}

impl PipelineBuilder {
    /// Start with a clean slate
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
            shaper: None,
            renderer: None,
            exporter: None,
        }
    }

    /// Add your own stage to the flow
    pub fn stage(mut self, stage: Box<dyn Stage>) -> Self {
        self.stages.push(stage);
        self
    }

    /// Choose who turns characters into glyphs
    pub fn shaper(mut self, shaper: Arc<dyn Shaper>) -> Self {
        self.shaper = Some(shaper);
        self
    }

    /// Choose who turns glyphs into pixels
    pub fn renderer(mut self, renderer: Arc<dyn Renderer>) -> Self {
        self.renderer = Some(renderer);
        self
    }

    /// Choose who packages the final output
    pub fn exporter(mut self, exporter: Arc<dyn Exporter>) -> Self {
        self.exporter = Some(exporter);
        self
    }

    /// Create the pipeline, ready to run
    pub fn build(self) -> Result<Pipeline> {
        // No custom stages? Use the classic six
        let stages = if self.stages.is_empty() {
            vec![
                Box::new(InputParsingStage) as Box<dyn Stage>,
                Box::new(UnicodeProcessingStage) as Box<dyn Stage>,
                Box::new(FontSelectionStage) as Box<dyn Stage>,
                Box::new(ShapingStage) as Box<dyn Stage>,
                Box::new(RenderingStage) as Box<dyn Stage>,
                Box::new(ExportStage) as Box<dyn Stage>,
            ]
        } else {
            self.stages
        };

        Ok(Pipeline {
            stages,
            shaper: self.shaper,
            renderer: self.renderer,
            exporter: self.exporter,
        })
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// The six stages that make up the default pipeline

struct InputParsingStage;
impl Stage for InputParsingStage {
    fn name(&self) -> &'static str {
        "InputParsing"
    }

    fn process(&self, context: PipelineContext) -> Result<PipelineContext> {
        log::trace!("Parsing input");
        // TODO: Validate and structure the raw input
        Ok(context)
    }
}

struct UnicodeProcessingStage;
impl Stage for UnicodeProcessingStage {
    fn name(&self) -> &'static str {
        "UnicodeProcessing"
    }

    fn process(&self, context: PipelineContext) -> Result<PipelineContext> {
        log::trace!("Processing Unicode");
        // TODO: Normalize text, resolve bidi, segment by script
        Ok(context)
    }
}

struct FontSelectionStage;
impl Stage for FontSelectionStage {
    fn name(&self) -> &'static str {
        "FontSelection"
    }

    fn process(&self, context: PipelineContext) -> Result<PipelineContext> {
        log::trace!("Selecting font");
        // TODO: Match characters to fonts, handle fallbacks
        Ok(context)
    }
}

struct ShapingStage;
impl Stage for ShapingStage {
    fn name(&self) -> &'static str {
        "Shaping"
    }

    fn process(&self, mut context: PipelineContext) -> Result<PipelineContext> {
        let shaper = context
            .shaper()
            .ok_or_else(|| TypfError::Pipeline("No shaper configured".into()))?;

        let font = context
            .font()
            .ok_or_else(|| TypfError::Pipeline("No font selected".into()))?;

        let text = context.text();
        let params = context.shaping_params();

        log::debug!("Shaping text with backend: {}", shaper.name());
        let shaped = shaper.shape(text, font, params)?;

        context.set_shaped(shaped);
        Ok(context)
    }
}

struct RenderingStage;
impl Stage for RenderingStage {
    fn name(&self) -> &'static str {
        "Rendering"
    }

    fn process(&self, mut context: PipelineContext) -> Result<PipelineContext> {
        let renderer = context
            .renderer()
            .ok_or_else(|| TypfError::Pipeline("No renderer configured".into()))?;

        let shaped = context
            .shaped()
            .ok_or_else(|| TypfError::Pipeline("No shaped result available".into()))?;

        let font = context
            .font()
            .ok_or_else(|| TypfError::Pipeline("No font available".into()))?;

        let params = context.render_params();

        log::debug!("Rendering with backend: {}", renderer.name());
        let output = renderer.render(shaped, font, params)?;

        context.set_output(output);
        Ok(context)
    }
}

struct ExportStage;
impl Stage for ExportStage {
    fn name(&self) -> &'static str {
        "Export"
    }

    fn process(&self, mut context: PipelineContext) -> Result<PipelineContext> {
        if let Some(exporter) = context.exporter() {
            let output = context
                .output()
                .ok_or_else(|| TypfError::Pipeline("No render output available".into()))?;

            log::debug!("Exporting with backend: {}", exporter.name());
            let exported = exporter.export(output)?;

            context.set_exported(exported);
        }

        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        BitmapData, BitmapFormat, Direction, PositionedGlyph, RenderOutput, ShapingResult,
    };
    use std::sync::Arc;

    // Mock implementations for testing
    struct MockShaper;
    impl Shaper for MockShaper {
        fn name(&self) -> &'static str {
            "MockShaper"
        }
        fn shape(
            &self,
            text: &str,
            _font: Arc<dyn FontRef>,
            params: &ShapingParams,
        ) -> Result<ShapingResult> {
            Ok(ShapingResult {
                glyphs: text
                    .chars()
                    .enumerate()
                    .map(|(i, c)| PositionedGlyph {
                        id: c as u32,
                        x: i as f32 * 10.0,
                        y: 0.0,
                        advance: 10.0,
                        cluster: i as u32,
                    })
                    .collect(),
                advance_width: text.len() as f32 * 10.0,
                advance_height: params.size,
                direction: Direction::LeftToRight,
            })
        }
    }

    struct MockRenderer;
    impl Renderer for MockRenderer {
        fn name(&self) -> &'static str {
            "MockRenderer"
        }
        fn render(
            &self,
            shaped: &ShapingResult,
            _font: Arc<dyn FontRef>,
            _params: &RenderParams,
        ) -> Result<RenderOutput> {
            let width = shaped.advance_width as u32 + 1;
            let height = shaped.advance_height as u32 + 1;
            Ok(RenderOutput::Bitmap(BitmapData {
                width,
                height,
                format: BitmapFormat::Rgba8,
                data: vec![0u8; (width * height * 4) as usize],
            }))
        }
        fn supports_format(&self, _format: &str) -> bool {
            true
        }
    }

    struct MockExporter;
    impl Exporter for MockExporter {
        fn name(&self) -> &'static str {
            "MockExporter"
        }
        fn export(&self, output: &RenderOutput) -> Result<Vec<u8>> {
            match output {
                RenderOutput::Bitmap(bitmap) => Ok(bitmap.data.clone()),
                _ => Ok(vec![]),
            }
        }
        fn extension(&self) -> &'static str {
            "bin"
        }
        fn mime_type(&self) -> &'static str {
            "application/octet-stream"
        }
    }

    struct MockFont;
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
        fn advance_width(&self, _glyph_id: u32) -> f32 {
            500.0
        }
    }

    #[test]
    fn test_pipeline_builder() {
        let pipeline = Pipeline::builder()
            .shaper(Arc::new(MockShaper))
            .renderer(Arc::new(MockRenderer))
            .exporter(Arc::new(MockExporter))
            .build();

        assert!(pipeline.is_ok());
    }

    #[test]
    fn test_pipeline_process() {
        let pipeline = Pipeline::builder()
            .shaper(Arc::new(MockShaper))
            .renderer(Arc::new(MockRenderer))
            .exporter(Arc::new(MockExporter))
            .build()
            .unwrap();

        let font = Arc::new(MockFont);
        let shaping_params = ShapingParams::default();
        let render_params = RenderParams::default();

        let result = pipeline.process("Hello", font, &shaping_params, &render_params);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_pipeline_missing_shaper() {
        let pipeline = Pipeline::builder()
            .renderer(Arc::new(MockRenderer))
            .exporter(Arc::new(MockExporter))
            .build()
            .unwrap();

        let font = Arc::new(MockFont);
        let shaping_params = ShapingParams::default();
        let render_params = RenderParams::default();

        let result = pipeline.process("Hello", font, &shaping_params, &render_params);
        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_missing_renderer() {
        let pipeline = Pipeline::builder()
            .shaper(Arc::new(MockShaper))
            .exporter(Arc::new(MockExporter))
            .build()
            .unwrap();

        let font = Arc::new(MockFont);
        let shaping_params = ShapingParams::default();
        let render_params = RenderParams::default();

        let result = pipeline.process("Hello", font, &shaping_params, &render_params);
        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_missing_exporter() {
        let pipeline = Pipeline::builder()
            .shaper(Arc::new(MockShaper))
            .renderer(Arc::new(MockRenderer))
            .build()
            .unwrap();

        let font = Arc::new(MockFont);
        let shaping_params = ShapingParams::default();
        let render_params = RenderParams::default();

        let result = pipeline.process("Hello", font, &shaping_params, &render_params);
        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_execute_with_context() {
        let pipeline = Pipeline::builder()
            .shaper(Arc::new(MockShaper))
            .renderer(Arc::new(MockRenderer))
            .exporter(Arc::new(MockExporter))
            .build()
            .unwrap();

        let font = Arc::new(MockFont);
        let mut context = PipelineContext::new("Test".to_string(), "test.ttf".to_string());
        context.set_font(font);

        let result = pipeline.execute(context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_six_stage_pipeline() {
        let pipeline = Pipeline::builder()
            .shaper(Arc::new(MockShaper))
            .renderer(Arc::new(MockRenderer))
            .exporter(Arc::new(MockExporter))
            .build()
            .unwrap();

        // Verify all 6 stages are created
        assert_eq!(pipeline.stages.len(), 6);
    }

    #[test]
    fn test_pipeline_stage_names() {
        let pipeline = Pipeline::builder().build().unwrap();

        let expected_stages = vec![
            "InputParsing",
            "UnicodeProcessing",
            "FontSelection",
            "Shaping",
            "Rendering",
            "Export",
        ];

        for (i, expected_name) in expected_stages.iter().enumerate() {
            assert_eq!(pipeline.stages[i].name(), *expected_name);
        }
    }

    #[test]
    fn test_pipeline_empty_text() {
        let pipeline = Pipeline::builder()
            .shaper(Arc::new(MockShaper))
            .renderer(Arc::new(MockRenderer))
            .exporter(Arc::new(MockExporter))
            .build()
            .unwrap();

        let font = Arc::new(MockFont);
        let shaping_params = ShapingParams::default();
        let render_params = RenderParams::default();

        let result = pipeline.process("", font, &shaping_params, &render_params);
        assert!(result.is_ok());
    }
}
