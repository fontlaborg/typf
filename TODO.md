# TODO: Advanced Rendering Backends

## Phase S.1: SDF Core & CPU Renderer (Priority: Medium)

### S.1.1 typf-sdf-core Crate Setup
- [ ] Create `crates/typf-sdf-core/` directory
- [ ] Create `Cargo.toml` with skrifa dependency
- [ ] Add crate to workspace members

### S.1.2 Core Types
- [ ] Implement `SdfGlyph` struct
- [ ] Implement `SizeBand` enum
- [ ] Implement `SdfGlyphKey` for caching
- [ ] Implement `SdfAtlas` struct
- [ ] Implement `SdfGlyphEntry` for atlas placement

### S.1.3 SDF Generation
- [ ] Implement outline extraction from skrifa
- [ ] Implement signed distance calculation for line segments
- [ ] Implement signed distance calculation for quadratic Beziers
- [ ] Implement signed distance calculation for cubic Beziers
- [ ] Implement winding number calculation for sign
- [ ] Implement distance normalization to [-1, +1]
- [ ] Add unit tests for known shapes (circle, square)

### S.1.4 Atlas Packing
- [ ] Implement `SkylineNode` struct
- [ ] Implement `AtlasPacker` with skyline algorithm
- [ ] Add unit tests for packing

### S.1.5 typf-render-sdf Backend
- [ ] Create `backends/typf-render-sdf/` directory
- [ ] Implement `SdfRenderer` struct
- [ ] Implement `Renderer` trait
- [ ] Implement bilinear SDF sampling
- [ ] Implement smoothstep distance→coverage conversion

### S.1.6 Testing & Validation
- [ ] Unit tests for SDF sampler
- [ ] Integration test with NotoSans
- [ ] Quality comparison vs Opixa at various sizes

---

## Phase S.2: GPU SDF Renderer (Future)

- [ ] Create `backends/typf-render-sdf-gpu/` with wgpu
- [ ] Implement WGSL shaders for SDF sampling
- [ ] Add MSDF support for sharp corners

---

## Deferred / Future Tasks

### Performance & Quality
- [ ] Quality comparison: Vello vs Opixa (PSNR metrics)
- [ ] Performance benchmark: Vello GPU vs Opixa at large sizes
- [ ] Add performance benchmarks to docs

### Platform Support
- [ ] Test Vello GPU on Linux (Vulkan)
- [ ] Test Vello GPU on Windows (DX12/Vulkan)
- [ ] Add WASM/WebGPU support for Vello

### Integration
- [ ] Add `sdf` to CLI `--renderer` options
- [ ] Add `sdf` to Python bindings
- [ ] Update Python type hints

---

## Completed

### Phase V.1: Vello CPU Backend ✓
- [x] Create `typf-render-vello-cpu` crate
- [x] Implement `Renderer` trait using `vello_cpu::RenderContext`
- [x] Add CLI integration (`--renderer vello-cpu`)
- [x] Add Python bindings (`Typf(renderer="vello-cpu")`)
- [x] 16 tests passing (4 unit + 12 integration including color fonts)

### Phase V.2: Vello GPU Backend ✓
- [x] Create `typf-render-vello` crate with wgpu
- [x] Implement `Renderer` trait with GPU context
- [x] Add GPU→CPU readback for bitmap export
- [x] Add CLI integration (`--renderer vello`)
- [x] Add Python bindings (`Typf(renderer="vello")`)
- [x] 15 tests passing (3 unit + 12 integration including color fonts)

### Documentation ✓
- [x] Update README with Vello renderers
- [x] Add renderer comparison table
- [x] Document when to use each renderer
- [x] Update src_docs/06-backend-architecture.md

### Previous Work ✓
- [x] CBDT Bitmap Font Support
- [x] Security & Reliability (fuzzing, input validation)
- [x] Color font support (COLR, SVG, sbix, CBDT)
- [x] Real font integration tests
- [x] Benchmark infrastructure

---

## Notes

### Testing
- Vello CPU: 16 tests (4 unit + 12 integration)
- Vello GPU: 15 tests (3 unit + 12 integration)
- Total workspace: 378+ tests

### Dependencies
- Vello requires external/vello submodule
- wgpu backends: metal (macOS), vulkan (Linux), dx12 (Windows)
