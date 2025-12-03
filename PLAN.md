# Typf Advanced Rendering Backends Plan

**Version:** 2.5.0
**Status:** Implementation Phase

## Executive Summary

This plan introduces two breakthrough rendering improvements for typf:

1. **Vello Integration** - GPU compute-centric 2D rendering via the Vello engine (already in `external/vello`)
2. **SDF/MSDF Backend** - Signed Distance Field rendering for size-independent text

Both approaches complement the existing Opixa rasterizer rather than replacing it.

---

## Part 1: Vello Integration (Priority: High)

Vello is a modern GPU compute-centric 2D renderer written in Rust. It's already available in `external/vello` and provides:

- **GPU rendering** via wgpu (cross-platform: Vulkan, Metal, DX12, WebGPU)
- **CPU fallback** via `vello_cpu` (pure Rust, no GPU required)
- **Glyph caching** with outline caches and hinting support
- **Full color font support** (outline, bitmap, COLR)
- **Uses skrifa** (same font library as typf)

### Why Vello?

| Feature | Opixa | Vello GPU | Vello CPU |
|---------|-------|-----------|-----------|
| Platform | All | GPU required | All |
| Performance | Good | Excellent (10-100x) | Very Good |
| Color fonts | Via typf-render-color | Native | Native |
| Complex effects | Limited | Full (gradients, blend modes) | Full |
| Memory efficiency | Per-size bitmaps | Glyph caching | Glyph caching |

### Architecture

```
backends/
├── typf-render-vello/          # GPU renderer (wgpu)
│   ├── src/
│   │   ├── lib.rs              # VelloRenderer implementation
│   │   ├── scene.rs            # Scene construction from ShapingResult
│   │   └── context.rs          # wgpu context management
│   └── Cargo.toml
│
└── typf-render-vello-cpu/      # CPU renderer (no GPU)
    ├── src/
    │   ├── lib.rs              # VelloCpuRenderer implementation
    │   └── context.rs          # RenderContext wrapper
    └── Cargo.toml
```

### Implementation Phases

#### Phase V.1: Vello CPU Backend (Simpler, No GPU) ✓ COMPLETE

**Goal:** Integrate `vello_cpu` as a high-quality CPU renderer.

**Implementation:** `backends/typf-render-vello-cpu/src/lib.rs`

```rust
pub struct VelloCpuRenderer {
    config: VelloCpuConfig,
}

impl Renderer for VelloCpuRenderer {
    fn name(&self) -> &'static str { "vello-cpu" }
    // Uses RenderContext.glyph_run() for text rendering
    // Supports font hinting, foreground/background colors
}
```

**Completed Tasks:**
- [x] Create `typf-render-vello-cpu` crate
- [x] Implement `Renderer` trait using `vello_cpu::RenderContext`
- [x] Convert `ShapingResult` glyphs to Vello `Glyph` format
- [x] Handle font data conversion (typf FontRef → peniko FontData)
- [x] Add glyph caching integration (via RenderContext)
- [x] Add tests with real fonts (13 tests passing)
- [x] Add CLI integration (`--renderer vello-cpu`)
- [x] Add Python bindings (`Typf(renderer="vello-cpu")`)

#### Phase V.2: Vello GPU Backend ✓ COMPLETE

**Goal:** Integrate full Vello GPU renderer for maximum performance.

```rust
// backends/typf-render-vello/src/lib.rs
pub struct VelloRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: vello::Renderer,
    config: VelloConfig,
}

impl Renderer for VelloRenderer {
    fn name(&self) -> &'static str { "vello" }

    fn render(
        &self,
        shaped: &ShapingResult,
        font: Arc<dyn FontRef>,
        params: &RenderParams,
    ) -> Result<RenderOutput> {
        // 1. Create vello::Scene
        // 2. Build glyph run and fill/stroke
        // 3. Render to texture via render_to_texture
        // 4. Read back to CPU bitmap (for export)
        // 5. Return as RenderOutput::Bitmap
    }
}
```

**Tasks:**
- [x] Create `typf-render-vello` crate
- [x] Add wgpu context initialization (via GpuContext)
- [x] Implement scene construction from ShapingResult
- [x] Add GPU→CPU readback for bitmap export (256-byte row alignment)
- [x] Add CLI and Python bindings integration
- [ ] Add async rendering option (future)
- [ ] Add WASM/WebGPU support (future)

---

## Part 2: SDF/MSDF Backend (Priority: Medium)

SDF rendering provides resolution-independent text with near-constant rendering cost across all sizes.

### Why SDF?

| Aspect | Traditional Scanline | SDF/MSDF |
|--------|---------------------|----------|
| Scaling cost | O(size²) | O(1) |
| GPU integration | Complex | Trivial (texture sampling) |
| Effects (outline, glow) | Expensive | Single shader uniform |
| Memory per glyph | O(resolution²) | O(fixed SDF size²) |

### Architecture

```
crates/
└── typf-sdf-core/              # SDF math, types, atlas management
    ├── src/
    │   ├── lib.rs              # Module exports
    │   ├── types.rs            # SdfGlyph, SdfAtlas, SizeBand
    │   ├── generator.rs        # SDF generation from outlines
    │   ├── atlas.rs            # Skyline packing
    │   └── cache.rs            # SdfAtlasManager
    └── Cargo.toml

backends/
└── typf-render-sdf/            # CPU SDF renderer
    ├── src/
    │   ├── lib.rs              # SdfRenderer implementation
    │   ├── sampler.rs          # Bilinear sampling, smoothstep
    │   └── compositor.rs       # Glyph composition
    └── Cargo.toml
```

