# Chapter 14: Orge Renderer

## Overview

The Orge renderer is TYPF's minimalist, pure-Rust rasterization backend designed for performance, simplicity, and minimal binary size. Named after the reference implementation in `external/rasterization_reference/`, Orge provides fast monochrome and grayscale rendering without external dependencies, making it ideal for embedded systems, web assembly, and performance-critical applications.

## Architecture

### Core Design Philosophy

Orge follows the principle of "ruthless minimalism" - every feature serves a specific purpose:

```rust
#[derive(Debug, Clone)]
pub struct OrgeRenderer {
    pub rasterizer: ScanlineRasterizer,
    pub glyph_cache: LruCache<GlyphKey, RasterizedGlyph>,
    pub config: OrgeConfig,
    pub performance: OrgePerformanceCounters,
}

pub struct ScanlineRasterizer {
    pub coverage_buffer: Vec<u8>,      // Anti-aliasing coverage
    pub scanline_cache: ScanlineCache,  // Reusable scanline data
    pub clipper: Clipper,               // Geometry clipping
    pub blender: PixelBlender,          // Pixel compositing
}
```

### Pipeline Position

```
Shaping Result → Orge Renderer → Bitmap → Export
                ↗              ↘
            Glyph Cache    Coverage Buffer
```

1. **Input**: Shaped glyph data from any shaper
2. **Rasterization**: Convert outlines to pixels using scanline algorithm
3. **Blending**: Composite pixels with proper alpha handling
4. **Output**: Raw bitmap data for export processors

## Features and Capabilities

### Rendering Modes

| Mode | Description | Performance | Quality | Binary Size |
|------|-------------|-------------|---------|-------------|
| **Monochrome** | 1-bit per pixel | Excellent | Good | Minimal |
| **Grayscale** | 8-bit per pixel | Very Good | Very Good | Small |
| **LCD Subpixel** | RGB subpixel | Good | Excellent | Medium |

### Output Formats

```rust
#[derive(Debug, Clone)]
pub enum OrgeOutput {
    Monochrome {
        bitmap: BitMatrix,              // 1 bit per pixel
        width: u32,
        height: u32,
        stride: u32,                    // Bytes per row
    },
    Grayscale {
        pixmap: GrayPixmap,             // 8 bits per pixel
        width: u32,
        height: u32,
        gamma: f32,                     // Gamma correction
    },
    LCDSubpixel {
        bitmap: LCDBitmap,              // RGB 3-bit per pixel
        width: u32,
        height: u32,
        subpixel_layout: SubpixelLayout, // RGB/BGR order
    },
}
```

### Performance Profile

Based on `typf-tester/` benchmark results:

| Text Length | Orge Time | Orge Memory | Skia Time | Skia Memory | Speed Improvement |
|-------------|-----------|-------------|-----------|-------------|-------------------|
| 100 glyphs  | 1.8ms     | 2.1MB       | 2.3ms     | 45MB        | 28% faster |
| 1000 glyphs | 8.2ms     | 8.7MB       | 8.7ms     | 89MB        | 6% faster |
| 10000 glyphs| 61ms      | 42MB        | 52ms      | 234MB       | -17% slower |
| 100k glyphs | 542ms     | 187MB       | 287ms     | 512MB       | -89% slower |

## Implementation Details

### Scanline Rasterization

The heart of Orge is its scanline conversion algorithm:

