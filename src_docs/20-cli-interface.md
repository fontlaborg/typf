# CLI Interface

TYPF's command-line interface provides fast text rendering from the terminal.

## Basic Usage

```bash
# Simple text rendering
typf render "Hello World" --font font.ttf --output hello.png

# Use specific backends
typf render "Text" --font font.ttf --shaper harfbuzz --renderer skia --output text.png

# Quick render with defaults
typf quick "Hello CLI" --output hello.png
```

## Installation

```bash
# Install from crates.io
cargo install typf-cli

# Build from source
cargo install --path crates/typf-cli

# Build with features
cargo install --path crates/typf-cli --features "shaping-harfbuzz,render-skia"
```

## Commands

### render

Main command for text rendering.

```bash
typf render [OPTIONS] <TEXT> --font <FONT>

Arguments:
  <TEXT>          Text to render

Options:
  -f, --font <FONT>           Font file path [required]
  -o, --output <OUTPUT>       Output file path
  -s, --size <SIZE>           Font size in pixels [default: 16]
  -w, --width <WIDTH>         Image width [default: 800]
  -h, --height <HEIGHT>       Image height [default: 600]
      --shaper <SHAPER>       Text shaper backend
      --renderer <RENDERER>   Rendering backend
      --format <FORMAT>       Output format [png|svg|pdf|pnm|json]
      --color <COLOR>         Text color (rgba)
      --background <COLOR>    Background color (rgba)
      --dpi <DPI>             Output resolution [default: 72]
      --no-antialiasing       Disable edge smoothing
      --hinting <HINTING>     Font hinting mode
  -v, --verbose               Show detailed output
```

### Examples

```bash
# Basic rendering
typf render "Hello World" --font Roboto.ttf --output hello.png

# Large text for printing
typf render "Print Title" --font serif.ttf --size 48 --dpi 300 --output title.png

# SVG export
typf render "Vector Text" --font sans.ttf --format svg --output vector.svg

# Custom colors
typf render "Red Text" --font font.ttf --color "255,0,0,255" --background "240,240,240,255" --output red.png

# Using backends
typf render "Hi-Quality" --font font.ttf --shaper harfbuzz --renderer skia --output high.png
```

### quick

Simplified rendering with sensible defaults.

```bash
typf quick [OPTIONS] <TEXT> --output <OUTPUT>

Arguments:
  <TEXT>          Text to render

Options:
  -o, --output <OUTPUT>       Output file path [required]
  -f, --font <FONT>           Font file (uses system fonts if omitted)
      --size <SIZE>           Font size [default: 24]
      --format <FORMAT>       Output format [default: png]
```

### Examples

```bash
# Quick PNG with system fonts
typf quick "Hello" --output hello.png

# Quick SVG
typf quick "SVG Text" --output text.svg --format svg

# Custom font
typf quick "Custom Font" --output custom.png --font myfont.ttf
```

### batch

Render multiple texts from a file or stdin.

```bash
typf batch [OPTIONS] --input <INPUT> --template <TEMPLATE>

Options:
  -i, --input <INPUT>         Input file path or "-" for stdin
  -t, --template <TEMPLATE>   Output filename template
  -f, --font <FONT>           Font file path
      --format <FORMAT>       Output format
      --config <CONFIG>       JSON config file
```

### Template Variables

Use variables in output filenames:

```bash
# Template with line number
typf batch --input texts.txt --template "output_{line}.png"

# Template with text hash
typf batch --input texts.txt --template "output_{hash}.svg"

# Template with timestamp
typf batch --input texts.txt --template "output_{time}.png"
```

### Input Formats

Plain text file:
```
Hello World
Second Line  
TypF CLI
```

JSON config:
```json
{
  "font": "Roboto.ttf",
  "size": 16,
  "format": "png",
  "texts": [
    {"text": "Hello", "output": "hello.png"},
    {"text": "World", "size": 24, "output": "world.png"}
  ]
}
```

### font

Font information and testing.

```bash
typf font [OPTIONS] <COMMAND>

Commands:
  info     Show font information
  list     List available fonts
  test     Test font rendering
  search   Search for fonts
```

#### font info

```bash
typf font info <FONT_PATH>

Options:
  -v, --verbose    Show detailed glyph information
```

Output:
```
Font: Roboto-Regular.ttf
Family: Roboto
Style: Regular
Units per EM: 2048
Ascender: 1900
Descender: -500
Line Gap: 0
Supported scripts: Latin, Cyrillic, Greek
```

#### font list

```bash
typf font list [OPTIONS]

Options:
  -f, --family <FAMILY>    Filter by font family
  -s, --style <STYLE>      Filter by style
  --system                 Include system fonts
```

#### font test

```bash
typf font test <FONT_PATH> --output <OUTPUT>

Options:
      --size <SIZE>         Test font size [default: 16]
      --text <TEXT>         Test text [default: "Hello World"]
      --sample              Generate comprehensive test
```

### shape

Text shaping analysis and debugging.

```bash
typf shape [OPTIONS] <TEXT> --font <FONT>

Options:
  -f, --font <FONT>           Font file path
  -s, --shaper <SHAPER>       Shaper backend
      --direction <DIR>       Text direction [ltr|rtl|ttb]
      --script <SCRIPT>       Unicode script
      --language <LANG>       Language code
  -o, --output <OUTPUT>       Save analysis to file
      --format <FORMAT>       Output format [json|yaml]
```

### Examples

```bash
# Basic shaping analysis
typf shape "Hello World" --font Roboto.ttf

# Right-to-left text
typf shape "مرحبا بالعالم" --font arabic.ttf --direction rtl

# Save analysis
typf shape "Analysis" --font font.ttf --output analysis.json
```

### benchmark

Performance testing and benchmarks.

```bash
typf benchmark [OPTIONS]

Options:
  -f, --font <FONT>           Font file path
  -t, --text <TEXT>           Test text
      --iterations <N>        Number of iterations [default: 100]
      --backends <BACKENDS>   Test specific backends
      --output <OUTPUT>       Save results to file
      --format <FORMAT>       Results format [json|csv|markdown]
```

### Examples

```bash
# Quick benchmark
typf benchmark --font Roboto.ttf

# Custom test
typf benchmark --font font.ttf --text "Performance test" --iterations 1000

# Test specific backends
typf benchmark --backends "harfbuzz+orge,harfbuzz+skia" --output results.json
```

### repl

Interactive REPL for testing.

```bash
typf repl [OPTIONS]

Options:
  -f, --font <FONT>           Default font
      --size <SIZE>           Default font size
      --output <DIR>          Default output directory
```

REPL Commands:
```
> render "Hello World" --font Roboto.ttf
Rendered to output_001.png

> shape "Unicode test 😊" --font emoji.ttf
Text: Unicode test 😊
Glyphs: 14 (includes emoji)
Script: Mixed

> help
Available commands: render, shape, font, benchmark, quit

> quit
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

---

The CLI provides fast, scriptable text rendering from the command line. Use batch mode for bulk processing, benchmark for performance testing, and the REPL for interactive experimentation.
