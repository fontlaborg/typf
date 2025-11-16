// this_file: backends/typf-icu-hb/src/shaping.rs

//! Text shaping using HarfBuzz (ported from haforu).
//!
//! This module shapes text into positioned glyphs, handling complex scripts,
//! ligatures, kerning, and other OpenType features.

use harfbuzz_rs::{Direction, Feature, GlyphBuffer, Language, Tag, UnicodeBuffer};
use typf_fontdb::font_cache::FontInstance;
use std::path::Path;
use std::str::FromStr;

/// Shaped text with positioned glyphs.
///
/// Result of text shaping operation containing glyph IDs and positions.
#[derive(Debug, Clone)]
pub struct ShapedText {
    /// Positioned glyphs
    pub glyphs: Vec<ShapedGlyph>,
    /// Font size in points
    pub font_size: f32,
}

/// Single shaped glyph with position information.
///
/// All positions are in font units (26.6 fixed point from HarfBuzz).
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    /// Glyph ID in the font
    pub glyph_id: u32,
    /// Horizontal advance (in font units)
    pub x_advance: i32,
    /// Vertical advance (in font units, typically 0)
    pub y_advance: i32,
    /// Horizontal offset from cursor (in font units)
    pub x_offset: i32,
    /// Vertical offset from baseline (in font units)
    pub y_offset: i32,
}

/// Input parameters for shaping text.
///
/// Provides control over script detection, directionality, language,
/// and OpenType feature activation.
pub struct ShapeRequest<'a> {
    /// Literal text to shape.
    pub text: &'a str,
    /// Script hint (e.g., "Latn", "Arab", "Cyrl").
    /// If None, HarfBuzz will auto-detect.
    pub script: Option<&'a str>,
    /// Direction hint: "ltr" (left-to-right), "rtl" (right-to-left),
    /// "ttb" (top-to-bottom), "btt" (bottom-to-top).
    /// If None, HarfBuzz will auto-detect based on script.
    pub direction: Option<&'a str>,
    /// Language hint (BCP-47 tag, e.g., "en", "ar-EG").
    /// Used for language-specific shaping rules.
    pub language: Option<&'a str>,
    /// OpenType features to force on/off.
    /// Format: "feature=value" (e.g., "liga=1", "kern=0").
    pub features: &'a [String],
}

impl<'a> Default for ShapeRequest<'a> {
    fn default() -> Self {
        Self {
            text: "",
            script: None,
            direction: None,
            language: None,
            features: &[],
        }
    }
}

/// Text shaper using HarfBuzz.
///
/// Stateless shaper that processes text with a font instance.
pub struct TextShaper;

impl TextShaper {
    /// Create a new text shaper.
    pub fn new() -> Self {
        Self
    }

    /// Shape text using the provided font instance (simple API).
    ///
    /// Uses default shaping parameters (auto-detect script/direction).
    ///
    /// # Arguments
    /// * `font_instance` - Font with applied variations
    /// * `text` - Text to shape
    /// * `font_size` - Font size in points
    /// * `path` - Font file path (for error reporting)
    ///
    /// # Returns
    /// ShapedText with positioned glyphs
    ///
    /// # Errors
    /// Returns error if shaping fails (font data invalid, etc.)
    ///
    /// # Examples
    /// ```ignore
    /// use typf_icu_hb::shaping::TextShaper;
    ///
    /// let shaper = TextShaper::new();
    /// let shaped = shaper.shape(&font_instance, "Hello", 72.0, path)?;
    /// for glyph in &shaped.glyphs {
    ///     println!("Glyph {}: advance {}", glyph.glyph_id, glyph.x_advance);
    /// }
    /// ```
    pub fn shape(
        &self,
        font_instance: &FontInstance,
        text: &str,
        font_size: f32,
        path: &Path,
    ) -> Result<ShapedText, ShapingError> {
        let request = ShapeRequest {
            text,
            ..Default::default()
        };
        self.shape_with_request(font_instance, &request, font_size, path)
    }

