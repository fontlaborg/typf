//! Speed up your pipeline with intelligent caching
//!
//! Two levels of caching keep frequently-used data at your fingertips:
//! - L1: Blazing fast hot cache (<50ns access) for the most recent items
//! - L2: Larger LRU cache for everything else that still matters
//!
//! Shaping results and rendered glyphs get cached automatically,
//! so repeated text or fonts feel instant.

use lru::LruCache;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

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

/// Data that's been cached, plus useful metadata
#[derive(Debug, Clone)]
pub struct CachedValue<T> {
    /// The actual cached data
    pub data: T,
    /// When we first cached this
    pub timestamp: Instant,
    /// How popular this entry has been
    pub hit_count: u32,
}

/// The sprinter: small, blindingly fast, for the hottest data
pub struct L1Cache<K: Hash + Eq + Clone, V: Clone> {
    cache: Arc<RwLock<HashMap<K, CachedValue<V>>>>,
    max_size: usize,
}

impl<K: Hash + Eq + Clone, V: Clone> L1Cache<K, V> {
    /// Create a new L1 cache with the specified capacity
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::with_capacity(max_size))),
            max_size,
        }
    }

    /// Grab data if we have it, update stats along the way
    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write();
        if let Some(entry) = cache.get_mut(key) {
            entry.hit_count += 1;
            Some(entry.data.clone())
        } else {
            None
        }
    }

    /// Store something valuable for fast access later
    pub fn insert(&self, key: K, value: V) {
        let mut cache = self.cache.write();

        // Evict the oldest entry when we're full
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

    /// Get a snapshot of cache performance
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read();
        let total_hits: u32 = cache.values().map(|v| v.hit_count).sum();
        CacheStats {
            size: cache.len(),
            capacity: self.max_size,
            total_hits,
            hit_rate: 0.0, // Calculated at higher levels
        }
    }
}

/// The marathon runner: bigger, smart about what to fopixat
pub struct L2Cache<K: Hash + Eq + Clone, V: Clone> {
    cache: Arc<RwLock<LruCache<K, CachedValue<V>>>>,
    capacity: NonZeroUsize,
}

/// Default L2 cache capacity
const DEFAULT_L2_CAPACITY: NonZeroUsize = match NonZeroUsize::new(1000) {
    Some(v) => v,
    None => unreachable!(),
};

impl<K: Hash + Eq + Clone, V: Clone> L2Cache<K, V> {
    /// Create a new L2 cache with LRU eviction
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or(DEFAULT_L2_CAPACITY);
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            capacity,
        }
    }

    /// Find data we cached recently
    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write();
        if let Some(entry) = cache.get_mut(key) {
            entry.hit_count += 1;
            Some(entry.data.clone())
        } else {
            None
        }
    }

    /// Remember this for next time
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

    /// Performance metrics for this cache level
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

/// Both levels working together for the best of both worlds
pub struct MultiLevelCache<K: Hash + Eq + Clone, V: Clone> {
    l1: L1Cache<K, V>,
    l2: L2Cache<K, V>,
    stats: Arc<RwLock<CacheMetrics>>,
}

impl<K: Hash + Eq + Clone, V: Clone> MultiLevelCache<K, V> {
    /// Build a two-level cache with specified capacities
    pub fn new(l1_size: usize, l2_size: usize) -> Self {
        Self {
            l1: L1Cache::new(l1_size),
            l2: L2Cache::new(l2_size),
            stats: Arc::new(RwLock::new(CacheMetrics::default())),
        }
    }

    /// Smart lookup: L1 first, then L2 with auto-promotion
    pub fn get(&self, key: &K) -> Option<V> {
        let start = Instant::now();
        let mut stats = self.stats.write();
        stats.total_requests += 1;

        // L1: The sprinter responds instantly
        if let Some(value) = self.l1.get(key) {
            stats.l1_hits += 1;
            stats.total_l1_time += start.elapsed();
            return Some(value);
        }

        // L2: The marathon runner helps out
        if let Some(value) = self.l2.get(key) {
            stats.l2_hits += 1;
            stats.total_l2_time += start.elapsed();
            // Popular data gets promoted to L1
            self.l1.insert(key.clone(), value.clone());
            return Some(value);
        }

        stats.misses += 1;
        None
    }

    /// Store in both levels for maximum availability
    pub fn insert(&self, key: K, value: V) {
        self.l1.insert(key.clone(), value.clone());
        self.l2.insert(key, value);
    }

    /// How often do we find what we're looking for?
    pub fn hit_rate(&self) -> f64 {
        let stats = self.stats.read();
        if stats.total_requests == 0 {
            0.0
        } else {
            let hits = stats.l1_hits + stats.l2_hits;
            hits as f64 / stats.total_requests as f64
        }
    }

    /// Average time to fetch cached data
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
}

/// Basic cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
    pub total_hits: u32,
    pub hit_rate: f32,
}

/// Everything you need to know about cache performance
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

/// One place to manage all your caches
pub struct CacheManager {
    shaping_cache: MultiLevelCache<ShapingCacheKey, Arc<Vec<u8>>>,
    glyph_cache: MultiLevelCache<GlyphCacheKey, Arc<Vec<u8>>>,
}

impl CacheManager {
    /// Create a manager with sensible default sizes
    pub fn new() -> Self {
        Self {
            // Shaping: 100 hot, 10k total (shaping is expensive)
            shaping_cache: MultiLevelCache::new(100, 10_000),
            // Glyphs: 1000 hot, 100k total (individual glyphs are cheap)
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
            "Cache Performance:\n\
             Shaping Cache:\n\
             - Overall Hit Rate: {:.2}%\n\
             - L1 (Hot) Hit Rate: {:.2}%\n\
             - Average Access: {:?}\n\
             Glyph Cache:\n\
             - Overall Hit Rate: {:.2}%\n\
             - L1 (Hot) Hit Rate: {:.2}%\n\
             - Average Access: {:?}",
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
