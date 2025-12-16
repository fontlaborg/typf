<!-- this_file: PLANSTEPS/04-api-extension-typfpy.md -->

## 3. API Extension Proposal: Python Bindings (`typfpy`)

The strategy focuses on high-throughput data exchange via memory views exposed through the Python FFI boundary.

1.  **FFI Glyph Data Bridge (Stage 4)**: To interface with C libraries like Pycairo, a memory layout compatible with `cairo_glyph_t` (which requires a glyph index and double-precision x/y offsets) must be guaranteed.
    *   The `typfpy` bindings **MUST** include a Python method, `get_cairo_glyphs_view(text)`, which converts the Rust `typf::PositionedGlyph` into an array of `#[repr(C)]` Rust structures (`CairoGlyph`).
    *   This method returns a NumPy `ndarray` view of this Rust-owned memory via `PyArray::borrow_from_array`, enabling near-zero-copy transfer to Python consumers like Pycairo.

2.  **Vector Path Export (Stage 5)**: To support vector-based document tools like ReportLab, a method is needed to serialize geometry.
    *   Add `export_vector_paths_as_primitives(text)` to the `typfpy.Typf` class. This method exports the vector outlines generated in Stage 5 as an idiomatic Python list of dictionary primitives (e.g., `{'type': 'lineTo', 'x': 10.0, 'y': 20.0}`) for consumption by `reportlab.pdfgen.canvas`.

3.  **Metadata Access (Stage 3)**: To support font auditing via tools like `fontTools`, internal metadata must be exposed idiomatically.
    *   Add `get_font_metrics(font_name)` to return complex font metadata (metrics, variations) as standard Python dictionaries or lists.

