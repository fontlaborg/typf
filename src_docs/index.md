# TYPF: Modern Font Rendering Engine

> Production-ready, cross-platform font rendering with Rust performance and Python convenience

## What is TYPF?

TYPF is a modern, cross-platform font rendering engine providing unified text layout and rasterization. It's built in Rust for performance and safety, with Python bindings for ease of use.

**Key Design Goals:**
- **Performance:** Sub-millisecond rendering, lock-free concurrency, zero-copy font loading
- **Correctness:** Pixel-perfect output, comprehensive testing, fuzzing-ready
- **Cross-Platform:** Native backends for macOS (CoreText), Windows (DirectWrite), Linux (HarfBuzz)
- **Flexible Output:** PNG, SVG, NumPy arrays, PGM, raw bitmaps

---

## Features

### Core Capabilities

✅ **Multiple Shaping Backends**
- CoreText (macOS native)
- DirectWrite (Windows native)
- ICU + HarfBuzz (cross-platform fallback)
- Automatic backend selection based on platform

✅ **Multiple Rasterizers**
- `orge` - Custom CPU rasterizer (F26Dot6 fixed-point, scan conversion)
- `tiny-skia` - Vector renderer (feature-gated)
- `zeno` - Alternative rasterizer (feature-gated)

✅ **Modern Font Stack**
- Built exclusively on `skrifa` + `read-fonts`
- Full OpenType support (TTF, CFF, CFF2)
- Variable font support (wght, wdth, and all registered axes)
- Named instance support

✅ **Output Formats**
- PNG (via `resvg` + `png` crate)
- SVG paths with `kurbo`
- NumPy arrays (Python bindings)
- PGM (P5 format)
- Raw RGBA/BGRA bitmaps

✅ **Advanced Features**
- COLRv1/CPAL color font support
- Gradients and clip paths
- Complex script shaping (Arabic, Devanagari, CJK)
- Bidirectional text (via `unicode_bidi`)
- Text segmentation (ICU-based)

---

## Architecture

### Project Structure

```
typh/
├── backends/               # Platform-specific rendering
│   ├── typf-core/         # Shared traits, types, caching (1,086 lines)
│   ├── typf-icu-hb/       # HarfBuzz+ICU backend (~2,000 lines)
│   ├── typf-orge/         # Custom rasterizer (~500 lines)
│   ├── typf-mac/          # CoreText backend (~800 lines)
│   ├── typf-win/          # DirectWrite backend (~1,000 lines)
│   ├── typf-pure/         # Minimal pure-Rust fallback (~350 lines)
│   └── typf-zeno/         # Zeno rasterizer (~200 lines)
├── crates/                # Modular components
│   ├── typf-api/          # Public API facade
│   ├── typf-batch/        # Batch job processing
│   ├── typf-fontdb/       # Font discovery & loading
│   ├── typf-render/       # Output utilities (SVG/PNG)
│   ├── typf-shaping/      # Shaping helpers
│   └── typf-unicode/      # Text segmentation
├── python/                # PyO3 bindings
│   ├── src/lib.rs         # Rust FFI layer
│   └── typf/              # Python wrapper
├── typf-cli/              # Command-line tool
├── tests/                 # Integration tests
└── examples/              # Rust & Python examples
```

### Caching Architecture

**Three-Layer Caching:**

1. **Backend Font Cache** (Per-backend, LRU)
   - CoreText: CTFont instances (64-128 capacity)
   - DirectWrite: IDWriteFont objects (64-128 capacity)
   - HarfBuzz: hb_face_t objects (64-128 capacity)

2. **Shape Result Cache** (Global, Multi-Shard)
   - Implementation: 16-shard DashMap + per-shard LRU
   - Key: (text, font_key, size, features)
   - Benefit: Eliminates lock contention on concurrent workloads
   - Capacity: Configurable (1,000-10,000 entries typical)

3. **System Font Database** (Global, OnceCell)
   - Location: typf-fontdb
   - DashMap-backed for thread-safe lookups
   - Loads system fonts + custom directories once

**Memory Management:**
- Arc-based shared ownership for font data
- memmap2 for file-backed zero-copy loading
- Mmap kept alive via FontKey references
- Explicit `clear_cache()` methods on all backends

---

## Installation

### From Source

**Prerequisites:**
- Rust 1.70+ (`rustup install stable`)
- Python 3.12+ (for Python bindings)
- Platform-specific dependencies:
  - macOS: Xcode command-line tools
  - Windows: Visual Studio Build Tools
  - Linux: HarfBuzz + FreeType development packages

