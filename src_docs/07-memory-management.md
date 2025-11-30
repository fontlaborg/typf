---
title: Memory Management
icon: lucide/hard-drive
tags:
  - Memory
  - Performance
  - Optimization
---

# Memory Management

Fast text rendering needs smart memory use. Here's how Typf handles it.

## Font Data Strategy

Fonts stay loaded once, shared everywhere.

```rust
pub struct Font {
    pub data: &'static [u8],           // Leaked memory, never freed
    pub face: FontFace,                // Parsed font metadata
    pub cache: GlyphCache,             // Pre-rendered glyphs
}

impl FontDatabase {
    pub fn load_font(&self, path: &Path) -> Result<Arc<Font>> {
        // Load font data once, leak it permanently
        let data = std::fs::read(path)?;
        let leaked = Box::leak(data.into_boxed_slice());
        let face = FontFace::from_bytes(leaked)?;
        
        Ok(Arc::new(Font {
            data: leaked,
            face,
            cache: GlyphCache::new(),
        }))
    }
}
```

**Why leak memory?** Fonts are read-only data used throughout the program. Loading once prevents duplicate parsing and improves performance.

## Glyph Caching

Rendered glyphs get cached to avoid重复的渲染。

```rust
pub struct GlyphCache {
    bitmap_cache: LruCache<GlyphKey, BitmapData>,
    outline_cache: LruCache<GlyphKey, Outline>,
    metrics_cache: LruCache<GlyphKey, GlyphMetrics>,
}

impl GlyphCache {
    pub fn get_or_render<F>(&mut self, key: GlyphKey, render_fn: F) -> Result<&BitmapData> 
    where F: FnOnce() -> Result<BitmapData> {
        if !self.bitmap_cache.contains(&key) {
            let bitmap = render_fn()?;
            self.bitmap_cache.put(key, bitmap);
        }
        
        Ok(self.bitmap_cache.get(&key).unwrap())
    }
}
```

Cache layers:
1. **Bitmap cache** - Final rendered pixels
2. **Outline cache** - Vector outlines for scaling
3. **Metrics cache** - Glyph measurements

LRU eviction keeps memory usage bounded.

## Shaping Results

Text shaping results get cached for reuse.

```rust
pub struct ShapingCache {
    cache: Arc<DashMap<ShapingKey, ShapingResult>>,
    max_entries: usize,
}

#[derive(Hash, Eq, PartialEq)]
struct ShapingKey {
    text_hash: u64,
    font_id: FontId,
    options_hash: u64,
}

impl ShapingCache {
    pub fn get_or_shape<F>(&self, key: ShapingKey, shape_fn: F) -> Result<ShapingResult>
    where F: FnOnce() -> Result<ShapingResult> {
        entry(self.cache.entry(key))
            .or_try_insert_with(shape_fn)
            .map(|entry| entry.clone())
    }
}
```

Thread-safe with DashMap for concurrent access.

## Render Output Pool

Reuse render buffers instead of allocating new ones.

```rust
pub struct RenderPool {
    bitmaps: Vec<BitmapData>,
    vectors: Vec<VectorData>,
    max_size: usize,
}

impl RenderPool {
    pub fn get_bitmap(&mut self, width: u32, height: u32) -> BitmapData {
        // Find existing buffer that fits
        if let Some(pos) = self.bitmaps.iter().position(|b| b.width >= width && b.height >= height) {
            self.bitmaps.swap_remove(pos)
        } else {
            // Allocate new if none available
            BitmapData::new(width, height)
        }
    }
    
    pub fn return_bitmap(&mut self, bitmap: BitmapData) {
        if self.bitmaps.len() < self.max_size {
            self.bitmaps.push(bitmap);
        }
        // Otherwise drop it
    }
}
```

## Memory Pools

Pre-allocate common data structures.

```rust
pub struct MemoryPool {
    glyph_buffer: Vec<Glyph>,
    position_buffer: Vec<Position>,
    segment_buffer: Vec<TextSegment>,
}

impl MemoryPool {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            glyph_buffer: Vec::with_capacity(capacity),
            position_buffer: Vec::with_capacity(capacity),
            segment_buffer: Vec::with_capacity(capacity / 10), // Estimate
        }
    }
    
    pub fn borrow_buffers(&mut self) -> (&mut Vec<Glyph>, &mut Vec<Position>) {
        self.glyph_buffer.clear();
        self.position_buffer.clear();
        (&mut self.glyph_buffer, &mut self.position_buffer)
    }
}
```

## Concurrent Access

Share data safely across threads.

```rust
pub struct SharedResources {
    fonts: Arc<DashMap<FontId, Arc<Font>>>,
    glyph_cache: Arc<RwLock<GlyphCache>>,
    shaping_cache: Arc<ShapingCache>,
    render_pool: Arc<Mutex<RenderPool>>,
}

impl SharedResources {
    pub fn new() -> Self {
        Self {
            fonts: Arc::new(DashMap::new()),
            glyph_cache: Arc::new(RwLock::new(GlyphCache::new(1000))),
            shaping_cache: Arc::new(ShapingCache::new(500)),
            render_pool: Arc::new(Mutex::new(RenderPool::new())),
        }
    }
}
```

