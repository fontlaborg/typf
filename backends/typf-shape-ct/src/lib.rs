//! macOS-native text shaping with CoreText precision
//!
//! CoreText is Apple's professional text shaping engine, built right into macOS.
//! It understands every script Apple supports, handles variable fonts flawlessly,
//! and integrates perfectly with the system font rendering pipeline. This is
//! the shaper you want on macOS for that native Apple typography feel.

#![cfg(target_os = "macos")]

use std::sync::Arc;
use typf_core::{
    error::{Result, ShapingError, TypfError},
    traits::{FontRef, Shaper},
    types::{PositionedGlyph, ShapingResult},
    ShapingParams,
};

use core_foundation::{
    attributed_string::CFMutableAttributedString,
    base::{CFRange, TCFType},
    dictionary::CFDictionary,
    number::CFNumber,
    string::CFString,
};
use core_graphics::{
    data_provider::CGDataProvider,
    font::CGFont,
    geometry::{CGPoint, CGSize},
};
use core_text::{
    font::{new_from_CGFont, new_from_descriptor, CTFont},
    font_descriptor::CTFontDescriptor,
    line::CTLine,
    run::{CTRun, CTRunRef},
    string_attributes::{kCTFontAttributeName, kCTKernAttributeName, kCTLigatureAttributeName},
};
use lru::LruCache;
use parking_lot::RwLock;

// CoreText needs these for variable font support
#[link(name = "CoreText", kind = "framework")]
extern "C" {
    static kCTFontVariationAttribute: core_foundation::string::CFStringRef;

    fn CTFontDescriptorCreateWithAttributes(
        attributes: core_foundation::dictionary::CFDictionaryRef,
    ) -> core_text::font_descriptor::CTFontDescriptorRef;
}

/// How we identify fonts in our cache
type FontCacheKey = String;

/// How we identify shaping results in our cache
type ShapeCacheKey = String;

/// Font data wrapper that CoreGraphics likes
struct ProviderData {
    bytes: Arc<[u8]>,
}

impl AsRef<[u8]> for ProviderData {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

/// Professional text shaping powered by macOS CoreText
pub struct CoreTextShaper {
    /// Cache fonts to avoid expensive CTFont creation
    font_cache: RwLock<LruCache<FontCacheKey, Arc<CTFont>>>,
    /// Cache shaping results to avoid redundant work
    shape_cache: RwLock<LruCache<ShapeCacheKey, Arc<ShapingResult>>>,
}

impl CoreTextShaper {
    /// Creates a new shaper ready to work with CoreText
    pub fn new() -> Self {
        Self {
            font_cache: RwLock::new(LruCache::new(std::num::NonZeroUsize::new(100).unwrap())),
            shape_cache: RwLock::new(LruCache::new(std::num::NonZeroUsize::new(1000).unwrap())),
        }
    }

    /// Makes a unique key for caching fonts with their settings
    fn font_cache_key(font: &Arc<dyn FontRef>, params: &ShapingParams) -> String {
        // Create a simple hash from first 32 bytes of font data
        let font_hash = font
            .data()
            .get(..32)
            .map(|bytes| {
                bytes
                    .iter()
                    .fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64))
            })
            .unwrap_or(0);

        // Include variations in cache key - critical for variable fonts!
        let var_key = if params.variations.is_empty() {
            String::new()
        } else {
            let mut sorted_vars: Vec<_> = params.variations.iter().collect();
            sorted_vars.sort_by(|a, b| a.0.cmp(&b.0));
            sorted_vars
                .iter()
                .map(|(tag, val)| format!("{}={:.1}", tag, val))
                .collect::<Vec<_>>()
                .join(",")
        };

