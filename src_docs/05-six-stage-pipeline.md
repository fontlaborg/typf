---
title: The Six-Stage Pipeline
icon: lucide/git-merge
tags:
  - Pipeline
  - Architecture
  - Data Flow
---

# The Six-Stage Pipeline

The heart of TYPF v2.0 is its modular six-stage pipeline that transforms raw text input into beautifully rendered output. Each stage has clear responsibilities, well-defined interfaces, and can be swapped with alternative implementations.

## Pipeline Overview

```
Input Parsing → Unicode Processing → Font Selection → Shaping → Rendering → Export
     ↓               ↓                  ↓           ↓          ↓        ↓
   TextBuffer    ProcessedText     FontHandle   GlyphBuffer RenderOutput ExportResult
```

## Stage 1: Input Parsing

### Purpose

Convert raw input into structured, validated data ready for processing by the rest of the pipeline.

### Responsibilities

- **Text Validation**: Ensure input is valid UTF-8/UTF-16
- **Parameter Extraction**: Parse font size, color, alignment options
- **Format Detection**: Identify input format (plain text, markup, etc.)
- **Normalization**: Convert to internal representation

### Key Data Structures

```rust
pub struct TextBuffer {
    pub text: String,
    pub language: Option<Language>,
    pub script: Option<Script>,
    pub direction: Direction,
    pub metadata: HashMap<String, String>,
}

pub struct ParseOptions {
    pub default_language: Language,
    pub auto_detect_script: bool,
    pub preserve_whitespace: bool,
    pub normalize_text: bool,
}
```

### Implementation Details

```rust
impl InputParser {
    pub fn parse(&self, input: &str, options: &ParseOptions) -> Result<TextBuffer> {
        // 1. Validate encoding
        let text = self.validate_encoding(input)?;
        
        // 2. Extract metadata
        let metadata = self.extract_metadata(input)?;
        
        // 3. Normalize if requested
        let normalized = if options.normalize_text {
            self.normalize_unicode(&text)?
        } else {
            text
        };
        
        // 4. Create buffer
        Ok(TextBuffer {
            text: normalized,
            language: options.default_language,
            script: if options.auto_detect_script {
                self.detect_script(&text)?
            } else {
                None
            },
            direction: Direction::LeftToRight, // Will be determined in Unicode stage
            metadata,
        })
    }
}
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid UTF-8 sequence at byte {position}")]
    InvalidUtf8 { position: usize },
    
    #[error("Unsupported input format: {format}")]
    UnsupportedFormat { format: String },
    
    #[error("Text too large: {size} bytes (maximum: {max})")]
    TextTooLarge { size: usize, max: usize },
}
```

### Performance Considerations

- **Zero-Copy**: Use `Cow<str>` for borrowed data when possible
- ** SIMD UTF-8 Validation**: Use vectorized operations for large texts
- **Memory Pooling**: Reuse buffers for repeated processing

## Stage 2: Unicode Processing

### Purpose

Prepare text for complex script rendering by analyzing Unicode properties and handling bidirectional text.

### Responsibilities

- **Script Detection**: Identify writing systems (Latin, Arabic, Devanagari, etc.)
- **Bidirectional Analysis**: Handle LTR/RTL mixed text
- **Text Segmentation**: Break text into logical units
- **Normalization**: Apply Unicode normalization forms

### Key Data Structures

```rust
pub struct ProcessedText {
    pub original: String,
    pub segments: Vec<TextSegment>,
    pub base_direction: Direction,
    pub unicode_version: UnicodeVersion,
}

pub struct TextSegment {
    pub range: Range<usize>,
    pub script: Script,
    pub direction: Direction,
    pub language: Option<Language>,
    pub breaks: Vec<BreakPosition>,
}
```

### Implementation Details

