# Chapter 18: Rust API

## Overview

The TYPF Rust API provides a comprehensive, type-safe interface for integrating high-quality text rendering into Rust applications. Built with Rust's ownership system and zero-cost abstractions in mind, the API offers both high-level convenience functions and low-level control for advanced use cases. This chapter covers the complete Rust API surface, from basic text rendering to advanced pipeline configuration.

## Architecture

### Core API Structure

```rust
// Main entry point
pub struct Typf {
    pipeline: Pipeline,
    config: TypfConfig,
    font_database: Arc<FontDatabase>,
}

// Configuration
#[derive(Debug, Clone)]
pub struct TypfConfig {
    pub shaper: ShaperConfig,
    pub renderer: RendererConfig,
    pub exporter: ExporterConfig,
    pub cache: CacheConfig,
}

// High-level convenience API
impl Typf {
    pub fn new() -> Result<Self, TypfError>;
    pub fn with_config(config: TypfConfig) -> Result<Self, TypfError>;
    pub fn default() -> Self;
}

// Low-level pipeline API
impl Typf {
    pub fn pipeline(&self) -> &Pipeline;
    pub fn pipeline_mut(&mut self) -> &mut Pipeline;
    pub fn font_database(&self) -> &Arc<FontDatabase>;
}
```

## Basic Usage

### Simple Text Rendering

```rust
use typf::{Typf, ExportFormat};

fn basic_rendering() -> Result<(), Box<dyn std::error::Error>> {
    // Create Typf instance with default configuration
    let typf = Typf::new()?;
    
    // Render text to PNG
    let result = typf.render_text_to_bytes(
        "Hello, TYPF!",
        "/path/to/font.ttf",
        32.0,  // font size
        ExportFormat::PNG,
    )?;
    
    // Save to file
    std::fs::write("output.png", result.data)?;
    
    println!("Text rendered successfully!");
    Ok(())
}
```

### Custom Configuration

```rust
use typf::{Typf, TypfConfig, ShaperConfig, RendererConfig};
use typf::shapers::{HarfBuzzShaper, HarfBuzzConfig};
use typf::renderers::{SkiaRenderer, SkiaConfig};

fn custom_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let config = TypfConfig {
        shaper: ShaperConfig::HarfBuzz(HarfBuzzConfig {
            enable_kerning: true,
            enable_ligatures: true,
            script_detection: true,
        }),
        renderer: RendererConfig::Skia(SkiaConfig {
            antialiasing: true,
            subpixel_rendering: true,
            hinting: SkiaHinting::Slight,
        }),
        exporter: Default::default(),
        cache: Default::default(),
    };
    
    let typf = Typf::with_config(config)?;
    
    // Use custom configuration
    let result = typf.render_text_to_bytes(
        "Custom styled text",
        "/path/to/font.ttf",
        24.0,
        ExportFormat::PNG,
    )?;
    
    std::fs::write("custom_output.png", result.data)?;
    Ok(())
}
```

## Pipeline API

### Pipeline Construction

```rust
use typf::{Pipeline, PipelineBuilder, Context};
use typf::shapers::{Shaper, HarfBuzzShaper};
use typf::renderers::{Renderer, SkiaRenderer};

fn pipeline_construction() -> Result<(), Box<dyn std::error::Error>> {
    // Build custom pipeline
    let pipeline = PipelineBuilder::new()
        .with_shaper(HarfBuzzShaper::new()?)
        .with_renderer(SkiaRenderer::new()?)
        .with_exporter(PngExporter::default())
        .build()?;
    
    // Create context
    let mut context = Context::new();
    context.set_font_size(48.0);
    context.set_text("Pipeline rendering");
    
    // Execute pipeline
    let result = pipeline.execute(&context)?;
    
    println!("Pipeline executed successfully!");
    Ok(())
}
```

### Context Management

```rust
use typf::{Context, FontSettings, TextSettings};

fn context_management() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::new();
    
    // Configure text
    context.set_text("Advanced TYPF usage");
    context.set_text_settings(TextSettings {
        direction: TextDirection::LeftToRight,
        script: Script::Latin,
        language: Some("en".to_string()),
    });
    
    // Configure font
    context.set_font_path("/path/to/font.otf")?;
    context.set_font_settings(FontSettings {
        size: 36.0,
        weight: FontWeight::Regular,
        style: FontStyle::Normal,
        stretch: FontStretch::Normal,
    });
    
    // Configure rendering
    context.set_render_settings(RenderSettings {
        width: Some(800),
        height: Some(400),
        dpi: Some(300.0),
        background_color: Some(Color::WHITE),
    });
    
    Ok(())
}
```

