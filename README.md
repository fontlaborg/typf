# Typf v2.4.5

[![CI](https://github.com/fontlaborg/typf/workflows/CI/badge.svg)](https://github.com/fontlaborg/typf/actions)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

Your text looks wrong. Arabic renders backwards, Hindi characters break, Thai glyphs collide. Typf fixes this in under a millisecond.

Render "Hello, مرحبا, 你好!" correctly the first time.

> Note: Typf is a community project by [FontLab](https://www.fontlab.org/) and is currently published under an [evaluation license](./LICENSE).

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
| Pure Rust | `harfbuzz + opixa` | 2K ops/sec | 25 levels (mono) |

## Backend comparison

### Shapers

| Backend | Scripts | Features | Performance | Platform |
|---------|---------|----------|-------------|----------|
| **none** | Latin only | Simple LTR | 25K ops/sec | All |
| **harfbuzz** | All (200+) | Full OpenType | 4K ops/sec | All |
| **icu-hb** | All + normalization | Unicode + OpenType | 3.5K ops/sec | All |
| **coretext** | All | Native macOS | 4.5K ops/sec | macOS only |

### Renderers

| Backend | Anti-alias | Color | Output | Performance | Platform |
|---------|------------|-------|--------|-------------|----------|
| **opixa** | Monochrome | No | Bitmap | 2K ops/sec | All (pure Rust) |
| **skia** | 256 levels | Yes (COLR/SVG/bitmap) | Bitmap/SVG | 3.5K ops/sec | All |
| **zeno** | 256 levels | Yes (COLR/SVG/bitmap) | Bitmap/SVG | 3K ops/sec | All (pure Rust) |
| **vello-cpu** | 256 levels | Yes (COLR/bitmap) | Bitmap | 3.5K ops/sec | All (pure Rust) |
| **vello** | 256 levels | Yes (COLR/bitmap) | Bitmap | 10K+ ops/sec | GPU required |
| **coregraphics** | 256 levels | Yes (sbix/COLR) | Bitmap | 4K ops/sec | macOS only |
| **json** | N/A | N/A | JSON data | 25K ops/sec | All |

### GPU Renderers (Vello)

The Vello renderers use compute-centric GPU rendering for maximum performance:

| Backend | Hardware | Performance | Use Case |
|---------|----------|-------------|----------|
| **vello** | GPU (Metal/Vulkan/DX12) | 10K+ ops/sec | High-throughput, large text |
| **vello-cpu** | CPU only | 3.5K ops/sec | Server, no GPU |

```bash
# GPU rendering (requires GPU)
typf render "Hello" --renderer vello -o out.png

# CPU rendering (pure Rust, no GPU)
typf render "Hello" --renderer vello-cpu -o out.png
```

Both use the [Vello](https://github.com/linebender/vello) engine with skrifa for font parsing. Build with `--features render-vello` or `--features render-vello-cpu`.

### Linra Renderers (Single-Pass)

For maximum performance, linra renderers combine shaping and rendering in a single OS call:

| Backend | Speed vs Separate | Platform | Use Case |
|---------|------------------|----------|----------|
| **coretext-linra** | 2.52x faster | macOS only | High-throughput rendering |

The linra renderer bypasses the intermediate glyph extraction step, allowing CoreText to optimize the entire pipeline internally.

### Export Formats

| Format | Size | Antialiasing | Use Case |
|--------|------|--------------|----------|
| **PNG** | Small | Yes | Web, print |
| **SVG** | Scalable | Vector | Icons, logos |
| **JSON** | Smallest | N/A | Analysis, debug |
| **PGM/PPM** | Large | Yes/No | Testing, legacy |

### Font Feature Support Matrix

| Feature | none | harfbuzz | icu-hb | coretext |
|---------|------|----------|--------|----------|
| OpenType Layout (GPOS/GSUB) | ❌ | ✅ | ✅ | ✅ |
| OpenType Variations (fvar) | ✅ | ✅ | ✅ | ✅ |
| Complex scripts (Arabic, Devanagari) | ❌ | ✅ | ✅ | ✅ |
| Emoji (segmentation) | ❌ | ⚠️ | ✅ | ✅ |

### Glyph Format Support

| Format | opixa | skia | zeno | vello-cpu | vello | coregraphics | svg |
|--------|-------|------|------|-----------|-------|--------------|-----|
| TrueType outlines (glyf) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| CFF outlines (CFF ) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| CFF2 outlines (CFF2) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Variable fonts (gvar) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| COLR v0 (layered colors) | ❌ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ❌ |
| COLR v1 (gradients) | ❌ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ❌ |
| SVG glyphs (SVG table) | ❌ | ✅ | ✅ | ❌ | ❌ | ⚠️ | ❌ |
| Bitmap glyphs (CBDT/sbix) | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |

**Legend:** ✅ Full support | ⚠️ Partial/via OS | ❌ Not supported

> **Note:** Color glyph support in skia/zeno requires the `resvg` feature for SVG glyphs and `bitmap` feature for CBDT/sbix. Vello renderers use skrifa's native color font support.

## Caching

Typf includes two-level caches for shaping and rendering results. **Caching is disabled by default** to ensure predictable behavior and memory usage.

### Enable caching

**Rust:**
```rust
use typf::cache_config;

// Enable caching for repeated operations
cache_config::set_caching_enabled(true);

// Check status
if cache_config::is_caching_enabled() {
    println!("Caching is ON");
}

// Disable when done
cache_config::set_caching_enabled(false);
```

**Python:**
```python
import typf

typf.set_caching_enabled(True)   # Enable
typf.is_caching_enabled()        # Check: returns True/False
typf.set_caching_enabled(False)  # Disable
```

**Environment variable:**
```bash
TYPF_CACHE=1 ./your_app          # Enable at startup
```

### When to use caching

| Scenario | Caching | Reason |
|----------|---------|--------|
| One-shot CLI renders | Off (default) | No repeated work |
| Interactive UI | On | Same text re-rendered often |
| Batch processing different texts | Off | Each text unique |
| Batch processing same text/fonts | On | Cache hits save time |
| Memory-constrained environment | Off | Caches use memory |

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

The linra CLI supports both Rust (`typf`) and Python (`typfpy`) with identical syntax.

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

# Glyph source control (color fonts)
# Prefer COLR over SVG glyphs
typf render "Emoji" -f color.ttf \
  --glyph-source prefer=colr1,colr0,svg \
  -o emoji.png

# Disable color glyphs (force outline rendering)
typf render "Text" -f color.ttf \
  --glyph-source deny=colr0,colr1,svg,sbix,cbdt,ebdt \
  -o mono.png
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

### Benchmark CLI

For comprehensive performance testing with JSON output:

```bash
# Quick sanity check (level 0)
cargo run -p typf-bench --release -- -i test-fonts -l 0

# Full benchmark (level 1-5, higher = more extensive)
cargo run -p typf-bench --release -- -i /path/to/fonts -l 2

# JSON output for CI comparison
cargo run -p typf-bench --release -- -i test-fonts -l 1 --json -o benchmark.json
```

The benchmark tool tests all shaper × renderer combinations across fonts, sizes, and text samples.

## Status

**v2.5.x** - Production ready. All features work:

- ✅ 6-stage pipeline
- ✅ 4 shapers, 7 renderers (28 combinations)
- ✅ PNM, PNG, SVG, JSON export
- ✅ Linra CLI (Rust + Python)
- ✅ Python bindings (PyO3)
- ✅ Linux, macOS, Windows, WASM
- ✅ 348+ tests passing across workspace
- ✅ macOS native backends (CoreText + CoreGraphics)
- ✅ Comprehensive backend documentation and examples
- ✅ COLR v0/v1 color glyph support (skia/zeno/vello)
- ✅ SVG table glyph support via resvg (skia/zeno)
- ✅ Bitmap glyph support (sbix/CBDT/EBDT)
- ✅ Configurable glyph source selection (`--glyph-source`)
- ✅ GPU-accelerated rendering via Vello (Metal/Vulkan/DX12)
- ✅ High-quality CPU rendering via Vello CPU

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

- [Quickstart](QUICKSTART.md) - Use typf in your Rust project
- [Architecture](ARCHITECTURE.md) - Pipeline, backends, and data flow
- [Documentation](src_docs/) - 24 chapters
- [Examples](examples/README.md) - Working code samples
- [Contributing](CONTRIBUTING.md) - Development setup
- [PLAN.md](PLAN.md) - Roadmap

## License

[EVALUATION LICENSE](./LICENSE)