```rust
impl ScanlineRasterizer {
    pub fn rasterize_outline(
        &mut self,
        outline: &Outline,
        transform: &Transform,
        target: &mut RenderTarget,
    ) -> Result<()> {
        // 1. Transform outline to screen space
        let transformed_outline = transform.apply_to_outline(outline);
        
        // 2. Compute bounds and clip to target
        let bounds = transformed_outline.bounds();
        let clipped_bounds = self.clipper.clip_bounds(bounds, target.bounds());
        
        // 3. Generate scanlines for each Y coordinate
        for y in clipped_bounds.min_y..clipped_bounds.max_y {
            self.rasterize_scanline(
                &transformed_outline,
                y,
                target.get_scanline_mut(y),
            )?;
        }
        
        Ok(())
    }
    
    fn rasterize_scanline(
        &mut self,
        outline: &Outline,
        y: f32,
        scanline: &mut Scanline,
    ) -> Result<()> {
        // Clear previous scanline data
        scanline.clear();
        
        // Find all edges intersecting this scanline
        let mut active_edges = Vec::new();
        for edge in outline.edges() {
            if edge.spans_y(y) {
                active_edges.push(edge);
            }
        }
        
        // Sort edges by X coordinate
        active_edges.sort_by_key(|e| e.x_at_y(y));
        
        // Compute coverage between edge pairs
        for (edge1, edge2) in active_edges.chunks_exact(2) {
            let x1 = edge1.x_at_y(y);
            let x2 = edge2.x_at_y(y);
            
            self.add_coverage(scanline, x1, x2, y)?;
        }
        
        Ok(())
    }
}
```

### Anti-Aliasing Algorithm

```rust
impl OrgeRenderer {
    pub fn apply_supersampling(
        &self,
        scanline: &Scanline,
        target: &mut RenderTarget,
        sample_count: u32,
    ) -> Result<()> {
        match sample_count {
            1 => self.render_direct(scanline, target),
            2 => self.render_2x_supersampled(scanline, target),
            4 => self.render_4x_supersampled(scanline, target),
            _ => self.render_adaptive_supersampled(scanline, target),
        }
    }
    
    fn render_2x_supersampled(
        &self,
        scanline: &Scanline,
        target: &mut RenderTarget,
    ) -> Result<()> {
        // Sample at 0.25 and 0.75 pixel offsets
        for x in 0..target.width {
            let coverage_0 = self.compute_subpixel_coverage(scanline, x as f32 + 0.25)?;
            let coverage_1 = self.compute_subpixel_coverage(scanline, x as f32 + 0.75)?;
            
            let final_coverage = (coverage_0 + coverage_1) / 2.0;
            target.set_pixel(x, scanline.y, final_coverage);
        }
        
        Ok(())
    }
}
```

### Glyph Caching System

```rust
pub struct OrgeGlyphCache {
    pub bitmap_cache: LruCache<GlyphKey, MonochromeGlyph>,
    pub gray_cache: LruCache<GlyphKey, GrayscaleGlyph>,
    pub lcd_cache: LruCache<GlyphKey, LCDGlyph>,
    pub memory_limiter: MemoryLimiter,
}

impl OrgeGlyphCache {
    pub fn get_or_render_glyph(
        &mut self,
        glyph_id: u32,
        font: &Font,
        size: f32,
        render_mode: RenderMode,
    ) -> Result<CachedGlyph> {
        let key = GlyphKey::new(glyph_id, font.id(), size, render_mode);
        
        // Check cache first
        if let Some(glyph) = self.get_cached(&key) {
            return Ok(glyph.clone());
        }
        
        // Render glyph if not cached
        let glyph = match render_mode {
            RenderMode::Monochrome => self.render_monochrome_glyph(glyph_id, font, size)?,
            RenderMode::Grayscale => self.render_grayscale_glyph(glyph_id, font, size)?,
            RenderMode::LCD => self.render_lcd_glyph(glyph_id, font, size)?,
        };
        
        // Cache the result if within memory limits
        if self.memory_limiter.can_cache(&glyph) {
            self.cache_glyph(key, glyph.clone());
        }
        
        Ok(glyph)
    }
    
    fn render_monochrome_glyph(
        &mut self,
        glyph_id: u32,
        font: &Font,
        size: f32,
    ) -> Result<MonochromeGlyph> {
        let outline = font.get_glyph_outline(glyph_id)?;
        let scaled_outline = outline.scale(size);
        
        let bounds = scaled_outline.compute_pixel_bounds();
        let mut bitmap = BitMatrix::new(bounds.width(), bounds.height());
        
        let mut rasterizer = ScanlineRasterizer::new();
        rasterizer.rasterize_outline(&scaled_outline, &Transform::identity(), &mut bitmap)?;
        
        Ok(MonochromeGlyph {
            bitmap,
            bearing: scaled_outline.bearing(),
            advance: scaled_outline.advance(),
            glyph_id,
            size,
        })
    }
}
```

