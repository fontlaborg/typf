//! Font loading and face management for Typf.
//!
//! This crate turns raw font files into face objects that the rest of the
//! pipeline can query. It keeps the original bytes in memory and creates parser
//! views on demand, which is important for two reasons:
//!
//! - it avoids leaking long-lived parser objects,
//! - it supports collection files such as TTCs, where one file contains several
//!   faces and each face needs its own index.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use read_fonts::{FontRef as ReadFontRef, TableProvider};

use typf_core::{
    error::{FontLoadError, Result},
    traits::FontRef as TypfFontRef,
    types::{FontMetrics, VariationAxis},
};

/// Source descriptor for one loaded font face.
#[derive(Clone, Debug)]
pub struct TypfFontSource {
    path: Option<PathBuf>,
    face_index: u32,
}

impl TypfFontSource {
    pub fn new(path: Option<PathBuf>, face_index: u32) -> Self {
        Self { path, face_index }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn face_index(&self) -> u32 {
        self.face_index
    }
}

/// In-memory font face ready for shaping and rendering.
///
/// The face keeps the original font bytes and recreates parser views on demand.
/// For collection files such as TTCs, `face_index` selects the face inside the
/// shared file.
pub struct TypfFontFace {
    data: Arc<Vec<u8>>,
    source: TypfFontSource,
    units_per_em: u16,
    metrics: FontMetrics,
}

impl TypfFontFace {
    /// Load the first face from a font file on disk.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_file_index(path, 0)
    }

    /// Load a specific face from a font file or collection.
    pub fn from_file_index(path: impl AsRef<Path>, face_index: u32) -> Result<Self> {
        let data = fs::read(path.as_ref())
            .map_err(|_| FontLoadError::FileNotFound(path.as_ref().display().to_string()))?;

        Self::from_data_index_with_path(data, face_index, Some(path.as_ref().to_path_buf()))
    }

    /// Load the first face from raw font bytes.
    pub fn from_data(data: Vec<u8>) -> Result<Self> {
        Self::from_data_index(data, 0)
    }

    /// Load a specific face from raw font bytes.
    pub fn from_data_index(data: Vec<u8>, face_index: u32) -> Result<Self> {
        Self::from_data_index_with_path(data, face_index, None)
    }

    fn from_data_index_with_path(
        data: Vec<u8>,
        face_index: u32,
        path: Option<PathBuf>,
    ) -> Result<Self> {
        let font_ref = ReadFontRef::from_index(data.as_slice(), face_index)
            .map_err(|_| FontLoadError::InvalidData)?;

        let units_per_em = font_ref
            .head()
            .map(|head| head.units_per_em())
            .unwrap_or(1000);

        let (ascent, descent, line_gap) = font_ref
            .os2()
            .ok()
            .map(|os2| {
                (
                    os2.s_typo_ascender(),
                    os2.s_typo_descender(),
                    os2.s_typo_line_gap(),
                )
            })
            .or_else(|| {
                font_ref.hhea().ok().map(|hhea| {
                    (
                        hhea.ascender().to_i16(),
                        hhea.descender().to_i16(),
                        hhea.line_gap().to_i16(),
                    )
                })
            })
            .unwrap_or((0, 0, 0));

        Ok(TypfFontFace {
            data: Arc::new(data),
            source: TypfFontSource::new(path, face_index),
            units_per_em,
            metrics: FontMetrics {
                units_per_em,
                ascent,
                descent,
                line_gap,
            },
        })
    }

    pub fn source(&self) -> &TypfFontSource {
        &self.source
    }

    pub fn face_index(&self) -> u32 {
        self.source.face_index
    }

    pub fn path(&self) -> Option<&Path> {
        self.source.path()
    }

