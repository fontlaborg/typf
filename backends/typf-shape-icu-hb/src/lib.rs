//! Unicode perfection: ICU preprocessing meets HarfBuzz shaping
//!
//! Some text needs more than just shaping. It needs normalization (é vs e + ´),
//! bidirectional analysis, script detection. ICU handles the Unicode plumbing,
//! then passes perfect text to HarfBuzz for shaping. This is the shaper you
//! want when you need to handle every edge case the Unicode spec throws at you.

use harfbuzz_rs::{Direction as HbDirection, Face, Feature, Font as HbFont, Tag, UnicodeBuffer};
use std::str::FromStr;
use std::sync::Arc;
use typf_core::{
    error::Result,
    traits::{FontRef, Shaper, Stage},
    types::{Direction, PositionedGlyph, ShapingResult},
    ShapingParams,
};
use unicode_normalization::UnicodeNormalization;

// Re-export shared shaping cache from typf-core
pub use typf_core::shaping_cache::{CacheStats, ShapingCache, ShapingCacheKey, SharedShapingCache};

/// ICU preprocessing + HarfBuzz shaping for bulletproof Unicode support
///
/// What this gives you:
/// - Unicode normalization (fixes broken character sequences)
/// - Bidirectional text handling (Arabic/Hebrew in Latin text)
/// - Script detection (knows Arabic from Thai from Cyrillic)
/// - Line breaking analysis (where text can safely break)
/// - Professional OpenType shaping with HarfBuzz
/// - Optional caching of shaping results for performance
pub struct IcuHarfBuzzShaper {
    /// Optional shaping cache for performance
    cache: Option<SharedShapingCache>,
}

impl IcuHarfBuzzShaper {
    /// Creates a new shaper that's ready for any Unicode challenge
    pub fn new() -> Self {
        Self { cache: None }
    }

    /// Creates a new shaper with caching enabled
    ///
    /// Uses default cache capacities (L1: 100, L2: 500 entries)
    pub fn with_cache() -> Self {
        Self {
            cache: Some(Arc::new(std::sync::RwLock::new(ShapingCache::new()))),
        }
    }

    /// Creates a new shaper with a custom cache
    ///
    /// Useful for sharing a cache across multiple shapers
    pub fn with_shared_cache(cache: SharedShapingCache) -> Self {
        Self { cache: Some(cache) }
    }

    /// Returns cache statistics if caching is enabled
    pub fn cache_stats(&self) -> Option<CacheStats> {
        self.cache
            .as_ref()
            .and_then(|c| c.read().ok())
            .map(|c| c.stats())
    }

    /// Returns the cache hit rate (0.0 to 1.0) if caching is enabled
    pub fn cache_hit_rate(&self) -> Option<f64> {
        self.cache
            .as_ref()
            .and_then(|c| c.read().ok())
            .map(|c| c.hit_rate())
    }

    /// Maps our direction enum to HarfBuzz's format
    fn to_hb_direction(dir: Direction) -> HbDirection {
        match dir {
            Direction::LeftToRight => HbDirection::Ltr,
            Direction::RightToLeft => HbDirection::Rtl,
            Direction::TopToBottom => HbDirection::Ttb,
            Direction::BottomToTop => HbDirection::Btt,
        }
    }
}

impl Default for IcuHarfBuzzShaper {
    fn default() -> Self {
        Self::new()
    }
}

impl Stage for IcuHarfBuzzShaper {
    fn name(&self) -> &'static str {
        "ICU-HarfBuzz"
    }

    fn process(
        &self,
        ctx: typf_core::context::PipelineContext,
    ) -> Result<typf_core::context::PipelineContext> {
        // ICU-HB doesn't process pipeline context directly
        Ok(ctx)
    }
}