Different synchronization for different use cases:
- **DashMap** - Lock-free concurrent hash map
- **RwLock** - Multiple readers, exclusive writer
- **Mutex** - Exclusive access for complex operations

## Memory Usage Tracking

Monitor what's using memory.

```rust
pub struct MemoryTracker {
    font_usage: AtomicUsize,
    cache_usage: AtomicUsize,
    pool_usage: AtomicUsize,
    peak_usage: AtomicUsize,
}

impl MemoryTracker {
    pub fn allocate(&self, size: usize) {
        self.font_usage.fetch_add(size, Ordering::Relaxed);
        self.update_peak();
    }
    
    pub fn cache_add(&self, size: usize) {
        self.cache_usage.fetch_add(size, Ordering::Relaxed);
        self.update_peak();
    }
    
    fn update_peak(&self) {
        let current = self.font_usage.load(Ordering::Relaxed) + 
                     self.cache_usage.load(Ordering::Relaxed) +
                     self.pool_usage.load(Ordering::Relaxed);
        
        let mut peak = self.peak_usage.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_usage.compare_exchange_weak(
                peak, current, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(actual) => peak = actual,
            }
        }
    }
}
```

## Cleanup Strategies

Control memory growth.

### Cache Size Limits

```rust
impl GlyphCache {
    pub fn enforce_limits(&mut self) {
        while self.bitmap_cache.len() > MAX_BITMAP_CACHE {
            self.bitmap_cache.pop_lru();
        }
        
        while self.outline_cache.len() > MAX_OUTLINE_CACHE {
            self.outline_cache.pop_lru();
        }
    }
}
```

### Periodic Cleanup

```rust
pub struct CacheJanitor {
    cleanup_interval: Duration,
    max_age: Duration,
}

impl CacheJanitor {
    pub fn start(&self, caches: Vec<Arc<dyn Cacheable>>) {
        for cache in caches {
            let interval = self.cleanup_interval;
            let max_age = self.max_age;
            
            thread::spawn(move || {
                loop {
                    thread::sleep(interval);
                    cache.cleanup_old_entries(max_age);
                }
            });
        }
    }
}
```

## Memory Profiling

Find memory bottlenecks.

```rust
#[cfg(debug_assertions)]
pub struct MemoryProfiler {
    allocations: HashMap<String, usize>,
    allocations_by_type: HashMap<std::any::TypeId, usize>,
}

impl MemoryProfiler {
    pub fn track_allocation<T>(&mut self, size: usize) {
        let type_name = std::any::type_name::<T>();
        *self.allocations.entry(type_name.to_string()).or_insert(0) += size;
        
        let type_id = std::any::TypeId::of::<T>();
        *self.allocations_by_type.entry(type_id).or_insert(0) += size;
    }
    
    pub fn report(&self) -> String {
        let mut report = String::new();
        for (name, size) in &self.allocations {
            report.push_str(&format!("{}: {} bytes\n", name, size));
        }
        report
    }
}
```

## Platform-Specific Optimizations

### macOS

```rust
#[cfg(target_os = "macos")]
pub fn allocate_font_data(path: &Path) -> Result<&'static [u8]> {
    use std::os::macos::fs::MetadataExt;
    
    let file = std::fs::File::open(path)?;
    let metadata = file.metadata()?;
    let size = metadata.st_size() as usize;
    
    let fd = file.as_raw_fd();
    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ,
            libc::MAP_PRIVATE,
            fd,
            0,
        )
    };
    
    if ptr == libc::MAP_FAILED {
        return Err(MemoryError::MapFailed);
    }
    
    Ok(unsafe { std::slice::from_raw_parts(ptr as *const u8, size) })
}
```

### Linux/Windows

```rust
#[cfg(any(target_os = "linux", target_os = "windows"))]
pub fn allocate_font_data(path: &Path) -> Result<&'static [u8]> {
    let data = std::fs::read(path)?;
    Ok(Box::leak(data.into_boxed_slice()))
}
```

## Best Practices

1. **Load once, share everywhere** - Font data is immutable
2. **Cache aggressively, evict intelligently** - LRU for most things
3. **Reuse buffers** - Object pools for temporary data
4. **Track usage** - Know where memory goes
5. **Clean up periodically** - Don't let caches grow forever
6. **Profile in debug** - Find memory leaks early

## Common Pitfalls

### Don't clone font data

```rust
// BAD - Duplicates font data
let font_data = std::fs::read("font.ttf")?;
let font1 = Font::from_data(font_data.clone())?;
let font2 = Font::from_data(font_data)?;

// GOOD - Shared reference
let font_data = load_font_data("font.ttf")?; // &'static [u8]
let font1 = Font::from_data(font_data)?;
let font2 = Font::from_data(font_data)?;
```

### Don't over-cache

```rust
// BAD - Caching everything
cache.put(image_1000x1000); // Uses 4MB
cache.put(image_2000x2000); // Uses 16MB
cache.put(image_4000x4000); // Uses 64MB

// GOOD - Size limits and intelligent eviction
if cache.total_size() < MAX_CACHE_SIZE {
    cache.put(image);
} else {
    cache.evict_lru();
    cache.put(image);
}
```

---

Smart memory management makes text rendering fast. Load once, cache wisely, clean up regularly.
