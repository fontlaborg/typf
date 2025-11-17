# TYPF: Modern Font Rendering Engine

> Production-ready, cross-platform font rendering with Rust performance and Python convenience

[![Status](https://img.shields.io/badge/status-production--ready-green)]()
[![Language](https://img.shields.io/badge/rust-1.70+-orange)]()
[![Python](https://img.shields.io/badge/python-3.12+-blue)]()

**For overall project coordination:** See root [README.md](../../README.md) | [PLAN.md](../../PLAN.md) | [TODO.md](../../TODO.md)

---

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
typf/
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

### Backend Comparison

| Backend | Platform | Performance | Completeness | Use Case |
|---------|----------|-------------|--------------|----------|
| **CoreText** | macOS | Excellent | ✅ Full | macOS primary |
| **DirectWrite** | Windows | Excellent | ✅ Full | Windows primary |
| **HarfBuzz+ICU** | All | Good | ✅ Full | Cross-platform fallback |
| **orge** | All | Excellent* | ⚠️ Partial | Custom rasterization (in progress) |
| **tiny-skia** | All | Good | ✅ Full | Vector rendering (feature-gated) |
| **zeno** | All | Good | ⚠️ Partial | Alternative rasterizer |
| **pure** | All | Basic | ⚠️ Minimal | WASM compatibility fallback |

*orge core algorithm complete; backend integration pending

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

> ⚠️ **Important**: Python bindings MUST be built inside an active virtual environment. System Python builds will fail with linker errors.

```bash
# 1. Create and activate virtual environment
cd github.fontlaborg/typf
uv venv --python 3.12      # or: python3.12 -m venv .venv
source .venv/bin/activate  # macOS/Linux
# .venv\Scripts\activate   # Windows

# 2. Install maturin in the venv
uv pip install maturin     # or: pip install maturin

# 3. Build the bindings
cd python
maturin develop --release --features "python,icu,mac"  # macOS
# maturin develop --release --features "python,icu"    # Linux
# maturin develop --release --features "python,windows" # Windows

# 4. Verify installation
python -c "import typf; print(typf.__version__)"
```

**Troubleshooting:**

If you encounter linker errors or missing Python symbols:
- **macOS**: Install Xcode CLI tools: `xcode-select --install`
- **Linux**: Install Python dev headers: `sudo apt install python3-dev` (Debian/Ubuntu) or `sudo dnf install python3-devel` (Fedora/RHEL)
- **Windows**: Install Visual Studio Build Tools with Python development workload
- **All platforms**: Ensure you're inside an activated virtual environment before running `maturin develop`

To build a distributable wheel:
```bash
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

---

## Performance

**Rendering Speed:**
- Typical glyph: Sub-millisecond
- Complex scripts (Arabic, Devanagari): 1-5ms
- Batch processing: Rayon-parallelized across CPU cores

**Memory:**
- Zero-copy font loading via memmap2
- Arc-based shared ownership (no unnecessary clones)
- Multi-shard cache eliminates lock contention

**Caching:**
- First render: Font load + shape + rasterize (~5-20ms)
- Cached render: Shape lookup only (~0.1-1ms)
- Cache hit rate: 80-95% typical for repeated text

---

## Testing

**Run all tests:**
```bash
cargo test --workspace --all-features
```

**Run platform-specific tests:**
```bash
# macOS only
cargo test --features mac

# Windows only
cargo test --features windows

# HarfBuzz only
cargo test --features icu
```

**Run integration tests:**
```bash
cargo test --test integration
```

**Python binding tests:**
```bash
cd python
pytest tests/
```

**Benchmarks:**
```bash
cargo bench --workspace
```

---

## Current Status

### Production-Ready ✅

- All 3 platform backends (CoreText, DirectWrite, HarfBuzz)
- Multi-shard LRU caching
- Python bindings (PyO3) with automatic backend selection
- CLI with batch/stream/render commands
- 38+ integration tests passing
- SVG/PNG output with COLRv1 color font support

### In Progress ⏳

- Orge rasterizer backend integration
- FFI panic handling (std::panic::catch_unwind wrappers)
- Visual regression framework (SSIM-based)
- Comprehensive documentation

### Planned 📋

- SIMD-accelerated fixed-point math (SSE2/AVX2/Neon)
- cargo-fuzz + cargo-miri in CI
- Unified typf_error::Error enum
- seccomp sandboxing for untrusted fonts
- Publishto crates.io

---

## Development

**Guidelines:** See [CLAUDE.md](./CLAUDE.md) for development workflow, coding standards, and testing requirements.

**Quick commands:**
```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --workspace --all-features -- -D warnings

# Build release
cargo build --workspace --release

# Build Python bindings
cd python && maturin develop --release
```

**For tasks and roadmap:** See root [TODO.md](../../TODO.md) (look for `**(typf)**` prefix)

---

## Contributing

We follow a philosophy of **ruthless minimalism** and **test-driven development**:

1. **Write tests first** (RED → GREEN → REFACTOR)
2. **Minimize** code (delete > add)
3. **No panics** in library code (use `Result<T, TypfError>`)
4. **Document** all public APIs (rustdoc with examples)
5. **Benchmark** performance-critical changes

See [CLAUDE.md](./CLAUDE.md) for detailed contribution guidelines.

---

## License

See [LICENSE](./LICENSE) file.

---

made by FontLab https://www.fontlab.com/

---


---

**Last Updated:** November 16, 2025
**Status:** Production-ready; focused on hardening and optimization
**Total Code:** ~15,000 lines Rust + 500 lines Python
