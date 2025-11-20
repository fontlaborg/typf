//! When you just need characters laid out: the simplest possible shaper
//!
//! This is our "just the basics" shaper. No ligatures, no kerning, no complex
//! script support. Just takes your text, finds the matching glyphs, and lays
//! them out left-to-right. Perfect for ASCII, debugging, or when you don't
//! want the complexity of HarfBuzz.

use std::sync::Arc;
use typf_core::{
    error::Result,
    traits::{FontRef, Shaper},
    types::{PositionedGlyph, ShapingResult},
    ShapingParams,
};

/// The simplest shaper: one character = one glyph, laid out left-to-right
pub struct NoneShaper;

impl NoneShaper {
    /// Creates a new shaper that does the absolute minimum
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoneShaper {
    fn default() -> Self {
        Self::new()
    }
}

impl Shaper for NoneShaper {
    fn name(&self) -> &'static str {
        "none"
    }

    fn shape(
        &self,
        text: &str,
        font: Arc<dyn FontRef>,
        params: &ShapingParams,
    ) -> Result<ShapingResult> {
        log::debug!("NoneShaper: Shaping {} chars", text.chars().count());

        let mut glyphs = Vec::new();
        let mut x_advance = 0.0;
        let scale = params.size / font.units_per_em() as f32;

        // One character becomes one glyph, positioned sequentially
        for (cluster, ch) in text.char_indices() {
            // Find which glyph draws this character
            let glyph_id = font.glyph_id(ch).unwrap_or(0); // Use .notdef (0) if not found

            // Get the glyph's width and scale it to our display size
            let advance_unscaled = font.advance_width(glyph_id);
            let advance = advance_unscaled * scale + params.letter_spacing;

            // Place the glyph at the current position
            glyphs.push(PositionedGlyph {
                id: glyph_id,
                x: x_advance,
                y: 0.0,
                advance,
                cluster: cluster as u32,
            });

            // Move to the right for the next glyph
            x_advance += advance;
        }

        Ok(ShapingResult {
            glyphs,
            advance_width: x_advance,
            advance_height: params.size,
            direction: params.direction,
        })
    }

    fn supports_script(&self, _script: &str) -> bool {
        // We're honest about our limitations
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typf_core::types::Direction;

    // Mock font for testing
    struct MockFont;

    impl FontRef for MockFont {
        fn data(&self) -> &[u8] {
            &[]
        }

        fn units_per_em(&self) -> u16 {
            1000
        }

        fn glyph_id(&self, ch: char) -> Option<u32> {
            // Simple mapping: ASCII characters to their values
            if ch.is_ascii() {
                Some(ch as u32)
            } else {
                None
            }
        }

        fn advance_width(&self, _glyph_id: u32) -> f32 {
            500.0 // Fixed advance for simplicity
        }
    }

    #[test]
    fn test_basic_shaping() {
        let shaper = NoneShaper::new();
        let font = Arc::new(MockFont);
        let params = ShapingParams {
            size: 16.0,
            ..Default::default()
        };

        let result = shaper.shape("Hello", font, &params).unwrap();

        assert_eq!(result.glyphs.len(), 5);
        assert_eq!(result.direction, Direction::LeftToRight);

        // Check that glyphs are positioned sequentially
        for i in 1..result.glyphs.len() {
            assert!(result.glyphs[i].x > result.glyphs[i - 1].x);
        }
    }

    #[test]
    fn test_empty_text() {
        let shaper = NoneShaper::new();
        let font = Arc::new(MockFont);
        let params = ShapingParams::default();

        let result = shaper.shape("", font, &params).unwrap();

        assert_eq!(result.glyphs.len(), 0);
        assert_eq!(result.advance_width, 0.0);
    }
}