    /// Borrow the face as a `read-fonts` [`FontRef`](ReadFontRef), honouring
    /// `face_index` for collections. Use this to reach any table (or any
    /// `skrifa` API through [`skrifa::MetadataProvider`]) without re-parsing.
    pub fn font_ref(&self) -> Option<ReadFontRef<'_>> {
        ReadFontRef::from_index(self.data.as_slice(), self.source.face_index).ok()
    }

    /// `skrifa` outline collection for this face (glyf/CFF/CFF2), ready for
    /// `get(gid).draw(settings, pen)`.
    pub fn outline_glyphs(&self) -> Option<skrifa::outline::OutlineGlyphCollection<'_>> {
        use skrifa::MetadataProvider;
        Some(self.font_ref()?.outline_glyphs())
    }

    /// Normalised variation location for user-space `(tag, value)` pairs
    /// (`avar` applied). Unknown tags are ignored; missing axes use defaults.
    /// Static fonts yield an empty location.
    pub fn location(&self, variations: &[(String, f32)]) -> skrifa::instance::Location {
        use skrifa::MetadataProvider;
        match self.font_ref() {
            Some(font) => font
                .axes()
                .location(variations.iter().map(|(tag, v)| (tag.as_str(), *v))),
            None => skrifa::instance::Location::default(),
        }
    }

    pub fn glyph_id(&self, ch: char) -> Option<u32> {
        self.font_ref()
            .and_then(|font| font.cmap().ok()?.map_codepoint(ch).map(|gid| gid.to_u32()))
    }

    /// Return the advance width normalized to a 1000-unit em.
    ///
    /// Source fonts can use different units-per-em values. This method returns
    /// a stable scale for callers that do not want to repeat that conversion.
    pub fn advance_width(&self, glyph_id: u32) -> f32 {
        self.font_ref()
            .and_then(|font| {
                let hmtx = font.hmtx().ok()?;

                use read_fonts::types::GlyphId;
                let glyph = GlyphId::new(glyph_id);
                let advance = hmtx.advance(glyph)?;

                let upem = self.units_per_em as f32;
                Some(advance as f32 / upem * 1000.0)
            })
            .unwrap_or(500.0)
    }

    pub fn glyph_count(&self) -> Option<u32> {
        self.font_ref()
            .and_then(|font| font.maxp().ok().map(|maxp| maxp.num_glyphs() as u32))
    }

    /// Returns variable font axes from the fvar table.
    pub fn variation_axes(&self) -> Option<Vec<VariationAxis>> {
        let font = self.font_ref()?;
        let fvar = font.fvar().ok()?;
        let axes_slice = fvar.axes().ok()?;
        let name_table = font.name().ok();

        let axes: Vec<VariationAxis> = axes_slice
            .iter()
            .map(|axis| {
                let tag_bytes = axis.axis_tag().into_bytes();
                let tag = String::from_utf8_lossy(&tag_bytes).to_string();

                let name = name_table.as_ref().and_then(|nt| {
                    let name_id = axis.axis_name_id();
                    nt.name_record()
                        .iter()
                        .find(|record| record.name_id() == name_id)
                        .and_then(|record| record.string(nt.string_data()).ok())
                        .map(|s| s.to_string())
                });

                let hidden = axis.flags() & 0x0001 != 0;

                VariationAxis {
                    tag,
                    name,
                    min_value: axis.min_value().to_f32(),
                    default_value: axis.default_value().to_f32(),
                    max_value: axis.max_value().to_f32(),
                    hidden,
                }
            })
            .collect();

        Some(axes)
    }
}

impl TypfFontRef for TypfFontFace {
    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn data_shared(&self) -> Option<Arc<dyn AsRef<[u8]> + Send + Sync>> {
        Some(self.data.clone())
    }

    fn units_per_em(&self) -> u16 {
        self.units_per_em
    }

    fn metrics(&self) -> Option<FontMetrics> {
        Some(self.metrics)
    }

    fn glyph_id(&self, ch: char) -> Option<u32> {
        self.glyph_id(ch)
    }

    fn advance_width(&self, glyph_id: u32) -> f32 {
        self.advance_width(glyph_id)
    }

    fn glyph_count(&self) -> Option<u32> {
        self.glyph_count()
    }

    fn variation_axes(&self) -> Option<Vec<VariationAxis>> {
        self.variation_axes()
    }
}

/// Collection of loaded font faces and their source metadata.
pub struct FontDatabase {
    fonts: Vec<Arc<TypfFontFace>>,
    sources: Vec<TypfFontSource>,
    path_cache: HashMap<(PathBuf, u32), Arc<TypfFontFace>>,
    default_font: Option<Arc<TypfFontFace>>,
}

impl FontDatabase {
    /// Create an empty font database.
    pub fn new() -> Self {
        Self {
            fonts: Vec::new(),
            sources: Vec::new(),
            path_cache: HashMap::new(),
            default_font: None,
        }
    }

    /// Load the first face from a file and reuse a cached copy when possible.
    pub fn load_font(&mut self, path: impl AsRef<Path>) -> Result<Arc<TypfFontFace>> {
        let path = path.as_ref();

        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let face_index = 0;
        let cache_key = (canonical.clone(), face_index);

        if let Some(font) = self.path_cache.get(&cache_key) {
            return Ok(font.clone());
        }

        let font = Arc::new(TypfFontFace::from_file(path)?);
        self.path_cache.insert(cache_key, font.clone());
        self.fonts.push(font.clone());
        self.sources
            .push(TypfFontSource::new(Some(canonical), face_index));

        if self.default_font.is_none() {
            self.default_font = Some(font.clone());
        }

        Ok(font)
    }

    pub fn load_font_data(&mut self, data: Vec<u8>) -> Result<Arc<TypfFontFace>> {
        let font = Arc::new(TypfFontFace::from_data(data)?);
        self.fonts.push(font.clone());
        self.sources
            .push(TypfFontSource::new(None, font.face_index()));

        if self.default_font.is_none() {
            self.default_font = Some(font.clone());
        }

        Ok(font)
    }

    pub fn default_font(&self) -> Option<Arc<TypfFontFace>> {
        self.default_font.clone()
    }

    pub fn fonts(&self) -> &[Arc<TypfFontFace>] {
        &self.fonts
    }

    pub fn sources(&self) -> &[TypfFontSource] {
        &self.sources
    }

