# Performance Optimization

Make Typf fast through strategic optimization techniques.

## Performance Summary

| Metric | Target | Current |
|--------|--------|---------|
| Simple Latin shaping | <10µs/100 chars | ~5µs |
| Complex Arabic shaping | <50µs/100 chars | ~45µs |
| Glyph rasterization (16px) | <1µs/glyph | ~0.8µs |
| RGBA blending (SIMD) | >10GB/s | ~12GB/s |
| L1 cache hit latency | <50ns | ~40ns |
| Binary size (minimal) | <500KB | ~500KB |
| Memory (1M chars) | <100MB | ~85MB |

## Backend Performance

### Comprehensive Performance Rankings

Based on extensive benchmark testing across different scripts, sizes, and font types:

| Backend | Performance | Quality | Use Case |
|---------|-------------|---------|----------|
| JSON Export | 15,506-22,661 ops/sec | Data only | Analysis, debug |
| CoreGraphics | 3,805-4,583 ops/sec | High quality | macOS best |
| Zeno | 3,048-3,675 ops/sec | High | Vector quality |
| Opixa | 1,959-2,302 ops/sec | Medium | Pure Rust, SIMD |
| Skia | 1,611-1,829 ops/sec | High quality | Cross-platform |

### Shaper Performance Analysis

| Shaper | Avg Time | Ops/sec | Recommendations |
|--------|----------|---------|----------------|
| none | 0.041ms | 24,673 | Testing/debug only |
| HarfBuzz | 0.050ms | 20,630 | **Best overall choice** |
| ICU-HarfBuzz | 0.061ms | 17,368 | Complex scripts only |
| CoreText | 0.065ms | 22,799 | macOS native apps |

**Key Insight**: HarfBuzz provides the best balance of speed and correctness for cross-platform applications.

### Renderer Performance Analysis

| Renderer | Avg Time | Ops/sec | Strengths | Weaknesses |
|----------|----------|---------|-----------|------------|
| CoreGraphics | 0.353ms | 5,045 | Fastest rasterization | macOS only |
| Opixa | 1.125ms | 2,538 | Pure Rust, portable | Medium quality |
| Zeno | 1.738ms | 785 | Vector quality | Slower rasterization |
| Skia | 1.762ms | 913 | Cross-platform, color | Larger dependency |

### Platform-Specific Recommendations

**macOS**: CoreText + CoreGraphics = Native performance (~4,500 ops/sec)
**Cross-platform**: HarfBuzz + Opixa = Best balance (~2,500 ops/sec)
**Linux**: HarfBuzz + Skia = Most compatible (~900 ops/sec)
**Windows**: DirectWrite + Direct2D = Native performance (when available)

### Performance Regressions Detected

Recent benchmarks identified several performance regressions requiring attention:

| Backend | Script | Size | Slowdown | Likely Cause |
|---------|--------|------|----------|--------------|
| coretext + JSON | arab | 128px | **+786%** | May be measurement noise |
| coretext + JSON | latn | 64px | +136% | Possible CoreText issue |
| HarfBuzz + zeno | mixd | 64px | +129% | Zeno backend regression |
| HarfBuzz + zeno | mixd | 128px | +101% | Mixed script overhead |

**Action Required**: Profile the Zeno backend and validate CoreText measurements.

## Quick Wins

### Font Caching

```rust
// Enable font caching (default)
let mut pipeline = PipelineBuilder::new()
    .enable_font_cache(true)
    .cache_size(100 * 1024 * 1024) // 100MB
    .build()?;

// Preload common fonts
pipeline.load_font("Roboto-Regular.ttf")?;
pipeline.load_font("OpenSans-Regular.ttf")?;
```

### Backend Selection

Based on benchmark results, here are the optimal backend configurations:

