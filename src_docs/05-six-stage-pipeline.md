---
title: The Six-Stage Pipeline
icon: lucide/git-merge
tags:
  - Pipeline
  - Architecture
  - Data Flow
---

# The Six-Stage Pipeline

Text flows through six stages to become pixels.

```
Input → Unicode → Font → Shaping → Rendering → Export
   ↓        ↓       ↓        ↓         ↓        ↓
TextBuffer  ProcessedText  FontHandle  GlyphBuffer  RenderOutput  ExportResult
```

## Stage 1: Input Parsing

Raw text becomes structured data.

```rust
pub struct TextBuffer {
    pub text: String,
    pub language: Option<Language>,
    pub script: Option<Script>,
    pub direction: Direction,
    pub metadata: HashMap<String, String>,
}
```

The parser validates UTF-8, extracts metadata, and normalizes the input for the next stage.

**What it does:**
- Validates text encoding
- Extracts language and script hints
- Normalizes Unicode if requested
- Creates the initial TextBuffer

## Stage 2: Unicode Processing

Text analysis for complex scripts.

```rust
pub struct ProcessedText {
    pub original: String,
    pub segments: Vec<TextSegment>,
    pub base_direction: Direction,
    pub unicode_version: UnicodeVersion,
}
```

Identifies writing systems, handles bidirectional text, and breaks text into logical segments.

**What it does:**
- Detects scripts (Latin, Arabic, Devanagari)
- Analyzes bidirectional text flow
- Segments text for proper rendering
- Applies Unicode normalization

## Stage 3: Font Selection

Matches text to fonts that can display it.

```rust
pub struct FontSelectionResult {
    pub selections: Vec<FontSelection>,
    pub fallbacks_used: bool,
    pub unsupported_chars: Vec<char>,
}
```

Finds fonts supporting required characters, handles fallbacks, and manages font matching.

**What it does:**
- Matches fonts to text segments
- Selects fallback fonts for missing glyphs
- Considers font style and weight
- Tracks unsupported characters

## Stage 4: Shaping

Characters become positioned glyphs.

```rust
pub struct ShapingResult {
    pub glyphs: Vec<Glyph>,
    pub advances: Vec<f32>,
    pub positions: Vec<Position>,
    pub clusters: Vec<usize>,
    pub direction: Direction,
    pub script: Script,
}
```

Applies complex script rules, ligatures, kerning, and calculates glyph positions.

**What it does:**
- Substitutes glyphs (ligatures, contextual forms)
- Positions glyphs with proper spacing
- Handles complex script shaping
- Calculates text metrics

## Stage 5: Rendering

Glyphs become visual output.

```rust
pub struct RenderOutput {
    pub data: RenderData,
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub dpi: f32,
    pub transform: Transform,
}
```

Rasterizes or vectorizes glyphs, applies colors and effects.

**What it does:**
- Converts glyphs to pixels or vectors
- Applies colors and effects
- Handles subpixel rendering
- Transforms and positions final output

## Stage 6: Export

Output becomes file formats.

```rust
pub struct ExportResult {
    pub data: Vec<u8>,
    pub format: ExportFormat,
    pub metadata: ExportMetadata,
    pub size: FileSize,
}
```

Encodes to PNG, SVG, PDF, or other formats with compression and metadata.

**What it does:**
- Encodes to target formats
- Applies compression
- Embeds metadata
- Validates output compliance

## Pipeline Orchestration

Connect all stages:

```rust
impl Pipeline {
    pub fn process(&self, input: &str, options: &ProcessOptions) -> Result<ExportResult> {
        // Stage 1: Parse input
        let text_buffer = self.parser.parse(input, &options.parse_options)?;
        
        // Stage 2: Process Unicode
        let processed_text = self.unicode_processor.process(&text_buffer)?;
        
        // Stage 3: Select fonts
        let font_selection = self.font_selector.select_fonts(&processed_text)?;
        
        // Stage 4: Shape text
        let shaping_results = self.shape_segments(&processed_text, &font_selection, &options.shape_options)?;
        
        // Stage 5: Render glyphs
        let render_output = self.render_shaped(&shaping_results, &options.render_options)?;
        
        // Stage 6: Export result
        let export_result = self.exporter.export(&render_output, &options.export_options)?;
        
        Ok(export_result)
    }
}
```

## Error Handling

Each stage reports specific failures:

```rust
pub enum PipelineError {
    InputFailed { source: ParseError },
    UnicodeFailed { source: UnicodeError },
    FontSelectionFailed { source: FontError },
    ShapingFailed { source: ShapingError },
    RenderingFailed { source: RenderError },
    ExportFailed { source: ExportError },
}
```

## Performance Monitoring

Track what matters:

```rust
pub struct PipelineMetrics {
    pub stage_durations: HashMap<String, Duration>,
    pub memory_usage: HashMap<String, usize>,
    pub cache_hit_rates: HashMap<String, f64>,
    pub total_duration: Duration,
}
```

## Next Steps

- [Backend Architecture](06-backend-architecture.md) - How backends implement stages
- [Memory Management](07-memory-management.md) - Efficient resource usage
- [Performance](08-performance-fundamentals.md) - Optimization strategies

---

Six stages transform text into pixels. Each stage handles one job well.