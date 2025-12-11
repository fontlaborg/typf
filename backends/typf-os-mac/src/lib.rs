//! Single-pass text rendering using macOS CoreText
//!
//! This backend shapes AND renders text in a single operation using CoreText's
//! CTLineDraw API. By letting CoreText control the entire pipeline, we get:
//!
//! - Optimal performance (no intermediate glyph extraction)
//! - Native macOS text quality
//! - Correct handling of variable fonts and OpenType features
//!
//! ## Performance
//!
//! Traditional pipeline:
//! 1. CoreText shapes text → extract glyphs → ShapingResult
//! 2. CoreGraphics renders each glyph → composite to bitmap
//!
//! Linra pipeline:
//! 1. CTLineDraw: shape + render in one call
//!
//! The linra approach eliminates per-glyph overhead and allows CoreText
//! to optimize internally (e.g., batch GPU operations).

#![cfg(target_os = "macos")]

use std::cell::RefCell;
use std::ffi::c_void;
use std::num::NonZeroUsize;
use std::ptr::{self, NonNull};
use std::sync::Arc;

use objc2_core_foundation::{
    CFDictionary, CFMutableAttributedString, CFNumber, CFRange, CFRetained, CFString, CFType,
    CGFloat, CGPoint, CGRect, CGSize,
};
use objc2_core_graphics::{
    CGBitmapContextCreate, CGColorSpace, CGContext, CGDataProvider, CGFont, CGImageAlphaInfo,
    CGTextDrawingMode,
};
use objc2_core_text::{
    kCTFontAttributeName, kCTFontVariationAttribute, kCTKernAttributeName,
    kCTLigatureAttributeName, CTFont, CTFontDescriptor, CTLine,
};

use lru::LruCache;

use typf_core::{
    error::{RenderError, Result, TypfError},
    linra::{LinraRenderParams, LinraRenderer},
    traits::FontRef,
    types::{BitmapData, BitmapFormat, RenderOutput},
    Color,
};

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

/// Cache key for CTFont instances
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FontCacheKey {
    /// Hash of font data
    font_hash: u64,
    /// Font size (as integer for stable hashing)
    size: u32,
    /// Sorted variation string
    variations: String,
}

impl FontCacheKey {
    fn new(font_data: &[u8], size: f32, variations: &[(String, f32)]) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Hash font data
        let mut hasher = DefaultHasher::new();
        font_data.hash(&mut hasher);
        let font_hash = hasher.finish();

        // Sort variations for consistent key
        let mut sorted_vars: Vec<_> = variations.iter().collect();
        sorted_vars.sort_by(|a, b| a.0.cmp(&b.0));
        let var_str = sorted_vars
            .iter()
            .map(|(tag, val)| format!("{}={:.1}", tag, val))
            .collect::<Vec<_>>()
            .join(",");

        Self {
            font_hash,
            size: (size * 100.0) as u32,
            variations: var_str,
        }
    }
}

/// Cached font entry that keeps font data alive
///
/// CoreText may hold internal pointers to the font data, so we must
/// ensure the data outlives the CTFont to prevent use-after-free crashes.
struct CachedFont {
    /// The CTFont itself
    ct_font: CFRetained<CTFont>,
    /// Font data kept alive to prevent use-after-free
    /// CGFont/CTFont may hold internal pointers to this data
    _data: Arc<[u8]>,
}

// Thread-local font cache to avoid cross-thread CTFont destruction.
// CoreText fonts have thread affinity - destroying a CTFont on a different
// thread than it was created can cause memory corruption.
thread_local! {
    static FONT_CACHE: RefCell<LruCache<FontCacheKey, CachedFont>> =
        RefCell::new(LruCache::new(NonZeroUsize::new(100).unwrap()));
}

/// Single-pass text renderer using macOS CoreText
///
/// This renderer combines text shaping and rendering into a single CTLineDraw
/// call for maximum performance on macOS.
pub struct CoreTextLinraRenderer {
    // No fields - cache is thread-local
}

impl CoreTextLinraRenderer {
    /// Creates a new linra renderer
    pub fn new() -> Self {
        Self {}
    }

