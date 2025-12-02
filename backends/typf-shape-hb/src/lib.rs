//! Where text gets professionally shaped: HarfBuzz backend
//!
//! HarfBuzz is the gold standard for OpenType text shaping. It understands
//! Arabic joins, Devanagari conjuncts, Thai vowel positioning, and all the
//! complex ways that characters turn into glyphs. This is the shaper you want
//! for real-world text in any language.

use std::str::FromStr;
use std::sync::Arc;

use harfbuzz_rs::{Direction as HbDirection, Face, Feature, Font as HbFont, Tag, UnicodeBuffer};

use typf_core::{
    error::Result,
    traits::{FontRef, Shaper, Stage},
    types::{Direction, PositionedGlyph, ShapingResult},
    ShapingParams,
};

// Re-export shared shaping cache from typf-core
pub use typf_core::shaping_cache::{CacheStats, ShapingCache, ShapingCacheKey, SharedShapingCache};

/// Professional text shaping powered by HarfBuzz
///
/// Optionally caches shaping results to avoid expensive re-shaping of identical text.
pub struct HarfBuzzShaper {
    /// Optional shaping cache for performance
    cache: Option<SharedShapingCache>,
}

impl HarfBuzzShaper {
    /// Creates a new HarfBuzz shaper ready to handle any script
    pub fn new() -> Self {
        Self { cache: None }
    }

    /// Creates a new HarfBuzz shaper with caching enabled
    ///
    /// Uses default cache capacities (L1: 100, L2: 500 entries)
    pub fn with_cache() -> Self {
        Self {
            cache: Some(Arc::new(std::sync::RwLock::new(ShapingCache::new()))),
        }
    }

    /// Creates a new HarfBuzz shaper with a custom cache
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

    /// Translates our direction enum to HarfBuzz's format
    fn to_hb_direction(dir: Direction) -> HbDirection {
        match dir {
            Direction::LeftToRight => HbDirection::Ltr,
            Direction::RightToLeft => HbDirection::Rtl,
            Direction::TopToBottom => HbDirection::Ttb,
            Direction::BottomToTop => HbDirection::Btt,
        }
    }
}

impl Default for HarfBuzzShaper {
    fn default() -> Self {
        Self::new()
    }
}

impl Stage for HarfBuzzShaper {
    fn name(&self) -> &'static str {
        "HarfBuzz"
    }

    fn process(
        &self,
        ctx: typf_core::context::PipelineContext,
    ) -> Result<typf_core::context::PipelineContext> {
        // HarfBuzz doesn't process pipeline context directly
        Ok(ctx)
    }
}

impl Shaper for HarfBuzzShaper {
    fn name(&self) -> &'static str {
        "HarfBuzz"
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

        // Try to get the actual font data
        let font_data = font.data();

