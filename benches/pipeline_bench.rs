//! Pipeline performance testing - measure every stage of text rendering
//!
//! This benchmark suite focuses on the Pipeline builder pattern and how it performs
//! with different text lengths. Unlike comprehensive.rs, this targets the high-level
//! API that most users will interact with, measuring realistic usage patterns.
//!
//! Use these benchmarks to:
//! - Validate Pipeline performance after API changes
//! - Compare stage-by-stage performance vs end-to-end
//! - Test scaling with text length and complexity
//! - Measure the overhead of the builder pattern

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;

use typf::{Color, Pipeline, RenderParams, ShapingParams};
use typf_core::traits::{FontRef, Renderer, Shaper};
use typf_export::{PnmExporter, PnmFormat};
use typf_render_opixa::OpixaRenderer;
use typf_shape_none::NoneShaper;

/// Consistent test font for all benchmarks
struct BenchFont;

impl FontRef for BenchFont {
    fn data(&self) -> &[u8] {
        &[] // No actual font data needed for benchmarking
    }
    fn units_per_em(&self) -> u16 {
        1000 // Standard font coordinate space
    }
    fn glyph_id(&self, ch: char) -> Option<u32> {
        Some(ch as u32) // Direct character-to-glyph mapping
    }
    fn advance_width(&self, _: u32) -> f32 {
        500.0 // Consistent glyph width
    }
}

/// Measure text shaping performance across different lengths
///
/// Shaping transforms characters into positioned glyphs. This benchmark
/// isolates the shaping stage to measure how well it scales with text length.
/// We test short labels, medium paragraphs, and long documents.
fn bench_shaping(c: &mut Criterion) {
    let font = Arc::new(BenchFont);
    let shaper = NoneShaper::new();
    let params = ShapingParams::default();

    // Short UI elements - should be essentially instant
    c.bench_function("shape_short_text", |b| {
        b.iter(|| {
            let text = black_box("Hello World");
            shaper.shape(text, font.clone(), &params).unwrap()
        })
    });

    // Medium paragraphs - typical UI text
    c.bench_function("shape_medium_text", |b| {
        let text = "The quick brown fox jumps over the lazy dog. ".repeat(5);
        b.iter(|| {
            shaper
                .shape(black_box(&text), font.clone(), &params)
                .unwrap()
        })
    });

    // Long documents - stress test scalability
    c.bench_function("shape_long_text", |b| {
        let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(20);
        b.iter(|| {
            shaper
                .shape(black_box(&text), font.clone(), &params)
                .unwrap()
        })
    });
}

/// Measure rasterization speed - glyphs to pixels
///
/// Rendering is where the computational heavy lifting happens. This benchmark
/// tests how the renderer performs with different glyph counts, from a few
/// characters to long paragraphs. Performance here directly impacts UI
/// responsiveness.
fn bench_rendering(c: &mut Criterion) {
    let font = Arc::new(BenchFont);
    let shaper = NoneShaper::new();
    let renderer = OpixaRenderer::new();

    // Shape test data once to isolate rendering performance
    let shaped_short = shaper
        .shape("Hello", font.clone(), &ShapingParams::default())
        .unwrap();
    let shaped_long = shaper
        .shape(
            &"Test text for rendering benchmark. ".repeat(10),
            font.clone(),
            &ShapingParams::default(),
        )
        .unwrap();

    let params = RenderParams::default();

    // Render short text - UI labels and buttons
    c.bench_function("render_short_text", |b| {
        b.iter(|| {
            renderer
                .render(&shaped_short, font.clone(), &params)
                .unwrap()
        })
    });

    // Render long text - paragraphs and documents
    c.bench_function("render_long_text", |b| {
        b.iter(|| {
            renderer
                .render(&shaped_long, font.clone(), &params)
                .unwrap()
        })
    });
}

/// End-to-end pipeline performance - what users actually see
///
/// This is the most important benchmark - it measures the complete text-to-image
/// process including shaping, rendering, and export. The Pipeline builder pattern
/// adds some overhead for flexibility, so we need to ensure it's still fast.
fn bench_full_pipeline(c: &mut Criterion) {
    // Build a complete pipeline with all stages
    let pipeline = Pipeline::builder()
        .shaper(Arc::new(NoneShaper::new()))
        .renderer(Arc::new(OpixaRenderer::new()))
        .exporter(Arc::new(PnmExporter::ppm()))
        .build()
        .unwrap();

    let font = Arc::new(BenchFont);
    let shaping_params = ShapingParams::default();
    let render_params = RenderParams {
        foreground: Color::black(),
        background: Some(Color::white()),
        padding: 5,
        antialias: false, // Faster for benchmarking
        variations: Vec::new(),
    };

    // Complete pipeline for short text - common UI scenarios
    c.bench_function("full_pipeline_short", |b| {
        let text = "Hello TYPF!";
        b.iter(|| {
            pipeline
                .process(black_box(text), font.clone(), &shaping_params, &render_params)
                .unwrap()
        })
    });

    // Complete pipeline for paragraph text - body content
    c.bench_function("full_pipeline_paragraph", |b| {
        let text = "The quick brown fox jumps over the lazy dog. This is a test of the TYPF text rendering pipeline. ".repeat(3);
        b.iter(|| {
            pipeline.process(
                black_box(&text),
                font.clone(),
                &shaping_params,
                &render_params,
            ).unwrap()
        })
    });
}

criterion_group!(benches, bench_shaping, bench_rendering, bench_full_pipeline);
criterion_main!(benches);