```rust
impl UnicodeProcessor {
    pub fn process(&self, text_buffer: &TextBuffer) -> Result<ProcessedText> {
        // 1. Script detection
        let script_runs = self.detect_scripts(&text_buffer.text)?;
        
        // 2. Bidi analysis
        let bidi_runs = self.analyze_bidirectional(&text_buffer.text)?;
        
        // 3. Merge runs and create segments
        let segments = self.create_segments(script_runs, bidi_runs)?;
        
        // 4. Determine base direction
        let base_direction = self.determine_base_direction(&segments);
        
        Ok(ProcessedText {
            original: text_buffer.text.clone(),
            segments,
            base_direction,
            unicode_version: UNICODE_VERSION,
        })
    }
    
    fn detect_scripts(&self, text: &str) -> Result<Vec<ScriptRun>> {
        let mut detector = ScriptDetector::new();
        let mut runs = Vec::new();
        
        for (char_index, ch) in text.char_indices() {
            let script = detector.script_for_char(ch)?;
            
            if runs.is_empty() || runs.last().unwrap().script != script {
                runs.push(ScriptRun {
                    start: char_index,
                    script,
                    count: 1,
                });
            } else {
                runs.last_mut().unwrap().count += 1;
            }
        }
        
        Ok(runs)
    }
}
```

### Script Detection

```rust
pub struct ScriptDetector {
    // Use ICU properties for accurate script detection
    icu_script_detector: icu_properties::Script,
}

impl ScriptDetector {
    pub fn script_for_char(&self, ch: char) -> Result<Script> {
        match self.icu_script_detector.get_script(ch) {
            icu_properties::Script::Latin => Ok(Script::Latin),
            icu_properties::Script::Arabic => Ok(Script::Arabic),
            icu_properties::Script::Devanagari => Ok(Script::Devanagari),
            // ... more scripts
            icu_properties::Script::Common => Ok(Script::Common),
            icu_properties::Script::Inherited => Ok(Script::Inherited),
            script => Err(Error::UnsupportedScript(format!("{:?}", script))),
        }
    }
}
```

### Bidirectional Text Handling

```rust
pub struct BidiAnalyzer {
    bidi_para: unicode_bidi::BidiInfo,
}

impl BidiAnalyzer {
    pub fn analyze(&self, text: &str) -> Result<Vec<BidiRun>> {
        let bidi_info = unicode_bidi::BidiInfo::new(text, None);
        
        let mut runs = Vec::new();
        for line in bidi_info.paragraphs {
            runs.extend(self.analyze_paragraph(&line)?);
        }
        
        Ok(runs)
    }
}
```

### Performance Optimizations

- **Vectorized Script Detection**: Use SIMD for character classification
- **Lazy Bidi Analysis**: Only analyze when RTL characters are present
- **Segment Caching**: Cache results for repeated text segments

## Stage 3: Font Selection

### Purpose

Choose the optimal fonts for rendering each text segment, handling font fallback and matching.

### Responsibilities

- **Font Matching**: Find fonts that support required characters/scripts
- **Fallback Selection**: Choose backup fonts for unsupported characters
- **Style Matching**: Match weight, width, and style requirements
- **System Integration**: Use system font databases and caches

### Key Data Structures

```rust
pub struct FontHandle {
    pub font: Arc<Font>,
    pub id: FontId,
    pub family: String,
    pub style: FontStyle,
    pub supports_scripts: HashSet<Script>,
}

pub struct FontSelectionResult {
    pub selections: Vec<FontSelection>,
    pub fallbacks_used: bool,
    pub unsupported_chars: Vec<char>,
}

pub struct FontSelection {
    pub range: Range<usize>,
    pub font: FontHandle,
    pub reason: SelectionReason,
}
```

### Implementation Details

```rust
impl FontSelector {
    pub fn select_fonts(&self, text: &ProcessedText) -> Result<FontSelectionResult> {
        let mut selections = Vec::new();
        let mut unsupported_chars = Vec::new();
        let mut fallbacks_used = false;
        
        for segment in &text.segments {
            let segment_text = &text.original[segment.range.clone()];
            
            // 1. Try primary font
            if let Some(font) = self.try_primary_font(segment, &text.metadata)? {
                selections.push(FontSelection {
                    range: segment.range.clone(),
                    font,
                    reason: SelectionReason::PrimaryMatch,
                });
                continue;
            }
            
            // 2. Try script-specific fonts
            if let Some(font) = self.try_script_font(segment.script)? {
                selections.push(FontSelection {
                    range: segment.range.clone(),
                    font,
                    reason: SelectionReason::ScriptMatch,
                });
                fallbacks_used = true;
                continue;
            }
            
            // 3. Character-level fallback
            let char_selections = self.character_fallback(segment_text, segment)?;
            selections.extend(char_selections);
            fallbacks_used = true;
        }
        
        Ok(FontSelectionResult {
            selections,
            fallbacks_used,
            unsupported_chars,
        })
    }
    
    fn try_primary_font(&self, segment: &TextSegment, metadata: &HashMap<String, String>) -> Result<Option<FontHandle>> {
        if let Some(font_name) = metadata.get("font") {
            if let Some(font) = self.font_database.get_font(font_name)? {
                if font.supports_scripts.contains(&segment.script) {
                    return Ok(Some(font));
                }
            }
        }
        Ok(None)
    }
}
```

