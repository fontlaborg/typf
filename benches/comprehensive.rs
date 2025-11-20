//! Push TYPF to its limits - find bottlenecks before your users do
//!
//! This benchmark suite measures every part of the rendering pipeline under stress.
//! We test text shaping speeds, rendering throughput, cache efficiency, and SIMD
//! optimizations. Run these benchmarks when you need to:
//!
//! - Validate performance after code changes
//! - Compare backend performance (NoneShaper vs HarfBuzz)
//! - Measure cache hit rates and memory patterns
//! - Verify SIMD optimizations are working
//! - Benchmark the complete end-to-end pipeline

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

/// Measure how fast TYPF converts characters to positioned glyphs
///
/// Shaping is the computational heart of text rendering. We test both the simple
/// NoneShaper and the professional HarfBuzz shaper across different text lengths
/// and complexity levels. Throughput is measured in bytes/second to normalize
/// across different text lengths.
fn bench_shaping(c: &mut Criterion) {
    let mut group = c.benchmark_group("shaping");

    // Text samples that challenge shapers differently
    let texts = vec![
        ("short", "Hello"),
        ("medium", "The quick brown fox jumps over the lazy dog"),
        ("long", "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."),
        ("unicode", "Hello 世界 مرحبا עולם"), // Mixed scripts stress Unicode handling
    ];

    // Mock font for consistent benchmarking
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

    // Test NoneShaper - the baseline performance
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

    // Test HarfBuzz when available - the professional-grade alternative
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

/// Test rendering speed - how fast glyphs become pixels
///
/// Rendering transforms positioned glyphs into actual bitmap data. This benchmark
/// measures performance scaling with glyph count using the OrgeRenderer. We use
/// synthetic glyph data to isolate rendering performance from shaping time.
fn bench_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering");

    // Different text lengths stress different aspects of the renderer
    let glyph_counts = vec![
        ("10_glyphs", 10),    // Short UI labels
        ("100_glyphs", 100),  // Paragraph text
        ("1000_glyphs", 1000), // Long documents
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

    // Create synthetic shaping results with varying glyph counts
    for (name, count) in glyph_counts {
        let shaped = typf_core::types::ShapingResult {
            glyphs: (0..count)
                .map(|i| typf_core::types::PositionedGlyph {
                    id: 42,
                    x: (i * 10) as f32, // Position glyphs horizontally
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

/// Cache performance can make or break real-world rendering speed
///
/// A good cache turns repeated text rendering from expensive to instant. This benchmark
/// tests both cache insertion speed and the critical difference between cache hits
/// and misses. In production, cache hit rates above 90% are common for repeated
/// UI elements and documents.
fn bench_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache");

    let cache_manager = CacheManager::new();

    // Test how quickly we can add new items to the cache
    group.bench_function("insert_shaping", |b| {
        let mut i = 0u64;
        b.iter(|| {
            i += 1;
            let key = ShapingCacheKey {
                text_hash: i,
                font_id: "test".to_string(),
                params_hash: i,
            };
            let data = Arc::new(vec![0u8; 1024]); // 1KB shaping result
            cache_manager.cache_shaped(key, data);
        });
    });

    // Pre-populate cache for hit rate testing - realistic scenario
    for i in 0..1000 {
        let key = ShapingCacheKey {
            text_hash: i,
            font_id: "test".to_string(),
            params_hash: i,
        };
        let data = Arc::new(vec![0u8; 1024]);
        cache_manager.cache_shaped(key, data);
    }

    // Cache hits should be essentially free - just a lookup and Arc clone
    group.bench_function("get_shaping_hit", |b| {
        b.iter(|| {
            let key = ShapingCacheKey {
                text_hash: 500, // This key exists in our pre-populated cache
                font_id: "test".to_string(),
                params_hash: 500,
            };
            cache_manager.get_shaped(black_box(&key))
        });
    });

    // Cache misses trigger the full shaping pipeline - much more expensive
    group.bench_function("get_shaping_miss", |b| {
        b.iter(|| {
            let key = ShapingCacheKey {
                text_hash: 10000, // This key doesn't exist
                font_id: "test".to_string(),
                params_hash: 10000,
            };
            cache_manager.get_shaped(black_box(&key))
        });
    });

    group.finish();
}

/// SIMD acceleration - the secret sauce for fast rendering
///
/// Modern CPUs can process 16+ bytes simultaneously using SIMD instructions.
/// This benchmark compares scalar (one-by-one) processing against SIMD
/// (many-at-once) for alpha blending operations. Good SIMD implementations
/// show 4x+ speedup on large buffers.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
fn bench_simd_blending(c: &mut Criterion) {
    use typf_render_orge::simd;

    let mut group = c.benchmark_group("simd_blending");

    // Test different buffer sizes - SIMD shines on larger data
    let sizes = vec![
        ("small", 256),      // UI elements
        ("medium", 1024),    // Small paragraphs
        ("large", 4096),     // Large text blocks
        ("huge", 16384),     // Full pages
    ];

    for (name, size) in sizes {
        let mut dst = vec![100u8; size * 4]; // RGBA destination buffer
        let src = vec![200u8; size * 4];     // RGBA source buffer

        group.throughput(Throughput::Bytes((size * 4) as u64));

        // Scalar fallback - processes one pixel at a time
        group.bench_function(
            BenchmarkId::new("scalar", name),
            |b| {
                b.iter(|| {
                    simd::blend_over_scalar(black_box(&mut dst), black_box(&src));
                });
            },
        );

        // SIMD accelerated - processes many pixels in parallel
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

/// Complete pipeline performance - what users actually experience
///
/// This benchmark measures the full text-to-image process that users see:
/// shaping → rendering → export. It uses real fonts when available for
/// realistic performance numbers. This is the benchmark that matters
/// most for user-facing performance.
fn bench_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline");

    // Try to use a real system font for authentic performance measurement
    let font = if let Ok(f) = Font::from_file("/System/Library/Fonts/Helvetica.ttc") {
        Arc::new(f) as Arc<dyn typf_core::traits::FontRef>
    } else {
        // Fall back to mock font if system font unavailable (CI environments)
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

    // Real-world text samples
    let texts = vec![
        ("hello", "Hello, World!"),
        ("paragraph", "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs."),
    ];

    for (name, text) in texts {
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                // Stage 1: Shape text into positioned glyphs
                let shaped = shaper.shape(
                    black_box(text),
                    font.clone(),
                    &ShapingParams::default(),
                ).unwrap();

                // Stage 2: Render glyphs into bitmap
                let rendered = renderer.render(
                    &shaped,
                    font.clone(),
                    &RenderParams::default(),
                ).unwrap();

                // Stage 3: Export bitmap to file format
                let _exported = exporter.export(&rendered).unwrap();
            });
        });
    }

    group.finish();
}

/// Memory efficiency matters - Arc sharing vs copying
///
/// Text rendering involves sharing large data structures (fonts, glyphs, bitmaps).
/// This benchmark shows why Arc (atomic reference counting) is preferred over
/// cloning. Arc shares the same memory; cloning creates entirely new copies.
/// The difference becomes dramatic with multi-megabyte font data.
fn bench_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    // Realistic data size - similar to a small font or large glyph cache
    let data = vec![0u8; 1024 * 1024]; // 1MB of data
    let arc_data = Arc::new(data.clone());

    // Arc cloning is cheap - just increments a counter
    group.bench_function("arc_clone", |b| {
        b.iter(|| {
            black_box(arc_data.clone())
        });
    });

    // Vec cloning is expensive - copies all the bytes
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
