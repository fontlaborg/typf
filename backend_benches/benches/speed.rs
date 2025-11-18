// this_file: backend_benches/benches/speed.rs
//
// Benchmarks the default rendering backend (platform-specific).
// For multi-backend comparison, use the Python toy.py render command.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use typf_api::{Session, SessionBuilder};
use typf_core::types::Font;

const SAMPLE_TEXT: &str = "The quick brown fox jumps over the lazy dog.";
const FONT_SIZE: f32 = 48.0;

fn setup_session() -> Session {
    let font = Font::new("Arial", FONT_SIZE);
    SessionBuilder::new(font).build()
}

fn bench_rendering(c: &mut Criterion) {
    let session = setup_session();

    c.bench_function("render_monochrome", |b| {
        b.iter(|| {
            session.render(
                black_box(SAMPLE_TEXT),
                black_box(FONT_SIZE),
                black_box(None),
                black_box(false),
            )
        })
    });

    c.bench_function("render_grayscale", |b| {
        b.iter(|| {
            session.render(
                black_box(SAMPLE_TEXT),
                black_box(FONT_SIZE),
                black_box(None),
                black_box(true),
            )
        })
    });
}

criterion_group!(benches, bench_rendering);
criterion_main!(benches);
