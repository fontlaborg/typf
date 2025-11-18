//! Multi-level cache system for TYPF
//!
//! Implements a three-level cache hierarchy:
//! - L1: Ultra-fast in-memory cache (target <50ns access)
//! - L2: Larger LRU cache with bounded size
//! - L3: Optional persistent cache (disk-based)

use lru::LruCache;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Cache key for shaped text
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShapingCacheKey {
    /// Text content hash
    pub text_hash: u64,
    /// Font identifier
    pub font_id: String,
    /// Shaping parameters hash
    pub params_hash: u64,
}

/// Cache key for rendered glyphs
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GlyphCacheKey {
    /// Font identifier
    pub font_id: String,
    /// Glyph ID
    pub glyph_id: u32,
    /// Size in pixels
    pub size: u32,
    /// Rendering parameters hash
    pub params_hash: u64,
}

/// Cached value with metadata
#[derive(Debug, Clone)]
pub struct CachedValue<T> {
    /// The cached data
    pub data: T,
    /// When this was cached
    pub timestamp: Instant,
    /// How many times this has been accessed
    pub hit_count: u32,
}

/// L1 Cache - Ultra-fast, small capacity
pub struct L1Cache<K: Hash + Eq + Clone, V: Clone> {
    cache: Arc<RwLock<HashMap<K, CachedValue<V>>>>,
    max_size: usize,
}

impl<K: Hash + Eq + Clone, V: Clone> L1Cache<K, V> {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::with_capacity(max_size))),
            max_size,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write();
        if let Some(entry) = cache.get_mut(key) {
            entry.hit_count += 1;
            Some(entry.data.clone())
        } else {
            None
        }
    }

    pub fn insert(&self, key: K, value: V) {
        let mut cache = self.cache.write();

        // Simple eviction: remove oldest if at capacity
        if cache.len() >= self.max_size && !cache.contains_key(&key) {
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, v)| v.timestamp)
                .map(|(k, _)| k.clone())
            {
                cache.remove(&oldest_key);
            }
        }

        cache.insert(
            key,
            CachedValue {
                data: value,
                timestamp: Instant::now(),
                hit_count: 0,
            },
        );
    }

    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read();
        let total_hits: u32 = cache.values().map(|v| v.hit_count).sum();
        CacheStats {
            size: cache.len(),
            capacity: self.max_size,
            total_hits,
            hit_rate: 0.0, // Calculated externally
        }
    }
}

/// L2 Cache - Larger LRU cache
pub struct L2Cache<K: Hash + Eq + Clone, V: Clone> {
    cache: Arc<RwLock<LruCache<K, CachedValue<V>>>>,
    capacity: NonZeroUsize,
}

impl<K: Hash + Eq + Clone, V: Clone> L2Cache<K, V> {
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(1000).unwrap());
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            capacity,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write();
        if let Some(entry) = cache.get_mut(key) {
            entry.hit_count += 1;
            Some(entry.data.clone())
        } else {
            None
        }
    }

    pub fn insert(&self, key: K, value: V) {
        let mut cache = self.cache.write();
        cache.put(
            key,
            CachedValue {
                data: value,
                timestamp: Instant::now(),
                hit_count: 0,
            },
        );
    }

    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read();
        let total_hits: u32 = cache.iter().map(|(_, v)| v.hit_count).sum();
        CacheStats {
            size: cache.len(),
            capacity: self.capacity.get(),
            total_hits,
            hit_rate: 0.0,
        }
    }
}

/// Multi-level cache combining L1 and L2
pub struct MultiLevelCache<K: Hash + Eq + Clone, V: Clone> {
    l1: L1Cache<K, V>,
    l2: L2Cache<K, V>,
    stats: Arc<RwLock<CacheMetrics>>,
}