### Core Types

```rust
/// Single-channel signed distance field for a glyph
pub struct SdfGlyph {
    pub glyph_id: u32,
    pub width: u16,
    pub height: u16,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub advance: f32,
    pub data: Vec<f32>,         // Signed distances [-range, +range]
    pub range: f32,             // Max distance (in SDF pixels)
    pub scale: f32,             // SDF pixels per font unit
}

/// Size bands for atlas organization
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum SizeBand {
    Small,    // 8-24px output, 32x32 SDF
    Medium,   // 24-64px output, 48x48 SDF
    Large,    // 64-200px output, 64x64 SDF
}

/// Atlas containing multiple SDF glyphs
pub struct SdfAtlas {
    pub texture_width: u32,
    pub texture_height: u32,
    pub glyphs: HashMap<SdfGlyphKey, SdfGlyphEntry>,
    pub data: Vec<f32>,
}
```

### Implementation Phases

#### Phase S.1: CPU SDF Prototype

**Tasks:**
- [ ] Create `typf-sdf-core` crate with types
- [ ] Implement SDF generation from glyph outlines
- [ ] Implement skyline atlas packing
- [ ] Create `typf-render-sdf` with CPU renderer
- [ ] Add quality validation vs Opixa

#### Phase S.2: GPU SDF (Future)

**Tasks:**
- [ ] Create `typf-render-sdf-gpu` with wgpu
- [ ] Implement WGSL fragment shader for SDF sampling
- [ ] Add MSDF support for sharp corners

---

## Workspace Integration

### Feature Flags

```toml
# Cargo.toml (workspace)
[workspace.metadata.features]
render-vello = []           # GPU renderer
render-vello-cpu = []       # CPU renderer (no GPU)
render-sdf = []             # SDF CPU renderer
render-sdf-gpu = []         # SDF GPU renderer (future)

[workspace.dependencies]
typf-render-vello = { path = "backends/typf-render-vello", version = "2.0.0" }
typf-render-vello-cpu = { path = "backends/typf-render-vello-cpu", version = "2.0.0" }
typf-sdf-core = { path = "crates/typf-sdf-core", version = "2.0.0" }
typf-render-sdf = { path = "backends/typf-render-sdf", version = "2.0.0" }

# External Vello
vello = { path = "external/vello/vello" }
vello_cpu = { path = "external/vello/sparse_strips/vello_cpu" }
vello_common = { path = "external/vello/sparse_strips/vello_common" }
```

### CLI Extension

```bash
# New renderer options
typf render --renderer vello "Hello World" --output hello.png
typf render --renderer vello-cpu "Hello World" --output hello.png
typf render --renderer sdf "Hello World" --output hello.png

# List available renderers
typf info --renderers
```

### Python Bindings

```python
from typf import Typf

# GPU rendering
t = Typf(renderer="vello")
result = t.render_text("Hello", font_path, size=48)

# CPU rendering (Vello)
t = Typf(renderer="vello-cpu")
result = t.render_text("Hello", font_path, size=48)

# SDF rendering
t = Typf(renderer="sdf")
result = t.render_text("Hello", font_path, size=48)
```

---

## Testing Strategy

### Unit Tests
- Glyph conversion: typf → Vello format
- SDF distance calculation accuracy
- Atlas packing correctness

### Integration Tests
- Full pipeline with real fonts
- Quality comparison vs Opixa/CoreGraphics
- Memory limits and cache behavior

### Benchmarks
- Scaling behavior across sizes
- Comparison: Opixa vs Vello CPU vs Vello GPU vs SDF
- Memory usage patterns

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Vello CPU quality | Comparable to Opixa (PSNR > 35 dB) |
| Vello GPU performance | > 10x faster than Opixa at 128px |
| SDF scaling | < 2x slowdown from 16px to 128px |
| Integration | Full CLI/Python/Rust API support |

---

## Non-Goals

- **Replacing Opixa**: These backends are complementary
- **Complex text effects**: Focus on rendering, not animation
- **Async-first API**: Sync API sufficient for current use cases

---

## References

1. [Vello - GPU 2D renderer](https://github.com/linebender/vello)
2. [Vello CPU thesis](https://ethz.ch/content/dam/ethz/special-interest/infk/inst-pls/plf-dam/documents/StudentProjects/MasterTheses/2025-Laurenz-Thesis.pdf)
3. [Valve's SDF Paper (2007)](https://steamcdn-a.akamaihd.net/apps/valve/2007/SIGGRAPH2007_AlphaTestedMagnification.pdf)
4. [msdfgen](https://github.com/Chlumsky/msdfgen)

---

## Implementation Order

1. **Phase V.1**: `typf-render-vello-cpu` (highest value, no GPU dependency)
2. **Phase V.2**: `typf-render-vello` (GPU acceleration)
3. **Phase S.1**: `typf-sdf-core` + `typf-render-sdf` (alternative approach)
4. **Phase S.2**: GPU SDF (future, if needed)

All phases follow existing typf patterns: `thiserror` for errors, comprehensive tests, clippy clean.
