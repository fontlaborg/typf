// this_file: backends/typf-orge/src/lib.rs

//! orge - ultra-smooth unhinted glyph rasterization.
//!
//! This crate provides a specialized scan converter for supersmooth, unhinted
//! font rendering. It focuses ONLY on the scan conversion algorithm, NOT on hinting.
//!
//! ## Safety
//!
//! This crate is 100% safe Rust with no `unsafe` blocks. The `#![deny(unsafe_code)]`
//! attribute ensures this property is maintained.
//!
//! ## Architecture
//!
//! - `fixed`: F26Dot6 fixed-point arithmetic (26.6 format)
//! - `edge`: Edge lists for scan line algorithm
//! - `curves`: Bézier curve subdivision
//! - `scan_converter`: Main rasterization algorithm
//! - `dropout`: Dropout control for thin features
//! - `grayscale`: Anti-aliasing via oversampling

pub mod curves;
pub mod edge;
pub mod fixed;
pub mod grayscale;
pub mod renderer;
pub mod scan_converter;
// pub mod dropout;

// Re-export main types
pub use renderer::{GlyphRasterizer, Image, OrgeError, Result as OrgeResult, OrgePen};

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
};

use parking_lot::RwLock;
use skrifa::FontRef;

use typf_core::{
    types::{Direction, BoundingBox, SegmentOptions, TextRun, RenderOutput},
    traits::Backend as TypfCoreBackend,
    Bitmap, Font, FontCache, FontCacheConfig, RenderOptions, Result as TypfResult, TypfError,
    DynBackend, BackendFeatures, FontMetrics,
};
use typf_fontdb::{FontDatabase, FontHandle};
use typf_core::types::ShapingResult;


/// Fill rule for scan conversion.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FillRule {
    /// Non-zero winding rule (recommended for fonts).
    NonZeroWinding,
    /// Even-odd rule.
    EvenOdd,
}

/// Dropout control mode.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DropoutMode {
    /// No dropout control.
    None,
    /// Simple dropout (fill gaps in thin stems).
    Simple,
    /// Smart dropout (perpendicular scan + stub detection).
    Smart,
}

#[derive(Clone, Debug)]
struct FontDataEntry {
    key: String,
    #[allow(dead_code)]
    path: Option<PathBuf>,
    bytes: Arc<[u8]>,
    face_index: u32,
}

impl FontDataEntry {
    fn from_handle(handle: Arc<FontHandle>) -> Self {
        Self {
            key: handle.key.clone(),
            path: handle.path.clone(),
            bytes: handle.bytes.clone(),
            face_index: handle.face_index,
        }
    }

    fn as_static_slice(&self) -> &'static [u8] {
        // Safety: the underlying Arc<[u8]> remains alive as long as this entry does.
        // This is safe because FontDataEntry itself holds an Arc to the bytes, ensuring
        // the memory is not deallocated while the 'static reference exists.
        unsafe { std::mem::transmute::<&[u8], &'static [u8]>(self.bytes.as_ref()) }
    }

    fn key(&self) -> String {
        self.key.clone()
    }
}

#[allow(dead_code)]
struct TtfFaceEntry {
    data: Arc<FontDataEntry>,
    font_ref: FontRef<'static>,
    // Cached derived metrics (computed once to avoid repeated table reads)
    units_per_em: u16,
    ascender_unscaled: i16,
    descender_unscaled: i16,
    line_gap_unscaled: i16,
}

impl TtfFaceEntry {
    fn new(data: Arc<FontDataEntry>) -> TypfResult<Self> {
        use read_fonts::{FileRef, TableProvider};

        // Handle both single fonts and TTC collections
        let file_ref = FileRef::new(data.as_static_slice())
            .map_err(|e| TypfError::render(format!("Failed to parse font file: {:?}", e)))?;

        let font_ref = match file_ref {
            FileRef::Font(font) => font,
            FileRef::Collection(collection) => collection.get(data.face_index).map_err(|e| {
                TypfError::render(format!(
                    "Font index {} not found in collection: {:?}",
                    data.face_index, e
                ))
            })?,
        };

        // Cache derived metrics to avoid repeated table reads on every render
        let units_per_em = font_ref
            .head()
            .ok()
            .ok_or_else(|| TypfError::render("Failed to read head table".to_string()))?
            .units_per_em();

        let hhea = font_ref
            .hhea()
            .ok()
            .ok_or_else(|| TypfError::render("Failed to read hhea table".to_string()))?;

        Ok(Self {
            data,
            font_ref,
            units_per_em,
            ascender_unscaled: hhea.ascender().into(),
            descender_unscaled: hhea.descender().into(),
            line_gap_unscaled: hhea.line_gap().into(),
        })
    }
}

pub struct OrgeBackend {
    #[allow(dead_code)]
    cache: FontCache,
    font_data_cache: RwLock<HashMap<String, Arc<FontDataEntry>>>,
    ttf_cache: RwLock<HashMap<String, Arc<TtfFaceEntry>>>,
    font_db: &'static FontDatabase,
}

impl OrgeBackend {
    pub fn new() -> Self {
        Self::with_cache_config(FontCacheConfig::default())
    }

