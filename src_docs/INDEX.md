# Typf Documentation

Typf turns text into pixels, fast. 

> Note: Typf is a community project by [FontLab](https://www.fontlab.org/) and is currently published under an [evaluation license](https://github.com/fontlaborg/typf/blob/main/LICENSE).

## Quick Start

```bash
# Build everything
./build.sh

# Render text now
typf-cli render --text "Hello 世界" --font font.ttf

# Python version
python -m typf render --text "Hello 世界" --font font.ttf
```

## What This Is

Typf turns text into pixels, fast. Six stages: Input → Unicode → Font → Shape → Render → Export. Each stage has multiple backends. You pick what works for your system.

Performance: ~50ns per glyph with SIMD, multi-level caching, zero-copy font loading.

---

# Documentation

## Getting Started
- [01 - Introduction](01-introduction.md) - Why Typf exists
- [02 - Quick Start](02-quick-start.md) - Running in minutes
- [03 - Architecture](03-architecture-overview.md) - How it works
- [04 - Installation](04-installation.md) - Setup details

## Core System
- [05 - Pipeline](05-six-stage-pipeline.md) - The six stages explained
- [06 - Backends](06-backend-architecture.md) - Mixing and matching components
- [07 - Memory](07-memory-management.md) - Font caching without leaks
- [08 - Performance](08-performance-fundamentals.md) - Speed basics

## Shaping Text
- [09 - HarfBuzz](09-harfbuzz-shaping.md) - Cross-platform Unicode shaping
- [10 - Platform Shapers](10-platform-shapers.md) - CoreText and DirectWrite
- [11 - ICU+HarfBuzz](11-icu-harfbuzz-composition.md) - Complex text processing
- [12 - None Shaper](12-none-shaper.md) - Testing and debugging

## Rendering Pixels
- [13 - Skia](13-skia-renderer.md) - Hardware-accelerated rendering
- [14 - Platform Renderers](15-platform-renderers.md) - CoreGraphics and Direct2D
- [15 - Opixa](14-opixa-renderer.md) - Pure Rust foundation
- [16 - Zeno](16-zeno-renderer.md) - Vector quality output

## Using Typf
- [17 - Rust API](18-rust-api.md) - The core Rust library
- [18 - Python API](19-python-api.md) - Python bindings
- [19 - CLI](20-cli-interface.md) - Command-line tools
- [20 - WebAssembly](21-webassembly-integration.md) - Browser rendering

## Production
- [21 - Export Formats](17-export-formats.md) - PNG, SVG, PNM, JSON
- [22 - Performance](22-performance-optimization.md) - Production tuning
- [23 - Deployment](23-deployment-integration.md) - Production patterns
- [24 - Troubleshooting](24-troubleshooting-best-practices.md) - Problem solving
- [25 - Benchmark Analysis](25-benchmark-analysis.md) - Performance data and insights

---

## How to Read This

New to Typf? Start with Getting Started.

Understanding core concepts? Read Core System.

Need specific backend info? Jump to Shaping or Rendering sections.

Building applications? Check the Using Typf section.

Deploying to production? Read the Production section.

Each chapter builds on previous ones, but you can jump to what you need.

---

## Code Examples

Every chapter includes working code examples. Rust and Python side by side. Copy, paste, run. No toy examples—real code you can use.

## Get Help

Stuck? Check the relevant chapter first. Each section has troubleshooting examples. File GitHub issues for bugs you find.

---

Typf: Fast text rendering that works.
