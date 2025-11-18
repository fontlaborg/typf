# TYPF v2.0 - Text Rendering Pipeline Framework

[![CI](https://github.com/fontlaborg/typf/workflows/CI/badge.svg)](https://github.com/fontlaborg/typf/actions)
[![Fuzz Testing](https://img.shields.io/badge/fuzz-3%20targets-purple.svg)](#fuzz-testing)
[![Tests](https://img.shields.io/badge/tests-113%20passing-brightgreen.svg)](#testing)
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

## License

Apache-2.0

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## Documentation

- [Architecture](ARCHITECTURE.md) - System design and pipeline details
- [Benchmarks](BENCHMARKS.md) - Performance targets, methodology, and current results
- [Security](SECURITY.md) - Security policy and vulnerability reporting
- [Release Process](RELEASE.md) - Release checklist and procedures
- [API Docs](https://docs.rs/typf) - Rust API documentation (run `cargo doc --open`)
- [Examples](examples/README.md) - Working code examples for all features