<!-- this_file: TODO.md -->
# TODO (derived from TASKS.md)

**Version:** 5.0.2
**Updated:** 2026-02-11
**Source:** `PLANSTEPS/` (split from `TASKS.md`)

## Remaining Tasks

No remaining tasks for v5.0.2. All planned work complete.

## Completed (2026-02-11 JSONL quality sprint)

- [x] Parse `text.features` in JSONL jobs and feed validated values into `ShapingParams.features`
- [x] Make JSONL `JobSpec` accept `version` plus legacy `_version` compatibility alias
- [x] Parallelize JSONL batch job execution with deterministic output ordering and regression tests

## Completed (2026-02-11 maintenance sprint)

- [x] Add repo-root `./test.sh` wrapper as the canonical verification entrypoint
- [x] Update `scripts/test.sh` to use `cargo fmt --check` instead of `cargo fmt --all --check`
- [x] Align `.github/workflows/ci.yml` lint formatting check with local command (`cargo fmt --check`)

## Completed (v5.0.2)

<details>
<summary>Click to expand completed tasks</summary>

### Project Structure
- [x] Keep `TASKS.md` as an index (TLDR + links) and keep `PLANSTEPS/` authoritative
- [x] Add/maintain a single flat backlog here; avoid nested lists
- [x] Add `DEPENDENCIES.md` (major dependencies + rationale)
- [x] Align Cargo workspace versioning with git tags/docs (v5.0.1)

### Baseline Standardization
- [x] Inventory baseline math for Opixa/Skia/Zeno/Vello(-cpu) and document deltas vs CoreGraphics
- [x] Decide on one baseline contract (font metrics vs per-glyph bounds) and write it down
- [x] Implement the chosen contract consistently across renderers (or document why not)
- [x] Add regression tests that compare baseline placement across at least 2 renderers

### Vello-GPU Color Fonts
- [x] Confirm current behavior (blank output) and ensure CLI/docs steer users to `vello-cpu`
- [x] Add a clear runtime warning/error when `vello` is selected with bitmap/COLR glyphs
- [x] Track upstream status (issue link + minimal reproduction)

### Glyph Source & Color Fonts
- [x] Confirm `GlyphSource` covers Glyf, Cff, Cff2, Colr0, Colr1, Svg, Sbix, Cbdt, Ebdt
- [x] Ensure `typf-render-color` tries sources in `GlyphSourcePreference` priority order
- [x] Bitmap availability checks don't depend on outline presence
- [x] Centralize and harden bitmap decoding in `typf-render-color/src/bitmap.rs`
- [x] SVG renderer: opt-in bitmap embedding for color glyphs
- [x] SVG renderer: placeholder fallback when bitmap embedding disabled
- [x] Skia/Zeno: delegate complex glyph composition to `typf-render-color`
- [x] Color fixtures: expand regression coverage for COLR/SVG/sbix/CBDT

### Stage 4 (Shaping) Contract
- [x] Define a stable shaped-glyph output contract for zero-copy consumers
- [x] Define a C-ABI-safe glyph struct (repr(C), alignment, no padding surprises)
- [x] Add a "decoupled glyph iterator" API for layout engines

### Stage 5 (Rendering) Contract
- [x] Define an optional geometry output path (mesh/path ops) for GPU pipelines
- [x] Define a minimal path-op iterator API (PathOp enum + GlyphPath + GeometryData)
- [x] Define RenderMesh/vertex ABI for zero-copy GPU upload (Vertex2D, VertexUV, VertexColor, GlyphMesh, RenderMesh)

### API Extensions
- [x] Font bytes access: `FontRef::data_shared()` for zero-copy downstream access
- [x] Font metadata access: `FontRef::metrics()` for ascent/descent/units_per_em

### Python FFI
- [x] Expose vector path primitives (PathOp + GlyphPath types)
- [x] Expose font metrics/variations metadata (FontInfo enhancements)
- [x] Expose zero-copy shaped-glyph view for Pycairo-style consumers (ShapedGlyphs class with for_cairo(), iteration, indexing)

### Rust Integration
- [x] Validate typf ↔ cosmic-text/parley integration patterns
- [x] Add example showing typf as rasterization backend (external_layout_integration.rs)

### SDF Decision
- [x] Decide whether SDF is in-scope → OUT OF SCOPE for v5.x
- [-] Implement outline→SDF generation (SKIPPED)

### Platform Support
- [x] Evaluate WASM/WebGPU constraints and document in src_docs/21-webassembly-integration.md

### WGPU Integration
- [x] Prototype zero-copy mesh upload path with types + example (wgpu_mesh_upload.rs)

### Platform Support
- [x] Define vello-gpu test matrix for Linux/Windows (src_docs/26-vello-gpu-platform-support.md)

### Verification
- [x] Run full workspace tests + clippy and record results in `WORK.md`

</details>

## SDF Scoping Decision (2025-12-16)

**Decision**: SDF is OUT OF SCOPE for typf v5.x

**Rationale**:
1. SDF is primarily a GPU optimization technique; typf focuses on shaping + rasterization
2. Existing Rust solutions (kaku, easy-signed-distance-field) already serve this niche
3. SDF generation is better done offline (msdfgen CLI) for production use
4. Vello GPU renderer already provides high-quality GPU text without SDF
5. Adding SDF would introduce significant complexity for a narrow use case

**Recommendation**: Use msdfgen offline + shader-side MSDF sampling, or kaku for wgpu
