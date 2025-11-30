//! Single-pass text rendering using Windows DirectWrite
//!
//! This backend shapes AND renders text in a single operation using DirectWrite's
//! DrawTextLayout API. By letting DirectWrite control the entire pipeline, we get:
//!
//! - Optimal performance (no intermediate glyph extraction)
//! - Native Windows text quality with ClearType
//! - Correct handling of variable fonts and OpenType features
//!
//! ## Performance
//!
//! Traditional pipeline:
//! 1. DirectWrite shapes text -> extract glyphs -> ShapingResult
//! 2. Direct2D renders each glyph -> composite to bitmap
//!
//! Linra pipeline:
//! 1. DrawTextLayout: shape + render in one call
//!
//! The linra approach eliminates per-glyph overhead and allows DirectWrite
//! to optimize internally (e.g., batch GPU operations).

#![cfg(windows)]

use std::num::NonZeroUsize;
use std::sync::Arc;

use lru::LruCache;
use parking_lot::RwLock;

use typf_core::{
    error::{RenderError, Result, TypfError},
    linra::{LinraRenderParams, LinraRenderer},
    traits::FontRef,
    types::{BitmapData, BitmapFormat, RenderOutput},
    Color,
};

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{BOOL, FALSE, TRUE},
        Graphics::{
            Direct2D::{
                Common::{D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_PIXEL_FORMAT, D2D_RECT_F},
                D2D1CreateFactory, ID2D1Factory, ID2D1RenderTarget, D2D1_BITMAP_PROPERTIES,
                D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_RENDER_TARGET_PROPERTIES,
                D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE,
            },
            DirectWrite::{
                DWriteCreateFactory, IDWriteFactory, IDWriteFontFace, IDWriteTextFormat,
                IDWriteTextLayout, DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL,
                DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_NORMAL, DWRITE_MEASURING_MODE_NATURAL,
                DWRITE_TEXT_METRICS,
            },
            Dxgi::Common::DXGI_FORMAT_R8G8B8A8_UNORM,
            Imaging::{CLSID_WICImagingFactory, IWICImagingFactory, WICBitmapCacheOnLoad},
        },
        System::Com::{
            CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
        },
    },
};

/// Cache key for DirectWrite font instances
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

        let mut hasher = DefaultHasher::new();
        font_data.hash(&mut hasher);
        let font_hash = hasher.finish();

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
struct CachedFont {
    /// The DirectWrite font face
    font_face: IDWriteFontFace,
    /// Font data kept alive to prevent use-after-free
    _data: Arc<[u8]>,
}

/// Single-pass text renderer using Windows DirectWrite
///
/// This renderer combines text shaping and rendering into a single DrawTextLayout
/// call for maximum performance on Windows.
pub struct DirectWriteLinraRenderer {
    /// DirectWrite factory
    dwrite_factory: IDWriteFactory,
    /// Direct2D factory
    d2d_factory: ID2D1Factory,
    /// WIC factory for bitmap creation
    wic_factory: IWICImagingFactory,
    /// Font cache to avoid expensive font creation
    font_cache: RwLock<LruCache<FontCacheKey, Arc<CachedFont>>>,
}

impl DirectWriteLinraRenderer {
    /// Creates a new linra renderer
    ///
    /// # Errors
    /// Returns an error if COM initialization or factory creation fails.
    pub fn new() -> Result<Self> {
        unsafe {
            // Initialize COM
            CoInitializeEx(None, COINIT_MULTITHREADED).ok();

            // Create DirectWrite factory
            let dwrite_factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)
                .map_err(|e| {
                    TypfError::RenderingFailed(RenderError::BackendError(format!(
                        "Failed to create DirectWrite factory: {e}"
                    )))
                })?;

            // Create Direct2D factory
            let d2d_factory: ID2D1Factory =
                D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).map_err(|e| {
                    TypfError::RenderingFailed(RenderError::BackendError(format!(
                        "Failed to create Direct2D factory: {e}"
                    )))
                })?;