```rust
// macOS - Best performance (native)
#[cfg(target_os = "macos")]
let pipeline = PipelineBuilder::new()
    .shaper(ShaperBackend::CoreText)  // 22,799 ops/sec
    .renderer(RendererBackend::CoreGraphics)  // 5,045 ops/sec
    .build()?;

// Cross-platform - Best balance
let pipeline = PipelineBuilder::new()
    .shaper(ShaperBackend::HarfBuzz)  // 20,630 ops/sec
    .renderer(RendererBackend::Opixa)  // 2,584 ops/sec
    .build()?;

// High quality - When quality > speed
let pipeline = PipelineBuilder::new()
    .shaper(ShaperBackend::HarfBuzz)
    .renderer(RendererBackend::Skia)  // Color font support
    .build()?;

// Vector output - For SVG/PDF workflows
let pipeline = PipelineBuilder::new()
    .shaper(ShaperBackend::HarfBuzz)
    .renderer(RendererBackend::Zeno)  // Best vector quality
    .build()?;
```

### Font Format Compatibility

Based on testing with emoji, CBDT, sbix, COLR and mixed SVG fonts:

| Shaper | Renderer | COLR | SVG | sbix | CBDT | Notes |
|--------|----------|------|-----|------|------|-------|
| HarfBuzz | opixa | ✓ | ✓ | ✓ | ⚠️ | CBDT renders blank |
| HarfBuzz | skia | ✓ | ✓ | ✓ | ✗ | CBDT fails entirely |
| HarfBuzz | zeno | ✓ | ✓ | ✓ | ✗ | CBDT fails entirely |
| HarfBuzz | coregraphics | ✓ | ✓ | ✓ | ✗ | CBDT fails entirely |
| CoreText | opixa | ✓ | ✓ | ✓ | ✗ | CBDT can't shape |

**Recommendation**: Use HarfBuzz + Opixa for maximum compatibility. Avoid CBDT fonts for production use.

### Memory Efficiency

```rust
// Use appropriate buffer sizes
let render_options = RenderOptions {
    width: 800,
    height: 600,
    // Match font size to avoid unnecessary scaling
    pixel_size: font_size.round() as u32,
    ..Default::default()
};

// Reuse buffers
let mut buffer = BitmapBuffer::new(800, 600)?;
pipeline.render_to_buffer(&text, &font, &mut buffer)?;
```

## Measurement & Profiling

### Built-in Profiling

```rust
// Enable performance profiling
let pipeline = PipelineBuilder::new()
    .enable_profiling(true)
    .build()?;

let result = pipeline.render_text("Profiling test", &font, &options);
let metrics = result.get_performance_metrics();

println!("Shaping time: {}ms", metrics.shaping_time);
println!("Render time: {}ms", metrics.render_time);
println!("Total time: {}ms", metrics.total_time);
```

### Benchmarking

```rust
use typf::benchmark::{BenchmarkSuite, BenchmarkConfig};

let config = BenchmarkConfig {
    iterations: 1000,
    warmup_iterations: 100,
    text_samples: vec![
        "Hello World".to_string(),
        "Lorem ipsum dolor sit amet".to_string(),
        "Unicode test: 你好世界 🌍".to_string(),
    ],
    font_sizes: vec![12, 16, 24, 48],
};

let suite = BenchmarkSuite::new(config);
let results = suite.run_all_backends()?;

println!("Fastest backend: {:?}", results.fastest_backend());
println!("Average time: {:.2}ms", results.average_time());
```

### Custom Profiling

```rust
use std::time::Instant;

let start = Instant::now();
let shaped = pipeline.shape_text("Performance test", &font)?;
let shaping_time = start.elapsed();

let start = Instant::now();
let rendered = pipeline.render_shaped(&shaped, &options)?;
let render_time = start.elapsed();

log::info!("Shaping: {:?}, Render: {:?}", shaping_time, render_time);
```

## Memory Optimization

### Font Memory Management

```rust
// Use memory-mapped fonts for large files
let font = Font::from_memory_mapped_file("LargeFont.ttf")?;

// Font will be unmapped when dropped
drop(font);

// Cache management
let cache = pipeline.font_cache();
cache.set_max_size(50 * 1024 * 1024); // 50MB limit
cache.clear_expired(); // Remove unused fonts
```

### Efficient Text Processing

```rust
// Process text in chunks for large documents
fn render_large_document(text: &str, pipeline: &Pipeline) -> Result<Vec<Bitmap>> {
    let chunk_size = 1000; // characters per chunk
    let mut results = Vec::new();
    
    for chunk in text.as_bytes().chunks(chunk_size) {
        let chunk_str = std::str::from_utf8(chunk)?;
        let result = pipeline.render_text(chunk_str, &font, &options)?;
        results.push(result);
    }
    
    Ok(results)
}
```

