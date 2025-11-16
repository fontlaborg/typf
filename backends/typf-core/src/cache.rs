// this_file: backends/typf-core/src/cache.rs

//! Font caching infrastructure for efficient font management.

use crate::{Result, ShapingResult, TypfError};
use dashmap::DashMap;
use lru::LruCache;
use memmap2::Mmap;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs::File;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Key for font lookups
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FontKey {
    pub path: PathBuf,
    pub face_index: u32,
}

/// Key for shape cache lookups
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShapeKey {
    pub text: String,
    pub font_key: FontKey,
    pub size: u32, // Quantized size
    pub features: Vec<(String, bool)>,
}

/// Key for glyph cache lookups
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GlyphKey {
    pub font_key: FontKey,
    pub glyph_id: u32,
    pub size: u32, // Quantized size
    pub variations: Option<Vec<(String, u32)>>,
}

/// Parsed font face (backend-specific)
pub struct FontFace {
    pub data: Arc<Mmap>,
    pub face_index: u32,
    // Backend-specific parsed data would go here
}

/// Rendered glyph
pub struct RenderedGlyph {
    pub bitmap: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub left: f32,
    pub top: f32,
}

/// Number of shards for the shape cache (power of 2 for fast modulo)
const SHAPE_CACHE_SHARDS: usize = 16;

/// Sharded cache for better concurrent access performance
struct ShardedShapeCache {
    shards: Vec<Mutex<LruCache<ShapeKey, Arc<ShapingResult>>>>,
}

impl ShardedShapeCache {
    fn new(total_capacity: usize) -> Self {
        let shard_capacity = (total_capacity / SHAPE_CACHE_SHARDS).max(1);
        let mut shards = Vec::with_capacity(SHAPE_CACHE_SHARDS);

        for _ in 0..SHAPE_CACHE_SHARDS {
            shards.push(Mutex::new(LruCache::new(
                NonZeroUsize::new(shard_capacity).unwrap_or(NonZeroUsize::new(32).unwrap()),
            )));
        }

        Self { shards }
    }

    #[inline]
    fn shard_index(&self, key: &ShapeKey) -> usize {
        // Hash the key to determine which shard to use
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        // Fast modulo using bitwise AND (works because SHARDS is power of 2)
        (hash as usize) & (SHAPE_CACHE_SHARDS - 1)
    }

    fn get(&self, key: &ShapeKey) -> Option<Arc<ShapingResult>> {
        let shard_idx = self.shard_index(key);
        let mut shard = self.shards[shard_idx].lock();
        shard.get(key).cloned()
    }

    fn put(&self, key: ShapeKey, value: Arc<ShapingResult>) {
        let shard_idx = self.shard_index(&key);
        let mut shard = self.shards[shard_idx].lock();
        shard.put(key, value);
    }

    fn clear(&self) {
        for shard in &self.shards {
            shard.lock().clear();
        }
    }

    fn len(&self) -> usize {
        self.shards.iter().map(|shard| shard.lock().len()).sum()
    }
}

/// Font cache for efficient font and glyph management
pub struct FontCache {
    /// Memory-mapped font files
    mmap_cache: DashMap<PathBuf, Arc<Mmap>>,

    /// Parsed font faces
    face_cache: DashMap<FontKey, Arc<FontFace>>,

    /// Shaped text cache (sharded for better parallelism)
    shape_cache: ShardedShapeCache,

    /// Rendered glyph cache
    glyph_cache: DashMap<GlyphKey, Arc<RenderedGlyph>>,

    /// Hit/miss statistics (atomic for lock-free updates)
    glyph_hits: AtomicU64,
    glyph_misses: AtomicU64,
    shape_hits: AtomicU64,
    shape_misses: AtomicU64,
}

impl GlyphKey {
    pub fn new(
        font_key: FontKey,
        glyph_id: u32,
        size: u32,
        variations: &HashMap<String, f32>,
    ) -> Self {
        let variation_entries = if variations.is_empty() {
            None
        } else {
            let mut entries: Vec<_> = variations
                .iter()
                .map(|(tag, value)| (tag.clone(), value.to_bits()))
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            Some(entries)
        };

        Self {
            font_key,
            glyph_id,
            size,
            variations: variation_entries,
        }
    }
}

