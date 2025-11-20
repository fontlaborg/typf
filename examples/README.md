# TypF Examples - From Text to Beautiful Output

This is your playground for exploring TYPF's text rendering capabilities. Each example starts with raw text and shows how different shapers, renderers, and exporters transform it into final output. Start simple, then explore complex scripts, variable fonts, and vector graphics.

## 1. Quick Start

Run any example with a single command:

```bash
cargo run --example basic
```

## 2. Example Gallery

### 2.1. **basic** - First Steps in Text Rendering
The "Hello, World!" of TypF - see text transform into pixels through the complete pipeline.

```bash
cargo run --example basic
```

**What you'll learn:**
- How the six-stage pipeline works in practice
- Create your first shaper/renderer/exporter chain
- Set colors, backgrounds, and padding
- Save rendered text as an image

**Result:** `examples/output.ppm` - Your first rendered text!

---

### 2.2. **formats** - One Text, Five Formats
Shape and render once, export everywhere. Compare file sizes and quality across TYPF's export formats.

```bash
cargo run --example formats
```

**Formats demonstrated:**
- **PNG**: Web-optimized, compressed with alpha channel
- **SVG**: Vector graphics with embedded bitmap data
- **PPM**: Portable Pixmap - uncompressed RGB color
- **PGM**: Portable Graymap - single-channel grayscale
- **PBM**: Portable Bitmap - pure black and white

**Result:** `examples/output/test.{ppm,pgm,pbm,png,svg}` - Same text, every format

---

### 2.3. **harfbuzz** - World-Class Typography
Watch professional shaper HarfBuzz transform Arabic, Devanagari, and complex scripts with OpenType features.

```bash
cargo run --example harfbuzz --features shaping-hb
```

**Typography superpowers:**
- **Complex scripts**: Arabic RTL, Hindi connecting forms, Thai tone marks
- **OpenType features**: Ligatures, kerning, small caps, number styles
- **Language awareness**: Script-specific shaping rules
- **Performance boost**: Built-in caching for repeated text

**Output:** `examples/harfbuzz_output.ppm` - Publication-quality text

**Note:** Requires `shaping-hb` feature - brings in professional-grade typography

---

### 2.4. **pipeline** - Build Once, Process Forever
Create reusable text processing pipelines that compose, configure, and execute complex rendering workflows.

```bash
cargo run --example pipeline
```

**Pipeline patterns:**
- **Builder pattern**: Clear, declarative stage composition
- **Parameter control**: Fine-tune shaping and rendering independently
- **Reusability**: One pipeline, unlimited texts
- **Maintainability**: Separation of concerns, testable components

**Output:** `examples/pipeline_output.ppm` - Clean architecture in action

---

## 3. Output Directory

All examples create output files in:
- Individual examples: `examples/*.ppm`
- Formats example: `examples/output/*`

The `examples/output/` directory is created automatically and is git-ignored.

## 4. Feature Requirements

Some examples require specific Cargo features to be enabled:

| Example | Required Features | Default? |
|---------|------------------|----------|
| `basic` | (none) | ✓ |
| `formats` | `export-png`, `export-svg` | ✓ |
| `harfbuzz` | `shaping-hb` | ✓ |
| `pipeline` | (none) | ✓ |

All examples work with the default feature set when building with `cargo run --example <name>`.

## 5. Example Descriptions

### 5.1. Text Used

- **basic**: "Hello, TYPF!"
- **formats**: "Format Test"
- **harfbuzz**: "Complex Text with لغة"
- **pipeline**: "Pipeline Example"

### 5.2. Font Handling

Most examples use mock/stub fonts for demonstration purposes. For real font rendering:

1. Load a TrueType or OpenType font file
2. Use `typf_fontdb::Font::from_file(path)`
3. Pass the font to the shaper

Example:
```rust
use typf_fontdb::Font;
let font = Font::from_file("path/to/font.ttf")?;
```

## 6. Troubleshooting

### 6.1. "No such file or directory" errors
The `formats` example creates the `examples/output/` directory automatically. For other examples, ensure you're running from the project root.

### 6.2. HarfBuzz not found
If the `harfbuzz` example fails to compile, ensure HarfBuzz is installed:

```bash
# macOS
brew install harfbuzz

# Ubuntu/Debian
sudo apt-get install libharfbuzz-dev

# Fedora
sudo dnf install harfbuzz-devel
```

### 6.3. Viewing output files

**PNM files (PPM/PGM/PBM):**
- macOS: Preview.app, GIMP
- Linux: GIMP, ImageMagick (`display file.ppm`)
- Windows: IrfanView, GIMP

