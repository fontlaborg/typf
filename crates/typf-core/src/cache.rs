//! Scan-resistant caching with TinyLFU and byte-weighted eviction
//!
//! Uses Moka's TinyLFU eviction policy to handle scan workloads gracefully.
//! Unlike timestamp-based eviction, TinyLFU tracks access frequency and
//! rejects one-time "scan" entries that would pollute the cache.
//!
//! **Byte-weighted eviction**: Caches track actual memory usage, not just
//! entry counts. A 4MB emoji bitmap consumes 4000x more quota than a 1KB
//! glyph. This prevents memory explosions from pathological fonts.
//!
//! This prevents unbounded memory growth when processing many unique fonts
//! (e.g., font matching across hundreds of candidates).

use moka::sync::Cache;
use parking_lot::RwLock;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Default cache byte limit: 512 MB
///
/// Can be overridden via `TYPF_CACHE_MAX_BYTES` environment variable.
pub const DEFAULT_CACHE_MAX_BYTES: u64 = 512 * 1024 * 1024;

/// Uniquely identifies shaped text for caching
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShapingCacheKey {
    /// Hash of the text content
    pub text_hash: u64,
    /// Which font we're using
    pub font_id: String,
    /// How we want it shaped
    pub params_hash: u64,
}

/// Uniquely identifies rendered glyphs for caching
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GlyphCacheKey {
    /// Which font this glyph comes from
    pub font_id: String,
    /// The specific glyph
    pub glyph_id: u32,
    /// How big we want it
    pub size: u32,
    /// Rendering style parameters
    pub params_hash: u64,
}

/// Scan-resistant cache backed by Moka's TinyLFU
///
/// TinyLFU tracks frequency of both hits and misses, rejecting
/// infrequent "scan" accesses that would pollute a pure LRU cache.
pub struct MultiLevelCache<K, V>
where
    K: Hash + Eq + Send + Sync + Clone + 'static,
    V: Clone + Send + Sync + 'static,
{
    cache: Cache<K, V>,
    stats: Arc<RwLock<CacheMetrics>>,
}

impl<K, V> std::fmt::Debug for MultiLevelCache<K, V>
where
    K: Hash + Eq + Send + Sync + Clone + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiLevelCache")
            .field("entry_count", &self.cache.entry_count())
            .field("hit_rate", &self.hit_rate())
            .finish()
    }
}

impl<K, V> MultiLevelCache<K, V>
where
    K: Hash + Eq + Send + Sync + Clone + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Build a scan-resistant cache with specified capacity
    ///
    /// The `l1_size` and `l2_size` parameters are combined for total capacity.
    /// Moka's TinyLFU internally manages hot/cold separation.
    pub fn new(l1_size: usize, l2_size: usize) -> Self {
        let total_capacity = (l1_size + l2_size) as u64;
        let cache = Cache::builder()
            .max_capacity(total_capacity)
            // TinyLFU is the default, but be explicit
            .eviction_policy(moka::policy::EvictionPolicy::tiny_lfu())
            // Time-to-idle: evict entries not accessed for 10 minutes
            .time_to_idle(Duration::from_secs(600))
            .build();

        Self {
            cache,
            stats: Arc::new(RwLock::new(CacheMetrics::default())),
        }
    }

    /// Look up a cached value
    ///
    /// TinyLFU admission policy means frequently-accessed keys stay cached
    /// while scan-like one-time accesses are rejected.
    pub fn get(&self, key: &K) -> Option<V> {
        let start = Instant::now();
        let mut stats = self.stats.write();
        stats.total_requests += 1;

        if let Some(value) = self.cache.get(key) {
            stats.l1_hits += 1; // Count all hits as "L1" for API compatibility
            stats.total_l1_time += start.elapsed();
            Some(value)
        } else {
            stats.misses += 1;
            None
        }
    }

    /// Store a value in the cache
    ///
    /// Note: TinyLFU may reject this entry if the key hasn't been
    /// seen frequently enough. This is intentional for scan resistance.
    pub fn insert(&self, key: K, value: V) {
        self.cache.insert(key, value);
    }

    /// Cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let stats = self.stats.read();
        if stats.total_requests == 0 {
            0.0
        } else {
            let hits = stats.l1_hits + stats.l2_hits;
            hits as f64 / stats.total_requests as f64
        }
    }

    /// Average access time for cache hits
    pub fn avg_access_time(&self) -> Duration {
        let stats = self.stats.read();
        let total_time = stats.total_l1_time + stats.total_l2_time;
        let total_hits = stats.l1_hits + stats.l2_hits;
        if total_hits == 0 {
            Duration::ZERO
        } else {
            total_time / total_hits as u32
        }
    }

    /// Full performance snapshot
    pub fn metrics(&self) -> CacheMetrics {
        self.stats.read().clone()
    }

    /// Number of entries currently in cache
    pub fn len(&self) -> usize {
        self.cache.entry_count() as usize
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.entry_count() == 0
    }

    /// Clear all entries and reset stats
    pub fn clear(&self) {
        self.cache.invalidate_all();
        let mut stats = self.stats.write();
        *stats = CacheMetrics::default();
    }

    /// Force pending operations to complete (for testing)
    #[cfg(test)]
    pub fn sync(&self) {
        self.cache.run_pending_tasks();
    }
}

