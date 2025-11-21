# CLI Interface

TYPF's command-line interface provides fast text rendering from the terminal.

## Basic Usage

```bash
# Simple text rendering
typf render "Hello World" -f font.ttf -o hello.png

# Use specific backends
typf render "Text" -f font.ttf --shaper harfbuzz --renderer skia -o text.png

# Python CLI (identical syntax)
typfpy render "Hello World" -f font.ttf -o hello.png
```

## Installation

```bash
# Build from source
git clone https://github.com/fontlaborg/typf.git
cd typf
./build.sh

# Install Python bindings
cd bindings/python
uv sync
maturin develop
```

## Commands

### info

Show available backends and system information.

```bash
typf info [--shapers] [--renderers] [--formats]

Options:
  --shapers       Show available shaper backends
  --renderers     Show available renderer backends
  --formats       Show supported export formats
```

### render

Main command for text rendering.

```bash
typf render [OPTIONS] <TEXT>

Arguments:
  <TEXT>          Text to render

Options:
  -f, --font <FONT>           Font file path [required]
  -o, --output <OUTPUT>       Output file path
  -s, --size <SIZE>           Font size in pixels [default: 32]
  -w, --width <WIDTH>         Image width [default: auto]
  -h, --height <HEIGHT>       Image height [default: auto]
      --shaper <SHAPER>       Text shaper backend [none|hb|icu-hb|mac]
      --renderer <RENDERER>   Rendering backend [orge|skia|zeno|json|mac|cg]
      --format <FORMAT>       Output format [png|svg|pnm|json]
  -c, --color <COLOR>         Text color (RRGGBBAA hex) [default: 000000FF]
  -b, --background <COLOR>    Background color (RRGGBBAA hex) [default: 00000000]
      --direction <DIR>       Text direction [ltr|rtl|ttb] [default: auto]
      --language <LANG>       Language code [default: auto]
      --script <SCRIPT>       Unicode script [default: auto]
  -F, --features <FEATURES>   Font features (comma-separated)
      --dpi <DPI>             Output resolution [default: 72]
  -v, --verbose               Show detailed output
```

### Examples

```bash
# Basic rendering
typf render "Hello World" -f Roboto.ttf -o hello.png

# Large text for printing
typf render "Print Title" -f serif.ttf -s 48 --dpi 300 -o title.png

# SVG export
typf render "Vector Text" -f sans.ttf --format svg -o vector.svg

# Custom colors (hex format)
typf render "Red Text" -f font.ttf -c FF0000FF -b F0F0F0FF -o red.png

# Using specific backends
typf render "Hi-Quality" -f font.ttf --shaper harfbuzz --renderer skia -o high.png

# Complex script with proper settings
typf render "مرحبا بالعالم" -f arabic.ttf --shaper harfbuzz --language ar --script Arab --direction rtl -o arabic.png

# Font features
typf render "Ligatures" -f font.ttf -F "liga,kern,dlig" -o features.png
```

### batch

Render multiple texts from a JSONL file.

```bash
typf batch [OPTIONS] --input <INPUT> --output-dir <OUTPUT_DIR>

Options:
  -i, --input <INPUT>         Input JSONL file path
  -o, --output-dir <OUTPUT_DIR>  Output directory
  -f, --font <FONT>           Default font file path
      --format <FORMAT>       Default output format [default: png]
      --size <SIZE>           Default font size [default: 32]
```

### Input Format (JSONL)

Each line contains a JSON object:
```jsonl
{"text": "Hello World", "size": 24, "output": "hello.png"}
{"text": "Big Title", "size": 48, "output": "title.png", "shaper": "harfbuzz"}
{"text": "مرحبا", "language": "ar", "script": "Arab", "direction": "rtl", "output": "arabic.png"}
```

### Example

```bash
# Create jobs file
cat > jobs.jsonl << 'EOF'
{"text": "Title", "size": 72, "output": "title.png"}
{"text": "Subtitle", "size": 48, "output": "subtitle.png"}
{"text": "Body", "size": 16, "output": "body.png"}
EOF

# Process all jobs
typf batch -i jobs.jsonl -o ./rendered/
```

## Configuration

### Config File

Create `~/.config/typf/config.toml`:

```toml
[default]
font = "~/fonts/Roboto-Regular.ttf"
size = 16
width = 800
height = 600
format = "png"
shaper = "harfbuzz"
renderer = "orge"

[colors]
text = "0,0,0,255"
background = "255,255,255,0"

[performance]
cache_size = "100MB"
parallel_jobs = 4
```

### Environment Variables

```bash
export TYPF_FONT=~/fonts/MyFont.ttf
export TYPF_SIZE=24
export TYPF_OUTPUT_DIR=./output
export TYPF_CONFIG=~/my-typf-config.toml
```

### Command Precedence

1. Command line flags (highest)
2. Environment variables
3. Config file
4. Defaults (lowest)

## Backend Selection

### Available Backends

```bash
# List available backends
typf --list-backends

Shapers:
- none          No shaping (identity mapping)
- harfbuzz      HarfBuzz text shaper
- icu-harfbuzz ICU + HarfBuzz composition
- coretext     macOS CoreText (macOS only)
- directwrite  Windows DirectWrite (Windows only)

Renderers:
- orge          Pure Rust rasterizer
- skia          Skia graphics library
- coregraphics macOS CoreGraphics (macOS only)
- directwrite  Windows DirectWrite (Windows only)
- zeno          Vector graphics renderer
```

### Selecting Backends

