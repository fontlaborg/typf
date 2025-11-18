// this_file: backends/typf-icu-hb/src/lib.rs

//! ICU+HarfBuzz backend for cross-platform text rendering.

pub mod renderer;
pub mod shaping;

// Re-export shaping types for convenience
pub use shaping::{ShapeRequest, ShapedText, ShapingError, TextShaper};

use harfbuzz_rs::{Face as HbFace, Font as HbFont, Language, Owned, Tag, UnicodeBuffer};
use kurbo::{BezPath, Shape};

// PathEl only needed for legacy helper functions
#[cfg(feature = "tiny-skia-renderer")]
use kurbo::PathEl;
use lru::LruCache;
use parking_lot::RwLock;
use skrifa::{FontRef, GlyphId};
use std::collections::HashMap;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use typf_core::{
    cache::{FontKey, GlyphKey, RenderedGlyph},
    types::{Direction, FontSource, RenderFormat},
    utils::{calculate_bbox, quantize_size},
    traits::Backend as TypfCoreBackend, Bitmap, Font, FontCache, FontCacheConfig, Glyph, RenderOptions, RenderOutput,
    RenderSurface, Result, SegmentOptions, ShapingResult, TextRun, TypfError,
    DynBackend, BackendFeatures, FontMetrics,
};
use typf_fontdb::{script_fallbacks, FontDatabase, FontHandle};
#[cfg(feature = "tiny-skia-renderer")]
use typf_render::outlines::glyph_bez_path;
use typf_render::outlines::glyph_bez_path_with_variations;
use typf_unicode::TextSegmenter;
use skrifa::raw::TableProvider;

// tiny-skia is always available for image compositing
use tiny_skia::{Color, Pixmap, PixmapPaint, PixmapRef, Transform};

// Legacy tiny-skia specific types (for legacy helper functions)
#[cfg(feature = "tiny-skia-renderer")]
use tiny_skia::{Path as SkiaPath, PathBuilder};

/// Structured cache key for HarfBuzz font instances.
/// Avoids string formatting on hot path.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct HbCacheKey {
    font_key: String,
    size_quantized: u32,
    variations: Vec<(String, u32)>, // Sorted by axis name
}

impl HbCacheKey {
    fn new(font_key: String, size: f32, variations: &HashMap<String, f32>) -> Self {
        let size_quantized = quantize_size(size);

        let mut variations_vec: Vec<(String, u32)> = variations
            .iter()
            .map(|(k, v)| (k.clone(), (v * 1000.0) as u32)) // Quantize to 0.001 precision
            .collect();
        variations_vec.sort_by(|a, b| a.0.cmp(&b.0)); // Sort by axis name

        Self {
            font_key,
            size_quantized,
            variations: variations_vec,
        }
    }
}

pub struct HarfBuzzBackend {
    cache: FontCache,
    hb_cache: RwLock<LruCache<HbCacheKey, Arc<HbFontEntry>>>,
    ttf_cache: RwLock<HashMap<String, Arc<TtfFaceEntry>>>,
    font_data_cache: RwLock<HashMap<String, Arc<FontDataEntry>>>,
    font_db: &'static FontDatabase,
    segmenter: TextSegmenter,
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
        unsafe { std::mem::transmute::<&[u8], &'static [u8]>(self.bytes.as_ref()) }
    }

    fn key(&self) -> String {
        self.key.clone()
    }

    fn font_key(&self) -> FontKey {
        FontKey {
            path: PathBuf::from(&self.key),
            face_index: self.face_index,
        }
    }
}

#[derive(Debug)]
struct HbFontEntry {
    #[allow(dead_code)]
    data: Arc<FontDataEntry>,
    font: Owned<HbFont<'static>>,
}

impl HbFontEntry {
    fn new(data: Arc<FontDataEntry>, size: f32, variations: &HashMap<String, f32>) -> Result<Self> {
        let hb_face = HbFace::new(data.bytes.clone(), data.face_index);
        let mut hb_font = HbFont::new(hb_face);

        let scale = (size * 64.0).max(1.0) as i32;
        hb_font.set_scale(scale, scale);

        // Apply variable font variations if any
        if !variations.is_empty() {
            let variations_vec: Vec<_> = variations
                .iter()
                .filter_map(|(name, value)| {
                    let bytes = name.as_bytes();
                    if bytes.len() != 4 {
                        return None;
                    }
                    let tag = Tag::new(
                        bytes[0] as char,
                        bytes[1] as char,
                        bytes[2] as char,
                        bytes[3] as char,
                    );
                    Some(harfbuzz_rs::Variation::new(tag, *value))
                })
                .collect();
            hb_font.set_variations(&variations_vec);
        }

        Ok(Self {
            data,
            font: hb_font,
        })
    }

    fn font(&self) -> &HbFont<'static> {
        &self.font
    }
}

struct TtfFaceEntry {
    data: Arc<FontDataEntry>,
    font_ref: FontRef<'static>,
    // Cached derived metrics (computed once to avoid repeated table reads)
    units_per_em: u16,
    ascender_unscaled: i16,
}

