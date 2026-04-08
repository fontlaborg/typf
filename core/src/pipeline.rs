//! Pipeline orchestration for shaping, rendering, and export.
//!
//! [`Pipeline::process`] is the direct execution path: it calls the configured
//! shaper, renderer, and exporter in sequence. [`Pipeline::execute`] runs the
//! explicit stage list stored in a [`PipelineContext`]. In the default pipeline,
//! the first three stages are placeholders reserved for future preprocessing and
//! font-selection work.

use crate::{
    context::PipelineContext,
    error::{Result, TypfError},
    glyph_cache::{GlyphCache, GlyphCacheKey, SharedGlyphCache},
    shaping_cache::{ShapingCache, ShapingCacheKey, SharedShapingCache},
    traits::{Exporter, FontRef, Renderer, Shaper, Stage},
    RenderParams, ShapingParams,
};
use std::sync::{Arc, RwLock};

/// Pipeline for text shaping, rendering, and export.
///
/// Use [`Pipeline::process`] for the normal fast path. Use
/// [`Pipeline::execute`] when you need the explicit stage list and a prepared
/// [`PipelineContext`]. In the default configuration, only the shaping,
/// rendering, and export stages do work; the earlier stages are placeholders.
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
    #[allow(dead_code)]
    cache_policy: CachePolicy,
    #[allow(dead_code)]
    shaping_cache: Option<SharedShapingCache>,
    #[allow(dead_code)]
    glyph_cache: Option<SharedGlyphCache>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CachePolicy {
    pub shaping: bool,
    pub glyph: bool,
}

impl Pipeline {
    pub fn builder() -> PipelineBuilder {
        PipelineBuilder::new()
    }

    /// Run the configured shaper, renderer, and exporter directly.
    pub fn process(
        &self,
        text: &str,
        font: Arc<dyn FontRef>,
        shaping_params: &ShapingParams,
        render_params: &RenderParams,
    ) -> Result<Vec<u8>> {
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

        let shaped = shaper.shape(text, font.clone(), shaping_params)?;
        let rendered = renderer.render(&shaped, font, render_params)?;
        let exported = exporter.export(&rendered)?;

        Ok(exported)
    }

    pub fn execute(&self, mut context: PipelineContext) -> Result<PipelineContext> {
        if let Some(shaper) = &self.shaper {
            context.set_shaper(shaper.clone());
        }
        if let Some(renderer) = &self.renderer {
            context.set_renderer(renderer.clone());
        }
        if let Some(exporter) = &self.exporter {
            context.set_exporter(exporter.clone());
        }

        for stage in &self.stages {
            log::debug!("Executing stage: {}", stage.name());
            context = stage.process(context)?;
        }

        Ok(context)
    }
}

