//! Where fonts come to life: database and loading for Typf
//!
//! The third stage of the pipeline. Finds, loads, and manages fonts so
//! your text can wear the right glyphs. Without fonts, text is just
//! invisible characters floating in digital space.
//!
//! ## Memory Management
//!
//! Fonts store their raw data and create `FontRef` on-demand for parsing.
//! This avoids memory leaks from `Box::leak` and properly supports TTC
//! font collections with multiple faces.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use read_fonts::{FontRef as ReadFontRef, TableProvider};

use typf_core::{
    error::{FontLoadError, Result},
    traits::FontRef as TypfFontRef,
};

/// A font that's been brought into memory, ready to shape text
///
/// Stores the raw font data and creates `FontRef` on-demand for parsing.
/// For TTC collections, the `face_index` specifies which face to use.
pub struct Font {
    data: Vec<u8>,
    face_index: u32,
    units_per_em: u16,
}

impl Font {
    /// Opens a font file from disk and makes it usable
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_file_index(path, 0)
    }

    /// Opens a specific face from a font file (for TTC collections)
    pub fn from_file_index(path: impl AsRef<Path>, face_index: u32) -> Result<Self> {
        let data = fs::read(path.as_ref())
            .map_err(|_| FontLoadError::FileNotFound(path.as_ref().display().to_string()))?;

        Self::from_data_index(data, face_index)
    }

    /// Turns raw font bytes into something we can work with
    pub fn from_data(data: Vec<u8>) -> Result<Self> {
        Self::from_data_index(data, 0)
    }

    /// Turns raw font bytes into a specific face (for TTC collections)
    pub fn from_data_index(data: Vec<u8>, face_index: u32) -> Result<Self> {
        // Validate the font data by attempting to parse it
        let font_ref =
            ReadFontRef::from_index(&data, face_index).map_err(|_| FontLoadError::InvalidData)?;

        // Extract the fundamental measurement: units per em
        // This tells us how big the font's grid is
        let units_per_em = font_ref
            .head()
            .map(|head| head.units_per_em())
            .unwrap_or(1000);

        Ok(Font {
            data,
            face_index,
            units_per_em,
        })
    }

    /// Returns the face index for TTC collections (0 for single fonts)
    pub fn face_index(&self) -> u32 {
        self.face_index
    }

    /// Creates a FontRef on-demand for parsing operations
    fn font_ref(&self) -> Option<ReadFontRef<'_>> {
        ReadFontRef::from_index(&self.data, self.face_index).ok()
    }

    /// Finds which glyph draws this character
    pub fn glyph_id(&self, ch: char) -> Option<u32> {
        self.font_ref()
            .and_then(|font| font.cmap().ok()?.map_codepoint(ch).map(|gid| gid.to_u32()))
    }

    /// Measures how wide this glyph will be
    pub fn advance_width(&self, glyph_id: u32) -> f32 {
        self.font_ref()
            .and_then(|font| {
                // Look up the horizontal metrics table
                let hmtx = font.hmtx().ok()?;

                // Get the advance width for this specific glyph
                use read_fonts::types::GlyphId;
                let glyph = GlyphId::new(glyph_id);
                let advance = hmtx.advance(glyph)?;

                // Convert from font units to something predictable
                let upem = self.units_per_em as f32;
                Some(advance as f32 / upem * 1000.0)
            })
            .unwrap_or(500.0) // Reasonable default when metrics fail
    }

    /// Counts how many different glyphs this font contains
    pub fn glyph_count(&self) -> Option<u32> {
        self.font_ref()
            .and_then(|font| font.maxp().ok().map(|maxp| maxp.num_glyphs() as u32))
    }
}

impl TypfFontRef for Font {
    fn data(&self) -> &[u8] {
        &self.data
    }

    fn units_per_em(&self) -> u16 {
        self.units_per_em
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
}

/// Your font library: keeps track of all loaded fonts
pub struct FontDatabase {
    fonts: Vec<Arc<Font>>,
    /// Cache to prevent loading the same font file multiple times.
    /// Maps canonical paths to their loaded fonts.
    path_cache: HashMap<PathBuf, Arc<Font>>,
    default_font: Option<Arc<Font>>,
}

impl FontDatabase {
    /// Starts with an empty library
    pub fn new() -> Self {
        Self {
            fonts: Vec::new(),
            path_cache: HashMap::new(),
            default_font: None,
        }
    }

    /// Loads a font file and remembers it for future use.
    /// If the same path was already loaded, returns the cached font.
    pub fn load_font(&mut self, path: impl AsRef<Path>) -> Result<Arc<Font>> {
        let path = path.as_ref();

        // Try to canonicalize the path for reliable deduplication
        let cache_key = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        // Return cached font if already loaded
        if let Some(font) = self.path_cache.get(&cache_key) {
            return Ok(font.clone());
        }

        // Load and cache the font
        let font = Arc::new(Font::from_file(path)?);
        self.path_cache.insert(cache_key, font.clone());
        self.fonts.push(font.clone());

        // First font loaded becomes the default
        if self.default_font.is_none() {
            self.default_font = Some(font.clone());
        }

        Ok(font)
    }

    /// Adds a font from memory to the library
    pub fn load_font_data(&mut self, data: Vec<u8>) -> Result<Arc<Font>> {
        let font = Arc::new(Font::from_data(data)?);
        self.fonts.push(font.clone());

        // First font loaded becomes the default
        if self.default_font.is_none() {
            self.default_font = Some(font.clone());
        }

        Ok(font)
    }

    /// Returns the font we fall back to when nothing else is specified
    pub fn default_font(&self) -> Option<Arc<Font>> {
        self.default_font.clone()
    }

    /// Shows all fonts currently loaded
    pub fn fonts(&self) -> &[Arc<Font>] {
        &self.fonts
    }

    /// Looks up a font by name (simplified for now)
    pub fn find_font(&self, _name: &str) -> Option<Arc<Font>> {
        self.default_font.clone()
    }

    /// Clears all loaded fonts from the database.
    /// Memory is properly reclaimed when all Arc references are dropped.
    pub fn clear(&mut self) {
        self.fonts.clear();
        self.path_cache.clear();
        self.default_font = None;
    }

    /// Returns the number of fonts currently loaded.
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

    #[test]
    fn test_empty_database() {
        let db = FontDatabase::new();
        assert!(db.default_font().is_none());
        assert_eq!(db.fonts().len(), 0);
        assert_eq!(db.font_count(), 0);
    }

    #[test]
    fn test_font_from_data() {
        // Create a minimal font data (empty for test)
        let data = vec![0; 100];
        let result = Font::from_data(data);
        // This will fail with invalid data, which is expected
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_database() {
        let mut db = FontDatabase::new();
        // After clear, database should be empty
        db.clear();
        assert!(db.default_font().is_none());
        assert_eq!(db.font_count(), 0);
    }
}