## Shaper API

### Shaper Selection and Configuration

```rust
use typf::shapers::*;

fn shaper_examples() -> Result<(), Box<dyn std::error::Error>> {
    // HarfBuzz shaper (default)
    let hb_shaper = HarfBuzzShaper::with_config(HarfBuzzConfig {
        enable_kerning: true,
        enable_ligatures: true,
        script_detection: true,
        bidirectional_processing: true,
    })?;
    
    // None shaper (minimal)
    let none_shaper = NoneShaper::new();
    
    // ICU-HarfBuzz composition
    let icu_hb_shaper = IcuHarfBuzzShaper::new()?;
    
    // Platform shapers
    #[cfg(target_os = "macos")]
    let coretext_shaper = CoreTextShaper::new()?;
    
    #[cfg(target_os = "windows")]
    let directwrite_shaper = DirectWriteShaper::new()?;
    
    // Use shaper directly
    let font = Font::from_path("/path/to/font.ttf")?;
    let text = "Shaping demonstration";
    let shaped = hb_shaper.shape(text, &font, 24.0)?;
    
    println!("Shaped {} glyphs", shaped.glyph_count());
    Ok(())
}
```

### Shaping Configuration

```rust
#[derive(Debug, Clone)]
pub struct ShapingConfig {
    pub features: Vec<FeatureSetting>,
    pub variations: Vec<VariationSetting>,
    pub direction: TextDirection,
    pub script: Option<Script>,
    pub language: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FeatureSetting {
    pub tag: u32,           // OpenType feature tag (4 bytes)
    pub enabled: bool,     // Enable/disable feature
    pub value: u32,        // Feature value (if applicable)
}

#[derive(Debug, Clone)]
pub struct VariationSetting {
    pub axis: Tag,         // Variation axis tag
    pub value: f32,        // Axis value
}

fn advanced_shaping() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = ShapingConfig::default();
    
    // Enable OpenType features
    config.features.extend_from_slice(&[
        FeatureSetting { tag: b"liga", enabled: true, value: 0 },
        FeatureSetting { tag: b"kern", enabled: true, value: 0 },
        FeatureSetting { tag: b"dlig", enabled: false, value: 0 },
    ]);
    
    // Set font variations (for variable fonts)
    config.variations.push(VariationSetting {
        axis: b"wght",  // Weight axis
        value: 600.0,   // Semi-bold
    });
    
    let shaper = HarfBuzzShaper::with_config(config)?;
    let font = Font::from_path("/path/to/variable_font.otf")?;
    let text = "Advanced shaping";
    let shaped = shaper.shape(text, &font, 32.0)?;
    
    println!("Advanced shaping completed");
    Ok(())
}
```

## Renderer API

### Renderer Selection

```rust
use typf::renderers::*;

fn renderer_examples() -> Result<(), Box<dyn std::error::Error>> {
    // Skia renderer (default, most capable)
    let skia_renderer = SkiaRenderer::with_config(SkiaConfig {
        antialiasing: true,
        subpixel_rendering: true,
        hinting: SkiaHinting::Full,
        use_gamma_correction: true,
    })?;
    
    // Orge renderer (minimal, pure Rust)
    let orge_renderer = OrgeRenderer::with_config(OrgeConfig {
        antialiasing: true,
        gamma_correction: 2.2,
        simd_acceleration: true,
    });
    
    // Platform renderers
    #[cfg(target_os = "macos")]
    let coregraphics_renderer = CoreGraphicsRenderer::new()?;
    
    #[cfg(target_os = "windows")]
    let direct2d_renderer = Direct2DRenderer::new()?;
    
    // Zeno vector renderer
    let zeno_renderer = ZenoRenderer::with_config(ZenoConfig {
        format: VectorFormat::SVG,
        precision: 2.0,
        optimization: OptimizationConfig::default(),
    })?;
    
    Ok(())
}
```

### Rendering Configuration

```rust
use typf::{RenderConfig, Color, Transform};

fn rendering_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let config = RenderConfig {
        // Output dimensions
        width: Some(1024),
        height: Some(768),
        
        // Color settings
        text_color: Color::RGB(0, 0, 0),           // Black text
        background_color: Some(Color::RGB(255, 255, 255)), // White background
        
        // Quality settings
        antialiasing: true,
        subpixel_rendering: true,
        hinting: HintingLevel::Medium,
        
        // Transform
        transform: Transform::identity()
            .then_scale(1.5, 1.5)  // 150% scale
            .then_translate(50.0, 25.0),  // Offset
        
        // Advanced options
        use_gamma_correction: true,
        simulate_print_colors: false,
    };
    
    let renderer = SkiaRenderer::with_config(config)?;
    
    // Render with custom settings
    let font = Font::from_path("/path/to/font.ttf")?;
    let shaped = shape_text("Custom rendering", &font, 24.0)?;
    let result = renderer.render_shaped_text(&shaped)?;
    
    println!("Custom rendering completed");
    Ok(())
}
```