impl FontCache {
    /// Create a new font cache
    pub fn new(cache_size: usize) -> Self {
        Self {
            mmap_cache: DashMap::new(),
            face_cache: DashMap::new(),
            shape_cache: ShardedShapeCache::new(cache_size),
            glyph_cache: DashMap::new(),
            glyph_hits: AtomicU64::new(0),
            glyph_misses: AtomicU64::new(0),
            shape_hits: AtomicU64::new(0),
            shape_misses: AtomicU64::new(0),
        }
    }

    /// Get or load a font from disk
    pub fn get_or_load_font(&self, path: &Path, face_index: u32) -> Result<Arc<FontFace>> {
        let key = FontKey {
            path: path.to_owned(),
            face_index,
        };

        // Check face cache first
        if let Some(face) = self.face_cache.get(&key) {
            return Ok(face.clone());
        }

        // Get or create memory map
        let mmap = self.get_or_load_mmap(path)?;

        // Create font face
        let face = Arc::new(FontFace {
            data: mmap,
            face_index,
        });

        // Cache and return
        self.face_cache.insert(key, face.clone());
        Ok(face)
    }

    /// Get or create a memory map for a font file
    fn get_or_load_mmap(&self, path: &Path) -> Result<Arc<Mmap>> {
        // Check mmap cache first
        if let Some(mmap) = self.mmap_cache.get(path) {
            return Ok(mmap.clone());
        }

        // Load and memory map the file
        let file = File::open(path).map_err(|e| TypfError::font_load(path.to_owned(), e))?;

        let mmap =
            unsafe { Mmap::map(&file).map_err(|e| TypfError::font_load(path.to_owned(), e))? };

        let mmap = Arc::new(mmap);
        self.mmap_cache.insert(path.to_owned(), mmap.clone());
        Ok(mmap)
    }

    /// Get cached shaped text
    pub fn get_shaped(&self, key: &ShapeKey) -> Option<Arc<ShapingResult>> {
        let result = self.shape_cache.get(key);
        if result.is_some() {
            self.shape_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.shape_misses.fetch_add(1, Ordering::Relaxed);
        }
        result
    }

    /// Cache shaped text
    pub fn cache_shaped(&self, key: ShapeKey, shaped: ShapingResult) -> Arc<ShapingResult> {
        let shaped = Arc::new(shaped);
        self.shape_cache.put(key.clone(), shaped.clone());
        shaped
    }

    /// Get cached glyph
    pub fn get_glyph(&self, key: &GlyphKey) -> Option<Arc<RenderedGlyph>> {
        let result = self.glyph_cache.get(key).map(|g| g.clone());
        if result.is_some() {
            self.glyph_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.glyph_misses.fetch_add(1, Ordering::Relaxed);
        }
        result
    }

    /// Cache rendered glyph
    pub fn cache_glyph(&self, key: GlyphKey, glyph: RenderedGlyph) -> Arc<RenderedGlyph> {
        let glyph = Arc::new(glyph);
        self.glyph_cache.insert(key, glyph.clone());
        glyph
    }

    /// Clear all caches
    pub fn clear(&self) {
        self.mmap_cache.clear();
        self.face_cache.clear();
        self.shape_cache.clear();
        self.glyph_cache.clear();
    }

    /// Returns true when all cache layers are empty.
    pub fn is_empty(&self) -> bool {
        self.mmap_cache.is_empty()
            && self.face_cache.is_empty()
            && self.shape_cache.len() == 0
            && self.glyph_cache.is_empty()
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            mmap_count: self.mmap_cache.len(),
            face_count: self.face_cache.len(),
            shape_count: self.shape_cache.len(),
            glyph_count: self.glyph_cache.len(),
            glyph_hits: self.glyph_hits.load(Ordering::Relaxed),
            glyph_misses: self.glyph_misses.load(Ordering::Relaxed),
            shape_hits: self.shape_hits.load(Ordering::Relaxed),
            shape_misses: self.shape_misses.load(Ordering::Relaxed),
        }
    }
}

