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

use std::sync::Arc;

use core_foundation::{
    attributed_string::CFMutableAttributedString,
    base::{CFRange, TCFType},
    dictionary::CFDictionary,
    number::CFNumber,
    string::CFString,
};
use core_graphics::{
    base::CGFloat,
    color_space::CGColorSpace,
    context::{CGContext, CGTextDrawingMode},
    data_provider::CGDataProvider,
    font::CGFont,
    geometry::{CGPoint, CGRect, CGSize},
};
use core_text::{
    font::{new_from_CGFont, CTFont},
    font_descriptor::kCTFontVariationAttribute,
    line::CTLine,
    string_attributes::{kCTFontAttributeName, kCTKernAttributeName, kCTLigatureAttributeName},
};
use foreign_types::ForeignType;
use lru::LruCache;
use parking_lot::RwLock;
use std::num::NonZeroUsize;
use std::ptr;

use typf_core::{
    error::{RenderError, Result, TypfError},
    traits::FontRef,
    types::{BitmapData, BitmapFormat, RenderOutput},
    linra::{LinraRenderParams, LinraRenderer},
    Color,
};

/// Bridge between font bytes and CoreGraphics' data expectations
struct ProviderData {
    bytes: Arc<[u8]>,
}

impl AsRef<[u8]> for ProviderData {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
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
    ct_font: CTFont,
    /// Font data kept alive to prevent use-after-free
    /// CGFont/CTFont may hold internal pointers to this data
    _data: Arc<[u8]>,
}

/// Single-pass text renderer using macOS CoreText
///
/// This renderer combines text shaping and rendering into a single CTLineDraw
/// call for maximum performance on macOS.
pub struct CoreTextLinraRenderer {
    /// CTFont cache to avoid expensive font creation
    /// Stores both CTFont and font data to ensure data outlives the font
    font_cache: RwLock<LruCache<FontCacheKey, Arc<CachedFont>>>,
}