**PNG files:**
- Any modern image viewer

**SVG files:**
- Any modern web browser
- Inkscape
- Adobe Illustrator

### 6.4. **variable_fonts** - One Font, Infinite Styles
Experience variable fonts that morph between weights, widths, and optical sizes without loading multiple font files.

```bash
cargo run --example variable_fonts --features shaping-hb
```

**Design freedom unlocked:**
- **Weight axis**: Light to Black in real-time (100-900)
- **Width axis**: Compressed to Extended (75-125%)
- **Optical size**: Optimized for micro to macro displays
- **Multi-axis control**: Combine variations for exact typography
- **Performance**: One font file instead of dozens

**Note:** Load an actual variable font file to see the magic happen

---

### 6.5. **svg_export_example** - Vector Graphics That Never Pixelate
Extract real glyph outlines from fonts and convert them to SVG paths - not just bitmas wrapped in XML.

```bash
cargo run --example svg_export_example --features shaping-hb,export-svg
```

**Vector excellence:**
- **True paths**: Actual Bézier curves from font outlines
- **Infinite scaling**: Zoom forever without quality loss
- **Tiny files**: ~30x smaller than bitmap-in-SVG alternatives
- **Design friendly**: Editable in Illustrator, Inkscape, Figma
- **Web ready**: Perfect scalable graphics for any screen

**Output:** `output.svg` - Professional vector typography

---

### 6.6. **all_formats** - The Complete Export Tour
One text sample rendered through every format TypF offers - compare quality, file size, and use cases.

```bash
cargo run --example all_formats --features full
```

**Format showcase:**
- **PNG + Alpha**: Web-optimized with transparency
- **SVG Vector**: Infinite scalability with path data
- **PNM Family**: RGB, grayscale, and bitmap variants
- **JSON Debug**: Insight into shaping algorithms
- **Complex scripts**: Arabic text challenges each exporter

**Output**: `examples/output/*` - Complete format comparison suite

---

### 6.7. **backend_comparison** - Compare backends
Compare different shaping and rendering backends.

```bash
cargo run --example backend_comparison --features shaping-hb
```

**Features shown:**
- Multiple backend configurations
- Performance comparison
- Quality comparison

---

### 6.8. **long_text_handling** - Handle long text 🆕
Strategies for text exceeding bitmap width limits (~10,000 pixels).

```bash
cargo run --example long_text_handling --features shaping-hb,export-svg
```

**Features shown:**
- Width estimation and limit checking
- SVG export for unlimited width
- Word-based line wrapping implementation
- Adaptive font sizing calculator
- Chunked rendering approach

**Topics covered:**
1. Detecting when text is too long for bitmap rendering
2. Using SVG export as an alternative (no width limits)
3. Implementing simple line wrapping
4. Adaptive font sizing to fit target width
5. Chunked rendering for very long documents

**See also:**
- `bindings/python/examples/long_text_handling.py` - Python equivalent
- `docs/PERFORMANCE.md` - Performance optimization guide
- `README.md` - Known Limitations section

---

## 7. Python Examples

Python examples are located in `bindings/python/examples/`:

- **`long_text_handling.py`**: Handling long text with Python bindings

To run Python examples:

```bash
cd bindings/python
python examples/long_text_handling.py
```

**Prerequisites**:
1. Build Python bindings: `maturin develop --release --features shaping-hb,export-svg`
2. Install Python dependencies: `pip install fire pillow`

---

## 8. Benchmarking & Testing

For comprehensive benchmarking and testing:

```bash
cd typf-tester

# Test all backends
python typfme.py render --backend=harfbuzz --format=png

# Benchmark performance
python typfme.py bench --iterations=100

# Benchmark shaping only
python typfme.py bench-shaping --iterations=1000

# Benchmark rendering only
python typfme.py bench-rendering --iterations=100

# Test text length scaling
python typfme.py bench-scaling --iterations=50
```

**Outputs:**
- `output/benchmark_report.json` - Detailed JSON results
- `output/benchmark_summary.md` - Compact Markdown table

**See**: `typf-tester/README.md` for full testing documentation

---

## 9. Next Steps

After exploring these examples, check out:

- **Architecture docs**: `PLAN/00.md` - Full system design
- **Performance guide**: `docs/PERFORMANCE.md` - Optimization strategies
- **API docs**: Run `cargo doc --open`
- **Tests**: Run `cargo test --workspace` to see more usage patterns

---

*Made by FontLab - https://www.fontlab.com/*
