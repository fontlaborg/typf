---
title: Performance Fundamentals
icon: lucide/zap
tags:
  - Performance
  - Optimization
  - Speed
---

# Performance Fundamentals

Fast text rendering requires smart optimization. Here are the core principles.

## Measuring What Matters

Track the metrics that impact users.

```rust
pub struct PerformanceMetrics {
    pub shaping_time: Duration,
    pub rendering_time: Duration,
    pub export_time: Duration,
    pub total_time: Duration,
    pub memory_usage: usize,
    pub cache_hit_rate: f64,
}

impl PerformanceMetrics {
    pub fn start_measurement() -> Self {
        Self {
            shaping_time: Duration::ZERO,
            rendering_time: Duration::ZERO,
            export_time: Duration::ZERO,
            total_time: Instant::now().elapsed(),
            memory_usage: get_memory_usage(),
            cache_hit_rate: 0.0,
        }
    }
}
```

Focus on:
- **Latency** - Time from text to pixels
- **Throughput** - Characters per second
- **Memory** - Peak and average usage
- **Cache efficiency** - Hit rates across all caches

## Shaping Optimization

Text shaping often dominates performance.

### Cache Shaping Results

```rust
pub struct ShapingOptimizer {
    cache: Arc<DashMap<ShapingKey, ShapingResult>>,
    segment_cache: LruCache<TextSegment, ShapedSegment>,
}

impl ShapingOptimizer {
    pub fn shape_optimized(&mut self, text: &ProcessedText, font: &Font, options: &ShapeOptions) -> Result<ShapingResult> {
        // Check cache first
        let cache_key = ShapingKey::new(text, font.id, options);
        
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        // Split into cacheable segments
        let segments = self.split_cacheable_segments(text);
        let mut results = Vec::new();
        
        for segment in segments {
            if let Some(cached) = self.segment_cache.get(&segment) {
                results.push(cached.clone());
            } else {
                let shaped = self.shape_segment(&segment, font, options)?;
                self.segment_cache.put(segment.clone(), shaped.clone());
                results.push(shaped);
            }
        }
        
        let final_result = self.merge_results(results);
        self.cache.insert(cache_key, final_result.clone());
        
        Ok(final_result)
    }
}
```

### Incremental Shaping

```rust
impl ShapingOptimizer {
    pub fn shape_incremental(&mut self, text: &ProcessedText, previous: Option<&ShapingResult>) -> Result<ShapingResult> {
        match previous {
            Some(prev) if self.can_reuse(text, prev) => {
                // Only.reshape the changed portion
                let changed_range = self.find_changed_range(text, prev);
                let reshaped = self.shape_range(text, changed_range)?;
                Ok(self.merge_with_previous(prev, reshaped, changed_range))
            }
            _ => {
                // Full reshape when we can't reuse
                self.shape_optimized(text, font, options)
            }
        }
    }
}
```

## Rendering Optimization

Rasterization is the next bottleneck.

### SIMD Acceleration

```rust
#[cfg(target_arch = "x86_64")]
pub struct SimdRenderer {
    rasterizer: OrgeRasterizer,
    // SSE4.1, AVX2 support
}

impl SimdRenderer {
    pub fn render_scanline_simd(&self, pixels: &mut [u8], coverage: &[f32]) {
        if is_x86_feature_detected!("avx2") {
            self.render_scanline_avx2(pixels, coverage);
        } else if is_x86_feature_detected!("sse4.1") {
            self.render_scanline_sse41(pixels, coverage);
        } else {
            self.render_scanline_scalar(pixels, coverage);
        }
    }
    
    #[target_feature(enable = "avx2")]
    unsafe fn render_scanline_avx2(&self, pixels: &mut [u8], coverage: &[f32]) {
        // Process 8 pixels at once
        for (chunk, cov_chunk) in pixels.chunks_exact_mut(8).zip(coverage.chunks_exact(8)) {
            let coverage_vec = _mm256_load_ps(cov_chunk.as_ptr());
            let pixels_vec = _mm256_loadu_ps(pixels.as_ptr() as *const f32);
            let result = _mm256_add_ps(pixels_vec, coverage_vec);
            _mm256_storeu_ps(pixels.as_mut_ptr() as *mut f32, result);
        }
    }
}
```