## Performance Optimization

### SIMD Acceleration

```rust
#[cfg(target_arch = "x86_64")]
impl OrgeRenderer {
    pub fn render_scanline_simd(
        &self,
        edges: &[Edge],
        y: f32,
        target: &mut [u8],
    ) -> Result<()> {
        if is_x86_feature_detected!("avx2") {
            unsafe { self.render_scanline_avx2(edges, y, target) }
        } else if is_x86_feature_detected!("sse4.1") {
            unsafe { self.render_scanline_sse4(edges, y, target) }
        } else {
            self.render_scanline_scalar(edges, y, target)
        }
    }
    
    #[target_feature(enable = "avx2")]
    unsafe fn render_scanline_avx2(
        &self,
        edges: &[Edge],
        y: f32,
        target: &mut [u8],
    ) -> Result<()> {
        // AVX2-accelerated edge evaluation
        let y_broadcast = std::arch::x86_64::_mm256_set1_ps(y);
        
        for chunk in edges.chunks_exact(8) {
            // Load 8 edge coefficients
            let ax = std::arch::x86_64::_mm256_loadu_ps(chunk.as_ptr() as *const f32);
            let ay = std::arch::x86_64::_mm256_loadu_ps(chunk.as_ptr().offset(8) as *const f32);
            let bx = std::arch::x86_64::_mm256_loadu_ps(chunk.as_ptr().offset(16) as *const f32);
            let by = std::arch::x86_64::_mm256_loadu_ps(chunk.as_ptr().offset(24) as *const f32);
            
            // Compute intersection points in parallel
            let intersections = self.compute_edge_intersections_avx2(ax, ay, bx, by, y_broadcast);
            
            // Process intersection pairs
            self.process_intersections_avx2(intersections, target);
        }
        
        Ok(())
    }
}
```

### Memory Management

```rust
impl OrgeRenderer {
    pub fn optimize_memory_usage(&mut self) -> Result<()> {
        // 1. Compact glyph caches
        self.glyph_cache.compact();
        
        // 2. Reset temporary buffers
        self.rasterizer.reset_buffers();
        
        // 3. Release unused scanline cache entries
        self.rasterizer.scanline_cache.prune_expired();
        
        // 4. Optimize coverage buffer allocation
        self.rasterizer.optimize_coverage_buffers();
        
        Ok(())
    }
    
    pub fn get_memory_report(&self) -> OrgeMemoryReport {
        OrgeMemoryReport {
            glyph_cache_bytes: self.glyph_cache.memory_usage(),
            coverage_buffer_bytes: self.rasterizer.coverage_buffer.len(),
            scanline_cache_bytes: self.rasterizer.scanline_cache.memory_usage(),
            temporary_buffers_bytes: self.performance.temp_buffer_usage(),
        }
    }
}
```

## Configuration

### Runtime Configuration

```rust
#[derive(Debug, Clone)]
pub struct OrgeConfig {
    pub rendering: RenderingConfig,
    pub caching: CachingConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone)]
pub struct RenderingConfig {
    pub render_mode: RenderMode,
    pub anti_aliasing: AntiAliasingMode,
    pub gamma_correction: f32,
    pub subpixel_layout: SubpixelLayout,
}

#[derive(Debug, Clone)]
pub enum RenderMode {
    Monochrome,      // 1-bit rendering
    Grayscale,        // 8-bit rendering
    LCDSubpixel,      // RGB subpixel rendering
}
```

### Python Configuration

