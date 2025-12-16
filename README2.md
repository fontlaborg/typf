The `typf` project is a modular text rendering library built in Rust, designed to accurately process complex scripts and provide high-performance output across diverse platforms using a customizable pipeline architecture.

## 1. Purpose: What Typf Does and Why

The primary function of `typf` is to correctly transform complex Unicode text into positioned glyphs and ultimately into rendered pixels or vector paths. The core problem it solves is handling issues like Arabic text rendering backwards, Hindi characters breaking, and Thai glyphs colliding, ensuring correct complex script behavior and mixing of right-to-left (RTL) and left-to-right (LTR) languages in the same line.

The motivation for its architecture stems from the limitations of monolithic solutions: standard shaping libraries like HarfBuzz cannot render pixels, while rendering frameworks like Skia are large (e.g., 10MB) or platform APIs lock the user to a single operating system. `Typf` resolves this by providing swappable backends implementing standardized traits, allowing users to select components optimized for speed (e.g., Opixa), quality (e.g., Zeno/Skia), or platform integration (e.g., CoreText/CoreGraphics). This results in highly optimized builds, with the minimal configuration weighing less than 500KB.

## 2. Mechanism: The Six-Stage Pipeline

The entire operation is structured around a mandatory **six-stage pipeline** that guarantees separation of concerns and correctness:

1.  **Input Parsing**: Raw text and parameters (language, script) are processed into structured data.
2.  **Unicode Processing** (`typf-unicode`): Detects scripts, handles bidirectional text, and performs text segmentation.
3.  **Font Selection** (`typf-fontdb`): Finds and loads the correct font, handling font fallback and TrueType Collection (TTC) indexing.
4.  **Shaping**: Converts characters into positioned glyphs (`ShapingResult`), applying OpenType features and script rules.
5.  **Rendering**: Converts positioned glyphs into pixel data or vectors (`RenderOutput`), handling colors and anti-aliasing.
6.  **Export**: Encodes the rendered output into file formats like PNG, SVG, or JSON.

This sequence is managed by the central `Pipeline` structure defined in `typf-core`, which chains components implementing the `Stage` trait.

## 3. Internal Structure and Crate Organization

The codebase is organized as a Rust workspace (`Cargo.toml`) composed of modular crates designed for isolation and feature gating. The workspace explicitly excludes external repositories and fuzz targets.

### A. Core Crates (`crates/`)

The fundamental logic, shared data types, and infrastructure reside here.

| Crate | Role and Contents |
| :--- | :--- |
| `typf-core` | Defines the central types, the `Pipeline`, and the core abstraction traits (`Shaper`, `Renderer`, `Exporter`, `FontRef`, `Stage`). It also contains the cache implementation (`cache.rs`). |
| `typf-fontdb` | Handles font loading from files or memory and implements the `FontRef` trait, providing glyph metrics and TTC support. |
| `typf-unicode` | Provides Unicode processing, including bidirectional text handling, script detection, and normalization. |
| `typf-export` | Manages file encoding, notably PNG export, and format validation. |
| `typf-cli` | The main command-line application wrapper. |
| `typf` | The main library facade crate, typically re-exporting core functionality. |

### B. Backends (`backends/`)

These crates implement the primary logic using specialized algorithms or external libraries. They rely on feature flags for conditional compilation.

| Type | Crate(s) | Description and Notes |
| :--- | :--- | :--- |
| **Shapers** | `typf-shape-hb`, `typf-shape-icu-hb`, `typf-shape-ct`, `typf-shape-none` | Implement the `Shaper` trait. HarfBuzz is the industry standard; CoreText (`ct`) is macOS native; `icu-hb` adds enhanced Unicode features. |
| **Renderers** | `typf-render-opixa`, `typf-render-skia`, `typf-render-zeno`, `typf-render-cg`, `typf-render-json`, `typf-render-svg`, `typf-render-vello-cpu`, `typf-render-vello` | Implement the `Renderer` trait. Opixa is pure Rust SIMD rasterization. Skia, Zeno, and Vello offer varying levels of quality and color glyph support (COLR/SVG/bitmap). |
| **Linra** | `typf-os-mac`, `typf-os-win` | Implement the `LinraRenderer` trait (single-pass shape+render) for platform APIs like CoreText (macOS) and DirectWrite (Windows). |

### C. Development Infrastructure

The project emphasizes verification and tooling:

*   **Fuzzing** (`fuzz/`): Includes targets like `fuzz_font_parse` and `fuzz_harfbuzz_shape` to find crashes using malformed inputs.
*   **Benchmarking** (`benches/`, `typf-tester/`): Provides comprehensive performance testing scripts and analysis tools (`typfme.py`, `compare_quality.py`).
*   **Caching**: Features a multi-level cache (L1 HashMap + L2 LRU) to cache shaped results and glyph data, significantly speeding up repeated operations. Caching is opt-in by default.

## 4. Public API Description

The primary interaction points are the strongly-typed Rust API and the idiomatic Python bindings.

### A. Rust API

The Rust API exposes core functionality via traits and the `Pipeline` builder pattern.

#### Core Abstraction Traits

All custom components must implement one of these traits from `typf-core`:

*   `FontRef`: Defines methods to access font data, metrics (units per em), and character-to-glyph mapping.
*   `Shaper`: The contract for transforming raw text into `ShapingResult` (positioned glyphs).
*   `Renderer`: The contract for converting `ShapingResult` into `RenderOutput` (pixel or vector data).
*   `Exporter`: The contract for converting `RenderOutput` into raw file bytes.
*   `LinraRenderer`: An optional trait used by platform-specific backends (e.g., CoreText) to perform shaping and rendering in a single, optimized operation.

#### Pipeline Integration

Users typically interact using the `Pipeline` builder defined in `typf-core`:

```rust
// Pipeline Builder Pattern
let pipeline = Pipeline::builder()
    .shaper(Arc::new(HarfBuzzShaper::new()))
    .renderer(Arc::new(OpixaRenderer::new()))
    .exporter(Arc::new(PngExporter::new()))
    .build()?;

// Execution
let rendered = pipeline.process("Hello, World!", font, &ShapingParams::default(), &RenderParams::default())?;
```

The `ShapingParams` and `RenderParams` structs are used to pass configuration details, such as font size, OpenType features (`liga`), color palette, and desired `GlyphSourcePreference` (e.g., preferring COLR over SVG glyphs).

### B. Python Bindings (`typfpy`)

The Python bindings wrap the Rust core via PyO3, providing a comprehensive, idiomatic interface.

#### Main Rendering Class

The central object is `typfpy.Typf`, which instantiates the rendering pipeline:

```python
import typfpy
from typfpy import Typf, export_image

# 1. Initialize the engine, selecting backends
engine = Typf(shaper="harfbuzz", renderer="opixa")

# 2. Render text to an image dict
image_data = engine.render_text(
    "Typography is beautiful",
    font_path="/path/to/font.ttf",
    size=48,
    variations={"wght": 700.0} # Variable font axes
)

# 3. Export the raw image data to a file format
png_bytes = export_image(image_data, format="png")
```

The module also exposes helper functions like `render_simple()` (which uses a stub font for quick tests) and utility classes like `FontInfo` for accessing font metrics and metadata.

#### CLI Interface

A command-line interface, `typfpy` (or `typf` when the Rust binary is installed), mirrors the API, supporting rendering, batch processing, and retrieving information about available backends.

```bash
typfpy render "Hello World" -f font.ttf --shaper hb --renderer skia -o output.png
```

This ensures maximum utility and parity between the Rust and Python execution environments.
