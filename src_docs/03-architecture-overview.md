---
title: Architecture Overview
icon: lucide/box
tags:
  - Architecture
  - Design
  - Pipeline
---

# Architecture Overview

Understanding TYPF v2.0's architecture is key to leveraging its full power. This chapter explores the system design, component relationships, and architectural principles that make TYPF unique.

## High-Level Architecture

```mermaid
graph TB
    A[Input Text] --> B[Input Parser]
    B --> C[Unicode Processor]
    C --> D[Font Selector]
    D --> E[Shaper]
    E --> F[Renderer]
    F --> G[Exporter]
    
    H[Font Database] --> D
    I[Configuration] --> B
    I --> E
    I --> F
    I --> G
    
    J[Cache System] --> D
    J --> E
    J --> F
    
    subgraph "Backends"
        K[Shaping Backends]
        L[Rendering Backends]
    end
    
    E --> K
    F --> L
```

## The Six-Stage Pipeline

TYPF processes text through six distinct stages, each with clear responsibilities and interfaces:

### Stage 1: Input Parsing
**Purpose**: Convert raw input into structured text data

**Responsibilities**:
- Parse text strings and metadata
- Handle encodings and normalization
- Extract rendering parameters
- Validate input format

**Key Components**:
- [`InputParser`](17-rust-api.md#input-parser)
- [`TextBuffer`](17-rust-api.md#text-buffer)
- [`ParseOptions`](17-rust-api.md#parse-options)

### Stage 2: Unicode Processing
**Purpose**: Prepare text for complex script shaping

**Responsibilities**:
- Script detection and classification
- Bidirectional text analysis
- Text segmentation and boundaries
- Unicode normalization

**Key Components**:
- [`UnicodeProcessor`](17-rust-api.md#unicode-processor)
- [`ScriptDetector`](17-rust-api.md#script-detector)
- [`BidiAnalyzer`](17-rust-api.md#bidi-analyzer)

### Stage 3: Font Selection
**Purpose**: Choose optimal fonts for the text content

**Responsibilities**:
- Font matching and fallback
- Script-specific font selection
- Style and weight matching
- Font loading and caching

**Key Components**:
- [`FontSelector`](17-rust-api.md#font-selector)
- [`FontDatabase`](17-rust-api.md#font-database)
- [`FontLoader`](17-rust-api.md#font-loader)

### Stage 4: Shaping
**Purpose**: Convert characters to positioned glyphs

**Responsibilities**:
- Glyph substitution and positioning
- Complex script shaping (Arabic, Indic, etc.)
- Kerning and ligatures
- Metrics calculation

**Key Components**:
- [`Shaper`](17-rust-api.md#shaper-trait)
- [Shaping Backends](09-harfbuzz-shaping.md)
- [`GlyphBuffer`](17-rust-api.md#glyph-buffer)

### Stage 5: Rendering
**Purpose**: Convert glyphs to visual output

**Responsibilities**:
- Rasterization and vectorization
- Color and effect application
- Subpixel positioning
- Scaling and transformation

**Key Components**:
- [`Renderer`](17-rust-api.md#renderer-trait)
- [Rendering Backends](13-skia-rendering.md)
- [`RenderContext`](17-rust-api.md#render-context)

### Stage 6: Export
**Purpose**: Output rendered data in various formats

**Responsibilities**:
- Format conversion and encoding
- Metadata embedding
- Compression and optimization
- File writing and streaming

**Key Components**:
- [`Exporter`](17-rust-api.md#exporter-trait)
- [Export Formats](22-export-formats.md)
- [`ExportOptions`](17-rust-api.md#export-options)

## Backend Architecture

### Shaping Backends

```mermaid
graph LR
    A[Shaper Trait] --> B[HarfBuzz]
    A --> C[CoreText]
    A --> D[DirectWrite]
    A --> E[ICU-HB]
    A --> F[None]
    
    B --> G[Cross-platform]
    C --> H[macOS Optimized]
    D --> I[Windows Optimized]
    E --> J[Advanced Scripts]
    F --> K[Testing/Debugging]
```

Each shaping backend implements the [`Shaper`](17-rust-api.md#shaper-trait) trait:

```rust
pub trait Shaper: Send + Sync {
    fn shape(&self, text: &str, font: &Font, options: &ShapeOptions) -> Result<ShapingResult>;
    fn supports_script(&self, script: Script) -> bool;
    fn get_features(&self) -> ShaperFeatures;
}
```

### Rendering Backends

```mermaid
graph LR
    A[Renderer Trait] --> B[Skia]
    A --> C[CoreGraphics]
    A --> D[Direct2D]
    A --> E[Orge]
    A --> F[Zeno]
    A --> G[JSON]
    
    B --> H[Vector Graphics]
    C --> I[macOS Native]
    D --> J[Windows Native]
    E --> K[Pure Rust]
    F --> L[GPU Accelerated]
    G --> M[Data Export]
```

Each rendering backend implements the [`Renderer`](17-rust-api.md#renderer-trait) trait:

```rust
pub trait Renderer: Send + Sync {
    fn render(&self, glyphs: &[Glyph], context: &RenderContext) -> Result<RenderOutput>;
    fn supports_format(&self, format: PixelFormat) -> bool;
    fn get_features(&self) -> RendererFeatures;
}
```

## Component Relationships

### Core Components

```mermaid
graph TB
    A[Pipeline] --> B[PipelineContext]
    B --> C[Metrics]
    B --> D[Cache]
    B --> E[Configuration]
    
    F[BackendRegistry] --> G[ShaperRegistry]
    F --> H[RendererRegistry]
    F --> I[ExporterRegistry]
    
    A --> F
    
    J[FontDatabase] --> K[FontLoader]
    J --> L[FontCache]
    
    B --> J
```

### Data Flow

```mermaid
sequenceDiagram
    participant Client
    participant Pipeline
    participant Selector
    participant Shaper
    participant Renderer
    participant Exporter
    
    Client->>Pipeline: render(text, font, options)
    Pipeline->>Selector: select_font(text)
    Selector-->>Pipeline: FontHandle
    
    Pipeline->>Shaper: shape(text, font, options)
    Shaper-->>Pipeline: GlyphBuffer
    
    Pipeline->>Renderer: render(glyphs, context)
    Renderer-->>Pipeline: RenderOutput
    
    Pipeline->>Exporter: export(output, format)
    Exporter-->>Pipeline: ExportResult
    Pipeline-->>Client: Result
```

## Memory Management

### Font Handling

```mermaid
graph LR
    A[Font File] --> B[Memory Mapping]
    B --> C[Box::leak]
    C --> D[Arc<Font>]
    D --> E[LRU Cache]
    E --> F[Zero-Copy Access]
```

**Key Principles**:
- **Zero-Copy Loading**: Fonts are memory-mapped, not copied
- **Intentional Leaking**: `Box::leak()` for static font data
- **Shared Ownership**: `Arc<Font>` for safe sharing
- **LRU Eviction**: Automatic cache management

### Glyph Caching

```mermaid
graph TB
    A[Font + Size] --> B[FontCache]
    B --> C[GlyphKey]
    C --> D[GlyphCache]
    D --> E[RenderedGlyph]
    E --> F[OutputCache]
    
    G[SIMD Operations] --> E
    H[Subpixel Positioning] --> E
```

**Cache Hierarchy**:
1. **Font Cache**: Loaded font data (LRU eviction)
2. **Glyph Cache**: Rendered glyph images
3. **Output Cache**: Complete rendered frames

## Performance Architecture

### SIMD Acceleration

```rust
// Example: SIMD-optimized alpha compositing
fn composite_alpha_simd(dst: &mut [u8], src: &[u8], alpha: u8) {
    use std::arch::x86_64::*;
    
    unsafe {
        let alpha_vec = _mm_set1_epi8(alpha as i8);
        
        for (dst_chunk, src_chunk) in dst.chunks_exact_mut(16).zip(src.chunks_exact(16)) {
            let dst_vec = _mm_loadu_si128(dst_chunk.as_ptr() as *const __m128i);
            let src_vec = _mm_loadu_si128(src_chunk.as_ptr() as *const __m128i);
            
            let result = _mm_blendv_epi8(dst_vec, src_vec, alpha_vec);
            _mm_storeu_si128(dst_chunk.as_mut_ptr() as *mut __m128i, result);
        }
    }
}
```

### Concurrency Strategy

```mermaid
graph TB
    A[Input Text] --> B[Text Segments]
    B --> C[Thread Pool]
    C --> D[Worker 1]
    C --> E[Worker 2]
    C --> F[Worker N]
    
    D --> G[Shape Results]
    E --> G
    F --> G
    
    G --> H[Result Merger]
    H --> I[Final Output]
```

**Concurrency Patterns**:
- **Pipeline Parallelism**: Different stages can run concurrently
- **Data Parallelism**: Text segments processed in parallel
- **Cache Coherency**: Shared caches with proper synchronization
- **Lock-Free Structures**: `DashMap` for concurrent access

## Configuration Architecture

### Feature Flags

```toml
# Cargo.toml
[features]
default = ["shaping-hb", "render-skia", "export-png"]
minimal = ["shaping-none", "render-orge", "export-pnm"]
full = [
    "shaping-hb", "shaping-coretext", "shaping-directwrite",
    "render-skia", "render-coregraphics", "render-direct2d",
    "export-png", "export-svg", "export-json"
]

shaping-hb = ["harfbuzz_rs"]
shaping-coretext = ["coretext-rs"]
render-skia = ["skia-safe"]
export-png = ["image"]
```

### Runtime Configuration

```mermaid
graph TB
    A[Config File] --> B[TOML Parser]
    C[Environment] --> B
    D[CLI Args] --> B
    
    B --> E[Configuration]
    E --> F[Pipeline Settings]
    E --> G[Backend Selection]
    E --> H[Cache Parameters]
    
    F --> I[PipelineBuilder]
    G --> I
    H --> I
    
    I --> J[Pipeline Instance]
```

## Error Handling Architecture

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum TypfError {
    #[error("Font not found: {path}")]
    FontNotFound { path: PathBuf },
    
    #[error("Shaping failed: {backend}")]
    ShapingFailed { backend: String },
    
    #[error("Rendering failed: {backend}")]
    RenderingFailed { backend: String },
    
    #[error("Feature not compiled: {feature}")]
    FeatureNotCompiled { feature: String },
    
    #[error("Memory allocation failed")]
    OutOfMemory,
}
```

### Error Propagation

```mermaid
graph TB
    A[Low-Level Error] --> B[Wrap Context]
    B --> C[TypfError]
    C --> D[Pipeline Result]
    
    E[Python Exception] --> F[PyErr]
    F --> G[TypfError]
    G --> H[Python Result]
```

## Testing Architecture

### Test Pyramid

```mermaid
graph TB
    A[Integration Tests] --> B[Component Tests]
    B --> C[Unit Tests]
    
    D[Property-Based Tests] --> B
    E[Fuzz Tests] --> C
    F[Benchmark Tests] --> A
```

### Test Categories

1. **Unit Tests**: Individual component functionality
2. **Integration Tests**: Pipeline and backend interaction
3. **Property Tests**: Invariant preservation
4. **Fuzz Tests**: Robustness and crash resistance
5. **Benchmark Tests**: Performance regression detection

## Extensibility Architecture

### Backend Registration

```rust
pub struct BackendRegistry {
    shapers: HashMap<String, Box<dyn ShaperFactory>>,
    renderers: HashMap<String, Box<dyn RendererFactory>>,
    exporters: HashMap<String, Box<dyn ExporterFactory>>,
}

impl BackendRegistry {
    pub fn register_shaper(&mut self, name: &str, factory: Box<dyn ShaperFactory>) {
        self.shapers.insert(name.to_string(), factory);
    }
}
```

### Plugin Architecture (Future)

```mermaid
graph TB
    A[Core Library] --> B[Plugin Interface]
    B --> C[Plugin A]
    B --> D[Plugin B]
    B --> E[Plugin N]
    
    F[Dynamic Loading] --> B
    G[Safety Checks] --> B
```

## Implementation Phases

### Phase 1: Core Foundation
- Basic pipeline structure
- Essential traits and types
- Minimal working implementation

### Phase 2: Backend Implementation
- HarfBuzz shaping backend
- Skia rendering backend
- Basic export formats

### Phase 3: Platform Integration
- CoreText and DirectWrite
- Platform-specific optimizations
- System font integration

### Phase 4: Advanced Features
- SIMD optimizations
- Advanced caching
- Variable font support

### Phase 5: Python Bindings
- PyO3 integration
- Python API design
- Package distribution

### Phase 6: Production Ready
- Comprehensive testing
- Documentation completion
- Performance optimization

## Next Steps

Now that you understand the architecture, explore:

- [The Six-Stage Pipeline](05-six-stage-pipeline.md) - Deep dive into each stage
- [Backend Architecture](06-backend-architecture.md) - Backend implementation details
- [Memory Management](07-memory-management.md) - Efficient memory usage
- [Performance Fundamentals](08-performance-fundamentals.md) - Optimization strategies

---

**TYPF's architecture** is designed for performance, modularity, and extensibility. Each component has clear responsibilities and well-defined interfaces, making the system both powerful and maintainable.
