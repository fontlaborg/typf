//! Font database and loading module for TYPF

use std::fs;
use std::path::Path;
use std::sync::Arc;

use read_fonts::{FontRef as ReadFontRef, TableProvider};

use typf_core::{
    error::{FontLoadError, Result},
    traits::FontRef as TypfFontRef,
};

/// A loaded font with its data
pub struct Font {
    data: Vec<u8>,
    font_ref: Option<ReadFontRef<'static>>,
    units_per_em: u16,
}

impl Font {
    /// Load a font from a file path
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let data = fs::read(path.as_ref())
            .map_err(|_| FontLoadError::FileNotFound(path.as_ref().display().to_string()))?;

        Self::from_data(data)
    }

    /// Load a font from raw data
    pub fn from_data(data: Vec<u8>) -> Result<Self> {
        // Leak the data to get a 'static reference (font will own the data)
        let data_ref: &'static [u8] = Box::leak(data.clone().into_boxed_slice());

        // Parse the font (handle TrueType Collections)
        let font_ref =
            ReadFontRef::from_index(data_ref, 0).map_err(|_| FontLoadError::InvalidData)?;

        // Get units per em
        let units_per_em = font_ref
            .head()
            .map(|head| head.units_per_em())
            .unwrap_or(1000);

        Ok(Font {
            data,
            font_ref: Some(font_ref),
            units_per_em,
        })
    }

    /// Get glyph ID for a character
    pub fn glyph_id(&self, ch: char) -> Option<u32> {
        self.font_ref
            .as_ref()
            .and_then(|font| font.cmap().ok()?.map_codepoint(ch).map(|gid| gid.to_u32()))
    }

    /// Get advance width for a glyph
    pub fn advance_width(&self, glyph_id: u32) -> f32 {
        self.font_ref
            .as_ref()
            .and_then(|font| {
                // Get horizontal metrics table
                let hmtx = font.hmtx().ok()?;

                // Get advance width for this glyph
                use read_fonts::types::GlyphId;
                let glyph = GlyphId::new(glyph_id);
                let advance = hmtx.advance(glyph)?;

                // Convert from font units to a standard value
                let upem = self.units_per_em as f32;
                Some(advance as f32 / upem * 1000.0)
            })
            .unwrap_or(500.0)
    }

    /// Get the total number of glyphs in the font
    pub fn glyph_count(&self) -> Option<u32> {
        self.font_ref
            .as_ref()
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

/// Font database for managing multiple fonts
pub struct FontDatabase {
    fonts: Vec<Arc<Font>>,
    default_font: Option<Arc<Font>>,
}

impl FontDatabase {
    /// Create a new empty font database
    pub fn new() -> Self {
        Self {
            fonts: Vec::new(),
            default_font: None,
        }
    }

    /// Load a font and add it to the database
    pub fn load_font(&mut self, path: impl AsRef<Path>) -> Result<Arc<Font>> {
        let font = Arc::new(Font::from_file(path)?);
        self.fonts.push(font.clone());

        // Set as default if it's the first font
        if self.default_font.is_none() {
            self.default_font = Some(font.clone());
        }

        Ok(font)
    }

    /// Load font from data
    pub fn load_font_data(&mut self, data: Vec<u8>) -> Result<Arc<Font>> {
        let font = Arc::new(Font::from_data(data)?);
        self.fonts.push(font.clone());

        // Set as default if it's the first font
        if self.default_font.is_none() {
            self.default_font = Some(font.clone());
        }

        Ok(font)
    }

    /// Get the default font
    pub fn default_font(&self) -> Option<Arc<Font>> {
        self.default_font.clone()
    }

    /// Get all fonts
    pub fn fonts(&self) -> &[Arc<Font>] {
        &self.fonts
    }

    /// Find fonts by name (simplified - would need metadata in real implementation)
    pub fn find_font(&self, _name: &str) -> Option<Arc<Font>> {
        self.default_font.clone()
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
    }

    #[test]
    fn test_font_from_data() {
        // Create a minimal font data (empty for test)
        let data = vec![0; 100];
        let result = Font::from_data(data);
        // This will fail with invalid data, which is expected
        assert!(result.is_err());
    }
}