/// Cache statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of memory-mapped font files
    pub mmap_count: usize,
    /// Number of parsed font faces
    pub face_count: usize,
    /// Number of cached shaping results
    pub shape_count: usize,
    /// Number of cached rendered glyphs
    pub glyph_count: usize,
    /// Number of glyph cache hits
    pub glyph_hits: u64,
    /// Number of glyph cache misses
    pub glyph_misses: u64,
    /// Number of shape cache hits
    pub shape_hits: u64,
    /// Number of shape cache misses
    pub shape_misses: u64,
}

impl CacheStats {
    /// Returns true if all caches are empty
    pub fn is_empty(&self) -> bool {
        self.mmap_count == 0
            && self.face_count == 0
            && self.shape_count == 0
            && self.glyph_count == 0
    }

    /// Returns total number of cached items across all caches
    pub fn total_items(&self) -> usize {
        self.mmap_count + self.face_count + self.shape_count + self.glyph_count
    }

    /// Returns estimated memory usage in bytes (rough approximation)
    pub fn estimated_memory_bytes(&self) -> usize {
        // Rough estimates:
        // - mmap: ~100KB per font file
        // - face: ~50KB per parsed face
        // - shape: ~1KB per shaping result
        // - glyph: ~5KB per rendered glyph
        (self.mmap_count * 100_000)
            + (self.face_count * 50_000)
            + (self.shape_count * 1_000)
            + (self.glyph_count * 5_000)
    }

    /// Calculate glyph cache hit rate (0.0-1.0)
    pub fn glyph_hit_rate(&self) -> f64 {
        let total = self.glyph_hits + self.glyph_misses;
        if total == 0 {
            0.0
        } else {
            self.glyph_hits as f64 / total as f64
        }
    }

    /// Calculate shape cache hit rate (0.0-1.0)
    pub fn shape_hit_rate(&self) -> f64 {
        let total = self.shape_hits + self.shape_misses;
        if total == 0 {
            0.0
        } else {
            self.shape_hits as f64 / total as f64
        }
    }

    /// Calculate overall cache efficiency score (0.0-1.0)
    ///
    /// Weighted average of hit rates (glyph cache is weighted more heavily)
    pub fn efficiency_score(&self) -> f64 {
        let glyph_rate = self.glyph_hit_rate();
        let shape_rate = self.shape_hit_rate();
        // Weight glyph cache 70%, shape cache 30% (glyphs are more expensive to render)
        glyph_rate * 0.7 + shape_rate * 0.3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BoundingBox, Direction, Font, Glyph};
    use std::path::PathBuf;

    fn test_font_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata/fonts/NotoSans-Regular.ttf")
    }

    #[test]
    fn clear_removes_all_cached_entries() {
        let cache = FontCache::new(8);
        let path = test_font_path();
        cache
            .get_or_load_font(&path, 0)
            .expect("fixture font should load");

        let font_key = FontKey {
            path: path.clone(),
            face_index: 0,
        };
        let glyph_key = GlyphKey::new(font_key.clone(), 42, 1200, &HashMap::new());
        cache.cache_glyph(
            glyph_key,
            RenderedGlyph {
                bitmap: vec![255],
                width: 1,
                height: 1,
                left: 0.0,
                top: 0.0,
            },
        );

        let shape_key = ShapeKey {
            text: "abc".to_string(),
            font_key,
            size: 1200,
            features: vec![("liga".to_string(), true)],
        };
        cache.cache_shaped(
            shape_key,
            ShapingResult {
                text: "abc".to_string(),
                glyphs: vec![Glyph {
                    id: 1,
                    cluster: 0,
                    x: 0.0,
                    y: 0.0,
                    advance: 10.0,
                }],
                advance: 10.0,
                bbox: BoundingBox {
                    x: 0.0,
                    y: 0.0,
                    width: 10.0,
                    height: 10.0,
                },
                font: Some(Font::new("Test", 12.0)),
                direction: Direction::LeftToRight,
            },
        );

        assert!(
            !cache.is_empty(),
            "cache should report entries before clearing: {:?}",
            cache.stats()
        );
        cache.clear();
        assert!(cache.is_empty(), "cache should be empty after clear");
        assert!(cache.stats().is_empty(), "stats should reset after clear");
    }
}
