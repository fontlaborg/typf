# TYPF Examples

This directory contains working examples demonstrating various features of the TYPF text rendering pipeline.

## Running Examples

Each example can be run using cargo:

```bash
cargo run --example <example_name>
```

## Available Examples

### 1. **basic** - Simple text rendering
Demonstrates the most basic usage of TYPF for rendering text.

```bash
cargo run --example basic
```

**Features shown:**
- Creating a simple shaper (NoneShaper)
- Rendering with OrgeRenderer
- Exporting to PNM format
- Basic color and background settings

**Output:** `examples/basic_output.ppm`

---

### 2. **formats** - All export formats showcase
Demonstrates all supported export formats in a single example.

```bash
cargo run --example formats
```

**Features shown:**
- PNG export (production-ready compressed format)
- SVG export (vector graphics with embedded bitmap)
- PNM formats:
  - PPM (Portable Pixmap - color)
  - PGM (Portable Graymap - grayscale)
  - PBM (Portable Bitmap - black/white)
- Bitmap metadata inspection

**Output:** `examples/output/test.{ppm,pgm,pbm,png,svg}`

---

### 3. **harfbuzz** - Complex script shaping
Demonstrates HarfBuzz integration for complex text shaping.

```bash
cargo run --example harfbuzz --features shaping-hb
```

**Features shown:**
- HarfBuzz shaper integration
- OpenType feature support (liga, kern, smcp)
- Language and script tags
- Complex script handling (Arabic, Devanagari, etc.)
- Shaping cache usage

**Output:** `examples/harfbuzz_output.ppm`

**Note:** Requires `shaping-hb` feature to be enabled.

---

### 4. **pipeline** - Pipeline builder pattern
Demonstrates the Pipeline builder API for composing stages.

```bash
cargo run --example pipeline
```

**Features shown:**
- Pipeline builder pattern
- Custom shaping parameters
- Custom render parameters
- Foreground and background colors
- Padding control

**Output:** `examples/pipeline_output.ppm`

---

## Output Directory

All examples create output files in:
- Individual examples: `examples/*.ppm`
- Formats example: `examples/output/*`

The `examples/output/` directory is created automatically and is git-ignored.

## Feature Requirements

Some examples require specific Cargo features to be enabled:

| Example | Required Features | Default? |
|---------|------------------|----------|
| `basic` | (none) | ✓ |
| `formats` | `export-png`, `export-svg` | ✓ |
| `harfbuzz` | `shaping-hb` | ✓ |
| `pipeline` | (none) | ✓ |

All examples work with the default feature set when building with `cargo run --example <name>`.

## Example Descriptions

### Text Used

- **basic**: "Hello, TYPF!"
- **formats**: "Format Test"
- **harfbuzz**: "Complex Text with لغة"
- **pipeline**: "Pipeline Example"

### Font Handling

Most examples use mock/stub fonts for demonstration purposes. For real font rendering:

1. Load a TrueType or OpenType font file
2. Use `typf_fontdb::Font::from_file(path)`
3. Pass the font to the shaper

Example:
```rust
use typf_fontdb::Font;
let font = Font::from_file("path/to/font.ttf")?;
```

## Troubleshooting

### "No such file or directory" errors
The `formats` example creates the `examples/output/` directory automatically. For other examples, ensure you're running from the project root.

### HarfBuzz not found
If the `harfbuzz` example fails to compile, ensure HarfBuzz is installed:

```bash
# macOS
brew install harfbuzz

# Ubuntu/Debian
sudo apt-get install libharfbuzz-dev

# Fedora
sudo dnf install harfbuzz-devel
```

### Viewing output files

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

## Next Steps

After exploring these examples, check out:

- **Architecture docs**: `PLAN/00.md` - Full system design
- **API docs**: Run `cargo doc --open`
- **Tests**: Run `cargo test --workspace` to see more usage patterns

---

*Made by FontLab - https://www.fontlab.com/*