### Font Matching Algorithm

```rust
pub struct FontMatcher {
    criteria: MatchingCriteria,
}

impl FontMatcher {
    pub fn match_font(&self, required_chars: &[char], script: Script, style: &FontStyle) -> Result<Option<FontHandle>> {
        let candidates = self.get_candidates(script, style)?;
        
        for candidate in candidates {
            if self.supports_all_chars(&candidate, required_chars)? {
                return Ok(Some(candidate));
            }
        }
        
        Ok(None)
    }
    
    fn supports_all_chars(&self, font: &FontHandle, chars: &[char]) -> Result<bool> {
        for &ch in chars {
            if !font.font.contains_char(ch)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
```

### Performance Considerations

- **Font Caching**: LRU cache for recently used fonts
- **Lazy Loading**: Load fonts only when needed
- **Script Indexing**: Index fonts by supported scripts for fast lookup

## Stage 4: Shaping

### Purpose

Convert character sequences into positioned glyphs, handling complex script rules.

### Responsibilities

- **Glyph Substitution**: Apply GSUB features (ligatures, alternates)
- **Glyph Positioning**: Apply GPOS features (kerning, positioning)
- **Complex Rules**: Handle Arabic shaping, Indic reordering
- **Metrics Calculation**: Compute advances, bearings, and extents

### Key Data Structures

```rust
pub struct ShapingResult {
    pub glyphs: Vec<Glyph>,
    pub advances: Vec<f32>,
    pub positions: Vec<Position>,
    pub clusters: Vec<usize>,
    pub direction: Direction,
    pub script: Script,
}

pub struct Glyph {
    pub id: GlyphId,
    pub codepoint: char,
    pub font: FontHandle,
    pub x_advance: f32,
    pub y_advance: f32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub cluster: usize,
}
```

### Implementation Details

```rust
impl Shaper {
    pub fn shape(&self, text: &str, font: &FontHandle, options: &ShapeOptions) -> Result<ShapingResult> {
        let mut buffer = harfbuzz_rs::UnicodeBuffer::new();
        
        // 1. Set text properties
        buffer.set_direction(options.direction.into());
        buffer.set_script(options.script.into());
        buffer.set_language(options.language.into());
        
        // 2. Add text
        buffer.add_str(text);
        
        // 3. Set features
        let features = self.resolve_features(options)?;
        buffer.set_features(&features);
        
        // 4. Shape
        let output = harfbuzz_rs::shape(&font.font, buffer, &features);
        
        // 5. Convert to internal format
        self.convert_harfbuzz_output(output, font)
    }
    
    fn resolve_features(&self, options: &ShapeOptions) -> Result<Vec<harfbuzz_rs::Feature>> {
        let mut features = Vec::new();
        
        // Common features
        features.push(harfbuzz_rs::Feature::new('kern', 1, harfbuzz_rs::FeatureFlags::Global));
        features.push(harfbuzz_rs::Feature::new('liga', 1, harfbuzz_rs::FeatureFlags::Global));
        
        // Script-specific features
        match options.script {
            Script::Arabic => {
                features.push(harfbuzz_rs::Feature::new('rlig', 1, harfbuzz_rs::FeatureFlags::Global));
                features.push(harfbuzz_rs::Feature::new('calt', 1, harfbuzz_rs::FeatureFlags::Global));
            },
            Script::Devanagari => {
                features.push(harfbuzz_rs::Feature::new('locl', 1, harfbuzz_rs::FeatureFlags::Global));
                features.push(harfbuzz_rs::Feature::new('blwf', 1, harfbuzz_rs::FeatureFlags::Global));
            },
            _ => {}
        }
        
        Ok(features)
    }
}
```