## Export API

### Export Configuration

```rust
use typf::{ExportConfig, PngConfig, SvgConfig, JpegConfig};

fn export_configuration() -> Result<(), Box<dyn std::error::Error>> {
    // PNG export
    let png_config = ExportConfig::png(PngConfig {
        compression: PngCompression::Best,
        filter: PngFilter::Adaptive,
        color_type: PngColorType::RGBA,
    });
    
    // JPEG export
    let jpeg_config = ExportConfig::jpeg(JpegConfig {
        quality: 85,
        chroma_subsampling: ChromaSubsampling::Subsample420,
        color_space: JpegColorSpace::YCbCr,
    });
    
    // SVG export
    let svg_config = ExportConfig::svg(SvgConfig {
        pretty_print: true,
        precision: 2,
        embed_fonts: false,
        include_metadata: true,
    });
    
    // PDF export
    let pdf_config = ExportConfig::pdf(PdfConfig {
        create_outline: false,
        embed_fonts: true,
        compression: PdfCompression::Flate,
        metadata: PdfMetadata::default(),
    });
    
    Ok(())
}
```

### Batch Export

```rust
use typf::{BatchExporter, ExportJob};
use rayon::prelude::*;

fn batch_export() -> Result<(), Box<dyn std::error::Error>> {
    let exporter = BatchExporter::new()?;
    
    // Create export jobs
    let jobs = vec![
        ExportJob {
            text: "Sample 1".to_string(),
            font_path: "/path/to/font1.otf".to_string(),
            size: 24.0,
            format: ExportFormat::PNG,
            output_path: "sample1.png".to_string(),
        },
        ExportJob {
            text: "Sample 2".to_string(),
            font_path: "/path/to/font2.otf".to_string(),
            size: 32.0,
            format: ExportFormat::SVG,
            output_path: "sample2.svg".to_string(),
        },
    ];
    
    // Process jobs in parallel
    let results: Vec<Result<ExportResult, TypfError>> = jobs
        .into_par_iter()
        .map(|job| exporter.process_job(job))
        .collect();
    
    // Handle results
    for result in results {
        match result {
            Ok(output) => println!("Exported: {}", output.path),
            Err(e) => eprintln!("Export failed: {}", e),
        }
    }
    
    Ok(())
}
```

## Font Management

### Font Database

```rust
use typf::{FontDatabase, FontInfo, FontQuery};

fn font_management() -> Result<(), Box<dyn std::error::Error>> {
    // Create font database
    let mut font_db = FontDatabase::new();
    
    // Add fonts from directories
    font_db.add_font_directory("/System/Library/Fonts")?;
    font_db.add_font_directory("/usr/share/fonts")?;
    
    // Add individual font files
    font_db.add_font_file("/path/to/custom_font.otf")?;
    
    // Query fonts
    let query = FontQuery {
        family: Some("Helvetica".to_string()),
        weight: Some(FontWeight::Bold),
        style: Some(FontStyle::Normal),
        ..Default::default()
    };
    
    let fonts: Vec<FontInfo> = font_db.query_fonts(query)?;
    
    for font in &fonts {
        println!("Found font: {} {} {}", font.family, font.style, font.weight);
    }
    
    // Get font by family and style
    let helvetica_bold = font_db.get_font("Helvetica", FontWeight::Bold, FontStyle::Normal)?;
    
    Ok(())
}
```

### Font Loading and Caching

```rust
use typf::{Font, FontLoader, CacheConfig};

fn font_loading() -> Result<(), Box<dyn std::error::Error>> {
    // Configure font cache
    let cache_config = CacheConfig {
        max_memory: 100 * 1024 * 1024,  // 100MB
        max_fonts: 1000,
        ttl_seconds: 3600,  // 1 hour
    };
    
    let mut loader = FontLoader::with_cache(cache_config)?;
    
    // Load font with automatic caching
    let font = loader.load_font("/path/to/font.ttf")?;
    
    // Font is now cached for subsequent use
    let font_cached = loader.load_font("/path/to/font.ttf")?;  // From cache
    
    // Preload fonts into cache
    let font_paths = vec![
        "/path/to/font1.ttf",
        "/path/to/font2.otf",
        "/path/to/font3.woff2",
    ];
    
    loader.preload_fonts(font_paths)?;
    
    // Cache statistics
    let stats = loader.cache_stats();
    println!("Cache: {}/{} fonts, {}/{} bytes",
             stats.loaded_fonts, stats.max_fonts,
             stats.memory_used, stats.max_memory);
    
    Ok(())
}
```