```bash
# Use specific shaper
typf render "Text" --font font.ttf --shaper harfbuzz

# Use specific renderer
typf render "Text" --font font.ttf --renderer skia

# Combine backends
typf render "Text" --font font.ttf --shaper icu-harfbuzz --renderer zeno

# Auto-select best available
typf render "Text" --font font.ttf  # Uses defaults
```

## Output Formats

### Format Support

| Format | Extension | Type | Features |
|--------|-----------|------|----------|
| PNG | .png | Raster | Transparency, compression |
| SVG | .svg | Vector | Scalable, web-friendly |
| PDF | .pdf | Document | Print-optimized, fonts |
| PNM | .pnm/.pbm/.pgm | Raster | Simple, uncompressed |
| JSON | .json | Data | Debug information |

### Format Options

```bash
# PNG with quality
typf render "Text" --font font.ttf --output image.png --png-quality 9

# SVG with embedded fonts
typf render "Text" --font font.ttf --output vector.svg --svg-embed-fonts

# PDF with metadata
typf render "Report" --font font.ttf --output doc.pdf --pdf-title "Report" --pdf-author "Me"
```

## Color Specification

### Color Formats

```bash
# Hex colors
typf render "Text" --font font.ttf --color "#FF0000"    --background "#FFFFFF"

# RGB/RGBA tuples
typf render "Text" --font font.ttf --color "255,0,0,255" --background "240,240,240,128"

# Named colors
typf render "Text" --font font.ttf --color red --background white

# Transparent background
typf render "Text" --font font.ttf --background transparent
```

### Named Colors

- `black`, `white`, `red`, `green`, `blue`, `yellow`, `cyan`, `magenta`
- `gray`, `grey`, `lightgray`, `darkgray`
- `orange`, `purple`, `brown`, `pink`
- `transparent`

## Performance Options

### Parallel Processing

```bash
# Use multiple threads
typf batch --input texts.txt --template "out_{line}.png" --jobs 8

# Benchmark with parallel processing
typf benchmark --iterations 1000 --parallel
```

### Caching

```bash
# Enable caching (default)
typf render "Cached text" --font font.ttf --cache

# Disable caching
typf render "One-time text" --font font.ttf --no-cache

# Clear cache
typf cache clear
```

### Memory Management

```bash
# Limit memory usage
typf render "Large text" --font font.ttf --memory-limit 1GB

# Stream large files
typf batch --input huge.txt --template "out_{line}.png" --stream
```

## Error Handling

### Exit Codes

- `0` - Success
- `1` - General error
- `2` - Font loading error
- `3` - Backend unavailable
- `4` - Invalid configuration
- `5` - I/O error

### Error Messages

```bash
$ typf render "Test" --font nonexistent.ttf
Error: Font loading failed: File not found: nonexistent.ttf

$ typf render "Test" --font font.ttf --shaper nonexistent
Error: Backend unavailable: Shaper 'nonexistent' not compiled

$ typf render "Test" --font font.ttf --output invalid.xyz
Error: Invalid configuration: Unsupported output format: xyz
```

### Troubleshooting

```bash
# Show debug information
typf render "Debug" --font font.ttf --verbose

# Test backends
typf benchmark --test-backends

# Check font support
typf font info font.ttf --verbose
```

## Integration Examples

### Shell Scripts

```bash
#!/bin/bash
# Generate previews for all fonts

for font in ~/fonts/*.ttf; do
    basename=$(basename "$font" .ttf)
    typf quick "$basename Sample" --font "$font" --output "previews/$basename.png"
done
```

### Make Integration

```makefile
# Makefile for text assets

TEXTS = $(wildcard texts/*.txt)
IMAGES = $(TEXTS:texts/%.txt=output/%.png)

output/%.png: texts/%.txt
	typf render "$$(cat $<)" --font Roboto.ttf --output $@

all: $(IMAGES)

clean:
	rm -f output/*.png
```

### CI/CD Pipeline

```yaml
# GitHub Actions example
- name: Render text assets
  run: |
    typf batch \
      --input assets/texts.txt \
      --template "generated/{line}.png" \
      --config ci/typf-config.toml
    
    typf benchmark \
      --font Roboto.ttf \
      --output performance.json

- name: Upload artifacts
  uses: actions/upload-artifact@v3
  with:
    name: text-assets
    path: generated/
```

## Migration from v1.x

If you're upgrading from TYPF v1.x, the CLI has changed significantly:

### Command Structure Changes

**Before (v1.x)**:
```bash
typf "Hello" --font font.ttf --output hello.png --size 48
python -m typfpy render "Hello" hello.png --font=/path/to/font.ttf
```

**After (v2.0.0)**:
```bash
typf render "Hello" -f font.ttf -o hello.png -s 48
typfpy render "Hello" -f /path/to/font.ttf -o hello.png
```

### Key Changes

- **Subcommands**: Now uses `info`, `render`, `batch` subcommands
- **Option names**: Shortened (`--output` → `-o`, `--font` → `-f`, `--size` → `-s`)
- **Python CLI**: Renamed from `python -m typfpy` to `typfpy`
- **Features**: Enhanced with Unicode escapes, color parsing, font features

### New Features

- Unicode escape sequences: `\u{1F44B}` for emoji
- Color format: RRGGBBAA hex (e.g., `FF0000FF` for red)
- Font features: `-F "liga,kern,dlig"`
- Better backend detection and selection

For complete migration details, see the main repository's `CLI_MIGRATION.md` file.

---

The CLI provides fast, scriptable text rendering from the command line. Use batch mode for bulk processing, benchmark for performance testing, and the REPL for interactive experimentation.
