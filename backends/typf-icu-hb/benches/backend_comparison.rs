// this_file: backends/typf-icu-hb/benches/backend_comparison.rs

//! Backend rendering performance benchmarks
//!
//! Compares orge vs tiny-skia rendering performance for:
//! - Monochrome rendering (48pt simple glyph)
//! - Grayscale 2x2, 4x4, 8x8 rendering
//! - Variable font rendering
//!
//! Performance targets:
//! - Monochrome: <100μs per glyph
//! - Grayscale 4x4: <500μs per glyph
//! - orge within ±15% of tiny-skia

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use kurbo::{BezPath, PathEl, Point};

#[cfg(feature = "orge")]
use typf_icu_hb::renderer::OrgeRenderer;
use typf_icu_hb::renderer::GlyphRenderer;
#[cfg(feature = "tiny-skia-renderer")]
use typf_icu_hb::renderer::TinySkiaRenderer;

/// Create a simple rectangular path for benchmarking
fn create_rectangle_path(width: f64, height: f64) -> BezPath {
    let mut path = BezPath::new();
    path.push(PathEl::MoveTo(Point::new(0.0, 0.0)));
    path.push(PathEl::LineTo(Point::new(width, 0.0)));
    path.push(PathEl::LineTo(Point::new(width, height)));
    path.push(PathEl::LineTo(Point::new(0.0, height)));
    path.push(PathEl::ClosePath);
    path
}

/// Create a more complex path with curves (simulates glyph 'e')
fn create_curved_path() -> BezPath {
    let mut path = BezPath::new();

    // Outer contour
    path.push(PathEl::MoveTo(Point::new(10.0, 0.0)));
    path.push(PathEl::QuadTo(
        Point::new(40.0, 0.0),
        Point::new(50.0, 20.0),
    ));
    path.push(PathEl::QuadTo(
        Point::new(50.0, 40.0),
        Point::new(30.0, 50.0),
    ));
    path.push(PathEl::QuadTo(
        Point::new(10.0, 50.0),
        Point::new(5.0, 30.0),
    ));
    path.push(PathEl::QuadTo(Point::new(5.0, 10.0), Point::new(10.0, 0.0)));
    path.push(PathEl::ClosePath);

    // Inner counter (hole)
    path.push(PathEl::MoveTo(Point::new(20.0, 15.0)));
    path.push(PathEl::QuadTo(
        Point::new(35.0, 15.0),
        Point::new(38.0, 25.0),
    ));
    path.push(PathEl::QuadTo(
        Point::new(38.0, 35.0),
        Point::new(28.0, 38.0),
    ));
    path.push(PathEl::QuadTo(
        Point::new(18.0, 38.0),
        Point::new(15.0, 28.0),
    ));
    path.push(PathEl::QuadTo(
        Point::new(15.0, 18.0),
        Point::new(20.0, 15.0),
    ));
    path.push(PathEl::ClosePath);

    path
}

#[cfg(feature = "tiny-skia-renderer")]
fn bench_tiny_skia_monochrome(c: &mut Criterion) {
    let renderer = TinySkiaRenderer::new();
    let path = create_rectangle_path(48.0, 48.0);

    c.bench_function("tiny-skia/monochrome/simple", |b| {
        b.iter(|| {
            renderer.render_glyph(
                black_box(&path),
                black_box(64),
                black_box(64),
                black_box(false),
            )
        })
    });
}

#[cfg(feature = "tiny-skia-renderer")]
fn bench_tiny_skia_grayscale(c: &mut Criterion) {
    let renderer = TinySkiaRenderer::new();
    let path = create_rectangle_path(48.0, 48.0);

    c.bench_function("tiny-skia/grayscale/simple", |b| {
        b.iter(|| {
            renderer.render_glyph(
                black_box(&path),
                black_box(64),
                black_box(64),
                black_box(true),
            )
        })
    });
}

#[cfg(feature = "tiny-skia-renderer")]
fn bench_tiny_skia_complex(c: &mut Criterion) {
    let renderer = TinySkiaRenderer::new();
    let path = create_curved_path();

    c.bench_function("tiny-skia/grayscale/complex", |b| {
        b.iter(|| {
            renderer.render_glyph(
                black_box(&path),
                black_box(64),
                black_box(64),
                black_box(true),
            )
        })
    });
}

#[cfg(feature = "orge")]
fn bench_orge_monochrome(c: &mut Criterion) {
    let renderer = OrgeRenderer::new();
    let path = create_rectangle_path(48.0, 48.0);

    c.bench_function("orge/monochrome/simple", |b| {
        b.iter(|| {
            renderer.render_glyph(
                black_box(&path),
                black_box(64),
                black_box(64),
                black_box(false),
            )
        })
    });
}

#[cfg(feature = "orge")]
fn bench_orge_grayscale(c: &mut Criterion) {
    let renderer = OrgeRenderer::new();
    let path = create_rectangle_path(48.0, 48.0);

    c.bench_function("orge/grayscale/simple", |b| {
        b.iter(|| {
            renderer.render_glyph(
                black_box(&path),
                black_box(64),
                black_box(64),
                black_box(true),
            )
        })
    });
}

#[cfg(feature = "orge")]
fn bench_orge_complex(c: &mut Criterion) {
    let renderer = OrgeRenderer::new();
    let path = create_curved_path();

    c.bench_function("orge/grayscale/complex", |b| {
        b.iter(|| {
            renderer.render_glyph(
                black_box(&path),
                black_box(64),
                black_box(64),
                black_box(true),
            )
        })
    });
}

// Comparison benchmark group
fn bench_backend_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("backend_comparison");
    let path = create_curved_path();

    #[cfg(feature = "tiny-skia-renderer")]
    {
        let renderer = TinySkiaRenderer::new();
        group.bench_with_input(
            BenchmarkId::new("grayscale", "tiny-skia"),
            &path,
            |b, path| b.iter(|| renderer.render_glyph(black_box(path), 64, 64, true)),
        );
    }

    #[cfg(feature = "orge")]
    {
        let renderer = OrgeRenderer::new();
        group.bench_with_input(BenchmarkId::new("grayscale", "orge"), &path, |b, path| {
            b.iter(|| renderer.render_glyph(black_box(path), 64, 64, true))
        });
    }

    group.finish();
}

// Configure Criterion groups
#[cfg(feature = "tiny-skia-renderer")]
criterion_group!(
    tiny_skia_benches,
    bench_tiny_skia_monochrome,
    bench_tiny_skia_grayscale,
    bench_tiny_skia_complex
);

#[cfg(feature = "orge")]
criterion_group!(
    orge_benches,
    bench_orge_monochrome,
    bench_orge_grayscale,
    bench_orge_complex
);

criterion_group!(comparison_benches, bench_backend_comparison);

// Main entry point
#[cfg(all(feature = "tiny-skia-renderer", feature = "orge"))]
criterion_main!(tiny_skia_benches, orge_benches, comparison_benches);

#[cfg(all(feature = "tiny-skia-renderer", not(feature = "orge")))]
criterion_main!(tiny_skia_benches, comparison_benches);

#[cfg(all(feature = "orge", not(feature = "tiny-skia-renderer")))]
criterion_main!(orge_benches, comparison_benches);

#[cfg(not(any(feature = "tiny-skia-renderer", feature = "orge")))]
fn main() {
    eprintln!("Error: No renderer feature enabled!");
    eprintln!("Enable either 'tiny-skia-renderer' or 'orge' feature to run benchmarks.");
    std::process::exit(1);
}