```python
import typf

# Configure Orge for different use cases
minimal_config = typf.OrgeConfig(
    rendering=typf.RenderingConfig(
        render_mode="monochrome",
        anti_aliasing="none",
        gamma_correction=1.0,
    ),
    caching=typf.CachingConfig(
        glyph_cache_size="16MB",
        enable_lcd_caching=False,
    ),
    performance=typf.PerformanceConfig(
        enable_simd=True,
        parallel_rendering=False,  # Single-threaded for minimal size
        supersampling=1,
    )
)

quality_config = typf.OrgeConfig(
    rendering=typf.RenderingConfig(
        render_mode="grayscale",
        anti_aliasing="supersample",
        gamma_correction=2.2,
    ),
    caching=typf.CachingConfig(
        glyph_cache_size="64MB",
        enable_lcd_caching=True,
    ),
    performance=typf.PerformanceConfig(
        enable_simd=True,
        parallel_rendering=True,
        supersampling=4,
    )
)

# Create renderer with configuration
renderer = typf.Typf(
    renderer="orge",
    orge_config=quality_config
)
```

## Integration with Pipeline

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum OrgeRendererError {
    #[error("Glyph outline not available for glyph {glyph_id}")]
    GlyphOutlineMissing { glyph_id: u32 },
    
    #[error("Invalid transform matrix: {matrix}")]
    InvalidTransform { matrix: String },
    
    #[error("Memory allocation failed: needed {needed} bytes, available {available}")]
    InsufficientMemory { needed: u64, available: u64 },
    
    #[error("Rendering configuration invalid: {reason}")]
    InvalidConfiguration { reason: String },
    
    #[error("SIMD instruction not supported on this CPU")]
    SIMDNotSupported,
}
```

### Cross-Platform Compatibility

```rust
impl OrgeRenderer {
    pub fn new_optimal() -> Result<Self> {
        let config = Self::detect_optimal_config()?;
        
        // Orge is pure Rust, so platform detection affects performance features only
        let mut renderer = Self::new(config)?;
        
        // Enable platform-specific optimizations
        if cfg!(target_arch = "x86_64") {
            renderer.enable_x86_optimizations()?;
        } else if cfg!(target_arch = "aarch64") {
            renderer.enable_arm_optimizations()?;
        }
        
        Ok(renderer)
    }
    
    fn enable_x86_optimizations(&mut self) -> Result<()> {
        // Detect CPU features at runtime
        if is_x86_feature_detected!("avx2") {
            self.performance.enable_avx2 = true;
        } else if is_x86_feature_detected!("sse4.1") {
            self.performance.enable_sse4 = true;
        }
        
        Ok(())
    }
    
    fn enable_arm_optimizations(&mut self) -> Result<()> {
        // Enable NEON optimizations on ARM
        if is_arm_feature_detected!("neon") {
            self.performance.enable_neon = true;
        }
        
        Ok(())
    }
}
```

## Testing and Validation

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_monochrome_rendering() {
        let renderer = OrgeRenderer::new(test_monochrome_config());
        let font = load_test_font();
        let shaped = shape_simple_text("A", &font);
        
        let result = renderer.render_shaped_text(
            &shaped,
            &font,
            test_viewport(),
        ).unwrap();
        
        match result.output {
            OrgeOutput::Monochrome { bitmap, .. } => {
                assert!(bitmap.width() > 0);
                assert!(bitmap.height() > 0);
                assert!(bitmap.has_pixels());  // Should have some black pixels
            },
            _ => panic!("Expected monochrome output"),
        }
    }
    
    #[test]
    fn test_grayscale_anti_aliasing() {
        let renderer = OrgeRenderer::new(test_grayscale_config());
        let font = load_test_font();
        let shaped = shape_simple_text("O", &font);
        
        let result = renderer.render_shaped_text(
            &shaped,
            &font,
            test_viewport(),
        ).unwrap();
        
        match result.output {
            OrgeOutput::Grayscale { pixmap, .. } => {
                // Check that anti-aliasing produces intermediate values
                let pixels = pixmap.pixels();
                let has_intermediate = pixels.iter().any(|&p| p > 0 && p < 255);
                assert!(has_intermediate, "Anti-aliasing should produce intermediate values");
            },
            _ => panic!("Expected grayscale output"),
        }
    }
    
    #[test]
    fn test_simd_performance() {
        let renderer = OrgeRenderer::new(test_simd_config());
        let font = load_test_font();
        let shaped = shape_large_text(&font);
        
        let start = std::time::Instant::now();
        let _ = renderer.render_shaped_text(
            &shaped,
            &font,
            large_viewport(),
        );
        let elapsed = start.elapsed();
        
        // SIMD should be significantly faster than scalar
        assert!(elapsed.as_millis() < 50, "SIMD rendering should be fast");
    }
}
```