        if var_key.is_empty() {
            format!("{}:{}", font_hash, params.size as u32)
        } else {
            format!("{}:{}:{}", font_hash, params.size as u32, var_key)
        }
    }

    /// Makes a unique key for caching shaping results
    fn shape_cache_key(text: &str, font: &Arc<dyn FontRef>, params: &ShapingParams) -> String {
        format!("{}::{}", text, Self::font_cache_key(font, params))
    }

    /// Gets or creates a CoreText font from our font data
    fn build_ct_font(
        &self,
        font: &Arc<dyn FontRef>,
        params: &ShapingParams,
    ) -> Result<Arc<CTFont>> {
        // Create cache key to see if we already have this font
        let cache_key = Self::font_cache_key(font, params);

        // Check cache first
        {
            let cache = self.font_cache.read();
            if let Some(cached) = cache.peek(&cache_key) {
                log::debug!("CoreTextShaper: Font cache hit");
                return Ok(Arc::clone(cached));
            }
        }

        log::debug!("CoreTextShaper: Building new CTFont");

        // Create the font from our data
        let ct_font = Self::create_ct_font_from_data(font.data(), params)?;
        let arc_font = Arc::new(ct_font);

        // Save it for next time
        {
            let mut cache = self.font_cache.write();
            cache.put(cache_key, Arc::clone(&arc_font));
        }

        Ok(arc_font)
    }

    /// Turns raw font bytes into a CoreText CTFont
    fn create_ct_font_from_data(data: &[u8], params: &ShapingParams) -> Result<CTFont> {
        // Create Arc from font data
        let provider_data = Arc::new(ProviderData {
            bytes: Arc::from(data),
        });

        // Create CGDataProvider
        let provider = CGDataProvider::from_buffer(provider_data);

        // Create CGFont from data
        let cg_font = CGFont::from_data_provider(provider).map_err(|_| {
            TypfError::ShapingFailed(ShapingError::BackendError(
                "Failed to create CGFont from data".to_string(),
            ))
        })?;

        // Create base CTFont from CGFont
        let mut ct_font = new_from_CGFont(&cg_font, params.size as f64);

        // Apply variable font coordinates if specified
        if !params.variations.is_empty() {
            log::debug!(
                "CoreTextShaper: Applying {} variation coordinates",
                params.variations.len()
            );

            // Convert variations to CoreFoundation format
            let mut var_dict_entries = Vec::new();
            for (tag, value) in &params.variations {
                if tag.len() == 4 {
                    // Convert 4-character tag string to CFNumber key
                    let tag_num = u32::from_be_bytes([
                        tag.as_bytes()[0],
                        tag.as_bytes()[1],
                        tag.as_bytes()[2],
                        tag.as_bytes()[3],
                    ]);
                    let tag_cf = CFNumber::from(tag_num as i64);
                    let value_cf = CFNumber::from(*value as f64);
                    var_dict_entries.push((tag_cf, value_cf));
                }
            }

            if !var_dict_entries.is_empty() {
                // Create variation dictionary - pairs of (tag_number, value)
                let var_pairs: Vec<_> = var_dict_entries
                    .iter()
                    .map(|(k, v)| (k.as_CFType(), v.as_CFType()))
                    .collect();

                let var_dict = CFDictionary::from_CFType_pairs(&var_pairs);

                // Create font descriptor with variation attributes
                // SAFETY: kCTFontVariationAttribute is a valid CoreText constant
                let desc_pairs = vec![(
                    unsafe { CFString::wrap_under_get_rule(kCTFontVariationAttribute).as_CFType() },
                    var_dict.as_CFType(),
                )];

                let desc_attrs = CFDictionary::from_CFType_pairs(&desc_pairs);

                // Create CTFontDescriptor with variation attributes using FFI
                let descriptor = unsafe {
                    use core_foundation::base::TCFType;
                    let desc_ref =
                        CTFontDescriptorCreateWithAttributes(desc_attrs.as_concrete_TypeRef());
                    CTFontDescriptor::wrap_under_create_rule(desc_ref)
                };

                // Create new CTFont with variation coordinates applied
                ct_font = new_from_descriptor(&descriptor, params.size as f64);
            }
        }

        Ok(ct_font)
    }

    /// Create attributed string with font and features
    fn create_attributed_string(
        &self,
        text: &str,
        ct_font: &CTFont,
        params: &ShapingParams,
    ) -> CFMutableAttributedString {
        let cf_string = CFString::new(text);
        let mut attributed_string = CFMutableAttributedString::new();
        attributed_string.replace_str(&cf_string, CFRange::init(0, 0));

        let range = CFRange::init(0, attributed_string.char_len());

        // Set font attribute
        attributed_string.set_attribute(range, unsafe { kCTFontAttributeName }, ct_font);

        // Apply OpenType features
        Self::apply_features(&mut attributed_string, range, params);

        attributed_string
    }

    /// Apply OpenType features to attributed string
    fn apply_features(
        attr_string: &mut CFMutableAttributedString,
        range: CFRange,
        params: &ShapingParams,
    ) {
        // Apply ligature setting
        if let Some((_, value)) = params.features.iter().find(|(tag, _)| tag == "liga") {
            let ligature_value = CFNumber::from(if *value > 0 { 1 } else { 0 });
            attr_string.set_attribute(range, unsafe { kCTLigatureAttributeName }, &ligature_value);
        }

        // Apply kerning setting
        if let Some((_, value)) = params.features.iter().find(|(tag, _)| tag == "kern") {
            if *value == 0 {
                let zero = CFNumber::from(0.0f64);
                attr_string.set_attribute(range, unsafe { kCTKernAttributeName }, &zero);
            }
        }
    }

    /// Extract glyphs from CTLine
    fn extract_glyphs_from_line(
        &self,
        line: &CTLine,
        font: &Arc<dyn FontRef>,
    ) -> Result<Vec<PositionedGlyph>> {
        let runs = line.glyph_runs();
        let mut glyphs = Vec::new();

        // Get the font's glyph count for validation
        let max_glyph_id = font.glyph_count().unwrap_or(u32::MAX);

        for run in runs.iter() {
            Self::collect_run_glyphs(&run, &mut glyphs, max_glyph_id);
        }

        Ok(glyphs)
    }

    /// Collect glyphs from a single CTRun
    fn collect_run_glyphs(
        run: &CTRun,
        glyphs: &mut Vec<PositionedGlyph>,
        max_glyph_id: u32,
    ) -> f32 {
        let glyph_count = run.glyph_count();
        if glyph_count == 0 {
            return 0.0;
        }

        // Get glyph IDs
        let glyph_ids = run.glyphs();

        // Get positions
        let positions = run.positions();

        // Get string indices (clusters)
        let string_indices = run.string_indices();

        // Get advances
        let advances = Self::run_advances(run);

        let mut advance_sum = 0.0f32;

        for idx in 0..(glyph_count as usize) {
            let raw_glyph_id = *glyph_ids.get(idx).unwrap_or(&0) as u32;
            let position = positions.get(idx).unwrap_or(&CGPoint { x: 0.0, y: 0.0 });
            let cluster = string_indices.get(idx).unwrap_or(&0);
            let advance_size = advances.get(idx).unwrap_or(&CGSize {
                width: 0.0,
                height: 0.0,
            });

            let advance = advance_size.width as f32;

            // Validate glyph ID and use notdef (0) for invalid glyphs
            let glyph_id = if raw_glyph_id < max_glyph_id {
                raw_glyph_id
            } else {
                log::debug!(
                    "CoreTextShaper: Invalid glyph ID {} (max {}), using notdef",
                    raw_glyph_id,
                    max_glyph_id
                );
                0 // Use notdef glyph for invalid IDs
            };

            glyphs.push(PositionedGlyph {
                id: glyph_id,
                x: position.x as f32,
                y: position.y as f32,
                advance,
                cluster: (*cluster).max(0) as u32,
            });

            advance_sum += advance;
        }

        advance_sum
    }

    /// Get advances for all glyphs in a run
    fn run_advances(run: &CTRun) -> Vec<CGSize> {
        let glyph_count = run.glyph_count();
        if glyph_count <= 0 {
            return Vec::new();
        }

        let mut advances = vec![
            CGSize {
                width: 0.0,
                height: 0.0,
            };
            glyph_count as usize
        ];

        // Use FFI to call CTRunGetAdvances
        unsafe {
            CTRunGetAdvances(run.as_concrete_TypeRef(), CFRange::init(0, 0), advances.as_mut_ptr());
        }

        advances
    }
}

