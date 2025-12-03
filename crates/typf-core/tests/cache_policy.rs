use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use typf_core::traits::{FontRef, Renderer, Shaper, Stage};
use typf_core::{
    types::{BitmapData, BitmapFormat, PositionedGlyph, RenderOutput, ShapingResult},
    Pipeline, RenderParams, ShapingParams,
};

struct TestFont;

impl FontRef for TestFont {
    fn data(&self) -> &[u8] {
        b"font"
    }

    fn units_per_em(&self) -> u16 {
        1000
    }

    fn glyph_id(&self, _ch: char) -> Option<u32> {
        Some(1)
    }

    fn advance_width(&self, _glyph_id: u32) -> f32 {
        500.0
    }
}

struct CountingShaper {
    calls: Arc<AtomicUsize>,
}

impl CountingShaper {
    fn new(calls: Arc<AtomicUsize>) -> Self {
        Self { calls }
    }
}

impl Stage for CountingShaper {
    fn name(&self) -> &'static str {
        "counting-shaper"
    }

    fn process(
        &self,
        ctx: typf_core::PipelineContext,
    ) -> typf_core::Result<typf_core::PipelineContext> {
        Ok(ctx)
    }
}

impl Shaper for CountingShaper {
    fn name(&self) -> &'static str {
        "counting-shaper"
    }

    fn shape(
        &self,
        _text: &str,
        _font: Arc<dyn FontRef>,
        params: &ShapingParams,
    ) -> typf_core::Result<ShapingResult> {
        self.calls.fetch_add(1, Ordering::SeqCst);

        Ok(ShapingResult {
            glyphs: vec![PositionedGlyph {
                id: 1,
                x: 0.0,
                y: 0.0,
                advance: 10.0,
                cluster: 0,
            }],
            advance_width: 10.0,
            advance_height: params.size,
            direction: params.direction,
        })
    }
}

struct CountingRenderer {
    calls: Arc<AtomicUsize>,
}

impl CountingRenderer {
    fn new(calls: Arc<AtomicUsize>) -> Self {
        Self { calls }
    }
}

impl Stage for CountingRenderer {
    fn name(&self) -> &'static str {
        "counting-renderer"
    }

    fn process(
        &self,
        ctx: typf_core::PipelineContext,
    ) -> typf_core::Result<typf_core::PipelineContext> {
        Ok(ctx)
    }
}

impl Renderer for CountingRenderer {
    fn name(&self) -> &'static str {
        "counting-renderer"
    }

    fn render(
        &self,
        shaped: &ShapingResult,
        _font: Arc<dyn FontRef>,
        _params: &RenderParams,
    ) -> typf_core::Result<RenderOutput> {
        self.calls.fetch_add(1, Ordering::SeqCst);

        Ok(RenderOutput::Bitmap(BitmapData {
            width: 1,
            height: 1,
            format: BitmapFormat::Gray8,
            data: vec![shaped.glyphs.len() as u8],
        }))
    }
}

fn build_pipeline(shaper: Arc<dyn Shaper>, renderer: Arc<dyn Renderer>) -> Pipeline {
    Pipeline::builder()
        .shaper(shaper)
        .renderer(renderer)
        .exporter(Arc::new(DummyExporter))
        .build()
        .expect("pipeline build")
}

struct DummyExporter;

impl Stage for DummyExporter {
    fn name(&self) -> &'static str {
        "dummy-exporter"
    }

    fn process(
        &self,
        ctx: typf_core::PipelineContext,
    ) -> typf_core::Result<typf_core::PipelineContext> {
        Ok(ctx)
    }
}

impl typf_core::traits::Exporter for DummyExporter {
    fn name(&self) -> &'static str {
        "dummy-exporter"
    }

    fn export(&self, output: &RenderOutput) -> typf_core::Result<Vec<u8>> {
        match output {
            RenderOutput::Bitmap(bmp) => Ok(bmp.data.clone()),
            RenderOutput::Vector(v) => Ok(v.data.as_bytes().to_vec()),
            RenderOutput::Json(s) => Ok(s.as_bytes().to_vec()),
        }
    }

    fn extension(&self) -> &'static str {
        "bin"
    }

    fn mime_type(&self) -> &'static str {
        "application/octet-stream"
    }
}

#[test]
fn caches_hit_when_enabled() {
    let shaper_calls = Arc::new(AtomicUsize::new(0));
    let renderer_calls = Arc::new(AtomicUsize::new(0));

    let pipeline = build_pipeline(
        Arc::new(CountingShaper::new(shaper_calls.clone())),
        Arc::new(CountingRenderer::new(renderer_calls.clone())),
    );

    let font: Arc<dyn FontRef> = Arc::new(TestFont);
    let shaping = ShapingParams::default();
    let render = RenderParams::default();

    let _ = pipeline
        .process("hello", font.clone(), &shaping, &render)
        .unwrap();
    let _ = pipeline.process("hello", font, &shaping, &render).unwrap();

    assert_eq!(
        1,
        shaper_calls.load(Ordering::SeqCst),
        "shaper should hit cache"
    );
    assert_eq!(
        1,
        renderer_calls.load(Ordering::SeqCst),
        "renderer should hit cache"
    );
}

#[test]
fn caches_can_be_disabled() {
    let shaper_calls = Arc::new(AtomicUsize::new(0));
    let renderer_calls = Arc::new(AtomicUsize::new(0));

    let pipeline = Pipeline::builder()
        .enable_shaping_cache(false)
        .enable_glyph_cache(false)
        .shaper(Arc::new(CountingShaper::new(shaper_calls.clone())))
        .renderer(Arc::new(CountingRenderer::new(renderer_calls.clone())))
        .exporter(Arc::new(DummyExporter))
        .build()
        .expect("pipeline build");

    let font: Arc<dyn FontRef> = Arc::new(TestFont);
    let shaping = ShapingParams::default();
    let render = RenderParams::default();

    let _ = pipeline
        .process("hello", font.clone(), &shaping, &render)
        .unwrap();
    let _ = pipeline.process("hello", font, &shaping, &render).unwrap();

    assert_eq!(2, shaper_calls.load(Ordering::SeqCst), "shaper cache off");
    assert_eq!(
        2,
        renderer_calls.load(Ordering::SeqCst),
        "renderer cache off"
    );
}