### Complex Script Handling

```rust
pub struct ComplexScriptShaper {
    base_shaper: Box<dyn Shaper>,
    icu_processor: IcuProcessor,
}

impl ComplexScriptShaper {
    pub fn shape_complex(&self, text: &str, font: &FontHandle) -> Result<ShapingResult> {
        match font.supports_scripts.iter().next() {
            Some(Script::Arabic) => self.shape_arabic(text, font),
            Some(Script::Devanagari) => self.shape_devanagari(text, font),
            _ => self.base_shaper.shape(text, font, &Default::default()),
        }
    }
    
    fn shape_arabic(&self, text: &str, font: &FontHandle) -> Result<ShapingResult> {
        // 1. Apply Arabic contextual analysis
        let contextual_text = self.apply_arabic_context(text)?;
        
        // 2. Shape with HarfBuzz
        let shaped = self.base_shaper.shape(&contextual_text, font, &ShapeOptions {
            script: Script::Arabic,
            direction: Direction::RightToLeft,
            ..Default::default()
        })?;
        
        Ok(shaped)
    }
}
```

### Performance Optimizations

- **Result Caching**: Cache shaping results for repeated text
- **Partial Shaping**: Shape only changed segments
- **Parallel Processing**: Shape independent segments concurrently

## Stage 5: Rendering

### Purpose

Convert positioned glyphs into visual output, applying colors, effects, and transformations.

### Responsibilities

- **Rasterization**: Convert glyphs to pixel data or vectors
- **Color Application**: Apply colors, gradients, and effects
- **Subpixel Rendering**: Handle subpixel positioning and anti-aliasing
- **Transformations**: Apply scaling, rotation, and other transforms

### Key Data Structures

```rust
pub struct RenderOutput {
    pub data: RenderData,
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub dpi: f32,
    pub transform: Transform,
}

pub enum RenderData {
    Bitmap(Vec<u8>),
    Vector(Vec<PathElement>),
    Mixed(Vec<RenderLayer>),
}

pub struct RenderLayer {
    pub data: Vec<u8>,
    pub position: (f32, f32),
    pub blend_mode: BlendMode,
    pub opacity: f32,
}
```

### Implementation Details

```rust
impl Renderer {
    pub fn render(&self, glyphs: &[Glyph], context: &RenderContext) -> Result<RenderOutput> {
        let mut canvas = self.create_canvas(context.size, context.format)?;
        
        // 1. Clear background
        self.clear_background(&mut canvas, &context.background_color)?;
        
        // 2. Sort glyphs by z-order
        let mut sorted_glyphs = glyphs.to_vec();
        sorted_glyphs.sort_by(|a, b| a.z_order.cmp(&b.z_order));
        
        // 3. Render each glyph
        for glyph in &sorted_glyphs {
            self.render_glyph(&mut canvas, glyph, context)?;
        }
        
        // 4. Apply post-processing
        self.apply_post_processing(&mut canvas, context)?;
        
        Ok(RenderOutput {
            data: canvas.into_data(),
            width: context.size.width,
            height: context.size.height,
            format: context.format,
            dpi: context.dpi,
            transform: context.transform,
        })
    }
    
    fn render_glyph(&self, canvas: &mut Canvas, glyph: &Glyph, context: &RenderContext) -> Result<()> {
        let glyph_bitmap = self.rasterize_glyph(glyph, context.scale)?;
        let positioned = self.apply_positioning(glyph_bitmap, glyph.position)?;
        let colored = self.apply_color(positioned, &context.color)?;
        
        canvas.composite(colored, glyph.position, BlendMode::SourceOver)
    }
}
```

### Subpixel Rendering