impl TtfFaceEntry {
    fn new(data: Arc<FontDataEntry>) -> Result<Self> {
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

        let ascender_unscaled = font_ref
            .hhea()
            .ok()
            .ok_or_else(|| TypfError::render("Failed to read hhea table".to_string()))?
            .ascender()
            .to_i16();

        Ok(Self {
            data,
            font_ref,
            units_per_em,
            ascender_unscaled,
        })
    }

    fn font_ref(&self) -> &FontRef<'static> {
        &self.font_ref
    }

    fn font_key(&self) -> FontKey {
        self.data.font_key()
    }

    /// Validate and clamp variable font coordinates to their defined ranges.
    fn validate_variations(&self, variations: &HashMap<String, f32>) -> HashMap<String, f32> {
        use skrifa::MetadataProvider;

        if variations.is_empty() {
            return HashMap::new();
        }

        let mut validated = HashMap::new();

        // Get available axes from the font
        let axes = self.font_ref.axes();

        for (name, &user_value) in variations {
            // Try to match axis by tag (4-char string)
            if name.len() != 4 {
                log::warn!(
                    "Ignoring invalid axis name '{}' (must be 4 characters)",
                    name
                );
                continue;
            }

            let tag_bytes = name.as_bytes();

            // Find the matching axis in the font
            let mut found = false;
            for axis in axes.iter() {
                let axis_tag = axis.tag();
                if axis_tag.to_be_bytes() == tag_bytes {
                    // Clamp to the axis's min/max range
                    let min = axis.min_value();
                    let max = axis.max_value();
                    let clamped = user_value.clamp(min, max);

                    if clamped != user_value {
                        log::warn!(
                            "Axis '{}' value {} clamped to range [{}, {}] → {}",
                            name,
                            user_value,
                            min,
                            max,
                            clamped
                        );
                    }

                    validated.insert(name.clone(), clamped);
                    found = true;
                    break;
                }
            }

            if !found {
                log::warn!("Unknown axis '{}' for font, ignoring", name);
            }
        }

        validated
    }
}

impl HarfBuzzBackend {
    pub fn new() -> Self {
        Self::with_cache_config(FontCacheConfig::default())
    }

    pub fn with_cache_config(cache_config: FontCacheConfig) -> Self {
        Self {
            cache: FontCache::with_config(cache_config),
            hb_cache: RwLock::new(LruCache::new(NonZeroUsize::new(64).unwrap())),
            ttf_cache: RwLock::new(HashMap::new()),
            font_data_cache: RwLock::new(HashMap::new()),
            font_db: FontDatabase::global(),
            segmenter: TextSegmenter::new(),
        }
    }

    fn load_font_data(&self, font: &Font) -> Result<Arc<FontDataEntry>> {
        let handle = self.font_db.resolve(font)?;
        let key = handle.key.clone();
        if let Some(entry) = self.font_data_cache.read().get(&key) {
            return Ok(entry.clone());
        }

        let entry = Arc::new(FontDataEntry::from_handle(handle));
        self.font_data_cache.write().insert(key, entry.clone());
        Ok(entry)
    }
    fn get_or_create_ttf_face(&self, font: &Font) -> Result<Arc<TtfFaceEntry>> {
        let font_data = self.load_font_data(font)?;
        let cache_key = font_data.key();

        if let Some(entry) = self.ttf_cache.read().get(&cache_key) {
            return Ok(entry.clone());
        }

        let entry = Arc::new(TtfFaceEntry::new(font_data)?);
        self.ttf_cache.write().insert(cache_key, entry.clone());
        Ok(entry)
    }

    fn get_or_create_hb_font(&self, font: &Font) -> Result<Arc<HbFontEntry>> {
        let font_data = self.load_font_data(font)?;

        // Validate and clamp variations using skrifa's axis metadata
        let validated_variations = if !font.variations.is_empty() {
            let ttf_face = self.get_or_create_ttf_face(font)?;
            ttf_face.validate_variations(&font.variations)
        } else {
            HashMap::new()
        };

        // Use structured cache key (avoids string formatting on hot path)
        let cache_key = HbCacheKey::new(font_data.key(), font.size, &validated_variations);

        {
            let mut cache = self.hb_cache.write();
            if let Some(entry) = cache.get(&cache_key) {
                return Ok(entry.clone());
            }
        }

        let entry = Arc::new(HbFontEntry::new(
            font_data,
            font.size,
            &validated_variations,
        )?);
        {
            let mut cache = self.hb_cache.write();
            cache.push(cache_key, entry.clone());
        }
        Ok(entry)
    }

    fn resolve_run_font(&self, run: &TextRun, requested: &Font) -> Font {
        if let Some(run_font) = run.font.as_ref() {
            if self.font_supports_run(run_font, run) {
                return run_font.clone();
            }
        }

        if self.font_supports_run(requested, run) {
            return requested.clone();
        }

        for candidate in script_fallbacks(&run.script) {
            let mut fallback = requested.clone();
            fallback.family = candidate.to_string();
            fallback.source = FontSource::Family(candidate.to_string());
            if self.font_supports_run(&fallback, run) {
                return fallback;
            }
        }

        log::warn!(
            "No fallback font found for script '{}' using '{}'; falling back to specified font",
            run.script,
            requested.family
        );
        requested.clone()
    }

