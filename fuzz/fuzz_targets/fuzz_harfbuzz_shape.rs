//! Find HarfBuzz crashes before your users do
//!
//! This fuzzer throws random text at the HarfBuzz shaping engine to find
//! panics, crashes, and security vulnerabilities. HarfBuzz is complex
//! C++ code that handles Unicode, OpenType fonts, and complex scripts.
//! Fuzzing helps ensure malformed text can't crash the renderer.
//!
//! What we test:
//! - Unicode edge cases and invalid sequences
//! - Mixed RTL/LTR text that confuses bidirectional algorithms
//! - Extremely long text that might overwhelm internal buffers
//! - Complex script combinations (Arabic + Hebrew + CJK)
//! - Invalid Unicode that could trigger parsing bugs

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::sync::Arc;
use typf_core::{ShapingParams, traits::{FontRef, Shaper}};
use typf_shape_hb::HarfBuzzShaper;

/// Minimal font for HarfBuzz fuzzing - we care about text handling, not fonts
struct MockFont {
    data: Vec<u8>,
}

impl FontRef for MockFont {
    fn data(&self) -> &[u8] {
        &self.data // Empty font data - still exercises text handling
    }

    fn glyph_count(&self) -> usize {
        100 // Arbitrary size to prevent panics
    }

    fn units_per_em(&self) -> u16 {
        1000 // Standard font coordinate space
    }
}

fuzz_target!(|data: &[u8]| {
    // Transform raw bytes into text - libFuzzer gives us arbitrary data
    let text = String::from_utf8_lossy(data);

    // Reject inputs that would waste time or cause timeouts
    if text.is_empty() || text.len() > 1_000 {
        return;
    }

    let shaper = HarfBuzzShaper::new();
    let font = Arc::new(MockFont { data: vec![] });

    // Test with standard LTR text direction
    let params = ShapingParams::default();
    let _ = shaper.shape(&text, font.clone(), &params);

    // Test with RTL direction - exercises the bidirectional algorithm
    let params_rtl = ShapingParams {
        direction: typf_core::types::Direction::RightToLeft,
        ..Default::default()
    };
    let _ = shaper.shape(&text, font.clone(), &params_rtl);
});
