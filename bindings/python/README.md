# Typf Python Bindings

High-performance text rendering for Python applications.

## Installation

```bash
# Using uv (recommended)
uv pip install typfpy

# Or using pip
pip install typfpy
```

Or install from source with maturin:

```bash
cd bindings/python

# Using uv (recommended)
uv pip install maturin
maturin develop --release

# Or with uvx (no install needed)
uvx maturin develop --release
```

## Quick Start

### Python API

```python
from typfpy import Typf, render_simple, export_image

# Simple text rendering (no font file needed)
image = render_simple("Hello, Typf!", size=48)

# Export to PNG
png_data = export_image(image, format="png")
with open("output.png", "wb") as f:
    f.write(png_data)

# Advanced usage with real fonts
engine = Typf(shaper="harfbuzz", renderer="opixa")
image = engine.render_text(
    "Typography is beautiful",
    font_path="/System/Library/Fonts/Helvetica.ttc",
    size=48,
    color=(0, 0, 255, 255),      # Blue text (RGBA)
    background=(255, 255, 200, 255),  # Light yellow background
    padding=20
)

# Variable font instance (axis variations)
image = engine.render_text(
    "Wide",
    font_path="/path/to/variable-font.ttf",
    size=48,
    variations={"wdth": 125.0, "wght": 650.0},
)

# Access image data
print(f"Size: {image['width']}x{image['height']}")
print(f"Format: {image['format']}")

# Export to multiple formats
png_data = export_image(image, format="png")
svg_data = export_image(image, format="svg")
ppm_data = export_image(image, format="ppm")
```

### Command Line Interface

The Click-based CLI provides a powerful command-line interface:

```bash
# Render text to PNG
typfpy render "Hello World" -o output.png

# Specify font and size
typfpy render "مرحبا" -f /path/to/arabic.ttf -s 64 -o output.png

# Custom colors (RRGGBBAA hex)
typfpy render "Red Text" -c FF0000FF -b FFFFFFFF -o output.png

# Choose renderer (linra-mac for best performance on macOS)
typfpy render "Text" --renderer=linra-mac -O svg -o output.svg

# Get system info and available backends
typfpy info

# Show available shapers
typfpy info --shapers

# Show available renderers
typfpy info --renderers
```

## Features

- ✅ Professional text shaping with HarfBuzz
- ✅ High-performance rendering with SIMD optimizations
- ✅ Support for TrueType/OpenType fonts (including .ttc collections)
- ✅ Multiple export formats (PNG, SVG, PPM, PGM, PBM, JSON)
- ✅ Thread-safe for concurrent rendering
- ✅ Complex script support (Arabic, Hebrew, Devanagari, Thai, CJK)
- ✅ OpenType features (ligatures, kerning, small caps, etc.)
- ✅ Variable fonts with axis variations (`{"wght": 700, "wdth": 120}`)
- ✅ Click-based CLI for command-line usage

## API Reference

### `typfpy.Typf`

Main rendering engine.

```python
from typfpy import Typf
engine = Typf(shaper="harfbuzz", renderer="opixa")
```

**Parameters:**
- `shaper` (str): Shaping backend - "none" or "harfbuzz" (default: "harfbuzz")
- `renderer` (str): Rendering backend - "opixa" (default: "opixa")

**Methods:**

#### `render_text(text, font_path, size=16.0, color=None, background=None, padding=10, variations=None)`

Render text to an image.

**Parameters:**
- `text` (str): Text to render
- `font_path` (str): Path to TrueType/OpenType font file
- `size` (float): Font size in points (default: 16.0)
- `color` (tuple): Foreground color as (R, G, B, A) (default: black)
- `background` (tuple): Background color as (R, G, B, A) (default: transparent)
- `padding` (int): Padding in pixels (default: 10)
- `variations` (dict | None): Variable font axis settings, e.g., `{"wght": 700, "wdth": 120}`

**Returns:** Dictionary with `width`, `height`, `format`, and `data` keys