### Buffer Pool Pattern

```rust
struct BufferPool {
    available: Vec<BitmapBuffer>,
    created: usize,
}

impl BufferPool {
    fn get_buffer(&mut self, width: u32, height: u32) -> BitmapBuffer {
        self.available.pop()
            .map(|mut buf| {
                buf.resize(width, height);
                buf
            })
            .unwrap_or_else(|| {
                self.created += 1;
                BitmapBuffer::new(width, height)
            })
    }
    
    fn return_buffer(&mut self, buffer: BitmapBuffer) {
        if self.available.len() < 10 { // Keep max 10 buffers
            self.available.push(buffer);
        }
    }
}
```

## Shaping Performance

### Shaper Selection

```rust
// Fastest to slowest shapers
fn choose_shaper(text: &str) -> ShaperBackend {
    if text.is_ascii() && !text.contains_whitespace() {
        ShaperBackend::None // Fastest for simple text
    } else if is_latin_text(text) {
        ShaperBackend::HarfBuzz // Good balance for Latin
    } else if text.chars().any(|c| c.is_arabic()) {
        ShaperBackend::ICUHarfBuzz // Best for complex scripts
    } else {
        ShaperBackend::HarfBuzz // Default choice
    }
}
```

### Shaping Caching

```rust
// Cache shaped results
let mut shaping_cache = LruCache::new(1000);

let cache_key = (text.to_string(), font_hash, font_size);
if let Some(cached) = shaping_cache.get(&cache_key) {
    return cached.clone();
}

let shaped = pipeline.shape_text(text, &font)?;
shaping_cache.put(cache_key, shaped.clone());
```

### Text Segmentation

```rust
// Break large text into paragraphs for better parallelization
fn render_paragraphs(text: &str, pipeline: &Pipeline) -> Result<Vec<Bitmap>> {
    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    let mut results = Vec::new();
    
    for paragraph in paragraphs {
        if !paragraph.trim().is_empty() {
            let result = pipeline.render_text(paragraph, &font, &options)?;
            results.push(result);
        }
    }
    
    Ok(results)
}
```

## Rendering Performance

### Renderer Optimization

```rust
// Choose renderer based on use case
fn choose_renderer(output_format: OutputFormat) -> RendererBackend {
    match output_format {
        OutputFormat::PNG => RendererBackend::Skia, // Best quality
        OutputFormat::SVG => RendererBackend::Zeno, // Vector output
        OutputFormat::PDF => RendererBackend::Skia, // Print quality
        OutputFormat::Debug => RendererBackend::Opixa, // Fastest
    }
}
```

### SIMD Optimization

```rust
// Enable SIMD when available (auto-detected)
let pipeline = PipelineBuilder::new()
    .enable_simd(true) // Default: enabled if CPU supports it
    .build()?;

// Manual SIMD hinting
#[cfg(target_arch = "x86_64")]
if is_x86_feature_detected!("avx2") {
    // AVX2 optimized path will be used
}
```

### Parallel Rendering

```rust
use rayon::prelude::*;

// Render multiple texts in parallel
fn batch_render(texts: &[String], pipeline: &Pipeline) -> Result<Vec<Bitmap>> {
    texts.par_iter()
        .map(|text| pipeline.render_text(text, &font, &options))
        .collect()
}
```

### GPU Acceleration

```rust
// Use GPU backends when available (Skia GPU)
#[cfg(feature = "render-skia-gpu")]
let pipeline = PipelineBuilder::new()
    .renderer(RendererBackend::SkiaGPU)
    .build()?;

// GPU context management
let gpu_context = pipeline.gpu_context()?;
gpu_context.set_max_texture_size(4096);
```

## Caching Strategies

### Multi-level Caching

```rust
// Shaping cache
let shaping_cache = Arc::new(RwLock::new(LruCache::new(1000)));

// Glyph cache (for rasterizers)
let glyph_cache = Arc::new(RwLock::new(LruCache::new(10000)));

// Rendered image cache
let image_cache = Arc::new(RwLock::new(LruCache::new(100)));
```

### Cache Invalidation

