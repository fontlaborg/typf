---
title: Quick Start
icon: lucide/play-circle
tags:
  - Quick Start
  - Installation
  - Getting Started
---

# Quick Start Guide

Get TYPF v2.0 up and running in minutes with this comprehensive quick start guide.

## Prerequisites

### System Requirements

- **Operating System**: macOS 10.15+, Windows 10+, or Linux (Ubuntu 20.04+)
- **Python**: 3.12+ (for Python bindings)
- **Rust**: 1.70+ (for building from source)
- **Memory**: 8GB RAM recommended
- **Storage**: 2GB free space

### Required Tools

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install uv (Python package manager)
pip install uv

# Install system dependencies (Linux only)
sudo apt-get update
sudo apt-get install -y libharfbuzz-dev pkg-config
```

## Installation Methods

### Method 1: Build from Source (Recommended)

Clone the repository and build everything:

```bash
# Clone TYPF
git clone https://github.com/fontlaborg/typf.git
cd typf

# Build and install (this runs the comprehensive build script)
./build.sh
```

The build script will:
- 🏗️ Build the Rust workspace with all features
- 📦 Install the `typf-cli` tool
- 🐍 Set up Python environment and install `typfpy`
- 🧪 Run the test suite and benchmarks

### Method 2: Cargo Install (CLI Only)

For just the Rust CLI tool:

```bash
cargo install typf-cli
```

### Method 3: Python Package (Python Only)

```bash
# From PyPI (when published)
pip install typfpy

# Or from source
git clone https://github.com/fontlaborg/typf.git
cd typf/bindings/python
pip install -e .
```

## Your First Text Render

### Using the CLI

```bash
# Basic text rendering
typf-cli render \
  --text "Hello, 世界!" \
  --font /System/Library/Fonts/Arial.ttf \
  --output hello.png

# With specific size and color
typf-cli render \
  --text "Beautiful Text" \
  --font path/to/font.ttf \
  --size 48 \
  --color "#FF0000" \
  --output fancy.png
```

### Using Python

```python
import typfpy

# Simple rendering
result = typfpy.render_text(
    text="Hello, 世界!",
    font_path="/System/Library/Fonts/Arial.ttf",
    size=32.0
)

# Save to file
with open("hello.png", "wb") as f:
    f.write(result.png_data)

# Get dimensions
print(f"Width: {result.width}, Height: {result.height}")
```

## Exploring Available Fonts

### System Font Detection

```bash
# List available fonts
typf-cli font-list

# Search for specific fonts
typf-cli font-search --name "Arial"

# Get font information
typf-cli font-info --font /System/Library/Fonts/Arial.ttf
```

### Python Font Discovery

```python
import typfpy

# Create font database
font_db = typfpy.FontDatabase()

# Search for fonts
arabic_fonts = font_db.find_families(scripts=["Arab"])
print(f"Found {len(arabic_fonts)} Arabic font families")

# Get font details
font_info = font_db.get_font_info("Arial")
print(f"Supported scripts: {font_info.scripts}")
```

## Backend Selection

### Available Backends

```bash
# List available backends
typf-cli backend-list

# Output:
# Shaping backends: harfbuzz, coretext, directwrite, icu-hb, none
# Rendering backends: skia, coregraphics, direct2d, orge, zeno, json
```

### Using Specific Backends

```bash
# HarfBuzz + Skia (cross-platform)
typf-cli render \
  --text "Sample text" \
  --font font.ttf \
  --shaper harfbuzz \
  --renderer skia \
  --output output.png

# CoreText + CoreGraphics (macOS only)
typf-cli render \
  --text "Sample text" \
  --font font.ttf \
  --shaper coretext \
  --renderer coregraphics \
  --output output.png
```

### Python Backend Selection

```python
import typfpy

# Use specific backends
with typfpy.Typf(shaper="harfbuzz", renderer="skia") as typf:
    result = typf.render_text("Hello, world!", "font.ttf")
```

## Advanced Rendering Options

### Complex Scripts

```bash
# Arabic text with proper shaping
typf-cli render \
  --text "مرحبا بالعالم" \
  --font path/to/arabic/font.ttf \
  --shaper harfbuzz \
  --output arabic.png