/// Basic cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
    pub total_hits: u32,
    pub hit_rate: f32,
}

/// Performance metrics for cache operations
#[derive(Debug, Clone, Default)]
pub struct CacheMetrics {
    pub total_requests: u64,
    pub l1_hits: u64,
    pub l2_hits: u64,
    pub misses: u64,
    pub total_l1_time: Duration,
    pub total_l2_time: Duration,
}

impl CacheMetrics {
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.l1_hits + self.l2_hits) as f64 / self.total_requests as f64
        }
    }

    pub fn l1_hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.l1_hits as f64 / self.total_requests as f64
        }
    }
}

/// Get the cache byte limit from environment or use default.
///
/// Set `TYPF_CACHE_MAX_BYTES` to override (e.g., "268435456" for 256MB).
pub fn get_cache_max_bytes() -> u64 {
    std::env::var("TYPF_CACHE_MAX_BYTES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_CACHE_MAX_BYTES)
}

/// Byte-weighted cache for RenderOutput values
///
/// Unlike entry-count caches, this tracks actual memory usage.
/// A 4MB emoji bitmap consumes 4000x more quota than a 1KB glyph.
pub struct RenderOutputCache<K>
where
    K: Hash + Eq + Send + Sync + Clone + 'static,
{
    cache: Cache<K, crate::types::RenderOutput>,
    stats: Arc<RwLock<CacheMetrics>>,
    max_bytes: u64,
}

impl<K> std::fmt::Debug for RenderOutputCache<K>
where
    K: Hash + Eq + Send + Sync + Clone + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderOutputCache")
            .field("entry_count", &self.cache.entry_count())
            .field("weighted_size", &self.cache.weighted_size())
            .field("max_bytes", &self.max_bytes)
            .field("hit_rate", &self.hit_rate())
            .finish()
    }
}

impl<K> RenderOutputCache<K>
where
    K: Hash + Eq + Send + Sync + Clone + 'static,
{
    /// Create a byte-weighted cache with specified maximum bytes.
    pub fn new(max_bytes: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_bytes)
            .weigher(|_key: &K, value: &crate::types::RenderOutput| {
                // Weight = byte size, minimum 1 to avoid division issues
                value.byte_size().max(1) as u32
            })
            .eviction_policy(moka::policy::EvictionPolicy::tiny_lfu())
            .time_to_idle(Duration::from_secs(600))
            .build();

        Self {
            cache,
            stats: Arc::new(RwLock::new(CacheMetrics::default())),
            max_bytes,
        }
    }

    /// Create a cache with the default byte limit (512 MB or env override).
    pub fn with_default_limit() -> Self {
        Self::new(get_cache_max_bytes())
    }

    /// Look up a cached render output.
    pub fn get(&self, key: &K) -> Option<crate::types::RenderOutput> {
        let start = Instant::now();
        let mut stats = self.stats.write();
        stats.total_requests += 1;

        if let Some(value) = self.cache.get(key) {
            stats.l1_hits += 1;
            stats.total_l1_time += start.elapsed();
            Some(value)
        } else {
            stats.misses += 1;
            None
        }
    }

    /// Store a render output in the cache.
    ///
    /// Large outputs may be rejected by TinyLFU if not accessed frequently.
    pub fn insert(&self, key: K, value: crate::types::RenderOutput) {
        self.cache.insert(key, value);
    }

    /// Cache hit rate (0.0 to 1.0).
    pub fn hit_rate(&self) -> f64 {
        let stats = self.stats.read();
        if stats.total_requests == 0 {
            0.0
        } else {
            stats.l1_hits as f64 / stats.total_requests as f64
        }
    }

    /// Current weighted size in bytes.
    pub fn weighted_size(&self) -> u64 {
        self.cache.weighted_size()
    }

    /// Number of entries in cache.
    pub fn entry_count(&self) -> u64 {
        self.cache.entry_count()
    }

    /// Performance metrics.
    pub fn metrics(&self) -> CacheMetrics {
        self.stats.read().clone()
    }

    /// Clear all entries.
    pub fn clear(&self) {
        self.cache.invalidate_all();
        let mut stats = self.stats.write();
        *stats = CacheMetrics::default();
    }

    /// Force pending operations to complete (for testing).
    #[cfg(test)]
    pub fn sync(&self) {
        self.cache.run_pending_tasks();
    }
}

