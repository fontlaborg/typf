# CLI Migration Guide

## Overview

**Status**: ✅ Complete  
**Date**: 2025-11-21  
**Version**: TYPF v2.0.0-dev

Both Rust and Python CLIs have been migrated to modern, user-friendly frameworks with full feature parity.

---

## What Changed

### Rust CLI (`typf`)

**Before**: Manual argument parsing with flat structure  
**After**: Clap v4 with subcommands

```bash
# Old (v1.x)
typf "Hello" --font font.ttf --output hello.png --size 48

# New (v2.0)
typf render "Hello" -f font.ttf -o hello.png -s 48
```

### Python CLI (`typfpy`)

**Before**: Fire-based automatic CLI  
**After**: Click v8 with explicit commands

```bash
# Old (v1.x)
python -m typfpy render "Hello" hello.png --font=/path/to/font.ttf

# New (v2.0)
typfpy render "Hello" -f /path/to/font.ttf -o hello.png
```

---

## New Command Structure

Both CLIs now use identical subcommands:

### 1. Info Command

Display available backends and formats:

```bash
# Show all info
typf info
typfpy info

# Show specific info
typf info --shapers
typf info --renderers
typf info --formats
```

### 2. Render Command

Render text to images with full control:

```bash
# Basic rendering
typf render "Hello World" -o output.png
typfpy render "Hello World" -o output.png

# With font file
typf render "Text" -f font.ttf -o out.png -s 72
typfpy render "Text" -f font.ttf -o out.png -s 72

# Advanced: Arabic text with proper shaping
typf render "مرحبا بالعالم" \
  -f arabic.ttf \
  --shaper hb \
  --language ar \
  --script Arab \
  --direction rtl \
  -o arabic.svg

# Unicode escapes
typf render "Hello \u{1F44B} World" -o emoji.png

# Custom colors (RRGGBBAA format)
typf render "Alert" \
  -c FF0000FF \
  -b FFFFFFFF \
  -o alert.png

# Font features
typf render "Ligatures" \
  -f font.ttf \
  -F "liga,kern,-dlig" \
  -o features.png
```

### 3. Batch Command (Rust only)

Process multiple jobs from JSONL:

```bash
# Create jobs file
cat > jobs.jsonl << 'EOJ'
{"text": "Hello", "output": "hello.png", "size": 48}
{"text": "مرحبا", "output": "arabic.png", "language": "ar", "shaper": "hb"}
{"text": "World", "output": "world.svg", "format": "svg"}
EOJ

# Run batch processing
typf batch -i jobs.jsonl -o ./output/
```

---

## Complete Option Reference

### Global Options

```
-h, --help          Show help message
-V, --version       Show version
```

### Info Command Options

```
--shapers           List available shaping backends
--renderers         List available rendering backends  
--formats           List output formats
```

### Render Command Options

**Font Face**:
```
-f, --font-file <PATH>           Font file (.ttf, .otf, .ttc, .otc)
-y, --face-index <N>             Face index for TTC/OTC [default: 0]
-i, --instance <SPEC>            Named or dynamic instance
```

**Text Input** (one of):
```
<TEXT>                           Positional text argument
-t, --text <TEXT>                Text via option
-T, --text-file <PATH>           Read from file
(stdin)                          Read from standard input
```

**Backends**:
```
--shaper <NAME>                  Shaper: auto, none, hb, icu-hb, mac, win
--renderer <NAME>                Renderer: auto, orge, skia, zeno, mac, win
```

**Text Processing**:
```
-d, --direction <DIR>            Direction: auto, ltr, rtl, ttb, btt
-l, --language <TAG>             Language tag (BCP 47): en, ar, zh-Hans
-S, --script <TAG>               Script tag (ISO 15924): Latn, Arab, Hans
```

**Font Features**:
```
-F, --features <SPEC>            Feature settings: kern,+liga,-dlig,salt=2
```

**Size & Layout**:
```
-s, --font-size <SIZE>           Font size in pixels or 'em' [default: 200]
-L, --line-height <PCT>          Line height as % [default: 120]
-W, --width-height <SPEC>        Canvas: WxH, Wx, xH, or none
-m, --margin <PX>                Margin in pixels [default: 10]
--font-optical-sizing <MODE>     Optical sizing: auto, none
```

**Colors**:
```
-c, --foreground <COLOR>         Text color RRGGBB or RRGGBBAA [default: 000000FF]
-b, --background <COLOR>         Background RRGGBB or RRGGBBAA [default: FFFFFF00]
-p, --color-palette <N>          CPAL palette index [default: 0]
```

**Output**:
```
-o, --output-file <PATH>         Output file (stdout if omitted)
-O, --format <FMT>               Format: pbm, png1, pgm, png4, png8, png, svg
-q, --quiet                      Silent mode
--verbose                        Verbose output
```

### Batch Command Options (Rust only)

```
-i, --input <PATH>               Input JSONL file (stdin if omitted)
-o, --output <DIR>               Output directory [default: .]
-p, --pattern <PATTERN>          Filename pattern with {} [default: output_{}]
-q, --quiet                      Silent mode
--verbose                        Verbose output
```