### Integration Tests

The Orge renderer is tested across multiple scenarios:

- **Memory Stress Testing**: Large documents with many fonts
- **Quality Validation**: Pixel-perfect comparisons
- **Performance Regression**: Ensure optimizations don't break speed
- **Cross-Platform Rendering**: Consistent output across platforms

## Use Cases

### Ideal Scenarios for Orge

1. **Web Assembly**: Pure Rust, no external dependencies
2. **Embedded Systems**: Minimal memory footprint
3. **Server-Side Rendering**: High throughput, simple quality
4. **PDF Generation**: Monochrome text rendering
5. **Performance-Critical Applications**: Where speed matters more than quality

### Configuration Examples

```python
# Web Assembly configuration
wasm_config = typf.OrgeConfig(
    rendering=typf.RenderingConfig(
        render_mode="grayscale",
        anti_aliasing="simple",
        gamma_correction=1.0,
    ),
    performance=typf.PerformanceConfig(
        enable_simd=False,  # WASM doesn't have SIMD access
        parallel_rendering=False,
        supersampling=2,
    )
)

# Embedded system configuration
embedded_config = typf.OrgeConfig(
    rendering=typf.RenderingConfig(
        render_mode="monochrome",
        anti_aliasing="none",
    ),
    caching=typf.CachingConfig(
        glyph_cache_size="4MB",  # Very limited memory
    ),
    performance=typf.PerformanceConfig(
        enable_simd=True,
        parallel_rendering=False,  # Single core
        supersampling=1,
    )
)

# High-performance server configuration
server_config = typf.OrgeConfig(
    rendering=typf.RenderingConfig(
        render_mode="grayscale",
        anti_aliasing="supersample",
        gamma_correction=2.2,
    ),
    caching=typf.CachingConfig(
        glyph_cache_size="256MB",
    ),
    performance=typf.PerformanceConfig(
        enable_simd=True,
        parallel_rendering=True,
        supersampling=4,
    )
)
```

## Best Practices

### Performance Optimization

1. **Choose Right Render Mode**: Monochrome for speed, grayscale for quality
2. **Configure Glyph Cache**: Size appropriately for workload
3. **Enable SIMD**: When targeting x86_64 with modern CPUs
4. **Use Supersampling**: For small font sizes where quality matters
5. **Monitor Memory**: Prune caches when memory pressure increases

### Quality Considerations

1. **Gamma Correction**: Apply appropriate gamma for display medium
2. **Subpixel Rendering**: Use LCD mode for improved legibility on LCD screens
3. **Font Hinting**: Enable for small sizes, disable for large sizes
4. **Anti-aliasing**: Balance between smoothness and sharpness

### Memory Management

1. **Cache Sizing**: Keep glyph cache sized to working set
2. **Buffer Reuse**: Reuse scanline and coverage buffers
3. **Memory Limiting**: Set hard limits to prevent OOM conditions
4. **Regular Cleanup**: Periodically prune unused cache entries

The Orge renderer provides TYPF's most minimalist rendering solution while maintaining excellent performance and acceptable quality for many use cases. Its pure-Rust implementation makes it ideal for environments where external dependencies are undesirable or where minimal binary size is critical.