/// Builder for configuring a pipeline.
///
/// Use this to choose shaping, rendering, and export backends, or to replace
/// the default stage list with custom stages.
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
    cache_policy: CachePolicy,
    shaping_cache: Option<SharedShapingCache>,
    glyph_cache: Option<SharedGlyphCache>,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
            shaper: None,
            renderer: None,
            exporter: None,
            cache_policy: CachePolicy::default(),
            shaping_cache: None,
            glyph_cache: None,
        }
    }

    /// Add a custom stage to the explicit stage list.
    pub fn stage(mut self, stage: Box<dyn Stage>) -> Self {
        self.stages.push(stage);
        self
    }

    /// Set the shaper backend.
    pub fn shaper(mut self, shaper: Arc<dyn Shaper>) -> Self {
        self.shaper = Some(shaper);
        self
    }

    /// Set the renderer backend.
    pub fn renderer(mut self, renderer: Arc<dyn Renderer>) -> Self {
        self.renderer = Some(renderer);
        self
    }

    /// Set the exporter backend.
    pub fn exporter(mut self, exporter: Arc<dyn Exporter>) -> Self {
        self.exporter = Some(exporter);
        self
    }

    pub fn enable_shaping_cache(mut self, enabled: bool) -> Self {
        self.cache_policy.shaping = enabled;
        self
    }

    pub fn enable_glyph_cache(mut self, enabled: bool) -> Self {
        self.cache_policy.glyph = enabled;
        self
    }

    pub fn with_shaping_cache(mut self, cache: SharedShapingCache) -> Self {
        self.shaping_cache = Some(cache);
        self
    }

    pub fn with_glyph_cache(mut self, cache: SharedGlyphCache) -> Self {
        self.glyph_cache = Some(cache);
        self
    }

    /// Build the pipeline from the configured parts.
    pub fn build(self) -> Result<Pipeline> {
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

        let shaping_cache = if self.cache_policy.shaping {
            Some(
                self.shaping_cache
                    .unwrap_or_else(|| Arc::new(RwLock::new(ShapingCache::new()))),
            )
        } else {
            None
        };

        let glyph_cache = if self.cache_policy.glyph {
            Some(
                self.glyph_cache
                    .unwrap_or_else(|| Arc::new(RwLock::new(GlyphCache::new()))),
            )
        } else {
            None
        };

        let shaper = match (self.shaper, shaping_cache.as_ref()) {
            (Some(shaper), Some(cache)) => {
                Some(Arc::new(CachedShaper::new(shaper, cache.clone())) as Arc<dyn Shaper>)
            },
            (Some(shaper), None) => Some(shaper),
            (None, _) => None,
        };

        let renderer = match (self.renderer, glyph_cache.as_ref()) {
            (Some(renderer), Some(cache)) => {
                Some(Arc::new(CachedRenderer::new(renderer, cache.clone())) as Arc<dyn Renderer>)
            },
            (Some(renderer), None) => Some(renderer),
            (None, _) => None,
        };

        Ok(Pipeline {
            stages,
            shaper,
            renderer,
            exporter: self.exporter,
            cache_policy: self.cache_policy,
            shaping_cache,
            glyph_cache,
        })
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

struct InputParsingStage;
impl Stage for InputParsingStage {
    fn name(&self) -> &'static str {
        "InputParsing"
    }

    fn process(&self, context: PipelineContext) -> Result<PipelineContext> {
        log::trace!("InputParsing: pass-through (reserved for future use)");
        Ok(context)
    }
}

struct UnicodeProcessingStage;
impl Stage for UnicodeProcessingStage {
    fn name(&self) -> &'static str {
        "UnicodeProcessing"
    }

    fn process(&self, context: PipelineContext) -> Result<PipelineContext> {
        log::trace!("UnicodeProcessing: pass-through (reserved for future use)");
        Ok(context)
    }
}