---

## Special Features

### Unicode Escape Sequences

Both formats supported:

```bash
# 4-digit hex
typf render "Hello \uXXXX World" -o out.png

# Variable-length hex with braces
typf render "Wave \u{1F44B}" -o wave.png
```

### Feature Specifications

Multiple formats accepted:

```bash
# Simple tags
-F "kern,liga"

# Enable/disable prefix
-F "+liga,-dlig"

# With values
-F "salt=2,ss01=1"

# Mixed
-F "kern,+liga,-dlig,salt=2"
```

### Color Formats

Both RRGGBB and RRGGBBAA supported:

```bash
# RGB only (alpha=255)
-c FF0000 -b FFFFFF

# RGBA with alpha channel
-c FF0000FF -b FFFFFF80

# With # prefix (optional)
-c "#FF0000" -b "#FFFFFF"
```

### Stdin Input

Read text from stdin when not provided:

```bash
echo "Hello from stdin" | typf render -o stdin.png
cat text.txt | typfpy render -o output.svg
```

---

## Migration Checklist

If upgrading from v1.x:

- [ ] Update command structure to use subcommands (`info`, `render`, `batch`)
- [ ] Change option names:
  - `--output` → `-o` or `--output-file`
  - `--size` → `-s` or `--font-size`
  - `--font` → `-f` or `--font-file`
- [ ] Use new format names:
  - Old: `png`, `ppm`, `pgm`, `pbm`, `svg`
  - New: `png`, `png1`, `png4`, `png8`, `pgm`, `pbm`, `ppm`, `svg`
- [ ] Update color format to RRGGBB/RRGGBBAA hex
- [ ] Review feature specification syntax if using OpenType features
- [ ] Test with `--help` to verify options

---

## Examples

### Simple Text

```bash
typf render "Hello World" -o hello.png
```

### Custom Font & Size

```bash
typf render "Typography" -f /path/to/font.ttf -s 128 -o big.png
```

### Arabic Text (RTL)

```bash
typf render "مرحبا" \
  -f arabic.ttf \
  --shaper hb \
  --language ar \
  --script Arab \
  --direction rtl \
  -o arabic.png
```

### SVG Export

```bash
typf render "Vector" -f font.ttf -O svg -o vector.svg
```

### Colored Output

```bash
typf render "Red on White" \
  -c FF0000FF \
  -b FFFFFFFF \
  -o colored.png
```

### With Font Features

```bash
typf render "Ligatures" \
  -f font.ttf \
  -F "liga,kern,dlig" \
  -o ligatures.png
```

### Batch Processing

```bash
# Create jobs
cat > jobs.jsonl << 'EOJ'
{"text": "Title", "size": 72, "output": "title.png"}
{"text": "Subtitle", "size": 48, "output": "subtitle.png"}
{"text": "Body", "size": 16, "output": "body.png"}
EOJ

# Process
typf batch -i jobs.jsonl -o ./rendered/
```

---

## Backend Selection

### Auto-Detection

When using `--shaper auto` or `--renderer auto`, TYPF selects the best available backend:

**macOS**:
- Shaper: CoreText → HarfBuzz → None
- Renderer: CoreGraphics → TinySkia → Orge

**Windows**:
- Shaper: DirectWrite → HarfBuzz → None
- Renderer: DirectWrite → TinySkia → Orge

**Linux**:
- Shaper: HarfBuzz → None
- Renderer: TinySkia → Orge

### Manual Selection

Available backends shown via `typf info`:

```bash
$ typf info --shapers
Shapers:
  none              - No shaping (direct character mapping)
  hb                - HarfBuzz (Unicode-aware text shaping)

$ typf info --renderers
Renderers:
  orge              - Orge (pure Rust, monochrome/grayscale)
```

---

## Troubleshooting

### "Unknown backend" Error

**Problem**: `Error: Unknown or unavailable shaper: hb`

**Solution**: Backend not compiled. Check available backends:
```bash
typf info --shapers
```

### SVG Export Fails

**Problem**: `Error: SVG export requires a real font file`

**Solution**: Provide a font file with `-f`:
```bash
typf render "Text" -f font.ttf -O svg -o out.svg
```

### Unicode Escapes Not Working

**Problem**: `\u1234` appears literally in output

**Solution**: Ensure proper shell escaping:
```bash
# Wrong
typf render "\u1234" -o out.png

# Correct
typf render "\\u1234" -o out.png
# or
typf render '\u1234' -o out.png
```

### Colors Not Working

**Problem**: Colors appear wrong

**Solution**: Use RRGGBB or RRGGBBAA hex format:
```bash
# Wrong
-c "red" -b "white"

# Correct
-c FF0000FF -b FFFFFFFF
```

---

## Support

- **Documentation**: See `docs/` directory
- **Issues**: https://github.com/fontlaborg/typf/issues
- **Examples**: See `examples/` directory

---

**Migration Complete**: Both Rust and Python CLIs now follow the unified specification with full feature parity!
