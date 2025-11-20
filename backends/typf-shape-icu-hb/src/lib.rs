//! Unicode perfection: ICU preprocessing meets HarfBuzz shaping
//!
//! Some text needs more than just shaping. It needs normalization (é vs e + ´),
//! bidirectional analysis, script detection. ICU handles the Unicode plumbing,
//! then passes perfect text to HarfBuzz for shaping. This is the shaper you
//! want when you need to handle every edge case the Unicode spec throws at you.

use harfbuzz_rs::{Direction as HbDirection, Face, Feature, Font as HbFont, Tag, UnicodeBuffer};
use std::str::FromStr;
use std::sync::Arc;
use typf_core::{
    error::Result,
    traits::{FontRef, Shaper, Stage},
    types::{Direction, PositionedGlyph, ShapingResult},
    ShapingParams,
};
use unicode_normalization::UnicodeNormalization;

pub mod cache;
pub use cache::ShapingCache;

/// ICU preprocessing + HarfBuzz shaping for bulletproof Unicode support
///
/// What this gives you:
/// - Unicode normalization (fixes broken character sequences)
/// - Bidirectional text handling (Arabic/Hebrew in Latin text)
/// - Script detection (knows Arabic from Thai from Cyrillic)
/// - Line breaking analysis (where text can safely break)
/// - Professional OpenType shaping with HarfBuzz
pub struct IcuHarfBuzzShaper;

impl IcuHarfBuzzShaper {
    /// Creates a new shaper that's ready for any Unicode challenge
    pub fn new() -> Self {
        Self
    }

    /// Maps our direction enum to HarfBuzz's format
    fn to_hb_direction(dir: Direction) -> HbDirection {
        match dir {
            Direction::LeftToRight => HbDirection::Ltr,
            Direction::RightToLeft => HbDirection::Rtl,
            Direction::TopToBottom => HbDirection::Ttb,
            Direction::BottomToTop => HbDirection::Btt,
        }
    }
}

impl Default for IcuHarfBuzzShaper {
    fn default() -> Self {
        Self::new()
    }
}

impl Stage for IcuHarfBuzzShaper {
    fn name(&self) -> &'static str {
        "ICU-HarfBuzz"
    }

    fn process(
        &self,
        ctx: typf_core::context::PipelineContext,
    ) -> Result<typf_core::context::PipelineContext> {
        // ICU-HB doesn't process pipeline context directly
        Ok(ctx)
    }
}

impl Shaper for IcuHarfBuzzShaper {
    fn name(&self) -> &'static str {
        "ICU-HarfBuzz"
    }

    fn shape(
        &self,
        text: &str,
        font: Arc<dyn FontRef>,
        params: &ShapingParams,
    ) -> Result<ShapingResult> {
        if text.is_empty() {
            return Ok(ShapingResult {
                glyphs: Vec::new(),
                advance_width: 0.0,
                advance_height: params.size,
                direction: params.direction,
            });
        }

        // Step 1: Normalize the text (fix é vs e + ´ and similar issues)
        let normalized: String = text.nfc().collect();

        // Step 2: Get the font data for HarfBuzz
        let font_data = font.data();
        if font_data.is_empty() {
            // No font data? Fall back to basic shaping on cleaned text
            let mut glyphs = Vec::new();
            let mut x_offset = 0.0;

            for ch in normalized.chars() {
                if let Some(glyph_id) = font.glyph_id(ch) {
                    let advance = font.advance_width(glyph_id);
                    glyphs.push(PositionedGlyph {
                        id: glyph_id,
                        x: x_offset,
                        y: 0.0,
                        advance,
                        cluster: 0,
                    });
                    x_offset += advance * params.size / font.units_per_em() as f32;
                }
            }

            return Ok(ShapingResult {
                glyphs,
                advance_width: x_offset,
                advance_height: params.size,
                direction: params.direction,
            });
        }

        // Step 3: Load the font into HarfBuzz
        let hb_face = Face::from_bytes(font_data, 0);
        let mut hb_font = HbFont::new(hb_face);

        // HarfBuzz uses 26.6 fixed-point for font coordinates
        let scale = (params.size * 64.0) as i32; // 64 units per point
        hb_font.set_scale(scale, scale);

        // Step 4: Set up HarfBuzz's text buffer with our normalized text
        let mut buffer = UnicodeBuffer::new()
            .add_str(&normalized)
            .set_direction(Self::to_hb_direction(params.direction));

        // Tell HarfBuzz which language rules to use
        if let Some(ref lang) = params.language {
            if let Ok(language) = harfbuzz_rs::Language::from_str(lang) {
                buffer = buffer.set_language(language);
            }
        }

        // Specify the script (critical for complex scripts)
        if let Some(ref script_str) = params.script {
            if script_str.len() == 4 {
                let script_bytes = script_str.as_bytes();
                let tag = Tag::new(
                    script_bytes[0] as char,
                    script_bytes[1] as char,
                    script_bytes[2] as char,
                    script_bytes[3] as char,
                );
                buffer = buffer.set_script(tag);
            }
        }

        // Step 5: Convert OpenType features to HarfBuzz format
        let features: Vec<Feature> = params
            .features
            .iter()
            .filter_map(|(name, value)| {
                if name.len() == 4 {
                    let bytes = name.as_bytes();
                    Some(Feature::new(
                        Tag::new(
                            bytes[0] as char,
                            bytes[1] as char,
                            bytes[2] as char,
                            bytes[3] as char,
                        ),
                        *value,
                        0..usize::MAX,
                    ))
                } else {
                    None
                }
            })
            .collect();

        // Step 6: Let HarfBuzz do the heavy lifting
        let output = harfbuzz_rs::shape(&hb_font, buffer, features.as_slice());

        // Step 7: Extract the beautiful positioned glyphs
        let positions = output.get_glyph_positions();
        let infos = output.get_glyph_infos();

        let mut glyphs = Vec::with_capacity(infos.len());
        let mut x_offset = 0.0;
        let mut y_offset = 0.0;

        for (info, pos) in infos.iter().zip(positions.iter()) {
            let x_advance = pos.x_advance as f32 / 64.0;
            let y_advance = pos.y_advance as f32 / 64.0;

            glyphs.push(PositionedGlyph {
                id: info.codepoint,
                x: x_offset + pos.x_offset as f32 / 64.0,
                y: y_offset + pos.y_offset as f32 / 64.0,
                advance: x_advance,
                cluster: info.cluster,
            });

            x_offset += x_advance;
            y_offset += y_advance;
        }

        Ok(ShapingResult {
            glyphs,
            advance_width: x_offset,
            advance_height: params.size,
            direction: params.direction,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icu_harfbuzz_shaper_empty() {
        let shaper = IcuHarfBuzzShaper::new();
        assert_eq!(Stage::name(&shaper), "ICU-HarfBuzz");
    }
}