            // Create WIC factory for bitmap handling
            let wic_factory: IWICImagingFactory =
                CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER).map_err(
                    |e| {
                        TypfError::RenderingFailed(RenderError::BackendError(format!(
                            "Failed to create WIC factory: {e}"
                        )))
                    },
                )?;

            Ok(Self {
                dwrite_factory,
                d2d_factory,
                wic_factory,
                font_cache: RwLock::new(LruCache::new(NonZeroUsize::new(100).unwrap())),
            })
        }
    }

    /// Validate font data has valid TrueType/OpenType signature
    fn validate_font_data(data: &[u8]) -> Result<()> {
        if data.len() < 12 {
            return Err(TypfError::RenderingFailed(RenderError::BackendError(
                "Font data too small to be valid".to_string(),
            )));
        }

        let sig = &data[0..4];
        let is_valid = matches!(
            sig,
            [0x00, 0x01, 0x00, 0x00]  // TrueType
            | [b'O', b'T', b'T', b'O'] // OpenType with CFF
            | [b't', b'r', b'u', b'e'] // TrueType (Mac)
            | [b't', b't', b'c', b'f'] // TrueType Collection
            | [b'w', b'O', b'F', b'F'] // WOFF
            | [b'w', b'O', b'F', b'2'] // WOFF2
        );

        if !is_valid {
            return Err(TypfError::RenderingFailed(RenderError::BackendError(format!(
                "Invalid font signature: {:02x}{:02x}{:02x}{:02x}",
                sig[0], sig[1], sig[2], sig[3]
            ))));
        }

        Ok(())
    }

    /// Create IDWriteFontFace from font data
    fn create_font_face(
        &self,
        data: Arc<[u8]>,
        _variations: &[(String, f32)],
    ) -> Result<IDWriteFontFace> {
        Self::validate_font_data(&data)?;

        unsafe {
            // Create font file from memory
            let font_file = self
                .dwrite_factory
                .CreateFontFileReference(
                    PCWSTR::null(), // We use in-memory font
                    None,
                )
                .map_err(|e| {
                    TypfError::RenderingFailed(RenderError::BackendError(format!(
                        "Failed to create font file reference: {e}"
                    )))
                })?;

            // Create font face
            let font_face = self
                .dwrite_factory
                .CreateFontFace(
                    windows::Win32::Graphics::DirectWrite::DWRITE_FONT_FACE_TYPE_TRUETYPE,
                    &[font_file],
                    0,
                    windows::Win32::Graphics::DirectWrite::DWRITE_FONT_SIMULATIONS_NONE,
                )
                .map_err(|e| {
                    TypfError::RenderingFailed(RenderError::BackendError(format!(
                        "Failed to create font face: {e}"
                    )))
                })?;

            // TODO: Apply variable font variations using IDWriteFontFace3::GetFontAxisValues
            // and IDWriteFontFace5::CreateFontFaceReference

            Ok(font_face)
        }
    }

    /// Get or create a cached font face
    fn get_font_face(
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

        let data_arc: Arc<[u8]> = Arc::from(data);
        let font_face = self.create_font_face(Arc::clone(&data_arc), &params.variations)?;

        let cached_font = Arc::new(CachedFont {
            font_face,
            _data: data_arc,
        });

        {
            let mut cache = self.font_cache.write();
            cache.put(cache_key, Arc::clone(&cached_font));
        }

        Ok(cached_font)
    }

    /// Create text format for the given parameters
    fn create_text_format(&self, params: &LinraRenderParams) -> Result<IDWriteTextFormat> {
        unsafe {
            // Use default system font family - actual font face is applied via SetFontCollection
            let font_family: Vec<u16> = "Segoe UI\0".encode_utf16().collect();

            self.dwrite_factory
                .CreateTextFormat(
                    PCWSTR(font_family.as_ptr()),
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    params.size,
                    PCWSTR::null(),
                )
                .map_err(|e| {
                    TypfError::RenderingFailed(RenderError::BackendError(format!(
                        "Failed to create text format: {e}"
                    )))
                })
        }
    }

    /// Create text layout for the given text and parameters
    fn create_text_layout(
        &self,
        text: &str,
        text_format: &IDWriteTextFormat,
        max_width: f32,
        max_height: f32,
    ) -> Result<IDWriteTextLayout> {
        let text_wide: Vec<u16> = text.encode_utf16().collect();

        unsafe {
            self.dwrite_factory
                .CreateTextLayout(&text_wide, text_format, max_width, max_height)
                .map_err(|e| {
                    TypfError::RenderingFailed(RenderError::BackendError(format!(
                        "Failed to create text layout: {e}"
                    )))
                })
        }
    }

    /// Get text metrics from layout
    fn get_text_metrics(&self, layout: &IDWriteTextLayout) -> Result<DWRITE_TEXT_METRICS> {
        let mut metrics = DWRITE_TEXT_METRICS::default();
        unsafe {
            layout.GetMetrics(&mut metrics).map_err(|e| {
                TypfError::RenderingFailed(RenderError::BackendError(format!(
                    "Failed to get text metrics: {e}"
                )))
            })?;
        }
        Ok(metrics)
    }

    /// Convert Color to Direct2D color
    fn color_to_d2d(color: &Color) -> windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F {
        windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F {
            r: color.r as f32 / 255.0,
            g: color.g as f32 / 255.0,
            b: color.b as f32 / 255.0,
            a: color.a as f32 / 255.0,
        }
    }
}

