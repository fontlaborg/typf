#![no_main]

use libfuzzer_sys::fuzz_target;
use std::sync::Arc;
use typf_core::{ShapingParams, traits::{FontRef, Shaper}};
use typf_shape_hb::HarfBuzzShaper;

// Mock font for fuzzing
struct MockFont {
    data: Vec<u8>,
}

impl FontRef for MockFont {
    fn data(&self) -> &[u8] {
        &self.data
    }

    fn glyph_count(&self) -> usize {
        100 // Arbitrary
    }

    fn units_per_em(&self) -> u16 {
        1000
    }
}

fuzz_target!(|data: &[u8]| {
    // Convert fuzzer input to UTF-8 string (lossy)
    let text = String::from_utf8_lossy(data);

    // Skip empty or very large inputs
    if text.is_empty() || text.len() > 1_000 {
        return;
    }

    let shaper = HarfBuzzShaper::new();
    let font = Arc::new(MockFont { data: vec![] });

    // Fuzz with default params
    let params = ShapingParams::default();
    let _ = shaper.shape(&text, font.clone(), &params);

    // Fuzz with RTL
    let params_rtl = ShapingParams {
        direction: typf_core::types::Direction::RightToLeft,
        ..Default::default()
    };
    let _ = shaper.shape(&text, font.clone(), &params_rtl);
});
