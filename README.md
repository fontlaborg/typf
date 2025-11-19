# TYPF v2.0 - Text Rendering Pipeline Framework

[![CI](https://github.com/fontlaborg/typf/workflows/CI/badge.svg)](https://github.com/fontlaborg/typf/actions)
[![Fuzz Testing](https://img.shields.io/badge/fuzz-3%20targets-purple.svg)](#fuzz-testing)
[![Tests](https://img.shields.io/badge/tests-165%20passing-brightgreen.svg)](#testing)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE-APACHE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Memory Safe](https://img.shields.io/badge/memory-profiled-blue.svg)](docs/MEMORY.md)

A modular, high-performance text rendering pipeline for Rust with professional text shaping, real font support, and SIMD optimizations.

## Features

- 🚀 **High Performance**: SIMD-optimized blending (>1GB/s throughput)
- 🌍 **Professional Text Shaping**: HarfBuzz integration for complex scripts
- 📁 **Real Font Support**: Load TrueType/OpenType fonts (including .ttc collections)
- 🔧 **Modular Architecture**: Swappable backends for shaping and rendering
- 📦 **Minimal Footprint**: <500KB minimal build size
- 🛡️ **Production Ready**: Comprehensive CI/CD with multi-platform support

## Overview

TYPF v2.0 implements a six-stage text rendering pipeline:

1. **Input Parsing** - Parse text with metadata
2. **Unicode Processing** - Script detection, bidi analysis, segmentation
3. **Font Selection** - Font matching and fallback
4. **Shaping** - Glyph shaping via pluggable backends (HarfBuzz, platform-native)
5. **Rendering** - Rasterization via pluggable backends (Orge with SIMD)
6. **Export** - Output to various formats (PNM, PNG, SVG)

## Quick Start

### Using the CLI

```bash
# Build the project
cargo build --release

# Render text to PPM
./target/release/typf "Hello World" --output hello.ppm --size 24

# Different formats
./target/release/typf "Test" --output test.pgm --format pgm
./target/release/typf "Test" --output test.pbm --format pbm
```

### Using as a Library

```rust
use std::sync::Arc;
use typf_core::{ShapingParams, RenderParams, Color};
use typf_shape_none::NoneShaper;
use typf_render_orge::OrgeRenderer;
use typf_export::PnmExporter;

// Create components
let shaper = Arc::new(NoneShaper::new());
let renderer = Arc::new(OrgeRenderer::new());
let exporter = Arc::new(PnmExporter::ppm());

// Shape text
let shaped = shaper.shape(text, font, &shaping_params)?;

// Render to bitmap
let rendered = renderer.render(&shaped, font, &render_params)?;

// Export to file
let exported = exporter.export(&rendered)?;
```

## Architecture

TYPF uses a modular architecture with swappable backends:

### Available Backends

#### Shaping Backends
- ✅ **none**: Basic left-to-right advancement (minimal)
- ✅ **harfbuzz**: Professional text shaping with complex script support
- ✅ **icu-hb**: ICU + HarfBuzz for advanced Unicode handling (bidi, normalization, segmentation)
- 🚧 **coretext**: Native macOS text shaping (planned)
- 🚧 **directwrite**: Native Windows text shaping (planned)

#### Rendering Backends
- ✅ **orge**: Built-in rasterizer with SIMD optimizations (AVX2, SSE4.1, NEON)
- 🚧 **tiny-skia**: High-quality CPU rendering (planned)
- 🚧 **skia**: GPU-accelerated rendering via Skia (planned)

#### Export Formats
- ✅ **PNM**: PPM (RGB), PGM (grayscale), PBM (monochrome)
- ✅ **PNG**: Compressed bitmap output with proper color space conversion
- ✅ **SVG**: Vector output with embedded bitmaps
- ✅ **JSON**: HarfBuzz-compatible shaping result format
- 🚧 **PDF**: Document output (planned)

## Features

- **Minimal Build**: < 500KB binary with basic functionality
- **Selective Compilation**: Enable only the backends you need
- **Thread-Safe**: Concurrent processing with Arc/DashMap
- **Zero-Copy**: Memory-mapped font loading
- **Cache-Aware**: Multi-level caching for performance

## Building

### Minimal Build

```bash
cargo build --release --no-default-features --features minimal
```

### Full Build

```bash
cargo build --release --all-features
```

## Examples

TYPF includes 9 working examples demonstrating different features:

### Quick Reference

| Example | Description | Run Command |
|---------|-------------|-------------|
| **simple** | Basic pipeline usage | `cargo run --example simple` |
| **minimal** | Minimal build demonstration | `cargo run --example minimal --no-default-features --features minimal` |
| **backend_comparison** | Compare shaping backends | `cargo run --example backend_comparison` |
| **variable_fonts** | Variable font axis control | `cargo run --example variable_fonts` |
| **svg_export_example** | Vector SVG output | `cargo run --example svg_export_example --features export-svg` |
| **all_formats** | Export to all formats | `cargo run --example all_formats` |
| **long_text_handling** | Handle bitmap width limits | `cargo run --example long_text_handling --features shaping-hb,export-svg` |

### Python Examples

```bash
# Simple rendering
python bindings/python/examples/simple_render.py

# Advanced features
python bindings/python/examples/advanced_render.py

# Long text handling
python bindings/python/examples/long_text_handling.py
```

See [examples/README.md](examples/README.md) for detailed documentation of all examples.

## Testing

```bash
# Run all tests
cargo test --workspace --all-features

# Run with coverage
cargo tarpaulin --workspace --all-features
```

## Project Structure

```
typf/
├── crates/
│   ├── typf/           # Main library crate
│   ├── typf-core/      # Core types and traits
│   ├── typf-unicode/   # Unicode processing
│   ├── typf-export/    # Export formats
│   └── typf-cli/       # Command-line interface
├── backends/
│   ├── typf-shape-none/   # Null shaper
│   └── typf-render-orge/  # Orge renderer
└── tests/              # Integration tests
```

## Current Status

### Completed Features
- ✅ Core pipeline framework with 6-stage architecture
- ✅ Basic shaping (none backend)
- ✅ HarfBuzz integration with complex script support (Arabic, Devanagari, Hebrew, Thai, CJK)
- ✅ ICU integration (Unicode normalization, bidi, line breaking)
- ✅ Real font loading (TrueType/OpenType with .ttc support)
- ✅ SIMD-optimized rendering (orge backend with AVX2, SSE4.1, NEON)
- ✅ Multi-format export (PNM, PNG, SVG, JSON)
- ✅ Python bindings with PyO3 and Fire CLI
- ✅ CLI with argument parsing
- ✅ Comprehensive CI/CD pipeline
- ✅ WASM build support
- ✅ 95 tests passing across all modules (unit + integration + property-based + golden)

### Performance Metrics
- **Binary Size**: ~500KB (minimal build when stripped)
- **SIMD Blending**: 12.5 GB/s (AVX2), 8.4 GB/s (SSE4.1)
- **Simple Shaping**: ~5µs/100 chars (2x faster than target)
- **Complex Shaping**: ~45µs/100 chars (HarfBuzz with Arabic)
- **Cache Hit**: ~40ns (L1 cache)
- **Platform Support**: Linux, macOS, Windows, WASM
- **Test Coverage**: Multi-platform CI with comprehensive test suite

### In Development
- 🚧 Platform backends (CoreText, DirectWrite) - requires macOS/Windows
- 🚧 Advanced font features (variable fonts, color fonts)
- 🚧 Skia and Zeno rendering backends

## Known Limitations

### Bitmap Rendering Width Limit
Bitmap renderers (Orge, Skia, Zeno) have a maximum width limit of approximately **10,000 pixels** per dimension. This affects long single-line text rendering:

- **At 48px font size**: ~200-300 characters maximum
- **At 24px font size**: ~400-600 characters maximum
- **At 12px font size**: ~800-1200 characters maximum

**Solutions for longer texts:**
1. **Use smaller font sizes** - Scale down to fit within limits
2. **Implement line wrapping** - Break text into multiple lines (see examples)
3. **Use SVG export** - Vector output has no width limits
4. **Multi-pass rendering** - Render text in chunks and composite

**Example error:**
```
RenderingFailed(InvalidDimensions { width: 10911, height: 68 })
```

For production applications handling arbitrary-length text, we recommend implementing proper line wrapping or using SVG export for scalability.

## Troubleshooting

### Build Issues

**"error: failed to resolve: use of undeclared type"**

You're building with features that aren't enabled. Solution:

```bash
# For Python bindings
cd bindings/python
maturin develop --release --features shaping-hb,export-png,export-svg

# For Rust CLI with all features
cargo build --all-features
```

**"Package not found" errors**

Ensure dependencies are in `Cargo.toml`. For minimal builds:

```bash
cargo build --no-default-features --features minimal
```

### Runtime Issues

**"Module 'typf' has no attribute 'Typf'"**

Python bindings need rebuilding after code changes:

```bash
cd bindings/python
maturin develop --release --features shaping-hb,export-png,export-svg
```

**"Invalid bitmap dimensions" error**

Text is too long for bitmap rendering (>10,000 pixel width). Solutions:

1. Use smaller font size
2. Implement line wrapping (see `examples/long_text_handling.rs`)
3. Use SVG export instead of bitmap
4. Render in chunks and composite

See `docs/PERFORMANCE.md` for detailed guidance.

**"Font not found" or "Failed to load font"**

1. Verify font file exists and path is correct
2. Check font file permissions
3. Ensure font format is supported (TrueType, OpenType)
4. For variable fonts, check axis ranges are valid

### Performance Issues

**Rendering is very slow**

This is expected for large font sizes due to O(size²) bitmap scaling:

1. Use smaller font sizes when possible (7.6x speedup from 128px → 32px)
2. Cache rendered results for repeated text
3. Use platform-native backends (CoreText, DirectWrite) when available
4. Consider SVG export for large text

See `docs/PERFORMANCE.md` for optimization strategies.

**High memory usage**

1. Reduce glyph cache size if customized
2. Release font objects when done
3. Process large batches in chunks
4. Monitor with `cargo bench` memory profiling

### Testing Issues

**"All backends show unavailable" in typfme.py**

1. Rebuild Python bindings (see above)
2. Verify installation: `python -c "import typf; print(typf.__version__)"`
3. Check available backends: `python typfme.py info`

**Benchmarks produce inconsistent results**

1. Increase iteration count: `python typfme.py bench --iterations=1000`
2. Close other applications to reduce system load
3. Run multiple times and compare results
4. Check for thermal throttling on laptops

### Getting Help

If you encounter issues not covered here:

1. **Check documentation:**
   - `typf-tester/QUICKSTART.md` - Testing tool guide
   - `docs/PERFORMANCE.md` - Performance optimization
   - `examples/README.md` - Code examples

2. **Search existing issues:** https://github.com/fontlab/typf/issues

3. **Report new issues:** Include:
   - TYPF version (`cargo --version` or `python -c "import typf; print(typf.__version__)"`)
   - Operating system and version
   - Rust version (`rustc --version`)
   - Complete error message
   - Minimal reproduction steps

4. **Community support:** https://forum.fontlab.com/

## License

Apache-2.0

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## Documentation

### Getting Started
- **[Quick Start](typf-tester/QUICKSTART.md)** - Get up and running in 5 minutes
- **[Examples](examples/README.md)** - Working code examples for all features
- **[Troubleshooting](#troubleshooting)** - Common issues and solutions (this document)

### Performance & Optimization
- **[Performance Guide](docs/PERFORMANCE.md)** - Comprehensive optimization strategies
- **[Backend Comparison](docs/BACKEND_COMPARISON.md)** - Choose the right backend for your needs
- **[Benchmarks](BENCHMARKS.md)** - Performance targets, methodology, and results

### Architecture & Development
- **[Architecture](ARCHITECTURE.md)** - System design and pipeline details
- **[Contributing](CONTRIBUTING.md)** - Development guidelines
- **[API Docs](https://docs.rs/typf)** - Rust API documentation (run `cargo doc --open`)

### Project Management
- **[Security](SECURITY.md)** - Security policy and vulnerability reporting
- **[Release Process](RELEASE.md)** - Release checklist and procedures
- **[Changelog](CHANGELOG.md)** - Release notes and version history