/// Centralized cache manager for shaping and glyph caches
pub struct CacheManager {
    shaping_cache: MultiLevelCache<ShapingCacheKey, Arc<Vec<u8>>>,
    glyph_cache: MultiLevelCache<GlyphCacheKey, Arc<Vec<u8>>>,
}

impl CacheManager {
    /// Create a manager with sensible default sizes
    ///
    /// Default capacities are conservative to prevent memory issues:
    /// - Shaping: 10,100 entries (shapes are larger)
    /// - Glyphs: 101,000 entries (individual glyphs are smaller)
    pub fn new() -> Self {
        Self {
            shaping_cache: MultiLevelCache::new(100, 10_000),
            glyph_cache: MultiLevelCache::new(1000, 100_000),
        }
    }

    /// Look up previously shaped text
    pub fn get_shaped(&self, key: &ShapingCacheKey) -> Option<Arc<Vec<u8>>> {
        self.shaping_cache.get(key)
    }

    /// Save shaping results for next time
    pub fn cache_shaped(&self, key: ShapingCacheKey, data: Arc<Vec<u8>>) {
        self.shaping_cache.insert(key, data);
    }

    /// Find a rendered glyph we cached earlier
    pub fn get_glyph(&self, key: &GlyphCacheKey) -> Option<Arc<Vec<u8>>> {
        self.glyph_cache.get(key)
    }

    /// Remember this glyph for future renders
    pub fn cache_glyph(&self, key: GlyphCacheKey, data: Arc<Vec<u8>>) {
        self.glyph_cache.insert(key, data);
    }

