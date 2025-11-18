//! Comprehensive benchmarks for TYPF v2.0
//!
//! Measures performance across the entire pipeline:
//! - Text shaping performance
//! - Rendering throughput
//! - Cache hit rates
//! - SIMD optimization effectiveness
//! - End-to-end pipeline latency

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::sync::Arc;
use typf_core::{
    ShapingParams, RenderParams, Color,
    types::Direction,
    cache::{CacheManager, ShapingCacheKey, GlyphCacheKey},
};
use typf_fontdb::Font;
use typf_shape_none::NoneShaper;
#[cfg(feature = "shaping-hb")]
use typf_shape_hb::HarfBuzzShaper;
use typf_render_orge::OrgeRenderer;
use typf_export::PnmExporter;
use typf_core::traits::{Shaper, Renderer, Exporter};

/// Benchmark shaping performance with different backends
fn bench_shaping(c: &mut Criterion) {
    let mut group = c.benchmark_group("shaping");

    // Test data
    let texts = vec![
        ("short", "Hello"),
        ("medium", "The quick brown fox jumps over the lazy dog"),
        ("long", "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."),
        ("unicode", "Hello 世界 مرحبا עולם"),
    ];

    // Create mock font
    struct MockFont;
    impl typf_core::traits::FontRef for MockFont {
        fn data(&self) -> &[u8] { &[] }
        fn units_per_em(&self) -> u16 { 1000 }
        fn glyph_id(&self, _ch: char) -> Option<u32> { Some(42) }
        fn advance_width(&self, _glyph_id: u32) -> f32 { 500.0 }
    }
    let font = Arc::new(MockFont);

    let params = ShapingParams {
        size: 16.0,
        direction: Direction::LeftToRight,
        ..Default::default()
    };

    // Benchmark NoneShaper
    let none_shaper = NoneShaper::new();
    for (name, text) in &texts {
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("none", name),
            text,
            |b, text| {
                b.iter(|| {
                    none_shaper.shape(black_box(text), font.clone(), &params)
                });
            },
        );
    }

    // Benchmark HarfBuzz if available
    #[cfg(feature = "shaping-hb")]
    {
        let hb_shaper = HarfBuzzShaper::new();
        for (name, text) in &texts {
            group.throughput(Throughput::Bytes(text.len() as u64));
            group.bench_with_input(
                BenchmarkId::new("harfbuzz", name),
                text,
                |b, text| {
                    b.iter(|| {
                        hb_shaper.shape(black_box(text), font.clone(), &params)
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark rendering performance
fn bench_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering");

    // Create mock shaped results
    let glyph_counts = vec![
        ("10_glyphs", 10),
        ("100_glyphs", 100),
        ("1000_glyphs", 1000),
    ];

    let renderer = OrgeRenderer::new();

    struct MockFont;
    impl typf_core::traits::FontRef for MockFont {
        fn data(&self) -> &[u8] { &[] }
        fn units_per_em(&self) -> u16 { 1000 }
        fn glyph_id(&self, _ch: char) -> Option<u32> { Some(42) }
        fn advance_width(&self, _glyph_id: u32) -> f32 { 500.0 }
    }
    let font = Arc::new(MockFont);

    let params = RenderParams {
        foreground: Color::rgba(0, 0, 0, 255),
        background: Some(Color::rgba(255, 255, 255, 255)),
        padding: 10,
        ..Default::default()
    };

    for (name, count) in glyph_counts {
        let shaped = typf_core::types::ShapingResult {
            glyphs: (0..count)
                .map(|i| typf_core::types::PositionedGlyph {
                    id: 42,
                    x: (i * 10) as f32,
                    y: 0.0,
                    advance: 10.0,
                    cluster: i as u32,
                })
                .collect(),
            advance_width: (count * 10) as f32,
            advance_height: 20.0,
            direction: Direction::LeftToRight,
        };

        group.throughput(Throughput::Elements(count as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                renderer.render(black_box(&shaped), font.clone(), &params)
            });
        });
    }

    group.finish();
}

/// Benchmark cache performance
fn bench_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache");

    let cache_manager = CacheManager::new();

    // Benchmark cache insertion
    group.bench_function("insert_shaping", |b| {
        let mut i = 0u64;
        b.iter(|| {
            i += 1;
            let key = ShapingCacheKey {
                text_hash: i,
                font_id: "test".to_string(),
                params_hash: i,
            };
            let data = Arc::new(vec![0u8; 1024]);
            cache_manager.cache_shaped(key, data);
        });
    });

    // Benchmark cache retrieval
    // First, populate the cache
    for i in 0..1000 {
        let key = ShapingCacheKey {
            text_hash: i,
            font_id: "test".to_string(),
            params_hash: i,
        };
        let data = Arc::new(vec![0u8; 1024]);
        cache_manager.cache_shaped(key, data);
    }

    group.bench_function("get_shaping_hit", |b| {
        b.iter(|| {
            let key = ShapingCacheKey {
                text_hash: 500,
                font_id: "test".to_string(),
                params_hash: 500,
            };
            cache_manager.get_shaped(black_box(&key))
        });
    });

    group.bench_function("get_shaping_miss", |b| {
        b.iter(|| {
            let key = ShapingCacheKey {
                text_hash: 10000,
                font_id: "test".to_string(),
                params_hash: 10000,
            };
            cache_manager.get_shaped(black_box(&key))
        });
    });

    group.finish();
}

/// Benchmark SIMD optimizations
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
fn bench_simd_blending(c: &mut Criterion) {
    use typf_render_orge::simd;

    let mut group = c.benchmark_group("simd_blending");

    let sizes = vec![
        ("small", 256),
        ("medium", 1024),
        ("large", 4096),
        ("huge", 16384),
    ];

    for (name, size) in sizes {
        let mut dst = vec![100u8; size * 4]; // RGBA
        let src = vec![200u8; size * 4];

        group.throughput(Throughput::Bytes((size * 4) as u64));

        // Benchmark scalar implementation
        group.bench_function(
            BenchmarkId::new("scalar", name),
            |b| {
                b.iter(|| {
                    simd::blend_over_scalar(black_box(&mut dst), black_box(&src));
                });
            },
        );

        // Benchmark optimized implementation
        group.bench_function(
            BenchmarkId::new("simd", name),
            |b| {
                b.iter(|| {
                    simd::blend_over(black_box(&mut dst), black_box(&src));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark end-to-end pipeline
fn bench_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline");

    // Load a real font if available, otherwise use mock
    let font = if let Ok(f) = Font::from_file("/System/Library/Fonts/Helvetica.ttc") {
        Arc::new(f) as Arc<dyn typf_core::traits::FontRef>
    } else {
        struct MockFont;
        impl typf_core::traits::FontRef for MockFont {
            fn data(&self) -> &[u8] { &[] }
            fn units_per_em(&self) -> u16 { 1000 }
            fn glyph_id(&self, _ch: char) -> Option<u32> { Some(42) }
            fn advance_width(&self, _glyph_id: u32) -> f32 { 500.0 }
        }
        Arc::new(MockFont) as Arc<dyn typf_core::traits::FontRef>
    };

    let shaper = Arc::new(NoneShaper::new());
    let renderer = Arc::new(OrgeRenderer::new());
    let exporter = Arc::new(PnmExporter::ppm());

    let texts = vec![
        ("hello", "Hello, World!"),
        ("paragraph", "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs."),
    ];

    for (name, text) in texts {
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                // Shape
                let shaped = shaper.shape(
                    black_box(text),
                    font.clone(),
                    &ShapingParams::default(),
                ).unwrap();

                // Render
                let rendered = renderer.render(
                    &shaped,
                    font.clone(),
                    &RenderParams::default(),
                ).unwrap();

                // Export
                let _exported = exporter.export(&rendered).unwrap();
            });
        });
    }

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    // Benchmark Arc cloning vs copying
    let data = vec![0u8; 1024 * 1024]; // 1MB
    let arc_data = Arc::new(data.clone());

    group.bench_function("arc_clone", |b| {
        b.iter(|| {
            black_box(arc_data.clone())
        });
    });

    group.bench_function("vec_clone", |b| {
        b.iter(|| {
            black_box(data.clone())
        });
    });

    group.finish();
}

// Configure benchmarks
criterion_group!(
    benches,
    bench_shaping,
    bench_rendering,
    bench_cache,
    bench_pipeline,
    bench_memory
);

// Add SIMD benchmarks only on supported platforms
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
criterion_group!(
    simd_benches,
    bench_simd_blending
);

// Main benchmark runner
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
criterion_main!(benches, simd_benches);

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
criterion_main!(benches);