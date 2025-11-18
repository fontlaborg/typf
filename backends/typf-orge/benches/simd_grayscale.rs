// Benchmark for SIMD grayscale downsampling
// Made by FontLab https://www.fontlab.com/

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use typf_orge::grayscale::GrayscaleLevel;

// Re-implement the scalar and SIMD versions for benchmarking
// (Since the internal functions are private, we'll benchmark via the public API)

/// Scalar implementation for benchmark comparison
fn downsample_to_grayscale_scalar(
    mono: &[u8],
    mono_width: usize,
    mono_height: usize,
    out_width: usize,
    out_height: usize,
    level: GrayscaleLevel,
) -> Vec<u8> {
    let factor = level.factor();
    let max_coverage = level.samples_per_pixel();

    let mut output = vec![0u8; out_width * out_height];

    for out_y in 0..out_height {
        for out_x in 0..out_width {
            let mut coverage = 0u32;
            let src_x = out_x * factor;
            let src_y = out_y * factor;

            for dy in 0..factor {
                for dx in 0..factor {
                    let x = src_x + dx;
                    let y = src_y + dy;

                    if x < mono_width && y < mono_height && mono[y * mono_width + x] != 0 {
                        coverage += 1;
                    }
                }
            }

            let alpha = ((coverage * 255) / max_coverage as u32) as u8;
            output[out_y * out_width + out_x] = alpha;
        }
    }

    output
}

/// Optimized implementation that allows LLVM auto-vectorization
fn downsample_to_grayscale_simd(
    mono: &[u8],
    mono_width: usize,
    _mono_height: usize,
    out_width: usize,
    out_height: usize,
    level: GrayscaleLevel,
) -> Vec<u8> {
    let factor = level.factor();
    let max_coverage = level.samples_per_pixel() as u32;
    let normalization_factor = 255.0 / max_coverage as f32;

    let mut output = vec![0u8; out_width * out_height];

    for out_y in 0..out_height {
        let src_y_base = out_y * factor;
        let out_row_start = out_y * out_width;

        for out_x in 0..out_width {
            let src_x_base = out_x * factor;
            let mut coverage = 0u32;

            // Sum coverage in factor x factor block
            // This loop structure allows LLVM to auto-vectorize
            for dy in 0..factor {
                let src_row_start = (src_y_base + dy) * mono_width;
                let row_start = src_row_start + src_x_base;

                if row_start + factor <= mono.len() {
                    // Fast path: entire row is in bounds, LLVM can vectorize this
                    for i in 0..factor {
                        coverage += mono[row_start + i] as u32;
                    }
                } else {
                    // Slow path: bounds checking required
                    for i in 0..factor {
                        let x = src_x_base + i;
                        if x < mono_width {
                            coverage += mono[src_row_start + x] as u32;
                        }
                    }
                }
            }

            let alpha = (coverage as f32 * normalization_factor).round() as u8;
            output[out_row_start + out_x] = alpha;
        }
    }
    output
}

fn bench_downsample(c: &mut Criterion) {
    let mut group = c.benchmark_group("downsample");

    // Test different grayscale levels
    for level in [
        GrayscaleLevel::Level2x2,
        GrayscaleLevel::Level4x4,
        GrayscaleLevel::Level8x8,
    ] {
        let factor = level.factor();
        let out_width = 128;
        let out_height = 128;
        let mono_width = out_width * factor;
        let mono_height = out_height * factor;

        // Create test pattern: checkerboard
        let mut mono = vec![0u8; mono_width * mono_height];
        for y in 0..mono_height {
            for x in 0..mono_width {
                mono[y * mono_width + x] = ((x + y) % 2) as u8;
            }
        }

        let level_name = match level {
            GrayscaleLevel::Level2x2 => "2x2",
            GrayscaleLevel::Level4x4 => "4x4",
            GrayscaleLevel::Level8x8 => "8x8",
        };

        group.bench_with_input(
            BenchmarkId::new("scalar", level_name),
            &level,
            |b, &level| {
                b.iter(|| {
                    downsample_to_grayscale_scalar(
                        black_box(&mono),
                        mono_width,
                        mono_height,
                        out_width,
                        out_height,
                        level,
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("simd", level_name),
            &level,
            |b, &level| {
                b.iter(|| {
                    downsample_to_grayscale_simd(
                        black_box(&mono),
                        mono_width,
                        mono_height,
                        out_width,
                        out_height,
                        level,
                    )
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_downsample);
criterion_main!(benches);
