#![no_main]

use libfuzzer_sys::fuzz_target;
use typf_unicode::{UnicodeOptions, UnicodeProcessor};

fuzz_target!(|data: &[u8]| {
    // Convert fuzzer input to UTF-8 string (lossy)
    let text = String::from_utf8_lossy(data);

    // Skip empty or very large inputs
    if text.is_empty() || text.len() > 10_000 {
        return;
    }

    let processor = UnicodeProcessor::new();

    // Fuzz with normalization enabled
    let options = UnicodeOptions {
        normalize: true,
        detect_scripts: true,
        bidi_resolve: true,
        language: None,
    };

    // Process should not panic
    let _ = processor.process(&text, &options);

    // Fuzz with normalization disabled
    let options_no_norm = UnicodeOptions {
        normalize: false,
        detect_scripts: true,
        bidi_resolve: true,
        language: None,
    };

    let _ = processor.process(&text, &options_no_norm);
});
