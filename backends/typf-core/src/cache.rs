// this_file: backends/typf-core/src/cache.rs

//! Font caching infrastructure for efficient font management.

use crate::{Result, ShapingResult, TypfError};
use dashmap::DashMap;
use lru::LruCache;
use memmap2::Mmap;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Configurable cache limits for font-related resources.
#[derive(Debug, Clone, Copy)]
pub struct FontCacheConfig {
    max_fonts: usize,
    max_glyphs: usize,
    max_shapes: usize,
}

impl FontCacheConfig {
    /// Build a config with default production-friendly limits.
    pub const fn new() -> Self {
        Self {
            max_fonts: 512,
            max_glyphs: 2048,
            max_shapes: 512,
        }
    }

    /// Override the maximum number of cached fonts (0 disables font caching).
    pub const fn with_max_fonts(mut self, max_fonts: usize) -> Self {
        self.max_fonts = max_fonts;
        self
    }

    /// Override the maximum number of cached glyph bitmaps (0 disables glyph caching).
    pub const fn with_max_glyphs(mut self, max_glyphs: usize) -> Self {
        self.max_glyphs = max_glyphs;
        self
    }

    /// Override the maximum number of cached shaping results (0 disables shape caching).
    pub const fn with_max_shapes(mut self, max_shapes: usize) -> Self {
        self.max_shapes = max_shapes;
        self
    }

    /// Helper for callers that need to set all limits at once.
    pub const fn with_limits(
        mut self,
        max_fonts: usize,
        max_glyphs: usize,
        max_shapes: usize,
    ) -> Self {
        self.max_fonts = max_fonts;
        self.max_glyphs = max_glyphs;
        self.max_shapes = max_shapes;
        self
    }

    /// Fetch the configured maximum number of cached fonts.
    pub const fn max_fonts(&self) -> usize {
        self.max_fonts
    }

    /// Fetch the configured maximum number of cached glyphs.
    pub const fn max_glyphs(&self) -> usize {
        self.max_glyphs
    }

    /// Fetch the configured maximum number of cached shaping results.
    pub const fn max_shapes(&self) -> usize {
        self.max_shapes
    }
}

impl Default for FontCacheConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Key for font lookups
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FontKey {
    /// Absolute font path for cache lookups.
    pub path: PathBuf,
    /// Face index within a font collection.
    pub face_index: u32,
}

/// Key for shape cache lookups
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShapeKey {
    /// Text that produced the shaping result.
    pub text: String,
    /// Font identity (path + face).
    pub font_key: FontKey,
    /// Quantized font size used for shaping.
    pub size: u32,
    /// OpenType feature toggles applied during shaping.
    pub features: Vec<(String, bool)>,
}

/// Key for glyph cache lookups
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GlyphKey {
    /// Font identity (path + face).
    pub font_key: FontKey,
    /// Glyph identifier from the font.
    pub glyph_id: u32,
    /// Quantized glyph size.
    pub size: u32,
    /// Optional variation coordinates encoded as OpenType tags + bits.
    pub variations: Option<Vec<(String, u32)>>,
}

/// Parsed font face (backend-specific)
pub struct FontFace {
    /// Memory-mapped font bytes.
    pub data: Arc<Mmap>,
    /// Face index from the source collection.
    pub face_index: u32,
    // Backend-specific parsed data would go here
}

/// Rendered glyph
pub struct RenderedGlyph {
    /// Glyph bitmap pixels in RGBA order.
    pub bitmap: Vec<u8>,
    /// Bitmap width in pixels.
    pub width: u32,
    /// Bitmap height in pixels.
    pub height: u32,
    /// Left side bearing in pixels.
    pub left: f32,
    /// Top bearing in pixels.
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
    config: FontCacheConfig,
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
    /// Build a glyph cache key from the glyph metadata.
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
    /// Create a font cache with a custom shape cache capacity (legacy helper).
    pub fn new(cache_size: usize) -> Self {
        Self::with_config(FontCacheConfig::default().with_max_shapes(cache_size))
    }

