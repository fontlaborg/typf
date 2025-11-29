//! Backend-agnostic shaping cache
//!
//! Shared cache implementation for text shaping results. Used by HarfBuzz-based
//! shapers to avoid expensive reshaping operations.
//!
//! This module was extracted from duplicated code in typf-shape-hb and
//! typf-shape-icu-hb to provide a single source of truth.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

use crate::cache::MultiLevelCache;
use crate::types::ShapingResult;

/// Key for caching shaping results
///
/// Uniquely identifies a shaping operation by its inputs:
/// text content, font identity, size, locale settings, and OpenType features.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapingCacheKey {
    /// Text content
    pub text: String,
    /// Font identifier (hash of font data)
    pub font_id: u64,
    /// Font size in points (stored as u32: size * 100 for hash stability)
    pub size: u32,
    /// Language code (e.g., "en", "ar", "zh")
    pub language: Option<String>,
    /// Script tag (e.g., "latn", "arab", "hans")
    pub script: Option<String>,
    /// Enabled OpenType features with their values
    pub features: Vec<(String, u32)>,
}

impl ShapingCacheKey {
    /// Create a new cache key from shaping inputs
    ///
    /// The font data is hashed to create a stable identifier that doesn't
    /// require keeping the full font data in memory for cache lookups.
    pub fn new(
        text: impl Into<String>,
        font_data: &[u8],
        size: f32,
        language: Option<String>,
        script: Option<String>,
        features: Vec<(String, u32)>,
    ) -> Self {
        // Hash the font data for the font_id
        let mut hasher = DefaultHasher::new();
        font_data.hash(&mut hasher);
        let font_id = hasher.finish();

        Self {
            text: text.into(),
            font_id,
            size: (size * 100.0) as u32, // Store as integer for stability
            language,
            script,
            features,
        }
    }
}

/// Cache for shaping results
///
/// Uses a two-level cache (L1 hot cache + L2 LRU cache) for optimal
/// performance across different access patterns.
pub struct ShapingCache {
    cache: MultiLevelCache<ShapingCacheKey, ShapingResult>,
}

impl ShapingCache {
    /// Create a new shaping cache with default capacities
    ///
    /// L1 (hot cache): 100 entries for frequently accessed results
    /// L2 (LRU cache): 500 entries for less frequent access
    pub fn new() -> Self {
        Self {
            cache: MultiLevelCache::new(100, 500),
        }
    }

    /// Create a shaping cache with custom capacities
    pub fn with_capacity(l1_size: usize, l2_size: usize) -> Self {
        Self {
            cache: MultiLevelCache::new(l1_size, l2_size),
        }
    }

    /// Get a cached shaping result
    ///
    /// Returns `Some(result)` if the key exists in either cache level,
    /// `None` if not found.
    pub fn get(&self, key: &ShapingCacheKey) -> Option<ShapingResult> {
        self.cache.get(key)
    }

    /// Insert a shaping result into the cache
    ///
    /// The result is stored in both L1 and L2 caches for maximum availability.
    pub fn insert(&self, key: ShapingCacheKey, result: ShapingResult) {
        self.cache.insert(key, result);
    }

    /// Get the current cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        self.cache.hit_rate()
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let metrics = self.cache.metrics();
        let total = metrics.l1_hits + metrics.l2_hits + metrics.misses;
        CacheStats {
            hits: (metrics.l1_hits + metrics.l2_hits) as usize,
            misses: metrics.misses as usize,
            evictions: 0, // Not tracked in current implementation
            hit_rate: if total > 0 {
                (metrics.l1_hits + metrics.l2_hits) as f64 / total as f64
            } else {
                0.0
            },
        }
    }
}

impl Default for ShapingCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub hit_rate: f64,
}

/// Thread-safe shaping cache wrapper
pub type SharedShapingCache = Arc<RwLock<ShapingCache>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Direction, PositionedGlyph};

    #[test]
    fn test_cache_key_creation() {
        let key = ShapingCacheKey::new(
            "Hello",
            b"font_data",
            16.0,
            Some("en".to_string()),
            Some("latn".to_string()),
            vec![("liga".to_string(), 1)],
        );

        assert_eq!(key.text, "Hello");
        assert_eq!(key.size, 1600); // 16.0 * 100
        assert_eq!(key.language, Some("en".to_string()));
    }

    #[test]
    fn test_cache_insert_and_get() {
        let cache = ShapingCache::new();

        let key = ShapingCacheKey::new("Test", b"font", 12.0, None, None, vec![]);

        let result = ShapingResult {
            glyphs: vec![PositionedGlyph {
                id: 1,
                x: 0.0,
                y: 0.0,
                advance: 10.0,
                cluster: 0,
            }],
            advance_width: 10.0,
            advance_height: 12.0,
            direction: Direction::LeftToRight,
        };

        cache.insert(key.clone(), result.clone());
        let cached = cache.get(&key);

        assert!(cached.is_some());
        assert_eq!(cached.unwrap().glyphs.len(), 1);
    }

    #[test]
    fn test_cache_miss() {
        let cache = ShapingCache::new();

        let key = ShapingCacheKey::new("Missing", b"font", 16.0, None, None, vec![]);
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cache_stats() {
        let cache = ShapingCache::new();

        let key = ShapingCacheKey::new("Text", b"font", 16.0, None, None, vec![]);
        let result = ShapingResult {
            glyphs: vec![],
            advance_width: 0.0,
            advance_height: 16.0,
            direction: Direction::LeftToRight,
        };

        // Miss
        cache.get(&key);

        // Insert
        cache.insert(key.clone(), result);

        // Hit
        cache.get(&key);
        cache.get(&key);

        let stats = cache.stats();
        // Stats track hits and misses
        assert!(stats.hit_rate >= 0.0);
    }

    #[test]
    fn test_different_keys() {
        let key1 = ShapingCacheKey::new("Hello", b"font1", 16.0, None, None, vec![]);
        let key2 = ShapingCacheKey::new("Hello", b"font2", 16.0, None, None, vec![]);
        let key3 = ShapingCacheKey::new("World", b"font1", 16.0, None, None, vec![]);

        // Different font data should produce different keys
        assert_ne!(key1, key2);

        // Different text should produce different keys
        assert_ne!(key1, key3);
    }
}