**Rust Library:**
```bash
cd github.fontlaborg/typf
cargo build --release --workspace
cargo test --workspace --all-features  # Run tests
```

**Python Bindings:**
```bash
cd python
pip install maturin
maturin develop --release  # Development install
# OR
maturin build --release    # Build wheel
pip install target/wheels/typf-*.whl
```

**CLI Tool:**
```bash
cargo install --path typf-cli
typf --help
```

### Feature Flags

Control optional functionality via Cargo features:

```toml
[dependencies]
typf = { version = "*", features = ["mac", "tiny-skia-renderer"] }
```

**Available features:**
- `mac` - CoreText backend (macOS only, default on macOS)
- `windows` - DirectWrite backend (Windows only, default on Windows)
- `icu` - HarfBuzz+ICU backend (cross-platform, default)
- `tiny-skia-renderer` - tiny-skia rasterizer (optional)
- `orge` - Custom orge rasterizer (experimental)

---

## Usage

### Rust API

**Basic rendering:**
```rust
use typf::prelude::*;

// Auto-select best backend for platform
let backend = Backend::auto_select()?;

// Create font specification
let font = Font::new("Arial", 24.0)
    .weight(700)  // Bold
    .style(FontStyle::Italic);

// Render text
let options = RenderOptions::default()
    .format(RenderFormat::PNG)
    .size(800, 600);

let result = backend.render_text("Hello, TYPF!", &font, &options)?;

// result.data contains PNG bytes
std::fs::write("output.png", &result.data)?;
```

**Variable fonts:**
```rust
let font = Font::new("RobotoFlex", 24.0)
    .variation("wght", 800.0)
    .variation("wdth", 125.0);

let result = backend.render_text("Variable!", &font, &options)?;
```

**Font features:**
```rust
let font = Font::new("OpenSans", 18.0)
    .feature("liga", 1)   // Enable ligatures
    .feature("smcp", 1);  // Small caps

let result = backend.render_text("fi fl", &font, &options)?;
```

**SVG output:**
```rust
let options = RenderOptions::default()
    .format(RenderFormat::SVG);

let result = backend.render_text("SVG Text", &font, &options)?;
let svg = String::from_utf8(result.data)?;
```

### Python API

**Basic rendering:**
```python
from typf import TextRenderer, Font, RenderFormat

# Auto-select backend
renderer = TextRenderer()

# Create font
font = Font("Arial", size=24.0)

# Render text
result = renderer.render(
    text="Hello from Python!",
    font=font,
    format=RenderFormat.PNG
)

# result.data is bytes (PNG)
with open("output.png", "wb") as f:
    f.write(result.data)
```

**Variable fonts + NumPy:**
```python
import numpy as np
from PIL import Image

font = Font("RobotoFlex", size=48.0)
font.variation("wght", 700)
font.variation("wdth", 100)

result = renderer.render("Variable", font, format=RenderFormat.RAW)

# Convert to NumPy array
arr = np.frombuffer(result.data, dtype=np.uint8)
arr = arr.reshape(result.height, result.width, 4)  # RGBA

# Convert to PIL Image
img = Image.fromarray(arr)
img.show()
```

**Batch rendering:**
```python
jobs = [
    {"font": "Arial", "text": "Job 1", "size": 24},
    {"font": "Times", "text": "Job 2", "size": 32},
]

results = renderer.render_batch(jobs)
for i, result in enumerate(results):
    with open(f"output_{i}.png", "wb") as f:
        f.write(result.data)
```

### CLI

**Single render:**
```bash
# Render to PNG
typf render \
  --font=/path/to/font.ttf \
  --text="Hello, CLI!" \
  --size=48 \
  --output=output.png

# Render with variable font axes
typf render \
  --font=RobotoFlex.ttf \
  --text="Variable" \
  --axes="wght=700,wdth=100" \
  --output=variable.png
```

**Batch processing (JSONL):**
```bash
# Create job file
echo '{"font": "Arial", "text": "Line 1", "size": 24}' > jobs.jsonl
echo '{"font": "Times", "text": "Line 2", "size": 32}' >> jobs.jsonl

# Process batch
typf batch < jobs.jsonl

# Streaming mode (memory-efficient)
cat jobs.jsonl | typf stream
```

**Output formats:**
```bash
# PNG (default)
typf render --font=font.ttf --text="PNG" --format=png --output=out.png

# SVG
typf render --font=font.ttf --text="SVG" --format=svg --output=out.svg

# PGM (grayscale)
typf render --font=font.ttf --text="PGM" --format=pgm --output=out.pgm

# Metrics only (JSON)
typf render --font=font.ttf --text="Metrics" --format=metrics
```
