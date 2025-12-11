//! Pure Rust text shaping backend using harfrust
//!
//! Harfrust is a pure Rust port of HarfBuzz, providing text shaping without
//! any C dependencies. This makes it ideal for environments where compiling
//! HarfBuzz is difficult or when a fully auditable Rust dependency tree is
//! required.
//!
//! Performance is within 25% of the C HarfBuzz implementation for most fonts.

use std::str::FromStr;
use std::sync::Arc;

use harfrust::{
    Direction as HrDirection, Feature, FontRef as HrFontRef, GlyphBuffer, Language, Script,
    ShaperData, ShaperInstance, Tag, UnicodeBuffer, Variation,
};

use typf_core::{
    error::Result,
    traits::{FontRef, Shaper, Stage},
    types::{Direction, PositionedGlyph, ShapingResult},
    ShapingParams,
};

// Re-export shared shaping cache from typf-core
pub use typf_core::shaping_cache::{CacheStats, ShapingCache, ShapingCacheKey, SharedShapingCache};

/// Pure Rust text shaping powered by harfrust
///
/// Optionally caches shaping results to avoid expensive re-shaping of identical text.
pub struct HarfrustShaper {
    /// Optional shaping cache for performance
    cache: Option<SharedShapingCache>,
}

impl HarfrustShaper {
    /// Creates a new harfrust shaper ready to handle any script
    pub fn new() -> Self {
        Self { cache: None }
    }

    /// Creates a new harfrust shaper with caching enabled
    ///
    /// Uses default cache capacities (L1: 100, L2: 500 entries)
    pub fn with_cache() -> Self {
        Self {
            cache: Some(Arc::new(std::sync::RwLock::new(ShapingCache::new()))),
        }
    }

    /// Creates a new harfrust shaper with a custom cache
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

    /// Translates our direction enum to harfrust's format
    fn to_hr_direction(dir: Direction) -> HrDirection {
        match dir {
            Direction::LeftToRight => HrDirection::LeftToRight,
            Direction::RightToLeft => HrDirection::RightToLeft,
            Direction::TopToBottom => HrDirection::TopToBottom,
            Direction::BottomToTop => HrDirection::BottomToTop,
        }
    }

    /// Parse a 4-character tag string into a harfrust Tag
    fn parse_tag(tag_str: &str) -> Option<Tag> {
        if tag_str.len() == 4 {
            let bytes = tag_str.as_bytes();
            Some(Tag::new(&[bytes[0], bytes[1], bytes[2], bytes[3]]))
        } else {
            None
        }
    }

    /// Perform basic fallback shaping when font data is unavailable
    fn fallback_shape(
        &self,
        text: &str,
        font: Arc<dyn FontRef>,
        params: &ShapingParams,
    ) -> ShapingResult {
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

        ShapingResult {
            glyphs,
            advance_width: x_offset,
            advance_height: params.size,
            direction: params.direction,
        }
    }

    /// Extract positioned glyphs from harfrust's GlyphBuffer
    fn extract_glyphs(buffer: &GlyphBuffer, ppem: f32, upem: u16) -> (Vec<PositionedGlyph>, f32) {
        let mut glyphs = Vec::new();
        let mut x_offset = 0.0;
        let scale = ppem / upem as f32;

        let positions = buffer.glyph_positions();
        let infos = buffer.glyph_infos();

        for (info, pos) in infos.iter().zip(positions.iter()) {
            glyphs.push(PositionedGlyph {
                id: info.glyph_id,
                x: x_offset + (pos.x_offset as f32 * scale),
                y: pos.y_offset as f32 * scale,
                advance: pos.x_advance as f32 * scale,
                cluster: info.cluster,
            });

            x_offset += pos.x_advance as f32 * scale;
        }

        (glyphs, x_offset)
    }
}

impl Default for HarfrustShaper {
    fn default() -> Self {
        Self::new()
    }
}

impl Stage for HarfrustShaper {
    fn name(&self) -> &'static str {
        "Harfrust"
    }

    fn process(
        &self,
        ctx: typf_core::context::PipelineContext,
    ) -> Result<typf_core::context::PipelineContext> {
        // Harfrust doesn't process pipeline context directly
        Ok(ctx)
    }
}