    /// Shape text using detailed request parameters.
    ///
    /// Allows control over script, direction, language, and OpenType features.
    ///
    /// # Arguments
    /// * `font_instance` - Font with applied variations
    /// * `request` - Shaping parameters (text, script, direction, features)
    /// * `font_size` - Font size in points
    /// * `path` - Font file path (for error reporting)
    ///
    /// # Errors
    /// Returns error if shaping fails or parameters are invalid
    ///
    /// # Examples
    /// ```ignore
    /// let request = ShapeRequest {
    ///     text: "مرحبا",
    ///     script: Some("Arab"),
    ///     direction: Some("rtl"),
    ///     language: Some("ar"),
    ///     features: &["liga=1".to_string()],
    /// };
    /// let shaped = shaper.shape_with_request(&font_instance, &request, 72.0, path)?;
    /// ```
    pub fn shape_with_request(
        &self,
        font_instance: &FontInstance,
        request: &ShapeRequest<'_>,
        font_size: f32,
        path: &Path,
    ) -> Result<ShapedText, ShapingError> {
        // Create HarfBuzz buffer and add text (builder pattern)
        let mut buffer = UnicodeBuffer::new().add_str(request.text);

        // Set script if provided (builder pattern with move)
        if let Some(script_str) = request.script {
            if let Some(tag) = parse_feature_tag(script_str) {
                buffer = buffer.set_script(tag);
            } else {
                log::warn!("Invalid script tag '{}' - using auto-detection", script_str);
            }
        }

        // Set direction if provided (builder pattern with move)
        if let Some(dir_str) = request.direction {
            let direction = match dir_str.to_lowercase().as_str() {
                "ltr" => Direction::Ltr,
                "rtl" => Direction::Rtl,
                "ttb" => Direction::Ttb,
                "btt" => Direction::Btt,
                _ => {
                    log::warn!("Unknown direction '{}' - using LTR", dir_str);
                    Direction::Ltr
                }
            };
            buffer = buffer.set_direction(direction);
        }

        // Set language if provided (builder pattern with move)
        if let Some(lang_str) = request.language {
            if let Ok(lang) = Language::from_str(lang_str) {
                buffer = buffer.set_language(lang);
            } else {
                log::warn!("Invalid language tag '{}' - ignoring", lang_str);
            }
        }

        // Parse OpenType features
        let features: Vec<Feature> = request
            .features
            .iter()
            .filter_map(|feat_str| {
                // Format: "feature=value" or "feature" (defaults to on)
                if let Some((tag_str, val_str)) = feat_str.split_once('=') {
                    let tag = parse_feature_tag(tag_str)?;
                    let value = val_str.parse::<u32>().ok()?;
                    Some(Feature::new(tag, value, 0..))
                } else {
                    let tag = parse_feature_tag(feat_str)?;
                    Some(Feature::new(tag, 1, 0..))
                }
            })
            .collect();

        // Shape text using the cached HarfBuzz font
        let hb_font = font_instance.hb_font();
        let hb_font_guard = hb_font.lock().map_err(|_| ShapingError::FontLockFailed {
            path: path.to_path_buf(),
        })?;

        let glyph_buffer: GlyphBuffer = if features.is_empty() {
            harfbuzz_rs::shape(&hb_font_guard, buffer, &[])
        } else {
            harfbuzz_rs::shape(&hb_font_guard, buffer, &features)
        };

        // Extract glyph positions
        let positions = glyph_buffer.get_glyph_positions();
        let infos = glyph_buffer.get_glyph_infos();

        if positions.len() != infos.len() {
            return Err(ShapingError::Internal {
                reason: format!(
                    "HarfBuzz position/info count mismatch: {} vs {}",
                    positions.len(),
                    infos.len()
                ),
            });
        }

        let glyphs: Vec<ShapedGlyph> = infos
            .iter()
            .zip(positions.iter())
            .map(|(info, pos)| ShapedGlyph {
                glyph_id: info.codepoint,
                x_advance: pos.x_advance,
                y_advance: pos.y_advance,
                x_offset: pos.x_offset,
                y_offset: pos.y_offset,
            })
            .collect();

        Ok(ShapedText { glyphs, font_size })
    }
}

impl Default for TextShaper {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a 4-character OpenType feature tag.
fn parse_feature_tag(tag_str: &str) -> Option<Tag> {
    let chars: Vec<char> = tag_str.chars().collect();
    if chars.len() == 4 {
        Some(Tag::new(chars[0], chars[1], chars[2], chars[3]))
    } else {
        log::warn!("Invalid feature tag '{}' (must be 4 characters)", tag_str);
        None
    }
}

/// Errors that can occur during text shaping.
#[derive(Debug, thiserror::Error)]
pub enum ShapingError {
    /// Failed to acquire lock on HarfBuzz font object.
    #[error("Failed to lock HarfBuzz font for {}", path.display())]
    FontLockFailed { path: std::path::PathBuf },

    /// Internal shaping error.
    #[error("Shaping internal error: {}", reason)]
    Internal { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_feature_tag_valid() {
        let tag = parse_feature_tag("liga");
        assert!(tag.is_some());
    }

    #[test]
    fn test_parse_feature_tag_invalid() {
        let tag = parse_feature_tag("x"); // Too short
        assert!(tag.is_none());

        let tag = parse_feature_tag("toolong"); // Too long
        assert!(tag.is_none());
    }

    #[test]
    fn test_text_shaper_new() {
        let _shaper = TextShaper::new();
        // Just verify construction doesn't panic
    }

    #[test]
    fn test_shape_request_default() {
        let request = ShapeRequest::default();
        assert_eq!(request.text, "");
        assert!(request.script.is_none());
        assert!(request.direction.is_none());
        assert!(request.language.is_none());
        assert_eq!(request.features.len(), 0);
    }

    // Note: Integration tests with actual fonts should be in tests/ directory
}

// Implement typf-zeno traits when the feature is enabled
#[cfg(feature = "zeno-traits")]
mod zeno_compat {
    use super::{ShapedGlyph, ShapedText};
    use typf_zeno::{ShapedGlyphAccess, ShapedTextAccess};

    impl ShapedTextAccess for ShapedText {
        type Glyph = ShapedGlyph;

        fn glyphs(&self) -> &[Self::Glyph] {
            &self.glyphs
        }

        fn font_size(&self) -> f32 {
            self.font_size
        }
    }

    impl ShapedGlyphAccess for ShapedGlyph {
        fn glyph_id(&self) -> u32 {
            self.glyph_id
        }

        fn x_offset(&self) -> i32 {
            self.x_offset
        }

        fn y_offset(&self) -> i32 {
            self.y_offset
        }

        fn x_advance(&self) -> i32 {
            self.x_advance
        }
    }
}