    fn font_supports_run(&self, font: &Font, run: &TextRun) -> bool {
        use skrifa::MetadataProvider;
        match self.get_or_create_ttf_face(font) {
            Ok(entry) => run
                .text
                .chars()
                .all(|ch| entry.font_ref().charmap().map(ch).is_some()),
            Err(_) => false,
        }
    }

    fn rasterize_glyph(
        &self,
        font_ref: &FontRef<'static>,
        glyph: &Glyph,
        size: f32,
        scale: f32,
        variations: &HashMap<String, f32>,
        antialias: bool,
    ) -> Option<RenderedGlyph> {
        // Get BezPath outline from skrifa
        let gid = u16::try_from(glyph.id).ok()?;
        let path = match glyph_bez_path_with_variations(
            font_ref,
            GlyphId::from(gid),
            size,
            scale,
            Some(variations),
        ) {
            Some(p) => p,
            None => return Some(blank_rendered_glyph()),
        };

        let bounds = path.bounding_box();
        if bounds.width() <= 0.0 || bounds.height() <= 0.0 {
            return Some(blank_rendered_glyph());
        }

        // Calculate bitmap dimensions
        let width = bounds.width().ceil().max(1.0) as u32;
        let height = bounds.height().ceil().max(1.0) as u32;

        // Translate path to origin for rendering
        use kurbo::Affine;
        let translation = Affine::translate((-bounds.x0, -bounds.y0));
        let mut translated = BezPath::new();
        for el in path.iter() {
            translated.push(translation * el);
        }

        // Use the renderer abstraction
        let renderer = renderer::create_renderer();
        let rendered = match renderer.render_glyph(&translated, width, height, antialias) {
            Some(mut r) => {
                // Restore original bounds (renderers return 0,0 for left/top)
                r.left = bounds.x0 as f32;
                r.top = bounds.y0 as f32;
                r
            }
            None => {
                // Renderer failed - return blank glyph to maintain cache consistency
                blank_rendered_glyph()
            }
        };

        Some(rendered)
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_cached_glyph(
        &self,
        target: &mut Pixmap,
        glyph: &Glyph,
        cached: &RenderedGlyph,
        baseline_y: f32,
        padding: f32,
        scratch: &mut Vec<u8>,
        base_r: u16,
        base_g: u16,
        base_b: u16,
        text_alpha: u8,
    ) {
        if cached.width == 0 || cached.height == 0 {
            return;
        }

        let pixels = (cached.width * cached.height) as usize;
        let required = pixels * 4;
        scratch.clear();
        scratch.resize(required, 0);

        let alpha_component = u16::from(text_alpha);
        for (idx, coverage) in cached.bitmap.iter().enumerate() {
            let cov = u16::from(*coverage);
            let offset = idx * 4;
            scratch[offset] = ((base_r * cov + 127) / 255) as u8;
            scratch[offset + 1] = ((base_g * cov + 127) / 255) as u8;
            scratch[offset + 2] = ((base_b * cov + 127) / 255) as u8;
            scratch[offset + 3] = ((alpha_component * cov + 127) / 255) as u8;
        }

        let Some(pixmap_ref) =
            PixmapRef::from_bytes(&scratch[..required], cached.width, cached.height)
        else {
            return;
        };

        let dest_x = glyph.x + padding + cached.left;
        let dest_y = baseline_y + cached.top;
        let base_x = dest_x.floor() as i32;
        let base_y = dest_y.floor() as i32;
        let frac_x = dest_x - base_x as f32;
        let frac_y = dest_y - base_y as f32;

        let paint = PixmapPaint::default();
        target.draw_pixmap(
            base_x,
            base_y,
            pixmap_ref,
            &paint,
            Transform::from_translate(frac_x, frac_y),
            None,
        );
    }

    fn script_tag(script: &str) -> Tag {
        let lower = script.to_ascii_lowercase();
        match lower.as_str() {
            "latin" => Tag::new('L', 'a', 't', 'n'),
            "arabic" => Tag::new('A', 'r', 'a', 'b'),
            "hebrew" => Tag::new('H', 'e', 'b', 'r'),
            "cyrillic" => Tag::new('C', 'y', 'r', 'l'),
            "greek" => Tag::new('G', 'r', 'e', 'k'),
            "han" => Tag::new('H', 'a', 'n', 'i'),
            "hiragana" => Tag::new('H', 'i', 'r', 'a'),
            "katakana" => Tag::new('K', 'a', 'n', 'a'),
            "thai" => Tag::new('T', 'h', 'a', 'i'),
            "devanagari" => Tag::new('D', 'e', 'v', 'a'),
            _ => Tag::new('L', 'a', 't', 'n'),
        }
    }
}

impl TypfCoreBackend for HarfBuzzBackend {
    fn segment(&self, text: &str, options: &SegmentOptions) -> Result<Vec<TextRun>> {
        self.segmenter.segment(text, options)
    }

