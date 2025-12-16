//! macOS-native text shaping with CoreText precision
//!
//! CoreText is Apple's professional text shaping engine, built right into macOS.
//! It understands every script Apple supports, handles variable fonts flawlessly,
//! and integrates perfectly with the system font rendering pipeline. This is
//! the shaper you want on macOS for that native Apple typography feel.

#![cfg(target_os = "macos")]

// this_file: backends/typf-shape-ct/src/lib.rs

use std::{
    cell::RefCell,
    ffi::c_void,
    ptr::{self, NonNull},
    sync::Arc,
};
use typf_core::{
    error::{Result, ShapingError, TypfError},
    traits::{FontRef, Shaper},
    types::{PositionedGlyph, ShapingResult},
    ShapingParams,
};

use objc2_core_foundation::{
    CFDictionary, CFMutableAttributedString, CFNumber, CFRange, CFRetained, CFString, CFType,
    CGFloat, CGPoint, CGSize,
};
use objc2_core_graphics::{CGDataProvider, CGFont};
use objc2_core_text::{
    kCTFontAttributeName, kCTFontVariationAttribute, kCTKernAttributeName,
    kCTLigatureAttributeName, CTFont, CTFontDescriptor, CTLine, CTRun,
};

use lru::LruCache;
use parking_lot::RwLock;

// Thread-local font cache to avoid cross-thread CTFont destruction.
// CoreText fonts have thread affinity - destroying a CTFont on a different
// thread than it was created causes memory corruption in OTL::Lookup.
//
// Note: We use Arc here even though CFRetained<CTFont> is not Send+Sync
// because the cache is thread-local and Arcs never cross thread boundaries.
// This allows cloning the Arc within the same thread's cache operations.
thread_local! {
    static FONT_CACHE: RefCell<LruCache<FontCacheKey, Arc<CFRetained<CTFont>>>> =
        RefCell::new(LruCache::new(std::num::NonZeroUsize::new(50).unwrap()));
}

/// How we identify fonts in our cache
type FontCacheKey = String;

/// How we identify shaping results in our cache
type ShapeCacheKey = String;

/// Callback to release font data when CGDataProvider is done with it
unsafe extern "C-unwind" fn release_data_callback(
    info: *mut c_void,
    _data: NonNull<c_void>,
    _size: usize,
) {
    if !info.is_null() {
        // Reconstruct the Box to drop it properly
        let _ = unsafe { Box::from_raw(info as *mut Arc<[u8]>) };
    }
}

/// Professional text shaping powered by macOS CoreText
pub struct CoreTextShaper {
    /// Cache shaping results to avoid redundant work
    /// Note: font_cache is thread-local (FONT_CACHE) to ensure CTFont objects
    /// are always destroyed on the same thread they were created, avoiding
    /// memory corruption in CoreText's OTL lookup tables.
    shape_cache: Option<RwLock<LruCache<ShapeCacheKey, Arc<ShapingResult>>>>,
}

impl CoreTextShaper {
    /// Creates a new shaper ready to work with CoreText
    pub fn new() -> Self {
        Self::with_cache(true)
    }

    /// Creates a new shaper with optional shape caching
    pub fn with_cache(enabled: bool) -> Self {
        let cache = if enabled {
            Some(RwLock::new(LruCache::new(
                std::num::NonZeroUsize::new(1000).unwrap(),
            )))
        } else {
            None
        };

        Self { shape_cache: cache }
    }

