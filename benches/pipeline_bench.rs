//! Benchmark for the TYPF pipeline

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;

use typf::{Color, Pipeline, RenderParams, ShapingParams};
use typf_core::traits::{FontRef, Renderer, Shaper};
use typf_export::{PnmExporter, PnmFormat};
use typf_render_orge::OrgeRenderer;
use typf_shape_none::NoneShaper;

struct BenchFont;

impl FontRef for BenchFont {
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
        500.0
    }
}

fn bench_shaping(c: &mut Criterion) {
    let font = Arc::new(BenchFont);
    let shaper = NoneShaper::new();
    let params = ShapingParams::default();

    c.bench_function("shape_short_text", |b| {
        b.iter(|| {
            let text = black_box("Hello World");
            shaper.shape(text, font.clone(), &params).unwrap()
        })
    });

    c.bench_function("shape_medium_text", |b| {
        let text = "The quick brown fox jumps over the lazy dog. ".repeat(5);
        b.iter(|| {
            shaper
                .shape(black_box(&text), font.clone(), &params)
                .unwrap()
        })
    });

    c.bench_function("shape_long_text", |b| {
        let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(20);
        b.iter(|| {
            shaper
                .shape(black_box(&text), font.clone(), &params)
                .unwrap()
        })
    });
}

fn bench_rendering(c: &mut Criterion) {
    let font = Arc::new(BenchFont);
    let shaper = NoneShaper::new();
    let renderer = OrgeRenderer::new();

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

    c.bench_function("render_short_text", |b| {
        b.iter(|| {
            renderer
                .render(&shaped_short, font.clone(), &params)
                .unwrap()
        })
    });

    c.bench_function("render_long_text", |b| {
        b.iter(|| {
            renderer
                .render(&shaped_long, font.clone(), &params)
                .unwrap()
        })
    });
}

fn bench_full_pipeline(c: &mut Criterion) {
    let pipeline = Pipeline::builder()
        .shaper(Arc::new(NoneShaper::new()))
        .renderer(Arc::new(OrgeRenderer::new()))
        .exporter(Arc::new(PnmExporter::ppm()))
        .build()
        .unwrap();

    let font = Arc::new(BenchFont);
    let shaping_params = ShapingParams::default();
    let render_params = RenderParams {
        foreground: Color::black(),
        background: Some(Color::white()),
        padding: 5,
        antialias: false,
    };

    c.bench_function("full_pipeline_short", |b| {
        let text = "Hello TYPF!";
        b.iter(|| {
            pipeline
                .process(
                    black_box(text),
                    font.clone(),
                    &shaping_params,
                    &render_params,
                )
                .unwrap()
        })
    });

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
