<!-- this_file: PLANSTEPS/02-external-ecosystems.md -->

## 1. Analysis of External Ecosystems

### A. Rust Text Layout and Graphics

The modern Rust ecosystem favors decoupled components, validating the modular architecture of `typf`.

| Ecosystem Component | Role / Package | Data Requirement | Typf Integration Focus |
| :--- | :--- | :--- | :--- |
| **Shaping Core** | HarfRust, Allsorts, Swash::Shape | Sequence of positioned glyphs, cluster IDs | Standardized Intermediate Data Structure (SIDS) |
| **High-Level Layout** | Cosmic Text | Consume shaped glyph structs (`ShapeGlyph`) | Idiomatic conversion (`From`/`Into` traits) |
| **Low-Level Glyphs** | Ab Glyf | Glyph references, positional offsets (`point`) | Seamless conversion from SIDS |
| **GPU Rendering** | Egui, Iced, Wgpu | Structured vertex buffers (`RenderMesh`) | Zero-copy vertex generation (`GpuRenderer`) |
| **Vector Tessellation** | Pathfinder, Lyon | Raw vector geometry/path operations | Geometry generation capability in Stage 5 |
| **Font Parsing** | Ttf-parser, Swash::Scale | Raw font binary data, glyph outlines | Raw buffer access (`FontRef::as_bytes`) |

### B. Python FFI and Document Generation

Python integration relies on using Foreign Function Interface (FFI) for maximum efficiency, avoiding serialization overhead by using memory views.

| Python Target | Role / Package | Data Requirement | Typf Integration Focus |
| :--- | :--- | :--- | :--- |
| **Drawing/Layout** | Pycairo, Pango | C-compatible `cairo_glyph_t` structure (index, x, y doubles) | Zero-copy NumPy view via `#[repr(C)]` FFI structure |
| **Document Generation** | ReportLab | Sequential vector path commands (low-level drawing API) | Primitive path dictionary export |
| **Font Inspection** | FontTools | Font metadata (metrics, tables, axis settings) | Idiomatic dictionary/list metadata exposure |