**Example:**
```python
image = engine.render_text(
    "Hello",
    "/path/to/font.ttf",
    size=32,
    color=(255, 0, 0, 255),
    background=(255, 255, 255, 255),
    padding=15,
    variations={"wght": 700, "wdth": 110},
)
```

#### `get_shaper()` → str

Get current shaper name.

#### `get_renderer()` → str

Get current renderer name.

### `typfpy.FontInfo`

Font information and metrics.

```python
from typfpy import FontInfo
font = FontInfo("/path/to/font.ttf")
```

**Attributes:**
- `units_per_em` (int): Font units per em
- `path` (str): Path to font file

**Methods:**

#### `glyph_id(ch)` → int | None

Get glyph ID for a character.

**Example:**
```python
from typfpy import FontInfo
font = FontInfo("/System/Library/Fonts/Arial.ttf")
print(f"Units per em: {font.units_per_em}")
glyph_id = font.glyph_id('A')
print(f"Glyph ID for 'A': {glyph_id}")
```

### Module Functions

#### `typfpy.render_simple(text, size=16.0)` → dict

Simple rendering with stub font (no font file needed).

**Parameters:**
- `text` (str): Text to render
- `size` (float): Font size in points (default: 16.0)

**Returns:** Dictionary with `width`, `height`, `format`, and `data` keys

**Example:**
```python
from typfpy import render_simple
image = render_simple("Quick test", size=48)
```

#### `typfpy.export_image(image_data, format="ppm")` → bytes

Export image to various formats.

**Parameters:**
- `image_data` (dict): Image dictionary from `render_text()` or `render_simple()`
- `format` (str): Output format - "png", "svg", "ppm", "pgm", "pbm", or "json" (default: "ppm")

**Returns:** Bytes of the exported image

**Example:**
```python
from typfpy import render_simple, export_image
image = render_simple("Test")
png_bytes = export_image(image, format="png")
svg_bytes = export_image(image, format="svg")

with open("output.png", "wb") as f:
    f.write(png_bytes)
```

## Examples

See the `examples/` directory for complete examples:

- `simple_render.py` - Basic rendering with stub font
- `render_with_font.py` - Rendering with real font files
- `long_text_handling.py` - Strategies for long text (SVG, line wrapping)

## CLI Commands

### `typfpy render`

Render text to an image file.

```bash
typfpy render [TEXT] [OPTIONS]
```

**Options:**
- `-f, --font-file` - Path to font file (.ttf, .otf, .ttc, .otc)
- `-s, --font-size` - Font size in pixels (default: 200)
- `--shaper` - Shaping backend: auto, none, hb, icu-hb, mac (default: auto)
- `--renderer` - Rendering backend: auto, opixa, linra-mac, linra-win (default: auto)
- `-O, --format` - Output format: pbm, png1, pgm, png4, png8, png, svg (default: png)
- `-c, --foreground` - Text color as RRGGBB or RRGGBBAA (default: 000000FF)
- `-b, --background` - Background color as RRGGBB or RRGGBBAA (default: FFFFFF00)
- `-m, --margin` - Margin in pixels (default: 10)
- `-o, --output-file` - Output file path (stdout if omitted)
- `-q, --quiet` - Silent mode

### `typfpy info`

Display Typf version and available backends.

```bash
typfpy info [OPTIONS]
```

**Options:**
- `--shapers` - List available shaping backends
- `--renderers` - List available rendering backends
- `--formats` - List available output formats

## Development

### Building from Source

```bash
# Install maturin
pip install maturin

# Build and install in development mode
cd bindings/python
maturin develop

# Build release wheel
maturin build --release
```

### Running Tests

```bash
pytest
```

## Platform Support

- **macOS**: ✅ Full support (x86_64, ARM64)
- **Linux**: ✅ Full support (x86_64, ARM64)
- **Windows**: ✅ Full support (x86_64)

## Performance

Typf achieves industry-leading performance through:

- SIMD-optimized blending (10GB/s+ throughput)
- Multi-level caching (L1 < 50ns access)
- Parallel rendering with work-stealing
- Zero-copy font loading

## License

EVALUATION LICENSE

---

*Community project by FontLab - https://www.fontlab.org/*