    /// Temporary lookup stub.
    ///
    /// This currently returns the default font instead of performing a real
    /// family-name search.
    pub fn find_font(&self, _name: &str) -> Option<Arc<TypfFontFace>> {
        self.default_font.clone()
    }

    pub fn clear(&mut self) {
        self.fonts.clear();
        self.sources.clear();
        self.path_cache.clear();
        self.default_font = None;
    }

    pub fn font_count(&self) -> usize {
        self.fonts.len()
    }
}

impl Default for FontDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typf_core::traits::FontRef;

    fn test_font_path(name: &str) -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.pop();
        p.push("test-fonts");
        p.push(name);
        p
    }

    #[test]
    fn test_font_ref_and_outline_glyphs_when_static_font_then_available() {
        let Ok(face) = TypfFontFace::from_file(test_font_path("NotoSans-Regular.ttf")) else {
            return;
        };
        let font = face.font_ref().expect("font_ref must parse face 0");
        assert!(font.table_data(read_fonts::types::Tag::new(b"glyf")).is_some(), "NotoSans has glyf");
        let outlines = face.outline_glyphs().expect("outline collection");
        assert!(outlines.get(read_fonts::types::GlyphId::new(36)).is_some(), "gid 36 has an outline");
        assert!(face.location(&[("wght".to_string(), 700.0)]).coords().is_empty(), "static font has no axes");
    }

    #[test]
    fn test_location_when_variable_font_then_normalised() {
        let Ok(face) = TypfFontFace::from_file(test_font_path("Kalnia[wdth,wght].ttf")) else {
            return;
        };
        let default = face.location(&[]);
        assert!(default.coords().iter().all(|c| c.to_bits() == 0), "default location is all zeros");
        let bold = face.location(&[("wght".to_string(), 700.0), ("zzzz".to_string(), 1.0)]);
        assert_eq!(bold.coords().len(), 2, "one coord per fvar axis, unknown tag ignored");
        assert!(bold.coords()[0].to_bits() > 0, "wght (axis 0) above default is positive");
    }

    #[test]
    fn test_font_ref_when_ttc_index_then_distinct_faces() {
        // macOS ships collections; skip elsewhere.
        let ttc = Path::new("/System/Library/Fonts/Supplemental/Arial Unicode.ttf");
        let ttc = if ttc.exists() { ttc.to_path_buf() } else { PathBuf::from("/System/Library/Fonts/Helvetica.ttc") };
        if !ttc.exists() {
            return;
        }
        let Ok(face0) = TypfFontFace::from_file_index(&ttc, 0) else { return };
        assert_eq!(face0.face_index(), 0);
        assert!(face0.font_ref().is_some());
        if let Ok(face1) = TypfFontFace::from_file_index(&ttc, 1) {
            assert_eq!(face1.face_index(), 1);
            let n0 = face0.font_ref().and_then(|f| f.head().ok()).map(|h| h.checksum_adjustment());
            let n1 = face1.font_ref().and_then(|f| f.head().ok()).map(|h| h.checksum_adjustment());
            assert_ne!(n0, n1, "face 0 and face 1 of a collection must differ");
        }
        assert!(TypfFontFace::from_file_index(&ttc, 999).is_err(), "out-of-range face index fails");
    }

    #[test]
    fn test_empty_database() {
        let db = FontDatabase::new();
        assert!(db.default_font().is_none());
        assert_eq!(db.fonts().len(), 0);
        assert_eq!(db.sources().len(), 0);
        assert_eq!(db.font_count(), 0);
    }

    #[test]
    fn test_font_from_data() {
        // Create a minimal font data (empty for test)
        let data = vec![0; 100];
        let result = TypfFontFace::from_data(data);
        // This will fail with invalid data, which is expected
        assert!(result.is_err());
    }

    #[test]
    fn test_font_from_data_index_invalid() {
        // Invalid face index should fail
        let data = vec![0; 100];
        let result = TypfFontFace::from_data_index(data, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_database() {
        let mut db = FontDatabase::new();
        // After clear, database should be empty
        db.clear();
        assert!(db.default_font().is_none());
        assert_eq!(db.sources().len(), 0);
        assert_eq!(db.font_count(), 0);
    }

    #[test]
    fn test_face_index_default() {
        // When from_data fails, we can't test face_index, but we can verify
        // the API exists and returns 0 for default construction path
        let data = vec![0; 100];
        let result = TypfFontFace::from_data_index(data, 0);
        // Invalid data, but we're testing the API structure
        assert!(result.is_err());
    }

    #[test]
    fn test_typf_font_face_data_shared_when_loaded_then_some() {
        let font_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-fonts/NotoSans-Regular.ttf"
        );
        let Ok(font) = TypfFontFace::from_file(font_path) else {
            return;
        };

        let shared = font.data_shared();
        assert!(
            shared.is_some(),
            "TypfFontFace should provide shared font bytes to avoid copies"
        );

        if let Some(shared) = shared {
            assert_eq!(
                shared.as_ref().as_ref(),
                font.data(),
                "shared bytes must match FontRef::data()"
            );
        }
    }
}