## Error Handling

### Error Types

```rust
use typf::{TypfError, ShapingError, RenderingError, FontError};

fn error_handling_examples() {
    // Handle different error types
    match Typf::new() {
        Ok(typf) => println!("TYPF initialized successfully"),
        Err(TypfError::FontError(FontError::NotFound(path))) => {
            eprintln!("Font not found: {}", path);
        },
        Err(TypfError::ShapingError(ShapingError::UnsupportedScript(script))) => {
            eprintln!("Unsupported script: {:?}", script);
        },
        Err(TypfError::RenderingError(RenderingError::InvalidDimensions)) => {
            eprintln!("Invalid rendering dimensions");
        },
        Err(e) => eprintln!("General error: {}", e),
    }
}

fn error_recovery() -> Result<(), Box<dyn std::error::Error>> {
    // Create TYPF with fallback configuration
    let typf = match Typf::new() {
        Ok(t) => t,
        Err(e) => {
            // Try minimal configuration
            eprintln!("Full initialization failed, trying minimal: {}", e);
            Typf::with_config(TypfConfig::minimal())?
        }
    };
    
    // Render text with fallback
    let result = match typf.render_text_to_bytes(
        "Text with fallback",
        "/path/to/font.ttf",
        24.0,
        ExportFormat::PNG,
    ) {
        Ok(r) => r,
        Err(TypfError::FontError(_)) => {
            // Use system font as fallback
            println!("Using system font fallback");
            typf.render_text_to_bytes(
                "Text with fallback",
                "system-default",
                24.0,
                ExportFormat::PNG,
            )?
        },
        Err(e) => return Err(e.into()),
    };
    
    println!("Rendered successfully with fallback");
    Ok(())
}
```

## Performance API

### Performance Monitoring

```rust
use typf::{PerformanceMonitor, PerformanceStats};
use std::time::Instant;

fn performance_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    let mut monitor = PerformanceMonitor::new();
    
    monitor.start_timing("full_render")?;
    
    // Load font
    monitor.start_timing("font_load")?;
    let font = Font::from_path("/path/to/font.ttf")?;
    monitor.end_timing("font_load")?;
    
    // Shape text
    monitor.start_timing("shaping")?;
    let shaper = HarfBuzzShaper::new()?;
    let shaped = shaper.shape("Performance test", &font, 32.0)?;
    monitor.end_timing("shaping")?;
    
    // Render text
    monitor.start_timing("rendering")?;
    let renderer = SkiaRenderer::new()?;
    let result = renderer.render_shaped_text(&shaped)?;
    monitor.end_timing("rendering")?;
    
    monitor.end_timing("full_render")?;
    
    // Get performance statistics
    let stats = monitor.get_stats();
    
    println!("Performance Statistics:");
    println!(" Font loading: {:?}", stats.get_timing("font_load"));
    println!(" Shaping: {:?}", stats.get_timing("shaping"));
    println!(" Rendering: {:?}", stats.get_timing("rendering"));
    println!(" Total: {:?}", stats.get_timing("full_render"));
    
    Ok(())
}
```

### Memory Management

```rust
use typf::{MemoryManager, MemoryConfig};

fn memory_management() -> Result<(), Box<dyn std::error::Error>> {
    let config = MemoryConfig {
        max_font_memory: 50 * 1024 * 1024,  // 50MB for fonts
        max_glyph_cache: 20 * 1024 * 1024,  // 20MB for glyphs
        max_render_memory: 100 * 1024 * 1024, // 100MB for renders
        gc_threshold: 0.8,  // Trigger GC at 80% usage
    };
    
    let memory_manager = MemoryManager::with_config(config);
    
    // Monitor memory usage
    let usage = memory_manager.get_memory_usage();
    println!("Memory Usage:");
    println!(" Font memory: {}MB", usage.font_memory / 1024 / 1024);
    println!(" Glyph cache: {}MB", usage.glyph_cache / 1024 / 1024);
    println!(" Render memory: {}MB", usage.render_memory / 1024 / 1024);
    
    // Trigger garbage collection if needed
    if usage.total_fraction() > config.gc_threshold {
        println!("Triggering garbage collection");
        memory_manager.garbage_collect()?;
    }
    
    Ok(())
}
```