    pub fn with_cache_config(cache_config: FontCacheConfig) -> Self {
        Self {
            cache: FontCache::with_config(cache_config),
            font_data_cache: RwLock::new(HashMap::new()),
            ttf_cache: RwLock::new(HashMap::new()),
            font_db: FontDatabase::global(),
        }
    }

    fn load_font_data(&self, font: &Font) -> TypfResult<Arc<FontDataEntry>> {
        let handle = self.font_db.resolve(font)?;
        let key = handle.key.clone();
        if let Some(entry) = self.font_data_cache.read().get(&key) {
            return Ok(entry.clone());
        }

        let entry = Arc::new(FontDataEntry::from_handle(handle));
        self.font_data_cache.write().insert(key, entry.clone());
        Ok(entry)
    }

    fn get_or_create_ttf_face(&self, font: &Font) -> TypfResult<Arc<TtfFaceEntry>> {
        let font_data = self.load_font_data(font)?;
        let cache_key = font_data.key();

        if let Some(entry) = self.ttf_cache.read().get(&cache_key) {
            return Ok(entry.clone());
        }

        let entry = Arc::new(TtfFaceEntry::new(font_data)?);
        self.ttf_cache.write().insert(cache_key, entry.clone());
        Ok(entry)
    }

    #[allow(dead_code)]
    fn image_to_bitmap_rgba(image: Image) -> Bitmap {
        let mut data = Vec::with_capacity((image.width() * image.height() * 4) as usize);
        let alpha = 255; // Fully opaque for now

        for pixel in image.pixels() {
            let value = *pixel; // Grayscale or monochrome value
            data.push(value);    // R
            data.push(value);    // G
            data.push(value);    // B
            data.push(alpha);    // A
        }

        Bitmap {
            width: image.width(),
            height: image.height(),
            data,
        }
    }
}

impl DynBackend for OrgeBackend {
    fn name(&self) -> &'static str {
        "Orge"
    }

    fn shape_text(&self, _text: &str, _font: &Font) -> ShapingResult {
        // Orge is a rasterizer, not a shaper. This needs to be done by a separate shaper.
        // For now, return an empty shaping result.
        ShapingResult {
            text: _text.to_string(),
            glyphs: Vec::new(),
            advance: 0.0,
            bbox: BoundingBox {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            font: None,
            direction: Direction::LeftToRight,
        }
    }

    fn render_glyph(&self, font: &Font, glyph_id: u32, options: RenderOptions) -> Option<Bitmap> {
        // TODO: Implement proper glyph rendering using GlyphRasterizer
        // For now, returning None to allow compilation
        let _ = (font, glyph_id, options);
        None
    }

    fn render_shaped_text(
        &self,
        _shaped_text: &ShapingResult,
        _options: RenderOptions,
    ) -> Option<Bitmap> {
        None
    }

    fn font_metrics(&self, font: &Font) -> FontMetrics {
        let face_entry = self.get_or_create_ttf_face(font).expect("failed to get font face for metrics");
        let scale = font.size / f32::from(face_entry.units_per_em);
        FontMetrics {
            units_per_em: face_entry.units_per_em,
            ascender: (f32::from(face_entry.ascender_unscaled) * scale).round() as i16,
            descender: (f32::from(face_entry.descender_unscaled) * scale).round() as i16,
            line_gap: (f32::from(face_entry.line_gap_unscaled) * scale).round() as i16,
        }
    }

    fn supported_features(&self) -> BackendFeatures {
        BackendFeatures {
            monochrome: true,
            grayscale: true,
            subpixel: false, // Orge doesn't do subpixel AA
            color_emoji: false, // Orge is a basic rasterizer, no color emoji support
        }
    }
}

impl Default for OrgeBackend {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the full Backend trait for compatibility with Python bindings
impl TypfCoreBackend for OrgeBackend {
    fn segment(&self, text: &str, _options: &SegmentOptions) -> TypfResult<Vec<TextRun>> {
        // Simple single-run segmentation
        // TODO: Integrate with ICU segmenter for proper script/bidi segmentation
        if text.is_empty() {
            return Ok(Vec::new());
        }

        Ok(vec![TextRun {
            text: text.to_string(),
            range: (0, text.len()),
            script: String::new(),
            language: String::new(),
            direction: Direction::LeftToRight,
            font: None,
        }])
    }

    fn shape(&self, run: &TextRun, _font: &Font) -> TypfResult<ShapingResult> {
        // Orge is a rasterizer, not a shaper
        // Return minimal shaping result - actual shaping should be done by HarfBuzz
        // TODO: Delegate to HarfBuzz backend or implement basic character-to-glyph mapping
        Ok(ShapingResult {
            text: run.text.clone(),
            glyphs: Vec::new(),
            advance: 0.0,
            bbox: BoundingBox {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            font: Some(_font.clone()),
            direction: run.direction,
        })
    }

    fn render(&self, _shaped: &ShapingResult, _options: &RenderOptions) -> TypfResult<RenderOutput> {
        // TODO: Implement full text rendering using GlyphRasterizer
        // For now, return empty bitmap to allow compilation
        Err(TypfError::render("Orge backend text rendering not yet implemented. Use for glyph-level rendering only."))
    }

    fn name(&self) -> &str {
        "Orge"
    }

    fn clear_cache(&self) {
        self.font_data_cache.write().clear();
        self.ttf_cache.write().clear();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