```rust
pub struct SubpixelRenderer {
    subpixel_order: SubpixelOrder,
    filter_algorithm: FilterAlgorithm,
}

impl SubpixelRenderer {
    pub fn render_subpixel(&self, glyph: &Glyph) -> Result<SubpixelBitmap> {
        // 1. Render at 3x horizontal resolution
        let high_res = self.render_high_resolution(glyph)?;
        
        // 2. Filter for subpixel smoothing
        let filtered = self.apply_subpixel_filter(high_res)?;
        
        // 3. Downsample to final resolution
        let downsampled = self.downsample(filtered)?;
        
        Ok(downsampled)
    }
    
    fn apply_subpixel_filter(&self, bitmap: HighResBitmap) -> Result<HighResBitmap> {
        use std::arch::x86_64::*;
        
        unsafe {
            let filter = simd_load_filter(&self.filter_algorithm);
            
            for pixel_row in bitmap.pixels.chunks_exact_mut(8) {
                let pixels = _mm_loadu_si128(pixel_row.as_ptr() as *const __m128i);
                let filtered = _mm_mullo_epi16(pixels, filter);
                _mm_storeu_si128(pixel_row.as_mut_ptr() as *mut __m128i, filtered);
            }
        }
        
        Ok(bitmap)
    }
}
```

## Stage 6: Export

### Purpose

Convert rendered output into various file formats, applying encoding, compression, and metadata.

### Responsibilities

- **Format Encoding**: Convert to PNG, SVG, PDF, etc.
- **Metadata Embedding**: Add author, copyright, and creation info
- **Compression**: Apply optimal compression for each format
- **Validation**: Ensure output format compliance

### Key Data Structures

```rust
pub struct ExportResult {
    pub data: Vec<u8>,
    pub format: ExportFormat,
    pub metadata: ExportMetadata,
    pub size: FileSize,
}

pub struct ExportOptions {
    pub format: ExportFormat,
    pub quality: Quality,
    pub compression: CompressionLevel,
    pub metadata: ExportMetadata,
    pub color_profile: Option<ColorProfile>,
}
```

### Implementation Details

```rust
impl Exporter {
    pub fn export(&self, output: &RenderOutput, options: &ExportOptions) -> Result<ExportResult> {
        match options.format {
            ExportFormat::Png => self.export_png(output, options),
            ExportFormat::Svg => self.export_svg(output, options),
            ExportFormat::Pdf => self.export_pdf(output, options),
            ExportFormat::Jpeg => self.export_jpeg(output, options),
            ExportFormat::Json => self.export_json(output, options),
        }
    }
    
    fn export_png(&self, output: &RenderOutput, options: &ExportOptions) -> Result<ExportResult> {
        let encoder = image::codecs::png::PngEncoder::new_with_quality(
            Vec::new(),
            image::codecs::png::CompressionType::Best,
            options.compression.into(),
        );
        
        let image = image::ImageBuffer::from_raw(
            output.width,
            output.height,
            output.data.to_bitmap()?,
        ).ok_or(Error::InvalidImageData)?;
        
        let mut png_data = Vec::new();
        image.write_with_encoder(encoder, &mut png_data)?;
        
        Ok(ExportResult {
            data: png_data,
            format: ExportFormat::Png,
            metadata: options.metadata.clone(),
            size: FileSize::Bytes(png_data.len()),
        })
    }
}
```

### Vector Export

```rust
impl SvgExporter {
    pub fn export_svg(&self, output: &RenderOutput) -> Result<ExportResult> {
        let mut svg = SvgDocument::new(output.width, output.height);
        
        // Add metadata
        svg.add_metadata(&self.create_metadata(output)?);
        
        // Convert paths
        if let RenderData::Vector(paths) = &output.data {
            for path in paths {
                svg.add_path(path, &self.current_style()?;
            }
        }
        
        let svg_data = svg.finalize()?;
        
        Ok(ExportResult {
            data: svg_data.into_bytes(),
            format: ExportFormat::Svg,
            metadata: ExportMetadata::default(),
            size: FileSize::Bytes(svg_data.len()),
        })
    }
}
```

## Pipeline Orchestration

### Pipeline Builder