    /// Validate font data has valid TrueType/OpenType signature
    ///
    /// This prevents CoreText from crashing on corrupted or invalid font data.
    fn validate_font_data(data: &[u8]) -> Result<()> {
        if data.len() < 12 {
            return Err(TypfError::RenderingFailed(RenderError::BackendError(
                "Font data too small to be valid".to_string(),
            )));
        }

        // Check for valid font signatures (first 4 bytes)
        let sig = &data[0..4];
        let is_valid = matches!(
            sig,
            // TrueType
            [0x00, 0x01, 0x00, 0x00]
            // OpenType with CFF
            | [b'O', b'T', b'T', b'O']
            // TrueType (Mac)
            | [b't', b'r', b'u', b'e']
            // TrueType Collection
            | [b't', b't', b'c', b'f']
            // WOFF
            | [b'w', b'O', b'F', b'F']
            // WOFF2
            | [b'w', b'O', b'F', b'2']
        );

        if !is_valid {
            return Err(TypfError::RenderingFailed(RenderError::BackendError(
                format!(
                    "Invalid font signature: {:02x}{:02x}{:02x}{:02x}",
                    sig[0], sig[1], sig[2], sig[3]
                ),
            )));
        }

        Ok(())
    }

    /// Create CGFont from font data
    ///
    /// Takes Arc<[u8]> directly to ensure the same Arc is used by both
    /// CGDataProvider and CachedFont, preventing use-after-free.
    fn create_cg_font(data: Arc<[u8]>) -> Result<CFRetained<CGFont>> {
        // Validate font data first to prevent CoreText crashes
        Self::validate_font_data(&data)?;

        // Create Arc clone for the data provider callback
        let data_ptr = data.as_ptr();
        let data_len = data.len();

        // Box the Arc so we can pass ownership to the callback
        let boxed = Box::new(data);
        let info_ptr = Box::into_raw(boxed) as *mut c_void;

        // Create CGDataProvider
        let provider = unsafe {
            CGDataProvider::with_data(
                info_ptr,
                data_ptr as *const c_void,
                data_len,
                Some(release_data_callback),
            )
        }
        .ok_or_else(|| {
            TypfError::RenderingFailed(RenderError::BackendError(
                "Failed to create CGDataProvider".to_string(),
            ))
        })?;

        CGFont::with_data_provider(&provider).ok_or_else(|| {
            TypfError::RenderingFailed(RenderError::BackendError(
                "Failed to create CGFont from data".to_string(),
            ))
        })
    }

    /// Convert 4-char axis tag to numeric identifier
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

