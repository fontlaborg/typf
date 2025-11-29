# Troubleshooting & Best Practices

Solve common TypF problems and avoid pitfalls.

## Common Issues

### Font Loading Problems

**Problem**: "Font file not found" error
```rust
// Wrong: Relative path
let font = Font::from_file("Roboto.ttf")?; 

// Right: Absolute or well-known path
let font = Font::from_file("/usr/share/fonts/Roboto.ttf")?;
// Or use the font database
let font = fontdb.load_font_by_name("Roboto")?;
```

**Problem**: Corrupted font data
```rust
// Validate font before loading
fn safe_font_load(path: &Path) -> Result<Font> {
    let data = std::fs::read(path)?;
    
    // Check font magic bytes
    if data.len() < 12 {
        return Err(TypfError::InvalidFont("File too small"));
    }
    
    let magic = &data[0:4];
    if magic != b"\x00\x01\x00\x00" && magic != b"OTTO" {
        return Err(TypfError::InvalidFont("Invalid font format"));
    }
    
    Font::from_bytes(&data)
}
```

### Text Rendering Issues

**Problem**: Text appears upside down
```rust
// Check coordinate system match
let mut options = RenderOptions::default();
options.transform = Transform::identity();
// Don't set flip_y unless your coordinate system needs it

// For web canvases (Y-down):
options.flip_y = true;

// For PDF/printing (Y-up):
options.flip_y = false;
```

**Problem**: Wrong text size
```rust
// Points vs pixels confusion
let options = RenderOptions {
    // Use pixels for screen rendering
    font_size: 16.0, // pixels
    // Use points for print (72 points = 1 inch)
    font_size: points_to_pixels(12.0), // 12pt = 16px at 96 DPI
    dpi: 96.0,
    ..Default::default()
};
```

**Problem**: Text gets cut off
```rust
// Ensure sufficient bounds
let text_bounds = measure_text(&text, &font, font_size)?;
let options = RenderOptions {
    width: (text_bounds.width + padding) as u32,
    height: (text_bounds.height + padding) as u32,
    ..Default::default()
};
```

### Backend Selection Issues

**Problem**: Backend not available
```rust
// Check availability before using
fn choose_best_renderer() -> RendererBackend {
    if RendererBackend::Skia.is_available() {
        RendererBackend::Skia
    } else if RendererBackend::Opixa.is_available() {
        RendererBackend::Opixa
    } else {
        panic!("No renderer available");
    }
}

// Or handle gracefully
match pipeline.set_renderer(RendererBackend::Skia) {
    Ok(_) => println!("Using Skia renderer"),
    Err(e) => {
        println!("Skia not available, falling back to Opixa: {}", e);
        pipeline.set_renderer(RendererBackend::Opixa)?;
    }
}
```

**Problem**: Performance slower than expected
```rust
// Profile to find the bottleneck
let start = std::time::Instant::now();
let shaped = pipeline.shape_text(text, &font)?;
let shaping_time = start.elapsed();

let start = std::time::Instant::now();
let rendered = pipeline.render_shaped(&shaped, &options)?;
let render_time = start.elapsed();

println!("Shaping: {:?}, Rendering: {:?}", shaping_time, render_time);

// If shaping is slow, try a different shaper
// If rendering is slow, try GPU acceleration
```

## Debugging Tools

### Built-in Diagnostics

```rust
// Enable debug mode
let pipeline = PipelineBuilder::new()
    .debug_mode(true)
    .enable_profiling(true)
    .build()?;

// Get detailed error information
match pipeline.render_text("test", &font, &options) {
    Ok(result) => println!("Success"),
    Err(TypfError::ShapingError { source, details }) => {
        eprintln!("Shaping failed: {}", source);
        eprintln!("Details: {}", details);
        
        // Try with simpler text
        pipeline.render_text("simple", &font, &options)?;
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

### Font Inspector

```rust
fn debug_font(font: &Font) {
    println!("Font Information:");
    println!("  Family: {}", font.family_name());
    println!("  Style: {}", font.style_name());
    println!("  Units per EM: {}", font.units_per_em());
    println!("  Ascender: {}", font.ascender());
    println!("  Descender: {}", font.descender());
    println!("  Line Gap: {}", font.line_gap());
    
    // Check glyph coverage
    let test_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    for c in test_chars.chars() {
        if let Some(glyph_id) = font.glyph_index(c) {
            println!("  '{}' -> glyph {}", c, glyph_id);
        } else {
            println!("  '{}' -> MISSING", c);
        }
    }
}
```

### Text Analysis

```rust
fn analyze_text(text: &str) {
    println!("Text Analysis:");
    println!("  Length: {} characters", text.len());
    println!("  Unicode: {} code points", text.chars().count());
    
    // Script detection
    let scripts: HashSet<_> = text.chars()
        .filter_map(|c| unicode_script::UnicodeScript::script(c))
        .collect();
    
    println!("  Scripts: {:?}", scripts);
    
    // Direction detection
    let has_rtl = text.chars().any(|c| c.is_right_to_left());
    println!("  Direction: {}", if has_rtl { "RTL" } else { "LTR" });
    
    // Complex characters
    let complex_chars: Vec<_> = text.chars()
        .filter(|c| !c.is_ascii() && !c.is_alphanumeric())
        .collect();
    
    if !complex_chars.is_empty() {
        println!("  Complex characters: {:?}", complex_chars);
    }
}
```

## Performance Debugging

### Memory Leaks

```rust
// Track font usage
struct FontTracker {
    loaded_fonts: HashMap<String, Weak<Font>>,
}