```rust
// Smart cache keys that include relevant options
fn cache_key(text: &str, font: &Font, options: &RenderOptions) -> CacheKey {
    CacheKey {
        text_hash: hash_string(text),
        font_hash: font.hash(),
        options_hash: hash_options(options),
    }
}

// Time-based expiration
let mut cache = LruCache::new(1000);
cache.set_ttl(Duration::from_secs(3600)); // 1 hour
```

### Disk Caching

```rust
// Persist shaped results to disk
use std::collections::HashMap;

struct DiskCache {
    cache_dir: PathBuf,
}

impl DiskCache {
    fn get_cached(&self, key: &str) -> Option<Vec<u8>> {
        let file_path = self.cache_dir.join(format!("{}.cache", key));
        std::fs::read(file_path).ok()
    }
    
    fn store_cached(&self, key: &str, data: &[u8]) -> Result<()> {
        let file_path = self.cache_dir.join(format!("{}.cache", key));
        std::fs::write(file_path, data)?;
        Ok(())
    }
}
```

## Platform-Specific Optimizations

### Windows

```rust
#[cfg(target_os = "windows")]
let pipeline = PipelineBuilder::new()
    .shaper(ShaperBackend::DirectWrite)
    .renderer(RendererBackend::DirectWrite)
    .enable_directwrite_caching(true)
    .build()?;
```

### macOS

```rust
#[cfg(target_os = "macos")]
let pipeline = PipelineBuilder::new()
    .shaper(ShaperBackend::CoreText)
    .renderer(RendererBackend::CoreGraphics)
    .enable_coretext_optimization(true)
    .build()?;
```

### Linux

```rust
#[cfg(target_os = "linux")]
let pipeline = PipelineBuilder::new()
    .shaper(ShaperBackend::HarfBuzz)
    .renderer(RendererBackend::Skia)
    .enable_fontconfig(true)
    .build()?;
```

## Network & Storage

### Efficient Font Loading

```rust
// Stream fonts from network
async fn load_font_from_url(url: &str) -> Result<Font> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    Font::from_bytes(&bytes)
}

// Load fonts lazily
struct LazyFont {
    url: String,
    font: Option<Font>,
}

impl LazyFont {
    async fn get_font(&mut self) -> Result<&Font> {
        if self.font.is_none() {
            self.font = Some(load_font_from_url(&self.url).await?);
        }
        Ok(self.font.as_ref().unwrap())
    }
}
```

### Batch Processing

```rust
// Process multiple jobs efficiently
async fn process_batch(jobs: Vec<RenderJob>) -> Result<Vec<RenderResult>> {
    // Preload all fonts once
    let mut fonts = HashMap::new();
    for job in &jobs {
        if !fonts.contains_key(&job.font_path) {
            fonts.insert(job.font_path.clone(), load_font(&job.font_path).await?);
        }
    }
    
    // Process jobs in parallel
    let results: Vec<_> = futures::future::join_all(
        jobs.into_iter().map(|job| async move {
            render_job(job, &fonts).await
        })
    ).await;
    
    Ok(results.into_iter().collect::<Result<Vec<_>>>()?)
}
```

## Performance Monitoring

### Real-time Metrics

```rust
struct PerformanceMonitor {
    render_times: VecDeque<Duration>,
    memory_usage: VecDeque<usize>,
    cache_hit_rates: VecDeque<f64>,
}

impl PerformanceMonitor {
    fn record_render(&mut self, duration: Duration) {
        self.render_times.push_back(duration);
        if self.render_times.len() > 1000 {
            self.render_times.pop_front();
        }
    }
    
    fn average_render_time(&self) -> Duration {
        let total: Duration = self.render_times.iter().sum();
        total / self.render_times.len() as u32
    }
    
    fn is_performance_degraded(&self) -> bool {
        let avg = self.average_render_time();
        let recent = self.render_times.back().unwrap();
        *recent > avg * 2 // 2x slower than average
    }
}
```

### Debug Tools

```rust
// Performance debugging utilities
pub fn debug_pipeline_performance(pipeline: &Pipeline) {
    println!("=== Pipeline Performance ===");
    println!("Font cache size: {} bytes", pipeline.font_cache_size());
    println!("Shaping cache hits: {}", pipeline.shaping_cache_hits());
    println!("Render cache hits: {}", pipeline.render_cache_hits());
    println!("Memory usage: {} bytes", pipeline.memory_usage());
    
    // Backend-specific info
    if let Some(harfbuzz) = pipeline.harfbuzz_backend() {
        println!("HarfBuzz cache size: {}", harfbuzz.cache_size());
    }
}
```