## Advanced Features

### Custom Shapers

```rust
use typf::{Shaper, ShapingResult, GlyphPosition};

struct CustomShaper {
    config: CustomShaperConfig,
}

impl Shaper for CustomShaper {
    fn shape(
        &self,
        text: &str,
        font: &Font,
        size: f32,
    ) -> Result<ShapingResult, TypfError> {
        // Custom shaping logic
        let mut glyphs = Vec::new();
        let mut positions = Vec::new();
        
        for (i, ch) in text.chars().enumerate() {
            let glyph_id = font.get_glyph_id(ch)?;
            glyphs.push(glyph_id);
            
            positions.push(GlyphPosition {
                x_offset: 0.0,
                y_offset: 0.0,
                x_advance: size * 0.6,  // Simple advance calculation
                y_advance: 0.0,
            });
        }
        
        Ok(ShapingResult {
            glyphs,
            positions,
            text: text.to_string(),
            font_info: font.info().clone(),
            font_size: size,
            metrics: self.calculate_metrics(&glyphs, font, size)?,
        })
    }
}
```

### Pipeline Extensions

```rust
use typf::{Stage, PipelineBuilder, StageError};

struct CustomStage {
    name: String,
    config: CustomStageConfig,
}

impl Stage for CustomStage {
    type Input = ShapingResult;
    type Output = ShapingResult;
    
    fn process(&self, input: Self::Input) -> Result<Self::Output, StageError> {
        // Custom processing logic
        let mut processed = input.clone();
        
        // Apply custom transformations
        if self.config.optimize_spacing {
            processed = self.optimize_spacing(processed)?;
        }
        
        if self.config.adjust_metrics {
            processed = self.adjust_metrics(processed)?;
        }
        
        Ok(processed)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

fn custom_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    let custom_stage = CustomStage {
        name: "Spacing Optimizer".to_string(),
        config: CustomStageConfig::default(),
    };
    
    let pipeline = PipelineBuilder::new()
        .with_shaper(HarfBuzzShaper::new()?)
        .with_stage(Box::new(custom_stage))
        .with_renderer(SkiaRenderer::new()?)
        .build()?;
    
    println!("Custom pipeline created successfully");
    Ok(())
}
```

## Testing Support

### Test Utilities

```rust
#[cfg(test)]
mod tests {
    use typf::*;
    use typf::test_utils::*;
    
    #[test]
    fn test_basic_rendering() {
        let typf = Typf::new().unwrap();
        let result = typf.render_text_to_bytes(
            "Test",
            test_font_path(),
            24.0,
            ExportFormat::PNG,
        ).unwrap();
        
        assert!(!result.data.is_empty());
        assert_eq!(result.format, ExportFormat::PNG);
    }
    
    #[test]
    fn test_shaping_accuracy() {
        let font = load_test_font();
        let shaper = HarfBuzzShaper::new().unwrap();
        let shaped = shaper.shape("Hello", &font, 32.0).unwrap();
        
        // Compare against golden data
        let golden = load_golden_shaping_data("hello_32pt.json");
        assert_eq!(shaped, golden);
    }
    
    #[test]
    fn test_performance_benchmarks() {
        let typf = Typf::new().unwrap();
        let start = std::time::Instant::now();
        
        for i in 0..100 {
            let _ = typf.render_text_to_bytes(
                &format!("Text {}", i),
                test_font_path(),
                16.0,
                ExportFormat::PNG,
            ).unwrap();
        }
        
        let duration = start.elapsed();
        assert!(duration < std::time::Duration::from_millis(1000));
    }
}
```

## Best Practices

### Usage Patterns

1. **Reuse Instances**: Create one Typf instance and reuse it
2. **Configure Caching**: Set appropriate cache sizes for your workload
3. **Error Handling**: Always handle specific error types appropriately
4. **Memory Management**: Monitor and control memory usage in long-running applications
5. **Async Support**: Use async APIs in network or I/O bound applications

### Performance Optimization

1. **Preload Fonts**: Load frequently used fonts at startup
2. **Batch Processing**: Process multiple text items together
3. **Pipeline Configuration**: Choose minimal configuration for simple tasks
4. **Memory Pooling**: Reallocate buffers to reduce allocation overhead
5. **SIMD Acceleration**: Enable SIMD when available for performance-critical applications

The TYPF Rust API provides a comprehensive, performant, and type-safe interface for text rendering that leverages Rust's strengths in safety and performance while offering the flexibility needed for diverse use cases.