    /// Create a font cache with explicit limits for all layers.
    pub fn with_config(config: FontCacheConfig) -> Self {
        let shape_capacity = config.max_shapes().max(1);
        Self {
            config,
            mmap_cache: DashMap::new(),
            face_cache: DashMap::new(),
            shape_cache: ShardedShapeCache::new(shape_capacity),
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
        if self.config.max_fonts() > 0 {
            self.face_cache.insert(key, face.clone());
            enforce_limit(&self.face_cache, self.config.max_fonts());
        }
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
        if self.config.max_fonts() > 0 {
            self.mmap_cache.insert(path.to_owned(), mmap.clone());
            enforce_limit(&self.mmap_cache, self.config.max_fonts());
        }
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
        if self.config.max_shapes() > 0 {
            self.shape_cache.put(key.clone(), shaped.clone());
        }
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
        if self.config.max_glyphs() == 0 {
            return glyph;
        }

        self.glyph_cache.insert(key, glyph.clone());
        enforce_limit(&self.glyph_cache, self.config.max_glyphs());
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
            max_fonts: self.config.max_fonts(),
            max_glyphs: self.config.max_glyphs(),
            max_shapes: self.config.max_shapes(),
        }
    }

    /// Return the currently configured limits (useful for diagnostics).
    pub fn config(&self) -> FontCacheConfig {
        self.config
    }
}