impl Shaper for HarfrustShaper {
    fn name(&self) -> &'static str {
        "Harfrust"
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
                Shaper::name(self),
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
            let result = self.fallback_shape(text, font, params);

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

        // Create harfrust FontRef from font data
        let hr_font = match HrFontRef::new(font_data) {
            Ok(f) => f,
            Err(_) => {
                // Font data couldn't be parsed, fall back to basic shaping
                let result = self.fallback_shape(text, font.clone(), params);
                if let Some(key) = cache_key {
                    if let Some(ref cache) = self.cache {
                        if let Ok(cache_guard) = cache.write() {
                            cache_guard.insert(key, result.clone());
                        }
                    }
                }
                return Ok(result);
            }
        };

        // Create ShaperData - this caches font tables and is expensive
        let shaper_data = ShaperData::new(&hr_font);

        // Build variation instance if we have variations
        let instance = if !params.variations.is_empty() {
            let variations: Vec<Variation> = params
                .variations
                .iter()
                .filter_map(|(tag_str, value)| {
                    Self::parse_tag(tag_str).map(|tag| Variation { tag, value: *value })
                })
                .collect();
            Some(ShaperInstance::from_variations(&hr_font, variations))
        } else {
            None
        };

        // Build the shaper
        let mut builder = shaper_data.shaper(&hr_font);
        if let Some(ref inst) = instance {
            builder = builder.instance(Some(inst));
        }
        builder = builder.point_size(Some(params.size));
        let shaper = builder.build();

        // Create the text buffer
        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(text);
        buffer.set_direction(Self::to_hr_direction(params.direction));

        // Set language if specified
        if let Some(ref lang) = params.language {
            if let Ok(language) = Language::from_str(lang) {
                buffer.set_language(language);
            }
        }

        // Set script if specified
        if let Some(ref script_str) = params.script {
            if let Some(tag) = Self::parse_tag(script_str) {
                if let Some(script) = Script::from_iso15924_tag(tag) {
                    buffer.set_script(script);
                }
            }
        }

        // Convert OpenType features (liga, kern, etc.) to harfrust format
        let features: Vec<Feature> = params
            .features
            .iter()
            .filter_map(|(name, value)| {
                Self::parse_tag(name).map(|tag| Feature {
                    tag,
                    value: *value,
                    start: 0,
                    end: u32::MAX,
                })
            })
            .collect();

        // Let harfrust work its magic
        let output = shaper.shape(buffer, &features);

        // Extract the positioned glyphs
        let upem = font.units_per_em();
        let (glyphs, advance_width) = Self::extract_glyphs(&output, params.size, upem);

        let result = ShapingResult {
            glyphs,
            advance_width,
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

    fn supports_script(&self, _script: &str) -> bool {
        // Harfrust knows how to shape every script that HarfBuzz supports
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
        let shaper = HarfrustShaper::new();
        let font = Arc::new(TestFont { data: vec![] });
        let params = ShapingParams::default();

        let result = shaper.shape("", font, &params).unwrap();
        assert_eq!(result.glyphs.len(), 0);
        assert_eq!(result.advance_width, 0.0);
    }

    #[test]
    fn test_simple_text_no_font_data() {
        let shaper = HarfrustShaper::new();
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
            let shaper = HarfrustShaper::new();
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
    fn test_complex_text_shaping() {
        let shaper = HarfrustShaper::new();
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
        let shaper = HarfrustShaper::new();
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
        let shaper = HarfrustShaper::new();
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
        let shaper = HarfrustShaper::new();
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
        assert!(!result.glyphs.is_empty());
    }

    // ===================== CACHE TESTS =====================

    #[test]
    fn test_shaper_with_cache() {
        let shaper = HarfrustShaper::with_cache();
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
        let shaper = HarfrustShaper::new();

        // Cache stats should be None when caching is disabled
        assert!(shaper.cache_stats().is_none());
        assert!(shaper.cache_hit_rate().is_none());
    }

    #[test]
    fn test_clear_cache() {
        let shaper = HarfrustShaper::with_cache();
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
}