impl FontTracker {
    fn load_font(&mut self, path: &str) -> Result<Font> {
        // Clean up stale references first
        self.loaded_fonts.retain(|_, weak| weak.strong_count() > 0);
        
        if let Some(weak_font) = self.loaded_fonts.get(path) {
            if let Some(font) = weak_font.upgrade() {
                return Ok(font);
            }
        }
        
        let font = Font::from_file(path)?;
        self.loaded_fonts.insert(path.to_string(), Arc::downgrade(&font));
        Ok(font)
    }
}
```

### Cache Effectiveness

```rust
fn monitor_cache_performance(cache: &LRUCache<String, Vec<u8>>) {
    let stats = cache.stats();
    println!("Cache Statistics:");
    println!("  Hits: {}", stats.hits());
    println!("  Misses: {}", stats.misses());
    println!("  Hit rate: {:.2}%", stats.hit_rate() * 100.0);
    
    if stats.hit_rate() < 0.8 {
        eprintln!("Warning: Low cache hit rate. Consider increasing cache size.");
    }
}
```

### Render Bottlenecks

```rust
// Benchmark different approaches
fn benchmark_approaches(text: &str, font: &Font) {
    let iterations = 1000;
    
    // Approach 1: Direct render
    let start = Instant::now();
    for _ in 0..iterations {
        pipeline.render_text(text, font, &options)?;
    }
    let direct_time = start.elapsed();
    
    // Approach 2: Shape then render
    let start = Instant::now();
    let shaped = pipeline.shape_text(text, font)?;
    for _ in 0..iterations {
        pipeline.render_shaped(&shaped, &options)?;
    }
    let shaped_time = start.elapsed();
    
    println!("Direct render: {:?}", direct_time / iterations);
    println!("Shape + render: {:?}", shaped_time / iterations);
    
    if shaped_time < direct_time {
        println!("Recommendation: Cache shaped results for repeated renders");
    }
}
```

## Best Practices

### Font Management

```rust
// DO: Use a font database
let fontdb = FontDatabase::new();
fontdb.load_system_fonts()?; // Load once
fontdb.add_font_dir("./custom_fonts")?;

// DON'T: Load fonts repeatedly
for text in texts {
    let font = Font::from_file("Roboto.ttf")?; // Slow!
    pipeline.render_text(text, &font, &options)?;
}

// DO: Reuse font handles
let font = fontdb.load_font_by_name("Roboto")?;
for text in texts {
    pipeline.render_text(text, &font, &options)?;
}
```

### Error Handling

```rust
// DO: Handle specific errors
fn safe_render(pipeline: &Pipeline, text: &str, font: &Font) -> Option<Bitmap> {
    match pipeline.render_text(text, font, &options) {
        Ok(bitmap) => Some(bitmap),
        Err(TypfError::UnsupportedGlyph(c)) => {
            // Replace unsupported characters
            let safe_text = text.replace(c, "�");
            pipeline.render_text(&safe_text, font, &options).ok()
        }
        Err(TypfError::FontNotLoaded) => {
            eprintln!("Font not loaded, using fallback");
            fallback_pipeline.render_text(text, &fallback_font, &options).ok()
        }
        Err(e) => {
            eprintln!("Render failed: {}", e);
            None
        }
    }
}

// DON'T: Use unwrap() in production code
let result = pipeline.render_text(text, font, &options).unwrap(); // Crashes!
```

### Resource Management

```rust
// DO: Use RAII for cleanup
struct RenderContext {
    pipeline: Pipeline,
    font_cache: FontCache,
}

impl Drop for RenderContext {
    fn drop(&mut self) {
        self.font_cache.clear(); // Free memory
    }
}