    /// Human-readable performance report
    pub fn report_metrics(&self) -> String {
        let shaping = self.shaping_cache.metrics();
        let glyph = self.glyph_cache.metrics();

        format!(
            "Cache Performance (TinyLFU):\n\
             Shaping Cache:\n\
             - Hit Rate: {:.2}%\n\
             - Entries: {}\n\
             - Average Access: {:?}\n\
             Glyph Cache:\n\
             - Hit Rate: {:.2}%\n\
             - Entries: {}\n\
             - Average Access: {:?}",
            shaping.hit_rate() * 100.0,
            self.shaping_cache.len(),
            self.shaping_cache.avg_access_time(),
            glyph.hit_rate() * 100.0,
            self.glyph_cache.len(),
            self.glyph_cache.avg_access_time(),
        )
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_and_get() {
        let cache: MultiLevelCache<String, String> = MultiLevelCache::new(10, 100);

        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());

        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), Some("value2".to_string()));
        assert_eq!(cache.get(&"key3".to_string()), None);
    }

    #[test]
    fn test_cache_metrics() {
        let cache: MultiLevelCache<u32, String> = MultiLevelCache::new(10, 100);

        cache.insert(1, "one".to_string());
        cache.insert(2, "two".to_string());

        assert_eq!(cache.get(&1), Some("one".to_string()));
        assert_eq!(cache.get(&2), Some("two".to_string()));
        assert_eq!(cache.get(&3), None);

        let metrics = cache.metrics();
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.l1_hits, 2);
        assert_eq!(metrics.misses, 1);
    }

    #[test]
    fn test_cache_clear() {
        let cache: MultiLevelCache<u32, String> = MultiLevelCache::new(10, 100);

        cache.insert(1, "one".to_string());
        cache.sync(); // Force pending insert to complete
        assert!(!cache.is_empty());

        cache.clear();
        cache.sync(); // Force pending invalidation to complete
        assert!(cache.is_empty());
        assert_eq!(cache.get(&1), None);
    }

    #[test]
    fn test_scan_resistance_concept() {
        // TinyLFU tracks frequency - items accessed multiple times
        // are more likely to stay cached than one-time scans.
        // This test verifies the cache works; actual scan resistance
        // is handled by Moka's internal TinyLFU implementation.
        let cache: MultiLevelCache<u32, String> = MultiLevelCache::new(5, 5);

        // Simulate a "hot" key accessed multiple times
        cache.insert(1, "hot".to_string());
        for _ in 0..10 {
            cache.get(&1);
        }

        // Simulate scan of many unique keys
        for i in 100..200 {
            cache.insert(i, format!("scan_{}", i));
        }

        // Hot key should still be accessible (TinyLFU protects it)
        // Note: This is probabilistic - TinyLFU may or may not keep it
        // The important thing is the cache doesn't grow unbounded
        assert!(cache.len() <= 10, "Cache should respect capacity limit");
    }

    #[test]
    fn test_render_output_byte_size() {
        use crate::types::{BitmapData, BitmapFormat, RenderOutput, VectorData, VectorFormat};

        // Test bitmap byte size (should be data.len())
        let bitmap = RenderOutput::Bitmap(BitmapData {
            width: 100,
            height: 100,
            format: BitmapFormat::Rgba8,
            data: vec![0u8; 40_000], // 100x100x4 = 40KB
        });
        assert_eq!(bitmap.byte_size(), 40_000);

        // Test vector byte size (should be data.len())
        let vector = RenderOutput::Vector(VectorData {
            format: VectorFormat::Svg,
            data: "<svg>test</svg>".to_string(),
        });
        assert_eq!(vector.byte_size(), 15);

        // Test JSON byte size
        let json = RenderOutput::Json(r#"{"test": true}"#.to_string());
        assert_eq!(json.byte_size(), 14);
    }

    #[test]
    fn test_byte_weighted_cache_respects_limit() {
        use crate::types::{BitmapData, BitmapFormat, RenderOutput};

        // Create a cache with 100KB limit
        let cache: RenderOutputCache<u32> = RenderOutputCache::new(100_000);

        // Insert entries totaling ~150KB (should evict some)
        for i in 0..15 {
            let output = RenderOutput::Bitmap(BitmapData {
                width: 50,
                height: 50,
                format: BitmapFormat::Rgba8,
                data: vec![i as u8; 10_000], // 10KB each
            });
            cache.insert(i, output);
        }
        cache.sync(); // Force pending operations

        // Weighted size should be at or below the limit
        assert!(
            cache.weighted_size() <= 100_000,
            "Cache weighted size {} should be <= 100KB",
            cache.weighted_size()
        );
    }

    #[test]
    fn test_byte_weighted_cache_large_item_eviction() {
        use crate::types::{BitmapData, BitmapFormat, RenderOutput};

        // Create a cache with 50KB limit
        let cache: RenderOutputCache<u32> = RenderOutputCache::new(50_000);

        // Insert a small item
        let small = RenderOutput::Bitmap(BitmapData {
            width: 10,
            height: 10,
            format: BitmapFormat::Gray8,
            data: vec![0u8; 100], // 100 bytes
        });
        cache.insert(1, small);

        // Insert a large item (40KB)
        let large = RenderOutput::Bitmap(BitmapData {
            width: 100,
            height: 100,
            format: BitmapFormat::Rgba8,
            data: vec![0u8; 40_000], // 40KB
        });
        cache.insert(2, large);
        cache.sync();

        // Both should fit (100 + 40KB = ~40KB < 50KB limit)
        assert!(cache.weighted_size() <= 50_000);
    }
}
