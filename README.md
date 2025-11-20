# TYPF v2.0

[![CI](https://github.com/fontlaborg/typf/workflows/CI/badge.svg)](https://github.com/fontlaborg/typf/actions)
[![Tests](https://img.shields.io/badge/tests-206%20passing-brightgreen.svg)](#testing)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

Render text fast. TYPF shapes complex scripts and outputs to PNG, SVG, or JSON in under a millisecond.

## What it does

- **Fast**: SIMD rendering at >1GB/s
- **Complex scripts**: Arabic, Hindi, Thai with HarfBuzz
- **Real fonts**: TrueType, OpenType, variable fonts
- **Swappable parts**: Mix shaping + rendering backends
- **Small**: 500KB minimal build
- **Tested**: 206 tests pass

## How it works

Text flows through six stages:

1. **Input** - Parse text and options
2. **Unicode** - Detect scripts, handle RTL
3. **Fonts** - Match fonts, fallback when needed
4. **Shape** - Convert characters to glyphs
5. **Render** - Rasterize with SIMD
6. **Export** - Write PNG, SVG, or JSON

## Try it now

```bash
git clone https://github.com/fontlaborg/typf.git
cd typf
cargo build --release

./target/release/typf "Hello, World!" --output hello.ppm --size 48
open hello.ppm  # macOS/Linux
```

That's it—you've rendered text with professional shaping.

## What it looks like

**Mixed scripts**: "Hello, مرحبا, 你好!" with proper shaping and RTL support.

![Mixed Script Rendering](typf-tester/output/render-harfbuzz-orge-mixd.svg)

**Speed vs quality**: Different renderers, different trade-offs.

| CoreGraphics | Orge | Skia | Zeno |
|:---:|:---:|:---:|:---:|
| ![CG](typf-tester/output/render-harfbuzz-coregraphics-latn.png) | ![Orge](typf-tester/output/render-harfbuzz-orge-latn.png) | ![Skia](typf-tester/output/render-harfbuzz-skia-latn.png) | ![Zeno](typf-tester/output/render-harfbuzz-zeno-latn.png) |
| 0.38ms • 254 levels | 1.14ms • 98% smooth | 1.36ms • excellent AA | 0.76ms • 247 levels |

**SVG output**: Vector graphics that scale forever and render 23× faster than PNG.

![Arabic SVG](typf-tester/output/render-harfbuzz-zeno-arab.svg)

## Using it

**Command line:**

```bash
cargo build --release

./target/release/typf "Hello World" --output hello.ppm --size 24
./target/release/typf "Test" --output test.pgm --format pgm

# SVG (23× faster than PNG)
cargo build --release --features export-svg
./target/release/typf "Scalable Text" --output vector.svg --size 48
```

**Rust library:**

```rust
use std::sync::Arc;
use typf_shape_none::NoneShaper;
use typf_render_orge::OrgeRenderer;
use typf_export::PnmExporter;

let shaper = Arc::new(NoneShaper::new());
let renderer = Arc::new(OrgeRenderer::new());
let exporter = Arc::new(PnmExporter::ppm());

let shaped = shaper.shape(text, font, &params)?;
let rendered = renderer.render(&shaped, font, &render_params)?;
let exported = exporter.export(&rendered)?;
```

**Batch processing:**

```rust
use rayon::prelude::*;

let texts = vec!["Hello", "مرحبا", "你好", "Bonjour"];
let results: Vec<_> = texts
    .par_iter()
    .map(|text| shaper.shape(text, font.clone(), &params))
    .collect()?;

// Tips: Arc for thread sharing, JSON export for speed, cache font handles
```

## Speed

**Fastest combinations (macOS, Nov 2025):**

| Backend | Time | ops/sec | Best for |
|---------|------|---------|----------|
| CoreText + JSON | 0.049ms | 22,661 | Data export |
| none + JSON | 0.051ms | 21,385 | Simple text |
| HarfBuzz + JSON | 0.063ms | 17,652 | Complex scripts |
| Zeno | 0.318-0.366ms | 3,048-3,675 | Balanced speed/quality |
| Orge (pure Rust) | 1.113-1.268ms | 1,959-2,302 | No dependencies |

**What this means:**
- JSON export: 10-40× faster than rendering
- Native macOS: Fastest overall (4,000-22,000 ops/sec)
- Zeno: Best bitmap speed/quality trade-off
- All renderers: 100% success rate

**Text complexity:**

| Text | Time | ops/sec |
|------|------|---------|
| Arabic | 0.480ms | 6,807 |
| Mixed scripts | 0.421ms | 5,455 |
| Latin | 0.917ms | 6,162 |

*Platform: macOS 14, Apple Silicon. More in [typf-tester/README.md](typf-tester/README.md).*

## Test your system

```bash
cd typf-tester
python typfme.py bench

# Results: output/benchmark_report.json, output/benchmark_summary.md
python visual_diff.py --all
python unified_report.py
```

Tests all 20 backend combos, multiple scripts, performance metrics, and quality analysis.

## Architecture

**Shapers**: none (basic), harfbuzz (complex scripts), icu-hb (Unicode + bidi)

**Renderers**: orge (SIMD, pure Rust), skia (high quality), coregraphics (macOS), zeno (fast), json (data only)

**Exports**: PNM, PNG, SVG, JSON

## Choose your backends

| Need | Shaper | Renderer | Speed | Quality |
|------|--------|----------|-------|---------|
| Fastest | none | JSON | 25K ops/sec | Data only |
| Complex scripts | harfbuzz | zeno | 3K ops/sec | 247 levels |
| macOS best | coretext | coregraphics | 4K ops/sec | 254 levels (best) |
| Pure Rust | harfbuzz | orge | 2K ops/sec | 25 levels (mono) |

**Popular combos:**
- Production (macOS): harfbuzz + coregraphics
- Portable: harfbuzz + zeno
- Minimal: none + orge
- Data only: harfbuzz + JSON

## Build options

- **Minimal**: <500KB, basic features
- **Selective**: Enable only backends you need
- **Thread-safe**: Arc/DashMap for concurrency
- **Zero-copy**: Memory-mapped fonts
- **Caching**: Multi-level performance cache

## Build

```bash
# Minimal (500KB)
cargo build --release --no-default-features --features minimal

# Everything
cargo build --release --all-features
```

## Examples

| Example | What it shows | Run it |
|---------|---------------|--------|
| simple | Basic pipeline | `cargo run --example simple` |
| minimal | Smallest build | `cargo run --example minimal --no-default-features --features minimal` |
| backend_comparison | Compare shapers | `cargo run --example backend_comparison` |
| variable_fonts | Font axes | `cargo run --example variable_fonts` |
| svg_export_example | Vector output | `cargo run --example svg_export_example --features export-svg` |

**Python:**
```bash
python bindings/python/examples/simple_render.py
python bindings/python/examples/long_text_handling.py
```

## Test

```bash
cargo test --workspace --all-features
cargo tarpaulin --workspace --all-features
```

## Layout

```
typf/
├── crates/typf/          # Main library
├── crates/typf-core/     # Core types
├── crates/typf-unicode/  # Unicode handling
├── crates/typf-export/   # Export formats
├── crates/typf-cli/      # CLI tool
├── backends/             # Shaping/rendering
└── tests/                # Integration tests
```

## Status

Production ready. Everything works:

- ✅ 6-stage pipeline
- ✅ 4 shapers (None, HarfBuzz, ICU-HarfBuzz, CoreText)
- ✅ 5 renderers (Orge, Skia, Zeno, CoreGraphics, JSON)
- ✅ PNM, PNG, SVG, JSON export
- ✅ Python bindings (PyO3)
- ✅ 206 tests pass
- ✅ SIMD optimizations
- ✅ Linux, macOS, Windows, WASM

**Performance:**
- Binary: ~500KB (minimal, stripped)
- SIMD: 12.5 GB/s (AVX2), 8.4 GB/s (SSE4.1)
- Shaping: 5µs/100 chars (simple), 45µs/100 chars (complex)
- Cache: ~40ns (L1 hit)
- Throughput: 1,500-22,000 ops/sec

See [CHANGELOG.md](CHANGELOG.md) for details.

## Limits

**Bitmap width**: ~10,000 pixels max
- 48px font: ~200-300 characters
- 24px font: ~400-600 characters
- 12px font: ~800-1200 characters

**Fixes**: Smaller fonts, line wrapping, or SVG (no width limit).

## Problems

**Build errors:**
- "undeclared type" → Enable features: `cargo build --all-features`
- "Package not found" → Check Cargo.toml, or use minimal: `cargo build --no-default-features --features minimal`

**Runtime errors:**
- "no attribute 'Typf'" → Rebuild Python: `cd bindings/python && maturin develop --release --features shaping-hb,export-png,export-svg`
- "Invalid bitmap dimensions" → Text too wide (>10,000px). Use smaller font, line wrapping, or SVG
- "Font not found" → Check path, permissions, format (TrueType/OpenType)

**Slow rendering:**
- Large fonts are slow (O(size²)). Use smaller sizes, cache results, or platform-native backends
- High memory → Reduce cache size, process in chunks

**Testing:**
- "backends unavailable" → Rebuild Python bindings
- Inconsistent benchmarks → More iterations, close other apps, avoid thermal throttling

**Need help?**
- Docs: `typf-tester/QUICKSTART.md`, `examples/README.md`
- Issues: https://github.com/fontlaborg/typf/issues
- Forum: https://forum.fontlab.com/

## License

[EVALUATION LICENSE](./LICENSE)

## Contribute

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Documentation

- **[TYPF v2.0 Documentation](src_docs/)** - 24 chapters covering everything
- **[Examples](examples/README.md)** - Working code for all features
- **[Quick Start](typf-tester/QUICKSTART.md)** - 5-minute tutorial
- **[Contributing](CONTRIBUTING.md)** - Development setup
- **[API Docs](https://docs.rs/typf)** - Rust API (`cargo doc --open`)
- **[PLAN.md](PLAN.md)** - Roadmap and architecture
- **[TODO.md](TODO.md)** - Current tasks
- **[CHANGELOG.md](CHANGELOG.md)** - Release notes
- **[Features Matrix](FEATURES.md)** - Feature status
- **[Security](SECURITY.md)** - Security policy