// DO: Limit concurrent operations
let semaphore = Semaphore::new(10); // Max 10 renders
async def render_with_limit(text: &str) -> Result<Bitmap> {
    let _permit = semaphore.acquire().await?;
    pipeline.render_text(text, &font, &options)
}
```

### Testing Strategies

```rust
// DO: Test with diverse text samples
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unicode_samples() {
        let samples = [
            "Hello World",                    // ASCII
            "Café résumé",                    // Latin accents  
            "Москва",                        // Cyrillic
            "北京",                          // Chinese
            "تحية العالم",                    // Arabic
            "🌍🚀💻",                        // Emoji
            "Mixed: Hello تحية 🌍",          // Mixed scripts
        ];
        
        for sample in samples {
            let result = pipeline.render_text(sample, &font, &options);
            assert!(result.is_ok(), "Failed to render: {}", sample);
        }
    }
    
    #[test]
    fn test_edge_cases() {
        let edge_cases = [
            "",              // Empty string
            " ",             // Space only
            "\n\t\r",       // Whitespace only
            "A".repeat(10000), // Very long text
            "a".repeat(1),   // Single character
        ];
        
        for case in edge_cases {
            let result = pipeline.render_text(&case, &font, &options);
            assert!(result.is_ok(), "Failed edge case: {:?}", case);
        }
    }
}
```

## Configuration Guidelines

### Production Settings

```toml
# production.toml
[cache]
font_cache_size = "500MB"      # Balance memory and performance
render_cache_size = "1GB"
cache_ttl = 3600              # 1 hour

[performance]
max_concurrent_renders = 100   # Prevent overload
enable_simd = true             # Use CPU optimizations
gpu_acceleration = true        # Use GPU when available

[fonts]
preload_common = ["Roboto", "OpenSans", "NotoSans"]
font_search_paths = ["/usr/share/fonts", "/app/fonts"]

[security]
validate_fonts = true         # Check font integrity
max_font_size = "50MB"         # Prevent DoS
max_text_length = 10000       # Reasonable limits
```

### Development Settings

```toml
# development.toml
[cache]
font_cache_size = "100MB"      # Smaller for dev
enable_debug = true           # More logging

[performance]
max_concurrent_renders = 4    # Don't overwhelm dev machine
profiling_mode = true         # Detailed metrics

[debug]
save_intermediate = true      # For debugging
visualize_glyphs = true       # Visual debugging
```

## Security Considerations

### Input Validation

```rust
// DO: Validate all inputs
fn validate_render_input(text: &str, font_size: f32, bounds: (u32, u32)) -> Result<()> {
    if text.len() > 10000 {
        return Err(TypfError::InvalidInput("Text too long"));
    }
    
    if font_size < 1.0 || font_size > 1000.0 {
        return Err(TypfError::InvalidInput("Invalid font size"));
    }
    
    if bounds.0 > 10000 || bounds.1 > 10000 {
        return Err(TypfError::InvalidInput("Image dimensions too large"));
    }
    
    // Check for potentially malicious content
    if text.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
        return Err(TypfError::InvalidInput("Contains control characters"));
    }
    
    Ok(())
}
```

### Resource Limits

```rust
// DO: Implement resource quotas
struct ResourceMonitor {
    memory_used: AtomicUsize,
    renders_per_second: AtomicU64,
    last_reset: AtomicU64,
}

impl ResourceMonitor {
    fn check_limits(&self) -> Result<()> {
        let memory_mb = self.memory_used.load(Ordering::Relaxed) / (1024 * 1024);
        if memory_mb > 1024 {
            return Err(TypfError::ResourceLimit("Memory usage too high"));
        }
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        let last_reset = self.last_reset.load(Ordering::Relaxed);
        if now - last_reset > 60 {
            // Reset counter every minute
            self.renders_per_second.store(0, Ordering::Relaxed);
            self.last_reset.store(now, Ordering::Relaxed);
        }
        
        let rps = self.renders_per_second.load(Ordering::Relaxed);
        if rps > 1000 {
            return Err(TypfError::RateLimit("Too many requests per second"));
        }
        
        Ok(())
    }
}
```

## Monitoring Checklist

### Health Indicators

- [ ] **Render Success Rate**: Should be >95%
- [ ] **Average Render Time**: Should be <100ms for typical text
- [ ] **Memory Usage**: Stable, not growing continuously
- [ ] **Cache Hit Rate**: Should be >80% for repeated workloads
- [ ] **Error Distribution**: No single error type dominating

### Alert Conditions

```bash
# Example monitoring script
check_typf_health() {
    local success_rate=$(curl -s http://typf-service/metrics | grep render_success_rate | awk '{print $2}')
    local avg_time=$(curl -s http://typf-service/metrics | grep render_duration_avg | awk '{print $2}')
    local memory_mb=$(curl -s http://typf-service/metrics | grep memory_usage_bytes | awk '{print $2/1024/1024}')
    
    if (( $(echo "$success_rate < 0.95" | bc -l) )); then
        alert "Low success rate: $success_rate"
    fi
    
    if (( $(echo "$avg_time > 0.1" | bc -l) )); then
        alert "Slow renders: ${avg_time}s"
    fi
    
    if (( $(echo "$memory_mb > 1024" | bc -l) )); then
        alert "High memory usage: ${memory_mb}MB"
    fi
}
```

---

Debug systematically: isolate the problem, check inputs, verify backend availability, and measure performance. Always handle errors gracefully and monitor key health indicators in production.