```rust
pub struct PipelineBuilder {
    parser: Option<Box<dyn InputParser>>,
    unicode_processor: Option<Box<dyn UnicodeProcessor>>,
    font_selector: Option<Box<dyn FontSelector>>,
    shaper: Option<Box<dyn Shaper>>,
    renderer: Option<Box<dyn Renderer>>,
    exporter: Option<Box<dyn Exporter>>,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            parser: None,
            unicode_processor: None,
            font_selector: None,
            shaper: None,
            renderer: None,
            exporter: None,
        }
    }
    
    pub fn with_parser(mut self, parser: Box<dyn InputParser>) -> Self {
        self.parser = Some(parser);
        self
    }
    
    pub fn build(self) -> Result<Pipeline> {
        Ok(Pipeline {
            parser: self.parser.ok_or(Error::MissingComponent("parser"))?,
            unicode_processor: self.unicode_processor.ok_or(Error::MissingComponent("unicode_processor"))?,
            font_selector: self.font_selector.ok_or(Error::MissingComponent("font_selector"))?,
            shaper: self.shaper.ok_or(Error::MissingComponent("shaper"))?,
            renderer: self.renderer.ok_or(Error::MissingComponent("renderer"))?,
            exporter: self.exporter.ok_or(Error::MissingComponent("exporter"))?,
        })
    }
}
```

### Pipeline Execution

```rust
impl Pipeline {
    pub fn process(&self, input: &str, options: &ProcessOptions) -> Result<ExportResult> {
        // Stage 1: Input Parsing
        let text_buffer = self.parser.parse(input, &options.parse_options)?;
        
        // Stage 2: Unicode Processing
        let processed_text = self.unicode_processor.process(&text_buffer)?;
        
        // Stage 3: Font Selection
        let font_selection = self.font_selector.select_fonts(&processed_text)?;
        
        // Stage 4: Shaping
        let shaping_results = self.shape_segments(&processed_text, &font_selection, &options.shape_options)?;
        
        // Stage 5: Rendering
        let render_output = self.render_shaped(&shaping_results, &options.render_options)?;
        
        // Stage 6: Export
        let export_result = self.exporter.export(&render_output, &options.export_options)?;
        
        Ok(export_result)
    }
    
    fn shape_segments(&self, text: &ProcessedText, fonts: &FontSelectionResult, options: &ShapeOptions) -> Result<Vec<ShapedSegment>> {
        let mut results = Vec::new();
        
        for selection in &fonts.selections {
            let segment_text = &text.original[selection.range.clone()];
            let shaped = self.shaper.shape(segment_text, &selection.font, options)?;
            results.push(ShapedSegment {
                range: selection.range.clone(),
                shaped,
                font: selection.font.clone(),
            });
        }
        
        Ok(results)
    }
}
```

## Error Handling Across Stages

```rust
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Stage 1 (Input Parsing) failed: {source}")]
    InputFailed { source: ParseError },
    
    #[error("Stage 2 (Unicode Processing) failed: {source}")]
    UnicodeFailed { source: UnicodeError },
    
    #[error("Stage 3 (Font Selection) failed: {source}")]
    FontSelectionFailed { source: FontError },
    
    #[error("Stage 4 (Shaping) failed: {source}")]
    ShapingFailed { source: ShapingError },
    
    #[error("Stage 5 (Rendering) failed: {source}")]
    RenderingFailed { source: RenderError },
    
    #[error("Stage 6 (Export) failed: {source}")]
    ExportFailed { source: ExportError },
}
```

## Performance Monitoring

```rust
pub struct PipelineMetrics {
    pub stage_durations: HashMap<String, Duration>,
    pub memory_usage: HashMap<String, usize>,
    pub cache_hit_rates: HashMap<String, f64>,
    pub total_duration: Duration,
}

impl Pipeline {
    pub fn process_with_metrics(&self, input: &str, options: &ProcessOptions) -> Result<(ExportResult, PipelineMetrics)> {
        let start_time = Instant::now();
        let mut metrics = PipelineMetrics::default();
        
        // Stage 1 with timing
        let stage_start = Instant::now();
        let text_buffer = self.parser.parse(input, &options.parse_options)?;
        metrics.stage_durations.insert("parsing".to_string(), stage_start.elapsed());
        
        // ... continue for all stages
        
        metrics.total_duration = start_time.elapsed();
        
        Ok((export_result, metrics))
    }
}
```

## Next Steps

Now that you understand the six-stage pipeline:

- [Backend Architecture](06-backend-architecture.md) - How backends implement pipeline stages
- [Memory Management](07-memory-management.md) - Efficient memory usage
- [Performance Fundamentals](08-performance-fundamentals.md) - Optimization strategies

---

**The six-stage pipeline** provides a clean, modular architecture that makes TYPF both powerful and flexible. Each stage has clear responsibilities and can be optimized independently while maintaining excellent performance overall.