        // Check cache if enabled
        let cache_key = if self.cache.is_some() {
            let key = ShapingCacheKey::new(
                text,
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
            // No font data? Fall back to basic shaping
            let mut glyphs = Vec::new();
            let mut x_offset = 0.0;

            for ch in text.chars() {
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

        // Load the font into HarfBuzz
        let face = Face::from_bytes(font_data, 0);
        let mut hb_font = HbFont::new(face);

        // HarfBuzz uses 26.6 fixed-point for coordinates
        let scale = (params.size * 64.0) as i32; // 64 units per point
        hb_font.set_scale(scale, scale);

        // Apply variable font coordinates (weight, width, optical size, etc.)
        if !params.variations.is_empty() {
            let variations: Vec<harfbuzz_rs::Variation> = params
                .variations
                .iter()
                .filter_map(|(tag_str, value)| {
                    if tag_str.len() == 4 {
                        let bytes = tag_str.as_bytes();
                        let tag = Tag::new(
                            bytes[0] as char,
                            bytes[1] as char,
                            bytes[2] as char,
                            bytes[3] as char,
                        );
                        Some(harfbuzz_rs::Variation::new(tag, *value))
                    } else {
                        None
                    }
                })
                .collect();
            hb_font.set_variations(&variations);
        }

        // Set up the text buffer with all our parameters
        let mut buffer = UnicodeBuffer::new()
            .add_str(text)
            .set_direction(Self::to_hb_direction(params.direction));

        // Tell HarfBuzz which language rules to use
        if let Some(ref lang) = params.language {
            if let Ok(language) = harfbuzz_rs::Language::from_str(lang) {
                buffer = buffer.set_language(language);
            }
        }

        // Specify the script (crucial for languages like Arabic, Devanagari)
        if let Some(ref script_str) = params.script {
            if script_str.len() == 4 {
                let bytes = script_str.as_bytes();
                let tag = Tag::new(
                    bytes[0] as char,
                    bytes[1] as char,
                    bytes[2] as char,
                    bytes[3] as char,
                );
                buffer = buffer.set_script(tag);
            }
        }

        // Convert OpenType features (liga, kern, etc.) to HarfBuzz format
        let hb_features: Vec<Feature> = params
            .features
            .iter()
            .filter_map(|(name, value)| {
                if name.len() == 4 {
                    let bytes = name.as_bytes();
                    let tag = Tag::new(
                        bytes[0] as char,
                        bytes[1] as char,
                        bytes[2] as char,
                        bytes[3] as char,
                    );
                    Some(Feature::new(tag, *value, 0..text.len()))
                } else {
                    None
                }
            })
            .collect();

        // Let HarfBuzz work its magic
        let output = harfbuzz_rs::shape(&hb_font, buffer, &hb_features);

        // Pull out the positioned glyphs HarfBuzz created
        let mut glyphs = Vec::new();
        let mut x_offset = 0.0;

        let positions = output.get_glyph_positions();
        let infos = output.get_glyph_infos();

        for (info, pos) in infos.iter().zip(positions.iter()) {
            glyphs.push(PositionedGlyph {
                id: info.codepoint,
                x: x_offset + (pos.x_offset as f32 / 64.0),
                y: pos.y_offset as f32 / 64.0,
                advance: pos.x_advance as f32 / 64.0,
                cluster: info.cluster,
            });

            x_offset += pos.x_advance as f32 / 64.0;
        }

        let advance_width = x_offset;
        let advance_height = params.size;

        let result = ShapingResult {
            glyphs,
            advance_width,
            advance_height,
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

    fn supports_script(&self, _script: &str) -> bool {
        // HarfBuzz knows how to shape every script there is
        true
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
    fn test_empty_text() {
        let shaper = HarfBuzzShaper::new();
        let font = Arc::new(TestFont { data: vec![] });
        let params = ShapingParams::default();

        let result = shaper.shape("", font, &params).unwrap();
        assert_eq!(result.glyphs.len(), 0);
        assert_eq!(result.advance_width, 0.0);
    }

    #[test]
    fn test_simple_text_no_font_data() {
        let shaper = HarfBuzzShaper::new();
        let font = Arc::new(TestFont { data: vec![] });
        let params = ShapingParams::default();

        let result = shaper.shape("Hi", font, &params).unwrap();
        assert_eq!(result.glyphs.len(), 2);
        assert!(result.advance_width > 0.0);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_with_system_font() {
        use std::fs;

        // Try to load Helvetica system font on macOS
        let font_path = "/System/Library/Fonts/Helvetica.ttc";
        if let Ok(font_data) = fs::read(font_path) {
            let font = Arc::new(TestFont { data: font_data });
            let shaper = HarfBuzzShaper::new();
            let params = ShapingParams::default();

            let result = shaper.shape("Hello, World!", font, &params);
            assert!(result.is_ok());

            let shaped = result.unwrap();
            // Helvetica should shape "Hello, World!" to multiple glyphs
            assert!(shaped.glyphs.len() > 10);
            assert!(shaped.advance_width > 0.0);

            // Check that glyphs have valid IDs
            for glyph in &shaped.glyphs {
                assert!(glyph.id > 0);
                assert!(glyph.advance > 0.0);
            }
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_with_system_font_linux() {
        use std::fs;

        // Try common Linux font paths
        let font_paths = vec![
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
        ];

        for font_path in font_paths {
            if let Ok(font_data) = fs::read(font_path) {
                let font = Arc::new(TestFont { data: font_data });
                let shaper = HarfBuzzShaper::new();
                let params = ShapingParams::default();

                let result = shaper.shape("Test", font, &params);
                assert!(result.is_ok());

                let shaped = result.unwrap();
                assert_eq!(shaped.glyphs.len(), 4); // "Test" = 4 chars
                assert!(shaped.advance_width > 0.0);
                return; // Success with first available font
            }
        }
    }

    #[test]
    fn test_complex_text_shaping() {
        let shaper = HarfBuzzShaper::new();
        let font = Arc::new(TestFont { data: vec![] });

        // Test with various text directions
        let ltr_params = ShapingParams {
            direction: Direction::LeftToRight,
            ..Default::default()
        };

        let rtl_params = ShapingParams {
            direction: Direction::RightToLeft,
            ..Default::default()
        };

        // LTR text
        let ltr_result = shaper.shape("abc", font.clone(), &ltr_params).unwrap();
        assert_eq!(ltr_result.direction, Direction::LeftToRight);
        assert_eq!(ltr_result.glyphs.len(), 3);

        // RTL text (simulated)
        let rtl_result = shaper.shape("abc", font, &rtl_params).unwrap();
        assert_eq!(rtl_result.direction, Direction::RightToLeft);
        assert_eq!(rtl_result.glyphs.len(), 3);
    }

    #[test]
    fn test_font_size_variations() {
        let shaper = HarfBuzzShaper::new();
        let font = Arc::new(TestFont { data: vec![] });

        let text = "M"; // Use 'M' for consistent width testing

        // Test different font sizes
        for size in [12.0, 24.0, 48.0] {
            let params = ShapingParams {
                size,
                ..Default::default()
            };

            let result = shaper.shape(text, font.clone(), &params).unwrap();
            assert_eq!(result.glyphs.len(), 1);
            assert_eq!(result.advance_height, size);
        }
    }

    #[test]
    fn test_opentype_features() {
        let shaper = HarfBuzzShaper::new();
        let font = Arc::new(TestFont { data: vec![] });

        // Test with ligature feature
        let params_liga = ShapingParams {
            features: vec![("liga".to_string(), 1)],
            ..Default::default()
        };

        let result = shaper.shape("fi", font.clone(), &params_liga).unwrap();
        assert_eq!(result.glyphs.len(), 2); // Without real font, won't form ligature

        // Test with kerning feature
        let params_kern = ShapingParams {
            features: vec![("kern".to_string(), 1)],
            ..Default::default()
        };

        let result = shaper.shape("AV", font.clone(), &params_kern).unwrap();
        assert_eq!(result.glyphs.len(), 2);

        // Test with multiple features
        let params_multi = ShapingParams {
            features: vec![
                ("liga".to_string(), 1),
                ("kern".to_string(), 1),
                ("smcp".to_string(), 1), // Small caps
            ],
            ..Default::default()
        };

        let result = shaper.shape("Test", font, &params_multi).unwrap();
        assert_eq!(result.glyphs.len(), 4);
    }

    #[test]
    fn test_language_and_script() {
        let shaper = HarfBuzzShaper::new();
        let font = Arc::new(TestFont { data: vec![] });

        // Test with language set
        let params_lang = ShapingParams {
            language: Some("en".to_string()),
            ..Default::default()
        };

        let result = shaper.shape("Hello", font.clone(), &params_lang).unwrap();
        assert_eq!(result.glyphs.len(), 5);

        // Test with script set
        let params_script = ShapingParams {
            script: Some("latn".to_string()),
            ..Default::default()
        };

        let result = shaper.shape("Test", font.clone(), &params_script).unwrap();
        assert_eq!(result.glyphs.len(), 4);

        // Test with both language and script
        let params_both = ShapingParams {
            language: Some("ar".to_string()),
            script: Some("arab".to_string()),
            ..Default::default()
        };

        let result = shaper.shape("text", font, &params_both).unwrap();
        assert!(result.glyphs.len() > 0);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_features_with_real_font() {
        use std::fs;

        let font_path = "/System/Library/Fonts/Helvetica.ttc";
        if let Ok(font_data) = fs::read(font_path) {
            let font = Arc::new(TestFont { data: font_data });
            let shaper = HarfBuzzShaper::new();

            // Test ligature processing with real font
            let params_no_liga = ShapingParams {
                features: vec![("liga".to_string(), 0)], // Disable ligatures
                ..Default::default()
            };

            let result_no_liga = shaper
                .shape("fi fl", font.clone(), &params_no_liga)
                .unwrap();

            let params_liga = ShapingParams {
                features: vec![("liga".to_string(), 1)], // Enable ligatures
                ..Default::default()
            };

            let result_liga = shaper.shape("fi fl", font, &params_liga).unwrap();

            // Both should have glyphs (actual ligature formation depends on font)
            assert!(result_no_liga.glyphs.len() > 0);
            assert!(result_liga.glyphs.len() > 0);
        }
    }

    #[test]
    fn test_arabic_shaping() {
        let shaper = HarfBuzzShaper::new();
        let font = Arc::new(TestFont { data: vec![] });

        // Test Arabic text with proper script and direction
        let params = ShapingParams {
            language: Some("ar".to_string()),
            script: Some("arab".to_string()),
            direction: Direction::RightToLeft,
            ..Default::default()
        };

        // "Hello" in Arabic (مرحبا)
        let result = shaper.shape("مرحبا", font, &params).unwrap();
        assert_eq!(result.direction, Direction::RightToLeft);
        assert!(result.glyphs.len() > 0);
        // Arabic has contextual forms, so glyph count may differ from char count
        assert!(result.advance_width > 0.0);
    }

    #[test]
    fn test_devanagari_shaping() {
        let shaper = HarfBuzzShaper::new();
        let font = Arc::new(TestFont { data: vec![] });

        // Test Devanagari text with proper script
        let params = ShapingParams {
            language: Some("hi".to_string()),
            script: Some("deva".to_string()),
            direction: Direction::LeftToRight,
            ..Default::default()
        };

        // "Namaste" in Devanagari (नमस्ते)
        let result = shaper.shape("नमस्ते", font, &params).unwrap();
        assert_eq!(result.direction, Direction::LeftToRight);
        assert!(result.glyphs.len() > 0);
        // Devanagari has complex shaping with conjuncts and vowel marks
        assert!(result.advance_width > 0.0);
    }

    #[test]
    fn test_hebrew_shaping() {
        let shaper = HarfBuzzShaper::new();
        let font = Arc::new(TestFont { data: vec![] });

        // Test Hebrew text
        let params = ShapingParams {
            language: Some("he".to_string()),
            script: Some("hebr".to_string()),
            direction: Direction::RightToLeft,
            ..Default::default()
        };

        // "Shalom" in Hebrew (שלום)
        let result = shaper.shape("שלום", font, &params).unwrap();
        assert_eq!(result.direction, Direction::RightToLeft);
        assert_eq!(result.glyphs.len(), 4); // Hebrew doesn't join like Arabic
        assert!(result.advance_width > 0.0);
    }

    #[test]
    fn test_thai_shaping() {
        let shaper = HarfBuzzShaper::new();
        let font = Arc::new(TestFont { data: vec![] });

        // Test Thai text
        let params = ShapingParams {
            language: Some("th".to_string()),
            script: Some("thai".to_string()),
            ..Default::default()
        };

        // "Hello" in Thai (สวัสดี)
        let result = shaper.shape("สวัสดี", font, &params).unwrap();
        assert_eq!(result.direction, Direction::LeftToRight);
        assert!(result.glyphs.len() > 0);
        // Thai has complex vowel and tone mark positioning
        assert!(result.advance_width > 0.0);
    }

    #[test]
    fn test_cjk_shaping() {
        let shaper = HarfBuzzShaper::new();
        let font = Arc::new(TestFont { data: vec![] });

        // Test Chinese text
        let params = ShapingParams {
            language: Some("zh".to_string()),
            script: Some("hani".to_string()),
            ..Default::default()
        };

        // "Hello" in Chinese (你好)
        let result = shaper.shape("你好", font.clone(), &params).unwrap();
        assert_eq!(result.direction, Direction::LeftToRight);
        assert_eq!(result.glyphs.len(), 2); // CJK is mostly 1:1
        assert!(result.advance_width > 0.0);

        // Test Japanese (same script, different language)
        let params_ja = ShapingParams {
            language: Some("ja".to_string()),
            script: Some("hani".to_string()),
            ..Default::default()
        };

        // "Konnichiwa" in hiragana (こんにちは)
        let result = shaper.shape("こんにちは", font, &params_ja).unwrap();
        assert_eq!(result.glyphs.len(), 5);
        assert!(result.advance_width > 0.0);
    }

    #[test]
    fn test_mixed_script_text() {
        let shaper = HarfBuzzShaper::new();
        let font = Arc::new(TestFont { data: vec![] });

        // Test text with Latin + Arabic
        let params = ShapingParams {
            direction: Direction::LeftToRight, // Base direction
            ..Default::default()
        };

        let result = shaper.shape("Hello مرحبا World", font, &params).unwrap();
        assert!(result.glyphs.len() > 0);
        // HarfBuzz handles bidi internally
        assert!(result.advance_width > 0.0);
    }

    // ===================== CACHE TESTS =====================

    #[test]
    fn test_shaper_with_cache() {
        let shaper = HarfBuzzShaper::with_cache();
        let font = Arc::new(TestFont { data: vec![] });
        let params = ShapingParams::default();

        // First shape - cache miss
        let result1 = shaper.shape("Hello", font.clone(), &params).unwrap();
        assert_eq!(result1.glyphs.len(), 5);

        // Second shape - should hit cache
        let result2 = shaper.shape("Hello", font.clone(), &params).unwrap();
        assert_eq!(result2.glyphs.len(), 5);

        // Results should be identical
        assert_eq!(result1.advance_width, result2.advance_width);

        // Check cache hit rate (should be > 0 after second call)
        let hit_rate = shaper.cache_hit_rate().unwrap();
        assert!(
            hit_rate > 0.0,
            "Cache hit rate should be > 0 after repeat query"
        );
    }

    #[test]
    fn test_shaper_without_cache() {
        let shaper = HarfBuzzShaper::new();

        // Cache stats should be None when caching is disabled
        assert!(shaper.cache_stats().is_none());
        assert!(shaper.cache_hit_rate().is_none());
    }

    #[test]
    fn test_cache_stats() {
        let shaper = HarfBuzzShaper::with_cache();
        let font = Arc::new(TestFont { data: vec![] });
        let params = ShapingParams::default();

        // Initial state - no hits or misses
        let stats = shaper.cache_stats().unwrap();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);

        // First query - miss
        shaper.shape("Test", font.clone(), &params).unwrap();

        // Second query (same text) - hit
        shaper.shape("Test", font.clone(), &params).unwrap();

        let stats = shaper.cache_stats().unwrap();
        assert!(stats.hits >= 1, "Should have at least one hit");
    }

    #[test]
    fn test_shared_cache_across_shapers() {
        use std::sync::RwLock;

        // Create a shared cache
        let shared_cache: SharedShapingCache = Arc::new(RwLock::new(ShapingCache::new()));

        // Create two shapers sharing the same cache
        let shaper1 = HarfBuzzShaper::with_shared_cache(shared_cache.clone());
        let shaper2 = HarfBuzzShaper::with_shared_cache(shared_cache.clone());

        let font = Arc::new(TestFont { data: vec![] });
        let params = ShapingParams::default();

        // Shape with shaper1
        let result1 = shaper1.shape("Shared", font.clone(), &params).unwrap();

        // Shape same text with shaper2 - should hit shared cache
        let result2 = shaper2.shape("Shared", font.clone(), &params).unwrap();

        // Results should be identical
        assert_eq!(result1.glyphs.len(), result2.glyphs.len());
        assert_eq!(result1.advance_width, result2.advance_width);

        // Shared cache should have hits
        let shared_stats = shared_cache.read().unwrap().stats();
        assert!(
            shared_stats.hits >= 1,
            "Shared cache should have at least one hit"
        );
    }

    #[test]
    fn test_clear_cache() {
        let shaper = HarfBuzzShaper::with_cache();
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
    fn test_cache_different_params() {
        let shaper = HarfBuzzShaper::with_cache();
        let font = Arc::new(TestFont { data: vec![] });

        let params1 = ShapingParams {
            size: 12.0,
            ..Default::default()
        };

        let params2 = ShapingParams {
            size: 24.0,
            ..Default::default()
        };

        // Same text, different sizes should be cached separately
        let result1 = shaper.shape("Size", font.clone(), &params1).unwrap();
        let result2 = shaper.shape("Size", font.clone(), &params2).unwrap();

        // With fallback shaping (no font data), advance_height reflects size
        assert_eq!(result1.advance_height, 12.0);
        assert_eq!(result2.advance_height, 24.0);

        // Both should be cache misses (different cache keys)
        let stats = shaper.cache_stats().unwrap();
        assert!(stats.misses >= 2);
    }
}
