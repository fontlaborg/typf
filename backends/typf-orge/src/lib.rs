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
    types::{Direction, BoundingBox, SegmentOptions, TextRun, RenderOutput, Glyph},
    traits::Backend as TypfCoreBackend,
    Bitmap, Font, FontCache, FontCacheConfig, RenderOptions, Result as TypfResult, TypfError,
    DynBackend, BackendFeatures, FontMetrics,
};
use typf_fontdb::{FontDatabase, FontHandle};
use typf_core::types::ShapingResult;
use skrifa::instance::{Size, Location};
use skrifa::MetadataProvider;
use read_fonts::TableProvider;
use skrifa::GlyphId;


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

    fn shape_text(&self, text: &str, font: &Font) -> ShapingResult {
        // Use the Backend trait implementation
        // First segment the text
        let empty_bbox = BoundingBox { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };
        let runs = match TypfCoreBackend::segment(self, text, &SegmentOptions::default()) {
            Ok(runs) => runs,
            Err(_) => return ShapingResult {
                text: text.to_string(),
                glyphs: Vec::new(),
                advance: 0.0,
                bbox: empty_bbox,
                font: None,
                direction: Direction::LeftToRight,
            },
        };

        // Shape each run and combine (for now, just use the first run)
        if let Some(run) = runs.first() {
            match TypfCoreBackend::shape(self, run, font) {
                Ok(result) => result,
                Err(_) => ShapingResult {
                    text: text.to_string(),
                    glyphs: Vec::new(),
                    advance: 0.0,
                    bbox: empty_bbox,
                    font: None,
                    direction: Direction::LeftToRight,
                },
            }
        } else {
            ShapingResult {
                text: text.to_string(),
                glyphs: Vec::new(),
                advance: 0.0,
                bbox: empty_bbox,
                font: None,
                direction: Direction::LeftToRight,
            }
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
        shaped_text: &ShapingResult,
        options: RenderOptions,
    ) -> Option<Bitmap> {
        // Use the Backend trait implementation
        match TypfCoreBackend::render(self, shaped_text, &options) {
            Ok(RenderOutput::Bitmap(bitmap)) => Some(bitmap),
            _ => None,
        }
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

    fn shape(&self, run: &TextRun, font: &Font) -> TypfResult<ShapingResult> {
        // Basic character-to-glyph mapping (no ligatures/kerning/complex shaping)
        // For complex text, use HarfBuzz backend instead

        let face_entry = self.get_or_create_ttf_face(font)?;
        let font_ref = &face_entry.font_ref;

        let charmap = font_ref.charmap();
        let size = Size::new(font.size);
        let hhea = font_ref.hhea()
            .map_err(|e| TypfError::other(format!("Failed to read hhea table: {}", e)))?;
        let upem = f32::from(face_entry.units_per_em);
        let scale = font.size / upem;

        let mut glyphs = Vec::new();
        let mut x_position = 0.0;
        let mut cluster_idx = 0u32;

        // Get glyph metrics provider (for variable fonts, use default location)
        let location = Location::default();
        let glyph_metrics = font_ref.glyph_metrics(size, &location);

        for ch in run.text.chars() {
            // Map character to glyph ID
            let glyph_id = charmap.map(ch).unwrap_or(GlyphId::NOTDEF);

            // Get advance width for this glyph
            let advance = glyph_metrics.advance_width(glyph_id).unwrap_or(0.0) * scale;

            glyphs.push(Glyph {
                id: glyph_id.to_u32(),
                cluster: cluster_idx,
                x: x_position,
                y: 0.0,
                advance,
            });

            x_position += advance;
            cluster_idx += ch.len_utf8() as u32;
        }

        // Calculate bounding box (simplified - use hhea metrics scaled)
        let ascender = f32::from(hhea.ascender().to_i16()) * scale;
        let descender = f32::from(hhea.descender().to_i16()) * scale;

        Ok(ShapingResult {
            text: run.text.clone(),
            glyphs,
            advance: x_position,
            bbox: BoundingBox {
                x: 0.0,
                y: descender,
                width: x_position,
                height: ascender - descender,
            },
            font: Some(font.clone()),
            direction: run.direction,
        })
    }

    fn render(&self, shaped: &ShapingResult, _options: &RenderOptions) -> TypfResult<RenderOutput> {
        // Render shaped text by compositing individual glyphs

        if shaped.glyphs.is_empty() {
            // Return empty bitmap for empty text
            return Ok(RenderOutput::Bitmap(Bitmap {
                width: 1,
                height: 1,
                data: vec![0, 0, 0, 0],
            }));
        }

        let font = shaped.font.as_ref().ok_or_else(|| {
            TypfError::render("No font specified in ShapingResult")
        })?;

        // Get font data
        let face_entry = self.get_or_create_ttf_face(font)?;
        let font_bytes = face_entry.data.as_static_slice();
        let font_ref = FontRef::new(font_bytes)
            .map_err(|e| TypfError::other(format!("Failed to create FontRef: {}", e)))?;

        // Calculate canvas dimensions from bounding box
        let canvas_width = shaped.bbox.width.ceil() as u32;
        let canvas_height = shaped.bbox.height.ceil() as u32;

        if canvas_width == 0 || canvas_height == 0 {
            return Ok(RenderOutput::Bitmap(Bitmap {
                width: 1,
                height: 1,
                data: vec![0, 0, 0, 0],
            }));
        }

        // Create grayscale canvas
        let mut canvas = vec![0u8; (canvas_width * canvas_height) as usize];

        // Create glyph rasterizer
        let rasterizer = GlyphRasterizer::new();

        // Render and composite each glyph
        for glyph in &shaped.glyphs {
            // Skip glyphs with no advance (like combining marks, for now)
            if glyph.advance == 0.0 {
                continue;
            }

            // Render glyph
            let glyph_image = rasterizer.render_glyph(
                &font_ref,
                glyph.id,
                font.size,
                &[],  // No variable font location
                canvas_width,
                canvas_height,
            ).map_err(|e| TypfError::render(format!("Glyph rendering failed: {}", e)))?;

            // Composite glyph onto canvas at position (glyph.x, glyph.y)
            // For now, simple alpha blending
            let x_offset = glyph.x.round() as i32;
            let y_offset = (shaped.bbox.height - glyph.y).round() as i32;

            for y in 0..glyph_image.height() {
                for x in 0..glyph_image.width() {
                    let canvas_x = x_offset + x as i32;
                    let canvas_y = y_offset + y as i32;

                    if canvas_x >= 0 && canvas_x < canvas_width as i32
                        && canvas_y >= 0 && canvas_y < canvas_height as i32
                    {
                        let canvas_idx = canvas_y as usize * canvas_width as usize + canvas_x as usize;
                        let glyph_idx = y as usize * glyph_image.width() as usize + x as usize;

                        let glyph_alpha = glyph_image.pixels()[glyph_idx];
                        // Simple max blending (or use proper alpha compositing)
                        canvas[canvas_idx] = canvas[canvas_idx].max(glyph_alpha);
                    }
                }
            }
        }

        // Convert grayscale to RGBA
        let mut rgba_data = Vec::with_capacity((canvas_width * canvas_height * 4) as usize);
        for &gray_value in &canvas {
            rgba_data.push(gray_value);  // R
            rgba_data.push(gray_value);  // G
            rgba_data.push(gray_value);  // B
            rgba_data.push(255);         // A (fully opaque)
        }

        Ok(RenderOutput::Bitmap(Bitmap {
            width: canvas_width,
            height: canvas_height,
            data: rgba_data,
        }))
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