    /// Create CTFont with optional variation coordinates
    ///
    /// Takes Arc<[u8]> to ensure the same Arc flows through to CGDataProvider.
    fn create_ct_font(
        data: Arc<[u8]>,
        font_size: f64,
        variations: &[(String, f32)],
    ) -> Result<CFRetained<CTFont>> {
        let cg_font = Self::create_cg_font(data)?;

        if !variations.is_empty() {
            // Build variation dictionary with numeric axis IDs
            let var_pairs: Vec<(CFRetained<CFNumber>, CFRetained<CFNumber>)> = variations
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

                // Cast dictionary to CFType for the attributes
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
                        font_size as CGFloat,
                        ptr::null(),
                        Some(&desc),
                    )
                });
            }
        }

        // No variations - create font without descriptor
        Ok(
            unsafe {
                CTFont::with_graphics_font(&cg_font, font_size as CGFloat, ptr::null(), None)
            },
        )
    }

    /// Get or create a cached CTFont
    ///
    /// Uses thread-local cache since CTFont has thread affinity.
    /// Returns a reference to the CTFont via closure to avoid lifetime issues.
    fn with_ct_font<R>(
        font: &Arc<dyn FontRef>,
        params: &LinraRenderParams,
        f: impl FnOnce(&CTFont) -> Result<R>,
    ) -> Result<R> {
        let data = font.data();
        let cache_key = FontCacheKey::new(data, params.size, &params.variations);

        FONT_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();

            // Check cache first
            if let Some(cached) = cache.get(&cache_key) {
                return f(&cached.ct_font);
            }

            // Copy font data to keep it alive with the CTFont
            let data_arc: Arc<[u8]> = Arc::from(data);

            // Create new CTFont
            let ct_font = Self::create_ct_font(
                Arc::clone(&data_arc),
                params.size as f64,
                &params.variations,
            )?;

            // Wrap in CachedFont
            let cached_font = CachedFont {
                ct_font,
                _data: data_arc,
            };

            // Cache it and call the closure
            cache.put(cache_key.clone(), cached_font);
            let cached = cache.get(&cache_key).unwrap();
            f(&cached.ct_font)
        })
    }

    /// Create attributed string with font and features
    fn create_attributed_string(
        text: &str,
        ct_font: &CTFont,
        params: &LinraRenderParams,
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

    /// Apply OpenType features and letter spacing to attributed string
    fn apply_features(
        attr_string: &CFMutableAttributedString,
        range: CFRange,
        params: &LinraRenderParams,
    ) {
        // Ligatures
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

        // Kerning feature (disable kerning)
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

        // Letter spacing (tracking) - applied via kCTKernAttributeName
        // This adds extra spacing between each character pair
        if params.letter_spacing != 0.0 {
            let kern_value = CFNumber::new_f64(params.letter_spacing as f64);
            let kern_key: &CFString = unsafe { kCTKernAttributeName };
            let kern_type: &CFType =
                unsafe { &*(&*kern_value as *const CFNumber as *const CFType) };
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

    /// Convert Color to CoreGraphics normalized floats
    fn color_to_rgb(color: &Color) -> (CGFloat, CGFloat, CGFloat, CGFloat) {
        (
            color.r as CGFloat / 255.0,
            color.g as CGFloat / 255.0,
            color.b as CGFloat / 255.0,
            color.a as CGFloat / 255.0,
        )
    }
}

impl Default for CoreTextLinraRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl LinraRenderer for CoreTextLinraRenderer {
    fn name(&self) -> &'static str {
        "coretext-linra"
    }

    fn render_text(
        &self,
        text: &str,
        font: Arc<dyn FontRef>,
        params: &LinraRenderParams,
    ) -> Result<RenderOutput> {
        log::debug!(
            "CoreTextLinraRenderer: Rendering '{}' at size {}",
            text,
            params.size
        );

        // Handle empty text
        if text.is_empty() {
            return Ok(RenderOutput::Bitmap(BitmapData {
                width: 1,
                height: 1,
                format: BitmapFormat::Rgba8,
                data: vec![0, 0, 0, 0],
            }));
        }

        // Use closure-based approach to work with thread-local cache
        Self::with_ct_font(&font, params, |ct_font| {
            // Create attributed string using the CTFont from cache
            let attr_string = Self::create_attributed_string(text, ct_font, params);

            // Create CTLine - need to cast CFMutableAttributedString to CFAttributedString
            let attr_str_ref = unsafe {
                &*(&*attr_string as *const CFMutableAttributedString
                    as *const objc2_core_foundation::CFAttributedString)
            };
            let line = unsafe { CTLine::with_attributed_string(attr_str_ref) };

            // Get line metrics for sizing
            let mut ascent: CGFloat = 0.0;
            let mut descent: CGFloat = 0.0;
            let mut leading: CGFloat = 0.0;
            let line_width =
                unsafe { line.typographic_bounds(&mut ascent, &mut descent, &mut leading) };
            let line_height = ascent + descent;

            // Validate metrics are finite (corrupt fonts can produce NaN/Inf)
            if !line_width.is_finite()
                || !line_height.is_finite()
                || !ascent.is_finite()
                || !descent.is_finite()
            {
                return Err(TypfError::RenderingFailed(RenderError::BackendError(
                    format!(
                        "Invalid typographic bounds from font (width={}, height={}) - font may be corrupt",
                        line_width, line_height
                    ),
                )));
            }

            // Calculate canvas dimensions
            let padding = params.padding as CGFloat;
            let width = ((line_width + padding * 2.0).ceil().max(1.0) as u32).clamp(1, 16384);
            let height = ((line_height + padding * 2.0).ceil().max(1.0) as u32).clamp(1, 16384);

            log::debug!(
                "CoreTextLinraRenderer: Canvas {}x{}, line width {:.1}",
                width,
                height,
                line_width
            );

            // Create bitmap buffer
            let bytes_per_row = width as usize * 4;
            let mut buffer = vec![0u8; height as usize * bytes_per_row];

            // Create CGContext
            let color_space = CGColorSpace::new_device_rgb().ok_or_else(|| {
                TypfError::RenderingFailed(RenderError::BackendError(
                    "Failed to create color space".to_string(),
                ))
            })?;

            let context = unsafe {
                CGBitmapContextCreate(
                    buffer.as_mut_ptr() as *mut c_void,
                    width as usize,
                    height as usize,
                    8,
                    bytes_per_row,
                    Some(&color_space),
                    CGImageAlphaInfo::PremultipliedLast.0,
                )
            }
            .ok_or_else(|| {
                TypfError::RenderingFailed(RenderError::BackendError(
                    "Failed to create bitmap context".to_string(),
                ))
            })?;

            // Configure antialiasing
            CGContext::set_should_antialias(Some(&context), params.antialias);
            CGContext::set_should_smooth_fonts(Some(&context), params.antialias);

            // Fill background
            if let Some(bg_color) = &params.background {
                let (r, g, b, a) = Self::color_to_rgb(bg_color);
                CGContext::set_rgb_fill_color(Some(&context), r, g, b, a);
                CGContext::fill_rect(
                    Some(&context),
                    CGRect {
                        origin: CGPoint { x: 0.0, y: 0.0 },
                        size: CGSize {
                            width: width as CGFloat,
                            height: height as CGFloat,
                        },
                    },
                );
            } else {
                CGContext::clear_rect(
                    Some(&context),
                    CGRect {
                        origin: CGPoint { x: 0.0, y: 0.0 },
                        size: CGSize {
                            width: width as CGFloat,
                            height: height as CGFloat,
                        },
                    },
                );
            }

            // Set text color
            let (r, g, b, a) = Self::color_to_rgb(&params.foreground);
            CGContext::set_rgb_fill_color(Some(&context), r, g, b, a);

            // Position text at baseline
            // CoreGraphics uses bottom-left origin
            let baseline_y = padding + descent;
            let text_x = padding;

            unsafe {
                CGContext::save_g_state(Some(&context));
                CGContext::set_text_drawing_mode(Some(&context), CGTextDrawingMode::Fill);
                CGContext::set_text_position(Some(&context), text_x, baseline_y);

                // THE KEY OPERATION: CTLineDraw shapes AND renders in one call
                line.draw(&context);

                CGContext::restore_g_state(Some(&context));
            }

            Ok(RenderOutput::Bitmap(BitmapData {
                width,
                height,
                format: BitmapFormat::Rgba8,
                data: buffer,
            }))
        })
    }

    fn clear_cache(&self) {
        FONT_CACHE.with(|cache| cache.borrow_mut().clear());
    }

    fn supports_format(&self, format: &str) -> bool {
        matches!(format, "bitmap" | "rgba")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_renderer_creation() {
        let renderer = CoreTextLinraRenderer::new();
        assert_eq!(renderer.name(), "coretext-linra");
    }

    #[test]
    fn test_supports_format() {
        let renderer = CoreTextLinraRenderer::new();
        assert!(renderer.supports_format("bitmap"));
        assert!(renderer.supports_format("rgba"));
        assert!(!renderer.supports_format("svg"));
    }

    #[test]
    fn test_empty_text() {
        let renderer = CoreTextLinraRenderer::new();
        let font = Arc::new(MockFont { data: vec![] });
        let params = LinraRenderParams::default();

        let result = renderer.render_text("", font, &params);
        assert!(result.is_ok());

        if let Ok(RenderOutput::Bitmap(bitmap)) = result {
            assert_eq!(bitmap.width, 1);
            assert_eq!(bitmap.height, 1);
        }
    }

    #[test]
    fn test_font_cache_key() {
        let key1 = FontCacheKey::new(b"font1", 16.0, &[]);
        let key2 = FontCacheKey::new(b"font1", 16.0, &[]);
        let key3 = FontCacheKey::new(b"font2", 16.0, &[]);
        let key4 = FontCacheKey::new(b"font1", 24.0, &[]);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
        assert_ne!(key1, key4);
    }

    #[test]
    fn test_font_cache_key_with_variations() {
        let vars = vec![("wght".to_string(), 700.0), ("wdth".to_string(), 100.0)];
        let key1 = FontCacheKey::new(b"font", 16.0, &vars);

        // Order shouldn't matter (we sort internally)
        let vars_reversed = vec![("wdth".to_string(), 100.0), ("wght".to_string(), 700.0)];
        let key2 = FontCacheKey::new(b"font", 16.0, &vars_reversed);

        assert_eq!(key1, key2);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_with_system_font() {
        use std::fs;

        let font_path = "/System/Library/Fonts/Helvetica.ttc";
        if let Ok(font_data) = fs::read(font_path) {
            let font = Arc::new(MockFont { data: font_data });
            let renderer = CoreTextLinraRenderer::new();

            let params = LinraRenderParams {
                size: 24.0,
                foreground: Color::black(),
                background: Some(Color::white()),
                padding: 4,
                ..Default::default()
            };

            let result = renderer.render_text("Hello, World!", font, &params);
            assert!(result.is_ok());

            if let Ok(RenderOutput::Bitmap(bitmap)) = result {
                assert!(bitmap.width > 10);
                assert!(bitmap.height > 10);
                assert_eq!(bitmap.format, BitmapFormat::Rgba8);

                // Check we have some non-zero pixels (text was rendered)
                let has_content = bitmap.data.iter().any(|&b| b > 0);
                assert!(has_content, "Rendered bitmap should contain content");
            }
        }
    }
}