    fn shape(&self, run: &TextRun, font: &Font) -> Result<ShapingResult> {
        let resolved_font = self.resolve_run_font(run, font);
        let hb_entry = self.get_or_create_hb_font(&resolved_font)?;
        let hb_font = hb_entry.font();

        // Create script tag from script name
        let script_tag = Self::script_tag(&run.script);

        // Create HarfBuzz buffer
        let buffer = UnicodeBuffer::new()
            .add_str(&run.text)
            .set_direction(match run.direction {
                Direction::LeftToRight => harfbuzz_rs::Direction::Ltr,
                Direction::RightToLeft => harfbuzz_rs::Direction::Rtl,
                Direction::Auto => harfbuzz_rs::Direction::Ltr,
            })
            .set_script(script_tag)
            .set_language(Language::from_str(&run.language).unwrap_or_default());

        // Shape the text
        let output = harfbuzz_rs::shape(hb_font, buffer, &[]);

        // Extract glyph information
        let mut glyphs = Vec::new();
        let mut x_pos = 0.0;
        let scale = font.size / hb_font.face().upem() as f32;

        let positions = output.get_glyph_positions();
        let infos = output.get_glyph_infos();

        for (info, pos) in infos.iter().zip(positions.iter()) {
            glyphs.push(Glyph {
                id: info.codepoint,
                cluster: info.cluster,
                x: x_pos + (pos.x_offset as f32 * scale),
                y: pos.y_offset as f32 * scale,
                advance: pos.x_advance as f32 * scale,
            });
            x_pos += pos.x_advance as f32 * scale;
        }

        let bbox = calculate_bbox(&glyphs);

        Ok(ShapingResult {
            text: run.text.clone(),
            glyphs,
            advance: x_pos,
            bbox,
            font: Some(resolved_font),
            direction: run.direction,
        })
    }

    fn render(&self, shaped: &ShapingResult, options: &RenderOptions) -> Result<RenderOutput> {
        // Diagnostics removed for simplicity
        // Check if we have glyphs to render
        if shaped.glyphs.is_empty() {
            return Ok(RenderOutput::Bitmap(Bitmap {
                width: 1,
                height: 1,
                data: vec![0, 0, 0, 0],
            }));
        }

        // Get the font from ShapingResult
        let font = shaped
            .font
            .as_ref()
            .ok_or_else(|| TypfError::render("Font information missing from shaped result".to_string()))?;

        // Get the font for glyph rendering
        let face_entry = self.get_or_create_ttf_face(font)?;
        let validated_variations = face_entry.validate_variations(&font.variations);

        // Calculate image dimensions
        let padding = options.padding as f32;
        let width = (shaped.bbox.width + padding * 2.0).ceil() as u32;
        let height = (shaped.bbox.height + padding * 2.0).ceil() as u32;

        // Create pixmap
        let mut pixmap = Pixmap::new(width, height)
            .ok_or_else(|| TypfError::render("Failed to create pixmap".to_string()))?;

        // Parse colors
        let (text_r, text_g, text_b, text_a) =
            typf_core::utils::parse_color(&options.color).map_err(TypfError::render)?;

        // Fill background if not transparent
        if options.background != "transparent" {
            let (bg_r, bg_g, bg_b, bg_a) =
                typf_core::utils::parse_color(&options.background).map_err(TypfError::render)?;
            pixmap.fill(Color::from_rgba8(bg_r, bg_g, bg_b, bg_a));
        }

        // Calculate scale factor (use cached metrics from TtfFaceEntry)
        let scale = font.size / face_entry.units_per_em as f32;

        // Calculate baseline position
        // The bbox.y is typically negative (representing ascent above baseline).
        // Position baseline so glyphs render from the top: padding + abs(bbox.y)
        let baseline_y = padding + (-shaped.bbox.y).max(0.0);

        let font_key = face_entry.font_key();
        let glyph_size = quantize_size(font.size);
        let mut scratch_rgba = Vec::new();
        let base_r = (u16::from(text_r) * u16::from(text_a) + 127) / 255;
        let base_g = (u16::from(text_g) * u16::from(text_a) + 127) / 255;
        let base_b = (u16::from(text_b) * u16::from(text_a) + 127) / 255;

        // Render each glyph using the shared glyph cache
        for glyph in &shaped.glyphs {
            let glyph_key = GlyphKey::new(
                font_key.clone(),
                glyph.id,
                glyph_size,
                &validated_variations,
            );

            let cached = if let Some(entry) = self.cache.get_glyph(&glyph_key) {
                entry
            } else {
                match self.rasterize_glyph(
                    &face_entry.font_ref,
                    glyph,
                    font.size,
                    scale,
                    &validated_variations,
                    options.antialias != typf_core::types::AntialiasMode::None,
                ) {
                    Some(rendered) => self.cache.cache_glyph(glyph_key.clone(), rendered),
                    None => continue,
                }
            };

            self.draw_cached_glyph(
                &mut pixmap,
                glyph,
                cached.as_ref(),
                baseline_y,
                padding,
                &mut scratch_rgba,
                base_r,
                base_g,
                base_b,
                text_a,
            );
        }

        if options.format == RenderFormat::Svg {
            let svg_options = typf_core::types::SvgOptions::default();
            let renderer = typf_render::SvgRenderer::new(&svg_options);
            let svg = renderer.render(shaped, &svg_options);
            return Ok(RenderOutput::Svg(svg));
        }

        let surface = RenderSurface::from_rgba(width, height, pixmap.take(), true);
        surface.into_render_output(options.format)
    }

