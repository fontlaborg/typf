# TYPF v2.0 Documentation

## TLDR - Too Long Didn't Read

**TYPF v2.0** is a high-performance, modular text shaping and rendering library written in Rust with Python bindings. It features a **six-stage pipeline** (Input → Unicode → Font Selection → Shaping → Rendering → Export) with **multiple backends** for each stage, enabling cross-platform text processing with exceptional performance.

**Key Features:**
- 🚀 **Blazing Fast**: SIMD-accelerated rendering with >10 GB/s throughput
- 🔧 **Modular Architecture**: Pluggable backends for shaping (HarfBuzz, CoreText, DirectWrite) and rendering (Skia, CoreGraphics, Orge)
- 🐍 **First-class Python Support**: PyO3 bindings with seamless Rust integration
- 📦 **Selective Builds**: Feature flags enable minimal builds (<500KB) or full-featured releases
- 🌐 **Cross-Platform**: macOS, Windows, Linux, and WebAssembly support
- 🎯 **Production Ready**: Comprehensive testing, fuzzing, and property-based validation

**Quick Start:**
```bash
# Install and build
./build.sh

# Use CLI
typf-cli render --text "Hello 世界" --font path/to/font.ttf

# Use Python
python -m typf render --text "Hello 世界" --font path/to/font.ttf
```

**Performance:** ~50ns per glyph with SIMD acceleration, multi-level caching, and zero-copy font loading.

---

# Table of Contents

## Part I: Introduction (Chapters 1-4)
- [01 - Introduction](01-introduction.md) - What is TYPF and why it exists
- [02 - Quick Start](02-quick-start.md) - Get up and running in minutes
- [03 - Architecture Overview](03-architecture-overview.md) - High-level system design
- [04 - Installation](04-installation.md) - Detailed setup instructions

## Part II: Core Concepts (Chapters 5-8)
- [05 - The Six-Stage Pipeline](05-six-stage-pipeline.md) - Understanding the data flow
- [06 - Backend Architecture](06-backend-architecture.md) - Shaping and rendering backends
- [07 - Memory Management](07-memory-management.md) - Efficient font caching and zero-copy
- [08 - Performance Fundamentals](08-performance-fundamentals.md) - SIMD, caching, and optimization

## Part III: Shaping Backends (Chapters 9-12)
- [09 - HarfBuzz Shaping](09-harfbuzz-shaping.md) - Cross-platform Unicode shaping
- [10 - Platform Shapers](10-platform-shapers.md) - CoreText and DirectWrite integration
- [11 - ICU-HarfBuzz Composition](11-icu-harfbuzz-composition.md) - Advanced text processing
- [12 - None Shaper](12-none-shaper.md) - Pass-through for testing

## Part IV: Rendering Backends (Chapters 13-16)
- [13 - Skia Rendering](13-skia-rendering.md) - Vector graphics rendering
- [14 - Platform Renderers](14-platform-renderers.md) - CoreGraphics and Direct2D
- [15 - Orge Rasterizer](15-orge-rasterizer.md) - Pure Rust rasterization
- [16 - Zeno Renderer](16-zeno-renderer.md) - Experimental GPU rendering

## Part V: API Reference (Chapters 17-20)
- [17 - Rust API](17-rust-api.md) - Core Rust library usage
- [18 - Python Bindings](18-python-bindings.md) - PyO3 Python interface
- [19 - CLI Reference](19-cli-reference.md) - Command-line tool documentation
- [20 - Configuration Options](20-configuration-options.md) - Feature flags and settings

## Part VI: Advanced Topics (Chapters 21-24)
- [21 - Font Handling](21-font-handling.md) - Font loading and selection
- [22 - Export Formats](22-export-formats.md) - PNG, SVG, PNM, and JSON output
- [23 - Testing and Validation](23-testing-and-validation.md) - Test strategies and quality assurance
- [24 - Contributing Guidelines](24-contributing-guidelines.md) - How to contribute to TYPF

---

## Navigation

This documentation is organized to take you from beginner to advanced usage:

1. **Start with Part I** if you're new to TYPF
2. **Move to Part II** to understand core concepts
3. **Skip to Part III or IV** for specific backend information
4. **Reference Part V** for API details
5. **Explore Part VI** for advanced usage and contributions

## Code Examples

Throughout this documentation, you'll find:

- 🚀 **Performance benchmarks** showcasing TYPF's speed
- 💻 **Code snippets** in Rust and Python
- 📊 **Architecture diagrams** explaining data flow
- 🔧 **Configuration examples** for different use cases
- 🧪 **Test examples** demonstrating validation

## Getting Help

- 📖 **Check this documentation** first
- 🐛 **File issues** on GitHub for bugs
- 💬 **Join discussions** for questions
- 📧 **Contact maintainers** for support

---

**TYPF v2.0**: The future of high-performance text shaping and rendering.
