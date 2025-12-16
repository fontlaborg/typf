<!-- this_file: PLANSTEPS/03-api-extension-typf-core.md -->

## 2. API Extension Proposal: Rust Core (`typf-core`)

The key amendments ensure `typf` operates as a source of high-quality typographic data compatible with native Rust and FFI requirements.

### A. Standardizing Shaped Output (Stage 4)

To ensure interoperability, the output of Stage 4 (Shaping) must adhere to a standardized data model reflecting the HarfBuzz specification.

1.  **Define the Standardized Intermediate Data Structure (SIDS)**: The structure `typf::PositionedGlyph` is the canonical output. This structure **MUST** contain the glyph index (`glyph_id: u32`), the final calculated position offset (`position: Vector2D<f32>`), the horizontal advance (`advance: f32`), and the cluster index (`cluster_id: u32`).
2.  **Mandate Seamless Conversion**: The `typf-core` crate **MUST** implement the `From` and `Into` traits for `typf::PositionedGlyph` to allow seamless conversion to target library primitives, such as `ab_glyph::Glyph` (which needs scale and position components) and `cosmic_text::ShapeGlyph` (which uses cluster IDs).

### B. Extending Rendering for Geometry (Stage 5)

The base `typf::Renderer` trait is not sufficient for modern GPU pipelines that require meshes or raw vector paths, necessitating extension.

1.  **Introduce `GpuRenderer` Trait**: A specialized sibling trait, `GpuRenderer`, is required to handle vertex and geometry data generation.
2.  **Define Mesh Output**: The `GpuRenderer` **MUST** define methods for generating a `RenderMesh` composed of vertices intended for GPU upload. The core performance requirement is that the vertex structure (`RenderMesh::Vertex`) is **zero-copy compliant**. This is achieved by adhering to the C ABI layout (`#[repr(C)]`) and implementing `zerocopy` marker traits, specifically `FromBytes` and `KnownLayout`. This guarantees direct and maximally efficient upload to GPU buffers via libraries like `wgpu`.
3.  **Define Vector Path Output**: The trait **SHOULD** also include methods capable of yielding raw vector data (iterable path operations) suitable for external tessellation libraries.

### C. Refining Font Access (Stage 3)

The `typf::FontRef` trait **MUST** be amended to allow external libraries that perform low-level font operations to access the original data without copying it.

*   The trait **MUST** include `fn as_bytes(&self) -> &[u8]` to provide a slice reference to the raw font binary data. This is essential for zero-copy integration with crates like `swash::FontRef`.