struct FontSelectionStage;
impl Stage for FontSelectionStage {
    fn name(&self) -> &'static str {
        "FontSelection"
    }

    fn process(&self, context: PipelineContext) -> Result<PipelineContext> {
        log::trace!("FontSelection: pass-through (reserved for future use)");
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

struct CachedShaper {
    inner: Arc<dyn Shaper>,
    cache: SharedShapingCache,
}

impl CachedShaper {
    fn new(inner: Arc<dyn Shaper>, cache: SharedShapingCache) -> Self {
        Self { inner, cache }
    }
}

impl Shaper for CachedShaper {
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn shape(
        &self,
        text: &str,
        font: Arc<dyn FontRef>,
        params: &ShapingParams,
    ) -> Result<crate::types::ShapingResult> {
        let key = ShapingCacheKey::new(
            text,
            self.inner.name(),
            font.data(),
            params.size,
            params.language.clone(),
            params.script.clone(),
            params.features.clone(),
            params.variations.clone(),
        );

        if let Ok(cache) = self.cache.read() {
            if let Some(hit) = cache.get(&key) {
                return Ok(hit);
            }
        }

        let shaped = self.inner.shape(text, font, params)?;

        if let Ok(cache) = self.cache.write() {
            cache.insert(key, shaped.clone());
        }

        Ok(shaped)
    }
}

struct CachedRenderer {
    inner: Arc<dyn Renderer>,
    cache: SharedGlyphCache,
}

impl CachedRenderer {
    fn new(inner: Arc<dyn Renderer>, cache: SharedGlyphCache) -> Self {
        Self { inner, cache }
    }
}

impl Renderer for CachedRenderer {
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn render(
        &self,
        shaped: &crate::types::ShapingResult,
        font: Arc<dyn FontRef>,
        params: &RenderParams,
    ) -> Result<crate::types::RenderOutput> {
        let key = GlyphCacheKey::new(self.inner.name(), font.data(), shaped, params);

        if let Ok(cache) = self.cache.read() {
            if let Some(hit) = cache.get(&key) {
                return Ok(hit);
            }
        }

        let rendered = self.inner.render(shaped, font, params)?;

        if let Ok(cache) = self.cache.write() {
            cache.insert(key, rendered.clone());
        }

        Ok(rendered)
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
        let pipeline_result = Pipeline::builder()
            .shaper(Arc::new(MockShaper))
            .renderer(Arc::new(MockRenderer))
            .exporter(Arc::new(MockExporter))
            .build();
        let pipeline = match pipeline_result {
            Ok(pipeline) => pipeline,
            Err(e) => {
                unreachable!("pipeline build failed: {e}");
            },
        };

        let font = Arc::new(MockFont);
        let shaping_params = ShapingParams::default();
        let render_params = RenderParams::default();

        let result = pipeline.process("Hello", font, &shaping_params, &render_params);
        match result {
            Ok(bytes) => assert!(!bytes.is_empty()),
            Err(e) => unreachable!("pipeline process failed: {e}"),
        }
    }

    #[test]
    fn test_pipeline_missing_shaper() {
        let pipeline_result = Pipeline::builder()
            .renderer(Arc::new(MockRenderer))
            .exporter(Arc::new(MockExporter))
            .build();
        let pipeline = match pipeline_result {
            Ok(pipeline) => pipeline,
            Err(e) => {
                unreachable!("pipeline build failed: {e}");
            },
        };

        let font = Arc::new(MockFont);
        let shaping_params = ShapingParams::default();
        let render_params = RenderParams::default();

        let result = pipeline.process("Hello", font, &shaping_params, &render_params);
        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_missing_renderer() {
        let pipeline_result = Pipeline::builder()
            .shaper(Arc::new(MockShaper))
            .exporter(Arc::new(MockExporter))
            .build();
        let pipeline = match pipeline_result {
            Ok(pipeline) => pipeline,
            Err(e) => {
                unreachable!("pipeline build failed: {e}");
            },
        };

        let font = Arc::new(MockFont);
        let shaping_params = ShapingParams::default();
        let render_params = RenderParams::default();

        let result = pipeline.process("Hello", font, &shaping_params, &render_params);
        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_missing_exporter() {
        let pipeline_result = Pipeline::builder()
            .shaper(Arc::new(MockShaper))
            .renderer(Arc::new(MockRenderer))
            .build();
        let pipeline = match pipeline_result {
            Ok(pipeline) => pipeline,
            Err(e) => {
                unreachable!("pipeline build failed: {e}");
            },
        };

        let font = Arc::new(MockFont);
        let shaping_params = ShapingParams::default();
        let render_params = RenderParams::default();

        let result = pipeline.process("Hello", font, &shaping_params, &render_params);
        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_execute_with_context() {
        let pipeline_result = Pipeline::builder()
            .shaper(Arc::new(MockShaper))
            .renderer(Arc::new(MockRenderer))
            .exporter(Arc::new(MockExporter))
            .build();
        let pipeline = match pipeline_result {
            Ok(pipeline) => pipeline,
            Err(e) => {
                unreachable!("pipeline build failed: {e}");
            },
        };

        let font = Arc::new(MockFont);
        let mut context = PipelineContext::new("Test".to_string(), "test.ttf".to_string());
        context.set_font(font);

        let result = pipeline.execute(context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_six_stage_pipeline() {
        let pipeline_result = Pipeline::builder()
            .shaper(Arc::new(MockShaper))
            .renderer(Arc::new(MockRenderer))
            .exporter(Arc::new(MockExporter))
            .build();
        let pipeline = match pipeline_result {
            Ok(pipeline) => pipeline,
            Err(e) => {
                unreachable!("pipeline build failed: {e}");
            },
        };

        // Verify all 6 stages are created
        assert_eq!(pipeline.stages.len(), 6);
    }

    #[test]
    fn test_pipeline_stage_names() {
        let pipeline_result = Pipeline::builder().build();
        let pipeline = match pipeline_result {
            Ok(pipeline) => pipeline,
            Err(e) => {
                unreachable!("pipeline build failed: {e}");
            },
        };

        let expected_stages = [
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
        let pipeline_result = Pipeline::builder()
            .shaper(Arc::new(MockShaper))
            .renderer(Arc::new(MockRenderer))
            .exporter(Arc::new(MockExporter))
            .build();
        let pipeline = match pipeline_result {
            Ok(pipeline) => pipeline,
            Err(e) => {
                unreachable!("pipeline build failed: {e}");
            },
        };

        let font = Arc::new(MockFont);
        let shaping_params = ShapingParams::default();
        let render_params = RenderParams::default();

        let result = pipeline.process("", font, &shaping_params, &render_params);
        assert!(result.is_ok());
    }
}