## Optimization Checklist

### Before Optimization
- [ ] Profile current performance
- [ ] Identify bottlenecks (shaping vs rendering)
- [ ] Measure memory usage
- [ ] Check cache hit rates

### Quick Optimizations
- [ ] Enable appropriate platform backends
- [ ] Configure font caching
- [ ] Use output-specific renderers
- [ ] Preload frequently used fonts

### Advanced Optimizations
- [ ] Implement custom caching strategies
- [ ] Use parallel processing for batches
- [ ] Enable GPU acceleration when available
- [ ] Optimize memory allocation patterns

### Monitoring
- [ ] Set up performance metrics
- [ ] Monitor memory usage over time
- [ ] Track cache effectiveness
- [ ] Alert on performance regression

## Comprehensive Performance Recommendations

### Backend Selection Guide

Based on benchmark analysis, here are data-driven recommendations for different use cases:

#### Production Systems

```rust
// High-performance web service (cross-platform)
fn production_web_pipeline() -> Pipeline {
    PipelineBuilder::new()
        .shaper(ShaperBackend::HarfBuzz)     // 20,630 ops/sec
        .renderer(RendererBackend::Opixa)   // 2,584 ops/sec
        .enable_font_cache(true)
        .cache_size(500 * 1024 * 1024)      // 500MB
        .enable_simd(true)
        .build()
        .expect("Production pipeline must initialize")
}

// macOS native application (optimal performance)
#[cfg(target_os = "macos")]
fn macos_native_pipeline() -> Pipeline {
    PipelineBuilder::new()
        .shaper(ShaperBackend::CoreText)     // 22,799 ops/sec
        .renderer(RendererBackend::CoreGraphics)  // 5,045 ops/sec
        .enable_coretext_optimization(true)
        .build()
        .expect("macOS pipeline must initialize")
}

// High-quality print production
fn print_production_pipeline() -> Pipeline {
    PipelineBuilder::new()
        .shaper(ShaperBackend::HarfBuzz)
        .renderer(RendererBackend::Skia)    // Color font support
        .enable_high_quality(true)
        .build()
        .expect("Print pipeline must initialize")
}
```

#### Font Format Compatibility Matrix

| Font Type | Recommended Backend | Performance | Notes |
|-----------|-------------------|-------------|-------|
| Standard TTF/OTF | HarfBuzz + Opixa | 2,584 ops/sec | Best balance |
| Color Fonts (COLR) | HarfBuzz + Skia | 913 ops/sec | Full color support |
| SVG Fonts | HarfBuzz + Zeno | 785 ops/sec | Best vector output |
| CBDT Bitmap | HarfBuzz + Opixa | ~1,000 ops/sec | Limited support |
| Emoji Fonts | HarfBuzz + Skia | 913 ops/sec | Color essential |

### Performance Budget Guidelines

| Operation | Target | Maximum |
|-----------|--------|---------|
| Simple shaping (Latin) | 50µs | 100µs |
| Complex shaping (Arabic) | 100µs | 200µs |
| Rasterization (16px) | 500µs | 1ms |
| Color font rendering | 1ms | 2ms |
| Total pipeline | 2ms | 5ms |

**Note**: These targets are based on the benchmark analysis showing optimal backend combinations achieving 2,500-4,500 ops/sec.

### Advanced Optimization Strategies

#### Multi-level Caching Strategy

```rust
pub struct AdvancedCacheManager {
    l1_cache: LruCache<TextHash, ShapedText>,   // Memory L1
    l2_cache: DiskCache<TextHash, ShapedText>,  // SSD L2
    font_metrics: FontMetricsCache,
    glyph_cache: GlyphRasterCache,
}

impl AdvancedCacheManager {
    pub fn get_or_shape<F>(&mut self, text: &str, font: &Font, shaper: F) -> Result<ShapedText>
    where F: FnOnce(&str, &Font) -> Result<ShapedText>
    {
        let hash = hash_text_and_font(text, font);
        
        // L1 cache (fastest)
        if let Some(cached) = self.l1_cache.get(&hash) {
            return Ok(cached.clone());
        }
        
        // L2 cache (medium speed)
        if let Some(cached) = self.l2_cache.get(&hash)? {
            self.l1_cache.put(hash, cached.clone());
            return Ok(cached);
        }
        
        // Shape and cache
        let shaped = shaper(text, font)?;
        self.l1_cache.put(hash, shaped.clone());
        self.l2_cache.put(hash, shaped.clone())?;
        
        Ok(shaped)
    }
}
```