impl CoreTextLinraRenderer {
    /// Creates a new linra renderer
    pub fn new() -> Self {
        Self {
            font_cache: RwLock::new(LruCache::new(NonZeroUsize::new(100).unwrap())),
        }
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
    fn create_cg_font(data: Arc<[u8]>) -> Result<CGFont> {
        // Validate font data first to prevent CoreText crashes
        Self::validate_font_data(&data)?;

        // Use the same Arc - don't create a new one!
        let provider_data = Arc::new(ProviderData { bytes: data });

        let provider = CGDataProvider::from_buffer(provider_data);

        CGFont::from_data_provider(provider).map_err(|_| {
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
    ) -> Result<CTFont> {
        let cg_font = Self::create_cg_font(data)?;

        if !variations.is_empty() {
            // Build variation dictionary with numeric axis IDs
            let var_pairs: Vec<(CFNumber, CFNumber)> = variations
                .iter()
                .filter_map(|(tag, value)| {
                    Self::tag_to_axis_id(tag).map(|axis_id| {
                        (CFNumber::from(axis_id), CFNumber::from(*value as f64))
                    })
                })
                .collect();

            if !var_pairs.is_empty() {
                let var_dict: CFDictionary<CFNumber, CFNumber> =
                    CFDictionary::from_CFType_pairs(&var_pairs);

                unsafe {
                    use core_foundation::base::CFType;

                    let var_key = CFString::wrap_under_get_rule(kCTFontVariationAttribute);
                    let var_val = CFType::wrap_under_get_rule(var_dict.as_CFTypeRef());
                    let attrs: CFDictionary<CFString, CFType> =
                        CFDictionary::from_CFType_pairs(&[(var_key, var_val)]);

                    let desc = core_text::font_descriptor::new_from_attributes(&attrs);

                    #[link(name = "CoreText", kind = "framework")]
                    extern "C" {
                        fn CTFontCreateWithGraphicsFont(
                            graphicsFont: core_graphics::sys::CGFontRef,
                            size: CGFloat,
                            matrix: *const core_graphics::geometry::CGAffineTransform,
                            attributes: core_text::font_descriptor::CTFontDescriptorRef,
                        ) -> core_text::font::CTFontRef;
                    }

                    let font_ref = CTFontCreateWithGraphicsFont(
                        cg_font.as_ptr(),
                        font_size as CGFloat,
                        ptr::null(),
                        desc.as_concrete_TypeRef(),
                    );

                    return Ok(CTFont::wrap_under_create_rule(font_ref));
                }
            }
        }

        Ok(new_from_CGFont(&cg_font, font_size))
    }

    /// Get or create a cached CTFont
    ///
    /// Returns a CachedFont that keeps the font data alive alongside the CTFont.
    /// This prevents use-after-free crashes in CoreText.
    fn get_ct_font(
        &self,
        font: &Arc<dyn FontRef>,
        params: &LinraRenderParams,
    ) -> Result<Arc<CachedFont>> {
        let data = font.data();
        let cache_key = FontCacheKey::new(data, params.size, &params.variations);

        // Check cache
        {
            let cache = self.font_cache.read();
            if let Some(cached) = cache.peek(&cache_key) {
                return Ok(Arc::clone(cached));
            }
        }

        // Copy font data to keep it alive with the CTFont
        // This Arc is passed through create_ct_font -> create_cg_font -> CGDataProvider
        // ensuring a single Arc instance is used everywhere
        let data_arc: Arc<[u8]> = Arc::from(data);

        // Create new CTFont - pass Arc clone so CGDataProvider uses the same Arc
        let ct_font =
            Self::create_ct_font(Arc::clone(&data_arc), params.size as f64, &params.variations)?;

        // Wrap in CachedFont to ensure data outlives the CTFont
        // Note: data_arc is now held both here AND in CGDataProvider (same Arc)
        let cached_font = Arc::new(CachedFont {
            ct_font,
            _data: data_arc,
        });

        // Cache it
        {
            let mut cache = self.font_cache.write();
            cache.put(cache_key, Arc::clone(&cached_font));
        }

        Ok(cached_font)
    }

    /// Create attributed string with font and features
    fn create_attributed_string(
        text: &str,
        ct_font: &CTFont,
        params: &LinraRenderParams,
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

    /// Apply OpenType features and letter spacing to attributed string
    fn apply_features(
        attr_string: &mut CFMutableAttributedString,
        range: CFRange,
        params: &LinraRenderParams,
    ) {
        // Ligatures
        if let Some((_, value)) = params.features.iter().find(|(tag, _)| tag == "liga") {
            let ligature_value = CFNumber::from(if *value > 0 { 1 } else { 0 });
            attr_string.set_attribute(range, unsafe { kCTLigatureAttributeName }, &ligature_value);
        }

        // Kerning feature (disable kerning)
        if let Some((_, value)) = params.features.iter().find(|(tag, _)| tag == "kern") {
            if *value == 0 {
                let zero = CFNumber::from(0.0f64);
                attr_string.set_attribute(range, unsafe { kCTKernAttributeName }, &zero);
            }
        }

        // Letter spacing (tracking) - applied via kCTKernAttributeName
        // This adds extra spacing between each character pair
        if params.letter_spacing != 0.0 {
            let kern_value = CFNumber::from(params.letter_spacing as f64);
            attr_string.set_attribute(range, unsafe { kCTKernAttributeName }, &kern_value);
        }
    }

    /// Convert Color to CoreGraphics normalized floats
    fn color_to_rgb(color: &Color) -> (f64, f64, f64, f64) {
        (
            color.r as f64 / 255.0,
            color.g as f64 / 255.0,
            color.b as f64 / 255.0,
            color.a as f64 / 255.0,
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

        // Get or create cached font (keeps font data alive to prevent use-after-free)
        let cached_font = self.get_ct_font(&font, params)?;

        // Create attributed string using the CTFont from cache
        let attr_string = Self::create_attributed_string(text, &cached_font.ct_font, params);

        // Create CTLine - this performs shaping internally
        let line = CTLine::new_with_attributed_string(attr_string.as_concrete_TypeRef());

        // Get line metrics for sizing
        let typographic_bounds = line.get_typographic_bounds();
        let line_width = typographic_bounds.width;
        let ascent = typographic_bounds.ascent;
        let descent = typographic_bounds.descent;
        let line_height = ascent + descent;

        // Calculate canvas dimensions
        let padding = params.padding as f64;
        let width = ((line_width + padding * 2.0).ceil() as u32).max(1);
        let height = ((line_height + padding * 2.0).ceil() as u32).max(1);

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
        let color_space = CGColorSpace::create_device_rgb();
        let context = CGContext::create_bitmap_context(
            Some(buffer.as_mut_ptr() as *mut _),
            width as usize,
            height as usize,
            8,
            bytes_per_row,
            &color_space,
            core_graphics::base::kCGImageAlphaPremultipliedLast,
        );

        // Configure antialiasing
        context.set_should_antialias(params.antialias);
        context.set_should_smooth_fonts(params.antialias);

        // Fill background
        if let Some(bg_color) = &params.background {
            let (r, g, b, a) = Self::color_to_rgb(bg_color);
            context.set_rgb_fill_color(r, g, b, a);
            context.fill_rect(CGRect::new(
                &CGPoint::new(0.0, 0.0),
                &CGSize::new(width as f64, height as f64),
            ));
        } else {
            context.clear_rect(CGRect::new(
                &CGPoint::new(0.0, 0.0),
                &CGSize::new(width as f64, height as f64),
            ));
        }

        // Set text color
        let (r, g, b, a) = Self::color_to_rgb(&params.foreground);
        context.set_rgb_fill_color(r, g, b, a);

        // Position text at baseline
        // CoreGraphics uses bottom-left origin
        let baseline_y = padding + descent;
        let text_x = padding;

        context.save();
        context.set_text_drawing_mode(CGTextDrawingMode::CGTextFill);
        context.set_text_position(text_x, baseline_y);

        // THE KEY OPERATION: CTLineDraw shapes AND renders in one call
        line.draw(&context);

        context.restore();

        Ok(RenderOutput::Bitmap(BitmapData {
            width,
            height,
            format: BitmapFormat::Rgba8,
            data: buffer,
        }))
    }

    fn clear_cache(&self) {
        self.font_cache.write().clear();
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
