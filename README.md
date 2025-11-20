# TypF

[![CI](https://github.com/fontlaborg/typf/workflows/CI/badge.svg)](https://github.com/fontlaborg/typf/actions)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

Your text looks wrong. Arabic renders backwards, Hindi characters break, Thai glyphs collide. TypF fixes this in under a millisecond.

Render "Hello, مرحبا, 你好!" correctly the first time.

> Note: TypF is a community project by [FontLab](https://www.fontlab.org/) and is currently published under an [evaluation license](./LICENSE).

## Quick start

```bash
git clone https://github.com/fontlaborg/typf.git
cd typf
cargo build --release

./target/release/typf render "Hello, World!" -o hello.png -s 48
open hello.png
```

That's it. Your text is rendered.

## What it does

- **Complex scripts**: Arabic, Hindi, Thai with proper shaping
- **Mixed languages**: RTL and LTR in the same line  
- **Fast output**: PNG, SVG, JSON in <1ms
- **Small footprint**: 500KB minimal build
- **Multiple backends**: Choose speed vs quality

## How it works

1. Text enters the pipeline
2. Unicode scripts get detected  
3. Fonts match and fallback when needed
4. Characters become positioned glyphs
5. Renderer draws them with SIMD
6. Export writes your format

## Choose your setup

| Need | Command | Speed | Quality |
|------|---------|-------|---------|
| Fastest data | `none + JSON` | 25K ops/sec | Glyph data only |
| Complex scripts | `harfbuzz + zeno` | 3K ops/sec | 247 grayscales |
| macOS best | `coretext + coregraphics` | 4K ops/sec | 254 levels |
| Pure Rust | `harfbuzz + orge` | 2K ops/sec | 25 levels (mono) |

## Build options

```bash
# Minimal (500KB)
cargo build --release --no-default-features --features minimal

# Everything  
cargo build --release --all-features

# SVG export (23× faster than PNG)
cargo build --release --features export-svg
./target/release/typf render "Scalable" -o out.svg -s 48
```

## CLI usage

The unified CLI supports both Rust (`typf`) and Python (`typfpy`) with identical syntax.

**Show available backends:**
```bash
typf info
typf info --shapers --renderers --formats
```

**Basic rendering:**
```bash
# Simple text
typf render "Hello World" -o output.png

# With custom font and size
typf render "Typography" -f /path/to/font.ttf -s 128 -o big.png

# SVG output
typf render "Vector" -f font.ttf -O svg -o vector.svg
```

**Advanced options:**
```bash
# Arabic text with proper shaping
typf render "مرحبا بالعالم" \
  -f arabic.ttf \
  --shaper hb \
  --language ar \
  --script Arab \
  --direction rtl \
  -o arabic.png

# Custom colors (RRGGBBAA hex)
typf render "Colored Text" \
  -c FF0000FF \
  -b FFFFFFFF \
  -o colored.png

# Font features
typf render "Ligatures" \
  -f font.ttf \
  -F "liga,kern,dlig" \
  -o features.png

# Unicode escapes
typf render "Wave \u{1F44B}" -o emoji.png
```

**Batch processing:**
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

**Python CLI** (identical syntax):
```bash
typfpy info
typfpy render "Hello" -f font.ttf -o output.png -s 72
```

See [CLI_MIGRATION.md](./CLI_MIGRATION.md) for complete documentation.

## Use in code

**Rust:**
```rust
use typf::{Shaper, Renderer, Exporter};

let text = "Hello, مرحبا";
let shaped = shaper.shape(text, font, &params)?;
let rendered = renderer.render(&shaped, font, &render_params)?;
let exported = exporter.export(&rendered)?;
```

**Python:**
```python
import typf

result = typf.render_text("Hello, مرحبا", font_path="arial.ttf")
result.save("output.png")
```

## Test your system

```bash
cd typf-tester
python typfme.py bench
```

Tests all backend combos on your hardware. Results go to `output/`.

## Status

**v2.0.0** - Production ready. All features work:

- ✅ 6-stage pipeline
- ✅ 4 shapers, 5 renderers (20 combinations)
- ✅ PNM, PNG, SVG, JSON export
- ✅ Unified CLI (Rust + Python)
- ✅ Python bindings (PyO3)
- ✅ Linux, macOS, Windows, WASM
- ✅ 206 unit tests + 240 integration tests pass

## Limits

**Bitmap width**: ~10,000 pixels max
- 48px font: ~200 characters  
- 24px font: ~400 characters
- 12px font: ~800 characters

Fix: Use smaller fonts, line wrapping, or SVG (no width limit).

## Troubleshooting

**Build errors:**
- "undeclared type" → `cargo build --all-features`
- "Package not found" → `cargo build --no-default-features --features minimal`

**Runtime errors:**  
- "no attribute 'Typf'" → Rebuild Python bindings
- "Invalid bitmap dimensions" → Text too wide, use smaller font or SVG
- "Font not found" → Check path and format (TrueType/OpenType)

## Learn more

- [Documentation](src_docs/) - 24 chapters
- [Examples](examples/README.md) - Working code samples  
- [Contributing](CONTRIBUTING.md) - Development setup
- [PLAN.md](PLAN.md) - Roadmap and architecture

## License

[EVALUATION LICENSE](./LICENSE)