#### Intelligent Backend Selection

```rust
pub struct AdaptivePipeline {
    performance_monitor: PerformanceMonitor,
    backend_pool: BackendPool,
}

impl AdaptivePipeline {
    pub fn render_optimal(&mut self, text: &str, font: &Font, options: &RenderOptions) -> Result<Bitmap> {
        let backend_combo = self.choose_optimal_backend(text, font, options);
        
        match self.backend_pool.render(&backend_combo, text, font, options) {
            Ok(result) => {
                self.performance_monitor.record_success(&backend_combo);
                Ok(result)
            }
            Err(e) => {
                self.performance_monitor.record_failure(&backend_combo, &e);
                self.try_fallback_backend(text, font, options)
            }
        }
    }
    
    fn choose_optimal_backend(&self, text: &str, font: &Font, options: &RenderOptions) -> BackendCombo {
        // Consider: text complexity, font type, output format, recent performance
        if is_cbdt_font(font) {
            BackendCombo::HarfBuzzOpixa
        } else if options.requires_color {
            BackendCombo::HarfBuzzSkia
        } else if cfg!(target_os = "macos") {
            BackendCombo::CoreTextCoreGraphics
        } else {
            BackendCombo::HarfBuzzOpixa
        }
    }
}
```

### Performance Regression Detection

```rust
// Real-time performance monitoring
pub struct ProductionMonitor {
    metrics_collector: MetricsCollector,
    alert_thresholds: AlertThresholds,
    regression_detector: RegressionDetector,
}

#[derive(Debug)]
pub struct AlertThresholds {
    pub max_render_time_ms: f64,      // Alert if slower than this
    pub min_success_rate: f64,        // Alert if success rate below this
    pub max_memory_growth_mb: u64,    // Alert if memory grows too fast
    pub performance_regression_pct: f64, // Alert if performance drops
}

impl ProductionMonitor {
    pub fn check_performance(&self, current_metrics: &PerformanceMetrics) -> Vec<Alert> {
        let mut alerts = Vec::new();
        
        // Check render time against benchmark expectations
        let expected_time = self.get_expected_time(&current_metrics.backend_combo);
        if current_metrics.render_time.as_millis() as f64 > expected_time * 2.0 {
            alerts.push(Alert::PerformanceRegression {
                current: current_metrics.render_time,
                expected: Duration::from_millis(expected_time as u64),
            });
        }
        
        // Check for specific regressions identified in benchmarks
        if self.is_zeno_regression(current_metrics) {
            alerts.push(Alert::KnownRegression {
                backend: "Zeno".to_string(),
                description: "Zeno backend shows >100% regression for mixed scripts".to_string(),
            });
        }
        
        alerts
    }
}
```

### Optimization Roadmap

#### Short-term (1-2 weeks)
- [ ] Fix Zeno performance regression (mixed scripts +100% slower)
- [ ] Implement CBDT font detection and graceful handling
- [ ] Add performance regression alerts to monitoring

#### Medium-term (1-2 months)
- [ ] Implement multi-level caching (memory + disk)
- [ ] Add automatic backend selection based on content
- [ ] Optimize HarfBuzz + Skia combination for color fonts

#### Long-term (3-6 months)
- [ ] Develop GPU-accelerated renderers for batch processing
- [ ] Implement predictive caching based on usage patterns
- [ ] Add CBDT bitmap rendering support to all backends

---

Performance optimization starts with measurement. The benchmark analysis provides clear guidance: use HarfBuzz + CoreGraphics on macOS, HarfBuzz + Opixa cross-platform, and avoid CBDT fonts in production. Profile first, optimize the bottlenecks, implement smart caching, and monitor continuously to maintain speed.