impl<K: Hash + Eq + Clone, V: Clone> MultiLevelCache<K, V> {
    pub fn new(l1_size: usize, l2_size: usize) -> Self {
        Self {
            l1: L1Cache::new(l1_size),
            l2: L2Cache::new(l2_size),
            stats: Arc::new(RwLock::new(CacheMetrics::default())),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let start = Instant::now();
        let mut stats = self.stats.write();
        stats.total_requests += 1;

        // Try L1 first
        if let Some(value) = self.l1.get(key) {
            stats.l1_hits += 1;
            stats.total_l1_time += start.elapsed();
            return Some(value);
        }

        // Try L2
        if let Some(value) = self.l2.get(key) {
            stats.l2_hits += 1;
            stats.total_l2_time += start.elapsed();
            // Promote to L1
            self.l1.insert(key.clone(), value.clone());
            return Some(value);
        }

        stats.misses += 1;
        None
    }

    pub fn insert(&self, key: K, value: V) {
        // Insert into both L1 and L2
        self.l1.insert(key.clone(), value.clone());
        self.l2.insert(key, value);
    }

    pub fn hit_rate(&self) -> f64 {
        let stats = self.stats.read();
        if stats.total_requests == 0 {
            0.0
        } else {
            let hits = stats.l1_hits + stats.l2_hits;
            hits as f64 / stats.total_requests as f64
        }
    }

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

    pub fn metrics(&self) -> CacheMetrics {
        self.stats.read().clone()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
    pub total_hits: u32,
    pub hit_rate: f32,
}

/// Detailed cache metrics
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

/// Global cache manager
pub struct CacheManager {
    shaping_cache: MultiLevelCache<ShapingCacheKey, Arc<Vec<u8>>>,
    glyph_cache: MultiLevelCache<GlyphCacheKey, Arc<Vec<u8>>>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            // L1: 100 entries, L2: 10,000 entries
            shaping_cache: MultiLevelCache::new(100, 10_000),
            glyph_cache: MultiLevelCache::new(1000, 100_000),
        }
    }

    pub fn get_shaped(&self, key: &ShapingCacheKey) -> Option<Arc<Vec<u8>>> {
        self.shaping_cache.get(key)
    }

    pub fn cache_shaped(&self, key: ShapingCacheKey, data: Arc<Vec<u8>>) {
        self.shaping_cache.insert(key, data);
    }

    pub fn get_glyph(&self, key: &GlyphCacheKey) -> Option<Arc<Vec<u8>>> {
        self.glyph_cache.get(key)
    }

    pub fn cache_glyph(&self, key: GlyphCacheKey, data: Arc<Vec<u8>>) {
        self.glyph_cache.insert(key, data);
    }

    pub fn report_metrics(&self) -> String {
        let shaping = self.shaping_cache.metrics();
        let glyph = self.glyph_cache.metrics();

        format!(
            "Cache Metrics:\n\
             Shaping Cache:\n\
             - Hit Rate: {:.2}%\n\
             - L1 Hit Rate: {:.2}%\n\
             - Avg Access Time: {:?}\n\
             Glyph Cache:\n\
             - Hit Rate: {:.2}%\n\
             - L1 Hit Rate: {:.2}%\n\
             - Avg Access Time: {:?}",
            shaping.hit_rate() * 100.0,
            shaping.l1_hit_rate() * 100.0,
            self.shaping_cache.avg_access_time(),
            glyph.hit_rate() * 100.0,
            glyph.l1_hit_rate() * 100.0,
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
    fn test_l1_cache() {
        let cache: L1Cache<String, String> = L1Cache::new(2);

        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());

        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), Some("value2".to_string()));

        // Should evict oldest when full
        cache.insert("key3".to_string(), "value3".to_string());
        assert_eq!(cache.get(&"key3".to_string()), Some("value3".to_string()));
    }

    #[test]
    fn test_multi_level_cache() {
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
    fn test_cache_promotion() {
        let cache: MultiLevelCache<u32, String> = MultiLevelCache::new(1, 10);

        cache.insert(1, "one".to_string());
        cache.insert(2, "two".to_string()); // Will evict 1 from L1

        // First get should hit L2 and promote to L1
        assert_eq!(cache.get(&1), Some("one".to_string()));

        let metrics = cache.metrics();
        assert!(metrics.l2_hits > 0);
    }
}
