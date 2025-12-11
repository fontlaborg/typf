//! Backend-neutral glyph/render cache
//!
//! Caches complete `RenderOutput` values keyed by the shaped glyph stream,
//! render parameters, font identity, and renderer backend name. This sits in
//! `typf-core` so every renderer can benefit without bespoke cache logic.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

use crate::cache::MultiLevelCache;
use crate::cache_config;
use crate::types::{RenderOutput, ShapingResult};
use crate::RenderParams;

/// Stable key for render output caching
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlyphCacheKey {
    /// Renderer/backend identity
    pub renderer: String,
    /// Font identity hash
    pub font_id: u64,
    /// Hash of shaped glyph sequence (positions + ids + direction)
    pub shaped_hash: u64,
    /// Hash of render parameters (colors, AA, palette, variations, glyph sources)
    pub render_hash: u64,
}

impl GlyphCacheKey {
    /// Build a key from runtime inputs
    pub fn new(
        renderer: impl Into<String>,
        font_data: &[u8],
        shaped: &ShapingResult,
        render_params: &RenderParams,
    ) -> Self {
        let font_id = hash_bytes(font_data);
        let shaped_hash = hash_shaping_result(shaped);
        let render_hash = hash_render_params(render_params);

        Self {
            renderer: renderer.into(),
            font_id,
            shaped_hash,
            render_hash,
        }
    }
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

fn hash_shaping_result(shaped: &ShapingResult) -> u64 {
    let mut hasher = DefaultHasher::new();

    shaped.direction.hash(&mut hasher);
    shaped.advance_width.to_bits().hash(&mut hasher);
    shaped.advance_height.to_bits().hash(&mut hasher);

    for glyph in &shaped.glyphs {
        glyph.id.hash(&mut hasher);
        glyph.cluster.hash(&mut hasher);
        glyph.x.to_bits().hash(&mut hasher);
        glyph.y.to_bits().hash(&mut hasher);
        glyph.advance.to_bits().hash(&mut hasher);
    }

    hasher.finish()
}

fn hash_render_params(params: &RenderParams) -> u64 {
    let mut hasher = DefaultHasher::new();

    params.padding.hash(&mut hasher);
    params.antialias.hash(&mut hasher);
    params.color_palette.hash(&mut hasher);
    params.output.hash(&mut hasher);
    params.foreground.hash(&mut hasher);
    params.background.hash(&mut hasher);

    for (tag, value) in &params.variations {
        tag.hash(&mut hasher);
        value.to_bits().hash(&mut hasher);
    }

    for source in params.glyph_sources.effective_order() {
        source.hash(&mut hasher);
    }

    let mut denied: Vec<_> = params.glyph_sources.deny.iter().copied().collect();
    denied.sort();
    for deny in denied {
        deny.hash(&mut hasher);
    }

    hasher.finish()
}

/// Render-output cache with two-level hierarchy
pub struct GlyphCache {
    cache: MultiLevelCache<GlyphCacheKey, RenderOutput>,
}

impl GlyphCache {
    pub fn new() -> Self {
        // Render outputs are larger than shaping results; keep cache modest.
        Self {
            cache: MultiLevelCache::new(64, 1024),
        }
    }

    pub fn with_capacity(l1: usize, l2: usize) -> Self {
        Self {
            cache: MultiLevelCache::new(l1, l2),
        }
    }

    /// Get a cached render output
    ///
    /// Returns `None` if not found or if caching is globally disabled.
    pub fn get(&self, key: &GlyphCacheKey) -> Option<RenderOutput> {
        if !cache_config::is_caching_enabled() {
            return None;
        }
        self.cache.get(key)
    }

    /// Insert a render output into the cache
    ///
    /// Does nothing if caching is globally disabled.
    pub fn insert(&self, key: GlyphCacheKey, output: RenderOutput) {
        if !cache_config::is_caching_enabled() {
            return;
        }
        self.cache.insert(key, output);
    }

    pub fn hit_rate(&self) -> f64 {
        self.cache.hit_rate()
    }

    pub fn metrics(&self) -> crate::cache::CacheMetrics {
        self.cache.metrics()
    }
}

impl Default for GlyphCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe shared glyph cache
pub type SharedGlyphCache = Arc<RwLock<GlyphCache>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Direction, PositionedGlyph};

    fn shaped() -> ShapingResult {
        ShapingResult {
            glyphs: vec![PositionedGlyph {
                id: 42,
                x: 1.5,
                y: 0.0,
                advance: 10.0,
                cluster: 0,
            }],
            advance_width: 10.0,
            advance_height: 16.0,
            direction: Direction::LeftToRight,
        }
    }

    fn render_params() -> RenderParams {
        RenderParams::default()
    }

    #[test]
    fn key_changes_with_renderer() {
        let s = shaped();
        let p = render_params();
        let k1 = GlyphCacheKey::new("r1", b"font", &s, &p);
        let k2 = GlyphCacheKey::new("r2", b"font", &s, &p);
        assert_ne!(k1, k2);
    }

    #[test]
    fn cache_stores_and_retrieves() {
        // Enable caching for this test (caching is disabled by default)
        crate::cache_config::set_caching_enabled(true);

        let cache = GlyphCache::new();
        let key = GlyphCacheKey::new("r1", b"font", &shaped(), &render_params());
        let output = RenderOutput::Json("x".into());

        cache.insert(key.clone(), output.clone());
        let hit = cache.get(&key).unwrap();
        match hit {
            RenderOutput::Json(body) => assert_eq!(body, "x"),
            _ => panic!("expected json"),
        }

        // Reset to default state
        crate::cache_config::set_caching_enabled(false);
    }
}
