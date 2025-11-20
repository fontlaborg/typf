---
title: Quick Start
icon: lucide/play-circle
tags:
  - Quick Start
  - Installation
  - Getting Started
---

# Quick Start

Get TYPF running in five minutes.

## Install

```bash
git clone https://github.com/fontlaborg/typf.git
cd typf
./build.sh
```

That's it. The build script installs everything you need.

## Your First Render

CLI:
```bash
typf-cli render --text "Hello, 世界!" --font /System/Library/Fonts/Arial.ttf --output hello.png
```

Python:
```python
import typfpy
result = typfpy.render_text("Hello, 世界!", "/System/Library/Fonts/Arial.ttf", 32.0)
with open("hello.png", "wb") as f:
    f.write(result.png_data)
```

## Choose Your Backends

```bash
# See what's available
typf-cli backend-list

# Use specific backends
typf-cli render --text "Sample" --font font.ttf --shaper harfbuzz --renderer skia --output out.png
```

## What Just Happened

1. **Input**: Your text entered the pipeline
2. **Unicode**: Script detection and bidi analysis
3. **Font**: Font loaded and matched to your text
4. **Shaping**: HarfBuzz positioned each glyph
5. **Rendering**: Skia drew the pixels
6. **Export**: PNG file written to disk

## Next Steps

```bash
# Try the examples
python examples/simple_render.py
cargo run --example basic

# Test everything works
./typf-tester/typfme.py
```

Read [Architecture Overview](03-architecture-overview.md) to understand how it works.