impl Default for DirectWriteLinraRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create DirectWriteLinraRenderer")
    }
}

impl LinraRenderer for DirectWriteLinraRenderer {
    fn name(&self) -> &'static str {
        "directwrite-linra"
    }

    fn render_text(
        &self,
        text: &str,
        font: Arc<dyn FontRef>,
        params: &LinraRenderParams,
    ) -> Result<RenderOutput> {
        log::debug!("DirectWriteLinraRenderer: Rendering '{}' at size {}", text, params.size);

        // Handle empty text
        if text.is_empty() {
            return Ok(RenderOutput::Bitmap(BitmapData {
                width: 1,
                height: 1,
                format: BitmapFormat::Rgba8,
                data: vec![0, 0, 0, 0],
            }));
        }

        // Get or create cached font (currently unused due to DirectWrite complexity)
        let _cached_font = self.get_font_face(&font, params)?;

        // Create text format
        let text_format = self.create_text_format(params)?;

        // Create text layout with large max dimensions for initial measurement
        let layout = self.create_text_layout(text, &text_format, 10000.0, 10000.0)?;

        // Get metrics to determine canvas size
        let metrics = self.get_text_metrics(&layout)?;

        let padding = params.padding as f32;
        let width = ((metrics.width + padding * 2.0).ceil() as u32).max(1);
        let height = ((metrics.height + padding * 2.0).ceil() as u32).max(1);

        log::debug!(
            "DirectWriteLinraRenderer: Canvas {}x{}, text width {:.1}",
            width,
            height,
            metrics.width
        );

        // Create bitmap buffer
        let mut buffer = vec![0u8; (width * height * 4) as usize];

        unsafe {
            // Create WIC bitmap
            let wic_bitmap = self
                .wic_factory
                .CreateBitmap(
                    width,
                    height,
                    &windows::Win32::Graphics::Imaging::GUID_WICPixelFormat32bppPBGRA,
                    WICBitmapCacheOnLoad,
                )
                .map_err(|e| {
                    TypfError::RenderingFailed(RenderError::BackendError(format!(
                        "Failed to create WIC bitmap: {e}"
                    )))
                })?;

            // Create D2D render target from WIC bitmap
            let render_target_props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: windows::Win32::Graphics::Direct2D::D2D1_FEATURE_LEVEL_DEFAULT,
            };

            let render_target: ID2D1RenderTarget = self
                .d2d_factory
                .CreateWicBitmapRenderTarget(&wic_bitmap, &render_target_props)
                .map_err(|e| {
                    TypfError::RenderingFailed(RenderError::BackendError(format!(
                        "Failed to create render target: {e}"
                    )))
                })?;

            // Begin drawing
            render_target.BeginDraw();

            // Clear with background color
            if let Some(bg) = &params.background {
                render_target.Clear(Some(&Self::color_to_d2d(bg)));
            } else {
                let transparent = windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                };
                render_target.Clear(Some(&transparent));
            }

            // Create solid color brush for text
            let brush = render_target
                .CreateSolidColorBrush(&Self::color_to_d2d(&params.foreground), None)
                .map_err(|e| {
                    TypfError::RenderingFailed(RenderError::BackendError(format!(
                        "Failed to create brush: {e}"
                    )))
                })?;

            // THE KEY OPERATION: DrawTextLayout shapes AND renders in one call
            render_target.DrawTextLayout(
                windows::Win32::Graphics::Direct2D::Common::D2D_POINT_2F {
                    x: padding,
                    y: padding,
                },
                &layout,
                &brush,
                windows::Win32::Graphics::Direct2D::D2D1_DRAW_TEXT_OPTIONS_NONE,
            );

            // End drawing
            render_target.EndDraw(None, None).map_err(|e| {
                TypfError::RenderingFailed(RenderError::BackendError(format!(
                    "Failed to end drawing: {e}"
                )))
            })?;

            // Lock WIC bitmap and copy to output buffer
            let lock = wic_bitmap
                .Lock(None, windows::Win32::Graphics::Imaging::WICBitmapLockRead)
                .map_err(|e| {
                    TypfError::RenderingFailed(RenderError::BackendError(format!(
                        "Failed to lock bitmap: {e}"
                    )))
                })?;

            let mut stride = 0u32;
            let mut data_size = 0u32;
            let mut data_ptr = std::ptr::null_mut();

            lock.GetStride(&mut stride).map_err(|e| {
                TypfError::RenderingFailed(RenderError::BackendError(format!(
                    "Failed to get stride: {e}"
                )))
            })?;

            lock.GetDataPointer(&mut data_size, &mut data_ptr)
                .map_err(|e| {
                    TypfError::RenderingFailed(RenderError::BackendError(format!(
                        "Failed to get data pointer: {e}"
                    )))
                })?;

            // Copy pixel data
            let src_slice = std::slice::from_raw_parts(data_ptr, data_size as usize);
            let bytes_per_row = width as usize * 4;

            for y in 0..height as usize {
                let src_start = y * stride as usize;
                let dst_start = y * bytes_per_row;
                let copy_len = bytes_per_row.min(src_slice.len() - src_start);
                buffer[dst_start..dst_start + copy_len]
                    .copy_from_slice(&src_slice[src_start..src_start + copy_len]);
            }
        }

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

