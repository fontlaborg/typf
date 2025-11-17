# TYPF: Modern Font Rendering Engine

> Production-ready, cross-platform font rendering with Rust performance and Python convenience

[![Status](https://img.shields.io/badge/status-production--ready-green)]()
[![Language](https://img.shields.io/badge/rust-1.70+-orange)]()
[![Python](https://img.shields.io/badge/python-3.12+-blue)]()

## What is TYPF?

TYPF is a modern, cross-platform font rendering engine providing unified text layout and rasterization. It's built in Rust for performance and safety, with Python bindings for ease of use.

**Key Design Goals:**

- **Performance:** Sub-millisecond rendering, lock-free concurrency, zero-copy font loading
- **Correctness:** Pixel-perfect output, comprehensive testing, fuzzing-ready
- **Cross-Platform:** Native backends for macOS (CoreText), Windows (DirectWrite), Linux (HarfBuzz)
- **Flexible Output:** PNG, SVG, NumPy arrays, PGM, raw bitmaps

## Features

### Core Capabilities

✅ **Multiple Shaping Backends**
- CoreText (macOS native)
- DirectWrite (Windows native)
- ICU + HarfBuzz (cross-platform fallback)
- Automatic backend selection based on platform

✅ **Multiple Rasterizers**
- `orge` - Custom CPU rasterizer (F26Dot6 fixed-point, scan conversion)
- `tiny-skia` - Vector renderer (feature-gated)
- `zeno` - Alternative rasterizer (feature-gated)

✅ **Modern Font Stack**
- Built exclusively on `skrifa` + `read-fonts`
- Full OpenType support (TTF, CFF, CFF2)
- Variable font support (wght, wdth, and all registered axes)
- Named instance support

✅ **Output Formats**
- PNG (via `resvg` + `png` crate)
- SVG paths with `kurbo`
- NumPy arrays (Python bindings)
- PGM (P5 format)
- Raw RGBA/BGRA bitmaps

✅ **Advanced Features**
- COLRv1/CPAL color font support
- Gradients and clip paths
- Complex script shaping (Arabic, Devanagari, CJK)
- Bidirectional text (via `unicode_bidi`)
- Text segmentation (ICU-based)

## Quick Links

- [Installation](getting-started/installation.md) - Get started with TYPF
- [Quick Start](getting-started/quick-start.md) - Render your first text
- [Python Bindings](getting-started/python-bindings.md) - Using TYPF from Python
- [Backend Comparison](backends/comparison.md) - Choose the right backend
- [Architecture](architecture/overview.md) - Understanding TYPF's design

## Performance

- **Rendering:** Sub-millisecond per glyph
- **Font Loading:** Zero-copy, lazy parsing
- **Concurrency:** Lock-free caching, thread-safe
- **Memory:** Minimal allocations, buffer reuse

## Quick Example

### CLI

```bash
typf render \
  --font=/path/to/font.ttf \
  --text="Hello World" \
  --size=48 \
  --output=hello.png
```

### Python

```python
import typf

# Render text to PNG
result = typf.render_text(
    font_path="/path/to/font.ttf",
    text="Hello World",
    size=48,
    output_path="hello.png"
)
```

### Rust

```rust
use typf::render::{RenderOptions, render_to_png};

let options = RenderOptions::new()
    .font_path("/path/to/font.ttf")
    .text("Hello World")
    .size(48);

render_to_png(&options, "hello.png")?;
```

## Why TYPF?

- **Fast:** Rust performance with zero-cost abstractions
- **Safe:** No segfaults, memory leaks, or data races
- **Portable:** Works on macOS, Linux, Windows
- **Flexible:** Use as library, CLI, or Python module
- **Modern:** Built on latest OpenType specs and Rust ecosystem

---

**Made by [FontLab](https://www.fontlab.com/)**
