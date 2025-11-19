# TYPF Architecture

## Overview

TYPF v2.0 is a modular text rendering pipeline built on a six-stage architecture. Each stage is independent and can be replaced with different backend implementations.

## Six-Stage Pipeline

```
Text Input → Unicode Processing → Font Selection → Shaping → Rendering → Export
```

### Stage 1: Input Parsing
- **Purpose**: Parse text with metadata (language, direction, features)
- **Module**: `typf-input`
- **Output**: Structured text with attributes

### Stage 2: Unicode Processing
- **Purpose**: Script detection, bidirectional analysis, segmentation
- **Module**: `typf-unicode`
- **Dependencies**: ICU for complex Unicode operations
- **Output**: Text runs with script and direction information

### Stage 3: Font Selection
- **Purpose**: Font matching, fallback chain, feature resolution
- **Module**: `typf-fontdb`
- **Dependencies**: `fontdb`, `read-fonts`, `skrifa`
- **Output**: Font references with features

### Stage 4: Shaping
- **Purpose**: Convert text to positioned glyphs
- **Backends**:
  - `typf-shape-none`: Simple left-to-right advancement
  - `typf-shape-hb`: HarfBuzz shaping (planned)
  - `typf-shape-icu-hb`: ICU + HarfBuzz (planned)
  - Platform-specific: CoreText (macOS), DirectWrite (Windows)
- **Output**: `ShapingResult` with positioned glyphs

### Stage 5: Rendering
- **Purpose**: Rasterize glyphs to bitmaps or vectors
- **Backends**:
  - `typf-render-orge`: Basic CPU rasterization
  - `typf-render-skia`: Skia-based rendering (planned)
  - `typf-render-zeno`: Alternative rasterizer (planned)
  - Platform-specific: CoreGraphics (macOS), Direct2D (Windows)
- **Output**: `RenderOutput` (bitmap or vector data)

### Stage 6: Export
- **Purpose**: Convert rendered output to file formats
- **Module**: `typf-export`
- **Formats**:
  - PNM (PPM, PGM, PBM) - implemented
  - PNG, SVG, PDF - planned
  - JSON for shaping data
- **Output**: Serialized file data

## Core Components

### Trait Hierarchy

```rust
// Base trait for all pipeline stages
pub trait Stage {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
}

// Shaping backend trait
pub trait Shaper: Stage {
    fn shape(&self, text: &str, font: Arc<dyn FontRef>, params: &ShapingParams)
        -> Result<ShapingResult>;
}

// Rendering backend trait
pub trait Renderer: Stage {
    fn render(&self, shaped: &ShapingResult, font: Arc<dyn FontRef>, params: &RenderParams)
        -> Result<RenderOutput>;
}

// Export format trait
pub trait Exporter: Stage {
    fn export(&self, output: &RenderOutput) -> Result<Vec<u8>>;
}
```

### Pipeline Executor

The `Pipeline` struct orchestrates the six stages:

```rust
pub struct Pipeline {
    shaper: Arc<dyn Shaper>,
    renderer: Arc<dyn Renderer>,
    exporter: Arc<dyn Exporter>,
}
```

### Error Handling

Each stage has its own error type that can be converted to the unified `TypfError`:

- `InputError`: Input parsing failures
- `UnicodeError`: Unicode processing errors
- `FontError`: Font loading/selection errors
- `ShapingError`: Shaping backend errors
- `RenderError`: Rendering failures
- `ExportError`: Export format errors

## Feature Flags

### Build Configurations

- **minimal**: Smallest possible build (~500KB target)
  - NoneShaper + OrgeRenderer + PNM export
  - No external dependencies

- **default**: Common use cases
  - Adds Unicode processing and font database
  - ~1.5MB binary

- **full**: All features enabled
  - All backends and export formats
  - SIMD optimizations
  - Parallel processing

### Conditional Compilation

Features control which backends are compiled:

```toml
[features]
# Shaping backends
shaping-none = ["dep:typf-shape-none"]
shaping-hb = ["dep:typf-shape-hb"]

# Rendering backends
render-orge = ["dep:typf-render-orge"]
render-skia = ["dep:typf-render-skia"]
```

## Performance Architecture

### Caching Strategy

Three-level cache hierarchy:
1. **L1 Cache**: Hot path, <50ns access
   - Glyph metrics
   - Shaping results for recent strings
2. **L2 Cache**: Warm path, <500ns access
   - Rendered glyphs
   - Font data structures
3. **L3 Cache**: Cold path, persistent
   - Font files
   - Complex shaping results

### SIMD Optimizations

Platform-specific SIMD for critical paths:
- **x86_64**: AVX2, SSE4.1
- **ARM**: NEON
- **WASM**: SIMD128

Target: >10GB/s for RGBA blending operations

### Parallelization

- Work-stealing queue for glyph rendering
- Parallel shaping for independent text runs
- Async font loading with memory mapping

## Memory Management

### Zero-Copy Design

- Memory-mapped font files
- Borrowed string slices throughout pipeline
- Arc for shared immutable data

### Buffer Pooling

- Reusable buffers for shaping
- Glyph bitmap pools
- Export buffer recycling

## Platform Integration

### Auto-Backend Selection

When `auto-backend` feature is enabled:

```rust
#[cfg(target_os = "macos")]
type DefaultShaper = CoreTextShaper;

#[cfg(target_os = "windows")]
type DefaultShaper = DirectWriteShaper;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
type DefaultShaper = HarfBuzzShaper;
```

### Fused Paths

Platform backends can provide fused shaping+rendering:
- CoreText + CoreGraphics on macOS
- DirectWrite + Direct2D on Windows

## Testing Strategy

### Unit Tests
- Each module has isolated unit tests
- Mock implementations for trait testing
- Property-based testing for Unicode handling

### Integration Tests
- Full pipeline tests with real fonts
- Golden file comparisons for rendering
- Fuzzing for security-critical paths

### Performance Tests
- Benchmarks for each stage
- Memory profiling
- Cache hit rate monitoring

## Security Considerations

### Input Validation
- Bounds checking for all font data access
- UTF-8 validation for input text
- Safe integer arithmetic

### Memory Safety
- No unsafe code in core modules
- FFI boundaries carefully managed
- Panic handlers at API boundaries

## Future Extensibility

### Plugin System (Future)
- Dynamic backend loading
- Custom stages via traits
- Runtime feature detection

### WASM Support (Future)
- Browser-compatible builds
- WebGL rendering backend
- Streaming font loading

---

## Appendix: File Structure

```
typf/
├── crates/
│   ├── typf/              # Main library
│   ├── typf-core/         # Core traits and types
│   ├── typf-input/        # Input parsing
│   ├── typf-unicode/      # Unicode processing
│   ├── typf-fontdb/       # Font management
│   ├── typf-export/       # Export formats
│   └── typf-cli/          # CLI application
├── backends/
│   ├── typf-shape-*/      # Shaping backends
│   └── typf-render-*/     # Rendering backends
└── examples/              # Usage examples
```