### Parallel Rendering

```rust
pub struct ParallelRenderer {
    thread_pool: ThreadPool,
    max_threads: usize,
}

impl ParallelRenderer {
    pub fn render_parallel(&self, glyphs: &[Glyph], options: &RenderOptions) -> Result<RenderOutput> {
        let chunks = self.split_into_chunks(glyphs);
        let mut handles = Vec::new();
        
        for chunk in chunks {
            let handle = self.thread_pool.spawn(move || {
                self.render_chunk(chunk, options)
            });
            handles.push(handle);
        }
        
        let results: Result<Vec<_>> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();
        
        self.merge_render_results(results?)
    }
    
    fn split_into_chunks(&self, glyphs: &[Glyph]) -> Vec<&[Glyph]> {
        let chunk_size = (glyphs.len() + self.max_threads - 1) / self.max_threads;
        glyphs.chunks(chunk_size).collect()
    }
}
```

## Memory Optimization

Memory access patterns matter more than allocation.

### Cache-Friendly Data Layout

```rust
// BAD - Random memory access
pub struct GlyphDataBad {
    glyph_ids: Vec<GlyphId>,
    positions: Vec<Position>,
    advances: Vec<f32>,
}

// GOOD - Contiguous memory access
pub struct GlyphDataGood {
    glyphs: Vec<GlyphInfo>, // All data for one glyph together
}

#[repr(C)]
pub struct GlyphInfo {
    id: GlyphId,
    x: f32,
    y: f32,
    advance: f32,
}

impl GlyphDataGood {
    pub fn render(&self, renderer: &mut Renderer) {
        // Sequential memory access
        for glyph in &self.glyphs {
            renderer.render_glyph(glyph);
        }
    }
}
```

### Memory Pooling

```rust
pub struct MemoryPool {
    bitmap_pool: Vec<BitmapData>,
    vector_pool: Vec<VectorData>,
    temp_buffer: Vec<u8>,
}

impl MemoryPool {
    pub fn get_bitmap(&mut self, width: u32, height: u32) -> BitmapData {
        self.bitmap_pool.pop()
            .filter(|b| b.width >= width && b.height >= height)
            .unwrap_or_else(|| BitmapData::new(width, height))
    }
    
    pub fn return_bitmap(&mut self, bitmap: BitmapData) {
        if self.bitmap_pool.len() < 50 { // Limit pool size
            bitmap.clear(); // Reset to zero state
            self.bitmap_pool.push(bitmap);
        }
    }
}
```

## Backend Performance

Different backends have different performance characteristics.

### HarfBuzz Optimization

```rust
pub struct OptimizedHarfBuzz {
    font_cache: HashMap<FontId, HarfBuzzFont>,
    buffer_pool: Vec<harfbuzz_rs::Buffer>,
}

impl OptimizedHarfBuzz {
    pub fn shape_with_pool(&mut self, text: &str, font: &Font, options: &ShapeOptions) -> Result<ShapingResult> {
        // Reuse buffers to avoid allocation
        let mut buffer = self.buffer_pool.pop()
            .unwrap_or_else(|| harfbuzz_rs::Buffer::new());
        
        buffer.set_text(text, script, direction);
        
        // Apply features efficiently
        buffer.guess_segment_properties();
        
        // Shape
        buffer.shape(font.harfbuzz_font(), &features);
        
        let result = self.extract_result(&buffer);
        
        // Return buffer to pool
        buffer.clear();
        self.buffer_pool.push(buffer);
        
        Ok(result)
    }
}
```

### Skia Optimization

