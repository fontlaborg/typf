# TYPF Python Bindings

High-performance text rendering for Python applications.

## Installation

```bash
pip install typf
```

Or install from source with maturin:

```bash
cd bindings/python
maturin develop --release
```

## Quick Start

### Python API

```python
import typf

# Simple text rendering (no font file needed)
image = typf.render_simple("Hello, TYPF!", size=48)

# Export to PNG
png_data = typf.export_image(image, format="png")
with open("output.png", "wb") as f:
    f.write(png_data)

# Advanced usage with real fonts
engine = typf.Typf(shaper="harfbuzz", renderer="orge")
image = engine.render_text(
    "Typography is beautiful",
    font_path="/System/Library/Fonts/Helvetica.ttc",
    size=48,
    color=(0, 0, 255, 255),      # Blue text (RGBA)
    background=(255, 255, 200, 255),  # Light yellow background
    padding=20
)

# Access image data
print(f"Size: {image['width']}x{image['height']}")
print(f"Format: {image['format']}")

# Export to multiple formats
png_data = typf.export_image(image, format="png")
svg_data = typf.export_image(image, format="svg")
ppm_data = typf.export_image(image, format="ppm")
```

### Command Line Interface

The Fire-based CLI provides a powerful command-line interface:

```bash
# Render text to PNG
typf render "Hello World" output.png

# Specify font and size
typf render "مرحبا" output.png --font=/path/to/arabic.ttf --size=64

# Custom colors (R,G,B,A)
typf render "Red Text" output.png --color="255,0,0,255" --background="255,255,255,255"

# Choose shaper and renderer
typf render "Text" output.svg --shaper=harfbuzz --renderer=orge --format=svg

# Get version info
typf version

# Get system info
typf info

# Shape text (JSON output)
typf shape "Complex text" --font=/path/to/font.ttf --features="liga=1,kern=1"
```

## Features

- ✅ Professional text shaping with HarfBuzz
- ✅ High-performance rendering with SIMD optimizations
- ✅ Support for TrueType/OpenType fonts (including .ttc collections)
- ✅ Multiple export formats (PNG, SVG, PPM, PGM, PBM, JSON)
- ✅ Thread-safe for concurrent rendering
- ✅ Complex script support (Arabic, Hebrew, Devanagari, Thai, CJK)
- ✅ OpenType features (ligatures, kerning, small caps, etc.)
- ✅ Fire-based CLI for command-line usage

## API Reference

### `typf.Typf`

Main rendering engine.

```python
engine = typf.Typf(shaper="harfbuzz", renderer="orge")
```

**Parameters:**
- `shaper` (str): Shaping backend - "none" or "harfbuzz" (default: "harfbuzz")
- `renderer` (str): Rendering backend - "orge" (default: "orge")

**Methods:**

#### `render_text(text, font_path, size=16.0, color=None, background=None, padding=10)`

Render text to an image.

**Parameters:**
- `text` (str): Text to render
- `font_path` (str): Path to TrueType/OpenType font file
- `size` (float): Font size in points (default: 16.0)
- `color` (tuple): Foreground color as (R, G, B, A) (default: black)
- `background` (tuple): Background color as (R, G, B, A) (default: transparent)
- `padding` (int): Padding in pixels (default: 10)

**Returns:** Dictionary with `width`, `height`, `format`, and `data` keys

**Example:**
```python
image = engine.render_text(
    "Hello",
    "/path/to/font.ttf",
    size=32,
    color=(255, 0, 0, 255),
    background=(255, 255, 255, 255),
    padding=15
)
```

#### `get_shaper()` → str

Get current shaper name.

#### `get_renderer()` → str

Get current renderer name.

### `typf.FontInfo`

Font information and metrics.

```python
font = typf.FontInfo("/path/to/font.ttf")
```

**Attributes:**
- `units_per_em` (int): Font units per em
- `path` (str): Path to font file

**Methods:**

#### `glyph_id(ch)` → int | None

Get glyph ID for a character.

**Example:**
```python
font = typf.FontInfo("/System/Library/Fonts/Arial.ttf")
print(f"Units per em: {font.units_per_em}")
glyph_id = font.glyph_id('A')
print(f"Glyph ID for 'A': {glyph_id}")
```

### Module Functions

#### `typf.render_simple(text, size=16.0)` → dict

Simple rendering with stub font (no font file needed).

**Parameters:**
- `text` (str): Text to render
- `size` (float): Font size in points (default: 16.0)

**Returns:** Dictionary with `width`, `height`, `format`, and `data` keys

**Example:**
```python
image = typf.render_simple("Quick test", size=48)
```

#### `typf.export_image(image_data, format="ppm")` → bytes

Export image to various formats.

**Parameters:**
- `image_data` (dict): Image dictionary from `render_text()` or `render_simple()`
- `format` (str): Output format - "png", "svg", "ppm", "pgm", "pbm", or "json" (default: "ppm")

**Returns:** Bytes of the exported image

**Example:**
```python
image = typf.render_simple("Test")
png_bytes = typf.export_image(image, format="png")
svg_bytes = typf.export_image(image, format="svg")

with open("output.png", "wb") as f:
    f.write(png_bytes)
```

## Examples

See the `examples/` directory for complete examples:

- `simple_render.py` - Basic rendering with stub font
- `render_with_font.py` - Rendering with real font files

## CLI Commands

### `typf render`

Render text to an image file.

```bash
typf render TEXT OUTPUT [OPTIONS]
```

**Options:**
- `--font` - Path to font file (optional)
- `--size` - Font size in points (default: 48.0)
- `--shaper` - Shaping backend: "none" or "harfbuzz" (default: "harfbuzz")
- `--renderer` - Rendering backend: "orge" (default: "orge")
- `--format` - Output format (inferred from extension if not specified)
- `--color` - Foreground color as "R,G,B,A" (default: "0,0,0,255")
- `--background` - Background color as "R,G,B,A" (optional)
- `--padding` - Padding in pixels (default: 10)

### `typf shape`

Shape text and output glyph positioning.

```bash
typf shape TEXT [OPTIONS]
```

**Options:**
- `--font` - Path to font file
- `--size` - Font size in points (default: 48.0)
- `--shaper` - Shaping backend (default: "harfbuzz")
- `--features` - OpenType features as "key=value,key=value"
- `--language` - Language tag (e.g., "ar", "en")
- `--script` - Script tag (e.g., "arab", "latn")
- `--output` - Output file path (stdout if not specified)

### `typf info`

Display TYPF version and configuration.

### `typf version`

Display version information.

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

TYPF achieves industry-leading performance through:

- SIMD-optimized blending (10GB/s+ throughput)
- Multi-level caching (L1 < 50ns access)
- Parallel rendering with work-stealing
- Zero-copy font loading

## License

EVALUATION LICENSE

---

*Made by FontLab - https://www.fontlab.com/*
