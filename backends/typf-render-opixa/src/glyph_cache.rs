//! Glyph bitmap cache for efficient repeated rendering
//!
//! When rendering text, the same glyph often appears multiple times. This cache
//! stores rendered glyph bitmaps keyed by (font, glyph_id, size, variations) to
//! avoid redundant rasterization work.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

use crate::rasterizer::GlyphBitmap;

/// Cache key for rendered glyphs
///
/// Uniquely identifies a rendered glyph by its font, glyph ID, size, and variations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlyphCacheKey {
    /// Hash of font data (identifies the font)
    pub font_id: u64,
    /// Glyph ID within the font
    pub glyph_id: u32,
    /// Size in fixed-point (size * 100 for hash stability)
    pub size: u32,
    /// Hash of variation coordinates
    pub variations_hash: u64,
}

impl GlyphCacheKey {
    /// Create a new cache key
    pub fn new(font_data: &[u8], glyph_id: u32, size: f32, variations: &[(String, f32)]) -> Self {
        // Hash font data
        let mut hasher = DefaultHasher::new();
        font_data.hash(&mut hasher);
        let font_id = hasher.finish();

        // Hash variations
        let mut var_hasher = DefaultHasher::new();
        for (tag, val) in variations {
            tag.hash(&mut var_hasher);
            ((val * 1000.0) as i32).hash(&mut var_hasher);
        }
        let variations_hash = var_hasher.finish();

        Self {
            font_id,
            glyph_id,
            size: (size * 100.0) as u32,
            variations_hash,
        }
    }
}

/// LRU-style glyph cache with configurable capacity
pub struct GlyphCache {
    cache: RwLock<HashMap<GlyphCacheKey, GlyphBitmap>>,
    capacity: usize,
    hits: RwLock<u64>,
    misses: RwLock<u64>,
}

impl GlyphCache {
    /// Create a new glyph cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::with_capacity(capacity)),
            capacity,
            hits: RwLock::new(0),
            misses: RwLock::new(0),
        }
    }

    /// Get a cached glyph bitmap if available
    pub fn get(&self, key: &GlyphCacheKey) -> Option<GlyphBitmap> {
        let cache = self.cache.read().ok()?;
        if let Some(bitmap) = cache.get(key) {
            if let Ok(mut hits) = self.hits.write() {
                *hits += 1;
            }
            Some(bitmap.clone())
        } else {
            if let Ok(mut misses) = self.misses.write() {
                *misses += 1;
            }
            None
        }
    }

    /// Insert a glyph bitmap into the cache
    pub fn insert(&self, key: GlyphCacheKey, bitmap: GlyphBitmap) {
        let mut cache = match self.cache.write() {
            Ok(c) => c,
            Err(_) => return,
        };

        // Simple eviction: clear half the cache when full
        if cache.len() >= self.capacity {
            let keys_to_remove: Vec<_> = cache.keys().take(self.capacity / 2).cloned().collect();
            for k in keys_to_remove {
                cache.remove(&k);
            }
        }

        cache.insert(key, bitmap);
    }

    /// Get cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.read().map(|h| *h).unwrap_or(0);
        let misses = self.misses.read().map(|m| *m).unwrap_or(0);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> GlyphCacheStats {
        let cache = self.cache.read().ok();
        GlyphCacheStats {
            size: cache.map(|c| c.len()).unwrap_or(0),
            capacity: self.capacity,
            hits: self.hits.read().map(|h| *h).unwrap_or(0),
            misses: self.misses.read().map(|m| *m).unwrap_or(0),
        }
    }

    /// Clear the cache
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }
}

/// Glyph cache statistics
#[derive(Debug, Clone)]
pub struct GlyphCacheStats {
    pub size: usize,
    pub capacity: usize,
    pub hits: u64,
    pub misses: u64,
}

impl GlyphCacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_creation() {
        let key1 = GlyphCacheKey::new(b"font1", 65, 16.0, &[]);
        let key2 = GlyphCacheKey::new(b"font1", 65, 16.0, &[]);
        let key3 = GlyphCacheKey::new(b"font2", 65, 16.0, &[]);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_variations_affect_key() {
        let key1 = GlyphCacheKey::new(b"font", 65, 16.0, &[("wght".to_string(), 400.0)]);
        let key2 = GlyphCacheKey::new(b"font", 65, 16.0, &[("wght".to_string(), 700.0)]);
        let key3 = GlyphCacheKey::new(b"font", 65, 16.0, &[]);

        assert_ne!(key1, key2, "Different weights should have different keys");
        assert_ne!(key1, key3, "With/without variations should differ");
    }

    #[test]
    fn test_cache_insert_and_get() {
        let cache = GlyphCache::new(100);
        let key = GlyphCacheKey::new(b"font", 65, 16.0, &[]);
        let bitmap = GlyphBitmap {
            width: 10,
            height: 12,
            left: 1,
            top: 10,
            data: vec![128; 120],
        };

        cache.insert(key.clone(), bitmap.clone());
        let cached = cache.get(&key);

        assert!(cached.is_some());
        assert_eq!(cached.unwrap().width, 10);
    }

    #[test]
    fn test_cache_miss() {
        let cache = GlyphCache::new(100);
        let key = GlyphCacheKey::new(b"font", 65, 16.0, &[]);

        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cache_eviction() {
        let cache = GlyphCache::new(3);

        for i in 0..5 {
            let key = GlyphCacheKey::new(b"font", i, 16.0, &[]);
            let bitmap = GlyphBitmap {
                width: 10,
                height: 12,
                left: 1,
                top: 10,
                data: vec![128; 120],
            };
            cache.insert(key, bitmap);
        }

        // Should have evicted some entries
        let stats = cache.stats();
        assert!(stats.size <= 3, "Cache should not exceed capacity");
    }

    #[test]
    fn test_cache_stats() {
        let cache = GlyphCache::new(100);
        let key = GlyphCacheKey::new(b"font", 65, 16.0, &[]);
        let bitmap = GlyphBitmap {
            width: 10,
            height: 12,
            left: 1,
            top: 10,
            data: vec![128; 120],
        };

        // Miss
        cache.get(&key);

        // Insert
        cache.insert(key.clone(), bitmap);

        // Hits
        cache.get(&key);
        cache.get(&key);

        let stats = cache.stats();
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hits, 2);
        assert!(stats.hit_rate() > 0.6);
    }
}