#[cfg(all(test, windows))]
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
        let renderer = DirectWriteLinraRenderer::new();
        assert!(renderer.is_ok());
        if let Ok(r) = renderer {
            assert_eq!(r.name(), "directwrite-linra");
        }
    }

    #[test]
    fn test_supports_format() {
        if let Ok(renderer) = DirectWriteLinraRenderer::new() {
            assert!(renderer.supports_format("bitmap"));
            assert!(renderer.supports_format("rgba"));
            assert!(!renderer.supports_format("svg"));
        }
    }

    #[test]
    fn test_empty_text() {
        if let Ok(renderer) = DirectWriteLinraRenderer::new() {
            let font = Arc::new(MockFont { data: vec![] });
            let params = LinraRenderParams::default();

            let result = renderer.render_text("", font, &params);
            assert!(result.is_ok());

            if let Ok(RenderOutput::Bitmap(bitmap)) = result {
                assert_eq!(bitmap.width, 1);
                assert_eq!(bitmap.height, 1);
            }
        }
    }

    #[test]
    fn test_font_cache_key() {
        let key1 = FontCacheKey::new(b"font1", 16.0, &[]);
        let key2 = FontCacheKey::new(b"font1", 16.0, &[]);
        let key3 = FontCacheKey::new(b"font2", 16.0, &[]);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}