fn enforce_limit<K, V>(map: &DashMap<K, V>, limit: usize)
where
    K: Eq + Hash + Clone,
{
    if limit == 0 {
        return;
    }

    while map.len() > limit {
        if let Some(entry) = map.iter().next() {
            let key = entry.key().clone();
            drop(entry);
            map.remove(&key);
        } else {
            break;
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
    /// Configured maximum number of cached fonts (0 disables caching)
    pub max_fonts: usize,
    /// Configured maximum number of cached glyphs (0 disables caching)
    pub max_glyphs: usize,
    /// Configured maximum number of cached shaping results (0 disables caching)
    pub max_shapes: usize,
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
    use proptest::prelude::*;
    use std::path::PathBuf;

    fn test_font_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata/fonts/NotoSans-Regular.ttf")
    }

    fn fake_font_key() -> FontKey {
        FontKey {
            path: PathBuf::from("/virtual/font.ttf"),
            face_index: 0,
        }
    }

    fn dummy_glyph(id: u32) -> RenderedGlyph {
        RenderedGlyph {
            bitmap: vec![255; 4],
            width: 1,
            height: 1,
            left: id as f32,
            top: 0.0,
        }
    }

    fn dummy_shape_result(label: &str) -> ShapingResult {
        ShapingResult {
            text: label.to_string(),
            glyphs: label
                .chars()
                .enumerate()
                .map(|(idx, _)| Glyph {
                    id: idx as u32,
                    cluster: idx as u32,
                    x: idx as f32,
                    y: 0.0,
                    advance: 10.0,
                })
                .collect(),
            advance: 10.0 * label.len() as f32,
            bbox: BoundingBox {
                x: 0.0,
                y: 0.0,
                width: 10.0 * label.len() as f32,
                height: 10.0,
            },
            font: Some(Font::new("Test", 12.0)),
            direction: Direction::LeftToRight,
        }
    }

    fn make_shape_key(font_key: &FontKey, text: String, size: u32, liga: bool) -> ShapeKey {
        ShapeKey {
            text,
            font_key: font_key.clone(),
            size,
            features: vec![("liga".to_string(), liga)],
        }
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

    proptest! {
        #[test]
        fn glyph_hit_miss_tracking_matches_queries(hit_pattern in proptest::collection::vec(any::<bool>(), 1..64)) {
            let cache = FontCache::new(32);
            let font_key = fake_font_key();

            for glyph_id in 0..8u32 {
                let variations = HashMap::new();
                let key = GlyphKey::new(font_key.clone(), glyph_id, 1024, &variations);
                cache.cache_glyph(key, dummy_glyph(glyph_id));
            }

            let mut expected_hits = 0u64;
            let mut expected_misses = 0u64;

            for (idx, should_hit) in hit_pattern.iter().enumerate() {
                let glyph_id = if *should_hit {
                    (idx as u32) % 8
                } else {
                    10_000 + idx as u32
                };
                let variations = HashMap::new();
                let key = GlyphKey::new(font_key.clone(), glyph_id, 1024, &variations);
                let result = cache.get_glyph(&key);

                if *should_hit {
                    prop_assert!(result.is_some(), "glyph {} should have been cached", glyph_id);
                    expected_hits += 1;
                } else {
                    prop_assert!(result.is_none(), "glyph {} should miss", glyph_id);
                    expected_misses += 1;
                }
            }

            let stats = cache.stats();
            prop_assert_eq!(stats.glyph_hits, expected_hits);
            prop_assert_eq!(stats.glyph_misses, expected_misses);
        }
    }

    proptest! {
        #[test]
        fn shape_hit_miss_tracking_matches_queries(pattern in proptest::collection::vec((any::<bool>(), any::<u16>()), 1..64)) {
            let cache = FontCache::new(64);
            let font_key = fake_font_key();

            let seeded_keys: Vec<ShapeKey> = ["alpha", "beta", "gamma", "delta"]
                .iter()
                .enumerate()
                .map(|(idx, label)| {
                    let key = make_shape_key(
                        &font_key,
                        label.to_string(),
                        900 + idx as u32,
                        idx % 2 == 0,
                    );
                    cache.cache_shaped(key.clone(), dummy_shape_result(label));
                    key
                })
                .collect();

            let mut expected_hits = 0u64;
            let mut expected_misses = 0u64;

            for (idx, (expect_hit, salt)) in pattern.iter().enumerate() {
                if *expect_hit {
                    let key = seeded_keys[idx % seeded_keys.len()].clone();
                    prop_assert!(
                        cache.get_shaped(&key).is_some(),
                        "shape {:?} should be cached",
                        key.text
                    );
                    expected_hits += 1;
                } else {
                    let text = format!("miss-{}-{}", idx, salt);
                    let key = make_shape_key(
                        &font_key,
                        text,
                        800 + (salt % 32) as u32,
                        idx % 2 == 0,
                    );
                    prop_assert!(cache.get_shaped(&key).is_none(), "shape miss should not be cached");
                    expected_misses += 1;
                }
            }

            let stats = cache.stats();
            prop_assert_eq!(stats.shape_hits, expected_hits);
            prop_assert_eq!(stats.shape_misses, expected_misses);
        }
    }

    #[test]
    fn face_cache_respects_max_fonts_limit() {
        let config = FontCacheConfig::new()
            .with_max_fonts(2)
            .with_max_glyphs(8)
            .with_max_shapes(16);
        let cache = FontCache::with_config(config);
        let path = test_font_path();

        for face_index in 0..5 {
            cache
                .get_or_load_font(&path, face_index)
                .expect("fixture font should load");
        }

        let stats = cache.stats();
        assert_eq!(stats.face_count, 2, "face cache should enforce limit");
        assert!(
            stats.mmap_count <= 2,
            "mmap cache should respect same limit"
        );
    }

    #[test]
    fn disabling_font_cache_bypasses_storage() {
        let config = FontCacheConfig::new()
            .with_max_fonts(0)
            .with_max_glyphs(0)
            .with_max_shapes(0);
        let cache = FontCache::with_config(config);
        let path = test_font_path();

        cache
            .get_or_load_font(&path, 0)
            .expect("fixture font should load");
        cache.cache_glyph(
            GlyphKey::new(fake_font_key(), 1, 1024, &HashMap::new()),
            dummy_glyph(1),
        );
        cache.cache_shaped(
            make_shape_key(&fake_font_key(), "noop".into(), 900, true),
            dummy_shape_result("noop"),
        );

        let stats = cache.stats();
        assert_eq!(stats.face_count, 0, "font cache should be disabled");
        assert_eq!(stats.mmap_count, 0, "mmap cache should be disabled");
        assert_eq!(stats.glyph_count, 0, "glyph cache should be disabled");
        assert_eq!(
            stats.shape_count, 0,
            "shape cache should not retain entries"
        );
    }

    #[test]
    fn glyph_cache_respects_capacity() {
        let config = FontCacheConfig::new()
            .with_max_fonts(4)
            .with_max_glyphs(2)
            .with_max_shapes(4);
        let cache = FontCache::with_config(config);
        let font_key = fake_font_key();

        let miss_key = GlyphKey::new(font_key.clone(), 9999, 1024, &HashMap::new());
        let keep_key = GlyphKey::new(font_key.clone(), 1, 1024, &HashMap::new());

        cache.cache_glyph(keep_key.clone(), dummy_glyph(1));
        cache.cache_glyph(
            GlyphKey::new(font_key.clone(), 2, 1024, &HashMap::new()),
            dummy_glyph(2),
        );
        cache.cache_glyph(
            GlyphKey::new(font_key.clone(), 3, 1024, &HashMap::new()),
            dummy_glyph(3),
        );

        let stats = cache.stats();
        assert_eq!(stats.glyph_count, 2, "glyph cache should cap entries");
        assert!(
            cache.get_glyph(&keep_key).is_some(),
            "recent glyph should remain"
        );
        assert!(
            cache.get_glyph(&miss_key).is_none(),
            "missing glyph should not be cached"
        );
    }
}