# Devanagari text
typf-cli render \
  --text "नमस्ते दुनिया" \
  --font path/to/devanagari/font.ttf \
  --shaper harfbuzz \
  --output devanagari.png
```

### Multiple Formats

```bash
# Generate multiple formats
typf-cli render \
  --text "Export formats" \
  --font font.ttf \
  --output-base export \
  --formats png svg json

# This creates: export.png, export.svg, export.json
```

### Batch Processing

```bash
# Process multiple texts
echo -e "Hello\nWorld\n世界" | typf-cli batch-render \
  --font font.ttf \
  --output-dir batch_output/

# Process from file
typf-cli batch-render \
  --input texts.txt \
  --font font.ttf \
  --output-dir batch_output/
```

## Performance Testing

### Benchmark Your System

```bash
# Run comprehensive benchmarks
typf-cli benchmark \
  --font font.ttf \
  --text-size 1000 \
  --iterations 100

# Test specific backends
typf-cli benchmark \
  --shaper harfbuzz \
  --renderer skia \
  --font font.ttf
```

### Python Performance Testing

```python
import time
import typfpy

# Performance test
start_time = time.time()
for i in range(100):
    result = typfpy.render_text(
        text=f"Text {i}",
        font_path="font.ttf",
        size=24.0
    )
end_time = time.time()

print(f"Rendered 100 texts in {end_time - start_time:.3f} seconds")
print(f"Average: {(end_time - start_time) / 100 * 1000:.2f}ms per render")
```

## Configuration

### Default Configuration

Create `~/.typf/config.toml`:

```toml
[default]
shaper = "harfbuzz"
renderer = "skia"
font_size = 24.0
color = "#000000"

[cache]
max_fonts = 100
max_glyphs = 10000
cache_dir = "~/.typf/cache"
```

### Environment Variables

```bash
# Set default backends
export TYPF_SHAPER=harfbuzz
export TYPF_RENDERER=skia

# Set cache directory
export TYPF_CACHE_DIR=/tmp/typf_cache
```

## Troubleshooting

### Common Issues

#### Font Not Found

```bash
# Check if font is accessible
typf-cli font-check --font path/to/font.ttf

# List system fonts
typf-cli font-list | grep -i arial
```

#### Shaping Issues

```bash
# Try different shaper
typf-cli render --text "العربية" --shaper coretext --font font.ttf

# Debug shaping
typf-cli debug-shape --text "العربية" --font font.ttf
```

#### Performance Problems

```bash
# Check cache status
typf-cli cache-status

# Clear cache
typf-cli cache-clear

# Use minimal build for speed
cargo build --release --features minimal
```

### Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| "Font not found" | Invalid font path | Check file exists and is readable |
| "Unsupported script" | Font doesn't support text | Use font with proper script support |
| "Backend not available" | Feature not compiled | Install with appropriate features |
| "Memory allocation failed" | Insufficient RAM | Reduce text size or use minimal build |

## Next Steps

### Learning Path

1. **Basics**: Continue with [Architecture Overview](03-architecture-overview.md)
2. **Configuration**: Read [Configuration Options](20-configuration-options.md)
3. **APIs**: Explore [Rust API](17-rust-api.md) or [Python Bindings](18-python-bindings.md)
4. **Backends**: Dive into specific [Shaping](09-harfbuzz-shaping.md) or [Rendering](13-skia-rendering.md) backends

### Examples and Tutorials

```bash
# Try example scripts
python examples/simple_render.py
python examples/long_text_handling.py
cargo run --example harfbuzz

# Run comprehensive tests
./typf-tester/typfme.py render
./typf-tester/typfme.py bench
```

### Getting Help

- 📖 **Documentation**: Continue reading this guide
- 🐛 **Issues**: [GitHub Issues](https://github.com/fontlaborg/typf/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/fontlaborg/typf/discussions)
- 📧 **Email**: support@fontlab.com

---

**Congratulations!** You now have TYPF v2.0 running and generating beautiful text. Let's explore the architecture next.