// FFI declaration for CTRunGetAdvances
#[link(name = "CoreText", kind = "framework")]
extern "C" {
    fn CTRunGetAdvances(run: CTRunRef, range: CFRange, buffer: *mut CGSize);
}

impl Default for CoreTextShaper {
    fn default() -> Self {
        Self::new()
    }
}

impl Shaper for CoreTextShaper {
    fn name(&self) -> &'static str {
        "coretext"
    }

    fn shape(
        &self,
        text: &str,
        font: Arc<dyn FontRef>,
        params: &ShapingParams,
    ) -> Result<ShapingResult> {
        log::debug!("CoreTextShaper: Shaping {} chars", text.chars().count());

        // Create cache key
        let cache_key = Self::shape_cache_key(text, &font, params);

        // Check shape cache
        {
            let cache = self.shape_cache.read();
            if let Some(cached) = cache.peek(&cache_key) {
                log::debug!("CoreTextShaper: Shape cache hit");
                return Ok((**cached).clone());
            }
        }

        // Build CTFont
        let ct_font = self.build_ct_font(&font, params)?;

        // Create attributed string
        let attr_string = self.create_attributed_string(text, &ct_font, params);

        // Create CTLine
        let line = CTLine::new_with_attributed_string(attr_string.as_concrete_TypeRef());

        // Extract glyphs
        let glyphs = self.extract_glyphs_from_line(&line, &font)?;

        // Calculate metrics
        let advance_width = if let Some(last) = glyphs.last() {
            last.x + last.advance
        } else {
            0.0
        };

        let result = ShapingResult {
            glyphs,
            advance_width,
            advance_height: params.size,
            direction: params.direction,
        };

        // Cache the result
        {
            let mut cache = self.shape_cache.write();
            cache.put(cache_key, Arc::new(result.clone()));
        }

        Ok(result)
    }

    fn supports_script(&self, _script: &str) -> bool {
        // CoreText supports all scripts
        true
    }

    fn clear_cache(&self) {
        log::debug!("CoreTextShaper: Clearing caches");
        self.font_cache.write().clear();
        self.shape_cache.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock font for testing
    #[allow(dead_code)]
    struct MockFont {
        data: Vec<u8>,
    }

    impl FontRef for MockFont {
        fn data(&self) -> &[u8] {
            &self.data
        }

        fn units_per_em(&self) -> u16 {
            1000
        }

        fn glyph_id(&self, ch: char) -> Option<u32> {
            if ch.is_ascii() {
                Some(ch as u32)
            } else {
                None
            }
        }

        fn advance_width(&self, _glyph_id: u32) -> f32 {
            500.0
        }
    }

    #[test]
    fn test_shaper_creation() {
        let shaper = CoreTextShaper::new();
        assert_eq!(shaper.name(), "coretext");
    }

    #[test]
    fn test_supports_all_scripts() {
        let shaper = CoreTextShaper::new();
        assert!(shaper.supports_script("Latn"));
        assert!(shaper.supports_script("Arab"));
        assert!(shaper.supports_script("Deva"));
        assert!(shaper.supports_script("Hans"));
    }

    #[test]
    fn test_cache_clearing() {
        let shaper = CoreTextShaper::new();
        shaper.clear_cache();
        // Just verify it doesn't panic
    }
}