    /// Makes a unique key for caching fonts with their settings
    fn font_cache_key(font: &Arc<dyn FontRef>, params: &ShapingParams) -> String {
        // Create a robust hash using font length + samples from start, middle, and end.
        // Just using first 32 bytes was broken: many fonts have identical headers,
        // causing cache collisions that returned wrong glyph IDs for different fonts.
        let data = font.data();
        let len = data.len();

        // Hash: length XOR samples from beginning, middle, and end
        let mut font_hash = len as u64;

        // Sample first 64 bytes
        for (i, &b) in data.iter().take(64).enumerate() {
            font_hash = font_hash
                .wrapping_mul(31)
                .wrapping_add(b as u64)
                .wrapping_add(i as u64);
        }

        // Sample 64 bytes from middle
        if len > 128 {
            let mid = len / 2;
            for (i, &b) in data[mid..].iter().take(64).enumerate() {
                font_hash = font_hash
                    .wrapping_mul(37)
                    .wrapping_add(b as u64)
                    .wrapping_add(i as u64);
            }
        }

        // Sample last 64 bytes
        if len > 64 {
            for (i, &b) in data[len.saturating_sub(64)..].iter().enumerate() {
                font_hash = font_hash
                    .wrapping_mul(41)
                    .wrapping_add(b as u64)
                    .wrapping_add(i as u64);
            }
        }

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
    ) -> Result<Arc<CFRetained<CTFont>>> {
        // Create cache key to see if we already have this font
        let cache_key = Self::font_cache_key(font, params);

        // Use thread-local cache to ensure CTFont destruction happens on
        // the same thread as creation (CoreText thread-safety requirement)
        FONT_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();

            // Check cache first
            if let Some(cached) = cache.get(&cache_key) {
                log::debug!("CoreTextShaper: Font cache hit (thread-local)");
                return Ok(Arc::clone(cached));
            }

            log::debug!("CoreTextShaper: Building new CTFont (thread-local)");

            // Create the font from our data
            let ct_font = Self::create_ct_font_from_data(font.data(), params)?;
            // Arc is used even though CTFont isn't Send+Sync because this cache is
            // thread-local - the Arc never crosses thread boundaries, it just enables
            // cheap cloning within the same thread's cache operations.
            #[allow(clippy::arc_with_non_send_sync)]
            let arc_font = Arc::new(ct_font);

            // Save it for next time (on this thread only)
            cache.put(cache_key, Arc::clone(&arc_font));

            Ok(arc_font)
        })
    }

    /// Convert a 4-character axis tag to its numeric identifier.
    /// Example: "wght" -> 2003265652 (0x77676874)
    fn tag_to_axis_id(tag: &str) -> Option<i64> {
        if tag.len() != 4 {
            return None;
        }
        let bytes = tag.as_bytes();
        Some(
            ((bytes[0] as i64) << 24)
                | ((bytes[1] as i64) << 16)
                | ((bytes[2] as i64) << 8)
                | (bytes[3] as i64),
        )
    }

    /// Turns raw font bytes into a CoreText CTFont
    fn create_ct_font_from_data(data: &[u8], params: &ShapingParams) -> Result<CFRetained<CTFont>> {
        // Create Arc from font data to keep it alive
        let font_data: Arc<[u8]> = Arc::from(data);
        let data_ptr = font_data.as_ptr();
        let data_len = font_data.len();

        // Box the Arc so we can pass ownership to the callback
        let boxed = Box::new(font_data);
        let info_ptr = Box::into_raw(boxed) as *mut c_void;

        // Create CGDataProvider using the raw callback API
        let provider = unsafe {
            CGDataProvider::with_data(
                info_ptr,
                data_ptr as *const c_void,
                data_len,
                Some(release_data_callback),
            )
        }
        .ok_or_else(|| {
            TypfError::ShapingFailed(ShapingError::BackendError(
                "Failed to create CGDataProvider".to_string(),
            ))
        })?;

        // Create CGFont from data provider
        let cg_font = CGFont::with_data_provider(&provider).ok_or_else(|| {
            TypfError::ShapingFailed(ShapingError::BackendError(
                "Failed to create CGFont from data".to_string(),
            ))
        })?;

        // Apply variable font coordinates if specified
        if !params.variations.is_empty() {
            log::debug!(
                "CoreTextShaper: Applying {} variation coordinates",
                params.variations.len()
            );

            // Convert variations to CFDictionary<CFNumber(axis_id), CFNumber(value)>
            let var_pairs: Vec<(CFRetained<CFNumber>, CFRetained<CFNumber>)> = params
                .variations
                .iter()
                .filter_map(|(tag, value)| {
                    Self::tag_to_axis_id(tag).map(|axis_id| {
                        (CFNumber::new_i64(axis_id), CFNumber::new_f64(*value as f64))
                    })
                })
                .collect();

            if !var_pairs.is_empty() {
                // Build dictionary with axis ID -> value pairs
                let keys: Vec<&CFNumber> = var_pairs.iter().map(|(k, _)| k.as_ref()).collect();
                let values: Vec<&CFNumber> = var_pairs.iter().map(|(_, v)| v.as_ref()).collect();
                let var_dict = CFDictionary::from_slices(&keys, &values);

                // Create descriptor attributes dictionary with variation attribute
                let var_key: &CFString = unsafe { kCTFontVariationAttribute };

                // We need to cast the dictionary to CFType for the attributes
                let var_dict_type = unsafe { CFRetained::cast_unchecked::<CFType>(var_dict) };

                let attr_keys: [&CFString; 1] = [var_key];
                let attr_values: [&CFType; 1] = [&var_dict_type];
                let attrs_dict = CFDictionary::from_slices(&attr_keys, &attr_values);

                // Create font descriptor with variation attributes
                let attrs_untyped =
                    unsafe { CFRetained::cast_unchecked::<CFDictionary>(attrs_dict) };
                let desc = unsafe { CTFontDescriptor::with_attributes(&attrs_untyped) };

                // Create CTFont with the descriptor
                return Ok(unsafe {
                    CTFont::with_graphics_font(
                        &cg_font,
                        params.size as CGFloat,
                        ptr::null(),
                        Some(&desc),
                    )
                });
            }
        }

        // No variations or no valid axis tags - use base CGFont
        Ok(unsafe {
            CTFont::with_graphics_font(&cg_font, params.size as CGFloat, ptr::null(), None)
        })
    }

    /// Create attributed string with font and features
    fn create_attributed_string(
        &self,
        text: &str,
        ct_font: &CTFont,
        params: &ShapingParams,
    ) -> CFRetained<CFMutableAttributedString> {
        let cf_string = CFString::from_str(text);

        // Create empty mutable attributed string
        let attributed_string = CFMutableAttributedString::new(None, 0)
            .expect("Failed to create CFMutableAttributedString");

        // Replace content (append to empty string)
        let len = cf_string.length();
        unsafe {
            CFMutableAttributedString::replace_string(
                Some(&attributed_string),
                CFRange::new(0, 0),
                Some(&cf_string),
            );
        }

        let range = CFRange::new(0, len);

        // Set font attribute
        let font_key: &CFString = unsafe { kCTFontAttributeName };
        // CTFont needs to be cast to CFType
        let ct_font_type: &CFType = unsafe { &*(ct_font as *const CTFont as *const CFType) };
        unsafe {
            CFMutableAttributedString::set_attribute(
                Some(&attributed_string),
                range,
                Some(font_key),
                Some(ct_font_type),
            );
        }

        // Apply OpenType features
        Self::apply_features(&attributed_string, range, params);

        attributed_string
    }

    /// Apply OpenType features to attributed string
    fn apply_features(
        attr_string: &CFMutableAttributedString,
        range: CFRange,
        params: &ShapingParams,
    ) {
        // Apply ligature setting
        if let Some((_, value)) = params.features.iter().find(|(tag, _)| tag == "liga") {
            let ligature_value = CFNumber::new_i32(if *value > 0 { 1 } else { 0 });
            let lig_key: &CFString = unsafe { kCTLigatureAttributeName };
            let lig_type: &CFType =
                unsafe { &*(&*ligature_value as *const CFNumber as *const CFType) };
            unsafe {
                CFMutableAttributedString::set_attribute(
                    Some(attr_string),
                    range,
                    Some(lig_key),
                    Some(lig_type),
                );
            }
        }

        // Apply kerning setting
        if let Some((_, value)) = params.features.iter().find(|(tag, _)| tag == "kern") {
            if *value == 0 {
                let zero = CFNumber::new_f64(0.0);
                let kern_key: &CFString = unsafe { kCTKernAttributeName };
                let kern_type: &CFType = unsafe { &*(&*zero as *const CFNumber as *const CFType) };
                unsafe {
                    CFMutableAttributedString::set_attribute(
                        Some(attr_string),
                        range,
                        Some(kern_key),
                        Some(kern_type),
                    );
                }
            }
        }
    }

    /// Extract glyphs from CTLine
    fn extract_glyphs_from_line(
        &self,
        line: &CTLine,
        font: &Arc<dyn FontRef>,
    ) -> Result<Vec<PositionedGlyph>> {
        let runs = unsafe { line.glyph_runs() };
        let mut glyphs = Vec::new();

        // Get the font's glyph count for validation
        let max_glyph_id = font.glyph_count().unwrap_or(u32::MAX);

        // Iterate over CFArray of CTRun
        let run_count = runs.len();
        for i in 0..run_count {
            // Get run from array - it's a CFType that we cast to CTRun
            let run_ptr = unsafe { runs.value_at_index(i as isize) };
            if run_ptr.is_null() {
                continue;
            }
            let run: &CTRun = unsafe { &*(run_ptr as *const CTRun) };
            Self::collect_run_glyphs(run, &mut glyphs, max_glyph_id);
        }

        Ok(glyphs)
    }

    /// Collect glyphs from a single CTRun
    fn collect_run_glyphs(
        run: &CTRun,
        glyphs: &mut Vec<PositionedGlyph>,
        max_glyph_id: u32,
    ) -> f32 {
        let glyph_count = unsafe { run.glyph_count() };
        if glyph_count <= 0 {
            return 0.0;
        }
        let count = glyph_count as usize;

        // Get direct pointers to run data
        let glyphs_ptr = unsafe { run.glyphs_ptr() };
        let positions_ptr = unsafe { run.positions_ptr() };
        let indices_ptr = unsafe { run.string_indices_ptr() };

        // Get advances - need to allocate buffer and call advances method
        let mut advances = vec![
            CGSize {
                width: 0.0,
                height: 0.0
            };
            count
        ];
        if let Some(advances_nonnull) = NonNull::new(advances.as_mut_ptr()) {
            unsafe {
                run.advances(CFRange::new(0, 0), advances_nonnull);
            }
        }

        let mut advance_sum = 0.0f32;

        for idx in 0..count {
            // Get glyph ID
            let raw_glyph_id = if !glyphs_ptr.is_null() {
                (unsafe { *glyphs_ptr.add(idx) }) as u32
            } else {
                0
            };

            // Get position
            let position = if !positions_ptr.is_null() {
                unsafe { *positions_ptr.add(idx) }
            } else {
                CGPoint { x: 0.0, y: 0.0 }
            };

            // Get cluster (string index)
            let cluster = if !indices_ptr.is_null() {
                unsafe { *indices_ptr.add(idx) }
            } else {
                0
            };

            // Get advance
            let advance = advances.get(idx).map(|s| s.width as f32).unwrap_or(0.0);

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
                cluster: cluster.max(0) as u32,
            });

            advance_sum += advance;
        }

        advance_sum
    }
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
        if let Some(cache_lock) = &self.shape_cache {
            let cache = cache_lock.read();
            if let Some(cached) = cache.peek(&cache_key) {
                log::debug!("CoreTextShaper: Shape cache hit");
                return Ok((**cached).clone());
            }
        }

        // Build CTFont
        let ct_font = self.build_ct_font(&font, params)?;

        // Create attributed string
        let attr_string = self.create_attributed_string(text, &ct_font, params);

        // Create CTLine - need to cast CFMutableAttributedString to CFAttributedString
        let attr_str_ref = unsafe {
            &*(&*attr_string as *const CFMutableAttributedString
                as *const objc2_core_foundation::CFAttributedString)
        };
        let line = unsafe { CTLine::with_attributed_string(attr_str_ref) };

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
        if let Some(cache_lock) = &self.shape_cache {
            let mut cache = cache_lock.write();
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
        // Clear thread-local font cache
        FONT_CACHE.with(|cache| cache.borrow_mut().clear());
        // Clear shared shape cache
        if let Some(cache) = &self.shape_cache {
            cache.write().clear();
        }
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
    fn test_variations_preserve_font_identity() {
        use std::fs;
        use std::path::Path;

        // Locate Archivo variable font used across the test suite
        let font_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../runs/assets/candidates/Archivo[wdth,wght].ttf");

        if !font_path.exists() {
            eprintln!(
                "skipped: Archivo variable font not found at {:?}",
                font_path
            );
            return;
        }

        let data = match fs::read(&font_path) {
            Ok(data) => data,
            Err(e) => unreachable!("failed to read Archivo variable font: {e}"),
        };

        let base_params = ShapingParams {
            size: 32.0,
            ..ShapingParams::default()
        };

        // Base font without variations
        let base = match CoreTextShaper::create_ct_font_from_data(&data, &base_params) {
            Ok(base) => base,
            Err(e) => unreachable!("failed to create base CTFont: {e}"),
        };

        // Apply variations that previously triggered descriptor-based lookup
        let mut var_params = base_params.clone();
        var_params.variations = vec![("wght".to_string(), 900.0), ("wdth".to_string(), 100.0)];

        let with_vars = match CoreTextShaper::create_ct_font_from_data(&data, &var_params) {
            Ok(with_vars) => with_vars,
            Err(e) => unreachable!("failed to create CTFont with variations: {e}"),
        };

        // The font identity must stay the same; losing it would swap in a system font
        let base_name = unsafe { base.post_script_name() };
        let vars_name = unsafe { with_vars.post_script_name() };
        assert_eq!(
            base_name.to_string(),
            vars_name.to_string(),
            "Applying variations must not change the underlying font",
        );
    }

    #[test]
    fn test_cache_clearing() {
        let shaper = CoreTextShaper::new();
        shaper.clear_cache();
        // Just verify it doesn't panic
    }
}