    fn name(&self) -> &str {
        "skiahb"
    }

    fn clear_cache(&self) {
        self.cache.clear();
        self.hb_cache.write().clear();
        self.font_data_cache.write().clear();
        self.ttf_cache.write().clear();
    }
}

impl Default for HarfBuzzBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl DynBackend for HarfBuzzBackend {
    fn name(&self) -> &'static str {
        "skiahb"
    }

    fn shape_text(&self, text: &str, font: &Font) -> ShapingResult {
        let options = SegmentOptions::default();
        let runs = self
            .segment(text, &options)
            .expect("text segmentation failed");
        // For simplicity, we assume a single run for now, or merge them.
        // A more robust implementation would shape each run separately and combine.
        let first_run = runs.into_iter().next().unwrap_or_else(|| TextRun {
            text: text.to_string(),
            range: (0, text.len()),
            script: "Unknown".to_string(),
            language: "und".to_string(),
            direction: Direction::LeftToRight,
            font: None,
        });
        self.shape(&first_run, font).expect("text shaping failed")
    }

    fn render_glyph(&self, font: &Font, glyph_id: u32, options: RenderOptions) -> Option<Bitmap> {
        let face_entry = self.get_or_create_ttf_face(font).ok()?;
        let validated_variations = face_entry.validate_variations(&font.variations);

        // Construct a dummy Glyph for rasterization
        let dummy_glyph = Glyph {
            id: glyph_id,
            cluster: 0,
            x: 0.0,
            y: 0.0,
            advance: 0.0,
        };

        // Calculate scale factor
        let scale = options.font_size / face_entry.units_per_em as f32;

        self.rasterize_glyph(
            &face_entry.font_ref,
            &dummy_glyph,
            options.font_size,
            scale,
            &validated_variations,
            options.antialias != typf_core::types::AntialiasMode::None,
        )
        .map(|rg| {
            // Convert RenderedGlyph to Bitmap
            Bitmap {
                width: rg.width,
                height: rg.height,
                data: rg.bitmap,
            }
        })
    }

    fn render_shaped_text(&self, shaped_text: &ShapingResult, options: RenderOptions) -> Option<Bitmap> {
        match self.render(shaped_text, &options) {
            Ok(RenderOutput::Bitmap(bitmap)) => Some(bitmap),
            _ => None, // Handle other RenderOutput variants or errors as needed
        }
    }

    fn font_metrics(&self, font: &Font) -> FontMetrics {
        let face_entry = self.get_or_create_ttf_face(font).expect("failed to get font face for metrics");
        let scale = font.size / face_entry.units_per_em as f32;
        let hhea = face_entry.font_ref().hhea().expect("hhea table missing");
        FontMetrics {
            units_per_em: face_entry.units_per_em,
            ascender: (f32::from(face_entry.ascender_unscaled) * scale).round() as i16,
            descender: (f32::from(hhea.descender().to_i16()) * scale).round() as i16,
            line_gap: (f32::from(hhea.line_gap().to_i16()) * scale).round() as i16,
        }
    }

    fn supported_features(&self) -> BackendFeatures {
        BackendFeatures {
            monochrome: true, // HarfBuzz can render monochrome
            grayscale: true,  // HarfBuzz can render grayscale
            subpixel: false,  // TinySkia backend (used by HarfBuzz) might not directly support subpixel AA
            color_emoji: true, // HarfBuzz supports color emoji shaping
        }
    }
}

// Legacy tiny-skia helper - no longer used with renderer abstraction
#[cfg(feature = "tiny-skia-renderer")]
#[allow(dead_code)]
fn glyph_path(
    font_ref: &FontRef<'static>,
    glyph: &Glyph,
    size: f32,
    scale: f32,
) -> Option<SkiaPath> {
    let gid = u16::try_from(glyph.id).ok()?;
    let outline = glyph_bez_path(font_ref, GlyphId::from(gid), size, scale)?;
    bez_path_to_skia(&outline)
}

#[cfg(feature = "tiny-skia-renderer")]
#[allow(dead_code)]
fn bez_path_to_skia(path: &BezPath) -> Option<SkiaPath> {
    if path.elements().is_empty() {
        return None;
    }

    let mut builder = PathBuilder::new();
    for element in path.elements() {
        match *element {
            PathEl::MoveTo(p) => builder.move_to(p.x as f32, p.y as f32),
            PathEl::LineTo(p) => builder.line_to(p.x as f32, p.y as f32),
            PathEl::QuadTo(ctrl, end) => {
                builder.quad_to(ctrl.x as f32, ctrl.y as f32, end.x as f32, end.y as f32)
            }
            PathEl::CurveTo(c1, c2, end) => builder.cubic_to(
                c1.x as f32,
                c1.y as f32,
                c2.x as f32,
                c2.y as f32,
                end.x as f32,
                end.y as f32,
            ),
            PathEl::ClosePath => builder.close(),
        }
    }
    builder.finish()
}