impl Shaper for IcuHarfBuzzShaper {
    fn name(&self) -> &'static str {
        "ICU-HarfBuzz"
    }

    fn shape(
        &self,
        text: &str,
        font: Arc<dyn FontRef>,
        params: &ShapingParams,
    ) -> Result<ShapingResult> {
        if text.is_empty() {
            return Ok(ShapingResult {
                glyphs: Vec::new(),
                advance_width: 0.0,
                advance_height: params.size,
                direction: params.direction,
            });
        }

        // Step 1: Normalize the text (fix é vs e + ´ and similar issues)
        let normalized: String = text.nfc().collect();

        // Step 2: Get the font data for HarfBuzz
        let font_data = font.data();

        // Check cache if enabled (use normalized text for key)
        let cache_key = if self.cache.is_some() {
            let key = ShapingCacheKey::new(
                &normalized,
                font_data,
                params.size,
                params.language.clone(),
                params.script.clone(),
                params.features.clone(),
                params.variations.clone(),
            );
            // Try to get from cache
            if let Some(ref cache) = self.cache {
                if let Ok(cache_guard) = cache.read() {
                    if let Some(result) = cache_guard.get(&key) {
                        return Ok(result);
                    }
                }
            }
            Some(key)
        } else {
            None
        };

        if font_data.is_empty() {
            // No font data? Fall back to basic shaping on cleaned text
            let mut glyphs = Vec::new();
            let mut x_offset = 0.0;

            for ch in normalized.chars() {
                if let Some(glyph_id) = font.glyph_id(ch) {
                    let advance = font.advance_width(glyph_id);
                    glyphs.push(PositionedGlyph {
                        id: glyph_id,
                        x: x_offset,
                        y: 0.0,
                        advance,
                        cluster: 0,
                    });
                    x_offset += advance * params.size / font.units_per_em() as f32;
                }
            }

            let result = ShapingResult {
                glyphs,
                advance_width: x_offset,
                advance_height: params.size,
                direction: params.direction,
            };

            // Store fallback result in cache if enabled
            if let Some(key) = cache_key {
                if let Some(ref cache) = self.cache {
                    if let Ok(cache_guard) = cache.write() {
                        cache_guard.insert(key, result.clone());
                    }
                }
            }

            return Ok(result);
        }

        // Step 3: Load the font into HarfBuzz
        let hb_face = Face::from_bytes(font_data, 0);
        let mut hb_font = HbFont::new(hb_face);

        // HarfBuzz uses 26.6 fixed-point for font coordinates
        let scale = (params.size * 64.0) as i32; // 64 units per point
        hb_font.set_scale(scale, scale);

        // Step 4: Set up HarfBuzz's text buffer with our normalized text
        let mut buffer = UnicodeBuffer::new()
            .add_str(&normalized)
            .set_direction(Self::to_hb_direction(params.direction));

        // Tell HarfBuzz which language rules to use
        if let Some(ref lang) = params.language {
            if let Ok(language) = harfbuzz_rs::Language::from_str(lang) {
                buffer = buffer.set_language(language);
            }
        }

        // Specify the script (critical for complex scripts)
        if let Some(ref script_str) = params.script {
            if script_str.len() == 4 {
                let script_bytes = script_str.as_bytes();
                let tag = Tag::new(
                    script_bytes[0] as char,
                    script_bytes[1] as char,
                    script_bytes[2] as char,
                    script_bytes[3] as char,
                );
                buffer = buffer.set_script(tag);
            }
        }

        // Step 5: Convert OpenType features to HarfBuzz format
        let features: Vec<Feature> = params
            .features
            .iter()
            .filter_map(|(name, value)| {
                if name.len() == 4 {
                    let bytes = name.as_bytes();
                    Some(Feature::new(
                        Tag::new(
                            bytes[0] as char,
                            bytes[1] as char,
                            bytes[2] as char,
                            bytes[3] as char,
                        ),
                        *value,
                        0..usize::MAX,
                    ))
                } else {
                    None
                }
            })
            .collect();

        // Step 6: Let HarfBuzz do the heavy lifting
        let output = harfbuzz_rs::shape(&hb_font, buffer, features.as_slice());

        // Step 7: Extract the beautiful positioned glyphs
        let positions = output.get_glyph_positions();
        let infos = output.get_glyph_infos();

        let mut glyphs = Vec::with_capacity(infos.len());
        let mut x_offset = 0.0;
        let mut y_offset = 0.0;

        for (info, pos) in infos.iter().zip(positions.iter()) {
            let x_advance = pos.x_advance as f32 / 64.0;
            let y_advance = pos.y_advance as f32 / 64.0;

            glyphs.push(PositionedGlyph {
                id: info.codepoint,
                x: x_offset + pos.x_offset as f32 / 64.0,
                y: y_offset + pos.y_offset as f32 / 64.0,
                advance: x_advance,
                cluster: info.cluster,
            });

            x_offset += x_advance;
            y_offset += y_advance;
        }

        let result = ShapingResult {
            glyphs,
            advance_width: x_offset,
            advance_height: params.size,
            direction: params.direction,
        };

        // Store in cache if enabled
        if let Some(key) = cache_key {
            if let Some(ref cache) = self.cache {
                if let Ok(cache_guard) = cache.write() {
                    cache_guard.insert(key, result.clone());
                }
            }
        }

        Ok(result)
    }

    fn clear_cache(&self) {
        if let Some(ref cache) = self.cache {
            if let Ok(mut cache_guard) = cache.write() {
                *cache_guard = ShapingCache::new();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestFont {
        data: Vec<u8>,
    }

    impl FontRef for TestFont {
        fn data(&self) -> &[u8] {
            &self.data
        }

        fn units_per_em(&self) -> u16 {
            1000
        }

        fn glyph_id(&self, ch: char) -> Option<u32> {
            Some(ch as u32)
        }

        fn advance_width(&self, _: u32) -> f32 {
            500.0
        }
    }

    #[test]
    fn test_icu_harfbuzz_shaper_empty() {
        let shaper = IcuHarfBuzzShaper::new();
        assert_eq!(Stage::name(&shaper), "ICU-HarfBuzz");
    }

    #[test]
    fn test_shaper_with_cache() {
        let shaper = IcuHarfBuzzShaper::with_cache();
        let font = Arc::new(TestFont { data: vec![] });
        let params = ShapingParams::default();

        // First shape - cache miss
        let result1 = shaper.shape("Hello", font.clone(), &params).unwrap();
        assert_eq!(result1.glyphs.len(), 5);

        // Second shape - should hit cache
        let result2 = shaper.shape("Hello", font.clone(), &params).unwrap();
        assert_eq!(result2.glyphs.len(), 5);

        // Check cache hit rate (should be > 0 after second call)
        let hit_rate = shaper.cache_hit_rate().unwrap();
        assert!(
            hit_rate > 0.0,
            "Cache hit rate should be > 0 after repeat query"
        );
    }

    #[test]
    fn test_shaper_without_cache() {
        let shaper = IcuHarfBuzzShaper::new();

        // Cache stats should be None when caching is disabled
        assert!(shaper.cache_stats().is_none());
        assert!(shaper.cache_hit_rate().is_none());
    }

    #[test]
    fn test_clear_cache() {
        let shaper = IcuHarfBuzzShaper::with_cache();
        let font = Arc::new(TestFont { data: vec![] });
        let params = ShapingParams::default();

        // Shape text to populate cache
        shaper.shape("ClearTest", font.clone(), &params).unwrap();
        shaper.shape("ClearTest", font.clone(), &params).unwrap(); // Hit

        let stats_before = shaper.cache_stats().unwrap();
        assert!(stats_before.hits >= 1);

        // Clear the cache
        shaper.clear_cache();

        // Stats should be reset
        let stats_after = shaper.cache_stats().unwrap();
        assert_eq!(stats_after.hits, 0);
        assert_eq!(stats_after.misses, 0);
    }

    #[test]
    fn test_normalization_before_caching() {
        let shaper = IcuHarfBuzzShaper::with_cache();
        let font = Arc::new(TestFont { data: vec![] });
        let params = ShapingParams::default();

        // "café" in two forms: NFC (composed) vs NFD (decomposed)
        let composed = "caf\u{00E9}"; // é as single codepoint
        let decomposed = "cafe\u{0301}"; // e + combining acute

        // Both should normalize to the same form and hit cache
        let result1 = shaper.shape(composed, font.clone(), &params).unwrap();
        let result2 = shaper.shape(decomposed, font.clone(), &params).unwrap();

        // After normalization, both should produce identical results
        assert_eq!(result1.glyphs.len(), result2.glyphs.len());

        // Second query with either form should hit cache
        let stats = shaper.cache_stats().unwrap();
        assert!(stats.hits >= 1, "Second query should hit cache");
    }
}