```rust
pub struct OptimizedSkia {
    surface: Surface,
    canvas: Canvas,
    paint_pool: Vec<Paint>,
}

impl OptimizedSkia {
    pub fn render_batch(&mut self, glyphs: &[Glyph]) -> Result<RenderOutput> {
        let mut paint = self.paint_pool.pop()
            .unwrap_or_else(|| Paint::default());
        
        // Batch render glyphs with same properties
        let mut current_run = Vec::new();
        
        for glyph in glyphs {
            if current_run.len() == 0 || self.can_batch_with(&current_run[0], glyph) {
                current_run.push(glyph);
            } else {
                self.render_run(&current_run, &mut paint);
                current_run.clear();
                current_run.push(glyph);
            }
        }
        
        if !current_run.is_empty() {
            self.render_run(&current_run, &mut paint);
        }
        
        // Return paint to pool
        self.paint_pool.push(paint);
        
        Ok(self.capture_surface())
    }
}
```

## Performance Profiling

Find bottlenecks before optimizing.

```rust
#[cfg(debug_assertions)]
pub struct Profiler {
    timings: HashMap<String, Vec<Duration>>,
    counters: HashMap<String, usize>,
}

impl Profiler {
    pub fn time_section<F, R>(&mut self, name: &str, f: F) -> R 
    where F: FnOnce() -> R {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        
        self.timings.entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
        
        result
    }
    
    pub fn report(&self) -> String {
        let mut report = String::new();
        
        for (name, timings) in &self.timings {
            let avg = timings.iter().sum::<Duration>() / timings.len() as u32;
            let max = timings.iter().max().unwrap();
            let min = timings.iter().min().unwrap();
            
            report.push_str(&format!(
                "{}: avg={:?}, min={:?}, max={:?}, count={}\n",
                name, avg, min, max, timings.len()
            ));
        }
        
        report
    }
}
```

## Performance Targets

Know what "fast" means for your use case.

| Use Case | Target | Reason |
|----------|--------|--------|
| UI text | <1ms | User interaction latency |
| Document rendering | <100ms per page | Acceptable wait time |
| Batch processing | >1000 chars/sec | Throughput focus |
| Embedded systems | <10MB memory | Memory constraints |

## Common Performance Pitfalls

### Don't Optimize Too Early

```rust
// BAD - Premature optimization
pub fn render_glyph_cached(glyph: Glyph) {
    // Complex caching for rarely used function
    if !self.cache.contains(&glyph) {
        let result = expensive_render(glyph);
        self.cache.insert(glyph, result);
    }
}

// GOOD - Profile first
pub fn render_glyph_simple(glyph: Glyph) {
    // Simple, clear implementation
    expensive_render(glyph)
}
```

### Don't Over-Parallelize

```rust
// BAD - Too much overhead
let results: Vec<_> = glyphs.iter()
    .map(|g| thread::spawn(|| render_one(g))) // One thread per glyph!
    .collect();

// GOOD - Reasonable chunking
let chunks = glyphs.chunks(100).collect::<Vec<_>>();
let results: Vec<_> = chunks.iter()
    .map(|chunk| thread::spawn(|| render_batch(chunk)))
    .collect();
```

### Don't Cache Everything

```rust
// BAD - Infinite cache growth
cache.put(very_large_image);
cache.put(another_large_image);

// GOOD - Size limits
if cache.size() < MAX_CACHE_SIZE {
    cache.put(image);
} else {
    cache.evict_lru();
    cache.put(image);
}
```

## Performance Checklist

Before shipping performance-critical code:

- [ ] Profile with realistic data
- [ ] Measure before and after optimization
- [ ] Check memory usage trends
- [ ] Test on target hardware
- [ ] Verify cache hit rates
- [ ] Benchmark edge cases
- [ ] Monitor for regressions

---

Fast text rendering comes from smart caching, parallel processing, and memory-efficient data structures. Profile first, optimize second.