fn blank_rendered_glyph() -> RenderedGlyph {
    RenderedGlyph {
        bitmap: Vec::new(),
        width: 0,
        height: 0,
        left: 0.0,
        top: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use skrifa::MetadataProvider;
    use std::collections::HashSet;
    use std::{fs, path::PathBuf, sync::Once};

    #[derive(Deserialize)]
    struct ShapeFixture {
        text: String,
        glyph_ids: Vec<u32>,
        font: String,
    }

    fn fixture_font_path(name: &str) -> String {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(manifest_dir)
            .join("../../testdata/fonts")
            .join(name)
            .to_string_lossy()
            .into_owned()
    }

    fn fixture_font(name: &str) -> Font {
        Font::from_path(fixture_font_path(name), 48.0)
    }

    fn ensure_test_fonts() {
        static INSTALL: Once = Once::new();
        INSTALL.call_once(|| {
            let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata/fonts");
            let existing = std::env::var_os("TYPF_FONT_DIRS");
            let mut paths: Vec<PathBuf> = existing
                .map(|value| std::env::split_paths(&value).collect())
                .unwrap_or_default();
            if !paths.iter().any(|p| p == &dir) {
                paths.push(dir.clone());
            }
            let joined = std::env::join_paths(paths).expect("join font dirs");
            std::env::set_var("TYPF_FONT_DIRS", joined);
        });
    }

    fn load_fixture(name: &str) -> ShapeFixture {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../testdata/expected/harfbuzz/{name}.json"));
        let data = fs::read_to_string(&path).expect("fixture readable");
        serde_json::from_str(&data).expect("fixture valid")
    }

    #[test]
    fn test_backend_creation() {
        let backend = HarfBuzzBackend::new();
        assert_eq!(DynBackend::name(&backend), "HarfBuzz");
    }

    #[test]
    fn test_simple_segmentation() {
        let backend = HarfBuzzBackend::new();
        let options = SegmentOptions::default();

        let runs = backend.segment("Hello World", &options).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "Hello World");
        assert_eq!(runs[0].script, "Latin");
        assert_eq!(runs[0].direction, Direction::LeftToRight);
    }

    #[test]
    fn test_script_itemization_and_bidi() {
        let backend = HarfBuzzBackend::new();
        let mut options = SegmentOptions::default();
        options.script_itemize = true;
        options.bidi_resolve = true;

        let runs = backend.segment("Hello مرحبا", &options).unwrap();
        assert!(runs.len() >= 2);
        assert_eq!(runs[0].script, "Latin");
        assert_eq!(runs[0].direction, Direction::LeftToRight);
        assert_eq!(runs.last().unwrap().script, "Arabic");
        assert_eq!(runs.last().unwrap().direction, Direction::RightToLeft);
    }

    #[test]
    fn test_line_breaks_split_runs() {
        let backend = HarfBuzzBackend::new();
        let options = SegmentOptions::default();
        let runs = backend.segment("Line1\nLine2", &options).unwrap();
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].text.trim_end_matches('\n'), "Line1");
        assert_eq!(runs[1].text, "Line2");
    }

    #[test]
    fn test_word_boundaries_when_font_fallback_enabled() {
        let backend = HarfBuzzBackend::new();
        let mut options = SegmentOptions::default();
        options.font_fallback = true;
        let runs = backend.segment("Word One", &options).unwrap();
        assert!(runs.len() >= 2);
    }

    #[test]
    fn test_shape_arabic_text_produces_contextual_forms() {
        ensure_test_fonts();
        let backend = HarfBuzzBackend::new();
        let mut options = SegmentOptions::default();
        options.script_itemize = true;
        options.bidi_resolve = true;
        options.language = Some("ar".to_string());
        let fixture = load_fixture("arabic_glyphs");

        let runs = backend.segment(&fixture.text, &options).unwrap();
        assert_eq!(runs.len(), 1, "Arabic text should stay in a single run");
        let run = &runs[0];
        assert_eq!(
            run.direction,
            Direction::RightToLeft,
            "Arabic run must resolve to RTL"
        );

        let font = fixture_font(&fixture.font);
        let shaped = backend.shape(run, &font).expect("Arabic shaping succeeds");
        let glyph_ids: Vec<u32> = shaped.glyphs.iter().map(|g| g.id).collect();
        assert_eq!(glyph_ids, fixture.glyph_ids, "Arabic glyph ids regressed");

        let clusters: Vec<u32> = shaped.glyphs.iter().map(|g| g.cluster).collect();
        assert!(
            clusters.windows(2).all(|pair| pair[0] > pair[1]),
            "Arabic clusters should decrease for RTL text: {clusters:?}"
        );
        assert_eq!(
            clusters.last(),
            Some(&0),
            "Arabic clusters must end at byte offset 0"
        );
        assert!(
            shaped.advance > 0.0 && shaped.bbox.width > 0.0,
            "Arabic shaping should produce measurable geometry"
        );
    }

    #[test]
    fn test_shape_devanagari_text_reorders_marks() {
        ensure_test_fonts();
        let backend = HarfBuzzBackend::new();
        let mut options = SegmentOptions::default();
        options.script_itemize = true;
        options.bidi_resolve = true;
        options.language = Some("hi".to_string());
        let fixture = load_fixture("devanagari_glyphs");

        let runs = backend.segment(&fixture.text, &options).unwrap();
        assert_eq!(runs.len(), 1, "Devanagari text should be a single run");
        let run = &runs[0];
        assert_eq!(run.script, "Devanagari");
        assert_eq!(run.direction, Direction::LeftToRight);

        let font = fixture_font(&fixture.font);
        let shaped = backend
            .shape(run, &font)
            .expect("Devanagari shaping succeeds");
        let glyph_ids: Vec<u32> = shaped.glyphs.iter().map(|g| g.id).collect();
        assert_eq!(glyph_ids, fixture.glyph_ids, "Devanagari glyph ids changed");

        let clusters: Vec<u32> = shaped.glyphs.iter().map(|g| g.cluster).collect();
        assert!(
            clusters.windows(2).all(|pair| pair[0] <= pair[1]),
            "LTR clusters must be non-decreasing: {clusters:?}"
        );

        assert_eq!(
            shaped.glyphs[1].cluster, shaped.glyphs[2].cluster,
            "AA matra must attach to the conjunct cluster"
        );
        assert!(
            shaped.glyphs.iter().any(|g| g.advance == 0.0),
            "At least one mark should have zero advance after reordering"
        );
    }

    #[test]
    fn test_shape_arabic_text_uses_script_fallback_when_font_missing() {
        ensure_test_fonts();
        let backend = HarfBuzzBackend::new();
        let mut options = SegmentOptions::default();
        options.script_itemize = true;
        options.bidi_resolve = true;
        options.language = Some("ar".to_string());
        options.font_fallback = true;

        let fixture = load_fixture("arabic_glyphs");
        let runs = backend.segment(&fixture.text, &options).unwrap();
        let fallback_target = Font::new("Noto Naskh Arabic", 48.0); // Updated to match actual fontdb family name
        let template_run = runs.first().expect("at least one run");
        let merged_run = TextRun {
            text: fixture.text.clone(),
            range: (0, fixture.text.len()),
            script: template_run.script.clone(),
            language: template_run.language.clone(),
            direction: template_run.direction,
            font: None,
        };
        let shaped = backend
            .shape(&merged_run, &fallback_target)
            .expect("Fallback shaping succeeds");
        let glyph_ids: Vec<u32> = shaped.glyphs.iter().map(|g| g.id).collect();
        assert_eq!(glyph_ids, fixture.glyph_ids, "Fallback glyph ids changed");

        let resolved_font = shaped.font.as_ref().expect("fallback font present");
        assert_eq!(
            resolved_font.family,
            "Noto Naskh Arabic", // Updated here too
            "expected Arabic fallback font to be Noto Naskh"
        );
    }

    #[test]
    fn test_shape_devanagari_text_uses_script_fallback_when_font_missing() {
        ensure_test_fonts();
        let backend = HarfBuzzBackend::new();
        let mut options = SegmentOptions::default();
        options.script_itemize = true;
        options.bidi_resolve = true;
        options.language = Some("hi".to_string());
        options.font_fallback = true;

        let fixture = load_fixture("devanagari_glyphs");
        let runs = backend.segment(&fixture.text, &options).unwrap();
        let fallback_target = Font::new("Noto Sans Devanagari", 48.0); // Updated to match actual fontdb family name
        let template_run = runs.first().expect("at least one run");
        let merged_run = TextRun {
            text: fixture.text.clone(),
            range: (0, fixture.text.len()),
            script: template_run.script.clone(),
            language: template_run.language.clone(),
            direction: template_run.direction,
            font: None,
        };
        let shaped = backend
            .shape(&merged_run, &fallback_target)
            .expect("Fallback shaping succeeds");
        let glyph_ids: Vec<u32> = shaped.glyphs.iter().map(|g| g.id).collect();
        assert_eq!(glyph_ids, fixture.glyph_ids, "Fallback glyph ids changed");

        let resolved_font = shaped.font.as_ref().expect("fallback font present");
        assert_eq!(
            resolved_font.family,
            "Noto Sans Devanagari", // Updated here too
            "expected Devanagari fallback font to be Noto Sans Devanagari"
        );
    }

    #[test]
    fn test_render_populates_glyph_cache() {
        let backend = HarfBuzzBackend::new();
        let font = fixture_font("NotoSans-Regular.ttf");
        let runs = backend
            .segment("Cache test", &SegmentOptions::default())
            .unwrap();
        let shaped = backend.shape(&runs[0], &font).unwrap();
        let mut options = RenderOptions::default();
        options.format = typf_core::types::RenderFormat::Raw;

        backend.render(&shaped, &options).unwrap();
        let unique_glyphs: HashSet<u32> = shaped.glyphs.iter().map(|g| g.id).collect();
        let stats = backend.cache.stats();
        assert!(
            stats.glyph_count >= unique_glyphs.len(),
            "glyph cache should contain rendered glyphs"
        );
    }

    #[test]
    fn test_render_reuses_cached_glyphs() {
        let backend = HarfBuzzBackend::new();
        let font = fixture_font("NotoSans-Regular.ttf");
        let runs = backend
            .segment("Re-render", &SegmentOptions::default())
            .unwrap();
        let shaped = backend.shape(&runs[0], &font).unwrap();
        let mut options = RenderOptions::default();
        options.format = typf_core::types::RenderFormat::Raw;

        backend.render(&shaped, &options).unwrap();
        let first = backend.cache.stats().glyph_count;

        backend.render(&shaped, &options).unwrap();
        let second = backend.cache.stats().glyph_count;

        assert_eq!(
            first, second,
            "glyph cache should not grow when re-rendering the same glyphs"
        );
    }

    #[test]
    fn test_clear_cache_empties_internal_layers() {
        ensure_test_fonts();
        let backend = HarfBuzzBackend::new();
        let font = fixture_font("NotoSans-Regular.ttf");
        let runs = backend
            .segment("Cache warmup", &SegmentOptions::default())
            .unwrap();
        let shaped = backend.shape(&runs[0], &font).unwrap();
        backend.render(&shaped, &RenderOptions::default()).unwrap();

        assert!(
            backend.cache.stats().glyph_count > 0,
            "glyph cache should be populated before clearing"
        );
        assert!(backend.hb_cache.read().len() > 0);
        assert!(backend.ttf_cache.read().len() > 0);
        assert!(backend.font_data_cache.read().len() > 0);

        backend.clear_cache();
        let stats = backend.cache.stats();
        assert!(stats.is_empty(), "cache stats after clear: {:?}", stats);
        assert_eq!(backend.hb_cache.read().len(), 0);
        assert_eq!(backend.ttf_cache.read().len(), 0);
        assert_eq!(backend.font_data_cache.read().len(), 0);
    }

    #[test]
    fn test_validate_variations_clamps_and_drops_unknown_axes() {
        ensure_test_fonts();
        let backend = HarfBuzzBackend::new();
        let mut font = fixture_font("RobotoFlex-Variable.ttf");
        font.variations.insert("wght".into(), -500.0);
        font.variations.insert("wdth".into(), 500.0);
        font.variations.insert("bogs".into(), 1.0);

        let face = backend
            .get_or_create_ttf_face(&font)
            .expect("variable font face loads");
        let validated = face.validate_variations(&font.variations);
        let axes = face.font_ref().axes();
        let wght_axis = axes
            .iter()
            .find(|axis| axis.tag().to_be_bytes() == *b"wght")
            .expect("wght axis present");
        let wdth_axis = axes
            .iter()
            .find(|axis| axis.tag().to_be_bytes() == *b"wdth")
            .expect("wdth axis present");

        let wght = validated
            .get("wght")
            .copied()
            .expect("wght axis survived validation");
        assert!(
            (wght - wght_axis.min_value()).abs() < 0.01,
            "expected wght to clamp to min; got {wght}"
        );
        let wdth = validated
            .get("wdth")
            .copied()
            .expect("wdth axis survived validation");
        assert!(
            (wdth - wdth_axis.max_value()).abs() < 0.01,
            "expected wdth clamp to max; got {wdth}"
        );
        assert!(
            !validated.contains_key("bogs"),
            "unknown axis should be dropped"
        );
    }

    #[test]
    fn test_roboto_flex_variations_affect_metrics() {
        ensure_test_fonts();
        let backend = HarfBuzzBackend::new();
        let runs = backend
            .segment("Variable width", &SegmentOptions::default())
            .expect("segment succeeds");
        let run = runs.first().expect("at least one run");

        let flex_path = fixture_font_path("RobotoFlex-Variable.ttf");
        let mut narrow = Font::from_path(flex_path.clone(), 48.0);
        let mut wide = Font::from_path(flex_path, 48.0);

        let face = backend
            .get_or_create_ttf_face(&narrow)
            .expect("variable font face loads");
        let axes = face.font_ref().axes();
        let wght_axis = axes
            .iter()
            .find(|axis| axis.tag().to_be_bytes() == *b"wght")
            .expect("wght axis present");
        let wdth_axis = axes
            .iter()
            .find(|axis| axis.tag().to_be_bytes() == *b"wdth")
            .expect("wdth axis present");

        narrow
            .variations
            .insert("wght".into(), wght_axis.min_value());
        narrow
            .variations
            .insert("wdth".into(), wdth_axis.min_value());
        wide.variations.insert("wght".into(), wght_axis.max_value());
        wide.variations.insert("wdth".into(), wdth_axis.max_value());

        let narrow_shape = backend.shape(run, &narrow).expect("narrow shape ok");
        let wide_shape = backend.shape(run, &wide).expect("wide shape ok");

        assert!(
            narrow_shape.advance < wide_shape.advance,
            "Condensed settings should produce smaller advance"
        );
        assert!(
            (wide_shape.advance - narrow_shape.advance).abs() > 5.0,
            "Advance delta should be noticeable"
        );
    }
}
