---
title: Quick Start
icon: lucide/play-circle
tags:
  - Quick Start
  - Installation
  - Getting Started
---

# Quick Start

Get Typf running in five minutes.

## Install

```bash
git clone https://github.com/fontlaborg/typf.git
cd typf
./build.sh
```

That's it. The build script installs everything you need.

## Your First Render

Rust CLI:
```bash
typf render "Hello, 世界!" -f /System/Library/Fonts/Arial.ttf -o hello.png
```

Python CLI:
```bash
typfpy render "Hello, 世界!" -f /System/Library/Fonts/Arial.ttf -o hello.png
```

Python API:
```python
from typfpy import Typf, export_image

typf = Typf(shaper="harfbuzz", renderer="opixa")
image = typf.render_text("Hello, 世界!", "/System/Library/Fonts/Arial.ttf", size=48)
with open("hello.png", "wb") as f:
    f.write(export_image(image, format="png"))
```

## Choose Your Backends

```bash
# See what's available
typf info

# Use specific backends
typf render "Sample" -f font.ttf --shaper harfbuzz --renderer skia -o out.png
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
